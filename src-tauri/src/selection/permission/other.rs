/// 非 macOS 平台目前不需要辅助功能权限——所有平台相关的全局输入
/// 监听由 `rdev` 直接处理，系统不要求额外授权。
pub fn accessibility_permission_granted() -> bool {
    true
}
