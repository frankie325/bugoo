use std::fmt;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionCandidate {
    pub text: String,
    pub captured_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionReadError {
    PermissionDenied(String),
    PlatformUnavailable(String),
    PlatformReadFailed(String),
    ClipboardReadFailed(String),
    ClipboardWriteFailed(String),
    SimulateCopyFailed(String),
}

impl fmt::Display for SelectionReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectionReadError::PermissionDenied(message)
            | SelectionReadError::PlatformUnavailable(message)
            | SelectionReadError::PlatformReadFailed(message)
            | SelectionReadError::ClipboardReadFailed(message)
            | SelectionReadError::ClipboardWriteFailed(message)
            | SelectionReadError::SimulateCopyFailed(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for SelectionReadError {}
