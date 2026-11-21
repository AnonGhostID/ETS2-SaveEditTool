//! Operations sector: Garages and AI Drivers.
//!
//! Backend access is exclusively through `crate::gui::data::garages` and
//! `crate::gui::data::session`; every global effect is a `ShellCommand`.
//! Sub-views are supplied by the shell and only ever requested through
//! `ShellCommand::Navigate`.

use eframe::egui;

use crate::gui::components as comp;
use crate::gui::data::loading::BackgroundLoad;
use crate::gui::screens::{IssuesTab, Route, ScreenCtx, ShellCommand, SubView};
use crate::gui::theme;

// Model types come from the data layer's public re-exports.
type GarageList = crate::gui::data::garages::GarageListResult;
type DriverList = crate::gui::data::garages::AiDriverManagementResult;
type Ownership = crate::gui::data::garages::GarageOwnership;
type Size = crate::gui::data::garages::GarageSize;
type MutationRequest = crate::gui::data::garages::GarageMutationRequest;
type ResourceRequest = crate::gui::data::garages::GarageResourceAssignmentRequest;
type UpdateRequest = crate::gui::data::garages::GarageUpdateRequest;
type BuyAllRequest = crate::gui::data::garages::GarageBuyAllRequest;
type SellEmptyRequest = crate::gui::data::garages::GarageRelinquishEmptyRequest;
type GenerateRequest = crate::gui::data::garages::AiDriverGenerateRequest;
type RemoveRequest = crate::gui::data::garages::AiDriverRemoveRequest;
type TruckAssignmentRequest = crate::gui::data::garages::AiDriverTruckAssignmentRequest;

const GARAGE_HEADERS: &[&str] = &[
    "Garage / City",
    "Country",
    "Ownership",
    "Size",
    "Slots",
    "Drivers / Trucks",
];
const DRIVER_HEADERS: &[&str] = &["Driver", "Level / XP", "Garage", "Truck", "State"];
const UNASSIGNED: &str = "(none — unassign)";

// ------------------------------------------------------------- local DTO views

