use eframe::egui::{self, Color32, RichText};

use crate::features::save_editor::coaches::Career;
use crate::features::save_editor::commands::{
    KeyValuePayload, apply_custom_reset_values, edit_career_xp, edit_config_value,
    edit_console_value, edit_convoy_value, edit_developer_value, edit_level,
    edit_parking_doubles_value, edit_player_experience, edit_player_money, edit_save_config_value,
    edit_skill_value, edit_traffic_value, read_project_coaches,
};
use crate::gui::components::{self, UiExt};
use crate::gui::Ets2NativeApp;

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
            app.coaches_snapshot = read_project_coaches(app.profile_state()).ok();
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
                ui.label(format!(
                    "Current Money: €{} | Current XP: {} | Visited Cities: {} | Dealers: {}",
                    data.money.unwrap_or(0),
                    data.xp.unwrap_or(0),
                    data.visited_cities.unwrap_or(0),
                    data.dealers.unwrap_or(0)
                ));
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
                                format!("Updated player money to €{}.", app.edit_money_input),
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
                ui.label("Experience (XP):");
                ui.add(
                    egui::DragValue::new(&mut app.edit_xp_input)
                        .speed(1_000)
                        .range(0..=999_999_999_i64),
                );
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
                                format!("Updated player XP to {}.", app.edit_xp_input),
                                false,
                            );
                        }
                        Err(e) => app.set_status(format!("Failed to update XP: {e}"), true),
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Target Level:");
                ui.add(egui::DragValue::new(&mut app.edit_level_input).range(0..=500));
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
                                format!("Updated player level to {}.", app.edit_level_input),
                                false,
                            );
                        }
                        Err(e) => app.set_status(format!("Failed to update level: {e}"), true),
                    }
                }
                if ui.button("Apply Money + XP + Level Batch").clicked() {
                    match apply_custom_reset_values(
                        Some(app.edit_level_input.max(0) as u32),
                        Some(app.edit_xp_input.max(0) as u64),
                        Some(app.edit_money_input),
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    ) {
                        Ok(res) => {
                            app.reload_active_save_data();
                            app.set_status(
                                format!("Batch applied (backup {}).", res.undo_backup_id),
                                false,
                            );
                        }
                        Err(e) => app.set_status(format!("Batch update failed: {e}"), true),
                    }
                }
            });

            ui.separator();
            ui.heading("Player Skills (0 – 6)");
            let mut skill_to_save: Option<(&'static str, i64)> = None;

            ui.horizontal(|ui| {
                ui.label("ADR (Hazardous):");
                ui.add(egui::Slider::new(&mut app.skill_adr, 0..=6));
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
                    Ok(()) => app.set_status(format!("Saved skill `{key}` = {val}"), false),
                    Err(e) => app.set_status(format!("Skill update failed: {e}"), true),
                }
            }

            if ui.button("Max All Skills (Set all to 6)").clicked() {
                app.skill_adr = 6;
                app.skill_long_dist = 6;
                app.skill_heavy = 6;
                app.skill_fragile = 6;
                app.skill_urgent = 6;
                app.skill_mechanical = 6;
                for key in ["adr", "long_dist", "heavy", "fragile", "urgent", "mechanical"] {
                    let _ = edit_skill_value(
                        key.to_string(),
                        6,
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    );
                }
                app.reload_active_save_data();
                app.set_status("All 6 skills set to maximum (6).", false);
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
                    app.cfg_developer = i64::from(dev_bool);
                    let _ = edit_developer_value(
                        app.cfg_developer,
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    );
                }
            });

            ui.horizontal(|ui| {
                let mut con_bool = app.cfg_console != 0;
                if ui.checkbox(&mut con_bool, "Developer Console (`g_console`)").changed() {
                    app.cfg_console = i64::from(con_bool);
                    let _ = edit_console_value(
                        app.cfg_console,
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    );
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
                    app.cfg_parking_doubles = i64::from(doubles_bool);
                    let _ = edit_parking_doubles_value(
                        app.cfg_parking_doubles,
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    );
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
                if ui.button("Set").clicked() && !app.custom_save_cfg_key.trim().is_empty() {
                    match edit_save_config_value(
                        KeyValuePayload {
                            key: app.custom_save_cfg_key.trim().to_string(),
                            value: app.custom_save_cfg_value.trim().to_string(),
                        },
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                    ) {
                        Ok(()) => app.set_status("Updated profile config.cfg key.", false),
                        Err(e) => app.set_status(e, true),
                    }
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.heading("Project Coaches / Bus Career XP");
                if ui.small_button("Inspect Coaches").clicked() {
                    match read_project_coaches(app.profile_state()) {
                        Ok(snap) => app.coaches_snapshot = Some(snap),
                        Err(e) => app.set_status(format!("Coaches inspect: {e}"), true),
                    }
                }
            });
            if let Some(coaches) = app.coaches_snapshot.clone() {
                ui.label(format!(
                    "Truck XP: {:?} | Bus XP: {:?} | Discovered Bus Stops: {}",
                    coaches.truck_xp, coaches.bus_xp, coaches.discovered_stops
                ));
                ui.horizontal(|ui| {
                    if ui.button("Sync XP to Bus Career").clicked() {
                        match edit_career_xp(
                            app.edit_xp_input,
                            Career::Bus,
                            coaches.save_hash.clone(),
                            app.profile_state(),
                            app.profile_cache(),
                            app.decrypt_cache(),
                        ) {
                            Ok(()) => {
                                app.coaches_snapshot = read_project_coaches(app.profile_state()).ok();
                                app.set_status("Updated Bus Career XP.", false);
                            }
                            Err(e) => app.set_status(e, true),
                        }
                    }
                });
            }
        });
    });
}
