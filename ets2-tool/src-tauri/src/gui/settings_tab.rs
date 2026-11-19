use eframe::egui;

use crate::commands::data_import_ets2_datasets;
use crate::features::logging::commands::{
    clear_user_logs, export_logs_bundle, export_user_logs, get_log_status, get_user_logs,
};
use crate::features::logging::diagnostics::{export_log_file, get_log_preview};
use crate::features::settings::ets2_paths::{
    choose_ets2_data_location, reset_ets2_data_location, set_ets2_data_location_mode,
};
use crate::gui::components::{self, UiExt};
use crate::gui::Ets2NativeApp;
use crate::tauri::async_runtime;

pub fn render(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    components::page_header(
        ui,
        "Settings",
        "Game paths, local SQLite datasets, logs and diagnostics.",
    );
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        if components::primary_button(ui, "Refresh Status").clicked() {
            app.refresh_settings_and_location();
            app.log_status = get_log_status().ok();
            app.user_logs = get_user_logs(None, Some(100)).unwrap_or_default();
            app.diagnostic_log_preview =
                async_runtime::block_on(get_log_preview(app.diagnostics_state()))
                    .unwrap_or_default();
        }
    });
    ui.add_space(8.0);

    ui.columns(2, |cols| {
        cols[0].card(|ui| {
            ui.heading("ETS2 / ATS Homedir & Data Location");
            if let Some(loc) = &app.data_location {
                ui.label(format!("Mode: {:?}", loc.mode));
                ui.label(format!(
                    "Game Data Dir: {}",
                    loc.game_data_dir.as_deref().unwrap_or("-")
                ));
                ui.label(format!(
                    "Profiles Dir: {} (Exists: {})",
                    loc.profiles_dir.as_deref().unwrap_or("-"),
                    loc.validation.profiles_dir_exists
                ));
                ui.label(format!(
                    "Local Mod Dir: {} (Exists: {})",
                    loc.local_mod_dir.as_deref().unwrap_or("-"),
                    loc.validation.local_mod_dir_exists
                ));
                ui.label(format!(
                    "Config File: {}",
                    loc.config_file.as_deref().unwrap_or("-")
                ));
            }

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("📂 Browse Custom ETS2 Homedir...").clicked() {
                    match choose_ets2_data_location(
                        app.app.clone(),
                        None,
                        app.profile_state(),
                        app.decrypt_cache(),
                        app.profile_cache(),
                    ) {
                        Ok(Some(loc)) => {
                            app.data_location = Some(loc);
                            app.refresh_profiles_and_saves();
                            app.set_status("Updated custom ETS2 data directory.", false);
                        }
                        Ok(None) => {}
                        Err(e) => app.set_status(e, true),
                    }
                }
                if ui.button("↺ Reset to Default Location").clicked() {
                    match reset_ets2_data_location(
                        app.app.clone(),
                        app.profile_state(),
                        app.decrypt_cache(),
                        app.profile_cache(),
                    ) {
                        Ok(loc) => {
                            app.data_location = Some(loc);
                            app.refresh_profiles_and_saves();
                            app.set_status("Reset ETS2 data location to OS default.", false);
                        }
                        Err(e) => app.set_status(e, true),
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Manual Homedir Path:");
                ui.text_edit_singleline(&mut app.custom_homedir_input);
                if ui.button("Set Custom Path").clicked()
                    && !app.custom_homedir_input.trim().is_empty()
                {
                    match set_ets2_data_location_mode(
                        app.app.clone(),
                        "custom".to_string(),
                        Some(app.custom_homedir_input.trim().to_string()),
                        app.profile_state(),
                        app.decrypt_cache(),
                        app.profile_cache(),
                    ) {
                        Ok(loc) => {
                            app.data_location = Some(loc);
                            app.refresh_profiles_and_saves();
                            app.set_status("Applied custom homedir path.", false);
                        }
                        Err(e) => app.set_status(e, true),
                    }
                }
            });

            ui.separator();
            ui.heading("Central SQLite Database & ETS2 Datasets");
            if let Some(info) = &app.sqlite_info {
                ui.label(format!(
                    "DB Path: {} ({} bytes, {} tables)",
                    info.path,
                    info.size_bytes,
                    info.tables.len()
                ));
            }
            if let Some(counts) = &app.sqlite_counts {
                ui.label(format!(
                    "Companies: {} | Save Snapshots: {} | Depots: {} | Dispatcher Jobs: {}",
                    counts.ets2_companies,
                    counts.ets_save_snapshot,
                    counts.ets_save_depots,
                    counts.dispatcher_jobs
                ));
            }
            if ui.button("🔄 Re-Import ETS2 Datasets into SQLite").clicked() {
                match data_import_ets2_datasets(true, app.app.clone(), app.career_state()) {
                    Ok(summary) => {
                        app.refresh_settings_and_location();
                        app.set_status(
                            format!(
                                "Imported datasets ({} cities, {} companies).",
                                summary.city_count, summary.company_count
                            ),
                            false,
                        );
                    }
                    Err(e) => app.set_status(e, true),
                }
            }
        });

        cols[1].card(|ui| {
            ui.heading("Application Logs & Support Bundle");
            ui.horizontal_wrapped(|ui| {
                if ui.button("📄 Load Log Preview").clicked() {
                    app.user_logs = get_user_logs(None, Some(100)).unwrap_or_default();
                    app.diagnostic_log_preview =
                        async_runtime::block_on(get_log_preview(app.diagnostics_state()))
                            .unwrap_or_default();
                }
                if ui.button("📤 Export User Logs").clicked() {
                    match export_user_logs(app.app.clone(), app.profile_state()) {
                        Ok(path) => app.set_status(format!("Exported user logs: {path}"), false),
                        Err(e) => app.set_status(e, true),
                    }
                }
                if ui.button("📦 Export Support Log Bundle").clicked() {
                    match export_logs_bundle(app.app.clone(), app.profile_state()) {
                        Ok(Some(path)) => app.set_status(format!("Exported bundle: {path}"), false),
                        Ok(None) => {}
                        Err(e) => app.set_status(e, true),
                    }
                }
                if ui.button("📤 Export Diagnostic Log (.txt)").clicked() {
                    match async_runtime::block_on(export_log_file(
                        app.app.clone(),
                        app.diagnostics_state(),
                    )) {
                        Ok(Some(path)) => app.set_status(format!("Exported log: {path}"), false),
                        Ok(None) => {}
                        Err(e) => app.set_status(e, true),
                    }
                }
                if ui.button("Clear User Logs").clicked() {
                    let _ = clear_user_logs();
                    app.user_logs.clear();
                }
            });

            ui.separator();
            ui.label("Recent Diagnostic Log Preview:");
            egui::ScrollArea::vertical()
                .max_height(260.0)
                .show(ui, |ui| {
                    ui.monospace(&app.diagnostic_log_preview);
                });
        });
    });
}
