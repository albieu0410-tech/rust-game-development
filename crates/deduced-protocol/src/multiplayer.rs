use serde::{Deserialize, Serialize};

/// A single attribute comparison, shaped for display — the wire equivalent
/// of `deduced_core::AttributeComparison` without depending on `deduced-core`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComparisonDto {
    pub key: String,
    pub label: String,
    pub guessed_value: String,
    /// One of "match" | "higher" | "lower" | "different" | "partial".
    pub comparison: String,
}

/// Messages a connected player sends over the match WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Ready,
    Guess { answer_id: String },
    Leave,
}

/// Messages the server sends to one or both connected players.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Sent once both players are ready. Both clients reconstruct the same
    /// hidden target locally from `category_id` + `seed` + `content_version`.
    MatchStarted {
        category_id: String,
        seed: u64,
        content_version: String,
        max_attempts: usize,
    },
    /// The authoritative outcome of one of *your own* guesses.
    GuessResult {
        attempts_used: usize,
        max_attempts: usize,
        comparisons: Vec<ComparisonDto>,
        won: bool,
    },
    /// Your opponent made a guess. Their guess itself is never revealed —
    /// only that they used another attempt.
    OpponentProgress {
        attempts_used: usize,
    },
    OpponentSolved {
        attempts_used: usize,
    },
    OpponentLeft,
    /// `winner_id` is `None` if both players ran out of attempts without
    /// solving it.
    MatchFinished {
        winner_id: Option<String>,
    },
    Error {
        message: String,
    },
}
