use eframe::egui::{self, Color32};
use egui_extras::{Column, TableBuilder};

use crate::features::garages::commands::{
    assign_random_garage_resources, buy_all_garages, generate_ai_drivers, get_ai_driver_management,
    get_all_garages, purchase_garage, relinquish_empty_garages, relinquish_garage_ownership,
    remove_ai_drivers, update_garage, upgrade_owned_garage,
};
use crate::features::garages::models::{
    AiDriverGenerateRequest, AiDriverManagementResult, AiDriverRemoveRequest, GarageBuyAllRequest,
    GarageInfo, GarageListResult, GarageMutationRequest, GarageOwnership,
    GarageRelinquishEmptyRequest, GarageResourceAssignmentRequest, GarageSize,
    GarageUpdateRequest,
};
use crate::features::trailer_change::cache::TrailerChangeSessionCache;
use crate::features::truck_change::cache::TruckChangeSessionCache;
use crate::gui::components::{self, UiExt};
use crate::gui::{Ets2NativeApp, theme};
use crate::state::{AppProfileState, AppState, DecryptCache, ProfileCache};
use crate::tauri::{State, async_runtime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GarageSortKey {
    #[default]
    City,
    Ownership,
    Size,
    Slots,
    Drivers,
}

/// A destructive garage/driver action waiting on the confirm dialog.
#[derive(Debug, Clone)]
pub enum GarageConfirm {
    SellGarage { garage_id: String, label: String },
    SellEmpty { count: usize },
    BuyAll { count: usize },
    DismissDriver { driver_id: String, name: String },
}

/// Result of the background garages + drivers load.
pub type GarageLoadResult = Result<(GarageListResult, AiDriverManagementResult), String>;

pub fn render(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    components::page_header(
        ui,
        "Garages & Drivers",
        "Manage garage ownership, upgrades, headquarters and AI driver assignments.",
    );
    ui.add_space(8.0);

    // Pick up a finished background load, if one arrived.
    if let Some(result) = app.garages_load.poll() {
        match result {
            Ok((list, mgmt)) => {
                ensure_default_driver_garage(app, &list);
                app.garage_list = Some(list);
                app.ai_driver_mgmt = Some(mgmt);
                app.set_status("Loaded garages and AI drivers.", false);
            }
            Err(e) => app.set_status(format!("Garage load error: {e}"), true),
        }
    }
    if app.garages_load.is_loading() {
        // Keep repainting so the spinner animates and the result is
        // picked up promptly.
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(100));
    }

    ui.horizontal_wrapped(|ui| {
        let loading = app.garages_load.is_loading();
        if ui
            .add_enabled(!loading, egui::Button::new("Load Garages & Drivers"))
            .clicked()
        {
            start_garages_load(app);
        }
        if loading {
            ui.spinner();
            ui.label("Loading garages…");
        }
    });
    ui.add_space(8.0);

    let Some(garages_res) = app.garage_list.clone() else {
        components::empty_state(
            ui,
            "No garage data loaded",
            "Load garages and AI drivers to inspect and edit the active save.",
        );
        return;
    };

    // Stats strip.
    let hq_label = garages_res
        .headquarters_garage_id
        .as_deref()
        .and_then(|hq| {
            garages_res
                .garages
                .iter()
                .find(|g| g.garage_id == hq)
                .map(|g| city_label(g).to_string())
        })
        .unwrap_or_else(|| "-".to_string());
    let driver_count = app
        .ai_driver_mgmt
        .as_ref()
        .map(|mgmt| mgmt.drivers.len())
        .unwrap_or(0);
    ui.columns(4, |cols| {
        components::stat_tile(
            &mut cols[0],
            "Owned",
            &garages_res.diagnostics.owned_garage_count.to_string(),
            theme::SUCCESS,
        );
        components::stat_tile(
            &mut cols[1],
            "Unowned",
            &garages_res.diagnostics.not_owned_garage_count.to_string(),
            theme::MUTED,
        );
        components::stat_tile(&mut cols[2], "Headquarters", &hq_label, theme::ACCENT);
        components::stat_tile(
            &mut cols[3],
            "AI Drivers",
            &driver_count.to_string(),
            theme::ACCENT,
        );
    });
    ui.add_space(10.0);

    // Toolbar: search + filter + bulk actions.
    let empty_owned_count = garages_res
        .garages
        .iter()
        .filter(|g| {
            g.ownership == GarageOwnership::Owned
                && g.assigned_driver_count == 0
                && g.assigned_truck_count == 0
        })
        .count();
    let unowned_count = garages_res.diagnostics.not_owned_garage_count;
    ui.horizontal_wrapped(|ui| {
        ui.label("Search:");
        ui.text_edit_singleline(&mut app.garage_filter_text);
        ui.checkbox(&mut app.garage_only_owned, "Owned only");
        ui.separator();
        if ui
            .add_enabled(
                unowned_count > 0,
                egui::Button::new(format!("Buy All Garages ({unowned_count})")),
            )
            .clicked()
        {
            app.garage_confirm = Some(GarageConfirm::BuyAll {
                count: unowned_count,
            });
        }
        if ui
            .add_enabled(
                empty_owned_count > 0,
                egui::Button::new(format!("Sell Empty Garages ({empty_owned_count})")),
            )
            .clicked()
        {
            app.garage_confirm = Some(GarageConfirm::SellEmpty {
                count: empty_owned_count,
            });
        }
    });
    ui.add_space(8.0);

    // Filter + sort.
    let filter_lower = app.garage_filter_text.trim().to_ascii_lowercase();
    let mut visible: Vec<&GarageInfo> = garages_res
        .garages
        .iter()
        .filter(|g| {
            (!app.garage_only_owned || g.ownership == GarageOwnership::Owned)
                && (filter_lower.is_empty()
                    || city_label(g).to_ascii_lowercase().contains(&filter_lower)
                    || g.garage_id.to_ascii_lowercase().contains(&filter_lower))
        })
        .collect();
    let sort_key = app.garage_sort_key;
    let sort_asc = app.garage_sort_asc;
    visible.sort_by(|a, b| {
        let ord = match sort_key {
            GarageSortKey::City => city_label(a).cmp(city_label(b)),
            GarageSortKey::Ownership => {
                ownership_rank(&a.ownership).cmp(&ownership_rank(&b.ownership))
            }
            GarageSortKey::Size => size_rank(&a.size).cmp(&size_rank(&b.size)),
            GarageSortKey::Slots => a.occupied_slots.cmp(&b.occupied_slots),
            GarageSortKey::Drivers => a
                .assigned_driver_count
                .cmp(&b.assigned_driver_count),
        };
        if sort_asc {
            ord
        } else {
            ord.reverse()
        }
    });

    if visible.is_empty() {
        components::empty_state(
            ui,
            "No garages match",
            "Try a different search, or turn off the Owned-only filter.",
        );
    } else {
        // Selectable table with a sticky header, in its own scroll
        // area so the detail card below stays reachable with 200+
        // garages. TableBuilder keeps the header visible while the
        // rows scroll. Columns use fixed widths (not auto): auto
        // columns freeze at their first-frame 100px suggestion in
        // egui_extras 0.29, which wraps the wider sort-header
        // buttons; the city column takes the remaining width.
        TableBuilder::new(ui)
            .id_salt("garages_table")
            .striped(true)
            .max_scroll_height(420.0)
            .column(Column::remainder())
            .column(Column::initial(70.0))
            .column(Column::initial(115.0))
            .column(Column::initial(70.0))
            .column(Column::initial(155.0))
            .column(Column::initial(150.0))
            .header(20.0, |mut header| {
                header.col(|ui| sort_header(app, ui, "Garage / City", GarageSortKey::City));
                header.col(|ui| {
                    ui.strong("Country");
                });
                header.col(|ui| sort_header(app, ui, "Ownership", GarageSortKey::Ownership));
                header.col(|ui| sort_header(app, ui, "Size", GarageSortKey::Size));
                header.col(|ui| sort_header(app, ui, "Slots (Used/Max)", GarageSortKey::Slots));
                header.col(|ui| {
                    sort_header(app, ui, "Drivers / Trucks", GarageSortKey::Drivers)
                });
            })
            .body(|body| {
                body.rows(20.0, visible.len(), |mut row| {
                    let g = &visible[row.index()];
                    let selected =
                        app.selected_garage_id.as_deref() == Some(g.garage_id.as_str());
                    let label = format!(
                        "{}{}",
                        city_label(g),
                        if g.is_headquarters { " ⭐ (HQ)" } else { "" }
                    );
                    row.col(|ui| {
                        if ui.selectable_label(selected, label).clicked() {
                            app.selected_garage_id = Some(g.garage_id.clone());
                        }
                    });
                    row.col(|ui| {
                        ui.label(g.country_code.as_deref().unwrap_or("-"));
                    });
                    row.col(|ui| {
                        if g.ownership == GarageOwnership::Owned {
                            ui.colored_label(Color32::from_rgb(88, 214, 141), "Owned");
                        } else {
                            ui.label("Unowned");
                        }
                    });
                    row.col(|ui| {
                        ui.label(format!("{:?}", g.size));
                    });
                    row.col(|ui| {
                        ui.label(format!("{}/{}", g.occupied_slots, g.maximum_slot_count));
                    });
                    row.col(|ui| {
                        ui.label(format!(
                            "{} drv / {} trk",
                            g.assigned_driver_count, g.assigned_truck_count
                        ));
                    });
                });
            });
    }

    // Detail card for the selected garage.
    if let Some(selected_id) = app.selected_garage_id.clone() {
        if let Some(g) = garages_res
            .garages
            .iter()
            .find(|g| g.garage_id == selected_id)
        {
            render_garage_detail(app, ui, g, &garages_res.save_hash);
        }
    }

    // AI Drivers section.
    if let Some(ai_mgmt) = app.ai_driver_mgmt.clone() {
        ui.add_space(12.0);
        ui.separator();
        ui.heading(format!(
            "AI Drivers ({} Active | {} Available Pool)",
            ai_mgmt.drivers.len(),
            ai_mgmt.available_driver_count
        ));
        ui.add_space(6.0);

        ui.horizontal_wrapped(|ui| {
            ui.label("Garage ID:");
            ui.text_edit_singleline(&mut app.selected_garage_for_drivers);
            ui.label("Count:");
            ui.add(egui::DragValue::new(&mut app.generate_drivers_count).range(1..=5));
            ui.checkbox(&mut app.generate_drivers_assign_trucks, "Also assign trucks");
            if ui.button("Generate AI Drivers").clicked()
                && !app.selected_garage_for_drivers.trim().is_empty()
            {
                let garage_id = app.selected_garage_for_drivers.trim().to_string();
                let save_hash = ai_mgmt.save_hash.clone();
                let count = app.generate_drivers_count;
                let assign_trucks = app.generate_drivers_assign_trucks;
                run_garage_command(
                    app,
                    move |ps, pc, dc, tc, tr, st| {
                        generate_ai_drivers(
                            AiDriverGenerateRequest {
                                garage_id,
                                expected_save_hash: save_hash,
                                count,
                                assign_trucks,
                            },
                            ps, pc, dc, tc, tr, st,
                        )
                    },
                    |res| format!("Generated {} AI driver(s).", res.affected_driver_ids.len()),
                );
            }
        });
        ui.add_space(6.0);

        if ai_mgmt.drivers.is_empty() {
            components::empty_state(
                ui,
                "No AI drivers",
                "Generate drivers for an owned garage to put them to work.",
            );
        } else {
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
                            app.garage_confirm = Some(GarageConfirm::DismissDriver {
                                driver_id: drv.driver_id.clone(),
                                name: drv
                                    .display_name
                                    .clone()
                                    .unwrap_or_else(|| drv.driver_id.clone()),
                            });
                        }
                        ui.end_row();
                    }
                });
        }
    }

    render_garage_confirm(app, ui);
}