#[derive(Debug, Clone, Default)]
struct GarageData {
    hash: String,
    hq_label: String,
    rows: Vec<GarageRow>,
    owned_count: usize,
    unowned_count: usize,
    warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct GarageRow {
    id: String,
    label: String,
    country: String,
    ownership_label: String,
    size_label: String,
    slots: String,
    occupied: usize,
    owned: bool,
    hq: bool,
    sellable: bool,
    drivers: usize,
    trucks: usize,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct DriverData {
    hash: String,
    rows: Vec<DriverRow>,
    trucks: Vec<TruckChoice>,
    available_count: usize,
    warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct DriverRow {
    id: String,
    name: String,
    level: u32,
    xp: u64,
    garage_id: String,
    garage: String,
    truck_id: Option<String>,
    truck: String,
    state: String,
    removable: bool,
    warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct TruckChoice {
    id: String,
    label: String,
    garage_id: String,
}

fn garage_data_from(list: &GarageList) -> GarageData {
    let rows: Vec<GarageRow> = list
        .garages
        .iter()
        .map(|g| {
            let label = g
                .city_name
                .as_deref()
                .or(g.city_token.as_deref())
                .unwrap_or(&g.garage_id)
                .to_string();
            let owned = matches!(g.ownership, Ownership::Owned);
            // Mirror the backend's strict relinquish rule: HQ, occupied or
            // trailer-bearing garages are refused (audit G4/G5).
            let sellable = matches!(g.size, Size::Small | Size::Large)
                && matches!(g.status, Some(2) | Some(3))
                && owned
                && !g.is_headquarters
                && g.occupied_slots == 0
                && g.assigned_driver_count == 0
                && g.assigned_truck_count == 0
                && g.assigned_trailer_count == 0
                && g.trailer_slot_count == 0;
            GarageRow {
                id: g.garage_id.clone(),
                label,
                country: g.country_code.clone().unwrap_or_else(|| "-".to_string()),
                ownership_label: if owned {
                    "Owned".to_string()
                } else if matches!(g.ownership, Ownership::NotOwned) {
                    "Unowned".to_string()
                } else {
                    "Unknown".to_string()
                },
                size_label: format!("{:?}", g.size),
                slots: format!("{}/{}", g.occupied_slots, g.maximum_slot_count),
                occupied: g.occupied_slots,
                owned,
                hq: g.is_headquarters,
                sellable,
                drivers: g.assigned_driver_count,
                trucks: g.assigned_truck_count,
                warnings: g.warnings.clone(),
            }
        })
        .collect();
    let hq_label = list
        .headquarters_garage_id
        .as_deref()
        .and_then(|id| rows.iter().find(|row| row.id == id))
        .map(|row| row.label.clone())
        .unwrap_or_else(|| "-".to_string());
    GarageData {
        hash: list.save_hash.clone(),
        hq_label,
        rows,
        owned_count: list.diagnostics.owned_garage_count,
        unowned_count: list.diagnostics.not_owned_garage_count,
        warnings: list.diagnostics.warnings.clone(),
    }
}

fn driver_data_from(mgmt: &DriverList) -> DriverData {
    DriverData {
        hash: mgmt.save_hash.clone(),
        rows: mgmt
            .drivers
            .iter()
            .map(|driver| DriverRow {
                id: driver.driver_id.clone(),
                name: driver
                    .display_name
                    .clone()
                    .unwrap_or_else(|| driver.driver_id.clone()),
                level: driver.level.unwrap_or(0),
                xp: driver.experience_points.unwrap_or(0),
                garage_id: driver.garage_id.clone(),
                garage: driver
                    .garage_name
                    .clone()
                    .unwrap_or_else(|| driver.garage_id.clone()),
                truck_id: driver.truck_id.clone(),
                truck: driver.truck_id.clone().unwrap_or_else(|| "-".to_string()),
                state: driver
                    .state
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "?".to_string()),
                removable: driver.removable,
                warnings: driver.warnings.clone(),
            })
            .collect(),
        trucks: mgmt
            .available_trucks
            .iter()
            .map(|truck| TruckChoice {
                id: truck.truck_id.clone(),
                label: format!(
                    "{} — {}",
                    truck.truck_id,
                    truck
                        .garage_name
                        .clone()
                        .unwrap_or_else(|| truck.garage_id.clone())
                ),
                garage_id: truck.garage_id.clone(),
            })
            .collect(),
        available_count: mgmt.available_driver_count,
        warnings: mgmt.warnings.clone(),
    }
}

// ------------------------------------------------------------- deferred work

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum SortKey {
    #[default]
    City,
    Ownership,
    Size,
    Slots,
    Drivers,
}

type LoadOutcome = Result<(GarageData, DriverData), String>;

#[derive(Debug, Clone)]
enum Operation {
    Purchase {
        id: String,
        hash: String,
        label: String,
    },
    Upgrade {
        id: String,
        hash: String,
        label: String,
    },
    AssignRandom {
        id: String,
        hash: String,
        label: String,
    },
    SetHeadquarters {
        id: String,
        hash: String,
        label: String,
    },
    Sell {
        id: String,
        hash: String,
        label: String,
    },
    SellEmpty {
        hash: String,
        names: Vec<(String, String)>,
    },
    BuyAll {
        hash: String,
    },
    Generate {
        garage_id: String,
        hash: String,
        count: usize,
        assign_trucks: bool,
    },
    AssignTruck {
        driver_id: String,
        hash: String,
        truck_id: Option<String>,
        label: String,
    },
    ReassignRandom {
        garage_id: String,
        hash: String,
        label: String,
    },
    Dismiss {
        ids: Vec<String>,
        hash: String,
        labels: Vec<String>,
    },
}

#[derive(Debug, Clone)]
struct Confirm {
    title: &'static str,
    message: String,
    confirm_label: &'static str,
    action: Operation,
}

/// All local state. Navigation lives in `ScreenCtx`; this instance survives
/// tile switches (docs/ui-spec.md §1.3 rule 5).
#[derive(Default)]
pub struct ScreenState {
    load: BackgroundLoad<LoadOutcome>,
    garage_data: Option<GarageData>,
    driver_data: Option<DriverData>,
    load_error: Option<String>,
    last_error: Option<String>,
    last_message: Option<String>,
    staged: usize,
    filter_text: String,
    only_owned: bool,
    sort_key: SortKey,
    sort_asc: bool,
    selected_garage: Option<String>,
    driver_garage_input: String,
    driver_garage_ready: bool,
    generate_count: usize,
    assign_trucks: bool,
    selected_driver: String,
    selected_truck: String,
    dismiss_selection: Vec<String>,
    confirm: Option<Confirm>,
    pending: Option<Operation>,
    loading: Option<&'static str>,
}

impl ScreenState {
    fn schedule(&mut self, operation: Operation, message: &'static str) {
        self.pending = Some(operation);
        self.loading = Some(message);
    }

    fn ask_confirm(
        &mut self,
        title: &'static str,
        message: String,
        confirm_label: &'static str,
        action: Operation,
    ) {
        self.confirm = Some(Confirm {
            title,
            message,
            confirm_label,
            action,
        });
    }
}

// ---------------------------------------------------------------- feedback

fn toast(ctx: &mut ScreenCtx<'_>, message: String, is_error: bool) {
    ctx.commands.push(ShellCommand::Toast { message, is_error });
}

fn info(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, message, false);
}

fn error(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, message, true);
    ctx.commands
        .push(ShellCommand::OpenIssues(IssuesTab::Errors));
}

fn validation(ctx: &mut ScreenCtx<'_>, message: String) {
    // The shell routes the `Validation: ` prefix into the drawer's
    // Validation tab (docs/components.md §1.3).
    toast(ctx, format!("Validation: {message}"), false);
    ctx.commands
        .push(ShellCommand::OpenIssues(IssuesTab::Validation));
}

fn require(ok: bool, message: String, ctx: &mut ScreenCtx<'_>) -> bool {
    if ok {
        true
    } else {
        validation(ctx, message);
        false
    }
}

fn with_warnings(message: String, warnings: &[String]) -> String {
    if warnings.is_empty() {
        message
    } else {
        format!("{} Warnings: {}", message, warnings.join("; "))
    }
}

fn backup_note(created: bool) -> &'static str {
    if created {
        " A backup was created."
    } else {
        " No new backup was created."
    }
}

// ---------------------------------------------------------------- routing

fn navigate(route: Route, sub_view: SubView, ctx: &mut ScreenCtx<'_>) {
    ctx.commands.push(ShellCommand::Navigate {
        route,
        sub_view: Some(sub_view),
    });
}

