use eframe::egui::{self, Color32};

use crate::features::garages::commands::{
    assign_random_garage_resources, buy_all_garages, generate_ai_drivers, get_ai_driver_management,
    get_all_garages, purchase_garage, relinquish_empty_garages, relinquish_garage_ownership,
    remove_ai_drivers, update_garage, upgrade_owned_garage,
};
use crate::features::garages::models::{
    AiDriverGenerateRequest, AiDriverRemoveRequest, GarageBuyAllRequest, GarageMutationRequest,
    GarageOwnership, GarageRelinquishEmptyRequest, GarageResourceAssignmentRequest,
    GarageUpdateRequest,
};
use crate::gui::Ets2NativeApp;
use crate::tauri::async_runtime;

pub fn render(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("🏢 Garages & AI Driver Management");
        if ui.button("⟳ Load Garages & Drivers").clicked() {
            reload_garages_and_drivers(app);
        }
    });
    ui.separator();

    if app.garage_list.is_none() {
        ui.label("Click 'Load Garages & Drivers' to parse garages and AI drivers from the active save.");
        return;
    }

    if let Some(garages_res) = app.garage_list.clone() {
        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "Parsed Garages: {} | Owned: {} | Unowned: {} | HQ: {}",
                garages_res.diagnostics.parsed_garage_count,
                garages_res.diagnostics.owned_garage_count,
                garages_res.diagnostics.not_owned_garage_count,
                garages_res.headquarters_garage_id.as_deref().unwrap_or("-")
            ));
            ui.separator();
            ui.label("Filter:");
            ui.text_edit_singleline(&mut app.garage_filter_text);
            ui.checkbox(&mut app.garage_only_owned, "Owned only");

            if ui.button("🏙 Buy All Garages").clicked() {
                match buy_all_garages(
                    GarageBuyAllRequest {
                        expected_save_hash: garages_res.save_hash.clone(),
                    },
                    app.profile_state(),
                    app.profile_cache(),
                    app.decrypt_cache(),
                    app.app.state(),
                    app.app.state(),
                    app.app_state(),
                ) {
                    Ok(res) => {
                        reload_garages_and_drivers(app);
                        app.set_status(
                            format!("Purchased {} garages.", res.purchased_count),
                            false,
                        );
                    }
                    Err(e) => app.set_status(format!("Buy all garages failed: {e}"), true),
                }
            }

            if ui.button("🧹 Sell Empty Garages").clicked() {
                match relinquish_empty_garages(
                    GarageRelinquishEmptyRequest {
                        expected_save_hash: garages_res.save_hash.clone(),
                    },
                    app.profile_state(),
                    app.profile_cache(),
                    app.decrypt_cache(),
                    app.app.state(),
                    app.app.state(),
                    app.app_state(),
                ) {
                    Ok(res) => {
                        reload_garages_and_drivers(app);
                        app.set_status(
                            format!("Relinquished {} empty garages.", res.relinquished_count),
                            false,
                        );
                    }
                    Err(e) => app.set_status(format!("Relinquish empty failed: {e}"), true),
                }
            }
        });

        ui.add_space(6.0);
        let filter_lower = app.garage_filter_text.trim().to_ascii_lowercase();

        egui::Grid::new("garages_table_grid")
            .striped(true)
            .num_columns(7)
            .show(ui, |ui| {
                ui.strong("Garage / City");
                ui.strong("Country");
                ui.strong("Ownership");
                ui.strong("Size");
                ui.strong("Slots (Used/Max)");
                ui.strong("Drivers / Trucks");
                ui.strong("Actions");
                ui.end_row();

                for g in &garages_res.garages {
                    if app.garage_only_owned && g.ownership != GarageOwnership::Owned {
                        continue;
                    }
                    let city_label = g
                        .city_name
                        .as_deref()
                        .or(g.city_token.as_deref())
                        .unwrap_or(&g.garage_id);
                    if !filter_lower.is_empty()
                        && !city_label.to_ascii_lowercase().contains(&filter_lower)
                        && !g.garage_id.to_ascii_lowercase().contains(&filter_lower)
                    {
                        continue;
                    }

                    ui.label(format!(
                        "{}{}",
                        city_label,
                        if g.is_headquarters { " ⭐ (HQ)" } else { "" }
                    ));
                    ui.label(g.country_code.as_deref().unwrap_or("-"));
                    if g.ownership == GarageOwnership::Owned {
                        ui.colored_label(Color32::from_rgb(88, 214, 141), "Owned");
                    } else {
                        ui.label("Unowned");
                    }
                    ui.label(format!("{:?}", g.size));
                    ui.label(format!("{}/{}", g.occupied_slots, g.maximum_slot_count));
                    ui.label(format!(
                        "{} drv / {} trk",
                        g.assigned_driver_count, g.assigned_truck_count
                    ));

                    ui.horizontal(|ui| {
                        if g.ownership != GarageOwnership::Owned {
                            if ui.small_button("Buy").clicked() {
                                match purchase_garage(
                                    GarageMutationRequest {
                                        garage_id: g.garage_id.clone(),
                                        expected_save_hash: garages_res.save_hash.clone(),
                                    },
                                    app.profile_state(),
                                    app.profile_cache(),
                                    app.decrypt_cache(),
                                    app.app.state(),
                                    app.app.state(),
                                    app.app_state(),
                                ) {
                                    Ok(_) => {
                                        reload_garages_and_drivers(app);
                                        app.set_status(format!("Purchased {}.", g.garage_id), false);
                                    }
                                    Err(e) => app.set_status(e, true),
                                }
                            }
                        } else {
                            if ui.small_button("Upgrade").clicked() {
                                match upgrade_owned_garage(
                                    GarageMutationRequest {
                                        garage_id: g.garage_id.clone(),
                                        expected_save_hash: garages_res.save_hash.clone(),
                                    },
                                    app.profile_state(),
                                    app.profile_cache(),
                                    app.decrypt_cache(),
                                    app.app.state(),
                                    app.app.state(),
                                    app.app_state(),
                                ) {
                                    Ok(_) => {
                                        reload_garages_and_drivers(app);
                                        app.set_status(format!("Upgraded {}.", g.garage_id), false);
                                    }
                                    Err(e) => app.set_status(e, true),
                                }
                            }
                            if ui.small_button("+Drv+Trk").clicked() {
                                match assign_random_garage_resources(
                                    GarageResourceAssignmentRequest {
                                        garage_id: g.garage_id.clone(),
                                        expected_save_hash: garages_res.save_hash.clone(),
                                        assign_random_driver: true,
                                        assign_random_truck: true,
                                    },
                                    app.profile_state(),
                                    app.profile_cache(),
                                    app.decrypt_cache(),
                                    app.app.state(),
                                    app.app.state(),
                                    app.app_state(),
                                ) {
                                    Ok(_) => {
                                        reload_garages_and_drivers(app);
                                        app.set_status(
                                            format!("Assigned driver & truck to {}.", g.garage_id),
                                            false,
                                        );
                                    }
                                    Err(e) => app.set_status(e, true),
                                }
                            }
                            if !g.is_headquarters && ui.small_button("Set HQ").clicked() {
                                match update_garage(
                                    GarageUpdateRequest {
                                        garage_id: g.garage_id.clone(),
                                        expected_save_hash: garages_res.save_hash.clone(),
                                        target_size: None,
                                        set_as_headquarters: true,
                                    },
                                    app.profile_state(),
                                    app.profile_cache(),
                                    app.decrypt_cache(),
                                    app.app.state(),
                                    app.app.state(),
                                    app.app_state(),
                                ) {
                                    Ok(_) => {
                                        reload_garages_and_drivers(app);
                                        app.set_status(
                                            format!("Set {} as headquarters.", g.garage_id),
                                            false,
                                        );
                                    }
                                    Err(e) => app.set_status(e, true),
                                }
                            }
                            if ui.small_button("Sell").clicked() {
                                match relinquish_garage_ownership(
                                    GarageMutationRequest {
                                        garage_id: g.garage_id.clone(),
                                        expected_save_hash: garages_res.save_hash.clone(),
                                    },
                                    app.profile_state(),
                                    app.profile_cache(),
                                    app.decrypt_cache(),
                                    app.app.state(),
                                    app.app.state(),
                                    app.app_state(),
                                ) {
                                    Ok(_) => {
                                        reload_garages_and_drivers(app);
                                        app.set_status(
                                            format!("Relinquished {}.", g.garage_id),
                                            false,
                                        );
                                    }
                                    Err(e) => app.set_status(e, true),
                                }
                            }
                        }
                    });
                    ui.end_row();
                }
            });
    }

    if let Some(ai_mgmt) = app.ai_driver_mgmt.clone() {
        ui.add_space(12.0);
        ui.separator();
        ui.heading(format!(
            "👤 AI Drivers ({} Active | {} Available Pool)",
            ai_mgmt.drivers.len(),
            ai_mgmt.available_driver_count
        ));

        ui.horizontal(|ui| {
            ui.label("Garage ID:");
            ui.text_edit_singleline(&mut app.selected_garage_for_drivers);
            ui.label("Count:");
            ui.add(egui::DragValue::new(&mut app.generate_drivers_count).range(1..=5));
            ui.checkbox(&mut app.generate_drivers_assign_trucks, "Also assign trucks");
            if ui.button("➕ Generate AI Drivers").clicked()
                && !app.selected_garage_for_drivers.trim().is_empty()
            {
                match generate_ai_drivers(
                    AiDriverGenerateRequest {
                        garage_id: app.selected_garage_for_drivers.trim().to_string(),
                        expected_save_hash: ai_mgmt.save_hash.clone(),
                        count: app.generate_drivers_count,
                        assign_trucks: app.generate_drivers_assign_trucks,
                    },
                    app.profile_state(),
                    app.profile_cache(),
                    app.decrypt_cache(),
                    app.app.state(),
                    app.app.state(),
                    app.app_state(),
                ) {
                    Ok(res) => {
                        app.ai_driver_mgmt = Some(res.management);
                        app.set_status("Generated AI drivers.", false);
                    }
                    Err(e) => app.set_status(e, true),
                }
            }
        });

        egui::Grid::new("ai_drivers_grid")
            .striped(true)
            .num_columns(6)
            .show(ui, |ui| {
                ui.strong("Driver ID");
                ui.strong("Name");
                ui.strong("Level / XP");
                ui.strong("Garage");
                ui.strong("Assigned Truck");
                ui.strong("Action");
                ui.end_row();

                for drv in &ai_mgmt.drivers {
                    ui.label(&drv.driver_id);
                    ui.label(drv.display_name.as_deref().unwrap_or(&drv.driver_id));
                    ui.label(format!(
                        "Lv {} ({} XP)",
                        drv.level.unwrap_or(0),
                        drv.experience_points.unwrap_or(0)
                    ));
                    ui.label(drv.garage_name.as_deref().unwrap_or(&drv.garage_id));
                    ui.label(drv.truck_id.as_deref().unwrap_or("None"));
                    if ui
                        .add_enabled(drv.removable, egui::Button::new("Dismiss"))
                        .clicked()
                    {
                        match remove_ai_drivers(
                            AiDriverRemoveRequest {
                                driver_ids: vec![drv.driver_id.clone()],
                                expected_save_hash: ai_mgmt.save_hash.clone(),
                            },
                            app.profile_state(),
                            app.profile_cache(),
                            app.decrypt_cache(),
                            app.app.state(),
                            app.app.state(),
                            app.app_state(),
                        ) {
                            Ok(res) => {
                                app.ai_driver_mgmt = Some(res.management);
                                app.set_status(format!("Removed driver {}.", drv.driver_id), false);
                            }
                            Err(e) => app.set_status(e, true),
                        }
                    }
                    ui.end_row();
                }
            });
    }
}

fn reload_garages_and_drivers(app: &mut Ets2NativeApp) {
    match async_runtime::block_on(get_all_garages(
        app.profile_state(),
        app.decrypt_cache(),
        app.app_state(),
    )) {
        Ok(list) => {
            if app.selected_garage_for_drivers.is_empty() {
                if let Some(first_owned) = list
                    .garages
                    .iter()
                    .find(|g| g.ownership == GarageOwnership::Owned)
                {
                    app.selected_garage_for_drivers = first_owned.garage_id.clone();
                }
            }
            app.garage_list = Some(list);
        }
        Err(e) => app.set_status(format!("Garage load error: {e}"), true),
    }

    match async_runtime::block_on(get_ai_driver_management(
        app.profile_state(),
        app.decrypt_cache(),
        app.app_state(),
    )) {
        Ok(mgmt) => app.ai_driver_mgmt = Some(mgmt),
        Err(e) => app.set_status(format!("AI driver load error: {e}"), true),
    }
}