fn city_label(g: &GarageInfo) -> &str {
    g.city_name
        .as_deref()
        .or(g.city_token.as_deref())
        .unwrap_or(&g.garage_id)
}

fn ownership_rank(ownership: &GarageOwnership) -> u8 {
    match ownership {
        GarageOwnership::Owned => 0,
        GarageOwnership::NotOwned => 1,
        GarageOwnership::Unknown => 2,
    }
}

fn size_rank(size: &GarageSize) -> u8 {
    match size {
        GarageSize::Unowned => 0,
        GarageSize::Small => 1,
        GarageSize::Large => 2,
        GarageSize::Unknown => 3,
    }
}

fn sort_header(app: &mut Ets2NativeApp, ui: &mut egui::Ui, label: &str, key: GarageSortKey) {
    let arrow = if app.garage_sort_key == key {
        if app.garage_sort_asc {
            " ^"
        } else {
            " v"
        }
    } else {
        ""
    };
    if ui.button(format!("{label}{arrow}")).clicked() {
        if app.garage_sort_key == key {
            app.garage_sort_asc = !app.garage_sort_asc;
        } else {
            app.garage_sort_key = key;
            app.garage_sort_asc = true;
        }
    }
}

fn render_garage_detail(
    app: &mut Ets2NativeApp,
    ui: &mut egui::Ui,
    g: &GarageInfo,
    save_hash: &str,
) {
    ui.add_space(10.0);
    let label = city_label(g).to_string();
    let garage_id = g.garage_id.clone();
    let owned = g.ownership == GarageOwnership::Owned;
    let is_hq = g.is_headquarters;
    let save_hash = save_hash.to_string();
    ui.card(|ui| {
        ui.horizontal(|ui| {
            ui.heading(format!(
                "{}{}",
                label,
                if is_hq { " ⭐ Headquarters" } else { "" }
            ));
            ui.label(
                egui::RichText::new(&garage_id)
                    .size(11.0)
                    .color(theme::MUTED),
            );
        });
        ui.add_space(4.0);
        egui::Grid::new("garage_detail_grid")
            .num_columns(2)
            .spacing([24.0, 4.0])
            .show(ui, |ui| {
                ui.label("Ownership:");
                ui.label(if owned { "Owned" } else { "Unowned" });
                ui.end_row();
                ui.label("Size:");
                ui.label(format!("{:?}", g.size));
                ui.end_row();
                ui.label("Slots:");
                ui.label(format!(
                    "{} used / {} max ({} available)",
                    g.occupied_slots, g.maximum_slot_count, g.available_slots
                ));
                ui.end_row();
                ui.label("Drivers / Trucks / Trailers:");
                ui.label(format!(
                    "{} / {} / {}",
                    g.assigned_driver_count, g.assigned_truck_count, g.assigned_trailer_count
                ));
                ui.end_row();
            });
        if !g.warnings.is_empty() {
            ui.add_space(4.0);
            for warning in &g.warnings {
                ui.label(
                    egui::RichText::new(format!("⚠ {warning}"))
                        .size(11.5)
                        .color(theme::WARNING),
                );
            }
        }
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if !owned {
                let id = garage_id.clone();
                let hash = save_hash.clone();
                let name = label.clone();
                if components::primary_button(ui, "Buy Garage").clicked() {
                    run_garage_command(
                        app,
                        move |ps, pc, dc, tc, tr, st| {
                            purchase_garage(
                                GarageMutationRequest {
                                    garage_id: id,
                                    expected_save_hash: hash,
                                },
                                ps, pc, dc, tc, tr, st,
                            )
                        },
                        move |_| format!("Purchased {name}."),
                    );
                }
            } else {
                let id = garage_id.clone();
                let hash = save_hash.clone();
                let name = label.clone();
                if ui.button("Upgrade").clicked() {
                    run_garage_command(
                        app,
                        move |ps, pc, dc, tc, tr, st| {
                            upgrade_owned_garage(
                                GarageMutationRequest {
                                    garage_id: id,
                                    expected_save_hash: hash,
                                },
                                ps, pc, dc, tc, tr, st,
                            )
                        },
                        move |_| format!("Upgraded {name}."),
                    );
                }
                let id = garage_id.clone();
                let hash = save_hash.clone();
                let name = label.clone();
                if ui.button("Assign Driver + Truck").clicked() {
                    run_garage_command(
                        app,
                        move |ps, pc, dc, tc, tr, st| {
                            assign_random_garage_resources(
                                GarageResourceAssignmentRequest {
                                    garage_id: id,
                                    expected_save_hash: hash,
                                    assign_random_driver: true,
                                    assign_random_truck: true,
                                },
                                ps, pc, dc, tc, tr, st,
                            )
                        },
                        move |_| format!("Assigned driver & truck to {name}."),
                    );
                }
                if !is_hq {
                    let id = garage_id.clone();
                    let hash = save_hash.clone();
                    let name = label.clone();
                    if ui.button("Set as HQ").clicked() {
                        run_garage_command(
                            app,
                            move |ps, pc, dc, tc, tr, st| {
                                update_garage(
                                    GarageUpdateRequest {
                                        garage_id: id,
                                        expected_save_hash: hash,
                                        target_size: None,
                                        set_as_headquarters: true,
                                    },
                                    ps, pc, dc, tc, tr, st,
                                )
                            },
                            move |_| format!("Set {name} as headquarters."),
                        );
                    }
                }
                if ui.button("Sell").clicked() {
                    app.garage_confirm = Some(GarageConfirm::SellGarage {
                        garage_id: garage_id.clone(),
                        label: label.clone(),
                    });
                }
            }
        });
    });
}

