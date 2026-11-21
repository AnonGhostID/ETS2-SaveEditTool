//! Vehicles sector screen: player truck/trailer, truck and trailer switchers,
//! the read-only official powertrain catalog and the fleet inventory.
//!
//! Only shared components and typed data wrappers are used. Anything that
//! belongs to the shell (navigation, toasts, Issues, Save, context changes) is
//! requested through a `ShellCommand`. No backend function is called here.

use eframe::egui;

use crate::gui::components as comp;
use crate::gui::screens::{Route, ScreenCtx, ShellCommand, SubView};
use crate::gui::theme;

// -------------------------------------------------------------- local DTO views

#[derive(Debug, Clone)]
struct TruckView {
    id: String,
    brand: String,
    model: String,
    odometer: f32,
    fuel: f32,
    plate: String,
    wear: Vec<(String, f32)>,
}

#[derive(Debug, Clone)]
struct TrailerView {
    id: String,
    plate: String,
    cargo_mass: f32,
    cargo_damage: f32,
    body_wear: f32,
    chassis_wear: f32,
    message: String,
}

#[derive(Debug, Clone)]
struct SwitchRowView {
    id: String,
    label: String,
    sub: String,
    plate: String,
    garage: String,
    status: String,
    active: bool,
    switchable: bool,
}

