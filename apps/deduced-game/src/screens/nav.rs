use bevy::prelude::*;

use crate::state::AppState;
use crate::theme::{self, ACCENT, SURFACE, TEXT_DIM};

const NAV_HEIGHT: f32 = 64.0;

#[derive(Component)]
pub struct NavButton(pub AppState);

/// Spawns the persistent bottom nav bar (Home / Daily / Versus / Profile /
/// Store) as an absolutely-positioned child of `parent`, pinned to the
/// bottom regardless of how tall the rest of the screen's content is. Store
/// stays a placeholder — matching deduced-web and CLAUDE.md, which
/// intentionally leaves store/monetization unbuilt.
pub fn spawn(parent: &mut ChildSpawnerCommands, active: AppState) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                height: Val::Px(NAV_HEIGHT),
                flex_direction: FlexDirection::Row,
                border: UiRect::top(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(SURFACE),
            BorderColor::all(theme::BORDER),
        ))
        .with_children(|nav| {
            // Plain text labels only: the bundled font is a small ASCII
            // subset (FiraMono-subset) and doesn't cover the Unicode glyphs
            // (⌂ ▣ ⚔ ♙ ◇) a proper icon row would use — those rendered as
            // missing-glyph boxes when tried, so this sticks to what the
            // font actually has.
            let items = [
                ("Home", Some(AppState::Home)),
                ("Daily", Some(AppState::Daily)),
                ("Versus", Some(AppState::Versus)),
                ("Profile", Some(AppState::Profile)),
                ("Store", None),
            ];

            for (label, target) in items {
                let is_active = target == Some(active);
                let color = if is_active { ACCENT } else { TEXT_DIM };

                let item_node = Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_grow: 1.0,
                    ..default()
                };

                let mut item = if let Some(target) = target {
                    nav.spawn((Button, NavButton(target), item_node))
                } else {
                    nav.spawn(item_node)
                };

                item.with_children(|col| {
                    col.spawn((Text::new(label), theme::label_font(11.0), TextColor(color)));
                });
            }
        });
}

pub fn handle_nav_buttons(
    mut interactions: Query<(&Interaction, &NavButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, nav_button) in &mut interactions {
        if *interaction == Interaction::Pressed {
            next_state.set(nav_button.0);
        }
    }
}