/// Validate the shell-owned sub-view; never keep a local navigation state.
fn normalized_sub_view(route: Route, view: SubView) -> SubView {
    match route {
        Route::operations_garages => match view {
            SubView::GarageDetail | SubView::GarageWarnings => view,
            _ => SubView::GarageList,
        },
        Route::operations_ai_drivers => match view {
            SubView::DriverAssign | SubView::DriverDismiss => view,
            _ => SubView::DriverGenerate,
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
    ui.horizontal(|ui| {
        for (view, label) in items {
            if *view == current {
                ui.label(egui::RichText::new(*label).strong().color(theme::ACCENT));
            } else if comp::button(ui, label).clicked() {
                navigate(route, *view, ctx);
            }
        }
    });
    ui.add_space(6.0);
}

// ---------------------------------------------------------------- loading

fn start_load(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    if state.load.is_loading() {
        return;
    }
    let data = (*ctx.data).clone();
    state.load.start(move || {
        // Both reads run on a background thread; post-mutation refreshes use
        // the synchronous `reload` below so the next edit has a fresh hash.
        let list = crate::gui::data::garages::list_all_garages(&data);
        let drivers = crate::gui::data::garages::ai_driver_management(&data);
        match (list, drivers) {
            (Ok(list), Ok(mgmt)) => Ok((garage_data_from(&list), driver_data_from(&mgmt))),
            (Err(err), _) | (_, Err(err)) => Err(err.message),
        }
    });
}

/// Synchronous refresh after a mutation (the next edit needs the new hash).
fn reload(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match crate::gui::data::garages::list_all_garages(ctx.data) {
        Ok(list) => state.garage_data = Some(garage_data_from(&list)),
        Err(err) => {
            state.garage_data = None;
            state.selected_garage = None;
            error(ctx, format!("Garage load error: {}", err.message));
        }
    }
    match crate::gui::data::garages::ai_driver_management(ctx.data) {
        Ok(mgmt) => state.driver_data = Some(driver_data_from(&mgmt)),
        Err(err) => {
            state.driver_data = None;
            error(ctx, format!("AI driver load error: {}", err.message));
        }
    }
    state.staged = crate::gui::data::session::dirty_paths(ctx.data).len();
}

// --------------------------------------------------------- operation runner

fn execute(operation: Operation, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let outcome: Result<String, String> = match operation {
        Operation::Purchase { id, hash, label } => {
            let request = MutationRequest {
                garage_id: id,
                expected_save_hash: hash,
            };
            match crate::gui::data::garages::purchase_garage(ctx.data, request) {
                Ok(result) => Ok(with_warnings(
                    format!(
                        "Purchased {label}.{}",
                        backup_note(result.backup_created)
                    ),
                    &result.warnings,
                )),
                Err(err) => Err(err.message),
            }
        }
        Operation::Upgrade { id, hash, label } => {
            let request = MutationRequest {
                garage_id: id,
                expected_save_hash: hash,
            };
            match crate::gui::data::garages::upgrade_garage(ctx.data, request) {
                Ok(result) => Ok(with_warnings(
                    format!(
                        "Upgraded {label}.{}",
                        backup_note(result.backup_created)
                    ),
                    &result.warnings,
                )),
                Err(err) => Err(err.message),
            }
        }
        Operation::AssignRandom { id, hash, label } => {
            let request = ResourceRequest {
                garage_id: id,
                expected_save_hash: hash,
                assign_random_driver: true,
                assign_random_truck: true,
            };
            match crate::gui::data::garages::assign_random_resources(ctx.data, request) {
                Ok(result) => Ok(with_warnings(
                    format!(
                        "Assigned random driver and truck to {label}.{}",
                        backup_note(result.backup_created)
                    ),
                    &result.warnings,
                )),
                Err(err) => Err(err.message),
            }
        }
        Operation::SetHeadquarters { id, hash, label } => {
            let request = UpdateRequest {
                garage_id: id,
                expected_save_hash: hash,
                target_size: None,
                set_as_headquarters: true,
            };
            match crate::gui::data::garages::update_garage(ctx.data, request) {
                Ok(result) => Ok(with_warnings(
                    format!(
                        "Set {label} as headquarters.{}",
                        backup_note(result.backup_created)
                    ),
                    &result.warnings,
                )),
                Err(err) => Err(err.message),
            }
        }
        Operation::Sell { id, hash, label } => {
            let request = MutationRequest {
                garage_id: id,
                expected_save_hash: hash,
            };
            match crate::gui::data::garages::sell_garage(ctx.data, request) {
                Ok(result) => Ok(with_warnings(
                    format!(
                        "Relinquished {label}.{}",
                        backup_note(result.backup_created)
                    ),
                    &result.warnings,
                )),
                Err(err) => Err(err.message),
            }
        }
        Operation::SellEmpty { hash, names } => {
            let request = SellEmptyRequest {
                expected_save_hash: hash,
            };
            match crate::gui::data::garages::sell_empty_garages(ctx.data, request) {
                Ok(result) => {
                    let mut message = format!(
                        "Relinquished {} empty garages.{}",
                        result.relinquished_count,
                        backup_note(result.backup_created)
                    );
                    if !result.skipped_garage_ids.is_empty() {
                        let skipped: Vec<String> = result
                            .skipped_garage_ids
                            .iter()
                            .map(|id| {
                                names
                                    .iter()
                                    .find(|(candidate, _)| candidate == id)
                                    .map(|(_, name)| name.clone())
                                    .unwrap_or_else(|| id.clone())
                            })
                            .collect();
                        message.push_str(&format!(
                            " Skipped {} (headquarters or trailers): {}.",
                            skipped.len(),
                            skipped.join(", ")
                        ));
                    }
                    Ok(with_warnings(message, &result.warnings))
                }
                Err(err) => Err(err.message),
            }
        }
        Operation::BuyAll { hash } => {
            let request = BuyAllRequest {
                expected_save_hash: hash,
            };
            match crate::gui::data::garages::buy_all_garages(ctx.data, request) {
                Ok(result) => Ok(with_warnings(
                    format!(
                        "Purchased {} garages.{}",
                        result.purchased_count,
                        backup_note(result.backup_created)
                    ),
                    &result.warnings,
                )),
                Err(err) => Err(err.message),
            }
        }
        Operation::Generate {
            garage_id,
            hash,
            count,
            assign_trucks,
        } => {
            let request = GenerateRequest {
                garage_id,
                expected_save_hash: hash,
                count,
                assign_trucks,
            };
            match crate::gui::data::garages::generate_drivers(ctx.data, request) {
                Ok(result) => Ok(with_warnings(
                    format!(
                        "Generated {} AI driver(s).{}",
                        result.affected_driver_ids.len(),
                        backup_note(result.backup_created)
                    ),
                    &result.warnings,
                )),
                Err(err) => Err(err.message),
            }
        }
        Operation::AssignTruck {
            driver_id,
            hash,
            truck_id,
            label,
        } => {
            let request = TruckAssignmentRequest {
                driver_id,
                expected_save_hash: hash,
                truck_id,
            };
            match crate::gui::data::garages::update_driver_truck(ctx.data, request) {
                Ok(result) => Ok(with_warnings(
                    format!(
                        "Updated truck assignment for {label}.{}",
                        backup_note(result.backup_created)
                    ),
                    &result.warnings,
                )),
                Err(err) => Err(err.message),
            }
        }
        Operation::ReassignRandom {
            garage_id,
            hash,
            label,
        } => {
            let request = ResourceRequest {
                garage_id,
                expected_save_hash: hash,
                assign_random_driver: false,
                assign_random_truck: true,
            };
            match crate::gui::data::garages::assign_random_resources(ctx.data, request) {
                Ok(result) => Ok(with_warnings(
                    format!(
                        "Reassigned a random truck at {label}.{}",
                        backup_note(result.backup_created)
                    ),
                    &result.warnings,
                )),
                Err(err) => Err(err.message),
            }
        }
        Operation::Dismiss {
            ids,
            hash,
            labels,
        } => {
            let request = RemoveRequest {
                driver_ids: ids,
                expected_save_hash: hash,
            };
            match crate::gui::data::garages::remove_drivers(ctx.data, request) {
                Ok(result) => Ok(with_warnings(
                    format!(
                        "Removed {}.{}",
                        labels.join(", "),
                        backup_note(result.backup_created)
                    ),
                    &result.warnings,
                )),
                Err(err) => Err(err.message),
            }
        }
    };

    match outcome {
        Ok(message) => {
            state.last_error = None;
            state.last_message = Some(message.clone());
            reload(state, ctx);
            // Screen-level mutations can create backups; keep the Safety Spine
            // backup count and last outcome current without a full Save.
            ctx.commands.push(ShellCommand::RefreshSafety);
            info(ctx, message);
        }
        Err(message) => {
            // Optimistic-concurrency and validation failures stay visible in
            // the Detail Pane as well as in the toast/Issues surfaces.
            state.last_message = None;
            state.last_error = Some(message.clone());
            error(ctx, message);
        }
    }
}

// ---------------------------------------------------------------- rendering

/// Exact screen contract: one persistent state and one render function.
pub fn render(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    // Deferred work runs at frame start so its loading state can be shown.
    if let Some(operation) = state.pending.take() {
        execute(operation, state, ctx);
    }
    if let Some(result) = state.load.poll() {
        match result {
            Ok((garages, drivers)) => {
                state.garage_data = Some(garages);
                state.driver_data = Some(drivers);
                state.load_error = None;
                state.staged = crate::gui::data::session::dirty_paths(ctx.data).len();
                state.driver_garage_ready = false;
                info(ctx, "Loaded garages and AI drivers.".to_string());
            }
            Err(message) => {
                // G6: a failed load must not leave stale data with a stale
                // save hash behind.
                state.garage_data = None;
                state.driver_data = None;
                state.selected_garage = None;
                state.load_error = Some(message.clone());
                error(ctx, format!("Garage load error: {message}"));
            }
        }
    }
    if state.load.is_loading() {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(100));
    }
    if state.generate_count == 0 {
        state.generate_count = 1;
    }
    if !state.driver_garage_ready {
        state.driver_garage_input = state
            .garage_data
            .as_ref()
            .and_then(|data| data.rows.iter().find(|row| row.owned))
            .map(|row| row.id.clone())
            .unwrap_or_default();
        state.driver_garage_ready = true;
    }
    state.staged = crate::gui::data::session::dirty_paths(ctx.data).len();

    ui.horizontal_wrapped(|ui| {
        if comp::button(ui, "Load garages & drivers").clicked() {
            state.load_error = None;
            start_load(state, ctx);
        }
        if comp::button(ui, "Open Issues").clicked() {
            ctx.commands
                .push(ShellCommand::OpenIssues(IssuesTab::Errors));
        }
        ui.label(
            egui::RichText::new(format!(
                "{} staged edit(s) — edits stage until Save in the Safety Spine",
                state.staged
            ))
            .size(10.5)
            .color(theme::MUTED),
        );
    });
    ui.add_space(4.0);

    if let Some(message) = state.loading {
        comp::loading_state(ui, message);
    }
    if state.load.is_loading() {
        comp::loading_state(ui, "Loading garages and AI drivers…");
    }
    let failure = state.load_error.clone().or_else(|| state.last_error.clone());
    if let Some(message) = failure {
        if comp::error_state(ui, &format!("Garage data error: {message}"), Some("Reload")) {
            state.load_error = None;
            state.last_error = None;
            start_load(state, ctx);
        }
    }
    if let Some(message) = state.last_message.clone() {
        ui.label(
            egui::RichText::new(message)
                .small()
                .color(theme::SUCCESS),
        );
    }

    let view = normalized_sub_view(ctx.route, ctx.sub_view);
    match ctx.route {
        Route::operations_garages => sub_view_nav(
            ui,
            ctx,
            Route::operations_garages,
            view,
            &[
                (SubView::GarageList, "Garage list"),
                (SubView::GarageDetail, "Garage detail"),
                (SubView::GarageWarnings, "Warnings"),
            ],
        ),
        Route::operations_ai_drivers => sub_view_nav(
            ui,
            ctx,
            Route::operations_ai_drivers,
            view,
            &[
                (SubView::DriverGenerate, "Generate"),
                (SubView::DriverAssign, "Assign / reassign"),
                (SubView::DriverDismiss, "Dismiss"),
            ],
        ),
        _ => {}
    }

    match (ctx.route, view) {
        (Route::operations_garages, SubView::GarageDetail) => {
            render_garage_detail(ui, state, ctx)
        }
        (Route::operations_garages, SubView::GarageWarnings) => render_warnings(ui, state, ctx),
        (Route::operations_garages, _) => render_garage_list(ui, state, ctx),
        (Route::operations_ai_drivers, SubView::DriverAssign) => {
            render_driver_assign(ui, state, ctx)
        }
        (Route::operations_ai_drivers, SubView::DriverDismiss) => {
            render_driver_dismiss(ui, state, ctx)
        }
        (Route::operations_ai_drivers, _) => render_driver_generate(ui, state, ctx),
        _ => render_garage_list(ui, state, ctx),
    }

    render_confirm(ui, state, ctx);

    if state.pending.is_none() {
        state.loading = None;
    }
}

fn sort_header(state: &mut ScreenState, ui: &mut egui::Ui, label: &str, key: SortKey) {
    let arrow = if state.sort_key == key {
        if state.sort_asc { " ^" } else { " v" }
    } else {
        ""
    };
    if comp::button(ui, &format!("{label}{arrow}")).clicked() {
        if state.sort_key == key {
            state.sort_asc = !state.sort_asc;
        } else {
            state.sort_key = key;
            state.sort_asc = true;
        }
    }
}

fn visible_rows(state: &ScreenState) -> Vec<GarageRow> {
    let Some(data) = &state.garage_data else {
        return Vec::new();
    };
    let needle = state.filter_text.trim().to_ascii_lowercase();
    let mut rows: Vec<GarageRow> = data
        .rows
        .iter()
        .filter(|row| {
            (!state.only_owned || row.owned)
                && (needle.is_empty()
                    || row.label.to_ascii_lowercase().contains(&needle)
                    || row.id.to_ascii_lowercase().contains(&needle))
        })
        .cloned()
        .collect();
    let key = state.sort_key;
    let ascending = state.sort_asc;
    rows.sort_by(|a, b| {
        let order = match key {
            SortKey::City => a.label.cmp(&b.label),
            SortKey::Ownership => a.ownership_label.cmp(&b.ownership_label),
            SortKey::Size => a.size_label.cmp(&b.size_label),
            SortKey::Slots => a.occupied.cmp(&b.occupied),
            SortKey::Drivers => a.drivers.cmp(&b.drivers),
        };
        if ascending { order } else { order.reverse() }
    });
    rows
}

fn garage_table_rows(rows: &[GarageRow]) -> Vec<Vec<String>> {
    rows.iter()
        .map(|row| {
            vec![
                if row.hq {
                    format!("{} ⭐ (HQ)", row.label)
                } else {
                    row.label.clone()
                },
                row.country.clone(),
                row.ownership_label.clone(),
                row.size_label.clone(),
                row.slots.clone(),
                format!("{} drv / {} trk", row.drivers, row.trucks),
            ]
        })
        .collect()
}

/// Shared, selectable garage table with filter and sorting controls.
fn render_garage_table(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
    navigate_on_select: bool,
) {
    let Some(data) = state.garage_data.clone() else {
        comp::empty_state(
            ui,
            "No garages found for this save",
            "Use Load garages & drivers to inspect the active save.",
        );
        return;
    };
    comp::section_card(ui, "Garage overview", |ui| {
        ui.horizontal_wrapped(|ui| {
            comp::compact_stat_tile(ui, "Owned", &data.owned_count.to_string(), theme::SUCCESS);
            comp::compact_stat_tile(ui, "Unowned", &data.unowned_count.to_string(), theme::MUTED);
            comp::compact_stat_tile(ui, "Headquarters", &data.hq_label, theme::ACCENT);
            comp::compact_stat_tile(
                ui,
                "AI drivers",
                &state
                    .driver_data
                    .as_ref()
                    .map(|drivers| drivers.rows.len().to_string())
                    .unwrap_or_else(|| "-".to_string()),
                theme::ACCENT,
            );
        });
    });

    ui.horizontal_wrapped(|ui| {
        ui.label("Search:");
        comp::text_input(ui, &mut state.filter_text, "city or garage id");
        comp::toggle(ui, "garages_owned_only", "Owned only", &mut state.only_owned);
    });
    ui.horizontal_wrapped(|ui| {
        sort_header(state, ui, "Garage / City", SortKey::City);
        sort_header(state, ui, "Ownership", SortKey::Ownership);
        sort_header(state, ui, "Size", SortKey::Size);
        sort_header(state, ui, "Slots", SortKey::Slots);
        sort_header(state, ui, "Drivers", SortKey::Drivers);
    });

    let visible = visible_rows(state);
    if visible.is_empty() {
        comp::empty_state(
            ui,
            "No garages match",
            "Try another search or turn off the Owned-only filter.",
        );
        return;
    }
    let selected = state
        .selected_garage
        .as_deref()
        .and_then(|id| visible.iter().position(|row| row.id == id));
    let table_rows = garage_table_rows(&visible);
    if let Some(index) = comp::table(
        ui,
        "garages_table",
        GARAGE_HEADERS,
        &table_rows,
        selected,
    ) {
        state.selected_garage = Some(visible[index].id.clone());
        if navigate_on_select {
            navigate(Route::operations_garages, SubView::GarageDetail, ctx);
        }
    }

    for row in visible.iter().filter(|row| !row.warnings.is_empty()) {
        for warning in &row.warnings {
            comp::validation_message(ui, &format!("{}: {}", row.label, warning));
        }
    }
}

fn render_garage_list(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    render_garage_table(ui, state, ctx, true);
    let Some(data) = state.garage_data.clone() else {
        return;
    };
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        let sellable = data.rows.iter().filter(|row| row.sellable).count();
        if comp::button(ui, &format!("Buy all garages ({})", data.unowned_count)).clicked() {
            if require(
                data.unowned_count > 0,
                "There are no unowned garages to buy.".to_string(),
                ctx,
            ) {
                state.ask_confirm(
                    "Buy all garages",
                    format!(
                        "Buy all {} unowned garages? This spends a large amount of money.",
                        data.unowned_count
                    ),
                    "Buy all",
                    Operation::BuyAll {
                        hash: data.hash.clone(),
                    },
                );
            }
        }
        if comp::button(ui, &format!("Sell empty garages ({sellable})")).clicked() {
            if require(
                sellable > 0,
                "No empty, sellable garages were found.".to_string(),
                ctx,
            ) {
                let names: Vec<(String, String)> = data
                    .rows
                    .iter()
                    .map(|row| (row.id.clone(), row.label.clone()))
                    .collect();
                state.ask_confirm(
                    "Sell empty garages",
                    format!(
                        "Sell all {sellable} empty garages? The headquarters and garages with drivers, trucks, trailers or trailer slots are skipped."
                    ),
                    "Sell all",
                    Operation::SellEmpty {
                        hash: data.hash.clone(),
                        names,
                    },
                );
            }
        }
        if comp::button(ui, "AI drivers").clicked() {
            navigate(
                Route::operations_ai_drivers,
                SubView::DriverGenerate,
                ctx,
            );
        }
        if comp::button(ui, "All warnings").clicked() {
            navigate(Route::operations_garages, SubView::GarageWarnings, ctx);
        }
    });
}

