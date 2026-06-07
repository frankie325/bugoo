/// macOS 平台下检测是否已授予辅助功能权限。
///
/// 通过 `accessibility-sys` 绑定到 `AXIsProcessTrusted`，
/// 该 API 是系统级的，调用方进程必须是被询问对象本身。
pub fn accessibility_permission_granted() -> bool {
    unsafe { accessibility_sys::AXIsProcessTrusted() }
}
