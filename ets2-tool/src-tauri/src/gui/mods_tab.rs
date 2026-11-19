use eframe::egui::{self, Color32};

use crate::features::mod_profile_manager::commands::{
    add_mod_to_sandbox, apply_sandbox_to_active_profile, clear_manual_workshop_directory,
    compare_mod_preset, create_mod_preset, create_mod_sandbox, delete_mod_preset,
    delete_mod_sandbox, export_mod_preset, import_mod_preset, list_mod_presets,
    load_mod_profile_manager_state, load_mod_sandboxes, open_steam_console, open_workshop_page,
    scan_mods_deep, scan_mods_light, select_manual_workshop_directory,
};
use crate::features::save_analysis::commands::{
    analyze_mod_conflict_diagnostics, analyze_mod_conflict_diagnostics_deep,
    export_mod_conflict_diagnostics_report,
};
use crate::gui::components::{self, UiExt};
use crate::gui::{Ets2NativeApp, ModsSubTab};
use crate::state::AppProfileState;
use crate::tauri::async_runtime;

/// A destructive mod action waiting on the confirm dialog.
#[derive(Debug, Clone)]
pub enum ModConfirm {
    DeletePreset { id: String, name: String },
    DeleteSandbox { id: String, title: String },
}

pub fn render(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    let (title, subtitle) = match app.mods_sub_tab {
        ModsSubTab::ModManager => (
            "Mod Manager",
            "Inspect installed mods and apply them to the active profile.",
        ),
        ModsSubTab::PresetsAndSandboxes => (
            "Presets & Sandboxes",
            "Saved mod sets and isolated test profiles.",
        ),
        ModsSubTab::ConflictAnalyzer => (
            "Conflict Analyzer",
            "Diagnose crashes and conflicts between mods.",
        ),
    };
    components::page_header(ui, title, subtitle);
    ui.add_space(8.0);

    // Pick up a finished background scan, if one arrived.
    if let Some(result) = app.mods_scan_load.poll() {
        match result {
            Ok(state) => {
                app.mod_presets = state.presets.clone();
                app.mod_manager_state = Some(state);
                app.set_status("Completed mod scan.", false);
            }
            Err(e) => app.set_status(e, true),
        }
    }
    if app.mods_scan_load.is_loading() {
        // Keep repainting so the spinner animates and the result is
        // picked up promptly.
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(100));
    }

    ui.horizontal_wrapped(|ui| {
        if components::primary_button(ui, "Load State").clicked() {
            match load_mod_profile_manager_state(
                app.app.clone(),
                app.profile_state(),
                Some(app.selected_game.clone()),
            ) {
                Ok(state) => {
                    app.mod_presets = state.presets.clone();
                    app.mod_manager_state = Some(state);
                    app.set_status("Loaded mod manager state.", false);
                }
                Err(e) => app.set_status(e, true),
            }
        }
        let scanning = app.mods_scan_load.is_loading();
        if ui
            .add_enabled(!scanning, egui::Button::new("Quick Scan Mods"))
            .clicked()
        {
            start_mods_scan(app, false);
        }
        if ui
            .add_enabled(!scanning, egui::Button::new("Deep Scan Mods"))
            .clicked()
        {
            start_mods_scan(app, true);
        }
        if scanning {
            ui.spinner();
            ui.label("Scanning mods…");
        }
        if ui.button("Open Steam Console").clicked() {
            if let Err(e) = open_steam_console() {
                app.set_status(e, true);
            }
        }
    });
    ui.add_space(8.0);

    match app.mods_sub_tab {
        ModsSubTab::ModManager => render_mod_manager(app, ui),
        ModsSubTab::PresetsAndSandboxes => render_presets_and_sandboxes(app, ui),
        ModsSubTab::ConflictAnalyzer => render_conflict_analyzer(app, ui),
    }

    render_mod_confirm(app, ui);
}

fn start_mods_scan(app: &mut Ets2NativeApp, deep: bool) {
    let handle = app.app.clone();
    let game = Some(app.selected_game.clone());
    app.mods_scan_load.start(move || {
        let profile = handle.state::<AppProfileState>();
        if deep {
            scan_mods_deep(handle.clone(), profile, game)
        } else {
            scan_mods_light(handle.clone(), profile, game)
        }
    });
}

