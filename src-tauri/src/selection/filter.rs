use std::time::{Duration, Instant};

use super::types::SelectionCandidate;

const MAX_SELECTION_CHARS: usize = 50;
const TRIGGER_THROTTLE: Duration = Duration::from_millis(500);
const DUPLICATE_TEXT_SUPPRESS: Duration = Duration::from_secs(2);

#[derive(Debug, Default)]
pub struct SelectionFilter {
    last_text: Option<String>,
    last_triggered_at: Option<Instant>,
}

impl SelectionFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn accept(&mut self, raw_text: &str, captured_at: Instant) -> Option<SelectionCandidate> {
        let text = raw_text.trim();

        if text.is_empty() || text.chars().count() > MAX_SELECTION_CHARS {
            return None;
        }

        if let Some(last_triggered_at) = self.last_triggered_at {
            if captured_at.duration_since(last_triggered_at) < TRIGGER_THROTTLE {
                return None;
            }
        }

        if self.last_text.as_deref() == Some(text) {
            if let Some(last_triggered_at) = self.last_triggered_at {
                if captured_at.duration_since(last_triggered_at) < DUPLICATE_TEXT_SUPPRESS {
                    return None;
                }
            }
        }

        let text = text.to_string();
        self.last_text = Some(text.clone());
        self.last_triggered_at = Some(captured_at);

        Some(SelectionCandidate { text, captured_at })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_text_after_trim() {
        let mut state = SelectionFilter::new();
        let result = state.accept("   \n\t  ", Instant::now());
        assert_eq!(result, None);
    }

    #[test]
    fn rejects_text_longer_than_50_chars() {
        let mut state = SelectionFilter::new();
        let text = "a".repeat(51);
        let result = state.accept(&text, Instant::now());
        assert_eq!(result, None);
    }

    #[test]
    fn accepts_trimmed_text_within_limit() {
        let mut state = SelectionFilter::new();
        let now = Instant::now();
        let result = state.accept("  hello  ", now);
        assert_eq!(
            result,
            Some(SelectionCandidate {
                text: "hello".to_string(),
                captured_at: now,
            }),
        );
    }

    #[test]
    fn throttles_triggers_inside_500ms_window() {
        let mut state = SelectionFilter::new();
        let start = Instant::now();
        assert!(state.accept("hello", start).is_some());
        let result = state.accept("world", start + Duration::from_millis(499));
        assert_eq!(result, None);
    }

    #[test]
    fn allows_new_text_after_throttle_window() {
        let mut state = SelectionFilter::new();
        let start = Instant::now();
        assert!(state.accept("hello", start).is_some());
        let result = state.accept("world", start + Duration::from_millis(500));
        assert_eq!(
            result,
            Some(SelectionCandidate {
                text: "world".to_string(),
                captured_at: start + Duration::from_millis(500),
            }),
        );
    }

    #[test]
    fn suppresses_duplicate_text_after_throttle_window() {
        let mut state = SelectionFilter::new();
        let start = Instant::now();
        assert!(state.accept("hello", start).is_some());
        let result = state.accept("hello", start + Duration::from_millis(800));
        assert_eq!(result, None);
    }

    #[test]
    fn allows_duplicate_text_after_duplicate_suppress_window() {
        let mut state = SelectionFilter::new();
        let start = Instant::now();
        assert!(state.accept("hello", start).is_some());
        let result = state.accept("hello", start + Duration::from_secs(3));
        assert_eq!(
            result,
            Some(SelectionCandidate {
                text: "hello".to_string(),
                captured_at: start + Duration::from_secs(3),
            }),
        );
    }
}
