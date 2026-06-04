//! 平台特定的辅助功能权限检查。实际平台实现位于子模块，
//! 通过 `mod.rs` 的 `#[cfg(target_os)]` 守卫按平台暴露。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessibilityPermission {
    Granted,
    Missing,
}

pub fn accessibility_permission() -> AccessibilityPermission {
    if accessibility_permission_granted() {
        AccessibilityPermission::Granted
    } else {
        AccessibilityPermission::Missing
    }
}

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
mod other;

#[cfg(target_os = "macos")]
use macos::accessibility_permission_granted;
#[cfg(not(target_os = "macos"))]
use other::accessibility_permission_granted;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_shape_is_stable() {
        let permission = accessibility_permission();
        assert!(matches!(
            permission,
            AccessibilityPermission::Granted | AccessibilityPermission::Missing
        ));
    }
}
