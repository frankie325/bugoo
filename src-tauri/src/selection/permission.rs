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
pub fn accessibility_permission_granted() -> bool {
    unsafe { accessibility_sys::AXIsProcessTrusted() }
}

#[cfg(not(target_os = "macos"))]
pub fn accessibility_permission_granted() -> bool {
    true
}

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
