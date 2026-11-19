use eframe::egui::{self, Color32};

use crate::features::backup::commands::{
    list_active_save_backups, preview_backup_restore, restore_backup,
};
use crate::features::health_monitor::commands::{apply_save_health_fix, get_active_save_health};
use crate::features::profile_clone::commands::{clone_profile_command, validate_clone_target};
use crate::features::profile_controls::commands::copy_profile_controls;
use crate::features::profile_move_mods::commands::copy_mods_to_profile;
use crate::features::profile_rename::commands::profile_rename;
use crate::features::profile_sharing::commands::{
    export_shared_profile, get_profile_share_context, import_shared_profile,
    inspect_shared_profile_archive, pick_shared_profile_export_directory,
    pick_shared_profile_import_archive,
};
use crate::gui::components::{self, UiExt};
use crate::gui::{Ets2NativeApp, ProfileToolsSubTab};
use crate::tauri::async_runtime;

pub fn render(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    let (title, subtitle) = match app.profile_tools_sub_tab {
        ProfileToolsSubTab::CloneAndRename => (
            "Clone & Rename",
            "Duplicate the active profile or give it a new name.",
        ),
        ProfileToolsSubTab::ProfileSharing => (
            "Profile Sharing",
            "Export a profile as a shareable archive, or import one.",
        ),
        ProfileToolsSubTab::SaveHealth => (
            "Save Health",
            "Check the active save for problems and fix what can be fixed.",
        ),
        ProfileToolsSubTab::BackupsAndRestore => (
            "Backups & Restore",
            "Snapshots of your saves — restore one when something goes wrong.",
        ),
    };
    components::page_header(ui, title, subtitle);
    ui.add_space(8.0);

    match app.profile_tools_sub_tab {
        ProfileToolsSubTab::CloneAndRename => render_clone_and_rename(app, ui),
        ProfileToolsSubTab::ProfileSharing => render_profile_sharing(app, ui),
        ProfileToolsSubTab::SaveHealth => render_save_health(app, ui),
        ProfileToolsSubTab::BackupsAndRestore => render_backups_and_restore(app, ui),
    }
}