fn render_garage_confirm(app: &mut Ets2NativeApp, ui: &mut egui::Ui) {
    let Some(confirm) = app.garage_confirm.clone() else {
        return;
    };
    let (title, message, confirm_text) = match &confirm {
        GarageConfirm::SellGarage { label, .. } => (
            "Sell garage",
            format!(
                "Sell {label}? The garage and its assigned drivers and trucks go with it. You can buy it back later."
            ),
            "Sell",
        ),
        GarageConfirm::SellEmpty { count } => (
            "Sell empty garages",
            format!(
                "Sell all {count} empty garages? Only garages with no drivers or trucks are affected."
            ),
            "Sell all",
        ),
        GarageConfirm::BuyAll { count } => (
            "Buy all garages",
            format!(
                "Buy all {count} unowned garages? This spends a large amount of money."
            ),
            "Buy all",
        ),
        GarageConfirm::DismissDriver { name, .. } => (
            "Dismiss driver",
            format!("Dismiss {name}? This removes the driver from your company."),
            "Dismiss",
        ),
    };
    match components::confirm_dialog(ui.ctx(), title, &message, confirm_text, "Cancel") {
        Some(true) => {
            app.garage_confirm = None;
            run_garage_confirm(app, &confirm);
        }
        Some(false) => app.garage_confirm = None,
        None => {}
    }
}

