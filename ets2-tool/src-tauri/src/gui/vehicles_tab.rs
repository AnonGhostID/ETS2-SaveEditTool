use eframe::egui::{self, Color32};

use crate::features::trailer_change::commands::{
    apply_active_trailer_switch, initialize_trailer_change_session,
    list_owned_trailers_for_switch, preview_active_trailer_switch,
};
use crate::features::truck_change::commands::{
    apply_active_truck_switch, get_official_powertrain_catalog, initialize_truck_change_session,
    list_owned_trucks_for_switch, preview_active_truck_switch, preview_truck_powertrain_change,
};
use crate::features::vehicles::editor::{
    edit_truck_odometer, refuel_player_truck, repair_player_trailer, repair_player_truck,
    set_player_trailer_cargo_mass, set_player_trailer_license_plate, set_player_truck_fuel,
    set_player_truck_license_plate, set_player_truck_wear,
};
use crate::gui::components::{self, UiExt};
use crate::gui::{Ets2NativeApp, VehiclesSubTab};
use crate::tauri::async_runtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FleetTruckSort {
    #[default]
    Brand,
    Odometer,
    Fuel,
    Garage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FleetTrailerSort {
    #[default]
    Body,
    Cargo,
    Wear,
}

pub fn render(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    let (title, subtitle) = match app.vehicles_sub_tab {
        VehiclesSubTab::PlayerVehicleEditor => (
            "Vehicle Editor",
            "Maintain the active truck and trailer: repair, fuel, plates, odometer.",
        ),
        VehiclesSubTab::TruckSwitcher => (
            "Truck Switcher",
            "Swap the active truck for another one you own.",
        ),
        VehiclesSubTab::TrailerSwitcher => (
            "Trailer Switcher",
            "Swap the active trailer for another one you own.",
        ),
        VehiclesSubTab::PowertrainSwapper => (
            "Powertrain",
            "Swap the active truck's engine, gearbox or drivetrain.",
        ),
        VehiclesSubTab::FleetInventory => (
            "Fleet Inventory",
            "Every truck and trailer you own, in one place.",
        ),
    };
    components::page_header(ui, title, subtitle);
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        if components::primary_button(ui, "Reload Vehicles").clicked() {
            app.reload_active_save_data();
            app.set_status("Reloaded player and fleet vehicles.", false);
        }
    });
    ui.add_space(8.0);

    match app.vehicles_sub_tab {
        VehiclesSubTab::PlayerVehicleEditor => render_player_editor(app, ui),
        VehiclesSubTab::TruckSwitcher => render_truck_switcher(app, ui),
        VehiclesSubTab::TrailerSwitcher => render_trailer_switcher(app, ui),
        VehiclesSubTab::PowertrainSwapper => render_powertrain_swapper(app, ui),
        VehiclesSubTab::FleetInventory => render_fleet_inventory(app, ui),
    }
}

