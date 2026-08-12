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
}

#[derive(Resource)]
pub struct ContentRes(pub GameContent);

#[derive(Resource, Default)]
pub struct SelectedCategory(pub String);

#[derive(Resource, Default)]
pub struct RoundRes {
    pub controller: Option<GameController>,
}

#[derive(Resource)]
pub struct SaveRes {
    pub storage: FileSaveStorage,
    pub profile: Profile,
}