fn render_garage_detail(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    render_garage_table(ui, state, ctx, false);
    let Some(data) = state.garage_data.clone() else {
        return;
    };
    let Some(id) = state.selected_garage.clone() else {
        comp::empty_state(
            ui,
            "Select a garage",
            "Pick a row in the garage table to inspect and edit it.",
        );
        return;
    };
    let Some(row) = data.rows.iter().find(|row| row.id == id).cloned() else {
        comp::validation_message(ui, "The selected garage is no longer in the save data.");
        return;
    };
    let hash = data.hash.clone();
    comp::section_card(ui, &format!("{} — garage detail", row.label), |ui| {
        egui::Grid::new("garage_detail_grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("Garage ID:");
                ui.label(&row.id);
                ui.end_row();
                ui.label("Ownership:");
                ui.label(&row.ownership_label);
                ui.end_row();
                ui.label("Size:");
                ui.label(&row.size_label);
                ui.end_row();
                ui.label("Slots:");
                ui.label(&row.slots);
                ui.end_row();
                ui.label("Drivers / Trucks:");
                ui.label(format!("{} / {}", row.drivers, row.trucks));
                ui.end_row();
                ui.label("Headquarters:");
                ui.label(if row.hq { "Yes" } else { "No" });
                ui.end_row();
            });
        for warning in &row.warnings {
            comp::validation_message(ui, &format!("Warning: {warning}"));
        }
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if !row.owned && comp::button(ui, "Buy garage").clicked() {
                state.ask_confirm(
                    "Buy garage",
                    format!(
                        "Buy {} (expected save hash {})? This spends money and stages an edit in the save.",
                        row.label, hash
                    ),
                    "Buy",
                    Operation::Purchase {
                        id: row.id.clone(),
                        hash: hash.clone(),
                        label: row.label.clone(),
                    },
                );
            }
            if row.owned {
                if comp::button(ui, "Upgrade").clicked() {
                    state.ask_confirm(
                        "Upgrade garage",
                        format!(
                            "Upgrade {} (expected save hash {})? This spends money and stages an edit in the save.",
                            row.label, hash
                        ),
                        "Upgrade",
                        Operation::Upgrade {
                            id: row.id.clone(),
                            hash: hash.clone(),
                            label: row.label.clone(),
                        },
                    );
                }
                if comp::button(ui, "Assign driver + truck").clicked() {
                    state.ask_confirm(
                        "Assign driver + truck",
                        format!(
                            "Assign a random driver and truck to {} (expected save hash {})? This stages an edit in the save.",
                            row.label, hash
                        ),
                        "Assign",
                        Operation::AssignRandom {
                            id: row.id.clone(),
                            hash: hash.clone(),
                            label: row.label.clone(),
                        },
                    );
                }
                if !row.hq && comp::button(ui, "Set as HQ").clicked() {
                    state.ask_confirm(
                        "Set headquarters",
                        format!(
                            "Set {} (expected save hash {}) as headquarters? This stages an edit in the save.",
                            row.label, hash
                        ),
                        "Set HQ",
                        Operation::SetHeadquarters {
                            id: row.id.clone(),
                            hash: hash.clone(),
                            label: row.label.clone(),
                        },
                    );
                }
            }
            if !row.sellable {
                ui.label(
                    egui::RichText::new(
                        "Sell unavailable: only an empty, non-HQ garage can be sold.",
                    )
                    .small()
                    .color(theme::MUTED),
                );
            }
            if comp::danger_button(ui, "Sell garage").clicked() {
                if !row.sellable {
                    validation(
                        ctx,
                        "Only an empty, non-headquarters garage can be sold.".to_string(),
                    );
                } else {
                    state.ask_confirm(
                        "Sell garage",
                        format!(
                            "Sell {}? Only an empty garage can be sold — the headquarters and garages with drivers, trucks or trailers are refused. You can buy it back later.",
                            row.label
                        ),
                        "Sell",
                        Operation::Sell {
                            id: row.id.clone(),
                            hash: hash.clone(),
                            label: row.label.clone(),
                        },
                    );
                }
            }
        });
    });
}

