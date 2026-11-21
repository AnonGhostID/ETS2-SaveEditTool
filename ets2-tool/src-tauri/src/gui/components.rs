//! Small composable widgets shared by every workspace view.

pub mod data;
pub mod feedback;
pub mod form;

pub use data::*;
pub use feedback::*;
pub use form::*;

use eframe::egui::{
    self, Align2, Color32, FontId, Frame, Margin, Response, RichText, Rounding, Sense, Stroke, Ui,
};

use super::theme;

/// Card container used for every top-level panel in a tab.
pub trait UiExt {
    fn card<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R>;
}

impl UiExt for Ui {
    fn card<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
        Frame::none()
            .fill(theme::SURFACE)
            .stroke(Stroke::new(1.0_f32, theme::BORDER))
            .rounding(Rounding::same(10.0))
            .inner_margin(Margin::same(16.0))
            .show(self, add_contents)
    }
}

pub fn page_header(ui: &mut Ui, title: &str, subtitle: &str) {
    ui.label(
        RichText::new(title)
            .size(22.0)
            .strong()
            .color(theme::TEXT),
    );
    ui.add_space(3.0);
    ui.label(RichText::new(subtitle).size(12.5).color(theme::MUTED));
}

pub fn section_label(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .size(10.0)
            .strong()
            .color(theme::FAINT),
    );
}

pub fn nav_section(ui: &mut Ui, text: &str) {
    ui.add_space(12.0);
    section_label(ui, text);
    ui.add_space(4.0);
}

/// Full-width sidebar navigation row with an active indicator bar.
pub fn nav_item(ui: &mut Ui, label: &str, selected: bool) -> Response {
    let size = egui::vec2(ui.available_width(), 34.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let fill = if selected {
            theme::ACCENT_SOFT
        } else if response.hovered() {
            theme::SURFACE
        } else {
            Color32::TRANSPARENT
        };

        if fill != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, Rounding::same(7.0), fill);
        }

        if selected {
            let bar = egui::Rect::from_min_size(
                egui::pos2(rect.left() + 1.0, rect.center().y - 8.0),
                egui::vec2(2.0, 16.0),
            );
            ui.painter()
                .rect_filled(bar, Rounding::same(1.0), theme::ACCENT);
        }

        let fg = if selected { theme::ACCENT } else { theme::MUTED };
        ui.painter().text(
            egui::pos2(rect.left() + 14.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(13.5),
            fg,
        );
    }

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn primary_button(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        egui::Button::new(RichText::new(text).size(12.5).color(theme::ON_ACCENT))
            .fill(theme::ACCENT)
            .rounding(Rounding::same(7.0))
            .min_size(egui::vec2(0.0, 30.0)),
    )
}

/// Segmented sub-navigation. Returns the newly selected tab, if any.
pub fn sub_tabs<T: Copy + PartialEq>(ui: &mut Ui, current: T, items: &[(T, &str)]) -> Option<T> {
    let mut next = None;

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for &(tab, label) in items {
            if ui
                .selectable_label(current == tab, RichText::new(label).size(12.5))
                .clicked()
            {
                next = Some(tab);
            }
        }
    });

    next
}

/// A read-only stat tile. Fills the available width, so use it inside
/// `ui.columns()` (like the dashboard tiles) — NEVER inside
/// `horizontal_wrapped`: there the available width is the *remaining*
/// row width, so the first tile swallows the whole row and the rest
/// collapse to zero-width slivers (and egui does not wrap Frame-based
/// widgets in that row at all, so the extras overflow off-screen).
pub fn stat_tile(ui: &mut Ui, label: &str, value: &str, accent: Color32) {
    Frame::none()
        .fill(theme::SURFACE)
        .stroke(Stroke::new(1.0_f32, theme::BORDER))
        .rounding(Rounding::same(10.0))
        .inner_margin(Margin::symmetric(14.0, 12.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                RichText::new(label)
                    .size(9.5)
                    .strong()
                    .color(theme::MUTED),
            );
            ui.add_space(4.0);
            ui.label(RichText::new(value).size(19.0).strong().color(accent));
        });
}

/// A stat tile that acts as a button. Returns true when clicked.
pub fn clickable_stat_tile(ui: &mut Ui, label: &str, value: &str, accent: Color32) -> bool {
    let frame = Frame::none()
        .fill(theme::SURFACE)
        .stroke(Stroke::new(1.0f32, theme::BORDER))
        .rounding(Rounding::same(10.0))
        .inner_margin(Margin::symmetric(14.0, 12.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                RichText::new(label)
                    .size(9.5)
                    .strong()
                    .color(theme::MUTED),
            );
            ui.add_space(4.0);
            ui.label(RichText::new(value).size(19.0).strong().color(accent));
        });
    let response = ui.interact(
        frame.response.rect,
        egui::Id::new(("clickable_stat_tile", label)),
        Sense::click(),
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked()
}

pub fn action_card(ui: &mut Ui, title: &str, description: &str, button: &str) -> bool {
    let mut clicked = false;

    Frame::none()
        .fill(theme::SURFACE)
        .stroke(Stroke::new(1.0_f32, theme::BORDER))
        .rounding(Rounding::same(10.0))
        .inner_margin(Margin::same(16.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(104.0);
            ui.label(
                RichText::new(title)
                    .size(14.5)
                    .strong()
                    .color(theme::TEXT),
            );
            ui.add_space(3.0);
            ui.label(RichText::new(description).size(12.0).color(theme::MUTED));
            ui.add_space(10.0);
            clicked = primary_button(ui, button).clicked();
        });

    clicked
}

pub fn game_badge(ui: &mut Ui, game: &str) {
    Frame::none()
        .fill(theme::ACCENT_SOFT)
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::symmetric(9.0, 4.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(game.to_uppercase())
                    .size(10.0)
                    .strong()
                    .color(theme::ACCENT),
            );
        });
}

pub fn empty_state(ui: &mut Ui, title: &str, hint: &str) {
    Frame::none()
        .fill(theme::SURFACE)
        .stroke(Stroke::new(1.0_f32, theme::BORDER))
        .rounding(Rounding::same(10.0))
        .inner_margin(Margin::same(18.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(title)
                    .size(14.0)
                    .strong()
                    .color(theme::TEXT),
            );
            ui.add_space(2.0);
            ui.label(RichText::new(hint).size(12.0).color(theme::MUTED));
        });
}

pub fn status_dot(ui: &mut Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), 4.0, color);
}

/// Centered two-button confirmation dialog. Call each frame while the
/// dialog is open; returns `Some(true)` when the confirm button is
/// pressed, `Some(false)` for cancel, `None` while still open.
pub fn confirm_dialog(
    ctx: &egui::Context,
    title: &str,
    message: &str,
    confirm_text: &str,
    cancel_text: &str,
) -> Option<bool> {
    let mut choice = None;
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(message);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let confirm = ui.push_id(modal_focus_id(0), |ui| primary_button(ui, confirm_text));
                if confirm.inner.clicked() {
                    choice = Some(true);
                }
                let cancel = ui.push_id(modal_focus_id(1), |ui| ui.button(cancel_text));
                if cancel.inner.clicked() {
                    choice = Some(false);
                }
            });
        });
    choice
}
