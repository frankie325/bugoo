use core_graphics::display::CGDisplay;
use core_graphics::event::{CGEventField, CGEventType, EventField};
use core_graphics::geometry::CGPoint;
use std::os::raw::c_void;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

use crate::selection::mouse_event::{MousePosition, SelectionMouseEvent, SelectionMouseEventKind};

type CFMachPortRef = *const c_void;
type CFAllocatorRef = *const c_void;
type CFRunLoopRef = *const c_void;
type CFRunLoopSourceRef = *const c_void;
type CFRunLoopMode = *const c_void;
type SelectionMouseCallback = Box<dyn FnMut(SelectionMouseEvent) + Send>;

const LEFT_DOWN_EVENT_MASK: u64 = 1 << (CGEventType::LeftMouseDown as u64);
const LEFT_DRAGGED_EVENT_MASK: u64 = 1 << (CGEventType::LeftMouseDragged as u64);
const LEFT_UP_EVENT_MASK: u64 = 1 << (CGEventType::LeftMouseUp as u64);
const MOUSE_EVENT_MASK: u64 = LEFT_DOWN_EVENT_MASK | LEFT_DRAGGED_EVENT_MASK | LEFT_UP_EVENT_MASK;
const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const K_CG_EVENT_TAP_OPTION_LISTEN_ONLY: u32 = 1;
const DISPLAY_CACHE_TTL: Duration = Duration::from_secs(5);

static GLOBAL_CALLBACK: Mutex<Option<SelectionMouseCallback>> = Mutex::new(None);
static DISPLAY_CACHE: Mutex<Option<DisplayCoordinateCache>> = Mutex::new(None);

#[link(name = "Cocoa", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: TapCallback,
        user_info: *mut c_void,
    ) -> CFMachPortRef;
    fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef,
        tap: CFMachPortRef,
        order: isize,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode);
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CFRunLoopRun();
    fn CGEventGetLocation(event: *const c_void) -> CGPoint;
    fn CGEventGetIntegerValueField(event: *const c_void, field: CGEventField) -> i64;

    static kCFRunLoopCommonModes: CFRunLoopMode;
}

type TapCallback = unsafe extern "C" fn(
    proxy: *const c_void,
    event_type: CGEventType,
    cg_event: *const c_void,
    user_info: *mut c_void,
) -> *const c_void;

unsafe extern "C" fn tap_callback(
    _proxy: *const c_void,
    event_type: CGEventType,
    cg_event: *const c_void,
    _user_info: *mut c_void,
) -> *const c_void {
    let position = mouse_position_from_cg_event(cg_event);
    let click_count = mouse_click_count_from_cg_event(event_type, cg_event);
    if let Some(event) = map_mouse_event_type(event_type, position, click_count) {
        if let Ok(mut callback_slot) = GLOBAL_CALLBACK.lock() {
            if let Some(callback) = callback_slot.as_mut() {
                callback(event);
            }
        }
    }

    cg_event
}

pub(crate) fn map_mouse_event_type(
    event_type: CGEventType,
    position: MousePosition,
    click_count: Option<u8>,
) -> Option<SelectionMouseEvent> {
    let kind = match event_type {
        CGEventType::LeftMouseDown => Some(SelectionMouseEventKind::LeftDown),
        CGEventType::LeftMouseDragged => Some(SelectionMouseEventKind::LeftDragged),
        CGEventType::LeftMouseUp => Some(SelectionMouseEventKind::LeftUp { click_count }),
        _ => None,
    }?;

    Some(build_mouse_event(kind, position))
}

