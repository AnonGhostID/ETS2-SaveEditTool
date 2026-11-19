use eframe::egui::{self, Color32, RichText};

use crate::features::save_editor::commands::{
    KeyValuePayload, apply_custom_reset_values, edit_config_value, edit_console_value,
    edit_convoy_value, edit_developer_value, edit_level, edit_parking_doubles_value,
    edit_player_experience, edit_player_money, edit_save_config_value, edit_skill_value,
    edit_traffic_value, read_project_coaches,
};
use crate::gui::components::{self, UiExt};
use crate::gui::{Ets2NativeApp, WorkspaceTab};

pub fn render(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    components::page_header(
        ui,
        "Save Editor",
        "Edit economy, skills and game configuration for the active save.",
    );
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        if components::primary_button(ui, "Refresh Save Data").clicked() {
            app.reload_active_save_data();
            match read_project_coaches(app.profile_state()) {
                Ok(snapshot) => {
                    // Keep the Coaches tab's XP editor in sync with the
                    // freshly loaded snapshot (same as its own Load button).
                    app.edit_bus_xp_input = snapshot.bus_xp.unwrap_or(0);
                    app.coaches_snapshot = Some(snapshot);
                }
                Err(_) => app.coaches_snapshot = None,
            }
            app.set_status("Refreshed active save and config values.", false);
        }
    });
    ui.add_space(8.0);

    if app.selected_save_path.is_none() {
        components::empty_state(
            ui,
            "No active save selected",
            "Choose a profile and save slot in the top command bar to start editing.",
        );
        return;
    }

    ui.columns(2, |cols| {
        // Left Column: Money, XP, Level, Skills
        cols[0].card(|ui| {
            ui.heading("Player Economy & Experience");
            if let Some(data) = &app.save_data {
                let level = data
                    .xp
                    .map(|xp| {
                        crate::xp::command::calculate_level(xp.max(0) as u64)
                            .level
                            .to_string()
                    })
                    .unwrap_or_else(|| "-".to_string());
                ui.columns(2, |cols| {
                    components::stat_tile(
                        &mut cols[0],
                        "Money",
                        &format!("€{}", data.money.unwrap_or(0)),
                        crate::gui::theme::SUCCESS,
                    );
                    components::stat_tile(
                        &mut cols[1],
                        "Total XP",
                        &data.xp.unwrap_or(0).to_string(),
                        crate::gui::theme::ACCENT,
                    );
                });
                ui.add_space(6.0);
                ui.columns(2, |cols| {
                    components::stat_tile(
                        &mut cols[0],
                        "Level",
                        &level,
                        crate::gui::theme::ACCENT,
                    );
                    components::stat_tile(
                        &mut cols[1],
                        "Visited Cities",
                        &data.visited_cities.unwrap_or(0).to_string(),
                        crate::gui::theme::MUTED,
                    );
                });
            }
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Money (€):");
                ui.add(
                    egui::DragValue::new(&mut app.edit_money_input)
                        .speed(10_000)
                        .range(0..=999_999_999_999_i64),
                );
                if ui.button("Apply Money").clicked() {
                    match edit_player_money(
                        app.edit_money_input,
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    ) {
                        Ok(()) => {
                            app.reload_active_save_data();
                            app.set_status(
                                format!(
                                    "Staged money €{} (press Save to write).",
                                    app.edit_money_input
                                ),
                                false,
                            );
                        }
                        Err(e) => app.set_status(format!("Failed to update money: {e}"), true),
                    }
                }
            });

            ui.horizontal(|ui| {
                for (label, amount) in [
                    ("+100k", 100_000_i64),
                    ("+1M", 1_000_000_i64),
                    ("Set 10M", 10_000_000_i64),
                    ("Set 100M", 100_000_000_i64),
                ] {
                    if ui.small_button(label).clicked() {
                        if label.starts_with('+') {
                            app.edit_money_input = app.edit_money_input.saturating_add(amount);
                        } else {
                            app.edit_money_input = amount;
                        }
                    }
                }
            });

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Truck XP:");
                if ui
                    .add(
                        egui::DragValue::new(&mut app.edit_xp_input)
                            .speed(1_000)
                            .range(0..=999_999_999_i64),
                    )
                    .changed()
                {
                    // L2: the batch applies whichever of XP / Target
                    // Level was edited last.
                    app.batch_level_edited_last = false;
                }
                if ui.button("Apply XP").clicked() {
                    match edit_player_experience(
                        app.edit_xp_input,
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    ) {
                        Ok(()) => {
                            app.reload_active_save_data();
                            app.set_status(
                                format!(
                                    "Staged truck XP {} (press Save to write).",
                                    app.edit_xp_input
                                ),
                                false,
                            );
                        }
                        Err(e) if crate::gui::is_no_change_error(&e) => {
                            app.set_status("Truck XP is already at that value.", false);
                        }
                        Err(e) => app.set_status(format!("Failed to update XP: {e}"), true),
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Target Level:");
                if ui
                    .add(egui::DragValue::new(&mut app.edit_level_input).range(0..=256))
                    .changed()
                {
                    // L2: the batch applies whichever of XP / Target
                    // Level was edited last.
                    app.batch_level_edited_last = true;
                }
                if ui.button("Apply Level").clicked() {
                    match edit_level(
                        app.edit_level_input,
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    ) {
                        Ok(()) => {
                            app.reload_active_save_data();
                            app.set_status(
                                format!(
                                    "Staged level {} (press Save to write).",
                                    app.edit_level_input
                                ),
                                false,
                            );
                        }
                        Err(e) if crate::gui::is_no_change_error(&e) => {
                            app.set_status("Level is already at that value.", false);
                        }
                        Err(e) => app.set_status(format!("Failed to update level: {e}"), true),
                    }
                }
                if ui.button("Apply Money + XP + Level Batch").clicked() {
                    // L2: pass only the last-edited of level/XP — the
                    // backend resolves level -> XP only when XP is
                    // unset, and reports the level actually written.
                    let (level, xp) = if app.batch_level_edited_last {
                        (Some(app.edit_level_input.max(0) as u32), None)
                    } else {
                        (None, Some(app.edit_xp_input.max(0) as u64))
                    };
                    match apply_custom_reset_values(
                        level,
                        xp,
                        Some(app.edit_money_input),
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    ) {
                        Ok(res) => {
                            app.batch_level_edited_last = false;
                            app.reload_active_save_data();
                            let what = match (res.applied_level, res.applied_xp) {
                                (Some(level), _) => format!("level {level}"),
                                (None, Some(xp)) => format!("XP {xp}"),
                                (None, None) => "money".to_string(),
                            };
                            app.set_status(
                                format!("Batch staged ({what}; backup {}).", res.undo_backup_id),
                                false,
                            );
                        }
                        Err(e) => app.set_status(format!("Batch update failed: {e}"), true),
                    }
                }
            });

            ui.separator();
            ui.heading("Player Skills (ADR 0 – 63, others 0 – 6)");
            let mut skill_to_save: Option<(&'static str, i64)> = None;

            ui.horizontal(|ui| {
                ui.label("ADR (Hazardous):");
                // S1: ADR maxes at 63 in the save, not 6.
                ui.add(egui::Slider::new(&mut app.skill_adr, 0..=63));
                if ui.small_button("Save").clicked() {
                    skill_to_save = Some(("adr", app.skill_adr));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Long Distance:");
                ui.add(egui::Slider::new(&mut app.skill_long_dist, 0..=6));
                if ui.small_button("Save").clicked() {
                    skill_to_save = Some(("long_dist", app.skill_long_dist));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Heavy / High Value Cargo:");
                ui.add(egui::Slider::new(&mut app.skill_heavy, 0..=6));
                if ui.small_button("Save").clicked() {
                    skill_to_save = Some(("heavy", app.skill_heavy));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Fragile Cargo:");
                ui.add(egui::Slider::new(&mut app.skill_fragile, 0..=6));
                if ui.small_button("Save").clicked() {
                    skill_to_save = Some(("fragile", app.skill_fragile));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Just-In-Time (Urgent):");
                ui.add(egui::Slider::new(&mut app.skill_urgent, 0..=6));
                if ui.small_button("Save").clicked() {
                    skill_to_save = Some(("urgent", app.skill_urgent));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Eco Driving (Mechanical):");
                ui.add(egui::Slider::new(&mut app.skill_mechanical, 0..=6));
                if ui.small_button("Save").clicked() {
                    skill_to_save = Some(("mechanical", app.skill_mechanical));
                }
            });

            if let Some((key, val)) = skill_to_save {
                match edit_skill_value(
                    key.to_string(),
                    val,
                    app.profile_state(),
                    app.profile_cache(),
                    app.decrypt_cache(),
                ) {
                    Ok(()) => app.set_status(
                        format!("Staged skill `{key}` = {val} (press Save to write)."),
                        false,
                    ),
                    Err(e) => app.set_status(format!("Skill update failed: {e}"), true),
                }
            }

            if ui.button("Max All Skills (ADR 63, others 6)").clicked() {
                // S1/S4: ADR maxes at 63, the other five skills at 6;
                // surface per-skill failures instead of silently
                // continuing and claiming success.
                let mut failures: Vec<String> = Vec::new();
                for (key, max) in [
                    ("adr", 63),
                    ("long_dist", 6),
                    ("heavy", 6),
                    ("fragile", 6),
                    ("urgent", 6),
                    ("mechanical", 6),
                ] {
                    if let Err(err) = edit_skill_value(
                        key.to_string(),
                        max,
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    ) {
                        failures.push(format!("{key}: {err}"));
                    }
                }
                app.skill_adr = 63;
                app.skill_long_dist = 6;
                app.skill_heavy = 6;
                app.skill_fragile = 6;
                app.skill_urgent = 6;
                app.skill_mechanical = 6;
                app.reload_active_save_data();
                if failures.is_empty() {
                    app.set_status(
                        "Staged all skills at maximum (ADR 63, others 6).",
                        false,
                    );
                } else {
                    app.set_status(
                        format!("Max All Skills failed for: {}", failures.join("; ")),
                        true,
                    );
                }
            }
        });

        // Right Column: Base & Save Configs + Coaches XP
        cols[1].card(|ui| {
            ui.heading("Game & Save Configuration (`config.cfg`)");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Traffic Density (0–10):");
                ui.add(egui::Slider::new(&mut app.cfg_traffic, 0..=10));
                if ui.button("Apply").clicked() {
                    match edit_traffic_value(
                        app.cfg_traffic,
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    ) {
                        Ok(()) => app.set_status("Updated g_traffic.", false),
                        Err(e) => app.set_status(e, true),
                    }
                }
            });

            ui.horizontal(|ui| {
                let mut dev_bool = app.cfg_developer != 0;
                if ui.checkbox(&mut dev_bool, "Developer Mode (`g_developer`)").changed() {
                    // S3: surface a failed write instead of discarding
                    // it; the checkbox reverts next frame because the
                    // stored value is only updated on success.
                    let new_value = i64::from(dev_bool);
                    match edit_developer_value(
                        new_value,
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    ) {
                        Ok(()) => app.cfg_developer = new_value,
                        Err(e) => {
                            app.set_status(format!("Developer mode change failed: {e}"), true)
                        }
                    }
                }
            });

            ui.horizontal(|ui| {
                let mut con_bool = app.cfg_console != 0;
                if ui.checkbox(&mut con_bool, "Developer Console (`g_console`)").changed() {
                    // S3: see above.
                    let new_value = i64::from(con_bool);
                    match edit_console_value(
                        new_value,
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    ) {
                        Ok(()) => app.cfg_console = new_value,
                        Err(e) => {
                            app.set_status(format!("Developer console change failed: {e}"), true)
                        }
                    }
                }
            });

            ui.horizontal(|ui| {
                let mut doubles_bool = app.cfg_parking_doubles != 0;
                if ui
                    .checkbox(
                        &mut doubles_bool,
                        "Enable Double Trailer Parking (`g_simple_parking_doubles`)",
                    )
                    .changed()
                {
                    // S3: see above.
                    let new_value = i64::from(doubles_bool);
                    match edit_parking_doubles_value(
                        new_value,
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    ) {
                        Ok(()) => app.cfg_parking_doubles = new_value,
                        Err(e) => app.set_status(
                            format!("Double trailer parking change failed: {e}"),
                            true,
                        ),
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Max Convoy Size:");
                ui.add(egui::Slider::new(&mut app.cfg_convoy, 1..=128));
                if ui.button("Apply").clicked() {
                    match edit_convoy_value(
                        app.cfg_convoy,
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    ) {
                        Ok(()) => app.set_status("Updated max convoy size.", false),
                        Err(e) => app.set_status(e, true),
                    }
                }
            });

            ui.separator();
            egui::CollapsingHeader::new("Advanced: custom config keys")
                .default_open(false)
                .show(ui, |ui| {
                    ui.label(RichText::new("Custom Global `config.cfg` Key/Value").strong());
                    ui.horizontal(|ui| {
                        ui.label("Key:");
                        ui.text_edit_singleline(&mut app.custom_cfg_key);
                        ui.label("Value:");
                        ui.text_edit_singleline(&mut app.custom_cfg_value);
                        if ui.button("Set").clicked() && !app.custom_cfg_key.trim().is_empty() {
                            match edit_config_value(
                                KeyValuePayload {
                                    key: app.custom_cfg_key.trim().to_string(),
                                    value: app.custom_cfg_value.trim().to_string(),
                                },
                                app.profile_state(),
                                app.profile_cache(),
                                app.decrypt_cache(),
                            ) {
                                Ok(()) => app.set_status("Updated global config.cfg key.", false),
                                Err(e) => app.set_status(e, true),
                            }
                        }
                    });

                    ui.label(RichText::new("Custom Profile `config.cfg` Key/Value").strong());
                    ui.horizontal(|ui| {
                        ui.label("Key:");
                        ui.text_edit_singleline(&mut app.custom_save_cfg_key);
                        ui.label("Value:");
                        ui.text_edit_singleline(&mut app.custom_save_cfg_value);
                        if ui.button("Set").clicked() && !app.custom_save_cfg_key.trim().is_empty()
                        {
                            match edit_save_config_value(
                                KeyValuePayload {
                                    key: app.custom_save_cfg_key.trim().to_string(),
                                    value: app.custom_save_cfg_value.trim().to_string(),
                                },
                                app.profile_state(),
                                app.profile_cache(),
                                app.decrypt_cache(),
                            ) {
                                Ok(()) => {
                                    app.set_status("Updated profile config.cfg key.", false)
                                }
                                Err(e) => app.set_status(e, true),
                            }
                        }
                    });
                });

            ui.separator();
            ui.horizontal(|ui| {
                ui.heading("Project Coaches / Bus Career");
                if let Some(coaches) = &app.coaches_snapshot {
                    ui.label(format!(
                        "Bus XP: {} · Discovered Stops: {}",
                        coaches
                            .bus_xp
                            .map(|xp| xp.to_string())
                            .unwrap_or_else(|| "—".to_string()),
                        coaches.discovered_stops
                    ));
                }
            });
            if ui.button("Open Coaches ->").clicked() {
                app.active_tab = WorkspaceTab::Coaches;
            }
        });
    });
}
