use std::time::Instant;

use bevy::prelude::*;

use deduced_core::GameContent;
use deduced_gameplay::GameController;
use deduced_save::{FileSaveStorage, Profile};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Home,
    Categories,
    Playing,
    Result,
    Profile,
    Daily,
    Versus,
}

#[derive(Resource)]
pub struct ContentRes(pub GameContent);

#[derive(Resource, Default)]
pub struct SelectedCategory(pub String);

#[derive(Resource, Default)]
pub struct RoundRes {
    pub controller: Option<GameController>,
    pub started_at: Option<Instant>,
    /// Set before transitioning into `AppState::Playing` to force a specific
    /// seed (Daily / Versus reconstruct a server-issued round locally rather
    /// than rolling a random one) — `playing::setup` consumes this if set.
    pub pending_seed: Option<u64>,
    /// Whether the round currently underway is a Daily Challenge attempt, so
    /// `result::setup` knows to also submit it to the server for scoring.
    pub is_daily: bool,
    pub daily_challenge_id: Option<String>,
}

#[derive(Resource)]
pub struct SaveRes {
    pub storage: FileSaveStorage,
    pub profile: Profile,
}
