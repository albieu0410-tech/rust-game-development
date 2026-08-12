use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::stats::Stats;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    /// Local guest identity, generated once on first launch and persisted
    /// thereafter. No account/sign-in is required to have one.
    #[serde(default = "generate_player_id")]
    pub player_id: String,
    pub player_name: String,
    pub stats: Stats,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            player_id: generate_player_id(),
            player_name: "Player".to_string(),
            stats: Stats::default(),
        }
    }
}

fn generate_player_id() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
