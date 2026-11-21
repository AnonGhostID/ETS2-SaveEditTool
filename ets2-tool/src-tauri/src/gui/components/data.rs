//! Read-only data display primitives: tables, sections, stat tiles and gauges.

use eframe::egui::{self, Color32, RichText, Ui};

use crate::gui::theme;

use super::UiExt;

/// Compact selectable table. One row is selectable by clicking any of its
/// cells; returns the clicked row index. Uses `egui::Grid` with a fixed
/// header row so narrow layouts never scroll horizontally.
pub fn table(
    ui: &mut Ui,
    id: &str,
    headers: &[&str],
    rows: &[Vec<String>],
    selected: Option<usize>,
) -> Option<usize> {
    let mut clicked = None;
    egui::ScrollArea::vertical()
        .id_salt(id)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            egui::Grid::new(id)
                .striped(true)
                .min_col_width(78.0)
                .show(ui, |ui| {
                    for header in headers {
                        ui.label(RichText::new(*header).size(10.5).strong().color(theme::MUTED));
                    }
                    ui.end_row();

                    for (index, row) in rows.iter().enumerate() {
                        for cell in row {
                            if ui
                                .selectable_label(selected == Some(index), cell.as_str())
                                .clicked()
                            {
                                clicked = Some(index);
                            }
                        }
                        ui.end_row();
                    }
                });
        });
    clicked
}

/// Section card with a small caption and caller-supplied body.
pub fn section_card<R>(
    ui: &mut Ui,
    title: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    ui.card(|ui| {
        super::section_label(ui, title);
        ui.add_space(4.0);
        add_contents(ui)
    })
}

/// Fixed-size stat tile that does not consume the whole row; suitable for
/// `horizontal_wrapped` grids in the new shell.
pub fn compact_stat_tile(ui: &mut Ui, label: &str, value: &str, accent: Color32) -> egui::Response {
    let response = egui::Frame::none()
        .fill(theme::SURFACE)
        .stroke(egui::Stroke::new(1.0_f32, theme::BORDER))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.label(RichText::new(label).size(9.5).color(theme::MUTED));
            ui.label(RichText::new(value).size(16.0).strong().color(accent));
        })
        .response;
    response.on_hover_cursor(egui::CursorIcon::Default)
}

/// Safety Spine gauge chip. `detail` is the second line (for example
/// `last OK` or `3 staged`); compact layouts use the same widget.
pub fn status_gauge(ui: &mut Ui, label: &str, value: &str, detail: &str, accent: Color32) {
    egui::Frame::none()
        .fill(theme::SURFACE)
        .stroke(egui::Stroke::new(1.0_f32, theme::BORDER))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::symmetric(9.0, 7.0))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(label).size(9.0).color(theme::FAINT));
                ui.label(RichText::new(value).size(14.0).strong().color(accent));
                if !detail.is_empty() {
                    ui.label(RichText::new(detail).size(9.5).color(theme::MUTED));
                }
            });
        });
}

/// Progress row for background loads. Keeps its position while loading and
/// after completion, so focus order never shifts.
pub fn progress_row(ui: &mut Ui, message: &str, fraction: Option<f32>) {
    ui.horizontal(|ui| {
        ui.spinner();
        ui.label(RichText::new(message).size(11.5).color(theme::MUTED));
        if let Some(fraction) = fraction {
            ui.add(
                egui::ProgressBar::new(fraction.clamp(0.0, 1.0))
                    .desired_width(80.0)
                    .show_percentage(),
            );
        }
    });
}
