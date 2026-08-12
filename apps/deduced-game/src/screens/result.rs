use bevy::prelude::*;

use deduced_save::SaveStorage;

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

    let won = result.won;
    let (headline, headline_color, badge_color) = if won {
        ("You got it!".to_string(), MATCH, MATCH)
    } else {
        ("Out of attempts".to_string(), TEXT_DIM, DIFFERENT)
    };

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
                    Text::new(format!("Answer: {}", result.answer_name)),
                    theme::body_font(17.0),
                    TextColor(TEXT),
                ));
                card.spawn((
                    Text::new(format!("Score: {}", result.score.points)),
                    theme::body_font(15.0),
                    TextColor(TEXT_DIM),
                    Node {
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                ));

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