fn run_garage_confirm(app: &mut Ets2NativeApp, confirm: &GarageConfirm) {
    match confirm {
        GarageConfirm::SellGarage { garage_id, label } => {
            let Some(save_hash) = app
                .garage_list
                .as_ref()
                .map(|list| list.save_hash.clone())
            else {
                return;
            };
            let id = garage_id.clone();
            let name = label.clone();
            run_garage_command(
                app,
                move |ps, pc, dc, tc, tr, st| {
                    relinquish_garage_ownership(
                        GarageMutationRequest {
                            garage_id: id,
                            expected_save_hash: save_hash,
                        },
                        ps, pc, dc, tc, tr, st,
                    )
                },
                move |_| format!("Relinquished {name}."),
            );
        }
        GarageConfirm::SellEmpty { .. } => {
            let Some(save_hash) = app
                .garage_list
                .as_ref()
                .map(|list| list.save_hash.clone())
            else {
                return;
            };
            run_garage_command(
                app,
                move |ps, pc, dc, tc, tr, st| {
                    relinquish_empty_garages(
                        GarageRelinquishEmptyRequest {
                            expected_save_hash: save_hash,
                        },
                        ps, pc, dc, tc, tr, st,
                    )
                },
                |res| format!("Relinquished {} empty garages.", res.relinquished_count),
            );
        }
        GarageConfirm::BuyAll { .. } => {
            let Some(save_hash) = app
                .garage_list
                .as_ref()
                .map(|list| list.save_hash.clone())
            else {
                return;
            };
            run_garage_command(
                app,
                move |ps, pc, dc, tc, tr, st| {
                    buy_all_garages(
                        GarageBuyAllRequest {
                            expected_save_hash: save_hash,
                        },
                        ps, pc, dc, tc, tr, st,
                    )
                },
                |res| format!("Purchased {} garages.", res.purchased_count),
            );
        }
        GarageConfirm::DismissDriver { driver_id, name } => {
            let Some(save_hash) = app
                .ai_driver_mgmt
                .as_ref()
                .map(|mgmt| mgmt.save_hash.clone())
            else {
                return;
            };
            let id = driver_id.clone();
            let name = name.clone();
            run_garage_command(
                app,
                move |ps, pc, dc, tc, tr, st| {
                    remove_ai_drivers(
                        AiDriverRemoveRequest {
                            driver_ids: vec![id],
                            expected_save_hash: save_hash,
                        },
                        ps, pc, dc, tc, tr, st,
                    )
                },
                move |_| format!("Removed driver {name}."),
            );
        }
    }
}