fn render_clone_and_rename(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    ui.columns(2, |cols| {
        cols[0].card(|ui| {
            ui.heading("Clone or Rename Active Profile");
            ui.horizontal(|ui| {
                ui.label("New Profile Name:");
                ui.text_edit_singleline(&mut app.clone_new_name);
            });
            ui.checkbox(&mut app.clone_backup, "Create backup before cloning");
            ui.checkbox(&mut app.clone_replace_hex, "Update hex folder identifier");
            ui.checkbox(&mut app.clone_replace_text, "Update internal profile.sii name");

            ui.horizontal(|ui| {
                if ui.button("📋 Clone Active Profile").clicked() {
                    if let Some(src) = app.selected_profile_path.clone() {
                        match validate_clone_target(src.clone(), app.clone_new_name.clone()) {
                            Ok(st) if !st.valid => app.set_status(st.message, true),
                            _ => match clone_profile_command(
                                src,
                                app.clone_new_name.clone(),
                                app.clone_backup,
                                app.clone_replace_hex,
                                app.clone_replace_text,
                            ) {
                                Ok(msg) => {
                                    app.refresh_profiles_and_saves();
                                    app.set_status(msg, false);
                                }
                                Err(e) => app.set_status(e, true),
                            },
                        }
                    }
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Rename Active Profile:");
                ui.text_edit_singleline(&mut app.rename_profile_input);
                if ui.button("✏ Rename").clicked() && !app.rename_profile_input.trim().is_empty() {
                    match profile_rename(
                        app.rename_profile_input.trim().to_string(),
                        app.profile_state(),
                    ) {
                        Ok(msg) => {
                            app.refresh_profiles_and_saves();
                            app.set_status(msg, false);
                        }
                        Err(e) => app.set_status(e, true),
                    }
                }
            });
        });

        cols[1].card(|ui| {
            ui.heading("Copy Controls (`controls.sii`) or Active Mods to Another Profile");
            egui::ComboBox::from_label("Target Profile")
                .selected_text(if app.target_profile_for_copy.is_empty() {
                    "<Select Target Profile>"
                } else {
                    &app.target_profile_for_copy
                })
                .width(260.0)
                .show_ui(ui, |ui| {
                    for p in &app.profiles {
                        let label = p.name.as_deref().unwrap_or(&p.path);
                        ui.selectable_value(&mut app.target_profile_for_copy, p.path.clone(), label);
                    }
                });

            ui.horizontal(|ui| {
                if ui.button("🎮 Copy Controls to Target").clicked() {
                    if let Some(src) = app.selected_profile_path.clone() {
                        match copy_profile_controls(src, app.target_profile_for_copy.clone()) {
                            Ok(msg) => app.set_status(msg, false),
                            Err(e) => app.set_status(e, true),
                        }
                    }
                }
                if ui.button("Copy Mod List to Target").clicked() {
                    match copy_mods_to_profile(
                        app.target_profile_for_copy.clone(),
                        app.profile_state(),
                    ) {
                        Ok(msg) => app.set_status(msg, false),
                        Err(e) => app.set_status(e, true),
                    }
                }
            });
        });
    });
}

fn render_profile_sharing(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    if ui.button("⟳ Load Profile Share Context").clicked() {
        app.share_context =
            get_profile_share_context(app.selected_profile_path.clone(), app.profile_state()).ok();
    }

    ui.columns(2, |cols| {
        cols[0].card(|ui| {
            ui.heading("Export Active Profile as ZIP Archive");
            ui.horizontal(|ui| {
                ui.label("Export Folder:");
                ui.text_edit_singleline(&mut app.share_export_dir);
                if ui.button("Browse...").clicked() {
                    if let Ok(Some(folder)) = async_runtime::block_on(
                        pick_shared_profile_export_directory(app.app.clone(), app.profile_state()),
                    ) {
                        app.share_export_dir = folder;
                    }
                }
            });
            if ui.button("📤 Export Profile Archive").clicked() {
                if let Some(profile_path) = app.selected_profile_path.clone() {
                    let dir_override = if app.share_export_dir.trim().is_empty() {
                        None
                    } else {
                        Some(app.share_export_dir.trim().to_string())
                    };
                    match export_shared_profile(profile_path, dir_override, app.profile_state()) {
                        Ok(res) => app.set_status(
                            format!(
                                "Exported {} ({} files) to {}",
                                res.profile_name, res.exported_files, res.archive_path
                            ),
                            false,
                        ),
                        Err(e) => app.set_status(e, true),
                    }
                }
            }
        });

        cols[1].card(|ui| {
            ui.heading("Import Shared Profile Archive");
            ui.horizontal(|ui| {
                ui.label("Archive (.zip):");
                ui.text_edit_singleline(&mut app.share_import_archive);
                if ui.button("Browse...").clicked() {
                    if let Ok(Some(file)) = async_runtime::block_on(
                        pick_shared_profile_import_archive(app.app.clone(), app.profile_state()),
                    ) {
                        app.share_import_archive = file;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Profile Name Override (optional):");
                ui.text_edit_singleline(&mut app.share_import_name_override);
            });
            ui.horizontal(|ui| {
                if ui.button("🔍 Inspect Archive").clicked()
                    && !app.share_import_archive.trim().is_empty()
                {
                    let name_override = if app.share_import_name_override.trim().is_empty() {
                        None
                    } else {
                        Some(app.share_import_name_override.trim().to_string())
                    };
                    match inspect_shared_profile_archive(
                        app.share_import_archive.trim().to_string(),
                        name_override,
                        app.profile_state(),
                    ) {
                        Ok(prev) => app.share_import_preview = Some(prev),
                        Err(e) => app.set_status(e, true),
                    }
                }
                if ui.button("📥 Import Profile Now").clicked()
                    && !app.share_import_archive.trim().is_empty()
                {
                    let name_override = if app.share_import_name_override.trim().is_empty() {
                        None
                    } else {
                        Some(app.share_import_name_override.trim().to_string())
                    };
                    match import_shared_profile(
                        app.share_import_archive.trim().to_string(),
                        name_override,
                        app.profile_state(),
                    ) {
                        Ok(res) => {
                            app.refresh_profiles_and_saves();
                            app.set_status(
                                format!(
                                    "Imported profile `{}` ({} files).",
                                    res.profile_name, res.imported_files
                                ),
                                false,
                            );
                        }
                        Err(e) => app.set_status(e, true),
                    }
                }
            });
            if let Some(prev) = &app.share_import_preview {
                ui.label(format!(
                    "Detected: {} -> Final Name: {} | Files: {} | Conflict: {}",
                    prev.detected_profile_name,
                    prev.final_profile_name,
                    prev.file_count,
                    prev.profile_name_conflict
                ));
            }
        });
    });
}

fn render_save_health(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    if ui.button("Scan Active Save Health").clicked() {
        match async_runtime::block_on(get_active_save_health(
            app.profile_state(),
            app.decrypt_cache(),
        )) {
            Ok(rep) => {
                app.save_health_report = Some(rep);
                app.set_status("Completed save health inspection.", false);
            }
            Err(e) => app.set_status(e, true),
        }
    }

    if let Some(rep) = app.save_health_report.clone() {
        ui.card(|ui| {
            ui.heading(format!("Status: {} — {}", rep.status, rep.summary));
            ui.label(format!(
                "Problems Found: {} | Auto-Fixable: {}",
                rep.problem_count, rep.fixable_count
            ));
        });

        for prob in &rep.problems {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(
                        Color32::from_rgb(245, 176, 65),
                        format!("[{}] {}", prob.severity, prob.title),
                    );
                    if prob.auto_fix_available {
                        if let Some(fix_id) = &prob.fix_id {
                            if ui.small_button("🛠 Apply Auto-Fix").clicked() {
                                match apply_save_health_fix(
                                    fix_id.clone(),
                                    true,
                                    app.profile_state(),
                                    app.profile_cache(),
                                    app.decrypt_cache(),
                                ) {
                                    Ok(res) => {
                                        app.reload_active_save_data();
                                        app.set_status(res.message, !res.applied);
                                    }
                                    Err(e) => app.set_status(e, true),
                                }
                            }
                        }
                    }
                });
                ui.label(&prob.description);
                ui.label(format!("Suggestion: {}", prob.suggestion));
            });
        }
    }
}