pub fn listen_mouse_events<T>(callback: T) -> Result<(), String>
where
    T: FnMut(SelectionMouseEvent) + Send + 'static,
{
    unsafe {
        if let Ok(mut callback_slot) = GLOBAL_CALLBACK.lock() {
            *callback_slot = Some(Box::new(callback));
        } else {
            return Err("failed to acquire callback lock".to_string());
        }

        let tap = CGEventTapCreate(
            core_graphics::event::CGEventTapLocation::HID as u32,
            K_CG_HEAD_INSERT_EVENT_TAP,
            K_CG_EVENT_TAP_OPTION_LISTEN_ONLY,
            MOUSE_EVENT_MASK,
            tap_callback,
            std::ptr::null_mut(),
        );
        if tap.is_null() {
            clear_global_callback();
            return Err("failed to create CGEventTap".to_string());
        }

        let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
        if source.is_null() {
            clear_global_callback();
            return Err("failed to create run loop source from CGEventTap".to_string());
        }

        let run_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);
        CFRunLoopRun();
        clear_global_callback();
    }

    Ok(())
}

fn clear_global_callback() {
    if let Ok(mut callback_slot) = GLOBAL_CALLBACK.lock() {
        *callback_slot = None;
    }
}

fn build_mouse_event(
    kind: SelectionMouseEventKind,
    position: MousePosition,
) -> SelectionMouseEvent {
    SelectionMouseEvent::new(kind, Some(position), SystemTime::now())
}

unsafe fn mouse_position_from_cg_event(cg_event: *const c_void) -> MousePosition {
    let point = CGEventGetLocation(cg_event);
    let position = MousePosition {
        x: point.x,
        y: point.y,
    };
    let displays = active_display_coordinate_spaces();
    macos_event_position_to_tauri_physical(position, &displays, main_display_scale_factor())
}

#[derive(Debug, Clone, Copy)]
struct DisplayCoordinateSpace {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
    scale_factor: f64,
}

#[derive(Debug, Clone)]
struct DisplayCoordinateCache {
    queried_at: Instant,
    displays: Vec<DisplayCoordinateSpace>,
}

fn macos_event_position_to_tauri_physical(
    position: MousePosition,
    displays: &[DisplayCoordinateSpace],
    fallback_scale_factor: f64,
) -> MousePosition {
    let scale_factor = displays
        .iter()
        .find(|display| display.contains(position))
        .map(|display| display.scale_factor)
        .unwrap_or(fallback_scale_factor);

    MousePosition {
        x: position.x * scale_factor,
        y: position.y * scale_factor,
    }
}

impl DisplayCoordinateSpace {
    fn contains(self, position: MousePosition) -> bool {
        position.x >= self.min_x
            && position.x < self.max_x
            && position.y >= self.min_y
            && position.y < self.max_y
    }
}

fn active_display_coordinate_spaces() -> Vec<DisplayCoordinateSpace> {
    if let Ok(mut cache) = DISPLAY_CACHE.lock() {
        if let Some(cached) = cache.as_ref() {
            if cached.queried_at.elapsed() < DISPLAY_CACHE_TTL {
                return cached.displays.clone();
            }
        }

        let displays = query_active_display_coordinate_spaces();
        *cache = Some(DisplayCoordinateCache {
            queried_at: Instant::now(),
            displays: displays.clone(),
        });
        return displays;
    }

    query_active_display_coordinate_spaces()
}

fn query_active_display_coordinate_spaces() -> Vec<DisplayCoordinateSpace> {
    CGDisplay::active_displays()
        .map(|display_ids| {
            display_ids
                .into_iter()
                .map(|display_id| display_coordinate_space(CGDisplay::new(display_id)))
                .collect()
        })
        .unwrap_or_else(|error| {
            log::warn!("Failed to query active displays for mouse event coordinates: {error:?}");
            vec![display_coordinate_space(CGDisplay::main())]
        })
}

fn display_coordinate_space(display: CGDisplay) -> DisplayCoordinateSpace {
    let bounds = display.bounds();
    let scale_factor = display_scale_factor(&display, bounds.size.width);

    DisplayCoordinateSpace {
        min_x: bounds.origin.x,
        min_y: bounds.origin.y,
        max_x: bounds.origin.x + bounds.size.width,
        max_y: bounds.origin.y + bounds.size.height,
        scale_factor,
    }
}

fn main_display_scale_factor() -> f64 {
    let main_display = CGDisplay::main();
    let bounds = main_display.bounds();
    display_scale_factor(&main_display, bounds.size.width)
}

