use std::sync::Mutex;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use bevy::prelude::*;

use deduced_protocol::{DailyChallenge, DailySubmissionRequest};

use crate::screens::nav;
use crate::state::{AppState, RoundRes, SaveRes, SelectedCategory};
use crate::theme::{self, ACCENT, RADIUS_LG, RADIUS_MD, SURFACE, TEXT, TEXT_DIM};

#[derive(Component)]
pub struct OnDailyScreen;

#[derive(Component)]
pub(crate) struct DailyContent;

#[derive(Component)]
pub(crate) struct PlayDailyButton;

enum DailyFetch {
    Loaded(DailyChallenge),
    Failed(String),
}

/// Holds the in-flight background fetch of `/daily/current` and its result.
/// The receiver is polled non-blockingly each frame — never blocks Solo or
/// any other part of the client while the server is slow/unreachable.
#[derive(Resource, Default)]
pub struct DailyRes {
    pub challenge: Option<DailyChallenge>,
    pub error: Option<String>,
    receiver: Option<Mutex<Receiver<DailyFetch>>>,
}

pub fn setup(mut commands: Commands, mut daily_res: ResMut<DailyRes>) {
    daily_res.error = None;

    let (sender, receiver) = channel();
    daily_res.receiver = Some(Mutex::new(receiver));
    std::thread::spawn(move || {
        let url = format!("{}/daily/current", crate::server::BASE_URL);
        let result = ureq::get(&url)
            .call()
            .map_err(|err| err.to_string())
            .and_then(|mut response| {
                response
                    .body_mut()
                    .read_json::<DailyChallenge>()
                    .map_err(|err| err.to_string())
            });
        let message = match result {
            Ok(challenge) => DailyFetch::Loaded(challenge),
            Err(err) => DailyFetch::Failed(err),
        };
        let _ = sender.send(message);
    });

    commands
        .spawn((
            OnDailyScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(14.0),
                ..default()
            },
            theme::app_background(),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Daily Deduction"),
                theme::heading_font(24.0),
                TextColor(TEXT),
            ));

            root.spawn((
                DailyContent,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    margin: UiRect::top(Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(RADIUS_LG)),
                    ..default()
                },
                BackgroundColor(SURFACE),
                BorderColor::all(theme::BORDER),
                theme::card_shadow(),
            ))
            .with_children(|card| {
                card.spawn((
                    Text::new("Loading today's puzzle..."),
                    theme::body_font(14.0),
                    TextColor(TEXT_DIM),
                ));
            });

            nav::spawn(root, AppState::Daily);
        });
}

fn render_content(commands: &mut Commands, container: Entity, daily_res: &DailyRes) {
    commands.entity(container).despawn_children();
    commands.entity(container).with_children(|card| {
        if let Some(challenge) = &daily_res.challenge {
            card.spawn((
                Text::new("TODAY'S PUZZLE"),
                theme::label_font(10.0),
                TextColor(TEXT_DIM),
            ));
            card.spawn((
                Text::new(challenge.category_id.clone()),
                theme::heading_font(22.0),
                TextColor(TEXT),
            ));
            card.spawn((
                Text::new("Everyone gets the same puzzle. Solve it in the fewest attempts."),
                theme::body_font(13.0),
                TextColor(TEXT_DIM),
            ));

            card.spawn((
                PlayDailyButton,
                Button,
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(10.0)),
                    border_radius: BorderRadius::all(Val::Px(RADIUS_MD)),
                    ..default()
                },
                BackgroundColor(ACCENT),
                theme::button_shadow(),
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new("Play Today's Puzzle"),
                    theme::label_font(16.0),
                    TextColor(TEXT),
                ));
            });
        } else if let Some(error) = &daily_res.error {
            card.spawn((
                Text::new("Couldn't reach the server."),
                theme::label_font(14.0),
                TextColor(TEXT_DIM),
            ));
            card.spawn((
                Text::new(error.clone()),
                theme::body_font(11.0),
                TextColor(TEXT_DIM),
            ));
        }
    });
}

pub fn handle_fetch(
    mut commands: Commands,
    mut daily_res: ResMut<DailyRes>,
    container: Query<Entity, With<DailyContent>>,
) {
    let Some(receiver) = daily_res.receiver.as_ref() else {
        return;
    };

    let message = match receiver.lock().unwrap().try_recv() {
        Ok(message) => message,
        Err(TryRecvError::Empty) => return,
        Err(TryRecvError::Disconnected) => return,
    };

    daily_res.receiver = None;
    match message {
        DailyFetch::Loaded(challenge) => {
            daily_res.challenge = Some(challenge);
            daily_res.error = None;
        }
        DailyFetch::Failed(error) => {
            daily_res.error = Some(error);
        }
    }

    if let Ok(container) = container.single() {
        render_content(&mut commands, container, &daily_res);
    }
}

pub fn handle_play_button(
    mut interactions: Query<&Interaction, (Changed<Interaction>, With<PlayDailyButton>)>,
    daily_res: Res<DailyRes>,
    mut selected: ResMut<SelectedCategory>,
    mut round_res: ResMut<RoundRes>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &mut interactions {
        if *interaction == Interaction::Pressed
            && let Some(challenge) = &daily_res.challenge
        {
            selected.0 = challenge.category_id.clone();
            round_res.controller = None;
            round_res.pending_seed = Some(challenge.seed);
            round_res.is_daily = true;
            round_res.daily_challenge_id = Some(challenge.challenge_id.clone());
            next_state.set(AppState::Playing);
        }
    }
}

pub fn teardown(mut commands: Commands, query: Query<Entity, With<OnDailyScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Fire-and-forget submission of a finished Daily round for server-side
/// scoring/leaderboard — the client already trusts its own locally-computed
/// result (same `deduced-core` rules engine the server replays with), so this
/// never blocks the Result screen on the network.
pub fn submit_in_background(
    challenge_id: String,
    save_res: &SaveRes,
    guesses: Vec<String>,
    elapsed_ms: u64,
) {
    let request = DailySubmissionRequest {
        challenge_id,
        player_id: save_res.profile.player_id.clone(),
        guesses,
        elapsed_ms,
    };

    std::thread::spawn(move || {
        let url = format!("{}/daily/submit", crate::server::BASE_URL);
        if let Err(err) = ureq::post(&url).send_json(&request) {
            eprintln!("daily submission skipped (server unreachable?): {err}");
        }
    });
}