fn render_backups_and_restore(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    if ui.button("⟳ List Active Save Backups").clicked() {
        match async_runtime::block_on(list_active_save_backups(app.profile_state())) {
            Ok(list) => app.backups_list = list,
            Err(e) => app.set_status(e, true),
        }
    }
    ui.add_space(6.0);

    if app.backups_list.is_empty() {
        components::empty_state(
            ui,
            "No backups listed",
            "List the active save's backups to inspect or restore one.",
        );
        return;
    }

    // Selectable table; the detail card below shows the preview and the
    // Restore action for the selected backup.
    egui::Grid::new("backups_grid")
        .striped(true)
        .num_columns(5)
        .show(ui, |ui| {
            ui.strong("Backup ID");
            ui.strong("Created At (UTC)");
            ui.strong("Type");
            ui.strong("Reason");
            ui.strong("Files");
            ui.end_row();

            for b in app.backups_list.clone() {
                let selected = app.selected_backup_id == b.backup_id;
                if ui.selectable_label(selected, &b.backup_id).clicked() {
                    app.selected_backup_id = b.backup_id.clone();
                    app.backup_preview =
                        preview_backup_restore(b.backup_id.clone(), app.profile_state()).ok();
                }
                ui.label(&b.created_at_utc);
                ui.label(&b.backup_type);
                ui.label(&b.action_reason);
                ui.label(format!("{}", b.file_count));
                ui.end_row();
            }
        });

    if let Some(prev) = app.backup_preview.clone() {
        ui.add_space(10.0);
        ui.separator();
        ui.card(|ui| {
            ui.heading(format!("Backup Restore Preview: {}", prev.backup_id));
            ui.label(format!("Reason: {} | Created: {}", prev.action_reason, prev.created_at_utc));
            for f in &prev.files {
                ui.label(format!(
                    "• {} ({} changes, status={})",
                    f.relative_path, f.change_count, f.status
                ));
            }
            ui.add_space(8.0);
            if ui.button("Restore This Backup").clicked() {
                app.backup_confirm = Some(prev.backup_id.clone());
            }
        });
    }

    render_backup_confirm(app, ui);
}

fn render_backup_confirm(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    let Some(backup_id) = app.backup_confirm.clone() else {
        return;
    };
    let message = format!(
        "Restore backup {backup_id}? This overwrites the current save with the backup's files. Your current state is kept as a backup first."
    );
    match components::confirm_dialog(
        ui.ctx(),
        "Restore backup",
        &message,
        "Restore",
        "Cancel",
    ) {
        Some(true) => {
            app.backup_confirm = None;
            match restore_backup(
                backup_id,
                true,
                app.profile_state(),
                app.profile_cache(),
                app.decrypt_cache(),
            ) {
                Ok(res) => {
                    app.reload_active_save_data();
                    app.set_status(
                        format!(
                            "Restored {} files from backup {}.",
                            res.restored_file_count, res.backup_id
                        ),
                        false,
                    );
                }
                Err(e) => app.set_status(e, true),
            }
        }
        Some(false) => app.backup_confirm = None,
        None => {}
    }
}
