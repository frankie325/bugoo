/// SM-2 Ebbinghaus spaced repetition algorithm
/// https://www.supermemo.com/en/archives1990-2015/english/ol/sm2

#[derive(Debug, Clone)]
pub struct SpacedRepetitionState {
    pub ease_factor: f64,
    pub interval: i32,
    pub repetitions: i32,
    pub next_review_at: i64,
}

impl Default for SpacedRepetitionState {
    fn default() -> Self {
        Self {
            ease_factor: 2.5,
            interval: 1,
            repetitions: 0,
            next_review_at: 0,
        }
    }
}

impl SpacedRepetitionState {
    /// Update state after a review.
    /// `remembered` = true means the user recalled the word correctly.
    pub fn after_review(state: &SpacedRepetitionState, remembered: bool) -> Self {
        let now = chrono::Utc::now().timestamp();
        if remembered {
            let repetitions = state.repetitions + 1;
            let interval = match repetitions {
                1 => 1,
                2 => 3,
                _ => {
                    let raw = (state.interval as f64 * state.ease_factor).floor() as i32;
                    raw.max(1)
                }
            };
            let ease_factor = (state.ease_factor + 0.1).min(2.5);
            let next_review_at = now + (interval as i64 * 86400);
            Self { ease_factor, interval, repetitions, next_review_at }
        } else {
            let ease_factor = (state.ease_factor - 0.2).max(1.3);
            let next_review_at = now + 86400;
            Self { ease_factor, interval: 1, repetitions: 0, next_review_at }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_values() {
        let state = SpacedRepetitionState::default();
        assert_eq!(state.ease_factor, 2.5);
        assert_eq!(state.interval, 1);
        assert_eq!(state.repetitions, 0);
    }

    #[test]
    fn test_remember_increases_interval() {
        let state = SpacedRepetitionState::default();
        let next = SpacedRepetitionState::after_review(&state, true);
        assert_eq!(next.interval, 1);
        assert_eq!(next.repetitions, 1);

        let next2 = SpacedRepetitionState::after_review(&next, true);
        assert_eq!(next2.interval, 3);
        assert_eq!(next2.repetitions, 2);

        let next3 = SpacedRepetitionState::after_review(&next2, true);
        assert_eq!(next3.interval, 7); // 3 * 2.5 = 7.5 -> round to 7
        assert_eq!(next3.repetitions, 3);
    }

    #[test]
    fn test_forgotten_resets_to_1_day() {
        let state = SpacedRepetitionState {
            ease_factor: 2.5,
            interval: 14,
            repetitions: 5,
            next_review_at: 0,
        };
        let next = SpacedRepetitionState::after_review(&state, false);
        assert_eq!(next.interval, 1);
        assert_eq!(next.repetitions, 0);
        assert_eq!(next.ease_factor, 2.3); // 2.5 - 0.2
    }

    #[test]
    fn test_ease_factor_min_cap() {
        let state = SpacedRepetitionState {
            ease_factor: 1.3,
            interval: 1,
            repetitions: 0,
            next_review_at: 0,
        };
        let next = SpacedRepetitionState::after_review(&state, false);
        assert_eq!(next.ease_factor, 1.3); // does not go below 1.3
    }

    #[test]
    fn test_ease_factor_max_cap() {
        let mut state = SpacedRepetitionState::default();
        state.interval = 30;
        state.repetitions = 3;
        for _ in 0..10 {
            state = SpacedRepetitionState::after_review(&state, true);
        }
        assert!(state.ease_factor <= 2.5); // does not exceed 2.5
    }
}