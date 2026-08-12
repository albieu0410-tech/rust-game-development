mod screens;
mod state;
mod sync;
mod theme;

use std::path::Path;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use deduced_content::load_content_from_dir;
use deduced_save::{FileSaveStorage, SaveStorage};

use screens::{categories, home, playing, result};
use state::{AppState, ContentRes, RoundRes, SaveRes, SelectedCategory};

fn main() {
    let content = load_content_from_dir(Path::new("content"))
        .expect("game content must load from the content/ directory");

    let storage = FileSaveStorage::new(Path::new("save/profile.json"));
    let profile = storage.load_profile().unwrap_or_default();

    App::new()
        .insert_resource(ClearColor(theme::BG_BOTTOM))
        .insert_resource(ContentRes(content))
        .insert_resource(SelectedCategory::default())
        .insert_resource(RoundRes::default())
        .insert_resource(SaveRes { storage, profile })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "DEDUCED".into(),
                resolution: WindowResolution::new(390, 844),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(AppState::Home), home::setup)
        .add_systems(OnExit(AppState::Home), home::teardown)
        .add_systems(
            Update,
            home::handle_buttons.run_if(in_state(AppState::Home)),
        )
        .add_systems(OnEnter(AppState::Categories), categories::setup)
        .add_systems(OnExit(AppState::Categories), categories::teardown)
        .add_systems(
            Update,
            categories::handle_buttons.run_if(in_state(AppState::Categories)),
        )
        .add_systems(OnEnter(AppState::Playing), playing::setup)
        .add_systems(OnExit(AppState::Playing), playing::teardown)
        .add_systems(
            Update,
            (playing::handle_answer_buttons, playing::handle_scroll)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(OnEnter(AppState::Result), result::setup)
        .add_systems(OnExit(AppState::Result), result::teardown)
        .add_systems(
            Update,
            result::handle_buttons.run_if(in_state(AppState::Result)),
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
