use std::sync::Mutex;
use std::time::Instant;

use deduced_core::{GameContent, Round};

pub struct RoundSession {
    pub round: Round,
    pub started_at: Instant,
}

pub struct AppState {
    pub content: GameContent,
    pub session: Mutex<Option<RoundSession>>,
}

impl AppState {
    pub fn new(content: GameContent) -> Self {
        Self {
            content,
            session: Mutex::new(None),
        }
    }
}
