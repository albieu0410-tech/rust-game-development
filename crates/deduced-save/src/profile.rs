use serde::{Deserialize, Serialize};

use crate::stats::Stats;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    pub player_name: String,
    pub stats: Stats,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            player_name: "Player".to_string(),
            stats: Stats::default(),
        }
    }
}
