use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CategoryStats {
    pub rounds_played: u64,
    pub rounds_won: u64,
    pub best_score: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Stats {
    pub rounds_played: u64,
    pub rounds_won: u64,
    pub rounds_lost: u64,
    pub current_streak: u32,
    pub best_streak: u32,
    pub total_score: u64,
    pub best_score: u32,
    pub categories: HashMap<String, CategoryStats>,
}

impl Stats {
    /// Wins / rounds played, or `0.0` before any round has been recorded.
    pub fn win_rate(&self) -> f64 {
        if self.rounds_played == 0 {
            0.0
        } else {
            self.rounds_won as f64 / self.rounds_played as f64
        }
    }

    /// Folds the outcome of one finished round into the running totals, both
    /// overall and per-category.
    pub fn record_round(&mut self, category_id: &str, won: bool, score: u32) {
        self.rounds_played += 1;
        self.total_score += score as u64;
        self.best_score = self.best_score.max(score);

        if won {
            self.rounds_won += 1;
            self.current_streak += 1;
            self.best_streak = self.best_streak.max(self.current_streak);
        } else {
            self.rounds_lost += 1;
            self.current_streak = 0;
        }

        let category = self.categories.entry(category_id.to_string()).or_default();
        category.rounds_played += 1;
        category.best_score = category.best_score.max(score);
        if won {
            category.rounds_won += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn win_rate_is_zero_before_any_round() {
        assert_eq!(Stats::default().win_rate(), 0.0);
    }

    #[test]
    fn record_round_tracks_totals_streaks_and_best_score() {
        let mut stats = Stats::default();

        stats.record_round("cars", true, 100);
        stats.record_round("cars", true, 150);
        stats.record_round("countries", false, 0);
        stats.record_round("cars", true, 80);

        assert_eq!(stats.rounds_played, 4);
        assert_eq!(stats.rounds_won, 3);
        assert_eq!(stats.rounds_lost, 1);
        assert_eq!(stats.best_score, 150);
        assert_eq!(stats.total_score, 330);
        assert_eq!(stats.best_streak, 2);
        assert_eq!(stats.current_streak, 1);
        assert!((stats.win_rate() - 0.75).abs() < f64::EPSILON);

        let cars = &stats.categories["cars"];
        assert_eq!(cars.rounds_played, 3);
        assert_eq!(cars.rounds_won, 3);
        assert_eq!(cars.best_score, 150);

        let countries = &stats.categories["countries"];
        assert_eq!(countries.rounds_played, 1);
        assert_eq!(countries.rounds_won, 0);
    }

    #[test]
    fn a_loss_resets_the_current_streak_but_keeps_the_best() {
        let mut stats = Stats::default();

        stats.record_round("cars", true, 100);
        stats.record_round("cars", true, 100);
        stats.record_round("cars", false, 0);

        assert_eq!(stats.current_streak, 0);
        assert_eq!(stats.best_streak, 2);
    }
}
