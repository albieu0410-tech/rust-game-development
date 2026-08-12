use std::time::{SystemTime, UNIX_EPOCH};

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use deduced_core::{Round, RoundConfig, RoundStatus};

use crate::state::{AppState, ContentRes, RoundRes, SelectedCategory};
use crate::theme::{
    self, ACCENT, RADIUS_MD, RADIUS_PILL, RADIUS_SM, SURFACE, SURFACE_HOVER, TEXT, TEXT_DIM,
    comparison_color, comparison_symbol,
};

#[derive(Component)]
pub struct OnPlayingScreen;

#[derive(Component)]
pub(crate) struct AttemptsText;

#[derive(Component)]
pub(crate) struct AnswerButtonsContainer;

#[derive(Component)]
pub(crate) struct HistoryContainer;

#[derive(Component)]
pub(crate) struct AnswerButton(String);

#[derive(Component)]
pub(crate) struct Scrollable;

pub fn setup(
    mut commands: Commands,
    content: Res<ContentRes>,
    selected: Res<SelectedCategory>,
    mut round_res: ResMut<RoundRes>,
) {
    let Some(category) = content.0.category(&selected.0) else {
        return;
    };

    if round_res.round.is_none() {
        round_res.seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1);
        round_res.round = Round::new(
            &content.0.answers,
            RoundConfig {
                category: category.id.clone(),
                seed: round_res.seed,
                max_attempts: category.attempts,
            },
        )
        .ok();
    }
    let max_attempts = round_res
        .round
        .as_ref()
        .map(|round| round.max_attempts)
        .unwrap_or(category.attempts);

    commands
        .spawn((
            OnPlayingScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            theme::app_background(),
        ))
        .with_children(|root| {
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            })
            .with_children(|header| {
                header.spawn((
                    Text::new(category.name.clone()),
                    theme::heading_font(22.0),
                    TextColor(TEXT),
                ));

                header
                    .spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            border_radius: BorderRadius::all(Val::Px(RADIUS_PILL)),
                            ..default()
                        },
                        BackgroundColor(SURFACE),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            AttemptsText,
                            Text::new(format!("0 / {max_attempts}")),
                            theme::label_font(14.0),
                            TextColor(TEXT_DIM),
                        ));
                    });
            });

            root.spawn((
                AnswerButtonsContainer,
                Scrollable,
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    max_height: Val::Px(190.0),
                    overflow: Overflow::scroll_y(),
                    padding: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
            ))
            .with_children(|list| {
                for answer in content.0.answers_for_category(&category.id) {
                    list.spawn((
                        AnswerButton(answer.id.clone()),
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(9.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(RADIUS_PILL)),
                            ..default()
                        },
                        BackgroundColor(SURFACE),
                        BorderColor::all(theme::BORDER),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new(answer.name.clone()),
                            theme::body_font(14.0),
                            TextColor(TEXT),
                        ));
                    });
                }
            });

            root.spawn((
                HistoryContainer,
                Scrollable,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    flex_grow: 1.0,
                    overflow: Overflow::scroll_y(),
                    padding: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
            ));
        });
}

pub fn teardown(mut commands: Commands, query: Query<Entity, With<OnPlayingScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn handle_answer_buttons(
    mut commands: Commands,
    mut interactions: Query<
        (
            Entity,
            &Interaction,
            &AnswerButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    content: Res<ContentRes>,
    selected: Res<SelectedCategory>,
    mut round_res: ResMut<RoundRes>,
    mut attempts_text: Query<&mut Text, With<AttemptsText>>,
    history_container: Query<Entity, With<HistoryContainer>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(category) = content.0.category(&selected.0) else {
        return;
    };

    let mut pressed_entity = None;
    let mut pressed_answer_id = None;

    for (entity, interaction, answer_button, mut background, mut border) in &mut interactions {
        match interaction {
            Interaction::Pressed => {
                pressed_entity = Some(entity);
                pressed_answer_id = Some(answer_button.0.clone());
            }
            Interaction::Hovered => {
                *background = BackgroundColor(SURFACE_HOVER);
                *border = BorderColor::all(ACCENT);
            }
            Interaction::None => {
                *background = BackgroundColor(SURFACE);
                *border = BorderColor::all(theme::BORDER);
            }
        }
    }

    let Some(answer_id) = pressed_answer_id else {
        return;
    };
    let Some(guess) = content
        .0
        .answers_for_category(&category.id)
        .find(|answer| answer.id == answer_id)
    else {
        return;
    };

    let Some(round) = round_res.round.as_mut() else {
        return;
    };

    let Ok(result) = round.submit_guess(category, guess).cloned() else {
        return;
    };

    if let Some(entity) = pressed_entity {
        commands.entity(entity).despawn();
    }

    for mut text in &mut attempts_text {
        *text = Text::new(format!(
            "{} / {}",
            round.attempts_used(),
            round.max_attempts
        ));
    }

    if let Ok(container) = history_container.single() {
        commands.entity(container).with_children(|history| {
            history
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(RADIUS_MD)),
                        ..default()
                    },
                    BackgroundColor(SURFACE),
                    BorderColor::all(theme::BORDER),
                    theme::button_shadow(),
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new(result.answer_name.clone()),
                        theme::label_font(15.0),
                        TextColor(TEXT),
                    ));
                    row.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(6.0),
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|chips| {
                        for comparison in &result.comparisons {
                            chips
                                .spawn((
                                    Node {
                                        padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                                        border_radius: BorderRadius::all(Val::Px(RADIUS_SM)),
                                        ..default()
                                    },
                                    BackgroundColor(comparison_color(comparison.comparison)),
                                ))
                                .with_children(|chip| {
                                    chip.spawn((
                                        Text::new(format!(
                                            "{} {} {}",
                                            comparison.label,
                                            comparison.guessed_value.display_value(),
                                            comparison_symbol(comparison.comparison)
                                        )),
                                        theme::label_font(12.0),
                                        TextColor(TEXT),
                                    ));
                                });
                        }
                    });
                });
        });
    }

    if round.status != RoundStatus::Playing {
        next_state.set(AppState::Result);
    }
}

pub fn handle_scroll(
    mut wheel_events: MessageReader<MouseWheel>,
    mut scrollables: Query<&mut ScrollPosition, With<Scrollable>>,
) {
    for event in wheel_events.read() {
        for mut scroll_position in &mut scrollables {
            scroll_position.y -= event.y * 20.0;
        }
    }
}