fn render_warnings(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let Some(data) = state.garage_data.clone() else {
        comp::empty_state(ui, "No garages found for this save", "Load garages to see warnings.");
        return;
    };
    let driver_warnings = state
        .driver_data
        .as_ref()
        .map(|drivers| drivers.warnings.clone())
        .unwrap_or_default();
    let mut warnings = data.warnings.clone();
    warnings.extend(driver_warnings);
    comp::section_card(ui, "Save parse warnings", |ui| {
        if warnings.is_empty() {
            ui.label("No list-level or driver-level warnings were reported.");
        }
        for warning in &warnings {
            comp::validation_message(ui, warning);
        }
    });
    let warned: Vec<GarageRow> = data
        .rows
        .iter()
        .filter(|row| !row.warnings.is_empty())
        .cloned()
        .collect();
    if warned.is_empty() {
        comp::empty_state(ui, "No per-garage warnings", "Every garage parsed cleanly.");
        return;
    }
    for row in warned {
        let id = row.id.clone();
        comp::section_card(ui, &row.label, |ui| {
            for warning in &row.warnings {
                comp::validation_message(ui, warning);
            }
            if comp::button(ui, "Open garage detail").clicked() {
                state.selected_garage = Some(id.clone());
                navigate(Route::operations_garages, SubView::GarageDetail, ctx);
            }
        });
    }
}

