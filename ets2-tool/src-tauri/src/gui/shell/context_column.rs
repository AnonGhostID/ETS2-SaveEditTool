//! Context Column (left, or a 40 px row in compact layouts).
//!
//! It is never a navigation authority: it only changes global save context,
//! and every change is guarded by `AppShell::request_context_change`.

use eframe::egui::{self, RichText, Ui};

use crate::gui::screens::ContextAction;
use crate::gui::shell::AppShell;
use crate::gui::theme;

pub fn render_expanded(shell: &mut AppShell, ctx: &egui::Context) {
    egui::SidePanel::left("context_column")
        .resizable(false)
        .exact_width(200.0)
        .frame(
            egui::Frame::none()
                .fill(theme::SIDEBAR)
                .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                .stroke(egui::Stroke::new(1.0, theme::BORDER)),
        )
        .show(ctx, |ui| render_body(shell, ui, ctx, false));
}

pub fn render_compact(shell: &mut AppShell, ctx: &egui::Context) {
    egui::TopBottomPanel::top("context_row")
        .resizable(false)
        .exact_height(40.0)
        .frame(
            egui::Frame::none()
                .fill(theme::SIDEBAR)
                .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                .stroke(egui::Stroke::new(1.0, theme::BORDER)),
        )
        .show(ctx, |ui| render_body(shell, ui, ctx, true));
}

fn render_body(shell: &mut AppShell, ui: &mut Ui, ctx: &egui::Context, compact: bool) {
    if !compact {
        ui.label(RichText::new("CONTEXT").size(10.0).strong().color(theme::FAINT));
        ui.add_space(6.0);
    }

    // GAME
    ui.label(RichText::new("GAME").size(9.0).strong().color(theme::FAINT));
    let current_game = shell
        .pending_game
        .clone()
        .unwrap_or_else(|| shell.selected_game.clone());
    let mut new_game = current_game.clone();
    egui::ComboBox::from_id_salt("context_game_selector")
        .selected_text(current_game.to_uppercase())
        .width(if compact { 62.0 } else { 150.0 })
        .show_ui(ui, |ui| {
            for option in &shell.game_options.clone() {
                if ui
                    .selectable_value(&mut new_game, option.clone(), option.to_uppercase())
                    .clicked()
                {
                    // selection captured below
                }
            }
        });
    if new_game != current_game {
        shell.pending_game = Some(new_game.clone());
        shell.request_context_change(ContextAction::SwitchGame(new_game));
    }

    // PROFILE
    ui.label(RichText::new("PROFILE").size(9.0).strong().color(theme::FAINT));
    let current_profile = shell
        .pending_profile
        .clone()
        .or_else(|| shell.selected_profile_path.clone());
    let profile_label = shell
        .profiles
        .iter()
        .find(|option| Some(&option.path) == current_profile.as_ref())
        .map(|option| option.label.clone())
        .or_else(|| current_profile.clone())
        .unwrap_or_else(|| "<No Profile>".to_string());
    let mut new_profile = current_profile.clone();
    let profile_options: Vec<(String, String)> = shell
        .profiles
        .iter()
        .map(|option| (option.path.clone(), option.label.clone()))
        .collect();
    egui::ComboBox::from_id_salt("context_profile_selector")
        .selected_text(profile_label)
        .width(if compact { 96.0 } else { 150.0 })
        .show_ui(ui, |ui| {
            for (path, label) in &profile_options {
                if ui
                    .selectable_value(&mut new_profile, Some(path.clone()), label.clone())
                    .clicked()
                {
                    // selection captured below
                }
            }
        });
    if new_profile != current_profile {
        if let Some(path) = new_profile.clone() {
            shell.pending_profile = Some(path.clone());
            shell.request_context_change(ContextAction::SwitchProfile(path));
        }
    }

    // SAVE
    ui.label(RichText::new("SAVE").size(9.0).strong().color(theme::FAINT));
    let current_save = shell
        .pending_save
        .clone()
        .or_else(|| shell.selected_save_path.clone());
    let save_label = shell
        .saves
        .iter()
        .find(|option| Some(&option.path) == current_save.as_ref())
        .map(|option| option.label.clone())
        .or_else(|| current_save.clone())
        .unwrap_or_else(|| "<No Save>".to_string());
    let mut new_save = current_save.clone();
    let save_options: Vec<(String, String)> = shell
        .saves
        .iter()
        .map(|option| (option.path.clone(), option.label.clone()))
        .collect();
    egui::ComboBox::from_id_salt("context_save_selector")
        .selected_text(save_label)
        .width(if compact { 96.0 } else { 150.0 })
        .show_ui(ui, |ui| {
            for (path, label) in &save_options {
                if ui
                    .selectable_value(&mut new_save, Some(path.clone()), label.clone())
                    .clicked()
                {
                    // selection captured below
                }
            }
        });
    if new_save != current_save {
        if let Some(path) = new_save.clone() {
            shell.pending_save = Some(path.clone());
            shell.request_context_change(ContextAction::SwitchSave(path));
        }
    }

    if ui.button(if compact { "Reload" } else { "⟳ Reload" }).clicked() {
        shell.request_context_change(ContextAction::Reload);
    }

    let language_label = if shell.language.is_empty() {
        "LANG —".to_string()
    } else {
        format!("LANG {}", shell.language.to_uppercase())
    };
    ui.label(RichText::new(language_label).size(9.5).color(theme::MUTED))
        .on_hover_text(format!(
            "Saved language preference. Available: {}",
            shell
                .available_languages
                .iter()
                .map(|(code, _)| code.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ));

    if compact {
        let profile = shell
            .profiles
            .iter()
            .find(|option| Some(&option.path) == shell.selected_profile_path.as_ref())
            .map(|option| option.label.clone())
            .unwrap_or_else(|| "no profile".to_string());
        let save = shell
            .saves
            .iter()
            .find(|option| Some(&option.path) == shell.selected_save_path.as_ref())
            .map(|option| option.label.clone())
            .unwrap_or_else(|| "no save".to_string());
        let fact = format!(
            "{} · {} · {} · {} staged",
            shell.selected_game.to_uppercase(),
            profile,
            save,
            shell.dirty_paths
        );
        ui.label(RichText::new(fact).size(10.0).color(theme::MUTED));
        if ui.small_button("Context").clicked() {
            shell.context_popover_open = !shell.context_popover_open;
        }
        if shell.context_popover_open {
            egui::Window::new("Context")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::LEFT_TOP, [8.0, 44.0])
                .show(ctx, |ui| {
                    render_facts(shell, ui);
                    if ui.small_button("Close").clicked() {
                        shell.context_popover_open = false;
                    }
                });
        }
    } else {
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);
        render_facts(shell, ui);
    }
}

