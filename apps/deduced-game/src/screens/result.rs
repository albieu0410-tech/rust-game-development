use bevy::prelude::*;

use deduced_save::SaveStorage;

use crate::screens::nav;
use crate::state::{AppState, RoundRes, SaveRes};
use crate::theme::{
    self, ACCENT, ACCENT_HOVER, ACCENT_PRESSED, DIFFERENT, MATCH, RADIUS_LG, RADIUS_MD, SURFACE,
    SURFACE_HOVER, SURFACE_PRESSED, TEXT, TEXT_DIM,
};

#[derive(Component)]
pub struct OnResultScreen;

#[derive(Component)]
pub(crate) enum ResultAction {
    PlayAgain,
    Menu,
}

pub fn setup(mut commands: Commands, round_res: Res<RoundRes>, mut save_res: ResMut<SaveRes>) {
    let Some(controller) = round_res.controller.as_ref() else {
        return;
    };
    let Some(result) = controller.result() else {
        return;
    };

    save_res
        .profile
        .stats
        .record_round(&result.category_id, result.won, result.score.points);
    if let Err(err) = save_res.storage.save_profile(&save_res.profile) {
        warn!("failed to save profile: {err}");
    }
    crate::sync::sync_profile_in_background(&save_res.profile);

    if round_res.is_daily
        && let Some(challenge_id) = round_res.daily_challenge_id.clone()
    {
        let guesses = controller
            .round()
            .guesses
            .iter()
            .map(|guess| guess.answer_id.clone())
            .collect();
        let elapsed_ms = round_res
            .started_at
            .map(|started| started.elapsed().as_millis() as u64)
            .unwrap_or(0);
        crate::screens::daily::submit_in_background(challenge_id, &save_res, guesses, elapsed_ms);
    }

    let won = result.won;
    let (headline, headline_color, badge_color) = if won {
        ("DEDUCED!".to_string(), MATCH, MATCH)
    } else {
        ("Answer revealed".to_string(), TEXT_DIM, DIFFERENT)
    };
    let elapsed = round_res
        .started_at
        .map(|started| started.elapsed())
        .unwrap_or_default();
    let elapsed_label = format!("{}:{:02}", elapsed.as_secs() / 60, elapsed.as_secs() % 60);

    commands
        .spawn((
            OnResultScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            theme::app_background(),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    padding: UiRect::axes(Val::Px(28.0), Val::Px(32.0)),
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
                    Node {
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                        margin: UiRect::bottom(Val::Px(4.0)),
                        border_radius: BorderRadius::all(Val::Px(999.0)),
                        ..default()
                    },
                    BackgroundColor(badge_color.with_alpha(0.18)),
                ))
                .with_children(|badge| {
                    badge.spawn((
                        Text::new(if won { "SOLVED" } else { "NO ATTEMPTS LEFT" }),
                        theme::label_font(12.0),
                        TextColor(badge_color),
                    ));
                });

                card.spawn((
                    Text::new(headline),
                    theme::heading_font(28.0),
                    TextColor(headline_color),
                ));

                card.spawn((
                    Node {
                        width: Val::Px(190.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(18.0)),
                        margin: UiRect::vertical(Val::Px(6.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(RADIUS_MD)),
                        ..default()
                    },
                    BackgroundColor(SURFACE_HOVER),
                    BorderColor::all(theme::BORDER),
                ))
                .with_children(|answer_card| {
                    answer_card.spawn((
                        Text::new(result.answer_name.clone()),
                        theme::heading_font(19.0),
                        TextColor(TEXT),
                    ));
                });

                card.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                })
                .with_children(|stats| {
                    stat_box(
                        stats,
                        "Attempts",
                        &format!("{}/{}", result.attempts_used, result.max_attempts),
                    );
                    stat_box(stats, "Time", &elapsed_label);
                    stat_box(stats, "Score", &result.score.points.to_string());
                });

                card.spawn((
                    ResultAction::PlayAgain,
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(52.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(RADIUS_MD)),
                        ..default()
                    },
                    BackgroundColor(ACCENT),
                    theme::button_shadow(),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Play Again"),
                        theme::label_font(18.0),
                        TextColor(TEXT),
                    ));
                });

                card.spawn((
                    ResultAction::Menu,
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(52.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(RADIUS_MD)),
                        ..default()
                    },
                    BackgroundColor(SURFACE),
                    BorderColor::all(theme::BORDER),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Category Menu"),
                        theme::label_font(16.0),
                        TextColor(TEXT_DIM),
                    ));
                });
            });

            nav::spawn(root, AppState::Result);
        });
}

fn stat_box(parent: &mut ChildSpawnerCommands, label: &str, value: &str) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(10.0)),
                flex_grow: 1.0,
                border_radius: BorderRadius::all(Val::Px(RADIUS_MD)),
                ..default()
            },
            BackgroundColor(SURFACE),
        ))
        .with_children(|stat| {
            stat.spawn((
                Text::new(label.to_uppercase()),
                theme::body_font(9.0),
                TextColor(TEXT_DIM),
            ));
            stat.spawn((
                Text::new(value.to_string()),
                theme::heading_font(16.0),
                TextColor(TEXT),
            ));
        });
}

pub fn teardown(mut commands: Commands, query: Query<Entity, With<OnResultScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn handle_buttons(
    mut interactions: Query<
        (&Interaction, &ResultAction, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut round_res: ResMut<RoundRes>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, action, mut background) in &mut interactions {
        let (base, hover, pressed) = match action {
            ResultAction::PlayAgain => (ACCENT, ACCENT_HOVER, ACCENT_PRESSED),
            ResultAction::Menu => (SURFACE, SURFACE_HOVER, SURFACE_PRESSED),
        };

        match interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(pressed);
                match action {
                    ResultAction::PlayAgain => {
                        round_res.controller = None;
                        next_state.set(AppState::Playing);
                    }
                    ResultAction::Menu => {
                        round_res.controller = None;
                        next_state.set(AppState::Categories);
                    }
                }
            }
            Interaction::Hovered => *background = BackgroundColor(hover),
            Interaction::None => *background = BackgroundColor(base),
        }
    }
}
