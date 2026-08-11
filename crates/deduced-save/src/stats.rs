use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Stats {
    pub rounds_played: u64,
    pub rounds_won: u64,
    pub best_score: u32,
}
