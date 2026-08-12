use bevy::prelude::*;

use crate::state::AppState;
use crate::theme::{self, ACCENT, RADIUS_LG, RADIUS_MD, SURFACE, SURFACE_HOVER, TEXT, TEXT_DIM};

#[derive(Component)]
pub struct OnHomeScreen;

#[derive(Component)]
pub(crate) struct SoloButton;

fn mode_card_node() -> (Node, BackgroundColor, BorderColor, BoxShadow) {
    (
        Node {
            width: Val::Px(300.0),
            padding: UiRect::all(Val::Px(16.0)),
            margin: UiRect::top(Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(RADIUS_LG)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        },
        BackgroundColor(SURFACE),
        BorderColor::all(theme::BORDER),
        theme::button_shadow(),
    )
}

pub fn setup(mut commands: Commands) {
    commands
        .spawn((
            OnHomeScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(10.0),
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
                    margin: UiRect::top(Val::Px(36.0)),
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
                Text::new("Every wrong guess makes you smarter."),
                theme::body_font(15.0),
                TextColor(TEXT_DIM),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            root.spawn((SoloButton, Button, mode_card_node()))
                .with_children(|card| {
                    card.spawn((Text::new("Solo"), theme::label_font(18.0), TextColor(TEXT)));
                    card.spawn((
                        Text::new("Choose a category and start deducing"),
                        theme::body_font(13.0),
                        TextColor(TEXT_DIM),
                    ));
                });

            root.spawn(mode_card_node()).with_children(|card| {
                card.spawn((
                    Text::new("Daily Deduction"),
                    theme::label_font(18.0),
                    TextColor(TEXT_DIM),
                ));
                card.spawn((
                    Text::new("One puzzle shared worldwide (coming soon)"),
                    theme::body_font(13.0),
                    TextColor(TEXT_DIM),
                ));
            });

            root.spawn(mode_card_node()).with_children(|card| {
                card.spawn((
                    Text::new("Versus"),
                    theme::label_font(18.0),
                    TextColor(TEXT_DIM),
                ));
                card.spawn((
                    Text::new("Challenge another player (coming soon)"),
                    theme::body_font(13.0),
                    TextColor(TEXT_DIM),
                ));
            });
        });
}

pub fn teardown(mut commands: Commands, query: Query<Entity, With<OnHomeScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

type SoloButtonFilter = (Changed<Interaction>, With<SoloButton>);

pub fn handle_buttons(
    mut interactions: Query<(&Interaction, &mut BackgroundColor), SoloButtonFilter>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut background) in &mut interactions {
        match interaction {
            Interaction::Pressed => {
                next_state.set(AppState::Categories);
            }
            Interaction::Hovered => *background = BackgroundColor(SURFACE_HOVER),
            Interaction::None => *background = BackgroundColor(SURFACE),
        }
    }
}