fn render_driver_generate(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let Some(data) = state.driver_data.clone() else {
        comp::empty_state(
            ui,
            "No AI driver data loaded",
            "Use Load garages & drivers to inspect the active save.",
        );
        return;
    };
    comp::section_card(ui, "Generate AI drivers", |ui| {
        let options: Vec<String> = state
            .garage_data
            .as_ref()
            .map(|garages| {
                garages
                    .rows
                    .iter()
                    .filter(|row| row.owned)
                    .map(|row| row.id.clone())
                    .collect()
            })
            .unwrap_or_default();
        comp::select(
            ui,
            "driver_target_garage",
            &mut state.driver_garage_input,
            &options,
        );
        ui.add(egui::DragValue::new(&mut state.generate_count).range(1..=5));
        comp::toggle(
            ui,
            "driver_assign_trucks",
            "Also assign trucks",
            &mut state.assign_trucks,
        );
        ui.label(format!(
            "Available pool: {} driver(s), {} truck(s)",
            data.available_count,
            data.trucks.len()
        ));
        if comp::button(ui, "Generate AI drivers").clicked() {
            let garage_id = state.driver_garage_input.trim().to_string();
            let valid = require(
                !garage_id.is_empty() && options.iter().any(|id| id == &garage_id),
                "Select an owned garage before generating drivers.".to_string(),
                ctx,
            ) && require(
                (1..=5).contains(&state.generate_count),
                "Driver count must be between 1 and 5.".to_string(),
                ctx,
            );
            if valid {
                state.schedule(
                    Operation::Generate {
                        garage_id,
                        hash: data.hash.clone(),
                        count: state.generate_count,
                        assign_trucks: state.assign_trucks,
                    },
                    "Generating AI drivers…",
                );
            }
        }
    });
    for warning in &data.warnings {
        comp::validation_message(ui, warning);
    }
    ui.add_space(8.0);
    if data.rows.is_empty() {
        comp::empty_state(
            ui,
            "No AI drivers — generate some",
            "Generate drivers for an owned garage to put them to work.",
        );
        return;
    }
    let selected = data
        .rows
        .iter()
        .position(|driver| driver.id.as_str() == state.selected_driver.as_str());
    let rows: Vec<Vec<String>> = data
        .rows
        .iter()
        .map(|driver| {
            vec![
                driver.name.clone(),
                format!("Lv {} ({} XP)", driver.level, driver.xp),
                driver.garage.clone(),
                driver.truck.clone(),
                driver.state.clone(),
            ]
        })
        .collect();
    if let Some(index) = comp::table(ui, "drivers_table", DRIVER_HEADERS, &rows, selected) {
        state.selected_driver = data.rows[index].id.clone();
    }
    ui.horizontal_wrapped(|ui| {
        if comp::button(ui, "Assign / reassign truck").clicked() {
            navigate(
                Route::operations_ai_drivers,
                SubView::DriverAssign,
                ctx,
            );
        }
        if comp::danger_button(ui, "Dismiss selected driver").clicked() {
            let id = state.selected_driver.clone();
            let valid = require(
                !id.is_empty(),
                "Select a driver to dismiss.".to_string(),
                ctx,
            );
            if valid {
                let name = data
                    .rows
                    .iter()
                    .find(|driver| driver.id == id)
                    .map(|driver| driver.name.clone())
                    .unwrap_or_else(|| id.clone());
                state.ask_confirm(
                    "Dismiss driver",
                    format!("Dismiss {name}? This removes the driver from your company."),
                    "Dismiss",
                    Operation::Dismiss {
                        ids: vec![id],
                        hash: data.hash.clone(),
                        labels: vec![name],
                    },
                );
            }
        }
        if comp::button(ui, "Dismiss many").clicked() {
            navigate(
                Route::operations_ai_drivers,
                SubView::DriverDismiss,
                ctx,
            );
        }
    });
    for driver in data.rows.iter().filter(|driver| !driver.warnings.is_empty()) {
        for warning in &driver.warnings {
            comp::validation_message(ui, &format!("{}: {}", driver.name, warning));
        }
    }
}

