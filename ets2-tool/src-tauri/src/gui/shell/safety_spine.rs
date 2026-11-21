//! Safety Spine: the always-visible save-safety authority.

use eframe::egui::{self, RichText};

use crate::gui::components;
use crate::gui::screens::{IssuesTab, Route, SubView};
use crate::gui::shell::{AppShell, ConfirmAction};
use crate::gui::theme;

pub fn render(shell: &mut AppShell, ctx: &egui::Context, compact: bool) {
    let width = if compact { 120.0 } else { 160.0 };
    egui::SidePanel::right("safety_spine")
        .resizable(false)
        .exact_width(width)
        .frame(
            egui::Frame::none()
                .fill(theme::SIDEBAR)
                .inner_margin(egui::Margin::symmetric(8.0, 10.0))
                .stroke(egui::Stroke::new(1.0, theme::BORDER)),
        )
        .show(ctx, |ui| render_body(shell, ui, compact));
}

fn render_body(shell: &mut AppShell, ui: &mut egui::Ui, compact: bool) {
    ui.label(
        RichText::new("SAFETY")
            .size(10.0)
            .strong()
            .color(theme::FAINT),
    );
    ui.add_space(6.0);

    let dirty = crate::gui::data::session::dirty_paths(&shell.data).len();
    let can_undo = !crate::gui::data::session::is_dirty(&shell.data) && shell.can_undo;
    let issue_count = shell.validation_issues.len() + shell.errors.len();

    // Backups gauge + browser entry point.
    components::status_gauge(
        ui,
        "Backups",
        &shell.backup_count.to_string(),
        &shell.backup_outcome,
        theme::ACCENT,
    );
    if components::button(ui, "Backups").clicked() {
        shell.navigate(Route::safety_backups_restore, Some(SubView::BackupList));
    }
    ui.add_space(6.0);

    // Staged gauge + explicit Save. Disabled at zero staged paths.
    components::status_gauge(
        ui,
        "Staged",
        &dirty.to_string(),
        if compact { "staged" } else { "staged file(s)" },
        if dirty == 0 { theme::MUTED } else { theme::WARNING },
    );
    let save = ui.add_enabled(
        dirty > 0,
        egui::Button::new(RichText::new("Save staged edits").size(11.0))
            .rounding(egui::Rounding::same(7.0))
            .min_size(egui::vec2(0.0, 28.0)),
    );
    if save.on_hover_text("Write staged save edits to disk (creates a backup first)").clicked() {
        shell.flush_pending_changes();
    }
    ui.add_space(6.0);

    // Issues + errors/logs.
    components::status_gauge(
        ui,
        "Issues",
        &issue_count.to_string(),
        if compact { "issues" } else { "validation + errors" },
        if issue_count == 0 { theme::MUTED } else { theme::DANGER },
    );
    let issues_button = ui.push_id("safety_issues_button", |ui| components::button(ui, "Issues"));
    if issues_button.inner.clicked() {
        shell.open_issues(IssuesTab::Validation);
    }
    if components::button(ui, "Errors & Logs").clicked() {
        shell.open_issues(IssuesTab::Errors);
    }
    ui.add_space(6.0);

    // Undo is a backup restore: disabled while staged, always confirmed.
    let undo_tooltip = if let Some(label) = &shell.undo_label {
        format!("Restore last backup: {label}")
    } else {
        "Restore last backup (disabled while edits are staged)".to_string()
    };
    let undo = ui.add_enabled(
        can_undo,
        egui::Button::new(RichText::new("Undo").size(11.0))
            .rounding(egui::Rounding::same(7.0))
            .min_size(egui::vec2(0.0, 28.0)),
    );
    if undo.on_hover_text(undo_tooltip).clicked() {
        shell.confirm = Some(ConfirmAction::UndoLast);
        shell.focus_index = 0;
        shell.focus_return = Some(egui::Id::new("safety_undo_button"));

    }
    ui.add_space(4.0);
    ui.label(
        RichText::new(&shell.last_save_outcome)
            .size(9.0)

            .wrap(),
    );
}
