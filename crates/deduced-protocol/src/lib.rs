//! Shared wire types for `deduced-client` <-> `deduced-server` communication.
//!
//! This crate intentionally holds no game logic and does not depend on
//! `deduced-core`. It only describes message shapes so both sides agree on
//! the same schema; each side interprets the data with its own copy of the
//! rules engine.

pub mod daily;
pub mod health;
pub mod matches;
pub mod multiplayer;
pub mod profile;

pub use daily::{
    DailyChallenge, DailyLeaderboard, DailySubmissionRequest, DailySubmissionResult,
    LeaderboardEntry,
};
pub use health::HealthResponse;
pub use matches::{MatchHistoryEntry, QueueResponse};
pub use multiplayer::{ClientMessage, ComparisonDto, ServerMessage};
pub use profile::{ProfileSyncRequest, ProfileSyncResponse};
