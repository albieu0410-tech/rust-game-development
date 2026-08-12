use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchHistoryEntry {
    pub match_id: String,
    pub category_id: String,
    pub player_a: String,
    pub player_b: String,
    pub winner_id: Option<String>,
    pub finished_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueueResponse {
    /// "waiting" | "matched"
    pub status: String,
    pub match_id: Option<String>,
}
