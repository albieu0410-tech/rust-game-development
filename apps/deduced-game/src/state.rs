use bevy::prelude::*;

use deduced_core::{GameContent, Round};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Menu,
    Playing,
    Result,
}

#[derive(Resource)]
pub struct ContentRes(pub GameContent);

#[derive(Resource, Default)]
pub struct SelectedCategory(pub String);

#[derive(Resource, Default)]
pub struct RoundRes {
    pub round: Option<Round>,
    pub seed: u64,
}
