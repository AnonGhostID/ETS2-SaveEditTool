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
use crate::gui::{Ets2NativeApp, ModsSubTab};
use crate::tauri::async_runtime;

pub fn render(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("🧩 Mod Profile Manager & Conflict Analyzer");
        if ui.button("⟳ Load State").clicked() {
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
        if ui.button("🔍 Quick Scan Mods").clicked() {
            match scan_mods_light(
                app.app.clone(),
                app.profile_state(),
                Some(app.selected_game.clone()),
            ) {
                Ok(state) => {
                    app.mod_presets = state.presets.clone();
                    app.mod_manager_state = Some(state);
                    app.set_status("Completed light mod scan.", false);
                }
                Err(e) => app.set_status(e, true),
            }
        }
        if ui.button("🔬 Deep Scan Mods").clicked() {
            match scan_mods_deep(
                app.app.clone(),
                app.profile_state(),
                Some(app.selected_game.clone()),
            ) {
                Ok(state) => {
                    app.mod_presets = state.presets.clone();
                    app.mod_manager_state = Some(state);
                    app.set_status("Completed deep mod scan.", false);
                }
                Err(e) => app.set_status(e, true),
            }
        }
        if ui.button("🖥 Open Steam Console").clicked() {
            if let Err(e) = open_steam_console() {
                app.set_status(e, true);
            }
        }
    });

    ui.horizontal(|ui| {
        for (sub, label) in [
            (ModsSubTab::ModManager, "📦 Discovered Mods"),
            (ModsSubTab::PresetsAndSandboxes, "🗂 Mod Presets & Sandboxes"),
            (ModsSubTab::ConflictAnalyzer, "🩺 Mod Conflict & Crash Analyzer"),
        ] {
            if ui.selectable_label(app.mods_sub_tab == sub, label).clicked() {
                app.mods_sub_tab = sub;
            }
        }
    });
    ui.separator();

    match app.mods_sub_tab {
        ModsSubTab::ModManager => render_mod_manager(app, ui),
        ModsSubTab::PresetsAndSandboxes => render_presets_and_sandboxes(app, ui),
        ModsSubTab::ConflictAnalyzer => render_conflict_analyzer(app, ui),
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
            let _ = clear_manual_workshop_directory(app.app.clone(), app.selected_game.clone());
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
                            let _ = open_workshop_page(wid.clone());
                        }
                    } else {
                        ui.label("-");
                    }
                    ui.end_row();
                }
            });
    } else {
        ui.label("Click 'Quick Scan Mods' or 'Deep Scan Mods' above to inspect your local and Steam Workshop mods.");
    }
}

fn render_presets_and_sandboxes(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    ui.columns(2, |cols| {
        cols[0].group(|ui| {
            ui.heading("Mod Presets");
            ui.horizontal(|ui| {
                if ui.button("⟳ Load Presets").clicked() {
                    app.mod_presets =
                        list_mod_presets(app.app.clone(), Some(app.selected_game.clone()))
                            .unwrap_or_default();
                }
                if ui.button("📥 Import Preset JSON").clicked() {
                    if let Ok(Some(_)) = import_mod_preset(app.app.clone()) {
                        app.mod_presets =
                            list_mod_presets(app.app.clone(), Some(app.selected_game.clone()))
                                .unwrap_or_default();
                        app.set_status("Imported mod preset.", false);
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
                            app.mod_presets =
                                list_mod_presets(app.app.clone(), Some(app.selected_game.clone()))
                                    .unwrap_or_default();
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
                        app.preset_compare_result = compare_mod_preset(
                            app.app.clone(),
                            app.profile_state(),
                            preset.id.clone(),
                            app.selected_game.clone(),
                        )
                        .ok();
                    }
                    if ui.small_button("Export").clicked() {
                        let _ = export_mod_preset(app.app.clone(), preset.id.clone());
                    }
                    if ui.small_button("Delete").clicked() {
                        let _ = delete_mod_preset(app.app.clone(), preset.id.clone());
                        app.mod_presets =
                            list_mod_presets(app.app.clone(), Some(app.selected_game.clone()))
                                .unwrap_or_default();
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

        cols[1].group(|ui| {
            ui.heading("Mod Sandboxes");
            if ui.button("⟳ Load Sandboxes").clicked() {
                app.mod_sandboxes = load_mod_sandboxes(app.app.clone()).ok();
            }
            ui.horizontal(|ui| {
                ui.label("Title:");
                ui.text_edit_singleline(&mut app.new_sandbox_title);
                if ui.button("Create Sandbox").clicked() && !app.new_sandbox_title.trim().is_empty()
                {
                    app.mod_sandboxes = create_mod_sandbox(
                        app.app.clone(),
                        app.new_sandbox_title.trim().to_string(),
                        app.new_sandbox_desc.trim().to_string(),
                    )
                    .ok();
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
                                app.mod_sandboxes =
                                    delete_mod_sandbox(app.app.clone(), sb.id.clone()).ok();
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Workshop ID / URL:");
                            ui.text_edit_singleline(&mut app.workshop_mod_input);
                            if ui.small_button("+ Add Mod").clicked()
                                && !app.workshop_mod_input.trim().is_empty()
                            {
                                app.mod_sandboxes = add_mod_to_sandbox(
                                    app.app.clone(),
                                    sb.id.clone(),
                                    app.workshop_mod_input.trim().to_string(),
                                    Some(true),
                                )
                                .ok();
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
        if ui.button("🩺 Run Standard Conflict Diagnostics").clicked() {
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
        ui.group(|ui| {
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