fn render_player_editor(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    ui.columns(2, |cols| {
        // Player Truck
        cols[0].card(|ui| {
            ui.heading("Player Truck Maintenance & Customization");
            if let Some(truck) = &app.player_truck {
                ui.label(format!(
                    "ID: {} | Brand/Model: {} {}",
                    truck.truck_id, truck.brand, truck.model
                ));
                ui.label(format!(
                    "Odometer: {:.0} km | Fuel: {:.0}% | Plate: {}",
                    truck.odometer,
                    truck.fuel_relative * 100.0,
                    truck.license_plate.as_deref().unwrap_or("-")
                ));
                ui.label(format!(
                    "Wear — Engine: {:.1}%, Trans: {:.1}%, Cabin: {:.1}%, Chassis: {:.1}%",
                    truck.engine_wear * 100.0,
                    truck.transmission_wear * 100.0,
                    truck.cabin_wear * 100.0,
                    truck.chassis_wear * 100.0
                ));
            } else {
                ui.label("No active player truck loaded yet.");
            }
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.button("🛠 Repair Player Truck (0% Wear)").clicked() {
                    match async_runtime::block_on(repair_player_truck(
                        app.profile_state(),
                        app.decrypt_cache(),
                        app.profile_cache(),
                    )) {
                        Ok(()) => {
                            app.reload_active_save_data();
                            app.set_status("Player truck fully repaired.", false);
                        }
                        Err(e) => app.set_status(format!("Repair failed: {e}"), true),
                    }
                }
                if ui.button("⛽ Refuel Player Truck (100%)").clicked() {
                    match async_runtime::block_on(refuel_player_truck(
                        app.profile_state(),
                        app.decrypt_cache(),
                        app.profile_cache(),
                    )) {
                        Ok(()) => {
                            app.reload_active_save_data();
                            app.set_status("Player truck refueled to 100%.", false);
                        }
                        Err(e) => app.set_status(format!("Refuel failed: {e}"), true),
                    }
                }
            });

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Fuel Level:");
                ui.add(egui::Slider::new(&mut app.truck_fuel_slider, 0.0..=1.0));
                if ui.button("Set Fuel").clicked() {
                    match async_runtime::block_on(set_player_truck_fuel(
                        app.profile_state(),
                        app.decrypt_cache(),
                        app.profile_cache(),
                        app.truck_fuel_slider,
                    )) {
                        Ok(()) => {
                            app.reload_active_save_data();
                            app.set_status("Updated player truck fuel level.", false);
                        }
                        Err(e) => app.set_status(e, true),
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Set Component Wear:");
                ui.add(egui::Slider::new(&mut app.truck_wear_slider, 0.0..=1.0));
                for comp in ["engine", "transmission", "cabin", "chassis", "wheels"] {
                    if ui.small_button(comp).clicked() {
                        let _ = async_runtime::block_on(set_player_truck_wear(
                            app.profile_state(),
                            app.decrypt_cache(),
                            app.profile_cache(),
                            comp.to_string(),
                            app.truck_wear_slider,
                        ));
                        app.reload_active_save_data();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("License Plate:");
                ui.text_edit_singleline(&mut app.truck_plate_input);
                if ui.button("Apply Plate").clicked() {
                    match async_runtime::block_on(set_player_truck_license_plate(
                        app.truck_plate_input.clone(),
                        app.profile_state(),
                        app.decrypt_cache(),
                        app.profile_cache(),
                    )) {
                        Ok(()) => {
                            app.reload_active_save_data();
                            app.set_status("Updated player truck license plate.", false);
                        }
                        Err(e) => app.set_status(e, true),
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Odometer (km):");
                ui.add(egui::DragValue::new(&mut app.truck_odometer_input).range(0..=99_999_999));
                if ui.button("Apply Odometer").clicked() {
                    match async_runtime::block_on(edit_truck_odometer(
                        app.truck_odometer_input,
                        app.profile_state(),
                        app.decrypt_cache(),
                        app.profile_cache(),
                    )) {
                        Ok(()) => {
                            app.reload_active_save_data();
                            app.set_status("Updated truck odometer.", false);
                        }
                        Err(e) => app.set_status(e, true),
                    }
                }
            });
        });

        // Player Trailer
        cols[1].card(|ui| {
            ui.heading("Player Trailer Maintenance & Cargo");
            if let Some(res) = &app.player_trailer_result {
                ui.label(&res.message);
                if let Some(tr) = &res.trailer {
                    ui.label(format!(
                        "ID: {} | Plate: {}",
                        tr.trailer_id,
                        tr.display_license_plate
                            .as_deref()
                            .or(tr.license_plate.as_deref())
                            .unwrap_or("-")
                    ));
                    ui.label(format!(
                        "Cargo Mass: {:.1} kg | Cargo Damage: {:.1}% | Body Wear: {:.1}% | Chassis Wear: {:.1}%",
                        tr.cargo_mass,
                        tr.cargo_damage * 100.0,
                        tr.body_wear * 100.0,
                        tr.chassis_wear * 100.0
                    ));
                }
            } else {
                ui.label("No player trailer loaded.");
            }
            ui.add_space(6.0);

            if ui.button("🛠 Repair Player Trailer (0% Wear & Damage)").clicked() {
                match async_runtime::block_on(repair_player_trailer(
                    app.profile_state(),
                    app.decrypt_cache(),
                    app.profile_cache(),
                )) {
                    Ok(()) => {
                        app.reload_active_save_data();
                        app.set_status("Player trailer repaired.", false);
                    }
                    Err(e) => app.set_status(format!("Trailer repair failed: {e}"), true),
                }
            }

            ui.horizontal(|ui| {
                ui.label("Trailer License Plate:");
                ui.text_edit_singleline(&mut app.trailer_plate_input);
                if ui.button("Apply Plate").clicked() {
                    match async_runtime::block_on(set_player_trailer_license_plate(
                        app.trailer_plate_input.clone(),
                        app.profile_state(),
                        app.decrypt_cache(),
                        app.profile_cache(),
                    )) {
                        Ok(()) => {
                            app.reload_active_save_data();
                            app.set_status("Updated trailer license plate.", false);
                        }
                        Err(e) => app.set_status(e, true),
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Cargo Mass (kg):");
                ui.add(
                    egui::DragValue::new(&mut app.trailer_cargo_mass_input)
                        .speed(100.0)
                        .range(0.0..=200_000.0),
                );
                if ui.button("Set Cargo Mass").clicked() {
                    match async_runtime::block_on(set_player_trailer_cargo_mass(
                        app.trailer_cargo_mass_input,
                        app.profile_state(),
                        app.decrypt_cache(),
                        app.profile_cache(),
                    )) {
                        Ok(()) => {
                            app.reload_active_save_data();
                            app.set_status("Updated trailer cargo mass.", false);
                        }
                        Err(e) => app.set_status(e, true),
                    }
                }
            });
        });
    });
}

fn render_truck_switcher(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("🔍 Scan Owned Trucks for Switch").clicked() {
            let save_path = app.selected_save_path.clone();
            match async_runtime::block_on(initialize_truck_change_session(
                save_path.clone(),
                app.profile_state(),
                app.decrypt_cache(),
                app.app.state(),
            )) {
                Ok(session) => {
                    app.truck_change_session = Some(session);
                    app.truck_switch_list = async_runtime::block_on(list_owned_trucks_for_switch(
                        save_path,
                        app.profile_state(),
                        app.decrypt_cache(),
                    ))
                    .ok();
                    app.set_status("Initialized truck switch session.", false);
                }
                Err(e) => app.set_status(format!("Truck session error: {e}"), true),
            }
        }
    });

    let Some(session) = app.truck_change_session.clone() else {
        components::empty_state(
            ui,
            "No switch session",
            "Scan owned trucks to start a switch session.",
        );
        return;
    };
    ui.add_space(4.0);
    ui.label(format!(
        "Current Active Truck: {} ({} {})",
        session.current_truck.truck_id,
        session.current_truck.brand.as_deref().unwrap_or(""),
        session.current_truck.model.as_deref().unwrap_or(""),
    ));
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Search:");
        ui.text_edit_singleline(&mut app.truck_switch_search);
    });
    ui.add_space(6.0);

    // Master-detail: selectable table left, preview + Apply right.
    ui.columns(2, |cols| {
        let filter = app.truck_switch_search.trim().to_ascii_lowercase();
        egui::ScrollArea::vertical().max_height(420.0).show(&mut cols[0], |ui| {
            egui::Grid::new("truck_switch_grid")
                .striped(true)
                .num_columns(4)
                .show(ui, |ui| {
                    ui.strong("Truck");
                    ui.strong("Plate");
                    ui.strong("Garage");
                    ui.strong("Status");
                    ui.end_row();

                    for item in &session.owned_trucks {
                        let brand_model = format!(
                            "{} {}",
                            item.brand.as_deref().unwrap_or("-"),
                            item.model.as_deref().unwrap_or("-")
                        );
                        if !filter.is_empty()
                            && !brand_model.to_ascii_lowercase().contains(&filter)
                            && !item.truck_id.to_ascii_lowercase().contains(&filter)
                            && !item
                                .display_license_plate
                                .as_deref()
                                .unwrap_or("")
                                .to_ascii_lowercase()
                                .contains(&filter)
                        {
                            continue;
                        }
                        let selected = app.selected_switch_truck_id == item.truck_id;
                        if ui
                            .selectable_label(selected, format!("{brand_model}\n{}", item.truck_id))
                            .clicked()
                        {
                            app.selected_switch_truck_id = item.truck_id.clone();
                            if item.is_active || !item.is_switchable {
                                app.truck_switch_preview = None;
                            } else {
                                match async_runtime::block_on(preview_active_truck_switch(
                                    app.selected_save_path.clone(),
                                    item.truck_id.clone(),
                                    session.save_hash.clone(),
                                    app.profile_state(),
                                    app.decrypt_cache(),
                                )) {
                                    Ok(preview) => app.truck_switch_preview = Some(preview),
                                    Err(e) => app.set_status(format!("Preview failed: {e}"), true),
                                }
                            }
                        }
                        ui.label(item.display_license_plate.as_deref().unwrap_or("-"));
                        ui.label(item.garage_display_name.as_deref().unwrap_or("-"));
                        if item.is_active {
                            ui.colored_label(Color32::from_rgb(88, 214, 141), "Active");
                        } else if item.is_switchable {
                            ui.label("Switchable");
                        } else {
                            ui.colored_label(
                                Color32::from_rgb(231, 76, 60),
                                item.blocked_reason.as_deref().unwrap_or("Blocked"),
                            );
                        }
                        ui.end_row();
                    }
                });
        });

        if let Some(preview) = app.truck_switch_preview.clone() {
            cols[1].card(|ui| {
                ui.heading("Truck Switch Preview");
                ui.label(format!(
                    "Target: {} ({:?})",
                    preview.target_truck.truck_id, preview.mode
                ));
                ui.label(format!(
                    "Safe to write: {} | Can apply: {}",
                    preview.safe_to_write, preview.can_apply
                ));
                for warn in &preview.warnings {
                    ui.colored_label(Color32::from_rgb(245, 176, 65), format!("⚠ {warn}"));
                }
                ui.checkbox(
                    &mut app.create_backup_on_switch,
                    "Create persistent backup before switch",
                );
                ui.add_space(4.0);
                if ui
                    .add_enabled(
                        preview.can_apply,
                        egui::Button::new("Apply Active Truck Switch"),
                    )
                    .clicked()
                {
                    match async_runtime::block_on(apply_active_truck_switch(
                        app.selected_save_path.clone(),
                        preview.target_truck.truck_id.clone(),
                        preview.expected_file_hash.clone(),
                        Some(app.create_backup_on_switch),
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                        app.app.state(),
                    )) {
                        Ok(res) => {
                            app.truck_change_session = Some(res.refreshed_session);
                            app.truck_switch_preview = None;
                            app.reload_active_save_data();
                            app.set_status(
                                format!(
                                    "Switched active truck from {} to {}.",
                                    res.previous_truck_id, res.active_truck_id
                                ),
                                false,
                            );
                        }
                        Err(e) => app.set_status(format!("Truck switch failed: {e}"), true),
                    }
                }
            });
        } else {
            components::empty_state(
                &mut cols[1],
                "No truck selected",
                "Select a switchable truck to preview the switch.",
            );
        }
    });
}