fn render_mod_confirm(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    let Some(confirm) = app.mod_confirm.clone() else {
        return;
    };
    let (title, message) = match &confirm {
        ModConfirm::DeletePreset { name, .. } => (
            "Delete preset",
            format!("Delete the mod preset \"{name}\"? This cannot be undone."),
        ),
        ModConfirm::DeleteSandbox { title, .. } => (
            "Delete sandbox",
            format!("Delete the sandbox \"{title}\" and its mod list? This cannot be undone."),
        ),
    };
    match components::confirm_dialog(ui.ctx(), title, &message, "Delete", "Cancel") {
        Some(true) => {
            app.mod_confirm = None;
            match confirm {
                ModConfirm::DeletePreset { id, .. } => {
                    // P6/P7: a failed delete used to report success.
                    match delete_mod_preset(app.app.clone(), id) {
                        Ok(()) => {
                            reload_preset_list(app);
                            app.set_status("Deleted mod preset.", false);
                        }
                        Err(e) => app.set_status(format!("Delete failed: {e}"), true),
                    }
                }
                ModConfirm::DeleteSandbox { id, .. } => {
                    // P6/P7: a failed delete used to report success and
                    // wipe the displayed collection.
                    match delete_mod_sandbox(app.app.clone(), id) {
                        Ok(collection) => {
                            app.mod_sandboxes = Some(collection);
                            app.set_status("Deleted sandbox.", false);
                        }
                        Err(e) => app.set_status(format!("Delete failed: {e}"), true),
                    }
                }
            }
        }
        Some(false) => app.mod_confirm = None,
        None => {}
    }
}

/// Reload the preset list, reporting a failure instead of silently
/// showing an empty list (P6/P7).
fn reload_preset_list(app: &mut Ets2NativeApp) {
    match list_mod_presets(app.app.clone(), Some(app.selected_game.clone())) {
        Ok(list) => app.mod_presets = list,
        Err(e) => app.set_status(format!("Preset list reload failed: {e}"), true),
    }
}

fn render_mod_manager(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("📂 Set Manual Workshop Folder").clicked() {
            match select_manual_workshop_directory(app.app.clone(), app.selected_game.clone()) {
                Ok(Some(path)) => app.set_status(format!("Manual Workshop folder: {path}"), false),
                Ok(None) => {}
                Err(e) => app.set_status(e, true),
            }
        }
        if ui.button("✖ Clear Manual Workshop Folder").clicked() {
            // P6/P7: surface the result instead of discarding it.
            match clear_manual_workshop_directory(app.app.clone(), app.selected_game.clone()) {
                Ok(()) => app.set_status("Cleared manual Workshop folder.", false),
                Err(e) => app.set_status(e, true),
            }
        }
    });

    if let Some(state) = &app.mod_manager_state {
        ui.label(format!(
            "Local Mods: {} | Steam Workshop Mods: {} | Active in Profile: {} (Source: {})",
            state.summary.local_mods_found,
            state.summary.steam_workshop_mods_found,
            state.summary.active_mods_count,
            state.summary.load_order_source
        ));
        ui.separator();

        let mut workshop_error: Option<String> = None;
        egui::Grid::new("discovered_mods_grid")
            .striped(true)
            .num_columns(7)
            .show(ui, |ui| {
                ui.strong("Order");
                ui.strong("Mod Name");
                ui.strong("Source");
                ui.strong("Version / Author");
                ui.strong("Status");
                ui.strong("Workshop ID");
                ui.strong("Action");
                ui.end_row();

                for m in &state.mods {
                    ui.label(
                        m.load_order_index
                            .map(|i| i.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    );
                    ui.label(&m.name);
                    ui.label(format!("{:?}", m.source));
                    ui.label(format!(
                        "{} / {}",
                        m.version.as_deref().unwrap_or("-"),
                        m.author.as_deref().unwrap_or("-")
                    ));
                    ui.label(&m.status);
                    ui.label(m.workshop_id.as_deref().unwrap_or("-"));
                    if let Some(wid) = &m.workshop_id {
                        if ui.small_button("Steam Workshop").clicked() {
                            if let Err(e) = open_workshop_page(wid.clone()) {
                                workshop_error = Some(e);
                            }
                        }
                    } else {
                        ui.label("-");
                    }
                    ui.end_row();
                }
            });
        if let Some(e) = workshop_error {
            app.set_status(e, true);
        }
    } else {
        ui.label("Click 'Quick Scan Mods' or 'Deep Scan Mods' above to inspect your local and Steam Workshop mods.");
    }
}

