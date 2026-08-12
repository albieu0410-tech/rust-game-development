use serde::{Deserialize, Serialize};

/// Parameters for today's shared puzzle. The client reconstructs the round
/// locally from `category_id` + `seed` + `content_version` — the server does
/// not need to be involved in actually playing it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DailyChallenge {
    pub challenge_id: String,
    pub category_id: String,
    pub seed: u64,
    pub content_version: String,
}

/// A completed Daily replay submitted for server-side validation and scoring.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DailySubmissionRequest {
    pub challenge_id: String,
    /// Client-generated local player identifier (guest-first, no account required).
    pub player_id: String,
    /// Answer ids, in the order they were guessed.
    pub guesses: Vec<String>,
    pub elapsed_ms: u64,
}

/// The server's authoritative result after replaying a submission through
/// the shared rules engine. The client's own report of "I won" is never
/// trusted directly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DailySubmissionResult {
    pub won: bool,
    pub attempts_used: usize,
    pub max_attempts: usize,
    pub score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeaderboardEntry {
    pub player_id: String,
    pub won: bool,
    pub score: u32,
    pub attempts_used: usize,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DailyLeaderboard {
    pub challenge_id: String,
    pub entries: Vec<LeaderboardEntry>,
}