/// Run a garage/driver command with the standard state arguments, then
/// reload the garage/driver data and report the outcome as a toast.
fn run_garage_command<T>(
    app: &mut Ets2NativeApp,
    run: impl FnOnce(
        State<'_, AppProfileState>,
        State<'_, ProfileCache>,
        State<'_, DecryptCache>,
        State<'_, TruckChangeSessionCache>,
        State<'_, TrailerChangeSessionCache>,
        State<'_, AppState>,
    ) -> Result<T, String>,
    success_message: impl FnOnce(&T) -> String,
) {
    let result = run(
        app.profile_state(),
        app.profile_cache(),
        app.decrypt_cache(),
        app.app.state(),
        app.app.state(),
        app.app_state(),
    );
    match result {
        Ok(value) => {
            let message = success_message(&value);
            reload_garages_and_drivers(app);
            app.set_status(message, false);
        }
        Err(error) => app.set_status(error, true),
    }
}

/// Load garages + drivers on a background thread (the explicit Load
/// button only; post-mutation refreshes stay synchronous because the
/// next edit needs the fresh save hash immediately).
fn start_garages_load(app: &mut Ets2NativeApp) {
    let handle = app.app.clone();
    app.garages_load.start(move || {
        let profile = handle.state::<AppProfileState>();
        let decrypt = handle.state::<DecryptCache>();
        let app_state = handle.state::<AppState>();
        let list = async_runtime::block_on(get_all_garages(profile, decrypt, app_state));
        let mgmt = async_runtime::block_on(get_ai_driver_management(profile, decrypt, app_state));
        match (list, mgmt) {
            (Ok(list), Ok(mgmt)) => Ok((list, mgmt)),
            (Err(e), _) | (_, Err(e)) => Err(e),
        }
    });
}

fn ensure_default_driver_garage(app: &mut Ets2NativeApp, list: &GarageListResult) {
    if app.selected_garage_for_drivers.is_empty() {
        if let Some(first_owned) = list
            .garages
            .iter()
            .find(|g| g.ownership == GarageOwnership::Owned)
        {
            app.selected_garage_for_drivers = first_owned.garage_id.clone();
        }
    }
}

fn reload_garages_and_drivers(app: &mut Ets2NativeApp) {
    match async_runtime::block_on(get_all_garages(
        app.profile_state(),
        app.decrypt_cache(),
        app.app_state(),
    )) {
        Ok(list) => {
            ensure_default_driver_garage(app, &list);
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