fn render_trailer_switcher(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    if ui.button("🔍 Scan Owned Trailers for Switch").clicked() {
        let save_path = app.selected_save_path.clone();
        match async_runtime::block_on(initialize_trailer_change_session(
            save_path.clone(),
            app.profile_state(),
            app.decrypt_cache(),
            app.app.state(),
        )) {
            Ok(session) => {
                app.trailer_change_session = Some(session);
                app.trailer_switch_list = async_runtime::block_on(list_owned_trailers_for_switch(
                    save_path,
                    app.profile_state(),
                    app.decrypt_cache(),
                ))
                .ok();
                app.set_status("Initialized trailer switch session.", false);
            }
            Err(e) => app.set_status(format!("Trailer session error: {e}"), true),
        }
    }

    let Some(session) = app.trailer_change_session.clone() else {
        components::empty_state(
            ui,
            "No switch session",
            "Scan owned trailers to start a switch session.",
        );
        return;
    };
    ui.add_space(4.0);
    ui.label(format!(
        "Current Active Trailer: {} ({})",
        session.current_trailer.trailer_id, session.current_trailer.display_name
    ));
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Search:");
        ui.text_edit_singleline(&mut app.trailer_switch_search);
    });
    ui.add_space(6.0);

    // Master-detail: selectable table left, preview + Apply right.
    ui.columns(2, |cols| {
        let filter = app.trailer_switch_search.trim().to_ascii_lowercase();
        egui::ScrollArea::vertical().max_height(420.0).show(&mut cols[0], |ui| {
            egui::Grid::new("trailer_switch_grid")
                .striped(true)
                .num_columns(4)
                .show(ui, |ui| {
                    ui.strong("Trailer");
                    ui.strong("Plate");
                    ui.strong("Garage");
                    ui.strong("Status");
                    ui.end_row();

                    for item in &session.owned_trailers {
                        if !filter.is_empty()
                            && !item.display_name.to_ascii_lowercase().contains(&filter)
                            && !item.trailer_id.to_ascii_lowercase().contains(&filter)
                            && !item
                                .display_license_plate
                                .as_deref()
                                .unwrap_or("")
                                .to_ascii_lowercase()
                                .contains(&filter)
                        {
                            continue;
                        }
                        let selected = app.selected_switch_trailer_id == item.trailer_id;
                        if ui
                            .selectable_label(
                                selected,
                                format!("{}\n{}", item.display_name, item.trailer_id),
                            )
                            .clicked()
                        {
                            app.selected_switch_trailer_id = item.trailer_id.clone();
                            if item.is_active || !item.is_switchable {
                                app.trailer_switch_preview = None;
                            } else {
                                match async_runtime::block_on(preview_active_trailer_switch(
                                    app.selected_save_path.clone(),
                                    item.trailer_id.clone(),
                                    session.save_hash.clone(),
                                    app.profile_state(),
                                    app.decrypt_cache(),
                                )) {
                                    Ok(preview) => app.trailer_switch_preview = Some(preview),
                                    Err(e) => {
                                        app.set_status(format!("Trailer preview failed: {e}"), true)
                                    }
                                }
                            }
                        }
                        ui.label(item.display_license_plate.as_deref().unwrap_or("-"));
                        ui.label(item.garage_display_name.as_deref().unwrap_or("-"));
                        if item.is_active {
                            ui.colored_label(Color32::from_rgb(88, 214, 141), "Active");
                        } else if item.is_switchable {
                            ui.label("Switchable");
                        } else {
                            ui.colored_label(Color32::from_rgb(231, 76, 60), "Blocked");
                        }
                        ui.end_row();
                    }
                });
        });

        if let Some(preview) = app.trailer_switch_preview.clone() {
            cols[1].card(|ui| {
                ui.heading("Trailer Switch Preview");
                ui.label(format!(
                    "Target: {} ({})",
                    preview.target_trailer.trailer_id, preview.target_trailer.display_name
                ));
                ui.label(format!(
                    "Safe to write: {} | Can apply: {}",
                    preview.safe_to_write, preview.can_apply
                ));
                for warn in &preview.warnings {
                    ui.colored_label(Color32::from_rgb(245, 176, 65), format!("⚠ {warn}"));
                }
                ui.checkbox(
                    &mut app.create_backup_on_switch,
                    "Create persistent backup before switch",
                );
                ui.add_space(4.0);
                if ui
                    .add_enabled(
                        preview.can_apply,
                        egui::Button::new("Apply Active Trailer Switch"),
                    )
                    .clicked()
                {
                    match async_runtime::block_on(apply_active_trailer_switch(
                        app.selected_save_path.clone(),
                        preview.target_trailer.trailer_id.clone(),
                        preview.expected_file_hash.clone(),
                        Some(app.create_backup_on_switch),
                        app.profile_state(),
                        app.profile_cache(),
                        app.decrypt_cache(),
                        app.app.state(),
                    )) {
                        Ok(res) => {
                            app.trailer_change_session = Some(res.refreshed_session);
                            app.trailer_switch_preview = None;
                            app.reload_active_save_data();
                            app.set_status(
                                format!("Switched active trailer to {}.", res.active_trailer_id),
                                false,
                            );
                        }
                        Err(e) => app.set_status(format!("Trailer switch failed: {e}"), true),
                    }
                }
            });
        } else {
            components::empty_state(
                &mut cols[1],
                "No trailer selected",
                "Select a switchable trailer to preview the switch.",
            );
        }
    });
}