fn render_driver_assign(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let Some(data) = state.driver_data.clone() else {
        comp::empty_state(
            ui,
            "No AI driver data loaded",
            "Use Load garages & drivers to inspect the active save.",
        );
        return;
    };
    if data.rows.is_empty() {
        comp::empty_state(
            ui,
            "No AI drivers — generate some",
            "Assignments become available after drivers are generated.",
        );
        return;
    }
    if state.selected_driver.is_empty()
        || !data.rows.iter().any(|driver| driver.id.as_str() == state.selected_driver.as_str())
    {
        state.selected_driver = data.rows[0].id.clone();
    }
    let driver = data
        .rows
        .iter()
        .find(|driver| driver.id.as_str() == state.selected_driver.as_str())
        .cloned()
        .expect("selected driver is in the loaded list");
    let mut truck_options: Vec<String> = data
        .trucks
        .iter()
        .map(|truck| truck.id.clone())
        .collect();
    truck_options.insert(0, UNASSIGNED.to_string());
    if state.selected_truck.is_empty() || !truck_options.contains(&state.selected_truck) {
        state.selected_truck = UNASSIGNED.to_string();
    }

    comp::section_card(ui, "Assign / reassign a driver's truck", |ui| {
        let driver_ids: Vec<String> = data.rows.iter().map(|driver| driver.id.clone()).collect();
        comp::select(ui, "assign_driver", &mut state.selected_driver, &driver_ids);
        ui.label(format!(
            "{} — currently assigned to {} at {}",
            driver.name, driver.truck, driver.garage
        ));
        comp::select(ui, "assign_truck", &mut state.selected_truck, &truck_options);
        if data.trucks.is_empty() {
            comp::validation_message(
                ui,
                "No unassigned trucks are available; generate drivers with truck assignment or free a truck first.",
            );
        }
        ui.horizontal_wrapped(|ui| {
            if comp::button(ui, "Assign selected truck").clicked() {
                let truck = state.selected_truck.clone();
                if require(
                    truck != UNASSIGNED,
                    "Select a truck before assigning.".to_string(),
                    ctx,
                ) {
                    state.schedule(
                        Operation::AssignTruck {
                            driver_id: driver.id.clone(),
                            hash: data.hash.clone(),
                            truck_id: Some(truck),
                            label: driver.name.clone(),
                        },
                        "Assigning truck…",
                    );
                }
            }
            if comp::button(ui, "Unassign truck").clicked() {
                state.schedule(
                    Operation::AssignTruck {
                        driver_id: driver.id.clone(),
                        hash: data.hash.clone(),
                        truck_id: None,
                        label: driver.name.clone(),
                    },
                    "Unassigning truck…",
                );
            }
            if comp::button(ui, "Reassign random truck at garage").clicked() {
                state.schedule(
                    Operation::ReassignRandom {
                        garage_id: driver.garage_id.clone(),
                        hash: data.hash.clone(),
                        label: driver.garage.clone(),
                    },
                    "Reassigning random truck…",
                );
            }
        });
    });
    for truck in &data.trucks {
        if !truck.label.is_empty() && truck.id.as_str() == state.selected_truck.as_str() {
            ui.label(
                egui::RichText::new(format!("Selected truck: {}", truck.label))
                    .small()
                    .color(theme::MUTED),
            );
        }
    }
    for warning in &driver.warnings {
        comp::validation_message(ui, &format!("{}: {}", driver.name, warning));
    }
}