fn render_presets_and_sandboxes(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    ui.columns(2, |cols| {
        cols[0].card(|ui| {
            ui.heading("Mod Presets");
            ui.horizontal(|ui| {
                if ui.button("⟳ Load Presets").clicked() {
                    reload_preset_list(app);
                }
                if ui.button("📥 Import Preset JSON").clicked() {
                    // P6/P7: surface import failures, and say which
                    // game the preset belongs to — the list below is
                    // filtered to the selected game, so an imported
                    // preset for the other game would otherwise seem
                    // to vanish.
                    match import_mod_preset(app.app.clone()) {
                        Ok(Some(preset)) => {
                            let game_label = preset.game.display_name();
                            let preset_name = preset.name.clone();
                            let imported_game = preset.game.as_str();
                            reload_preset_list(app);
                            if imported_game == app.selected_game.as_str() {
                                app.set_status(
                                    format!("Imported mod preset `{preset_name}`."),
                                    false,
                                );
                            } else {
                                app.set_status(
                                    format!(
                                        "Imported mod preset `{preset_name}` for {game_label} — switch the game in the top bar to see it."
                                    ),
                                    false,
                                );
                            }
                        }
                        Ok(None) => {}
                        Err(e) => app.set_status(e, true),
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("New Preset Name:");
                ui.text_edit_singleline(&mut app.new_preset_name);
                if ui.button("Save Current as Preset").clicked()
                    && !app.new_preset_name.trim().is_empty()
                {
                    match create_mod_preset(
                        app.app.clone(),
                        app.profile_state(),
                        app.new_preset_name.trim().to_string(),
                        app.selected_game.clone(),
                        if app.new_preset_notes.trim().is_empty() {
                            None
                        } else {
                            Some(app.new_preset_notes.trim().to_string())
                        },
                        None,
                    ) {
                        Ok(_) => {
                            reload_preset_list(app);
                            app.set_status("Saved mod preset.", false);
                        }
                        Err(e) => app.set_status(e, true),
                    }
                }
            });

            ui.separator();
            for preset in app.mod_presets.clone() {
                ui.horizontal(|ui| {
                    ui.label(format!("{} ({} mods)", preset.name, preset.mods.len()));
                    if ui.small_button("Compare").clicked() {
                        // P6/P7: a failed compare used to silently clear
                        // the old diff; say so.
                        match compare_mod_preset(
                            app.app.clone(),
                            app.profile_state(),
                            preset.id.clone(),
                            app.selected_game.clone(),
                        ) {
                            Ok(result) => app.preset_compare_result = Some(result),
                            Err(e) => {
                                app.preset_compare_result = None;
                                app.set_status(format!("Compare failed: {e}"), true);
                            }
                        }
                    }
                    if ui.small_button("Export").clicked() {
                        // P6/P7: surface the result instead of discarding
                        // it (an empty path means the user cancelled the
                        // file picker).
                        match export_mod_preset(app.app.clone(), preset.id.clone()) {
                            Ok(path) if !path.is_empty() => {
                                app.set_status(format!("Exported preset to {path}."), false)
                            }
                            Ok(_) => {}
                            Err(e) => app.set_status(e, true),
                        }
                    }
                    if ui.small_button("Delete").clicked() {
                        app.mod_confirm = Some(ModConfirm::DeletePreset {
                            id: preset.id.clone(),
                            name: preset.name.clone(),
                        });
                    }
                });
            }

            if let Some(cmp) = &app.preset_compare_result {
                ui.separator();
                ui.label(format!(
                    "Compare `{}`: Missing: {} | Extra: {} | Load Order Diffs: {}",
                    cmp.preset.name,
                    cmp.summary.missing_mods_count,
                    cmp.summary.extra_mods_count,
                    cmp.summary.load_order_differences_count
                ));
            }
        });

        cols[1].card(|ui| {
            ui.heading("Mod Sandboxes");
            if ui.button("⟳ Load Sandboxes").clicked() {
                // P6/P7: surface load failures instead of silently
                // showing an empty list.
                match load_mod_sandboxes(app.app.clone()) {
                    Ok(collection) => app.mod_sandboxes = Some(collection),
                    Err(e) => app.set_status(format!("Sandbox load failed: {e}"), true),
                }
            }
            ui.horizontal(|ui| {
                ui.label("Title:");
                ui.text_edit_singleline(&mut app.new_sandbox_title);
                if ui.button("Create Sandbox").clicked() && !app.new_sandbox_title.trim().is_empty()
                {
                    // P6/P7: surface create failures instead of silently
                    // dropping the collection.
                    match create_mod_sandbox(
                        app.app.clone(),
                        app.new_sandbox_title.trim().to_string(),
                        app.new_sandbox_desc.trim().to_string(),
                    ) {
                        Ok(collection) => {
                            app.mod_sandboxes = Some(collection);
                            app.set_status("Created sandbox.", false);
                        }
                        Err(e) => app.set_status(format!("Create sandbox failed: {e}"), true),
                    }
                }
            });

            if let Some(coll) = app.mod_sandboxes.clone() {
                for sb in coll.sandboxes {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(&sb.title);
                            ui.label(format!("({} mods)", sb.mods.len()));
                            if ui.small_button("Apply to Active Profile").clicked() {
                                match apply_sandbox_to_active_profile(
                                    app.app.clone(),
                                    app.profile_state(),
                                    sb.id.clone(),
                                    false,
                                ) {
                                    Ok(res) => app.set_status(res.message, !res.success),
                                    Err(e) => app.set_status(e, true),
                                }
                            }
                            if ui.small_button("Delete").clicked() {
                                app.mod_confirm = Some(ModConfirm::DeleteSandbox {
                                    id: sb.id.clone(),
                                    title: sb.title.clone(),
                                });
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Workshop ID / URL:");
                            ui.text_edit_singleline(&mut app.workshop_mod_input);
                            if ui.small_button("+ Add Mod").clicked()
                                && !app.workshop_mod_input.trim().is_empty()
                            {
                                // P6/P7: surface add failures instead of
                                // silently dropping the collection.
                                match add_mod_to_sandbox(
                                    app.app.clone(),
                                    sb.id.clone(),
                                    app.workshop_mod_input.trim().to_string(),
                                    Some(true),
                                ) {
                                    Ok(collection) => {
                                        app.mod_sandboxes = Some(collection);
                                        app.set_status("Added mod to sandbox.", false);
                                    }
                                    Err(e) => {
                                        app.set_status(format!("Add mod failed: {e}"), true)
                                    }
                                }
                            }
                        });
                    });
                }
            }
        });
    });
}