fn render_powertrain_swapper(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    if ui.button("📚 Load Official Powertrain Catalog").clicked() {
        match async_runtime::block_on(get_official_powertrain_catalog(
            Some(app.selected_game.clone()),
            None,
        )) {
            Ok(catalog) => {
                if let Some(first_eng) = catalog.engines.first() {
                    if app.selected_engine_path.is_empty() {
                        app.selected_engine_path = first_eng.data_path.clone();
                    }
                }
                if let Some(first_tx) = catalog.transmissions.first() {
                    if app.selected_transmission_path.is_empty() {
                        app.selected_transmission_path = first_tx.data_path.clone();
                    }
                }
                app.powertrain_catalog = Some(catalog);
                app.set_status("Loaded official powertrain catalog.", false);
            }
            Err(e) => app.set_status(format!("Catalog load error: {e}"), true),
        }
    }

    if let Some(catalog) = app.powertrain_catalog.clone() {
        ui.label(format!(
            "Catalog: {} (v{}) — {} Engines, {} Transmissions",
            catalog.game,
            catalog.game_version,
            catalog.engines.len(),
            catalog.transmissions.len()
        ));

        egui::ComboBox::from_label("Engine (`data_path`)")
            .selected_text(&app.selected_engine_path)
            .width(420.0)
            .show_ui(ui, |ui| {
                for eng in &catalog.engines {
                    let label = format!(
                        "[{} {}] {} ({:.0} HP / {:.0} Nm)",
                        eng.brand,
                        eng.truck_model,
                        eng.name,
                        eng.power.unwrap_or(0.0),
                        eng.torque_nm.unwrap_or(0.0)
                    );
                    ui.selectable_value(&mut app.selected_engine_path, eng.data_path.clone(), label);
                }
            });

        egui::ComboBox::from_label("Transmission (`data_path`)")
            .selected_text(&app.selected_transmission_path)
            .width(420.0)
            .show_ui(ui, |ui| {
                for tx in &catalog.transmissions {
                    let label = format!(
                        "[{} {}] {} ({} gears, diff {:.2})",
                        tx.brand,
                        tx.truck_model,
                        tx.name,
                        tx.gears_forward.unwrap_or(0),
                        tx.differential_ratio.unwrap_or(0.0)
                    );
                    ui.selectable_value(
                        &mut app.selected_transmission_path,
                        tx.data_path.clone(),
                        label,
                    );
                }
            });

        if let Some(truck) = app.player_truck.clone() {
            if ui.button("🔍 Preview Powertrain Swap on Player Truck").clicked() {
                match async_runtime::block_on(preview_truck_powertrain_change(
                    app.selected_save_path.clone(),
                    truck.truck_id,
                    if app.selected_engine_path.is_empty() {
                        None
                    } else {
                        Some(app.selected_engine_path.clone())
                    },
                    if app.selected_transmission_path.is_empty() {
                        None
                    } else {
                        Some(app.selected_transmission_path.clone())
                    },
                    Some(app.selected_game.clone()),
                    None,
                    app.profile_state(),
                    app.decrypt_cache(),
                )) {
                    Ok(prev) => app.powertrain_preview = Some(prev),
                    Err(e) => app.set_status(format!("Powertrain preview failed: {e}"), true),
                }
            }
        }

        if let Some(prev) = &app.powertrain_preview {
            ui.group(|ui| {
                ui.label(format!(
                    "Truck: {} (Family: {:?}) | Cross-Brand Experimental: {}",
                    prev.truck_id, prev.truck_family, prev.experimental_cross_brand
                ));
                ui.label(format!(
                    "Current Engine: {:?} → Selected: {:?}",
                    prev.current_engine.current_data_path,
                    prev.new_engine.as_ref().and_then(|e| e.selected_name.as_ref())
                ));
                ui.label(format!(
                    "Current Transmission: {:?} → Selected: {:?}",
                    prev.current_transmission.current_data_path,
                    prev.new_transmission
                        .as_ref()
                        .and_then(|t| t.selected_name.as_ref())
                ));
                for w in &prev.warnings {
                    ui.colored_label(Color32::from_rgb(245, 176, 65), format!("⚠ {w}"));
                }
            });
        }
    }
}

