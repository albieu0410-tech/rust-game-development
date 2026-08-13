use std::time::{Instant, SystemTime, UNIX_EPOCH};

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use deduced_gameplay::{GameController, GameStatus, GameViewState, KnownFact};

use crate::state::{AppState, ContentRes, RoundRes, SelectedCategory};
use crate::theme::{
    self, ACCENT, DIFFERENT, HIGHER, LOWER, MATCH, PARTIAL, RADIUS_LG, RADIUS_MD, RADIUS_PILL,
    RADIUS_SM, SURFACE, SURFACE_HOVER, TEXT, TEXT_DIM, comparison_color, comparison_symbol,
};

#[derive(Component)]
pub struct OnPlayingScreen;

#[derive(Component)]
pub(crate) struct AttemptsText;

#[derive(Component)]
pub(crate) struct AttemptDotsContainer;

#[derive(Component)]
pub(crate) struct RevealCardCells;

#[derive(Component)]
pub(crate) struct RevealFill;

#[derive(Component)]
pub(crate) struct RevealLabel;

#[derive(Component)]
pub(crate) struct KnownFactsContainer;

#[derive(Component)]
pub(crate) struct AnswerButtonsContainer;

#[derive(Component)]
pub(crate) struct HistoryContainer;

#[derive(Component)]
pub(crate) struct AnswerButton(String);

#[derive(Component)]
pub(crate) struct Scrollable;

#[derive(Component)]
pub(crate) struct BackToCategoriesButton;

/// The "Not quite — clues added" banner that briefly flashes after a wrong
/// guess, then despawns itself once `TOAST_VISIBLE_SECS` has elapsed.
#[derive(Component)]
pub(crate) struct WrongGuessToast {
    shown_at: Instant,
}

const TOAST_VISIBLE_SECS: f32 = 1.1;
const REVEAL_CARD_CELLS: usize = 30;
const REVEAL_CARD_COLUMNS: usize = 6;

fn fmt_num(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        value.to_string()
    }
}

fn known_fact_text_and_color(fact: &KnownFact) -> (String, Color) {
    match fact {
        KnownFact::Exact { label, value, .. } => {
            (format!("{label}: {}", value.display_value()), MATCH)
        }
        KnownFact::Range {
            label, min, max, ..
        } => match (min, max) {
            (Some(min), Some(max)) => (
                format!("{label}: {} - {}", fmt_num(*min), fmt_num(*max)),
                PARTIAL,
            ),
            (Some(min), None) => (format!("{label} > {}", fmt_num(*min)), HIGHER),
            (None, Some(max)) => (format!("{label} < {}", fmt_num(*max)), LOWER),
            (None, None) => (label.clone(), TEXT_DIM),
        },
    }
}

fn render_known_facts(commands: &mut Commands, container: Entity, state: &GameViewState) {
    commands.entity(container).despawn_children();
    commands.entity(container).with_children(|panel| {
        for fact in &state.known_facts {
            let (text, color) = known_fact_text_and_color(fact);
            panel
                .spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                        border_radius: BorderRadius::all(Val::Px(RADIUS_PILL)),
                        ..default()
                    },
                    BackgroundColor(color.with_alpha(0.16)),
                ))
                .with_children(|chip| {
                    chip.spawn((Text::new(text), theme::label_font(12.0), TextColor(color)));
                });
        }
    });
}

fn render_attempt_dots(commands: &mut Commands, container: Entity, used: usize, max: usize) {
    commands.entity(container).despawn_children();
    commands.entity(container).with_children(|row| {
        for index in 0..max {
            let filled = index < used;
            row.spawn((
                Node {
                    width: Val::Px(10.0),
                    height: Val::Px(10.0),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(RADIUS_PILL)),
                    ..default()
                },
                BackgroundColor(if filled { ACCENT } else { Color::NONE }),
                BorderColor::all(if filled { ACCENT } else { TEXT_DIM }),
            ));
        }
    });
}

