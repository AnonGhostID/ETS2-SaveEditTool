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
use crate::gui::{Ets2NativeApp, VehiclesSubTab};
use crate::tauri::async_runtime;

pub fn render(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("🚛 Vehicles — Player Truck & Trailer, Switcher & Powertrain");
        if ui.button("⟳ Reload Vehicles").clicked() {
            app.reload_active_save_data();
            app.set_status("Reloaded player and fleet vehicles.", false);
        }
    });

    ui.horizontal(|ui| {
        for (sub, label) in [
            (VehiclesSubTab::PlayerVehicleEditor, "🔧 Active Truck & Trailer Editor"),
            (VehiclesSubTab::TruckSwitcher, "🔄 Active Truck Switcher"),
            (VehiclesSubTab::TrailerSwitcher, "🔄 Active Trailer Switcher"),
            (VehiclesSubTab::PowertrainSwapper, "⚙ Engine & Transmission Catalog"),
            (VehiclesSubTab::FleetInventory, "📋 Full Fleet Inventory"),
        ] {
            if ui
                .selectable_label(app.vehicles_sub_tab == sub, label)
                .clicked()
            {
                app.vehicles_sub_tab = sub;
            }
        }
    });
    ui.separator();

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
        cols[0].group(|ui| {
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
        cols[1].group(|ui| {
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
        ui.checkbox(
            &mut app.create_backup_on_switch,
            "Create persistent backup before switch",
        );
    });

    if let Some(session) = app.truck_change_session.clone() {
        ui.label(format!(
            "Current Active Truck: {} ({} {}) | Save Hash: {}",
            session.current_truck.truck_id,
            session.current_truck.brand.as_deref().unwrap_or(""),
            session.current_truck.model.as_deref().unwrap_or(""),
            session.save_hash
        ));
        ui.separator();

        egui::Grid::new("truck_switch_grid")
            .striped(true)
            .num_columns(7)
            .show(ui, |ui| {
                ui.strong("Truck ID");
                ui.strong("Brand / Model");
                ui.strong("Plate");
                ui.strong("Garage");
                ui.strong("Driver");
                ui.strong("Status");
                ui.strong("Action");
                ui.end_row();

                for item in &session.owned_trucks {
                    ui.label(&item.truck_id);
                    ui.label(format!(
                        "{} {}",
                        item.brand.as_deref().unwrap_or("-"),
                        item.model.as_deref().unwrap_or("-")
                    ));
                    ui.label(item.display_license_plate.as_deref().unwrap_or("-"));
                    ui.label(item.garage_display_name.as_deref().unwrap_or("-"));
                    ui.label(item.driver_display_name.as_deref().unwrap_or("Free"));
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

                    if ui
                        .add_enabled(!item.is_active, egui::Button::new("Preview Switch"))
                        .clicked()
                    {
                        app.selected_switch_truck_id = item.truck_id.clone();
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
                    ui.end_row();
                }
            });

        if let Some(preview) = app.truck_switch_preview.clone() {
            ui.separator();
            ui.group(|ui| {
                ui.heading("Truck Switch Preview");
                ui.label(format!(
                    "Target: {} ({:?}) | Safe to write: {} | Can apply: {}",
                    preview.target_truck.truck_id,
                    preview.mode,
                    preview.safe_to_write,
                    preview.can_apply
                ));
                for warn in &preview.warnings {
                    ui.colored_label(Color32::from_rgb(245, 176, 65), format!("⚠ {warn}"));
                }
                if ui
                    .add_enabled(
                        preview.can_apply,
                        egui::Button::new("✅ Apply Active Truck Switch"),
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
        }
    }
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

    if let Some(session) = app.trailer_change_session.clone() {
        ui.label(format!(
            "Current Active Trailer: {} ({}) | Save Hash: {}",
            session.current_trailer.trailer_id, session.current_trailer.display_name, session.save_hash
        ));
        ui.separator();

        egui::Grid::new("trailer_switch_grid")
            .striped(true)
            .num_columns(6)
            .show(ui, |ui| {
                ui.strong("Trailer ID");
                ui.strong("Name");
                ui.strong("Plate");
                ui.strong("Garage");
                ui.strong("Status");
                ui.strong("Action");
                ui.end_row();

                for item in &session.owned_trailers {
                    ui.label(&item.trailer_id);
                    ui.label(&item.display_name);
                    ui.label(item.display_license_plate.as_deref().unwrap_or("-"));
                    ui.label(item.garage_display_name.as_deref().unwrap_or("-"));
                    ui.label(if item.is_active {
                        "Active"
                    } else if item.is_switchable {
                        "Switchable"
                    } else {
                        "Blocked"
                    });
                    if ui
                        .add_enabled(!item.is_active, egui::Button::new("Preview Switch"))
                        .clicked()
                    {
                        match async_runtime::block_on(preview_active_trailer_switch(
                            app.selected_save_path.clone(),
                            item.trailer_id.clone(),
                            session.save_hash.clone(),
                            app.profile_state(),
                            app.decrypt_cache(),
                        )) {
                            Ok(preview) => app.trailer_switch_preview = Some(preview),
                            Err(e) => app.set_status(format!("Trailer preview failed: {e}"), true),
                        }
                    }
                    ui.end_row();
                }
            });

        if let Some(preview) = app.trailer_switch_preview.clone() {
            ui.separator();
            ui.group(|ui| {
                ui.heading("Trailer Switch Preview");
                ui.label(format!(
                    "Target: {} ({}) | Safe to write: {}",
                    preview.target_trailer.trailer_id,
                    preview.target_trailer.display_name,
                    preview.safe_to_write
                ));
                if ui
                    .add_enabled(
                        preview.can_apply,
                        egui::Button::new("✅ Apply Active Trailer Switch"),
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
        }
    }
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
    ui.heading(format!("Owned Trucks ({})", app.all_trucks.len()));
    egui::Grid::new("all_trucks_grid")
        .striped(true)
        .num_columns(6)
        .show(ui, |ui| {
            ui.strong("Truck ID");
            ui.strong("Brand & Model");
            ui.strong("License Plate");
            ui.strong("Odometer (km)");
            ui.strong("Fuel %");
            ui.strong("Garage");
            ui.end_row();
            for t in &app.all_trucks {
                ui.label(&t.truck_id);
                ui.label(format!("{} {}", t.brand, t.model));
                ui.label(t.license_plate.as_deref().unwrap_or("-"));
                ui.label(format!("{:.0}", t.odometer));
                ui.label(format!("{:.0}%", t.fuel_relative * 100.0));
                ui.label(t.assigned_garage.as_deref().unwrap_or("-"));
                ui.end_row();
            }
        });

    ui.add_space(12.0);
    ui.heading(format!("Owned Trailers ({})", app.all_trailers.len()));
    egui::Grid::new("all_trailers_grid")
        .striped(true)
        .num_columns(5)
        .show(ui, |ui| {
            ui.strong("Trailer ID");
            ui.strong("Body Type");
            ui.strong("License Plate");
            ui.strong("Cargo Mass (kg)");
            ui.strong("Body / Chassis Wear");
            ui.end_row();
            for tr in &app.all_trailers {
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
}
