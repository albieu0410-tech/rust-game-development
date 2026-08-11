use bevy::prelude::*;

use deduced_core::{RoundStatus, score_round};

use crate::state::{AppState, RoundRes};
use crate::theme::{BG, MATCH, PANEL, PANEL_LIGHT, TEXT, TEXT_DIM};

#[derive(Component)]
pub struct OnResultScreen;

#[derive(Component)]
pub(crate) enum ResultAction {
    PlayAgain,
    Menu,
}

pub fn setup(mut commands: Commands, round_res: Res<RoundRes>) {
    let Some(round) = round_res.round.as_ref() else {
        return;
    };

    let (headline, headline_color) = match round.status {
        RoundStatus::Won => ("You got it!".to_string(), MATCH),
        _ => ("Out of attempts".to_string(), TEXT_DIM),
    };

    let score = score_round(round);

    commands
        .spawn((
            OnResultScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(12.0),
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            BackgroundColor(BG),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new(headline),
                TextFont {
                    font_size: FontSize::Px(28.0),
                    ..default()
                },
                TextColor(headline_color),
            ));
            root.spawn((
                Text::new(format!("Answer: {}", round.answer.name)),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(TEXT),
            ));
            root.spawn((
                Text::new(format!("Score: {}", score.points)),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(TEXT_DIM),
            ));

            root.spawn((
                ResultAction::PlayAgain,
                Button,
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                BackgroundColor(PANEL),
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new("Play Again"),
                    TextFont {
                        font_size: FontSize::Px(18.0),
                        ..default()
                    },
                    TextColor(TEXT),
                ));
            });

            root.spawn((
                ResultAction::Menu,
                Button,
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(PANEL),
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new("Category Menu"),
                    TextFont {
                        font_size: FontSize::Px(18.0),
                        ..default()
                    },
                    TextColor(TEXT),
                ));
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
        match interaction {
            Interaction::Pressed => match action {
                ResultAction::PlayAgain => {
                    round_res.round = None;
                    next_state.set(AppState::Playing);
                }
                ResultAction::Menu => {
                    round_res.round = None;
                    next_state.set(AppState::Menu);
                }
            },
            Interaction::Hovered => *background = BackgroundColor(PANEL_LIGHT),
            Interaction::None => *background = BackgroundColor(PANEL),
        }
    }
}