/// Decorative "reveal card": a grid of tiles that clear from a solid block
/// down toward empty as attempts are used, giving the round a visual sense
/// of progress without needing real per-answer artwork (that's tracked
/// separately — see docs/phases.md Phase 5, blocked on an art/licensing
/// decision, not a technical one).
fn render_reveal_card(commands: &mut Commands, container: Entity, used: usize, max: usize) {
    commands.entity(container).despawn_children();
    commands.entity(container).with_children(|grid| {
        let cleared =
            ((used as f32 / max.max(1) as f32) * REVEAL_CARD_CELLS as f32).round() as usize;
        for index in 0..REVEAL_CARD_CELLS {
            let is_cleared = index < cleared;
            let base_alpha = if (index + 1) % 3 == 0 { 0.35 } else { 0.85 };
            grid.spawn((
                Node {
                    width: Val::Percent(100.0 / REVEAL_CARD_COLUMNS as f32),
                    height: Val::Percent(100.0 / (REVEAL_CARD_CELLS / REVEAL_CARD_COLUMNS) as f32),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(ACCENT.with_alpha(if is_cleared { 0.0 } else { base_alpha })),
                BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.1)),
            ));
        }
    });
}

pub fn setup(
    mut commands: Commands,
    content: Res<ContentRes>,
    selected: Res<SelectedCategory>,
    mut round_res: ResMut<RoundRes>,
) {
    let Some(category) = content.0.category(&selected.0) else {
        return;
    };

    if round_res.controller.is_none() {
        let seed = round_res.pending_seed.take().unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(1)
        });
        round_res.controller =
            GameController::new_solo(&content.0.answers, category.clone(), seed).ok();
        round_res.started_at = Some(Instant::now());
    }
    let max_attempts = round_res
        .controller
        .as_ref()
        .map(|controller| controller.state().max_attempts)
        .unwrap_or(category.attempts);
    let initial_reveal = round_res
        .controller
        .as_ref()
        .map(|controller| controller.state().reveal)
        .unwrap_or(deduced_gameplay::RevealState {
            level: 1,
            max_level: max_attempts.max(1) as u8,
        });

    let mut dots_container = None;
    let mut reveal_cells_container = None;

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
                header
                    .spawn((
                        BackToCategoriesButton,
                        Button,
                        Node {
                            width: Val::Px(36.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(RADIUS_SM)),
                            ..default()
                        },
                        BackgroundColor(SURFACE),
                    ))
                    .with_children(|button| {
                        button.spawn((Text::new("<"), theme::label_font(16.0), TextColor(TEXT)));
                    });

                header.spawn((
                    Text::new(category.name.clone()),
                    theme::heading_font(20.0),
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

            dots_container = Some(
                root.spawn((
                    AttemptDotsContainer,
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    },
                ))
                .id(),
            );

            root.spawn((
                Node {
                    height: Val::Px(150.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    overflow: Overflow::clip(),
                    border_radius: BorderRadius::all(Val::Px(RADIUS_LG)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(SURFACE),
                BorderColor::all(theme::BORDER),
                theme::card_shadow(),
            ))
            .with_children(|card| {
                reveal_cells_container = Some(
                    card.spawn((
                        RevealCardCells,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            ..default()
                        },
                    ))
                    .id(),
                );
            });

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|reveal_row| {
                reveal_row
                    .spawn((
                        Node {
                            flex_grow: 1.0,
                            height: Val::Px(6.0),
                            border_radius: BorderRadius::all(Val::Px(RADIUS_PILL)),
                            ..default()
                        },
                        BackgroundColor(SURFACE),
                    ))
                    .with_children(|track| {
                        track.spawn((
                            RevealFill,
                            Node {
                                width: Val::Percent(
                                    100.0 * initial_reveal.level as f32
                                        / initial_reveal.max_level as f32,
                                ),
                                height: Val::Percent(100.0),
                                border_radius: BorderRadius::all(Val::Px(RADIUS_PILL)),
                                ..default()
                            },
                            BackgroundColor(ACCENT),
                        ));
                    });

                reveal_row.spawn((
                    RevealLabel,
                    Text::new(format!(
                        "{} / {}",
                        initial_reveal.level, initial_reveal.max_level
                    )),
                    theme::label_font(12.0),
                    TextColor(TEXT_DIM),
                ));
            });

            root.spawn((
                Text::new("KNOWN FACTS"),
                theme::label_font(10.0),
                TextColor(TEXT_DIM),
            ));
            root.spawn((
                KnownFactsContainer,
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                    ..default()
                },
            ));

            root.spawn((
                Text::new("YOUR GUESSES"),
                theme::label_font(10.0),
                TextColor(TEXT_DIM),
            ));
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
                Text::new("YOUR DEDUCTIONS"),
                theme::label_font(10.0),
                TextColor(TEXT_DIM),
            ));
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

    if let Some(container) = dots_container {
        render_attempt_dots(&mut commands, container, 0, max_attempts);
    }
    if let Some(container) = reveal_cells_container {
        render_reveal_card(&mut commands, container, 0, max_attempts);
    }
}

