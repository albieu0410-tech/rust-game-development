use bevy::prelude::*;

use deduced_core::Comparison;

pub const BG: Color = Color::srgb(0.07, 0.07, 0.1);
pub const PANEL: Color = Color::srgb(0.13, 0.13, 0.18);
pub const PANEL_LIGHT: Color = Color::srgb(0.19, 0.19, 0.25);
pub const TEXT: Color = Color::srgb(0.95, 0.95, 0.97);
pub const TEXT_DIM: Color = Color::srgb(0.65, 0.65, 0.72);

pub const MATCH: Color = Color::srgb(0.25, 0.7, 0.35);
pub const HIGHER: Color = Color::srgb(0.85, 0.55, 0.2);
pub const LOWER: Color = Color::srgb(0.3, 0.55, 0.85);
pub const DIFFERENT: Color = Color::srgb(0.45, 0.16, 0.16);
pub const PARTIAL: Color = Color::srgb(0.75, 0.65, 0.2);

pub fn comparison_color(comparison: Comparison) -> Color {
    match comparison {
        Comparison::Match => MATCH,
        Comparison::Higher => HIGHER,
        Comparison::Lower => LOWER,
        Comparison::Different => DIFFERENT,
        Comparison::Partial => PARTIAL,
    }
}

pub fn comparison_symbol(comparison: Comparison) -> &'static str {
    match comparison {
        Comparison::Match => "=",
        Comparison::Higher => "^",
        Comparison::Lower => "v",
        Comparison::Different => "x",
        Comparison::Partial => "~",
    }
}
