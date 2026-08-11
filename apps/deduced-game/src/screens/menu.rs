use bevy::prelude::*;

use crate::state::{AppState, ContentRes, RoundRes, SelectedCategory};
use crate::theme::{BG, PANEL, PANEL_LIGHT, TEXT, TEXT_DIM};

#[derive(Component)]
pub struct OnMenuScreen;

#[derive(Component)]
pub(crate) struct CategoryButton(String);

pub fn setup(mut commands: Commands, content: Res<ContentRes>) {
    commands
        .spawn((
            OnMenuScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(14.0),
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            BackgroundColor(BG),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("DEDUCED"),
                TextFont {
                    font_size: FontSize::Px(34.0),
                    ..default()
                },
                TextColor(TEXT),
            ));
            root.spawn((
                Text::new("Pick a category"),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(TEXT_DIM),
            ));

            for category in &content.0.categories {
                root.spawn((
                    CategoryButton(category.id.clone()),
                    Button,
                    Node {
                        width: Val::Px(260.0),
                        height: Val::Px(56.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(category.name.clone()),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(TEXT),
                    ));
                });
            }
        });
}

pub fn teardown(mut commands: Commands, query: Query<Entity, With<OnMenuScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn handle_buttons(
    mut interactions: Query<
        (&Interaction, &CategoryButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut selected: ResMut<SelectedCategory>,
    mut round: ResMut<RoundRes>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, category_button, mut background) in &mut interactions {
        match interaction {
            Interaction::Pressed => {
                selected.0 = category_button.0.clone();
                round.round = None;
                next_state.set(AppState::Playing);
            }
            Interaction::Hovered => *background = BackgroundColor(PANEL_LIGHT),
            Interaction::None => *background = BackgroundColor(PANEL),
        }
    }
}
