use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::ui::{BoxShadow, ShadowStyle};

use deduced_core::Comparison;

// ---- Palette ----
pub const BG_TOP: Color = Color::srgb(0.10, 0.09, 0.17);
pub const BG_BOTTOM: Color = Color::srgb(0.045, 0.045, 0.075);

pub const SURFACE: Color = Color::srgb(0.15, 0.14, 0.21);
pub const SURFACE_HOVER: Color = Color::srgb(0.20, 0.19, 0.29);
pub const SURFACE_PRESSED: Color = Color::srgb(0.115, 0.11, 0.17);
pub const BORDER: Color = Color::srgba(1.0, 1.0, 1.0, 0.08);

pub const ACCENT: Color = Color::srgb(0.56, 0.42, 0.95);
pub const ACCENT_HOVER: Color = Color::srgb(0.65, 0.52, 1.0);
pub const ACCENT_PRESSED: Color = Color::srgb(0.45, 0.33, 0.82);

pub const TEXT: Color = Color::srgb(0.96, 0.96, 0.98);
pub const TEXT_DIM: Color = Color::srgb(0.68, 0.67, 0.75);

pub const MATCH: Color = Color::srgb(0.30, 0.78, 0.48);
pub const HIGHER: Color = Color::srgb(0.92, 0.62, 0.24);
pub const LOWER: Color = Color::srgb(0.32, 0.6, 0.92);
pub const DIFFERENT: Color = Color::srgb(0.55, 0.22, 0.28);
pub const PARTIAL: Color = Color::srgb(0.85, 0.72, 0.24);

// ---- Radii ----
pub const RADIUS_SM: f32 = 10.0;
pub const RADIUS_MD: f32 = 16.0;
pub const RADIUS_LG: f32 = 26.0;
pub const RADIUS_PILL: f32 = 999.0;

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

/// Full-screen top-to-bottom background gradient used behind every screen.
pub fn app_background() -> BackgroundGradient {
    BackgroundGradient::from(LinearGradient::to_bottom(vec![
        ColorStop::auto(BG_TOP),
        ColorStop::auto(BG_BOTTOM),
    ]))
}

fn font(size: f32, weight: FontWeight) -> TextFont {
    TextFont {
        font_size: FontSize::Px(size),
        weight,
        ..default()
    }
}

pub fn heading_font(size: f32) -> TextFont {
    font(size, FontWeight::BOLD)
}

pub fn label_font(size: f32) -> TextFont {
    font(size, FontWeight::SEMIBOLD)
}

pub fn body_font(size: f32) -> TextFont {
    font(size, FontWeight::NORMAL)
}

/// Subtle drop shadow used behind raised panels and cards.
pub fn card_shadow() -> BoxShadow {
    BoxShadow(vec![ShadowStyle {
        color: Color::srgba(0.0, 0.0, 0.0, 0.35),
        x_offset: Val::Px(0.0),
        y_offset: Val::Px(6.0),
        spread_radius: Val::Px(-2.0),
        blur_radius: Val::Px(18.0),
    }])
}

/// Tighter shadow used behind buttons and chips.
pub fn button_shadow() -> BoxShadow {
    BoxShadow(vec![ShadowStyle {
        color: Color::srgba(0.0, 0.0, 0.0, 0.3),
        x_offset: Val::Px(0.0),
        y_offset: Val::Px(3.0),
        spread_radius: Val::Px(-2.0),
        blur_radius: Val::Px(10.0),
    }])
}