fn render_fleet_inventory(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Search:");
        ui.text_edit_singleline(&mut app.fleet_search);
    });
    ui.add_space(8.0);

    let filter = app.fleet_search.trim().to_ascii_lowercase();
    let mut trucks: Vec<&crate::models::trucks::ParsedTruck> = app
        .all_trucks
        .iter()
        .filter(|t| {
            filter.is_empty()
                || format!("{} {}", t.brand, t.model)
                    .to_ascii_lowercase()
                    .contains(&filter)
                || t.truck_id.to_ascii_lowercase().contains(&filter)
                || t.license_plate
                    .as_deref()
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .contains(&filter)
        })
        .collect();
    let truck_sort = app.fleet_truck_sort;
    let truck_asc = app.fleet_truck_sort_asc;
    trucks.sort_by(|a, b| {
        let ord = match truck_sort {
            FleetTruckSort::Brand => format!("{} {}", a.brand, a.model)
                .cmp(&format!("{} {}", b.brand, b.model)),
            FleetTruckSort::Odometer => a
                .odometer
                .partial_cmp(&b.odometer)
                .unwrap_or(std::cmp::Ordering::Equal),
            FleetTruckSort::Fuel => a
                .fuel_relative
                .partial_cmp(&b.fuel_relative)
                .unwrap_or(std::cmp::Ordering::Equal),
            FleetTruckSort::Garage => a
                .assigned_garage
                .as_deref()
                .unwrap_or("")
                .cmp(b.assigned_garage.as_deref().unwrap_or("")),
        };
        if truck_asc {
            ord
        } else {
            ord.reverse()
        }
    });

    ui.heading(format!("Owned Trucks ({})", trucks.len()));
    if trucks.is_empty() {
        components::empty_state(
            ui,
            "No trucks match",
            "Try a different search, or reload the save data.",
        );
    } else {
        egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
            egui::Grid::new("all_trucks_grid")
                .striped(true)
                .num_columns(6)
                .show(ui, |ui| {
                    ui.strong("Truck ID");
                    fleet_sort_header(
                        &mut app.fleet_truck_sort,
                        &mut app.fleet_truck_sort_asc,
                        ui,
                        "Brand & Model",
                        FleetTruckSort::Brand,
                    );
                    ui.strong("License Plate");
                    fleet_sort_header(
                        &mut app.fleet_truck_sort,
                        &mut app.fleet_truck_sort_asc,
                        ui,
                        "Odometer (km)",
                        FleetTruckSort::Odometer,
                    );
                    fleet_sort_header(
                        &mut app.fleet_truck_sort,
                        &mut app.fleet_truck_sort_asc,
                        ui,
                        "Fuel %",
                        FleetTruckSort::Fuel,
                    );
                    fleet_sort_header(
                        &mut app.fleet_truck_sort,
                        &mut app.fleet_truck_sort_asc,
                        ui,
                        "Garage",
                        FleetTruckSort::Garage,
                    );
                    ui.end_row();
                    for t in &trucks {
                        ui.label(&t.truck_id);
                        ui.label(format!("{} {}", t.brand, t.model));
                        ui.label(t.license_plate.as_deref().unwrap_or("-"));
                        ui.label(format!("{:.0}", t.odometer));
                        ui.label(format!("{:.0}%", t.fuel_relative * 100.0));
                        ui.label(t.assigned_garage.as_deref().unwrap_or("-"));
                        ui.end_row();
                    }
                });
        });
    }

    ui.add_space(12.0);
    let mut trailers: Vec<&crate::models::trailers::ParsedTrailer> = app
        .all_trailers
        .iter()
        .filter(|tr| {
            filter.is_empty()
                || tr.body_type
                    .as_deref()
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .contains(&filter)
                || tr.trailer_id.to_ascii_lowercase().contains(&filter)
                || tr.display_license_plate
                    .as_deref()
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .contains(&filter)
        })
        .collect();
    let trailer_sort = app.fleet_trailer_sort;
    let trailer_asc = app.fleet_trailer_sort_asc;
    trailers.sort_by(|a, b| {
        let ord = match trailer_sort {
            FleetTrailerSort::Body => a
                .body_type
                .as_deref()
                .unwrap_or("")
                .cmp(b.body_type.as_deref().unwrap_or("")),
            FleetTrailerSort::Cargo => a
                .cargo_mass
                .partial_cmp(&b.cargo_mass)
                .unwrap_or(std::cmp::Ordering::Equal),
            FleetTrailerSort::Wear => (a.body_wear + a.chassis_wear)
                .partial_cmp(&(b.body_wear + b.chassis_wear))
                .unwrap_or(std::cmp::Ordering::Equal),
        };
        if trailer_asc {
            ord
        } else {
            ord.reverse()
        }
    });

    ui.heading(format!("Owned Trailers ({})", trailers.len()));
    if trailers.is_empty() {
        components::empty_state(
            ui,
            "No trailers match",
            "Try a different search, or reload the save data.",
        );
    } else {
        egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
            egui::Grid::new("all_trailers_grid")
                .striped(true)
                .num_columns(5)
                .show(ui, |ui| {
                    ui.strong("Trailer ID");
                    fleet_sort_header(
                        &mut app.fleet_trailer_sort,
                        &mut app.fleet_trailer_sort_asc,
                        ui,
                        "Body Type",
                        FleetTrailerSort::Body,
                    );
                    ui.strong("License Plate");
                    fleet_sort_header(
                        &mut app.fleet_trailer_sort,
                        &mut app.fleet_trailer_sort_asc,
                        ui,
                        "Cargo Mass (kg)",
                        FleetTrailerSort::Cargo,
                    );
                    fleet_sort_header(
                        &mut app.fleet_trailer_sort,
                        &mut app.fleet_trailer_sort_asc,
                        ui,
                        "Body / Chassis Wear",
                        FleetTrailerSort::Wear,
                    );
                    ui.end_row();
                    for tr in &trailers {
                        ui.label(&tr.trailer_id);
                        ui.label(tr.body_type.as_deref().unwrap_or("-"));
                        ui.label(
                            tr.display_license_plate
                                .as_deref()
                                .or(tr.license_plate.as_deref())
                                .unwrap_or("-"),
                        );
                        ui.label(format!("{:.1}", tr.cargo_mass));
                        ui.label(format!(
                            "{:.1}% / {:.1}%",
                            tr.body_wear * 100.0,
                            tr.chassis_wear * 100.0
                        ));
                        ui.end_row();
                    }
                });
        });
    }
}

/// A clickable table header that toggles sorting by `key`.
fn fleet_sort_header<T: Copy + PartialEq>(
    current: &mut T,
    ascending: &mut bool,
    ui: &mut egui::Ui,
    label: &str,
    key: T,
) {
    let arrow = if *current == key {
        if *ascending {
            " ▲"
        } else {
            " ▼"
        }
    } else {
        ""
    };
    if ui.button(format!("{label}{arrow}")).clicked() {
        if *current == key {
            *ascending = !*ascending;
        } else {
            *current = key;
            *ascending = true;
        }
    }
}
