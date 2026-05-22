use super::types::SelectionReadError;

pub trait PlatformSelectionReader {
    fn read_selected_text(&self) -> Result<Option<String>, SelectionReadError>;
}

pub trait ClipboardSelectionReader {
    fn read_selected_text_via_copy(&self) -> Result<Option<String>, SelectionReadError>;
}

pub fn read_selected_text<P, C>(
    platform: &P,
    clipboard: &C,
) -> Result<Option<String>, SelectionReadError>
where
    P: PlatformSelectionReader,
    C: ClipboardSelectionReader,
{
    #[cfg(target_os = "macos")]
    match platform.read_selected_text() {
        Ok(Some(text)) if !text.trim().is_empty() => Ok(Some(text)),
        Ok(_) => clipboard.read_selected_text_via_copy(),
        Err(SelectionReadError::PermissionDenied(message)) => {
            Err(SelectionReadError::PermissionDenied(message))
        }
        Err(error) => {
            log::info!(
                "macOS accessibility selection read failed, using clipboard fallback: {error}"
            );
            clipboard.read_selected_text_via_copy()
        }
    }

    #[cfg(not(target_os = "macos"))]
    match platform.read_selected_text() {
        Ok(Some(text)) if !text.trim().is_empty() => Ok(Some(text)),
        Ok(_) => clipboard.read_selected_text_via_copy(),
        Err(SelectionReadError::PermissionDenied(message)) => {
            Err(SelectionReadError::PermissionDenied(message))
        }
        Err(error) => {
            log::debug!("Platform selection read failed, using clipboard fallback: {error}");
            clipboard.read_selected_text_via_copy()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct FakePlatformReader {
        result: Result<Option<String>, SelectionReadError>,
    }

    impl PlatformSelectionReader for FakePlatformReader {
        fn read_selected_text(&self) -> Result<Option<String>, SelectionReadError> {
            self.result.clone()
        }
    }

    #[derive(Debug)]
    struct FakeClipboardReader {
        result: Result<Option<String>, SelectionReadError>,
    }

    impl ClipboardSelectionReader for FakeClipboardReader {
        fn read_selected_text_via_copy(&self) -> Result<Option<String>, SelectionReadError> {
            self.result.clone()
        }
    }

    #[test]
    fn uses_platform_text_without_clipboard_fallback() {
        let platform = FakePlatformReader {
            result: Ok(Some("hello".to_string())),
        };
        let clipboard = FakeClipboardReader {
            result: Ok(Some("clipboard".to_string())),
        };

        let result = read_selected_text(&platform, &clipboard);

        assert_eq!(result, Ok(Some("hello".to_string())));
    }

    #[test]
    fn uses_clipboard_when_platform_returns_empty() {
        let platform = FakePlatformReader { result: Ok(None) };
        let clipboard = FakeClipboardReader {
            result: Ok(Some("fallback".to_string())),
        };

        let result = read_selected_text(&platform, &clipboard);

        #[cfg(target_os = "macos")]
        assert_eq!(result, Ok(Some("fallback".to_string())));
        #[cfg(not(target_os = "macos"))]
        assert_eq!(result, Ok(Some("fallback".to_string())));
    }

    #[test]
    fn uses_clipboard_when_platform_read_fails() {
        let platform = FakePlatformReader {
            result: Err(SelectionReadError::PlatformReadFailed(
                "text pattern unavailable".to_string(),
            )),
        };
        let clipboard = FakeClipboardReader {
            result: Ok(Some("fallback".to_string())),
        };

        let result = read_selected_text(&platform, &clipboard);

        #[cfg(target_os = "macos")]
        assert_eq!(result, Ok(Some("fallback".to_string())));
        #[cfg(not(target_os = "macos"))]
        assert_eq!(result, Ok(Some("fallback".to_string())));
    }

    #[test]
    fn returns_permission_error_without_clipboard_attempt() {
        let platform = FakePlatformReader {
            result: Err(SelectionReadError::PermissionDenied(
                "accessibility permission required".to_string(),
            )),
        };
        let clipboard = FakeClipboardReader {
            result: Ok(Some("fallback".to_string())),
        };

        let result = read_selected_text(&platform, &clipboard);

        assert_eq!(
            result,
            Err(SelectionReadError::PermissionDenied(
                "accessibility permission required".to_string(),
            )),
        );
    }
}
