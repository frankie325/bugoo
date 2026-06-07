use tauri::PhysicalPosition;

/// macOS 的坐标系原点在左下角，需要把屏幕物理坐标翻转为 Cocoa 的 top-left 坐标。
pub(crate) fn macos_top_left_point_from_physical_position(
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
    fn converts_physical_position_to_macos_top_left_point() {
        let point = macos_top_left_point_from_physical_position(
            PhysicalPosition::new(400, 300),
            2.0,
            1800.0,
        );

        assert_eq!(point, (200.0, 1650.0));
    }
}
