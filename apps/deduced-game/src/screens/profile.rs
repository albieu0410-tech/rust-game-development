use bevy::prelude::*;

use crate::screens::nav;
use crate::state::{AppState, SaveRes};
use crate::theme::{self, ACCENT, RADIUS_LG, RADIUS_MD, SURFACE, TEXT, TEXT_DIM};

#[derive(Component)]
pub struct OnProfileScreen;

fn stat_box(parent: &mut ChildSpawnerCommands, label: &str, value: &str) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(12.0)),
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
                theme::heading_font(18.0),
                TextColor(TEXT),
            ));
        });
}

pub fn setup(mut commands: Commands, save_res: Res<SaveRes>) {
    let stats = &save_res.profile.stats;
    let win_rate = format!("{:.0}%", stats.win_rate() * 100.0);

    commands
        .spawn((
            OnProfileScreen,
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
                Text::new("Your Stats"),
                theme::heading_font(24.0),
                TextColor(TEXT),
            ));

            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(18.0)),
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
                    Node {
                        width: Val::Px(60.0),
                        height: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Percent(50.0)),
                        ..default()
                    },
                    BackgroundColor(ACCENT),
                ))
                .with_children(|ring| {
                    ring.spawn((
                        Text::new(stats.rounds_won.to_string()),
                        theme::heading_font(18.0),
                        TextColor(TEXT),
                    ));
                });

                card.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|info| {
                    info.spawn((
                        Text::new(save_res.profile.player_name.clone()),
                        theme::label_font(16.0),
                        TextColor(TEXT),
                    ));
                    info.spawn((
                        Text::new(format!("{} rounds played", stats.rounds_played)),
                        theme::body_font(12.0),
                        TextColor(TEXT_DIM),
                    ));
                });
            });

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row| {
                stat_box(row, "Played", &stats.rounds_played.to_string());
                stat_box(row, "Won", &stats.rounds_won.to_string());
                stat_box(row, "Win rate", &win_rate);
            });

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row| {
                stat_box(row, "Current streak", &stats.current_streak.to_string());
                stat_box(row, "Best streak", &stats.best_streak.to_string());
                stat_box(row, "Best score", &stats.best_score.to_string());
            });

            let mut categories: Vec<_> = stats.categories.iter().collect();
            categories.sort_by_key(|(_, stats)| std::cmp::Reverse(stats.rounds_played));

            if !categories.is_empty() {
                root.spawn((
                    Text::new("BY CATEGORY"),
                    theme::label_font(10.0),
                    TextColor(TEXT_DIM),
                    Node {
                        margin: UiRect::top(Val::Px(6.0)),
                        ..default()
                    },
                ));

                root.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|list| {
                    for (category_id, category_stats) in categories {
                        list.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(12.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(RADIUS_MD)),
                                ..default()
                            },
                            BackgroundColor(SURFACE),
                            BorderColor::all(theme::BORDER),
                        ))
                        .with_children(|entry| {
                            entry.spawn((
                                Text::new(category_id.clone()),
                                theme::label_font(13.0),
                                TextColor(TEXT),
                            ));
                            entry.spawn((
                                Text::new(format!(
                                    "{}/{} won",
                                    category_stats.rounds_won, category_stats.rounds_played
                                )),
                                theme::body_font(12.0),
                                TextColor(TEXT_DIM),
                            ));
                        });
                    }
                });
            }

            nav::spawn(root, AppState::Profile);
        });
}

pub fn teardown(mut commands: Commands, query: Query<Entity, With<OnProfileScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
