//! Form primitives: buttons, text inputs, selects, tabs and labelled fields.
//!
//! All widgets are layout-agnostic; the caller decides the surrounding
//! `Grid`, `Ui`, or `ScrollArea`.

use eframe::egui::{self, Response, RichText, Ui};

use crate::gui::theme;

/// Secondary action button (outline). Use `super::primary_button` for the
/// single primary action in a pane.
pub fn button(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        egui::Button::new(RichText::new(text).size(12.5))
            .rounding(egui::Rounding::same(7.0))
            .min_size(egui::vec2(0.0, 28.0)),
    )
}

/// Danger-tinted action button. Confirmation is still the caller's job.
pub fn danger_button(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        egui::Button::new(RichText::new(text).size(12.5).color(theme::DANGER))
            .rounding(egui::Rounding::same(7.0))
            .min_size(egui::vec2(0.0, 28.0)),
    )
}

/// Single-line text input with a placeholder hint. Returns the text response,
/// so `changed()` works as expected.
pub fn text_input(ui: &mut Ui, value: &mut String, hint: &str) -> Response {
    ui.add(
        egui::TextEdit::singleline(value)
            .hint_text(hint)
            .desired_width(ui.available_width().min(360.0)),
    )
}

/// Multi-line text input for notes and previews.
pub fn text_area(ui: &mut Ui, value: &mut String, hint: &str, rows: usize) -> Response {
    ui.add(
        egui::TextEdit::multiline(value)
            .hint_text(hint)
            .desired_rows(rows)
            .desired_width(ui.available_width().min(480.0)),
    )
}

/// String-valued combo box. `id` must be unique within the window.
/// Returns `true` when a different option was selected.
pub fn select(ui: &mut Ui, id: &str, current: &mut String, options: &[String]) -> bool {
    let mut changed = false;
    egui::ComboBox::from_id_salt(id)
        .selected_text(current.clone())
        .show_ui(ui, |ui| {
            for option in options {
                if ui
                    .selectable_value(current, option.clone(), option.clone())
                    .clicked()
                {
                    changed = true;
                }
            }
        });
    changed
}

/// Boolean toggle with a stable id and a descriptive label.
pub fn toggle(ui: &mut Ui, id: &str, label: &str, value: &mut bool) -> Response {
    let response = ui.add(egui::Checkbox::new(value, label));
    let _ = id; // checkbox derives its id from the label; ids are documented for callers
    response
}

/// Segmented tabs; alias of `super::sub_tabs` for new screens.
pub fn tabs<T: Copy + PartialEq>(
    ui: &mut Ui,
    current: T,
    items: &[(T, &str)],
) -> Option<T> {
    super::sub_tabs(ui, current, items)
}

/// Labelled field. The label is rendered above the widget, never as a
/// placeholder alone, so it survives compact layouts.
pub fn form_field<R>(
    ui: &mut Ui,
    label: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).size(10.5).color(theme::MUTED));
        let response = add_contents(ui);
        ui.add_space(3.0);
        response
    })
    .inner
}

/// Inline validation message. Renders nothing when `message` is empty.
pub fn validation_message(ui: &mut Ui, message: &str) {
    if !message.is_empty() {
        ui.label(RichText::new(message).size(11.5).color(theme::WARNING));
    }
}
