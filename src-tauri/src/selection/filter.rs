use std::time::Instant;

use super::types::SelectionCandidate;

const MAX_SELECTION_CHARS: usize = 50;
pub fn filter_selection_text(raw_text: &str, captured_at: Instant) -> Option<SelectionCandidate> {
    let text = raw_text.trim();
    if text.is_empty() || text.chars().nth(MAX_SELECTION_CHARS).is_some() {
        return None;
    }

    Some(SelectionCandidate {
        text: text.to_string(),
        captured_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_text_after_trim() {
        let now = Instant::now();
        assert_eq!(filter_selection_text("   \n\t  ", now), None);
    }

    #[test]
    fn rejects_text_longer_than_50_chars() {
        let now = Instant::now();
        let text = "a".repeat(51);
        assert_eq!(filter_selection_text(&text, now), None);
    }

    #[test]
    fn accepts_trimmed_text_within_limit() {
        let now = Instant::now();
        assert_eq!(
            filter_selection_text("  hello  ", now),
            Some(SelectionCandidate {
                text: "hello".to_string(),
                captured_at: now,
            }),
        );
    }

    #[test]
    fn accepts_same_text_repeatedly() {
        let now = Instant::now();
        assert!(filter_selection_text("hello", now).is_some());
        assert!(filter_selection_text("hello", now).is_some());
    }
}