fn display_scale_factor(display: &CGDisplay, logical_width: f64) -> f64 {
    if let Some(mode) = display.display_mode() {
        let width = mode.width();
        let pixel_width = mode.pixel_width();
        if width > 0 && pixel_width > 0 {
            return pixel_width as f64 / width as f64;
        }
    }

    let pixel_width = display.pixels_wide();
    if logical_width > 0.0 && pixel_width > 0 {
        pixel_width as f64 / logical_width
    } else {
        1.0
    }
}

unsafe fn mouse_click_count_from_cg_event(
    event_type: CGEventType,
    cg_event: *const c_void,
) -> Option<u8> {
    if !matches!(event_type, CGEventType::LeftMouseUp) {
        return None;
    }

    let value = CGEventGetIntegerValueField(cg_event, EventField::MOUSE_EVENT_CLICK_STATE);
    u8::try_from(value).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_left_down_with_position() {
        let event = map_mouse_event_type(CGEventType::LeftMouseDown, pos(10.0, 20.0), None)
            .expect("left down should map");
        assert_eq!(event.kind, SelectionMouseEventKind::LeftDown);
        assert_eq!(event.position, Some(pos(10.0, 20.0)));
    }

    #[test]
    fn maps_left_dragged_with_position() {
        let event = map_mouse_event_type(CGEventType::LeftMouseDragged, pos(12.0, 22.0), None)
            .expect("left dragged should map");
        assert_eq!(event.kind, SelectionMouseEventKind::LeftDragged);
        assert_eq!(event.position, Some(pos(12.0, 22.0)));
    }

    #[test]
    fn maps_left_up_with_click_count() {
        let event = map_mouse_event_type(CGEventType::LeftMouseUp, pos(15.0, 25.0), Some(2))
            .expect("left up should map");
        assert_eq!(
            event.kind,
            SelectionMouseEventKind::LeftUp {
                click_count: Some(2)
            }
        );
        assert_eq!(event.position, Some(pos(15.0, 25.0)));
    }

    #[test]
    fn ignores_non_left_mouse_events() {
        assert_eq!(
            map_mouse_event_type(CGEventType::MouseMoved, pos(0.0, 0.0), None),
            None
        );
        assert_eq!(
            map_mouse_event_type(CGEventType::KeyDown, pos(0.0, 0.0), None),
            None
        );
    }

    #[test]
    fn converts_macos_event_position_to_tauri_physical_coordinates_without_y_flip() {
        let displays = [DisplayCoordinateSpace {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1512.0,
            max_y: 982.0,
            scale_factor: 2.0,
        }];

        let position = macos_event_position_to_tauri_physical(pos(100.0, 200.0), &displays, 1.0);

        assert_eq!(position, pos(200.0, 400.0));
    }

    #[test]
    fn converts_macos_event_position_using_matching_display_scale() {
        let displays = [
            DisplayCoordinateSpace {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 1512.0,
                max_y: 982.0,
                scale_factor: 2.0,
            },
            DisplayCoordinateSpace {
                min_x: 1512.0,
                min_y: 0.0,
                max_x: 3432.0,
                max_y: 1080.0,
                scale_factor: 1.0,
            },
        ];

        let position = macos_event_position_to_tauri_physical(pos(1600.0, 300.0), &displays, 2.0);

        assert_eq!(position, pos(1600.0, 300.0));
    }

    fn pos(x: f64, y: f64) -> MousePosition {
        MousePosition { x, y }
    }
}

#[cfg(test)]
mod event_build_tests {
    use super::build_mouse_event;
    use crate::selection::mouse_event::{MousePosition, SelectionMouseEventKind};

    #[test]
    fn builds_event_for_left_release() {
        let position = MousePosition { x: 1.0, y: 2.0 };
        let event = build_mouse_event(
            SelectionMouseEventKind::LeftUp {
                click_count: Some(1),
            },
            position,
        );
        assert_eq!(
            event.kind,
            SelectionMouseEventKind::LeftUp {
                click_count: Some(1)
            }
        );
        assert_eq!(event.position, Some(position));
    }
}
