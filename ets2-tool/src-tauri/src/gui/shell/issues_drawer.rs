//! Issues drawer: Validation, Errors, User logs and Recent activity tabs.

use eframe::egui::{self, RichText};

use crate::gui::components;
use crate::gui::screens::IssuesTab;
use crate::gui::shell::AppShell;
use crate::gui::theme;

const TABS: [IssuesTab; 4] = [
    IssuesTab::Validation,
    IssuesTab::Errors,
    IssuesTab::UserLog,
    IssuesTab::Activity,
];

pub fn render(shell: &mut AppShell, ctx: &egui::Context) {
    if shell.unsaved_prompt.is_none()
        && shell.confirm.is_none()
        && !shell.screen_modal_open
        && ctx.input(|input| input.key_pressed(egui::Key::Escape))
    {
        shell.issues_open = false;
        shell.restore_focus(ctx);
        return;
    }
    if shell.issues_tab == IssuesTab::UserLog && !shell.user_log_loaded {
        load_user_logs(shell);
    }

    egui::SidePanel::right("issues_drawer")
        .resizable(false)
        .exact_width(320.0)
        .frame(
            egui::Frame::none()
                .fill(theme::SIDEBAR)
                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                .stroke(egui::Stroke::new(1.0, theme::BORDER)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("ISSUES").size(11.0).strong().color(theme::TEXT));
                let close = ui.push_id(components::modal_focus_id(4), |ui| ui.small_button("×"));
                if close.inner.clicked() {
                    shell.issues_open = false;
                    shell.restore_focus(ctx);
                }
            });
            ui.add_space(6.0);

            ui.horizontal_wrapped(|ui| {
                for (index, tab) in TABS.iter().enumerate() {
                    let response = ui.push_id(components::modal_focus_id(index), |ui| {
                        ui.selectable_label(shell.issues_tab == *tab, tab_label(*tab))
                    });
                    if response.inner.clicked() {
                        shell.issues_tab = *tab;
                        shell.focus_index = index;
                        if *tab == IssuesTab::UserLog && !shell.user_log_loaded {
                            load_user_logs(shell);
                        }
                    }
                }
            });
            ui.add_space(8.0);
            egui::ScrollArea::vertical()
                .id_salt("issues_content")
                .auto_shrink([false, false])
                .show(ui, |ui| match shell.issues_tab {
                    IssuesTab::Validation => render_list(
                        ui,
                        &shell.validation_issues,
                        "No validation messages",
                        "Inline validation appears here as well as beside the field.",
                    ),
                    IssuesTab::Errors => render_list(
                        ui,
                        &shell.errors,
                        "No errors recorded",
                        "Backend failures and dismissed error toasts stay here.",
                    ),
                    IssuesTab::UserLog => {
                        let mut filter = shell.user_log_filter.clone();
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Filter").size(10.0).color(theme::MUTED));
                            ui.add(
                                egui::TextEdit::singleline(&mut filter)
                                    .hint_text("contains…")
                                    .desired_width(140.0),
                            );
                        });
                        shell.user_log_filter = filter;
                        let needle = shell.user_log_filter.clone();
                        let visible: Vec<&String> = shell
                            .user_logs
                            .iter()
                            .filter(|line| needle.is_empty() || line.contains(&needle))
                            .collect();
                        if visible.is_empty() {
                            components::empty_state(
                                ui,
                                "No user log entries",
                                "Log output from the backend appears here.",
                            );
                        } else {
                            for line in visible {
                                ui.label(RichText::new(line).size(11.0).color(theme::TEXT));
                            }
                        }
                        if ui.small_button("Refresh logs").clicked() {
                            load_user_logs(shell);
                        }
                    }
                    IssuesTab::Activity => {
                        if ui.small_button("Open full log view").clicked() {
                            shell.issues_open = false;
                            shell.navigate(
                                crate::gui::screens::Route::system_logs_diagnostics,
                                Some(crate::gui::screens::SubView::LogsList),
                            );
                        }

                        if shell.recent_events.is_empty() {
                            components::empty_state(
                                ui,
                                "No activity recorded",
                                "Background events appear here as they arrive.",
                            );
                        } else {
                            for (event, payload) in shell.recent_events.iter().rev().take(50) {
                                ui.label(
                                    RichText::new(event).size(10.5).strong().color(theme::ACCENT),
                                );
                                ui.label(RichText::new(payload).size(10.5).color(theme::MUTED));
                                ui.add_space(4.0);
                            }
                        }
                    }
                });
        });
}

fn tab_label(tab: IssuesTab) -> &'static str {
    match tab {
        IssuesTab::Validation => "Validation",
        IssuesTab::Errors => "Errors",
        IssuesTab::UserLog => "User logs",
        IssuesTab::Activity => "Activity",
    }
}

fn render_list(ui: &mut egui::Ui, items: &[String], empty_title: &str, empty_hint: &str) {
    if items.is_empty() {
        components::empty_state(ui, empty_title, empty_hint);
    } else {
        for item in items {
            ui.label(RichText::new(item).size(11.0).color(theme::TEXT));
            ui.add_space(3.0);
        }
    }
}

fn load_user_logs(shell: &mut AppShell) {
    match crate::gui::data::system::user_logs(&shell.data) {
        Ok(entries) => {
            shell.user_logs = entries
                .into_iter()
                .map(|entry| {
                    format!(
                        "{} {} {} — {}",
                        entry.timestamp, entry.level, entry.module, entry.message
                    )
                })
                .collect();
        }
        Err(err) => shell.push_error(format!("User logs unavailable: {}", err.message)),
    }
    shell.user_log_loaded = true;
}
