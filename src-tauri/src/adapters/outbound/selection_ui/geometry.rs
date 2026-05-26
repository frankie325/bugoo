use tauri::{PhysicalPosition, PhysicalSize};

pub(super) const POPUP_OFFSET_PX: i32 = 12;

pub(super) fn is_cursor_inside_window_bounds(
    cursor: PhysicalPosition<f64>,
    window_position: PhysicalPosition<i32>,
    window_size: PhysicalSize<u32>,
) -> bool {
    let cursor_x = cursor.x.round() as i32;
    let cursor_y = cursor.y.round() as i32;
    let left = window_position.x;
    let top = window_position.y;
    let right = left + window_size.width as i32;
    let bottom = top + window_size.height as i32;

    cursor_x >= left && cursor_x < right && cursor_y >= top && cursor_y < bottom
}

pub(super) fn is_position_inside_monitor(
    position: PhysicalPosition<f64>,
    monitor_position: PhysicalPosition<i32>,
    monitor_size: PhysicalSize<u32>,
) -> bool {
    let x = position.x.round() as i32;
    let y = position.y.round() as i32;
    let left = monitor_position.x;
    let top = monitor_position.y;
    let right = left + monitor_size.width as i32;
    let bottom = top + monitor_size.height as i32;

    x >= left && x < right && y >= top && y < bottom
}

pub(super) fn calculate_popup_position(
    anchor: PhysicalPosition<f64>,
    popup_size: PhysicalSize<u32>,
    monitor_position: PhysicalPosition<i32>,
    monitor_size: PhysicalSize<u32>,
    offset: i32,
) -> PhysicalPosition<i32> {
    let anchor_x = anchor.x.round() as i32;
    let anchor_y = anchor.y.round() as i32;
    let popup_width = popup_size.width as i32;
    let popup_height = popup_size.height as i32;

    let monitor_left = monitor_position.x;
    let monitor_top = monitor_position.y;
    let monitor_right = monitor_position.x + monitor_size.width as i32;
    let monitor_bottom = monitor_position.y + monitor_size.height as i32;

    let mut x = anchor_x + offset;
    let mut y = anchor_y + offset;

    if x + popup_width > monitor_right {
        x = anchor_x - popup_width - offset;
    }
    if y + popup_height > monitor_bottom {
        y = anchor_y - popup_height - offset;
    }

    let max_x = monitor_right - popup_width;
    let max_y = monitor_bottom - popup_height;

    if max_x >= monitor_left {
        x = x.clamp(monitor_left, max_x);
    } else {
        x = monitor_left;
    }

    if max_y >= monitor_top {
        y = y.clamp(monitor_top, max_y);
    } else {
        y = monitor_top;
    }

    PhysicalPosition::new(x, y)
}

#[cfg(target_os = "macos")]
pub(super) fn macos_top_left_point_from_physical_position(
    position: PhysicalPosition<i32>,
    scale_factor: f64,
    main_display_pixels_high: f64,
) -> (f64, f64) {
    let logical_x = position.x as f64 / scale_factor;
    let logical_y = position.y as f64 / scale_factor;
    (logical_x, main_display_pixels_high - logical_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_position_defaults_to_bottom_right_offset() {
        let position = calculate_popup_position(
            PhysicalPosition::new(100.0, 200.0),
            PhysicalSize::new(320, 140),
            PhysicalPosition::new(0, 0),
            PhysicalSize::new(1920, 1080),
            POPUP_OFFSET_PX,
        );

        assert_eq!(position, PhysicalPosition::new(112, 212));
    }

    #[test]
    fn popup_position_flips_horizontally_on_right_edge() {
        let position = calculate_popup_position(
            PhysicalPosition::new(1900.0, 200.0),
            PhysicalSize::new(320, 140),
            PhysicalPosition::new(0, 0),
            PhysicalSize::new(1920, 1080),
            POPUP_OFFSET_PX,
        );

        assert_eq!(position.x, 1568);
        assert_eq!(position.y, 212);
    }

    #[test]
    fn popup_position_flips_vertically_on_bottom_edge() {
        let position = calculate_popup_position(
            PhysicalPosition::new(100.0, 1070.0),
            PhysicalSize::new(320, 140),
            PhysicalPosition::new(0, 0),
            PhysicalSize::new(1920, 1080),
            POPUP_OFFSET_PX,
        );

        assert_eq!(position.x, 112);
        assert_eq!(position.y, 918);
    }

    #[test]
    fn popup_position_flips_on_both_edges() {
        let position = calculate_popup_position(
            PhysicalPosition::new(1918.0, 1078.0),
            PhysicalSize::new(320, 140),
            PhysicalPosition::new(0, 0),
            PhysicalSize::new(1920, 1080),
            POPUP_OFFSET_PX,
        );

        assert_eq!(position, PhysicalPosition::new(1586, 926));
    }

    #[test]
    fn detects_anchor_inside_matching_monitor_bounds() {
        assert!(is_position_inside_monitor(
            PhysicalPosition::new(2100.0, 300.0),
            PhysicalPosition::new(1920, 0),
            PhysicalSize::new(1920, 1080),
        ));
    }

    #[test]
    fn detects_anchor_outside_monitor_bounds() {
        assert!(!is_position_inside_monitor(
            PhysicalPosition::new(1800.0, 300.0),
            PhysicalPosition::new(1920, 0),
            PhysicalSize::new(1920, 1080),
        ));
    }

    #[test]
    fn detects_cursor_inside_window_bounds() {
        assert!(is_cursor_inside_window_bounds(
            PhysicalPosition::new(120.0, 230.0),
            PhysicalPosition::new(100, 200),
            PhysicalSize::new(80, 40),
        ));
    }

    #[test]
    fn detects_cursor_outside_window_bounds() {
        assert!(!is_cursor_inside_window_bounds(
            PhysicalPosition::new(180.0, 241.0),
            PhysicalPosition::new(100, 200),
            PhysicalSize::new(80, 40),
        ));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn converts_physical_position_to_macos_top_left_point() {
        let point = macos_top_left_point_from_physical_position(
            PhysicalPosition::new(400, 300),
            2.0,
            1800.0,
        );

        assert_eq!(point, (200.0, 1650.0));
    }
}