fn render_conflict_analyzer(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("Run Standard Conflict Diagnostics").clicked() {
            match async_runtime::block_on(analyze_mod_conflict_diagnostics(
                app.profile_state(),
                app.decrypt_cache(),
            )) {
                Ok(rep) => {
                    app.conflict_report = Some(rep);
                    app.set_status("Completed mod conflict analysis.", false);
                }
                Err(e) => app.set_status(e, true),
            }
        }
        if ui.button("🔬 Run Deep Conflict Diagnostics").clicked() {
            match async_runtime::block_on(analyze_mod_conflict_diagnostics_deep(
                app.profile_state(),
                app.decrypt_cache(),
            )) {
                Ok(rep) => {
                    app.conflict_report = Some(rep);
                    app.set_status("Completed deep mod conflict analysis.", false);
                }
                Err(e) => app.set_status(e, true),
            }
        }
        if let Some(rep) = app.conflict_report.clone() {
            if ui.button("📤 Export Diagnostic Report").clicked() {
                match export_mod_conflict_diagnostics_report(app.app.clone(), rep, Some(true)) {
                    Ok(Some(path)) => app.set_status(format!("Exported report to {path}"), false),
                    Ok(None) => {}
                    Err(e) => app.set_status(e, true),
                }
            }
        }
    });

    if let Some(rep) = &app.conflict_report {
        ui.card(|ui| {
            ui.heading(format!(
                "[{}] {}",
                rep.overview.status_badge, rep.overview.summary
            ));
            ui.label(&rep.overview.confidence_note);
            ui.label(format!(
                "Crash Detected: {} | Errors: {} | Warnings: {} | Indexed Mods: {} | Active Mods: {}",
                rep.crash_summary.crash_detected,
                rep.crash_summary.error_count,
                rep.crash_summary.warning_count,
                rep.sources.indexed_mods_count,
                rep.sources.active_mods_count
            ));
        });

        ui.add_space(6.0);
        ui.heading(format!("Suspected Mods ({})", rep.suspected_mods.len()));
        egui::Grid::new("suspected_mods_grid")
            .striped(true)
            .num_columns(5)
            .show(ui, |ui| {
                ui.strong("Score");
                ui.strong("Confidence");
                ui.strong("Mod Name");
                ui.strong("Active State");
                ui.strong("Reasons");
                ui.end_row();
                for s in &rep.suspected_mods {
                    ui.colored_label(Color32::from_rgb(245, 176, 65), format!("{}", s.score));
                    ui.label(&s.confidence);
                    ui.label(&s.name);
                    ui.label(&s.active_state);
                    ui.label(s.reasons.join("; "));
                    ui.end_row();
                }
            });
    }
}