fn render_driver_dismiss(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let Some(data) = state.driver_data.clone() else {
        comp::empty_state(
            ui,
            "No AI driver data loaded",
            "Use Load garages & drivers to inspect the active save.",
        );
        return;
    };
    if data.rows.is_empty() {
        comp::empty_state(
            ui,
            "No AI drivers — generate some",
            "There are no drivers to dismiss in this save.",
        );
        return;
    }
    ui.horizontal_wrapped(|ui| {
        if comp::danger_button(ui, "Dismiss selected drivers").clicked() {
            let ids = state.dismiss_selection.clone();
            let valid = require(
                !ids.is_empty(),
                "Select at least one driver to dismiss.".to_string(),
                ctx,
            );
            if valid {
                let labels: Vec<String> = data
                    .rows
                    .iter()
                    .filter(|driver| ids.contains(&driver.id))
                    .map(|driver| driver.name.clone())
                    .collect();
                state.ask_confirm(
                    "Dismiss drivers",
                    format!(
                        "Dismiss {}? This permanently removes the selected drivers from your company.",
                        labels.join(", ")
                    ),
                    "Dismiss",
                    Operation::Dismiss {
                        ids,
                        hash: data.hash.clone(),
                        labels,
                    },
                );
            }
        }
        if comp::button(ui, "Generate drivers").clicked() {
            navigate(
                Route::operations_ai_drivers,
                SubView::DriverGenerate,
                ctx,
            );
        }
    });
    egui::Grid::new("dismiss_drivers_grid")
        .striped(true)
        .num_columns(6)
        .show(ui, |ui| {
            ui.strong("Select");
            ui.strong("Driver");
            ui.strong("Garage");
            ui.strong("Truck");
            ui.strong("Removable");
            ui.strong("Action");
            ui.end_row();
            for driver in &data.rows {
                let checked = state.dismiss_selection.contains(&driver.id);
                let mut checked = checked;
                if ui.checkbox(&mut checked, "").clicked() {
                    if checked {
                        state.dismiss_selection.push(driver.id.clone());
                    } else {
                        state.dismiss_selection.retain(|id| id != &driver.id);
                    }
                }
                ui.label(&driver.name);
                ui.label(&driver.garage);
                ui.label(&driver.truck);
                ui.label(if driver.removable { "Yes" } else { "No" });
                if comp::danger_button(ui, "Dismiss").clicked() {
                    if require(
                        driver.removable,
                        "The backend refuses to dismiss this driver.".to_string(),
                        ctx,
                    ) {
                        state.ask_confirm(
                            "Dismiss driver",
                            format!("Dismiss {}? This removes the driver from your company.", driver.name),
                            "Dismiss",
                            Operation::Dismiss {
                                ids: vec![driver.id.clone()],
                                hash: data.hash.clone(),
                                labels: vec![driver.name.clone()],
                            },
                        );
                    }
                }
                ui.end_row();
                for warning in &driver.warnings {
                    comp::validation_message(ui, &format!("{}: {}", driver.name, warning));
                    ui.end_row();
                }
            }
        });
}

/// Shared destructive-action confirmation, rendered once per frame.
fn render_confirm(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let Some(confirmation) = state.confirm.clone() else {
        return;
    };
    let mut message = confirmation.message.clone();
    if crate::gui::data::session::is_dirty(ctx.data) {
        message.push_str("\n\nStaged edits are still pending; this action may overwrite them.");
    }
    match comp::confirm_dialog(
        ui.ctx(),
        confirmation.title,
        &message,
        confirmation.confirm_label,
        "Cancel",
    ) {
        Some(true) => {
            state.confirm = None;
            state.schedule(confirmation.action, "Applying confirmed action…");
        }
        Some(false) => {
            state.confirm = None;
            info(ctx, "Action cancelled.".to_string());
        }
        None => {}
    }
}