pub fn teardown(mut commands: Commands, query: Query<Entity, With<OnPlayingScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// Bevy systems commonly take this many distinct SystemParams; splitting them
// into a bundle struct would hurt readability more than it helps here.
#[allow(clippy::too_many_arguments)]
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
    mut reveal_fill: Query<&mut Node, (With<RevealFill>, Without<AnswerButton>)>,
    mut reveal_label: Query<&mut Text, (With<RevealLabel>, Without<AttemptsText>)>,
    known_facts_container: Query<Entity, With<KnownFactsContainer>>,
    history_container: Query<Entity, With<HistoryContainer>>,
    dots_container: Query<Entity, With<AttemptDotsContainer>>,
    reveal_cells_container: Query<Entity, With<RevealCardCells>>,
    toast_root: Query<Entity, With<OnPlayingScreen>>,
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

    let Some(controller) = round_res.controller.as_mut() else {
        return;
    };

    let Ok(result) = controller.submit_guess(guess) else {
        return;
    };
    let state = controller.state();

    if let Some(entity) = pressed_entity {
        commands.entity(entity).despawn();
    }

    for mut text in &mut attempts_text {
        *text = Text::new(format!("{} / {}", state.attempts_used, state.max_attempts));
    }

    for mut node in &mut reveal_fill {
        node.width =
            Val::Percent(100.0 * state.reveal.level as f32 / state.reveal.max_level as f32);
    }
    for mut text in &mut reveal_label {
        *text = Text::new(format!(
            "{} / {}",
            state.reveal.level, state.reveal.max_level
        ));
    }
    if let Ok(container) = known_facts_container.single() {
        render_known_facts(&mut commands, container, &state);
    }
    if let Ok(container) = dots_container.single() {
        render_attempt_dots(
            &mut commands,
            container,
            state.attempts_used,
            state.max_attempts,
        );
    }
    if let Ok(container) = reveal_cells_container.single() {
        render_reveal_card(
            &mut commands,
            container,
            state.attempts_used,
            state.max_attempts,
        );
    }

    if state.status == GameStatus::Playing
        && let Ok(root) = toast_root.single()
    {
        spawn_wrong_guess_toast(&mut commands, root);
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

    if state.status != GameStatus::Playing {
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

fn spawn_wrong_guess_toast(commands: &mut Commands, root: Entity) {
    commands.entity(root).with_children(|parent| {
        parent
            .spawn((
                WrongGuessToast {
                    shown_at: Instant::now(),
                },
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(4.0),
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|row| {
                row.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(RADIUS_PILL)),
                        ..default()
                    },
                    BackgroundColor(DIFFERENT.with_alpha(0.92)),
                    BorderColor::all(DIFFERENT),
                ))
                .with_children(|pill| {
                    pill.spawn((
                        Text::new("Not quite - clues added"),
                        theme::label_font(12.0),
                        TextColor(TEXT),
                    ));
                });
            });
    });
}

pub fn handle_toast_expiry(mut commands: Commands, toasts: Query<(Entity, &WrongGuessToast)>) {
    for (entity, toast) in &toasts {
        if toast.shown_at.elapsed().as_secs_f32() > TOAST_VISIBLE_SECS {
            commands.entity(entity).despawn();
        }
    }
}

pub fn handle_back_button(
    mut interactions: Query<&Interaction, (Changed<Interaction>, With<BackToCategoriesButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &mut interactions {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::Categories);
        }
    }
}
