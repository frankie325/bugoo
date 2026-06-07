use std::time::Instant;

use super::types::SelectionCandidate;

pub fn filter_selection_text(
    raw_text: &str,
    captured_at: Instant,
) -> Option<SelectionCandidate> {
    let text = raw_text.trim();
    if text.is_empty() {
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

    #[test]
    fn accepts_long_text_regardless_of_length() {
        let now = Instant::now();
        let text = "a".repeat(100_000);
        assert!(filter_selection_text(&text, now).is_some());
    }
}
