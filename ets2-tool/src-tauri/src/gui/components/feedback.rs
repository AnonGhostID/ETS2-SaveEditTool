//! Transient feedback and overlay primitives: toasts, modal windows,
//! loading/empty/error states and the modal focus helper.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use eframe::egui::{self, RichText, Ui};

use crate::gui::theme;

/// Transient notification. Info toasts live 5 s, errors 10 s; the stack is
/// capped at four entries and supports manual dismissal.
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub is_error: bool,
    pub created: Instant,
}

/// Append a toast with last-message dedupe and a four-entry cap.
pub fn push_toast(toasts: &mut VecDeque<Toast>, message: impl Into<String>, is_error: bool) {
    let message = message.into();
    if let Some(last) = toasts.back() {
        if last.message == message && last.is_error == is_error {
            return;
        }
    }
    toasts.push_back(Toast {
        message,
        is_error,
        created: Instant::now(),
    });
    while toasts.len() > 4 {
        toasts.pop_front();
    }
}

/// Render the top-right toast stack, expire old entries and handle the
/// per-toast dismiss button.
pub fn toast_stack(ctx: &egui::Context, toasts: &mut VecDeque<Toast>) {
    let now = Instant::now();
    toasts.retain(|toast| {
        let ttl = if toast.is_error { 10 } else { 5 };
        now.duration_since(toast.created).as_secs() < ttl
    });
    if toasts.is_empty() {
        return;
    }
    ctx.request_repaint_after(Duration::from_millis(500));

    let mut dismiss = None;
    egui::Area::new("orbit_toast_stack".into())
        .anchor(egui::Align2::RIGHT_TOP, [-12.0, 12.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                for (index, toast) in toasts.iter().enumerate() {
                    egui::Frame::none()
                        .fill(theme::SURFACE)
                        .stroke(egui::Stroke::new(1.0, theme::BORDER))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.set_max_width(340.0);
                            ui.horizontal(|ui| {
                                super::status_dot(
                                    ui,
                                    if toast.is_error {
                                        theme::DANGER
                                    } else {
                                        theme::SUCCESS
                                    },
                                );
                                ui.label(RichText::new(&toast.message).size(12.0).color(theme::TEXT));
                                if ui.small_button("×").clicked() {
                                    dismiss = Some(index);
                                }
                            });
                        });
                    ui.add_space(6.0);
                }
            });
        });
    if let Some(index) = dismiss {
        toasts.remove(index);
    }
}

/// Generic loading row. Use inside the Detail Pane in place of the data view.
pub fn loading_state(ui: &mut Ui, message: &str) {
    super::progress_row(ui, message, None);
}

/// Generic error card with an optional Retry button. Callers must also append
/// the message to the Issues drawer; an error is never toast-only.
pub fn error_state(ui: &mut Ui, message: &str, retry_label: Option<&str>) -> bool {
    let mut retry = false;
    egui::Frame::none()
        .fill(theme::SURFACE)
        .stroke(egui::Stroke::new(1.0, theme::DANGER))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            ui.label(RichText::new(message).size(12.0).color(theme::TEXT));
            if let Some(label) = retry_label {
                ui.add_space(6.0);
                retry = super::button(ui, label).clicked();
            }
        });
    retry
}

/// Centered, non-resizable modal window. Overlays must wrap their buttons in
/// `ui.push_id(modal_focus_id(index), ...)` and call `trap_focus_step` from
/// `eframe::App::update` so Tab/Shift+Tab stay inside the overlay.
pub fn modal<R>(
    ctx: &egui::Context,
    title: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, add_contents)
        .map(|response| response.inner)
}

/// Stable focus id for overlay control `index` (zero-based, visual order).
pub fn modal_focus_id(index: usize) -> egui::Id {
    egui::Id::new(("orbit_modal_focus", index))
}

/// Consume Tab / Shift+Tab while an overlay is open and return the requested
/// focus step (-1 or +1). Escape is left intact for the overlay to interpret.
pub fn trap_focus_step(ctx: &egui::Context) -> i32 {
    let mut step = 0;
    ctx.input_mut(|input| {
        input.events.retain(|event| {
            if let egui::Event::Key {
                key: egui::Key::Tab,
                pressed: true,
                modifiers,
                ..
            } = event
            {
                step = if modifiers.shift { -1 } else { 1 };
                false
            } else {
                true
            }
        });
    });
    step
}
