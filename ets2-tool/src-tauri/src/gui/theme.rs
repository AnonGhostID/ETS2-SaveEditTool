//! Central design tokens and the global egui style for the native UI.
//!
//! Everything visual is defined here: one dark "workshop" palette with an
//! amber accent, compact typography and a consistent widget geometry.

use std::collections::BTreeMap;

use eframe::egui::{
    self, Color32, FontFamily, FontId, Rounding, Stroke, TextStyle, Visuals,
};

// Backgrounds
pub const BG: Color32 = Color32::from_rgb(13, 16, 22);
pub const SIDEBAR: Color32 = Color32::from_rgb(16, 20, 27);
pub const SURFACE: Color32 = Color32::from_rgb(22, 27, 37);
pub const SURFACE_ALT: Color32 = Color32::from_rgb(27, 33, 45);
pub const INPUT_BG: Color32 = Color32::from_rgb(9, 12, 17);

// Lines
pub const BORDER: Color32 = Color32::from_rgb(37, 45, 60);
pub const BORDER_STRONG: Color32 = Color32::from_rgb(54, 65, 85);

// Text
pub const TEXT: Color32 = Color32::from_rgb(228, 233, 242);
pub const MUTED: Color32 = Color32::from_rgb(138, 148, 167);
pub const FAINT: Color32 = Color32::from_rgb(92, 102, 120);

// Accent & status
pub const ACCENT: Color32 = Color32::from_rgb(240, 166, 58);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(48, 38, 20);
pub const ON_ACCENT: Color32 = Color32::from_rgb(26, 19, 7);
pub const SUCCESS: Color32 = Color32::from_rgb(76, 195, 138);
pub const WARNING: Color32 = Color32::from_rgb(224, 164, 88);
pub const DANGER: Color32 = Color32::from_rgb(229, 83, 75);

pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.text_styles = BTreeMap::from([
        (
            TextStyle::Heading,
            FontId::new(17.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(13.5, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(12.5, FontFamily::Proportional),
        ),
        (TextStyle::Small, FontId::new(10.5, FontFamily::Proportional)),
        (
            TextStyle::Monospace,
            FontId::new(12.0, FontFamily::Monospace),
        ),
    ]);

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.interact_size = egui::vec2(40.0, 30.0);
    style.spacing.slider_width = 150.0;
    style.spacing.indent = 16.0;
    style.spacing.combo_width = 120.0;
    style.animation_time = 0.12;
    style.visuals = visuals();

    ctx.set_style(style);
}

fn visuals() -> Visuals {
    let mut v = Visuals::dark();

    v.panel_fill = BG;
    v.window_fill = SURFACE;
    v.window_stroke = Stroke::new(1.0, BORDER);
    v.window_rounding = Rounding::same(10.0);
    v.menu_rounding = Rounding::same(8.0);
    v.extreme_bg_color = INPUT_BG;
    v.code_bg_color = INPUT_BG;
    v.faint_bg_color = SURFACE_ALT;

    v.selection = egui::style::Selection {
        bg_fill: ACCENT,
        stroke: Stroke::new(1.0, ON_ACCENT),
    };
    v.hyperlink_color = ACCENT;
    v.warn_fg_color = WARNING;
    v.error_fg_color = DANGER;

    v.button_frame = true;
    v.striped = true;
    v.slider_trailing_fill = true;
    v.indent_has_left_vline = false;

    v.widgets.noninteractive = widget(SURFACE, SURFACE, BORDER, TEXT, 8.0);
    v.widgets.inactive = widget(SURFACE_ALT, SURFACE_ALT, BORDER, TEXT, 7.0);
    v.widgets.hovered = widget(
        SURFACE_ALT,
        Color32::from_rgb(34, 42, 57),
        BORDER_STRONG,
        Color32::WHITE,
        7.0,
    );
    v.widgets.active = widget(
        ACCENT,
        Color32::from_rgb(45, 54, 71),
        ACCENT,
        Color32::WHITE,
        7.0,
    );
    v.widgets.open = widget(
        SURFACE_ALT,
        Color32::from_rgb(34, 42, 57),
        ACCENT,
        Color32::WHITE,
        7.0,
    );

    v
}

fn widget(
    bg_fill: Color32,
    weak_bg_fill: Color32,
    border: Color32,
    text: Color32,
    radius: f32,
) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill,
        weak_bg_fill,
        bg_stroke: Stroke::new(1.0, border),
        rounding: Rounding::same(radius),
        fg_stroke: Stroke::new(1.0, text),
        expansion: 0.0,
    }
}
