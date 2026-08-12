use bevy::prelude::*;

use crate::state::{AppState, ContentRes, RoundRes, SelectedCategory};
use crate::theme::{
    self, ACCENT, RADIUS_MD, SURFACE, SURFACE_HOVER, SURFACE_PRESSED, TEXT, TEXT_DIM,
};

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
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            theme::app_background(),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(72.0),
                    height: Val::Px(72.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(6.0)),
                    border_radius: BorderRadius::all(Val::Px(RADIUS_MD)),
                    ..default()
                },
                BackgroundColor(ACCENT),
                theme::button_shadow(),
            ))
            .with_children(|logo| {
                logo.spawn((Text::new("D"), theme::heading_font(34.0), TextColor(TEXT)));
            });

            root.spawn((
                Text::new("DEDUCED"),
                theme::heading_font(32.0),
                TextColor(TEXT),
            ));
            root.spawn((
                Text::new("Pick a category to start deducing"),
                theme::body_font(15.0),
                TextColor(TEXT_DIM),
                Node {
                    margin: UiRect::bottom(Val::Px(14.0)),
                    ..default()
                },
            ));

            for category in &content.0.categories {
                root.spawn((
                    CategoryButton(category.id.clone()),
                    Button,
                    Node {
                        width: Val::Px(280.0),
                        height: Val::Px(58.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(6.0)),
                        border: UiRect::all(Val::Px(1.5)),
                        border_radius: BorderRadius::all(Val::Px(RADIUS_MD)),
                        ..default()
                    },
                    BackgroundColor(SURFACE),
                    BorderColor::all(theme::BORDER),
                    theme::button_shadow(),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(category.name.clone()),
                        theme::label_font(19.0),
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
        (
            &Interaction,
            &CategoryButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut selected: ResMut<SelectedCategory>,
    mut round: ResMut<RoundRes>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, category_button, mut background, mut border) in &mut interactions {
        match interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(SURFACE_PRESSED);
                selected.0 = category_button.0.clone();
                round.round = None;
                next_state.set(AppState::Playing);
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
}
