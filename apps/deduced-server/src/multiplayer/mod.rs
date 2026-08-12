pub mod lobby;
pub mod match_actor;
pub mod matchmaking;
pub mod websocket;

use std::collections::HashMap;
use std::sync::Mutex;

use match_actor::MatchHandle;

#[derive(Default)]
pub struct MultiplayerState {
    pub matches: Mutex<HashMap<String, MatchHandle>>,
    pub join_codes: Mutex<HashMap<String, String>>,
    /// Quick Match queue: at most one player waits at a time (1v1 only).
    pub matchmaking_waiting: Mutex<Option<String>>,
    /// Filled in for the player who was already waiting once someone pairs
    /// with them; drained the next time they poll `/matchmaking/status`.
    pub matchmaking_matched: Mutex<HashMap<String, String>>,
}
