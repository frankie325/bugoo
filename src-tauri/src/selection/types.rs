use std::fmt;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionCandidate {
    pub text: String,
    pub captured_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionReadError {
    ReadFailed(String),
}

impl fmt::Display for SelectionReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectionReadError::ReadFailed(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for SelectionReadError {}