fn render_facts(shell: &AppShell, ui: &mut Ui) {
    ui.label(RichText::new("CONTEXT FACTS").size(9.5).strong().color(theme::FAINT));
    let profile = shell
        .profiles
        .iter()
        .find(|option| Some(&option.path) == shell.selected_profile_path.as_ref())
        .map(|option| option.label.clone())
        .unwrap_or_else(|| "—".to_string());
    let save = shell
        .saves
        .iter()
        .find(|option| Some(&option.path) == shell.selected_save_path.as_ref())
        .map(|option| option.label.clone())
        .unwrap_or_else(|| "—".to_string());
    ui.label(RichText::new(format!("Profile: {profile}")).size(11.0).color(theme::TEXT));
    ui.label(RichText::new(format!("Save: {save}")).size(11.0).color(theme::TEXT));
    ui.label(
        RichText::new(format!("Staged: {}", shell.dirty_paths))
            .size(11.0)
            .color(if shell.dirty_paths == 0 { theme::MUTED } else { theme::WARNING }),
    );
    ui.label(
        RichText::new(format!(
            "Backups: {} ({})",
            shell.backup_count, shell.backup_outcome
        ))
        .size(11.0)
        .color(theme::MUTED),
    );
    ui.label(
        RichText::new(format!(
            "Staging: {}",
            if shell.staging_enabled { "on" } else { "off" }
        ))
        .size(11.0)
        .color(theme::MUTED),
    );
    ui.label(
        RichText::new("Save Health: see Safety › Save Health")
            .size(10.5)
            .color(theme::FAINT),
    );
}
