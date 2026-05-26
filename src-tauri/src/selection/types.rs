use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionCandidate {
    pub text: String,
    pub captured_at: Instant,
}