#[derive(Debug, Clone, Default)]
struct SwitchSessionView {
    save_hash: String,
    current_label: String,
    warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct SwitchPreviewView {
    target_id: String,
    target_label: String,
    mode: String,
    warnings: Vec<String>,
    error_code: Option<String>,
    expected_hash: String,
    safe_to_write: bool,
    can_apply: bool,
    details: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
struct SwitchResultView {
    success: bool,
    backup_id: Option<String>,
    persistent_backup: bool,
    previous_id: String,
    active_id: String,
    validation_errors: Vec<String>,
    hash_before: String,
    hash_after: String,
}

#[derive(Debug, Clone)]
struct CatalogEntryView {
    label: String,
    data_path: String,
}

#[derive(Debug, Clone)]
struct CatalogView {
    game: String,
    version: String,
    engines: Vec<CatalogEntryView>,
    transmissions: Vec<CatalogEntryView>,
}

#[derive(Debug, Clone)]
struct PowertrainPreviewView {
    truck_id: String,
    family: Option<String>,
    cross_brand: bool,
    current_engine: Option<String>,
    new_engine: Option<String>,
    current_transmission: Option<String>,
    new_transmission: Option<String>,
    differential: Option<f64>,
    can_apply_later: bool,
    warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct FleetTruckView {
    id: String,
    brand_model: String,
    plate: String,
    odometer: f32,
    fuel: f32,
    garage: String,
}

#[derive(Debug, Clone)]
struct FleetTrailerView {
    id: String,
    body: String,
    plate: String,
    cargo_mass: f32,
    wear: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum TruckSort {
    #[default]
    Brand,
    Odometer,
    Fuel,
    Garage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum TrailerSort {
    #[default]
    Body,
    Cargo,
    Wear,
}

// --------------------------------------------------------------- deferred work

#[derive(Debug, Clone)]
enum Operation {
    ReloadAll,
    ReloadTruck,
    ReloadTrailer,
    ReloadFleet,
    RepairTruck,
    RefuelTruck,
    SetFuel(f32),
    SetWear(String, f32),
    SetTruckPlate(String),
    SetOdometer(i64),
    RepairTrailer,
    SetTrailerPlate(String),
    SetCargoMass(f32),
    InitTruckSwitch,
    PreviewTruck(String),
    ApplyTruck,
    ReinitializeTruck,
    InitTrailerSwitch,
    PreviewTrailer(String),
    ApplyTrailer,
    ReinitializeTrailer,
    LoadCatalog,
    PreviewPowertrain,
}

#[derive(Debug, Clone)]
struct Confirm {
    title: &'static str,
    message: String,
    action: Operation,
}

/// All local state. Navigation swaps this instance without clearing it; the
/// shell resets it on context change, Save/Discard/restore.
#[derive(Default)]
pub struct ScreenState {
    profile_path: Option<String>,
    save_path: Option<String>,

    // Player truck
    truck: Option<TruckView>,
    truck_tried: bool,
    truck_error: Option<String>,
    truck_message: Option<String>,
    truck_validation: Option<String>,
    truck_inputs_ready: bool,
    truck_fuel: f32,
    truck_wear: f32,
    wear_component: String,
    truck_plate: String,
    truck_odometer: i64,

    // Player trailer
    trailer: Option<TrailerView>,
    trailer_tried: bool,
    trailer_error: Option<String>,
    trailer_message: Option<String>,
    trailer_validation: Option<String>,
    trailer_inputs_ready: bool,
    trailer_plate: String,
    trailer_mass: f32,

    // Truck switcher
    truck_session: Option<SwitchSessionView>,
    truck_rows: Vec<SwitchRowView>,
    truck_warnings: Vec<String>,
    truck_switch_tried: bool,
    truck_switch_error: Option<String>,
    truck_switch_validation: Option<String>,
    truck_switch_message: Option<String>,
    truck_search: String,
    truck_selected: Option<String>,
    truck_preview: Option<SwitchPreviewView>,
    truck_result: Option<SwitchResultView>,
    truck_backup: Option<bool>,

    // Trailer switcher
    trailer_session: Option<SwitchSessionView>,
    trailer_rows: Vec<SwitchRowView>,
    trailer_warnings: Vec<String>,
    trailer_switch_tried: bool,
    trailer_switch_error: Option<String>,
    trailer_switch_validation: Option<String>,
    trailer_switch_message: Option<String>,
    trailer_search: String,
    trailer_selected: Option<String>,
    trailer_preview: Option<SwitchPreviewView>,
    trailer_result: Option<SwitchResultView>,
    trailer_backup: Option<bool>,

    // Powertrain (catalog and preview are read-only)
    catalog: Option<CatalogView>,
    catalog_tried: bool,
    catalog_error: Option<String>,
    catalog_game: String,
    catalog_validation: Option<String>,
    engine_path: String,
    transmission_path: String,
    powertrain_preview: Option<PowertrainPreviewView>,
    powertrain_tried: bool,
    powertrain_error: Option<String>,

    // Fleet inventory
    fleet_trucks: Vec<FleetTruckView>,
    fleet_trailers: Vec<FleetTrailerView>,
    fleet_tried: bool,
    fleet_error: Option<String>,
    fleet_search: String,
    truck_sort: TruckSort,
    trailer_sort: TrailerSort,
    truck_sort_asc: bool,
    trailer_sort_asc: bool,

    pending: Option<Operation>,
    confirmed: Option<Operation>,
    confirm: Option<Confirm>,
    loading: Option<&'static str>,
}

impl ScreenState {
    fn schedule(&mut self, operation: Operation, loading: &'static str) {
        self.pending = Some(operation);
        self.loading = Some(loading);
    }

    fn confirm(&mut self, title: &'static str, message: String, action: Operation) {
        self.confirm = Some(Confirm {
            title,
            message,
            action,
        });
    }

    /// Catalog game override. The backend defaults to `ets2` when no game is
    /// supplied; keeping the choice local preserves ATS catalog access.
    fn game(&self) -> String {
        let game = self.catalog_game.trim();
        if game.is_empty() {
            "ets2".to_string()
        } else {
            game.to_string()
        }
    }
}

// -------------------------------------------------------------------- feedback

fn toast(ctx: &mut ScreenCtx<'_>, message: String, is_error: bool) {
    ctx.commands.push(ShellCommand::Toast { message, is_error });
}

fn error(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, message, true);
}

fn info(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, message, false);
}

/// Validation is rendered inline *and* routed to the drawer's Validation tab.
fn validation(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, format!("Validation: {message}"), false);
}

fn fail(field: &mut Option<String>, ctx: &mut ScreenCtx<'_>, message: String) {
    *field = Some(message.clone());
    error(ctx, message);
}

fn invalid(field: &mut Option<String>, ctx: &mut ScreenCtx<'_>, message: String) {
    *field = Some(message.clone());
    validation(ctx, message);
}

fn navigate(route: Route, sub_view: SubView, ctx: &mut ScreenCtx<'_>) {
    ctx.commands.push(ShellCommand::Navigate {
        route,
        sub_view: Some(sub_view),
    });
}

/// The shell owns the current tile and sub-view. A foreign or invalid view
/// falls back to the tile default; no local navigation state is kept.
fn normalized_sub_view(route: Route, view: SubView) -> SubView {
    match route {
        Route::vehicles_player_truck => match view {
            SubView::TruckFuelWear | SubView::TruckPlateOdometer => view,
            _ => SubView::TruckFields,
        },
        Route::vehicles_player_trailer => SubView::TrailerFields,
        Route::vehicles_truck_switcher | Route::vehicles_trailer_switcher => match view {
            SubView::SwitchList
            | SubView::SwitchPreview
            | SubView::SwitchApply
            | SubView::SwitchResult => view,
            _ => SubView::SwitchSession,
        },
        Route::vehicles_powertrain => match view {
            SubView::PowertrainPreview => view,
            _ => SubView::PowertrainCatalog,
        },
        Route::vehicles_fleet_inventory => match view {
            SubView::FleetTrailers => view,
            _ => SubView::FleetTrucks,
        },
        _ => SubView::None,
    }
}

/// Sub-view buttons are navigation *requests* only.
fn sub_view_nav(
    ui: &mut egui::Ui,
    ctx: &mut ScreenCtx<'_>,
    route: Route,
    current: SubView,
    items: &[(SubView, &str)],
) {
    ui.horizontal_wrapped(|ui| {
        for (view, label) in items {
            if *view == current {
                ui.label(egui::RichText::new(*label).strong().color(theme::ACCENT));
            } else if comp::button(ui, label).clicked() {
                navigate(route, *view, ctx);
            }
        }
    });
    ui.add_space(5.0);
}

/// Resolve the active profile once, matching the shell's context selection.
fn ensure_profile(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) -> Result<String, String> {
    if let Some(path) = state.profile_path.clone() {
        if !path.is_empty() {
            return Ok(path);
        }
    }
    match crate::gui::data::context::read_last_profile(ctx.data) {
        Ok(Some(path)) if !path.trim().is_empty() => {
            state.profile_path = Some(path.clone());
            Ok(path)
        }
        Ok(_) => Err("No active profile is selected. Choose Game, Profile and Save first."
            .to_string()),
        Err(err) => Err(err.message),
    }
}

/// Resolve the active save once, preferring quicksave as the old shell did.
fn ensure_save(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) -> Result<String, String> {
    if let Some(path) = state.save_path.clone() {
        if !path.is_empty() {
            return Ok(path);
        }
    }
    let profile = ensure_profile(state, ctx)?;
    let saves =
        crate::gui::data::context::find_saves(ctx.data, profile).map_err(|err| err.message)?;
    let preferred = saves
        .iter()
        .find(|save| save.folder.eq_ignore_ascii_case("quicksave"))
        .or_else(|| saves.first())
        .map(|save| save.path.clone())
        .ok_or_else(|| "No save found for the active profile.".to_string())?;
    state.save_path = Some(preferred.clone());
    Ok(preferred)
}

// ------------------------------------------------------------------- rendering

/// Exact screen contract: one persistent state instance and one render fn.
pub fn render(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    if let Some(operation) = state.pending.take() {
        execute(operation, state, ctx);
    }
    if let Some(operation) = state.confirmed.take() {
        execute(operation, state, ctx);
    }

    // Data views load themselves on first display; the shell resets this
    // state after a context change or Save/Discard/restore.
    auto_load(state, ctx);

    ui.add_space(3.0);
    ui.horizontal(|ui| {
        if comp::button(ui, "Open Issues").clicked() {
            ctx.commands
                .push(ShellCommand::OpenIssues(crate::gui::screens::IssuesTab::Errors));
        }
        ui.label(
            egui::RichText::new("Edits stage until Save. Switch apply backups are controlled by the backend.")
                .size(10.5)
                .color(theme::MUTED),
        );
    });

    if let Some(label) = state.loading {
        comp::loading_state(ui, label);
    }

    let view = normalized_sub_view(ctx.route, ctx.sub_view);
    if state.loading.is_none() {
        match ctx.route {
            Route::vehicles_player_truck => render_truck(ui, state, ctx, view),
            Route::vehicles_player_trailer => render_trailer(ui, state, ctx, view),
            Route::vehicles_truck_switcher | Route::vehicles_trailer_switcher => {
                render_switch(ui, state, ctx, view)
            }
            Route::vehicles_powertrain => render_powertrain(ui, state, ctx, view),
            Route::vehicles_fleet_inventory => render_fleet(ui, state, ctx, view),
            _ => render_truck(ui, state, ctx, SubView::TruckFields),
        }
    }

    render_confirm(ui, state, ctx);

    if state.pending.is_none() && state.confirmed.is_none() {
        state.loading = None;
    }
}

fn auto_load(state: &mut ScreenState, ctx: &ScreenCtx<'_>) {
    if state.pending.is_some() || state.confirm.is_some() {
        return;
    }
    if ctx.route == Route::vehicles_player_truck && !state.truck_tried {
        state.schedule(Operation::ReloadTruck, "Loading player truck…");
    } else if ctx.route == Route::vehicles_player_trailer && !state.trailer_tried {
        state.schedule(Operation::ReloadTrailer, "Loading player trailer…");
    } else if ctx.route == Route::vehicles_fleet_inventory && !state.fleet_tried {
        state.schedule(Operation::ReloadFleet, "Loading fleet inventory…");
    } else if ctx.route == Route::vehicles_powertrain && !state.catalog_tried {
        state.schedule(Operation::LoadCatalog, "Loading official powertrain catalog…");
    }
}

// -------------------------------------------------------------------- loaders

fn load_truck(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    state.truck_tried = true;
    let profile = match ensure_profile(state, ctx) {
        Ok(path) => path,
        Err(message) => {
            fail(&mut state.truck_error, ctx, message);
            return;
        }
    };
    match crate::gui::data::vehicles::player_truck(ctx.data, profile) {
        Ok(truck) => {
            state.truck = Some(TruckView {
                id: truck.truck_id.clone(),
                brand: truck.brand.clone(),
                model: truck.model.clone(),
                odometer: truck.odometer,
                fuel: truck.fuel_relative,
                plate: truck
                    .license_plate
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
                wear: vec![
                    ("engine".to_string(), truck.engine_wear),
                    ("transmission".to_string(), truck.transmission_wear),
                    ("cabin".to_string(), truck.cabin_wear),
                    ("chassis".to_string(), truck.chassis_wear),
                ],
            });
            state.truck_error = None;
            if !state.truck_inputs_ready {
                state.truck_plate = truck.license_plate.unwrap_or_default();
                state.truck_odometer = truck.odometer.max(0.0) as i64;
                state.truck_fuel = truck.fuel_relative.clamp(0.0, 1.0);
                state.truck_wear = truck.engine_wear.clamp(0.0, 1.0);
                state.wear_component = "engine".to_string();
                state.truck_inputs_ready = true;
            }
        }
        Err(err) => fail(
            &mut state.truck_error,
            ctx,
            format!("Player truck could not be loaded: {}", err.message),
        ),
    }
}

fn load_trailer(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    state.trailer_tried = true;
    let profile = match ensure_profile(state, ctx) {
        Ok(path) => path,
        Err(message) => {
            fail(&mut state.trailer_error, ctx, message);
            return;
        }
    };
    match crate::gui::data::vehicles::player_trailer(ctx.data, profile) {
        Ok(result) => {
            if !result.success {
                fail(
                    &mut state.trailer_error,
                    ctx,
                    format!("Player trailer lookup failed: {}", result.message),
                );
                return;
            }
            state.trailer_error = None;
            match result.trailer {
                Some(trailer) => {
                    if !state.trailer_inputs_ready {
                        state.trailer_plate = trailer
                            .display_license_plate
                            .clone()
                            .or_else(|| trailer.license_plate.clone())
                            .unwrap_or_default();
                        state.trailer_mass = trailer.cargo_mass;
                        state.trailer_inputs_ready = true;
                    }
                    state.trailer = Some(TrailerView {
                        id: trailer.trailer_id.clone(),
                        plate: trailer
                            .display_license_plate
                            .or_else(|| trailer.license_plate)
                            .unwrap_or_else(|| "-".to_string()),
                        cargo_mass: trailer.cargo_mass,
                        cargo_damage: trailer.cargo_damage,
                        body_wear: trailer.body_wear,
                        chassis_wear: trailer.chassis_wear,
                        message: result.message.clone(),
                    });
                }
                None => {
                    state.trailer = None;
                    state.trailer_message = Some(result.message);
                }
            }
        }
        Err(err) => fail(
            &mut state.trailer_error,
            ctx,
            format!("Player trailer could not be loaded: {}", err.message),
        ),
    }
}

fn load_fleet(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    state.fleet_tried = true;
    let profile = match ensure_profile(state, ctx) {
        Ok(path) => path,
        Err(message) => {
            fail(&mut state.fleet_error, ctx, message);
            return;
        }
    };
    match crate::gui::data::vehicles::all_trucks(ctx.data, profile.clone()) {
        Ok(trucks) => {
            state.fleet_trucks = trucks
                .iter()
                .map(|truck| FleetTruckView {
                    id: truck.truck_id.clone(),
                    brand_model: format!("{} {}", truck.brand, truck.model),
                    plate: truck
                        .license_plate
                        .clone()
                        .unwrap_or_else(|| "-".to_string()),
                    odometer: truck.odometer,
                    fuel: truck.fuel_relative,
                    garage: truck
                        .assigned_garage
                        .clone()
                        .unwrap_or_else(|| "-".to_string()),
                })
                .collect();
            state.fleet_error = None;
        }
        Err(err) => {
            fail(
                &mut state.fleet_error,
                ctx,
                format!("Fleet trucks could not be loaded: {}", err.message),
            );
            return;
        }
    }
    match crate::gui::data::vehicles::all_trailers(ctx.data, profile) {
        Ok(trailers) => {
            state.fleet_trailers = trailers
                .iter()
                .map(|trailer| FleetTrailerView {
                    id: trailer.trailer_id.clone(),
                    body: trailer
                        .body_type
                        .clone()
                        .unwrap_or_else(|| "-".to_string()),
                    plate: trailer
                        .display_license_plate
                        .clone()
                        .or_else(|| trailer.license_plate.clone())
                        .unwrap_or_else(|| "-".to_string()),
                    cargo_mass: trailer.cargo_mass,
                    wear: format!(
                        "{:.1}% / {:.1}%",
                        trailer.body_wear * 100.0,
                        trailer.chassis_wear * 100.0
                    ),
                })
                .collect();
        }
        Err(err) => fail(
            &mut state.fleet_error,
            ctx,
            format!("Fleet trailers could not be loaded: {}", err.message),
        ),
    }
}

// -------------------------------------------------------------- operation runner

fn execute(operation: Operation, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match operation {
        Operation::ReloadAll => {
            state.profile_path = None;
            state.save_path = None;
            load_truck(state, ctx);
            load_trailer(state, ctx);
            load_fleet(state, ctx);
        }
        Operation::ReloadTruck => load_truck(state, ctx),
        Operation::ReloadTrailer => load_trailer(state, ctx),
        Operation::ReloadFleet => load_fleet(state, ctx),

        Operation::RepairTruck => match crate::gui::data::vehicles::repair_truck(ctx.data) {
            Ok(()) => {
                state.truck_message = Some("Player truck repaired to 0% wear.".to_string());
                state.truck_validation = None;
                info(ctx, "Player truck repaired.".to_string());
                state.schedule(Operation::ReloadTruck, "Refreshing player truck…");
            }
            Err(err) => fail(
                &mut state.truck_error,
                ctx,
                format!("Truck repair failed: {}", err.message),
            ),
        },
        Operation::RefuelTruck => match crate::gui::data::vehicles::refuel_truck(ctx.data) {
            Ok(()) => {
                state.truck_message = Some("Player truck refueled to 100%.".to_string());
                state.truck_validation = None;
                info(ctx, "Player truck refueled.".to_string());
                state.schedule(Operation::ReloadTruck, "Refreshing player truck…");
            }
            Err(err) => fail(
                &mut state.truck_error,
                ctx,
                format!("Truck refuel failed: {}", err.message),
            ),
        },
        Operation::SetFuel(level) => {
            if !(0.0..=1.0).contains(&level) {
                invalid(
                    &mut state.truck_validation,
                    ctx,
                    "Fuel level must be between 0 and 1.".to_string(),
                );
                return;
            }
            match crate::gui::data::vehicles::set_fuel(ctx.data, level) {
                Ok(()) => {
                    state.truck_message = Some(format!("Fuel set to {:.0}%.", level * 100.0));
                    state.truck_validation = None;
                    info(ctx, "Truck fuel level updated.".to_string());
                    state.schedule(Operation::ReloadTruck, "Refreshing player truck…");
                }
                Err(err) => fail(
                    &mut state.truck_error,
                    ctx,
                    format!("Fuel update failed: {}", err.message),
                ),
            }
        }
        Operation::SetWear(component, level) => {
            if !(0.0..=1.0).contains(&level) {
                invalid(
                    &mut state.truck_validation,
                    ctx,
                    "Wear must be between 0 and 1.".to_string(),
                );
                return;
            }
            match crate::gui::data::vehicles::set_wear(ctx.data, component.clone(), level) {
                Ok(()) => {
                    state.truck_message =
                        Some(format!("Set {component} wear to {:.0}%.", level * 100.0));
                    state.truck_validation = None;
                    info(ctx, format!("Truck {component} wear updated."));
                    state.schedule(Operation::ReloadTruck, "Refreshing player truck…");
                }
                Err(err) => fail(
                    &mut state.truck_error,
                    ctx,
                    format!("Failed to set {component} wear: {}", err.message),
                ),
            }
        }
        Operation::SetTruckPlate(plate) => {
            let plate = plate.trim().to_string();
            if plate.is_empty() {
                invalid(
                    &mut state.truck_validation,
                    ctx,
                    "Truck license plate must not be empty.".to_string(),
                );
                return;
            }
            match crate::gui::data::vehicles::set_plate(ctx.data, plate) {
                Ok(()) => {
                    state.truck_message = Some("Truck license plate updated.".to_string());
                    state.truck_validation = None;
                    info(ctx, "Truck license plate updated.".to_string());
                    state.schedule(Operation::ReloadTruck, "Refreshing player truck…");
                }
                Err(err) => fail(
                    &mut state.truck_error,
                    ctx,
                    format!("Plate update failed: {}", err.message),
                ),
            }
        }
        Operation::SetOdometer(value) => {
            if !(0..=99_999_999).contains(&value) {
                invalid(
                    &mut state.truck_validation,
                    ctx,
                    "Odometer must be between 0 and 99,999,999 km.".to_string(),
                );
                return;
            }
            match crate::gui::data::vehicles::set_odometer(ctx.data, value) {
                Ok(()) => {
                    state.truck_message = Some(format!("Odometer set to {value} km."));
                    state.truck_validation = None;
                    info(ctx, "Truck odometer updated.".to_string());
                    state.schedule(Operation::ReloadTruck, "Refreshing player truck…");
                }
                Err(err) => fail(
                    &mut state.truck_error,
                    ctx,
                    format!("Odometer update failed: {}", err.message),
                ),
            }
        }
        Operation::RepairTrailer => match crate::gui::data::vehicles::repair_trailer(ctx.data) {
            Ok(()) => {
                state.trailer_message = Some("Player trailer repaired.".to_string());
                state.trailer_validation = None;
                info(ctx, "Player trailer repaired.".to_string());
                state.schedule(Operation::ReloadTrailer, "Refreshing player trailer…");
            }
            Err(err) => fail(
                &mut state.trailer_error,
                ctx,
                format!("Trailer repair failed: {}", err.message),
            ),
        },
        Operation::SetTrailerPlate(plate) => {
            let plate = plate.trim().to_string();
            if plate.is_empty() {
                invalid(
                    &mut state.trailer_validation,
                    ctx,
                    "Trailer license plate must not be empty.".to_string(),
                );
                return;
            }
            match crate::gui::data::vehicles::set_trailer_plate(ctx.data, plate) {
                Ok(()) => {
                    state.trailer_message = Some("Trailer license plate updated.".to_string());
                    state.trailer_validation = None;
                    info(ctx, "Trailer license plate updated.".to_string());
                    state.schedule(Operation::ReloadTrailer, "Refreshing player trailer…");
                }
                Err(err) => fail(
                    &mut state.trailer_error,
                    ctx,
                    format!("Trailer plate update failed: {}", err.message),
                ),
            }
        }
        Operation::SetCargoMass(mass) => {
            if !(0.0..=200_000.0).contains(&mass) {
                invalid(
                    &mut state.trailer_validation,
                    ctx,
                    "Cargo mass must be between 0 and 200,000 kg.".to_string(),
                );
                return;
            }
            match crate::gui::data::vehicles::set_cargo_mass(ctx.data, mass) {
                Ok(()) => {
                    state.trailer_message = Some(format!("Cargo mass set to {mass:.1} kg."));
                    state.trailer_validation = None;
                    info(ctx, "Trailer cargo mass updated.".to_string());
                    state.schedule(Operation::ReloadTrailer, "Refreshing player trailer…");
                }
                Err(err) => fail(
                    &mut state.trailer_error,
                    ctx,
                    format!("Cargo mass update failed: {}", err.message),
                ),
            }
        }

        other => execute_advanced(other, state, ctx),
    }
}

fn execute_advanced(operation: Operation, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match operation {
        Operation::InitTruckSwitch => init_truck_switch(state, ctx),
        Operation::PreviewTruck(id) => preview_truck(state, ctx, id),
        Operation::ApplyTruck => apply_truck(state, ctx),
        Operation::ReinitializeTruck => {
            state.truck_session = None;
            state.truck_rows.clear();
            state.truck_preview = None;
            state.truck_result = None;
            state.truck_selected = None;
            state.truck_warnings.clear();
            state.truck_switch_validation = None;
            state.schedule(Operation::InitTruckSwitch, "Reinitializing truck switch session…");
        }
        Operation::InitTrailerSwitch => init_trailer_switch(state, ctx),
        Operation::PreviewTrailer(id) => preview_trailer(state, ctx, id),
        Operation::ApplyTrailer => apply_trailer(state, ctx),
        Operation::ReinitializeTrailer => {
            state.trailer_session = None;
            state.trailer_rows.clear();
            state.trailer_preview = None;
            state.trailer_result = None;
            state.trailer_selected = None;
            state.trailer_warnings.clear();
            state.trailer_switch_validation = None;
            state.schedule(
                Operation::InitTrailerSwitch,
                "Reinitializing trailer switch session…",
            );
        }
        Operation::LoadCatalog => load_catalog(state, ctx),
        Operation::PreviewPowertrain => preview_powertrain(state, ctx),
        _ => {}
    }
}

// --------------------------------------------------------------- switchers

fn option_text(value: &Option<String>) -> String {
    value.clone().unwrap_or_else(|| "-".to_string())
}

fn optional_path(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn init_truck_switch(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    state.truck_switch_tried = true;
    let save_path = match ensure_save(state, ctx) {
        Ok(path) => path,
        Err(message) => {
            fail(&mut state.truck_switch_error, ctx, message);
            return;
        }
    };
    match crate::gui::data::vehicles::init_truck_switch(ctx.data, Some(save_path.clone())) {
        Ok(session) => {
            state.truck_session = Some(SwitchSessionView {
                save_hash: session.save_hash.clone(),
                current_label: format!(
                    "{} {} ({})",
                    option_text(&session.current_truck.brand),
                    option_text(&session.current_truck.model),
                    session.current_truck.truck_id
                ),
                warnings: session.warnings.clone(),
            });
            state.truck_warnings = session.warnings.clone();
            state.truck_preview = None;
            state.truck_result = None;
            state.truck_selected = None;
            state.truck_switch_validation = None;
            state.truck_switch_error = None;
            match crate::gui::data::vehicles::list_trucks_for_switch(ctx.data, Some(save_path)) {
                Ok(list) => {
                    state.truck_rows = list
                        .trucks
                        .iter()
                        .map(|item| SwitchRowView {
                            id: item.truck_id.clone(),
                            label: format!(
                                "{} {}",
                                option_text(&item.brand),
                                option_text(&item.model)
                            ),
                            sub: item.truck_id.clone(),
                            plate: option_text(&item.display_license_plate),
                            garage: option_text(&item.garage_display_name),
                            status: if item.is_active {
                                "Active".to_string()
                            } else if item.is_switchable {
                                "Switchable".to_string()
                            } else {
                                option_text(&item.blocked_reason)
                            },
                            active: item.is_active,
                            switchable: item.is_switchable,
                        })
                        .collect();
                    state.truck_warnings.extend(list.warnings);
                }
                Err(err) => fail(
                    &mut state.truck_switch_error,
                    ctx,
                    format!("Truck switch list failed: {}", err.message),
                ),
            }
            info(ctx, "Truck switch session initialized.".to_string());
        }
        Err(err) => fail(
            &mut state.truck_switch_error,
            ctx,
            format!("Truck session error: {}", err.message),
        ),
    }
}

fn preview_truck(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>, id: String) {
    let Some(session) = state.truck_session.clone() else {
        invalid(
            &mut state.truck_switch_validation,
            ctx,
            "Initialize a truck switch session first.".to_string(),
        );
        return;
    };
    let save_path = match ensure_save(state, ctx) {
        Ok(path) => path,
        Err(message) => {
            fail(&mut state.truck_switch_error, ctx, message);
            return;
        }
    };
    match crate::gui::data::vehicles::preview_truck_switch(
        ctx.data,
        Some(save_path),
        id,
        session.save_hash.clone(),
    ) {
        Ok(preview) => {
            state.truck_preview = Some(SwitchPreviewView {
                target_id: preview.target_truck.truck_id.clone(),
                target_label: format!(
                    "{} {} ({})",
                    option_text(&preview.target_truck.brand),
                    option_text(&preview.target_truck.model),
                    preview.target_truck.truck_id
                ),
                mode: format!("{:?}", preview.mode),
                warnings: preview.warnings.clone(),
                error_code: preview.error_code.clone(),
                expected_hash: preview.expected_file_hash.clone(),
                safe_to_write: preview.safe_to_write,
                can_apply: preview.can_apply,
                details: vec![
                    (
                        "Current truck".to_string(),
                        preview.current_truck.truck_id.clone(),
                    ),
                    (
                        "Affected driver".to_string(),
                        preview
                            .affected_driver
                            .as_ref()
                            .and_then(|driver| driver.display_name.clone())
                            .unwrap_or_else(|| "-".to_string()),
                    ),
                    (
                        "Write case".to_string(),
                        preview
                            .swap_plan
                            .as_ref()
                            .and_then(|plan| plan.write_case.clone())
                            .unwrap_or_else(|| "-".to_string()),
                    ),
                ],
            });
            state.truck_switch_validation = None;
        }
        Err(err) => fail(
            &mut state.truck_switch_error,
            ctx,
            format!("Truck switch preview failed: {}", err.message),
        ),
    }
}

fn apply_truck(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let (Some(session), Some(preview)) =
        (state.truck_session.clone(), state.truck_preview.clone())
    else {
        invalid(
            &mut state.truck_switch_validation,
            ctx,
            "Preview a switchable truck before applying.".to_string(),
        );
        return;
    };
    if !preview.can_apply || !preview.safe_to_write {
        invalid(
            &mut state.truck_switch_validation,
            ctx,
            "Backend reports this switch is not safe to write.".to_string(),
        );
        return;
    }
    if preview.expected_hash != session.save_hash {
        invalid(
            &mut state.truck_switch_validation,
            ctx,
            "Save hash changed since preview. Reinitialize the session.".to_string(),
        );
        return;
    }
    let save_path = match ensure_save(state, ctx) {
        Ok(path) => path,
        Err(message) => {
            fail(&mut state.truck_switch_error, ctx, message);
            return;
        }
    };
    let backup = state.truck_backup.unwrap_or(true);
    match crate::gui::data::vehicles::apply_truck_switch(
        ctx.data,
        Some(save_path),
        preview.target_id.clone(),
        preview.expected_hash.clone(),
        Some(backup),
    ) {
        Ok(result) => {
            state.truck_result = Some(SwitchResultView {
                success: result.success,
                backup_id: result.backup_id.clone(),
                persistent_backup: result.persistent_backup_created,
                previous_id: result.previous_truck_id.clone(),
                active_id: result.active_truck_id.clone(),
                validation_errors: result.validation.errors.clone(),
                hash_before: result.file_hash_before.clone(),
                hash_after: result.file_hash_after.clone(),
            });
            state.truck_session = Some(SwitchSessionView {
                save_hash: result.refreshed_session.save_hash.clone(),
                current_label: format!(
                    "{} {} ({})",
                    option_text(&result.refreshed_session.current_truck.brand),
                    option_text(&result.refreshed_session.current_truck.model),
                    result.refreshed_session.current_truck.truck_id
                ),
                warnings: result.refreshed_session.warnings.clone(),
            });
            state.truck_preview = None;
            state.truck_selected = None;
            state.truck_switch_validation = None;
            state.truck_switch_error = None;
            state.truck_switch_message = Some(format!(
                "Active truck changed from {} to {}.",
                result.previous_truck_id, result.active_truck_id
            ));
            info(
                ctx,
                format!("Truck switched to {}.", result.active_truck_id),
            );
            state.schedule(Operation::ReloadAll, "Refreshing vehicles after switch…");
        }
        Err(err) => fail(
            &mut state.truck_switch_error,
            ctx,
            format!("Truck switch failed: {}", err.message),
        ),
    }
}

fn init_trailer_switch(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    state.trailer_switch_tried = true;
    let save_path = match ensure_save(state, ctx) {
        Ok(path) => path,
        Err(message) => {
            fail(&mut state.trailer_switch_error, ctx, message);
            return;
        }
    };
    match crate::gui::data::vehicles::init_trailer_switch(ctx.data, Some(save_path.clone())) {
        Ok(session) => {
            state.trailer_session = Some(SwitchSessionView {
                save_hash: session.save_hash.clone(),
                current_label: format!(
                    "{} ({})",
                    session.current_trailer.display_name,
                    session.current_trailer.trailer_id
                ),
                warnings: session.warnings.clone(),
            });
            state.trailer_warnings = session.warnings.clone();
            state.trailer_preview = None;
            state.trailer_result = None;
            state.trailer_selected = None;
            state.trailer_switch_validation = None;
            state.trailer_switch_error = None;
            match crate::gui::data::vehicles::list_trailers_for_switch(ctx.data, Some(save_path)) {
                Ok(list) => {
                    state.trailer_rows = list
                        .trailers
                        .iter()
                        .map(|item| SwitchRowView {
                            id: item.trailer_id.clone(),
                            label: item.display_name.clone(),
                            sub: item.trailer_id.clone(),
                            plate: option_text(&item.display_license_plate),
                            garage: option_text(&item.garage_display_name),
                            status: if item.is_active {
                                "Active".to_string()
                            } else if item.is_switchable {
                                "Switchable".to_string()
                            } else {
                                option_text(&item.availability_reason)
                            },
                            active: item.is_active,
                            switchable: item.is_switchable,
                        })
                        .collect();
                    state.trailer_warnings.extend(list.warnings);
                }
                Err(err) => fail(
                    &mut state.trailer_switch_error,
                    ctx,
                    format!("Trailer switch list failed: {}", err.message),
                ),
            }
            info(ctx, "Trailer switch session initialized.".to_string());
        }
        Err(err) => fail(
            &mut state.trailer_switch_error,
            ctx,
            format!("Trailer session error: {}", err.message),
        ),
    }
}

fn preview_trailer(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>, id: String) {
    let Some(session) = state.trailer_session.clone() else {
        invalid(
            &mut state.trailer_switch_validation,
            ctx,
            "Initialize a trailer switch session first.".to_string(),
        );
        return;
    };
    let save_path = match ensure_save(state, ctx) {
        Ok(path) => path,
        Err(message) => {
            fail(&mut state.trailer_switch_error, ctx, message);
            return;
        }
    };
    match crate::gui::data::vehicles::preview_trailer_switch(
        ctx.data,
        Some(save_path),
        id,
        session.save_hash.clone(),
    ) {
        Ok(preview) => {
            state.trailer_preview = Some(SwitchPreviewView {
                target_id: preview.target_trailer.trailer_id.clone(),
                target_label: format!(
                    "{} ({})",
                    preview.target_trailer.display_name,
                    preview.target_trailer.trailer_id
                ),
                mode: format!("{:?}", preview.mode),
                warnings: preview.warnings.clone(),
                error_code: preview.error_code.clone(),
                expected_hash: preview.expected_file_hash.clone(),
                safe_to_write: preview.safe_to_write,
                can_apply: preview.can_apply,
                details: vec![
                    (
                        "Current trailer".to_string(),
                        preview.current_trailer.trailer_id.clone(),
                    ),
                    (
                        "Write case".to_string(),
                        preview
                            .swap_plan
                            .as_ref()
                            .and_then(|plan| plan.write_case.clone())
                            .unwrap_or_else(|| "-".to_string()),
                    ),
                ],
            });
            state.trailer_switch_validation = None;
        }
        Err(err) => fail(
            &mut state.trailer_switch_error,
            ctx,
            format!("Trailer switch preview failed: {}", err.message),
        ),
    }
}

fn apply_trailer(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let (Some(session), Some(preview)) =
        (state.trailer_session.clone(), state.trailer_preview.clone())
    else {
        invalid(
            &mut state.trailer_switch_validation,
            ctx,
            "Preview a switchable trailer before applying.".to_string(),
        );
        return;
    };
    if !preview.can_apply || !preview.safe_to_write {
        invalid(
            &mut state.trailer_switch_validation,
            ctx,
            "Backend reports this switch is not safe to write.".to_string(),
        );
        return;
    }
    if preview.expected_hash != session.save_hash {
        invalid(
            &mut state.trailer_switch_validation,
            ctx,
            "Save hash changed since preview. Reinitialize the session.".to_string(),
        );
        return;
    }
    let save_path = match ensure_save(state, ctx) {
        Ok(path) => path,
        Err(message) => {
            fail(&mut state.trailer_switch_error, ctx, message);
            return;
        }
    };
    let backup = state.trailer_backup.unwrap_or(true);
    match crate::gui::data::vehicles::apply_trailer_switch(
        ctx.data,
        Some(save_path),
        preview.target_id.clone(),
        preview.expected_hash.clone(),
        Some(backup),
    ) {
        Ok(result) => {
            state.trailer_result = Some(SwitchResultView {
                success: result.success,
                backup_id: result.backup_id.clone(),
                persistent_backup: result.persistent_backup_created,
                previous_id: result.previous_trailer_id.clone(),
                active_id: result.active_trailer_id.clone(),
                validation_errors: result.validation.errors.clone(),
                hash_before: result.file_hash_before.clone(),
                hash_after: result.file_hash_after.clone(),
            });
            state.trailer_session = Some(SwitchSessionView {
                save_hash: result.refreshed_session.save_hash.clone(),
                current_label: format!(
                    "{} ({})",
                    result.refreshed_session.current_trailer.display_name,
                    result.refreshed_session.current_trailer.trailer_id
                ),
                warnings: result.refreshed_session.warnings.clone(),
            });
            state.trailer_preview = None;
            state.trailer_selected = None;
            state.trailer_switch_validation = None;
            state.trailer_switch_error = None;
            state.trailer_switch_message = Some(format!(
                "Active trailer changed from {} to {}.",
                result.previous_trailer_id, result.active_trailer_id
            ));
            info(
                ctx,
                format!("Trailer switched to {}.", result.active_trailer_id),
            );
            state.schedule(Operation::ReloadAll, "Refreshing vehicles after switch…");
        }
        Err(err) => fail(
            &mut state.trailer_switch_error,
            ctx,
            format!("Trailer switch failed: {}", err.message),
        ),
    }
}

// ------------------------------------------------------------------ powertrain

fn load_catalog(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    state.catalog_tried = true;
    let game = state.game();
    match crate::gui::data::vehicles::powertrain_catalog(ctx.data, Some(game.clone()), None) {
        Ok(catalog) => {
            state.catalog = Some(CatalogView {
                game: catalog.game.clone(),
                version: catalog.game_version.clone(),
                engines: catalog
                    .engines
                    .iter()
                    .map(|engine| CatalogEntryView {
                        label: format!(
                            "[{} {}] {} ({:.0} HP / {:.0} Nm)",
                            engine.brand,
                            engine.truck_model,
                            engine.name,
                            engine.power.unwrap_or(0.0),
                            engine.torque_nm.unwrap_or(0.0)
                        ),
                        data_path: engine.data_path.clone(),
                    })
                    .collect(),
                transmissions: catalog
                    .transmissions
                    .iter()
                    .map(|transmission| CatalogEntryView {
                        label: format!(
                            "[{} {}] {} ({} gears, diff {:.2})",
                            transmission.brand,
                            transmission.truck_model,
                            transmission.name,
                            transmission.gears_forward.unwrap_or(0),
                            transmission.differential_ratio.unwrap_or(0.0)
                        ),
                        data_path: transmission.data_path.clone(),
                    })
                    .collect(),
            });
            if state.engine_path.is_empty() {
                state.engine_path = catalog
                    .engines
                    .first()
                    .map(|engine| engine.data_path.clone())
                    .unwrap_or_default();
            }
            if state.transmission_path.is_empty() {
                state.transmission_path = catalog
                    .transmissions
                    .first()
                    .map(|transmission| transmission.data_path.clone())
                    .unwrap_or_default();
            }
            state.catalog_error = None;
            state.powertrain_preview = None;
        }
        Err(err) => fail(
            &mut state.catalog_error,
            ctx,
            format!("Catalog load error: {}", err.message),
        ),
    }
}

fn preview_powertrain(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    if state.engine_path.trim().is_empty() && state.transmission_path.trim().is_empty() {
        invalid(
            &mut state.catalog_validation,
            ctx,
            "Select an engine or a transmission before previewing.".to_string(),
        );
        return;
    }
    let truck_id = match state.truck.clone() {
        Some(truck) => truck.id.clone(),
        None => {
            let profile = match ensure_profile(state, ctx) {
                Ok(path) => path,
                Err(message) => {
                    fail(&mut state.catalog_error, ctx, message);
                    return;
                }
            };
            match crate::gui::data::vehicles::player_truck(ctx.data, profile) {
                Ok(truck) => {
                    let id = truck.truck_id.clone();
                    state.truck = Some(TruckView {
                        id: truck.truck_id.clone(),
                        brand: truck.brand.clone(),
                        model: truck.model.clone(),
                        odometer: truck.odometer,
                        fuel: truck.fuel_relative,
                        plate: truck
                            .license_plate
                            .clone()
                            .unwrap_or_else(|| "-".to_string()),
                        wear: vec![
                            ("engine".to_string(), truck.engine_wear),
                            ("transmission".to_string(), truck.transmission_wear),
                            ("cabin".to_string(), truck.cabin_wear),
                            ("chassis".to_string(), truck.chassis_wear),
                        ],
                    });
                    id
                }
                Err(err) => {
                    fail(
                        &mut state.powertrain_error,
                        ctx,
                        format!("Player truck is required for powertrain preview: {}", err.message),
                    );
                    return;
                }
            }
        }
    };
    let save_path = match ensure_save(state, ctx) {
        Ok(path) => path,
        Err(message) => {
            fail(&mut state.powertrain_error, ctx, message);
            return;
        }
    };
    let game = state.game();
    let engine = optional_path(&state.engine_path);
    let transmission = optional_path(&state.transmission_path);
    state.powertrain_tried = true;
    match crate::gui::data::vehicles::preview_powertrain(
        ctx.data,
        Some(save_path),
        truck_id,
        engine,
        transmission,
        Some(game),
        None,
    ) {
        Ok(preview) => {
            state.powertrain_error = None;
            state.catalog_validation = None;
            state.powertrain_preview = Some(PowertrainPreviewView {
                truck_id: preview.truck_id.clone(),
                family: preview.truck_family.clone(),
                cross_brand: preview.experimental_cross_brand,
                current_engine: preview.current_engine.selected_name.clone(),
                new_engine: preview
                    .new_engine
                    .as_ref()
                    .and_then(|component| component.selected_name.clone()),
                current_transmission: preview.current_transmission.selected_name.clone(),
                new_transmission: preview
                    .new_transmission
                    .as_ref()
                    .and_then(|component| component.selected_name.clone()),
                differential: preview.selected_differential_ratio,
                can_apply_later: preview.can_apply_later,
                warnings: preview.warnings.clone(),
            });
        }
        Err(err) => fail(
            &mut state.powertrain_error,
            ctx,
            format!("Powertrain preview failed: {}", err.message),
        ),
    }
}

// ------------------------------------------------------------------ detail views

fn render_truck(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>, view: SubView) {
    sub_view_nav(
        ui,
        ctx,
        Route::vehicles_player_truck,
        view,
        &[
            (SubView::TruckFields, "Fields"),
            (SubView::TruckFuelWear, "Fuel & wear"),
            (SubView::TruckPlateOdometer, "Plate & odometer"),
        ],
    );
    if let Some(err) = state.truck_error.clone() {
        if comp::error_state(
            ui,
            &format!("Player truck data is unavailable: {err}"),
            Some("Retry"),
        ) {
            state.schedule(Operation::ReloadTruck, "Loading player truck…");
        }
        return;
    }
    let Some(truck) = state.truck.clone() else {
        comp::empty_state(
            ui,
            "No player truck found",
            "Select a save in the context column, then retry the load.",
        );
        if comp::button(ui, "Retry").clicked() {
            state.schedule(Operation::ReloadTruck, "Loading player truck…");
        }
        return;
    };

    comp::section_card(ui, "Player truck", |ui| {
        ui.label(egui::RichText::new(format!("{} {}", truck.brand, truck.model)).strong());
        ui.label(format!("ID: {}", truck.id));
        ui.label(format!(
            "Odometer: {:.0} km · Fuel: {:.0}% · Plate: {}",
            truck.odometer,
            truck.fuel * 100.0,
            truck.plate
        ));
        ui.label(
            truck
                .wear
                .iter()
                .map(|(name, value)| format!("{name}: {:.0}%", value * 100.0))
                .collect::<Vec<_>>()
                .join(" · "),
        );
    });

    comp::section_card(ui, "Immediate maintenance", |ui| {
        if comp::button(ui, "Repair truck (0% wear)").clicked() {
            state.confirm(
                "Repair truck",
                "Set every repairable component of the active truck to 0% wear? The edit stages until Save.".to_string(),
                Operation::RepairTruck,
            );
        }
        if comp::button(ui, "Refuel truck (100%)").clicked() {
            state.confirm(
                "Refuel truck",
                "Fill the active truck to 100% fuel? The edit stages until Save.".to_string(),
                Operation::RefuelTruck,
            );
        }
    });

    if view != SubView::TruckPlateOdometer {
        comp::section_card(ui, "Fuel & component wear", |ui| {
            ui.horizontal(|ui| {
                ui.label("Fuel level");
                ui.add(egui::Slider::new(&mut state.truck_fuel, 0.0..=1.0));
            });
            if comp::button(ui, "Apply fuel level").clicked() {
                let level = state.truck_fuel;
                state.confirm(
                    "Set fuel level",
                    format!("Set truck fuel to {:.0}%? The edit stages until Save.", level * 100.0),
                    Operation::SetFuel(level),
                );
            }

            let wear_options = vec![
                "engine".to_string(),
                "transmission".to_string(),
                "cabin".to_string(),
                "chassis".to_string(),
                "wheels".to_string(),
            ];
            let mut component = state.wear_component.clone();
            comp::form_field(ui, "Wear component", |ui| {
                comp::select(ui, "fleet_wear_component", &mut component, &wear_options);
            });
            state.wear_component = component.clone();
            ui.horizontal(|ui| {
                ui.label("Wear level");
                ui.add(egui::Slider::new(&mut state.truck_wear, 0.0..=1.0));
            });
            if comp::button(ui, "Apply component wear").clicked() {
                let level = state.truck_wear;
                state.confirm(
                    "Set component wear",
                    format!("Set {component} wear to {:.0}%? The edit stages until Save.", level * 100.0),
                    Operation::SetWear(component.clone(), level),
                );
            }
        });
    }

    if view != SubView::TruckFuelWear {
        comp::section_card(ui, "Plate & odometer", |ui| {
            comp::form_field(ui, "License plate", |ui| {
                comp::text_input(ui, &mut state.truck_plate, "Truck license plate");
            });
            if comp::button(ui, "Apply plate").clicked() {
                state.confirm(
                    "Set truck plate",
                    "Change the active truck's license plate? The edit stages until Save.".to_string(),
                    Operation::SetTruckPlate(state.truck_plate.clone()),
                );
            }
            comp::form_field(ui, "Odometer (km)", |ui| {
                ui.add(
                    egui::DragValue::new(&mut state.truck_odometer).range(0..=99_999_999),
                );
            });
            if comp::button(ui, "Apply odometer").clicked() {
                state.confirm(
                    "Set odometer",
                    format!(
                        "Set the truck odometer to {} km? The edit stages until Save.",
                        state.truck_odometer
                    ),
                    Operation::SetOdometer(state.truck_odometer),
                );
            }
        });
    }

    comp::validation_message(ui, state.truck_validation.as_deref().unwrap_or(""));
    if let Some(message) = &state.truck_message {
        ui.label(egui::RichText::new(message.as_str()).color(theme::SUCCESS));
    }
}

fn render_trailer(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
    view: SubView,
) {
    sub_view_nav(
        ui,
        ctx,
        Route::vehicles_player_trailer,
        view,
        &[(SubView::TrailerFields, "Fields")],
    );
    if let Some(err) = state.trailer_error.clone() {
        if comp::error_state(
            ui,
            &format!("Player trailer data is unavailable: {err}"),
            Some("Retry"),
        ) {
            state.schedule(Operation::ReloadTrailer, "Loading player trailer…");
        }
        return;
    }
    let Some(trailer) = state.trailer.clone() else {
        comp::empty_state(
            ui,
            "No player trailer found",
            state
                .trailer_message
                .as_deref()
                .unwrap_or("Select a save in the context column, then retry the load."),
        );
        if comp::button(ui, "Retry").clicked() {
            state.schedule(Operation::ReloadTrailer, "Loading player trailer…");
        }
        return;
    };

    comp::section_card(ui, "Player trailer", |ui| {
        ui.label(egui::RichText::new(format!("Trailer {}", trailer.id)).strong());
        ui.label(format!("Plate: {}", trailer.plate));
        ui.label(format!(
            "Cargo: {:.1} kg · damage: {:.0}% · body wear: {:.0}% · chassis wear: {:.0}%",
            trailer.cargo_mass,
            trailer.cargo_damage * 100.0,
            trailer.body_wear * 100.0,
            trailer.chassis_wear * 100.0
        ));
        ui.label(&trailer.message);
    });

    comp::section_card(ui, "Trailer maintenance & cargo", |ui| {
        if comp::button(ui, "Repair trailer (0% wear and damage)").clicked() {
            state.confirm(
                "Repair trailer",
                "Repair the active trailer to 0% wear and damage? The edit stages until Save."
                    .to_string(),
                Operation::RepairTrailer,
            );
        }
        comp::form_field(ui, "License plate", |ui| {
            comp::text_input(ui, &mut state.trailer_plate, "Trailer license plate");
        });
        if comp::button(ui, "Apply plate").clicked() {
            state.confirm(
                "Set trailer plate",
                "Change the active trailer's license plate? The edit stages until Save."
                    .to_string(),
                Operation::SetTrailerPlate(state.trailer_plate.clone()),
            );
        }
        comp::form_field(ui, "Cargo mass (kg)", |ui| {
            ui.add(
                egui::DragValue::new(&mut state.trailer_mass)
                    .speed(100.0)
                    .range(0.0..=200_000.0),
            );
        });
        if comp::button(ui, "Apply cargo mass").clicked() {
            state.confirm(
                "Set cargo mass",
                format!(
                    "Set trailer cargo mass to {:.1} kg? The edit stages until Save.",
                    state.trailer_mass
                ),
                Operation::SetCargoMass(state.trailer_mass),
            );
        }
    });

    comp::validation_message(ui, state.trailer_validation.as_deref().unwrap_or(""));
    if let Some(message) = &state.trailer_message {
        ui.label(egui::RichText::new(message.as_str()).color(theme::SUCCESS));
    }
}

// ---------------------------------------------------------- switcher rendering

fn switch_table(
    ui: &mut egui::Ui,
    id: &str,
    rows: &[SwitchRowView],
    search: &str,
    selected: &Option<String>,
) -> Option<SwitchRowView> {
    let filter = search.trim().to_ascii_lowercase();
    let filtered: Vec<&SwitchRowView> = rows
        .iter()
        .filter(|row| {
            filter.is_empty()
                || row.label.to_ascii_lowercase().contains(&filter)
                || row.sub.to_ascii_lowercase().contains(&filter)
                || row.plate.to_ascii_lowercase().contains(&filter)
                || row.garage.to_ascii_lowercase().contains(&filter)
        })
        .collect();
    if filtered.is_empty() {
        comp::empty_state(
            ui,
            "No vehicles match this search",
            "Clear the search or reinitialize the switch session.",
        );
        return None;
    }
    let table_rows: Vec<Vec<String>> = filtered
        .iter()
        .map(|row| {
            vec![
                row.label.clone(),
                row.sub.clone(),
                row.plate.clone(),
                row.garage.clone(),
                row.status.clone(),
            ]
        })
        .collect();
    let selected_index = selected
        .as_ref()
        .and_then(|id| filtered.iter().position(|row| &row.id == id));
    comp::table(
        ui,
        id,
        &["Vehicle", "ID", "Plate", "Garage", "Status"],
        &table_rows,
        selected_index,
    )
    .and_then(|index| filtered.get(index).map(|row| (*row).clone()))
}

/// Returns true when the user requested a confirmed apply.
fn switch_preview_card(
    ui: &mut egui::Ui,
    preview: &SwitchPreviewView,
    result: Option<&SwitchResultView>,
    backup: &mut Option<bool>,
    current_hash: &str,
) -> bool {
    comp::section_card(ui, "Switch preview", |ui| {
        ui.label(egui::RichText::new(preview.target_label.as_str()).strong());
        ui.label(format!(
            "Mode: {} · Safe to write: {} · Can apply: {}",
            preview.mode, preview.safe_to_write, preview.can_apply
        ));
        ui.label(format!("Expected save hash: {}", preview.expected_hash));
        for (label, value) in &preview.details {
            ui.label(format!("{label}: {value}"));
        }
        if let Some(code) = &preview.error_code {
            ui.label(egui::RichText::new(format!("Error code: {code}")).color(theme::DANGER));
        }
        for warning in &preview.warnings {
            ui.label(egui::RichText::new(format!("Warning: {warning}")).color(theme::WARNING));
        }
        for validation in result.into_iter().flat_map(|item| item.validation_errors.iter()) {
            ui.label(egui::RichText::new(format!("Validation: {validation}")).color(theme::WARNING));
        }

        let hash_ok = preview.expected_hash == current_hash;
        if !hash_ok {
            comp::validation_message(
                ui,
                "Save hash changed since preview — reinitialize the session.",
            );
        }
        let mut enabled = backup.unwrap_or(true);
        comp::toggle(
            ui,
            "create_persistent_backup",
            "Create persistent backup before switch",
            &mut enabled,
        );
        *backup = Some(enabled);
        let can_apply = preview.can_apply && preview.safe_to_write && hash_ok;
        let apply = if can_apply {
            comp::primary_button(ui, "Apply active switch").clicked()
        } else {
            ui.add_enabled(false, egui::Button::new("Apply blocked by preview"))
                .clicked()
        };
        apply
    })
    .inner
}

fn switch_result_card(ui: &mut egui::Ui, result: &SwitchResultView) {
    comp::section_card(ui, "Switch result", |ui| {
        ui.label(if result.success {
            egui::RichText::new("Switch applied successfully.").color(theme::SUCCESS)
        } else {
            egui::RichText::new("Backend reported an unsuccessful switch.").color(theme::DANGER)
        });
        ui.label(format!(
            "{} → {}",
            result.previous_id, result.active_id
        ));
        ui.label(format!(
            "Persistent backup: {} ({})",
            result.persistent_backup,
            option_text(&result.backup_id)
        ));
        ui.label(format!(
            "Hash before: {} · after: {}",
            result.hash_before, result.hash_after
        ));
        for err in &result.validation_errors {
            ui.label(egui::RichText::new(err).color(theme::WARNING));
        }
    });
}

/// Renders errors, warnings, validation and the last outcome shared by both
/// switchers. Returns true when Retry was clicked.
fn switch_status(
    ui: &mut egui::Ui,
    error: &Option<String>,
    validation: &Option<String>,
    warnings: &[String],
    message: &Option<String>,
) -> bool {
    let retry = error
        .as_ref()
        .map(|err| {
            comp::error_state(
                ui,
                &format!("Switch session error: {err}"),
                Some("Reinitialize"),
            )
        })
        .unwrap_or(false);
    comp::validation_message(ui, validation.as_deref().unwrap_or(""));
    for warning in warnings {
        ui.label(egui::RichText::new(format!("Warning: {warning}")).color(theme::WARNING));
    }
    if let Some(message) = message {
        ui.label(egui::RichText::new(message.as_str()).color(theme::SUCCESS));
    }
    retry
}

fn render_switch(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>, view: SubView) {
    let is_truck = ctx.route == Route::vehicles_truck_switcher;
    let (route, name) = if is_truck {
        (Route::vehicles_truck_switcher, "Truck Switcher")
    } else {
        (Route::vehicles_trailer_switcher, "Trailer Switcher")
    };
    sub_view_nav(
        ui,
        ctx,
        route,
        view,
        &[
            (SubView::SwitchSession, "Session"),
            (SubView::SwitchList, "List"),
            (SubView::SwitchPreview, "Preview"),
            (SubView::SwitchApply, "Apply"),
            (SubView::SwitchResult, "Result"),
        ],
    );
    ui.label(
        egui::RichText::new(format!(
            "{name} — initialize a session, select a vehicle, preview, then apply with a backup."
        ))
        .color(theme::MUTED),
    );

    if is_truck {
        if switch_status(
            ui,
            &state.truck_switch_error,
            &state.truck_switch_validation,
            &state.truck_warnings,
            &state.truck_switch_message,
        ) {
            state.schedule(Operation::InitTruckSwitch, "Initializing truck switch session…");
        }
    } else if switch_status(
        ui,
        &state.trailer_switch_error,
        &state.trailer_switch_validation,
        &state.trailer_warnings,
        &state.trailer_switch_message,
    ) {
        state.schedule(
            Operation::InitTrailerSwitch,
            "Initializing trailer switch session…",
        );
    }

    if is_truck {
        let session = state.truck_session.clone();
        comp::section_card(ui, "Switch session", |ui| {
            match &session {
                Some(session) => {
                    ui.label(format!("Current active truck: {}", session.current_label));
                    ui.label(format!("Expected save hash: {}", session.save_hash));
                }
                None => {
                    ui.label("No session. Scan owned trucks to start one.");
                }
            }
        });
    } else {
        let session = state.trailer_session.clone();
        comp::section_card(ui, "Switch session", |ui| {
            match &session {
                Some(session) => {
                    ui.label(format!("Current active trailer: {}", session.current_label));
                    ui.label(format!("Expected save hash: {}", session.save_hash));
                }
                None => {
                    ui.label("No session. Scan owned trailers to start one.");
                }
            }
        });
    }

    // Scan buttons live outside the card so their click can schedule work directly.
    if is_truck {
        if comp::button(ui, "Scan owned trucks for switch").clicked() {
            state.schedule(Operation::InitTruckSwitch, "Loading owned trucks…");
        }
        if state.truck_session.is_some() && comp::button(ui, "Reinitialize truck session").clicked() {
            state.schedule(Operation::ReinitializeTruck, "Reinitializing truck session…");
        }
    } else {
        if comp::button(ui, "Scan owned trailers for switch").clicked() {
            state.schedule(Operation::InitTrailerSwitch, "Loading owned trailers…");
        }
        if state.trailer_session.is_some()
            && comp::button(ui, "Reinitialize trailer session").clicked()
        {
            state.schedule(Operation::ReinitializeTrailer, "Reinitializing trailer session…");
        }
    }

    if is_truck && state.truck_session.is_none() {
        comp::empty_state(
            ui,
            "No truck switch session",
            "Scan owned trucks to list switchable vehicles.",
        );
        return;
    }
    if !is_truck && state.trailer_session.is_none() {
        comp::empty_state(
            ui,
            "No trailer switch session",
            "Scan owned trailers to list switchable vehicles.",
        );
        return;
    }

    if matches!(view, SubView::SwitchSession | SubView::SwitchList) {
        if is_truck {
            comp::form_field(ui, "Search owned trucks", |ui| {
                comp::text_input(ui, &mut state.truck_search, "Brand, ID, plate or garage");
            });
            if let Some(row) = switch_table(
                ui,
                "fleet_truck_switch_table",
                &state.truck_rows,
                &state.truck_search,
                &state.truck_selected,
            ) {
                state.truck_selected = Some(row.id.clone());
                if row.active || !row.switchable {
                    state.truck_preview = None;
                    invalid(
                        &mut state.truck_switch_validation,
                        ctx,
                        format!("{} is active or blocked and cannot be selected.", row.label),
                    );
                } else {
                    state.schedule(
                        Operation::PreviewTruck(row.id.clone()),
                        "Preparing truck switch preview…",
                    );
                }
            }
        } else {
            comp::form_field(ui, "Search owned trailers", |ui| {
                comp::text_input(ui, &mut state.trailer_search, "Name, ID, plate or garage");
            });
            if let Some(row) = switch_table(
                ui,
                "fleet_trailer_switch_table",
                &state.trailer_rows,
                &state.trailer_search,
                &state.trailer_selected,
            ) {
                state.trailer_selected = Some(row.id.clone());
                if row.active || !row.switchable {
                    state.trailer_preview = None;
                    invalid(
                        &mut state.trailer_switch_validation,
                        ctx,
                        format!("{} is active or blocked and cannot be selected.", row.label),
                    );
                } else {
                    state.schedule(
                        Operation::PreviewTrailer(row.id.clone()),
                        "Preparing trailer switch preview…",
                    );
                }
            }
        }
    }

    if matches!(
        view,
        SubView::SwitchSession | SubView::SwitchPreview | SubView::SwitchApply
    ) {
        if is_truck {
            let hash = state
                .truck_session
                .as_ref()
                .map(|session| session.save_hash.clone())
                .unwrap_or_default();
            let Some(preview) = state.truck_preview.clone() else {
                comp::empty_state(
                    ui,
                    "No truck preview",
                    "Select a switchable truck to see warnings, hash and apply state.",
                );
                return;
            };
            if switch_preview_card(
                ui,
                &preview,
                None,
                &mut state.truck_backup,
                &hash,
            ) {
                state.confirm(
                    "Apply truck switch",
                    format!(
                        "Switch the active truck to {}? A backup will be created according to the toggle, and the switch takes effect before Save.\n{}",
                        preview.target_label,
                        staged_warning(ctx)
                    ),
                    Operation::ApplyTruck,
                );
            }
        } else {
            let hash = state
                .trailer_session
                .as_ref()
                .map(|session| session.save_hash.clone())
                .unwrap_or_default();
            let Some(preview) = state.trailer_preview.clone() else {
                comp::empty_state(
                    ui,
                    "No trailer preview",
                    "Select a switchable trailer to see warnings, hash and apply state.",
                );
                return;
            };
            if switch_preview_card(
                ui,
                &preview,
                None,
                &mut state.trailer_backup,
                &hash,
            ) {
                state.confirm(
                    "Apply trailer switch",
                    format!(
                        "Switch the active trailer to {}? A backup will be created according to the toggle, and the switch takes effect before Save.\n{}",
                        preview.target_label,
                        staged_warning(ctx)
                    ),
                    Operation::ApplyTrailer,
                );
            }
        }
    }

    if matches!(view, SubView::SwitchSession | SubView::SwitchResult) {
        if is_truck {
            if let Some(result) = &state.truck_result {
                switch_result_card(ui, result);
            }
        } else if let Some(result) = &state.trailer_result {
            switch_result_card(ui, result);
        }
    }
}

fn staged_warning(ctx: &ScreenCtx<'_>) -> &'static str {
    if crate::gui::data::session::is_dirty(ctx.data) {
        "Staged edits are still pending; this switch may overwrite them."
    } else {
        "No staged edits are pending."
    }
}

// --------------------------------------------------------- powertrain rendering

fn render_powertrain(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
    view: SubView,
) {
    sub_view_nav(
        ui,
        ctx,
        Route::vehicles_powertrain,
        view,
        &[
            (SubView::PowertrainCatalog, "Catalog"),
            (SubView::PowertrainPreview, "Preview"),
        ],
    );
    if let Some(err) = state.catalog_error.clone() {
        if comp::error_state(ui, &format!("Catalog unavailable: {err}"), Some("Retry")) {
            state.schedule(Operation::LoadCatalog, "Loading official powertrain catalog…");
        }
        return;
    }
    if let Some(err) = state.powertrain_error.clone() {
        if comp::error_state(ui, &format!("Powertrain preview failed: {err}"), Some("Retry")) {
            state.schedule(Operation::PreviewPowertrain, "Preparing powertrain preview…");
        }
        return;
    }
    let Some(catalog) = state.catalog.clone() else {
        comp::empty_state(
            ui,
            "No catalog entries for this game/version",
            "Retry the catalog load, or switch the catalog game below.",
        );
        return;
    };

    comp::section_card(ui, "Official powertrain catalog", |ui| {
        ui.label(format!(
            "Catalog: {} (v{}) — {} engines, {} transmissions",
            catalog.game,
            catalog.version,
            catalog.engines.len(),
            catalog.transmissions.len()
        ));
        let mut game = state.game();
        let games = vec!["ets2".to_string(), "ats".to_string()];
        comp::form_field(ui, "Catalog game", |ui| {
            comp::select(ui, "fleet_catalog_game", &mut game, &games);
        });
        if game != state.game() {
            state.catalog_game = game;
            state.schedule(Operation::LoadCatalog, "Loading official powertrain catalog…");
            return;
        }

        if catalog.engines.is_empty() && catalog.transmissions.is_empty() {
            ui.label("The catalog has no components for this selection.");
            return;
        }

        let engine_options: Vec<String> =
            catalog.engines.iter().map(|entry| entry.label.clone()).collect();
        let mut engine_label = catalog
            .engines
            .iter()
            .find(|entry| entry.data_path == state.engine_path)
            .map(|entry| entry.label.clone())
            .or_else(|| engine_options.first().cloned())
            .unwrap_or_default();
        comp::form_field(ui, "Engine (data_path)", |ui| {
            comp::select(ui, "fleet_powertrain_engine", &mut engine_label, &engine_options);
        });
        if let Some(entry) = catalog
            .engines
            .iter()
            .find(|entry| entry.label == engine_label)
        {
            state.engine_path = entry.data_path.clone();
        }

        let transmission_options: Vec<String> = catalog
            .transmissions
            .iter()
            .map(|entry| entry.label.clone())
            .collect();
        let mut transmission_label = catalog
            .transmissions
            .iter()
            .find(|entry| entry.data_path == state.transmission_path)
            .map(|entry| entry.label.clone())
            .or_else(|| transmission_options.first().cloned())
            .unwrap_or_default();
        comp::form_field(ui, "Transmission (data_path)", |ui| {
            comp::select(
                ui,
                "fleet_powertrain_transmission",
                &mut transmission_label,
                &transmission_options,
            );
        });
        if let Some(entry) = catalog
            .transmissions
            .iter()
            .find(|entry| entry.label == transmission_label)
        {
            state.transmission_path = entry.data_path.clone();
        }

        if comp::button(ui, "Preview powertrain change on player truck").clicked() {
            state.schedule(Operation::PreviewPowertrain, "Preparing powertrain preview…");
        }
    });

    comp::validation_message(ui, state.catalog_validation.as_deref().unwrap_or(""));
    ui.label(
        egui::RichText::new("Preview is read-only: the backend exposes no powertrain apply command.")
            .color(theme::MUTED),
    );
    if comp::button(ui, "Open player truck").clicked() {
        navigate(
            Route::vehicles_player_truck,
            SubView::TruckFields,
            ctx,
        );
    }

    if view == SubView::PowertrainPreview {
        let Some(preview) = state.powertrain_preview.clone() else {
            comp::empty_state(
                ui,
                "No powertrain preview",
                "Select a component and preview it against the player truck.",
            );
            return;
        };
        comp::section_card(ui, "Powertrain preview", |ui| {
            ui.label(format!(
                "Truck: {} (family: {})",
                preview.truck_id,
                preview.family.clone().unwrap_or_else(|| "-".to_string())
            ));
            ui.label(format!(
                "Cross-brand experimental: {} · Can apply later: {}",
                preview.cross_brand, preview.can_apply_later
            ));
            ui.label(format!(
                "Engine: {} → {}",
                option_text(&preview.current_engine),
                option_text(&preview.new_engine)
            ));
            ui.label(format!(
                "Transmission: {} → {}",
                option_text(&preview.current_transmission),
                option_text(&preview.new_transmission)
            ));
            if let Some(differential) = preview.differential {
                ui.label(format!("Selected differential ratio: {differential:.2}"));
            }
            for warning in &preview.warnings {
                ui.label(
                    egui::RichText::new(format!("Warning: {warning}")).color(theme::WARNING),
                );
            }
        });
    }
}

// --------------------------------------------------------- fleet inventory UI

fn sort_header<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    current: &mut T,
    ascending: &mut bool,
    label: &str,
    key: T,
) {
    let arrow = if *current == key {
        if *ascending { " ^" } else { " v" }
    } else {
        ""
    };
    if comp::button(ui, &format!("{label}{arrow}")).clicked() {
        if *current == key {
            *ascending = !*ascending;
        } else {
            *current = key;
            *ascending = true;
        }
    }
}

fn render_fleet(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>, view: SubView) {
    sub_view_nav(
        ui,
        ctx,
        Route::vehicles_fleet_inventory,
        view,
        &[
            (SubView::FleetTrucks, "Trucks"),
            (SubView::FleetTrailers, "Trailers"),
        ],
    );
    if let Some(err) = state.fleet_error.clone() {
        if comp::error_state(ui, &format!("Fleet inventory unavailable: {err}"), Some("Retry")) {
            state.schedule(Operation::ReloadFleet, "Loading fleet inventory…");
        }
        return;
    }
    if !state.fleet_tried {
        comp::loading_state(ui, "Loading fleet inventory…");
        return;
    }
    if state.fleet_trucks.is_empty() && state.fleet_trailers.is_empty() {
        comp::empty_state(
            ui,
            "No trucks/trailers found",
            "Select a save in the context column, then retry the load.",
        );
        if comp::button(ui, "Reload fleet inventory").clicked() {
            state.schedule(Operation::ReloadFleet, "Loading fleet inventory…");
        }
        return;
    }

    comp::form_field(ui, "Search fleet", |ui| {
        comp::text_input(ui, &mut state.fleet_search, "Brand, ID, plate, body or garage");
    });

    let filter = state.fleet_search.trim().to_ascii_lowercase();
    if view == SubView::FleetTrucks {
        let mut trucks: Vec<FleetTruckView> = state
            .fleet_trucks
            .iter()
            .filter(|truck| {
                filter.is_empty()
                    || truck.brand_model.to_ascii_lowercase().contains(&filter)
                    || truck.id.to_ascii_lowercase().contains(&filter)
                    || truck.plate.to_ascii_lowercase().contains(&filter)
                    || truck.garage.to_ascii_lowercase().contains(&filter)
            })
            .cloned()
            .collect();
        let sort = state.truck_sort;
        let ascending = state.truck_sort_asc;
        trucks.sort_by(|a, b| {
            let order = match sort {
                TruckSort::Brand => a.brand_model.cmp(&b.brand_model),
                TruckSort::Odometer => a
                    .odometer
                    .partial_cmp(&b.odometer)
                    .unwrap_or(std::cmp::Ordering::Equal),
                TruckSort::Fuel => a
                    .fuel
                    .partial_cmp(&b.fuel)
                    .unwrap_or(std::cmp::Ordering::Equal),
                TruckSort::Garage => a.garage.cmp(&b.garage),
            };
            if ascending {
                order
            } else {
                order.reverse()
            }
        });
        ui.label(egui::RichText::new(format!("Owned trucks ({})", trucks.len())).strong());
        if trucks.is_empty() {
            comp::empty_state(ui, "No trucks match", "Try a different search.");
        } else {
            ui.push_id("fleet_trucks_grid", |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("fleet_trucks_scroll")
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        egui::Grid::new("fleet_trucks_columns")
                            .striped(true)
                            .min_col_width(92.0)
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("Truck ID").strong());
                                sort_header(
                                    ui,
                                    &mut state.truck_sort,
                                    &mut state.truck_sort_asc,
                                    "Brand & model",
                                    TruckSort::Brand,
                                );
                                ui.label(egui::RichText::new("Plate").strong());
                                sort_header(
                                    ui,
                                    &mut state.truck_sort,
                                    &mut state.truck_sort_asc,
                                    "Odometer",
                                    TruckSort::Odometer,
                                );
                                sort_header(
                                    ui,
                                    &mut state.truck_sort,
                                    &mut state.truck_sort_asc,
                                    "Fuel",
                                    TruckSort::Fuel,
                                );
                                sort_header(
                                    ui,
                                    &mut state.truck_sort,
                                    &mut state.truck_sort_asc,
                                    "Garage",
                                    TruckSort::Garage,
                                );
                                ui.end_row();
                                for truck in trucks {
                                    ui.label(&truck.id);
                                    ui.label(&truck.brand_model);
                                    ui.label(&truck.plate);
                                    ui.label(format!("{:.0}", truck.odometer));
                                    ui.label(format!("{:.0}%", truck.fuel * 100.0));
                                    ui.label(&truck.garage);
                                    ui.end_row();
                                }
                            });
                    });
            });
        }
    } else {
        let mut trailers: Vec<FleetTrailerView> = state
            .fleet_trailers
            .iter()
            .filter(|trailer| {
                filter.is_empty()
                    || trailer.body.to_ascii_lowercase().contains(&filter)
                    || trailer.id.to_ascii_lowercase().contains(&filter)
                    || trailer.plate.to_ascii_lowercase().contains(&filter)
            })
            .cloned()
            .collect();
        let sort = state.trailer_sort;
        let ascending = state.trailer_sort_asc;
        trailers.sort_by(|a, b| {
            let order = match sort {
                TrailerSort::Body => a.body.cmp(&b.body),
                TrailerSort::Cargo => a
                    .cargo_mass
                    .partial_cmp(&b.cargo_mass)
                    .unwrap_or(std::cmp::Ordering::Equal),
                TrailerSort::Wear => a.wear.cmp(&b.wear),
            };
            if ascending {
                order
            } else {
                order.reverse()
            }
        });
        ui.label(
            egui::RichText::new(format!("Owned trailers ({})", trailers.len())).strong(),
        );
        if trailers.is_empty() {
            comp::empty_state(ui, "No trailers match", "Try a different search.");
        } else {
            ui.push_id("fleet_trailers_grid", |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("fleet_trailers_scroll")
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        egui::Grid::new("fleet_trailers_columns")
                            .striped(true)
                            .min_col_width(92.0)
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("Trailer ID").strong());
                                sort_header(
                                    ui,
                                    &mut state.trailer_sort,
                                    &mut state.trailer_sort_asc,
                                    "Body type",
                                    TrailerSort::Body,
                                );
                                ui.label(egui::RichText::new("Plate").strong());
                                sort_header(
                                    ui,
                                    &mut state.trailer_sort,
                                    &mut state.trailer_sort_asc,
                                    "Cargo mass",
                                    TrailerSort::Cargo,
                                );
                                sort_header(
                                    ui,
                                    &mut state.trailer_sort,
                                    &mut state.trailer_sort_asc,
                                    "Body / chassis wear",
                                    TrailerSort::Wear,
                                );
                                ui.end_row();
                                for trailer in trailers {
                                    ui.label(&trailer.id);
                                    ui.label(&trailer.body);
                                    ui.label(&trailer.plate);
                                    ui.label(format!("{:.1}", trailer.cargo_mass));
                                    ui.label(&trailer.wear);
                                    ui.end_row();
                                }
                            });
                    });
            });
        }
    }
}

// -------------------------------------------------------- shared confirm layer

fn render_confirm(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let Some(confirmation) = state.confirm.clone() else {
        return;
    };
    let mut message = confirmation.message.clone();
    if matches!(
        &confirmation.action,
        Operation::ApplyTruck | Operation::ApplyTrailer | Operation::ReloadAll
    ) && crate::gui::data::session::is_dirty(ctx.data)
    {
        message.push_str("\n\nStaged edits are still pending; this action may overwrite them.");
    }
    match comp::confirm_dialog(
        ui.ctx(),
        confirmation.title,
        &message,
        "Confirm",
        "Cancel",
    ) {
        Some(true) => state.schedule(confirmation.action, "Applying confirmed action…"),
        Some(false) => info(ctx, "Action cancelled.".to_string()),
        None => {}
    }
}
