//! Overview sector screen: Dashboard, Quick Actions and Recent Activity.
//!
//! This screen is a read-only hub plus a few staged mutations. It never owns
//! navigation state: every deep link is a [`ShellCommand::Navigate`].

use eframe::egui::{self, Color32};

use crate::gui::components as comp;
use crate::gui::screens::{
    ALL_SECTORS, IssuesTab, Route, ScreenCtx, Sector, ShellCommand, SubView,
};
use crate::gui::theme;

// ------------------------------------------------- shell-owned view helpers

fn normalized_sub_view(route: Route, view: SubView) -> SubView {
    match route {
        Route::overview_dashboard => match view {
            SubView::DashboardHeader
            | SubView::DashboardCurrentSetup
            | SubView::DashboardStatTiles
            | SubView::DashboardSectorCards
            | SubView::DashboardStagingNote => view,
            _ => SubView::DashboardHeader,
        },
        Route::overview_quick_actions => SubView::QuickActionsList,
        Route::overview_recent_activity => SubView::ActivityFeed,
        _ => SubView::DashboardHeader,
    }
}

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

fn navigate(route: Route, sub_view: SubView, ctx: &mut ScreenCtx<'_>) {
    ctx.commands.push(ShellCommand::Navigate {
        route,
        sub_view: Some(sub_view),
    });
}

fn toast(ctx: &mut ScreenCtx<'_>, message: String, is_error: bool) {
    ctx.commands.push(ShellCommand::Toast { message, is_error });
}

fn error(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, message, true);
}

fn info(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, message, false);
}

fn validation(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, format!("Validation: {message}"), false);
}

// ------------------------------------------------------------ local DTO views

#[derive(Debug, Clone)]
struct TileView {
    label: String,
    value: String,
    route: Route,
    sub_view: SubView,
    color: Color32,
}

#[derive(Debug, Clone, Default)]
struct HeaderView {
    lines: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct SetupView {
    lines: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct FactsView {
    tiles: Vec<TileView>,
    setup: Option<SetupView>,
}

#[derive(Debug, Clone)]
struct ActivityRow {
    label: String,
    detail: String,
}

#[derive(Debug, Clone)]
enum Operation {
    RefreshHeader,
    RefreshFacts,
    RefreshActivity,
    RefreshAll,
    RepairRefuel,
    MaxSkills,
}

#[derive(Debug, Clone)]
struct Confirm {
    title: &'static str,
    message: String,
    action: Operation,
}

/// All local UI state. Navigation swaps this instance in and out but never
/// clears it (docs/ui-spec.md §1.3 rule 5).
#[derive(Default)]
pub struct ScreenState {
    header_tried: bool,
    header: Option<HeaderView>,
    header_error: Option<String>,

    facts_tried: bool,
    facts: Option<FactsView>,
    facts_error: Option<String>,

    activity_tried: bool,
    activity: Vec<ActivityRow>,
    activity_error: Option<String>,

    action_loading: Option<&'static str>,
    action_message: Option<String>,
    action_error: Option<String>,
    action_validation: Option<String>,
    last_action: Option<Operation>,

    pending: Option<Operation>,
    confirm: Option<Confirm>,
    loading: Option<&'static str>,
}

impl ScreenState {
    fn schedule(&mut self, operation: Operation, message: &'static str) {
        self.pending = Some(operation);
        self.loading = Some(message);
        self.last_action = Some(operation);
    }

    fn confirm(&mut self, title: &'static str, message: String, action: Operation) {
        self.confirm = Some(Confirm {
            title,
            message,
            action,
        });
    }
}

// ------------------------------------------------------------------ rendering

/// Exact screen contract: one state instance and one render function.
pub fn render(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    if let Some(operation) = state.pending.take() {
        execute(operation, state, ctx);
    }

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if comp::button(ui, "Refresh dashboard").clicked() {
            state.schedule(Operation::RefreshAll, "Refreshing dashboard…");
        }
        if comp::button(ui, "Errors & Logs").clicked() {
            ctx.commands
                .push(ShellCommand::OpenIssues(IssuesTab::Errors));
        }
        ui.label(
            egui::RichText::new("Read-only overview · edits stage until Save in the Safety Spine.")
                .size(10.5)
                .color(theme::MUTED),
        );
    });
    ui.add_space(4.0);

    if !state.header_tried && !state.facts_tried && state.pending.is_none() {
        state.schedule(Operation::RefreshAll, "Loading dashboard…");
    }
    if ctx.recent_events.is_empty() && !state.activity_tried && state.pending.is_none() {
        state.schedule(Operation::RefreshActivity, "Reading recent activity…");
    }
    if let Some(message) = state.loading {
        comp::loading_state(ui, message);
    }

    let view = normalized_sub_view(ctx.route, ctx.sub_view);
    match ctx.route {
        Route::overview_dashboard => render_dashboard(ui, state, ctx, view),
        Route::overview_quick_actions => render_quick_actions(ui, state, ctx),
        Route::overview_recent_activity => render_activity(ui, state, ctx),
        _ => render_dashboard(ui, state, ctx, SubView::DashboardHeader),
    }

    render_confirm(ui, state, ctx);

    if state.pending.is_none() {
        state.loading = None;
    }
}

fn render_dashboard(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
    view: SubView,
) {
    sub_view_nav(
        ui,
        ctx,
        Route::overview_dashboard,
        view,
        &[
            (SubView::DashboardHeader, "Header"),
            (SubView::DashboardCurrentSetup, "Current Setup"),
            (SubView::DashboardStatTiles, "Stats"),
            (SubView::DashboardSectorCards, "Sectors"),
            (SubView::DashboardStagingNote, "Staging"),
        ],
    );

    render_header(ui, state, ctx);
    ui.add_space(8.0);
    render_stat_tiles(ui, state, ctx);
    ui.add_space(8.0);
    render_current_setup(ui, state, ctx);
    ui.add_space(8.0);
    render_sector_cards(ui, ctx);
    ui.add_space(8.0);
    render_staging_note(ui, ctx);
}

fn render_header(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Active profile & save", |ui| {
        if let Some(message) = state.header_error.clone() {
            if comp::error_state(ui, message, Some("Retry")) {
                state.schedule(Operation::RefreshHeader, "Reading save context…");
            }
            if comp::button(ui, "View error in Issues").clicked() {
                ctx.commands
                    .push(ShellCommand::OpenIssues(IssuesTab::Errors));
            }
            return;
        }
        if !state.header_tried {
            return;
        }
        let Some(header) = state.header.clone() else {
            comp::empty_state(
                ui,
                "No save selected",
                "Choose Game, Profile and Save in the Context Column.",
            );
            return;
        };
        for line in &header.lines {
            ui.label(line.as_str());
        }
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(
                "Live save facts and the active save selection are refreshed by the shell.",
            )
            .size(10.5)
            .color(theme::MUTED),
        );
    });
}

fn render_stat_tiles(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "At a glance", |ui| {
        if let Some(message) = state.facts_error.clone() {
            if comp::error_state(ui, message, Some("Retry")) {
                state.schedule(Operation::RefreshFacts, "Reading save facts…");
            }
            if comp::button(ui, "View error in Issues").clicked() {
                ctx.commands
                    .push(ShellCommand::OpenIssues(IssuesTab::Errors));
            }
            return;
        }
        if !state.facts_tried {
            return;
        }
        let Some(facts) = state.facts.clone() else {
            comp::empty_state(
                ui,
                "No save data loaded",
                "Select an active save to populate the dashboard tiles.",
            );
            return;
        };
        ui.horizontal_wrapped(|ui| {
            for tile in &facts.tiles {
                let response = comp::compact_stat_tile(ui, &tile.label, &tile.value, tile.color);
                let click = ui.interact(
                    response.rect,
                    egui::Id::new(("dashboard_stat_tile", tile.label.as_str())),
                    egui::Sense::click(),
                );
                if click.clicked() {
                    navigate(tile.route, tile.sub_view, ctx);
                }
                click.on_hover_cursor(egui::CursorIcon::PointingHand);
            }
        });
    });
}

fn render_current_setup(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Current Setup", |ui| {
        if let Some(message) = state.facts_error.clone() {
            if comp::error_state(ui, message, Some("Retry")) {
                state.schedule(Operation::RefreshFacts, "Reading save facts…");
            }
            if comp::button(ui, "View error in Issues").clicked() {
                ctx.commands
                    .push(ShellCommand::OpenIssues(IssuesTab::Errors));
            }
            return;
        }
        if !state.facts_tried {
            return;
        }
        let Some(facts) = state.facts.clone() else {
            comp::empty_state(
                ui,
                "No current setup",
                "Truck, trailer and headquarters appear once a save is loaded.",
            );
            return;
        };
        let Some(setup) = facts.setup else {
            comp::empty_state(
                ui,
                "No current setup",
                "Truck, trailer and headquarters appear once a save is loaded.",
            );
            return;
        };
        for line in &setup.lines {
            ui.label(line.as_str());
        }
    });
}

fn sector_description(sector: Sector) -> &'static str {
    match sector {
        Sector::Overview => "Dashboard, quick actions and recent activity.",
        Sector::SaveData => "Money, XP, skills, coaches and configuration keys.",
        Sector::Vehicles => "Truck and trailer editing, switchers and fleet inventory.",
        Sector::Operations => "Garages, drivers, telemetry, dispatcher and logbook.",
        Sector::Library => "Mods, presets, sandboxes and conflict diagnostics.",
        Sector::Safety => "Backups, save health, clone, rename and profile sharing.",
        Sector::System => "Paths, logs, diagnostics and application information.",
    }
}

fn render_sector_cards(ui: &mut egui::Ui, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Open a screen", |ui| {
        for sector in ALL_SECTORS {
            if sector == Sector::Overview {
                continue;
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(sector.label()).strong());
                    ui.label(
                        egui::RichText::new(sector_description(sector))
                            .size(11.0)
                            .color(theme::MUTED),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if comp::button(ui, "Open").clicked() {
                        navigate(sector.default_route(), SubView::None, ctx);
                    }
                });
            });
        }
    });
}

fn render_staging_note(ui: &mut egui::Ui, ctx: &mut ScreenCtx<'_>) {
    let staged = crate::gui::data::session::dirty_paths(ctx.data).len();
    let note = if staged == 0 {
        "No staged edits. Every save creates a restorable backup.".to_string()
    } else {
        format!("{staged} staged edit(s). Press Save in the Safety Spine; a backup is created first.")
    };
    let color = if staged == 0 { theme::MUTED } else { theme::WARNING };
    comp::section_card(ui, "Staging", |ui| {
        ui.label(egui::RichText::new(note).size(11.5).color(color));
    });
}

fn render_quick_actions(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Quick actions", |ui| {
        if let Some(message) = state.action_error.clone() {
            if comp::error_state(ui, message, Some("Retry")) {
                match state.last_action.clone() {
                    Some(Operation::MaxSkills) => state.confirm(
                        "Max all skills",
                        "Stage ADR 63 and the other five skills at 6?".to_string(),
                        Operation::MaxSkills,
                    ),
                    Some(action) => state.schedule(action, "Retrying quick action…"),
                    None => state.schedule(Operation::RepairRefuel, "Retrying quick action…"),
                }
            }
            if comp::button(ui, "View error in Issues").clicked() {
                ctx.commands
                    .push(ShellCommand::OpenIssues(IssuesTab::Errors));
            }
        }
        if let Some(message) = state.action_loading {
            comp::loading_state(ui, message);
        }
        ui.horizontal_wrapped(|ui| {
            if comp::button(ui, "Repair + Refuel Truck").clicked() {
                state.schedule(
                    Operation::RepairRefuel,
                    "Repairing and refuelling the active truck…",
                );
            }
            if comp::button(ui, "Max All Skills…").clicked() {
                state.confirm(
                    "Max all skills",
                    "Stage ADR at 63 and long distance, heavy, fragile, urgent and mechanical at 6?".to_string(),
                    Operation::MaxSkills,
                );
            }
            if comp::button(ui, "Scan Save Health →").clicked() {
                navigate(
                    Route::safety_save_health,
                    SubView::HealthScan,
                    ctx,
                );
            }
            if comp::button(ui, "Generate Market Jobs →").clicked() {
                navigate(
                    Route::operations_dispatcher,
                    SubView::DispatcherGeneration,
                    ctx,
                );
            }
        });
        if let Some(message) = &state.action_validation {
            comp::validation_message(ui, message);
        }
        if let Some(message) = &state.action_message {
            ui.label(message.as_str());
        } else if state.action_error.is_none() {
            comp::empty_state(
                ui,
                "No quick action run yet",
                "Repair/refuel and Max All Skills stage changes; the other two open their screens.",
            );
        }
        ui.label(
            egui::RichText::new("Staged changes are written only when you press Save in the Safety Spine.")
                .size(10.5)
                .color(theme::MUTED),
        );
    });
}

fn render_activity(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let shell_rows: Vec<Vec<String>> = ctx
        .recent_events
        .iter()
        .rev()
        .take(5)
        .map(|(event, payload)| vec![event.clone(), payload.clone()])
        .collect();
    let using_shell_events = !shell_rows.is_empty();
    comp::section_card(ui, "Recent activity", |ui| {
        if comp::button(ui, "Open full activity").clicked() {
            ctx.commands
                .push(ShellCommand::OpenIssues(IssuesTab::Activity));
        }
        if using_shell_events {
            comp::table(
                ui,
                "overview_recent_activity",
                &["Event", "Payload"],
                &shell_rows,
                None,
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Live shell events (last five).")
                    .size(10.5)
                    .color(theme::MUTED),
            );
            return;
        }
        if let Some(message) = state.activity_error.clone() {
            if comp::error_state(ui, message, Some("Retry")) {
                state.schedule(Operation::RefreshActivity, "Reading recent activity…");
            }
            if comp::button(ui, "View error in Issues").clicked() {
                ctx.commands
                    .push(ShellCommand::OpenIssues(IssuesTab::Errors));
            }
            return;
        }
        if !state.activity_tried {
            return;
        }
        if state.activity.is_empty() {
            comp::empty_state(
                ui,
                "No activity recorded",
                "No shell events or user-log entries yet.",
            );
            return;
        }
        let rows: Vec<Vec<String>> = state
            .activity
            .iter()
            .map(|row| vec![row.label.clone(), row.detail.clone()])
            .collect();
        comp::table(
            ui,
            "overview_recent_activity",
            &["Entry", "Detail"],
            &rows,
            None,
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(
                "The shell event feed is empty; showing the last five persisted user-log entries.",
            )
            .size(10.5)
            .color(theme::MUTED),
        );
    });
}

// ----------------------------------------------------------- operation runner

/// Resolve the active profile path from the shell's selected profile, then
/// the last-profile marker and finally the first discovered profile. Errors
/// from the marker are surfaced, never rendered as a false empty view.
fn active_profile_path(ctx: &mut ScreenCtx<'_>) -> Result<Option<String>, String> {
    if let Ok(path) = crate::gui::data::context::selected_profile_path(ctx.data) {
        if !path.is_empty() {
            return Ok(Some(path));
        }
    }
    let profiles = crate::gui::data::context::find_profiles(ctx.data);
    match crate::gui::data::context::read_last_profile(ctx.data) {
        Ok(Some(path)) => Ok(Some(path)),
        Ok(None) => Ok(profiles.first().map(|profile| profile.path.clone())),
        Err(err) => match profiles.first() {
            Some(profile) => {
                error(ctx, err.message);
                Ok(Some(profile.path.clone()))
            }
            None => Err(err.message),
        },
    }
}

fn execute(operation: Operation, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match operation {
        Operation::RefreshAll => {
            refresh_header(state, ctx);
            refresh_facts(state, ctx);
        }
        Operation::RefreshHeader => refresh_header(state, ctx),
        Operation::RefreshFacts => refresh_facts(state, ctx),
        Operation::RefreshActivity => refresh_activity(state, ctx),
        Operation::RepairRefuel => {
            state.action_error = None;
            state.action_validation = None;
            let repaired = crate::gui::data::quick_actions::repair_truck(ctx.data);
            let refuelled = crate::gui::data::quick_actions::refuel_truck(ctx.data);
            match (repaired, refuelled) {
                (Ok(()), Ok(())) => {
                    let message = "Truck repaired and refuelled. Changes are staged.".to_string();
                    state.action_message = Some(message.clone());
                    info(ctx, message);
                }
                (repair, refuel) => {
                    let mut failures = Vec::new();
                    if let Err(err) = repair {
                        failures.push(format!("Repair: {}", err.message));
                    }
                    if let Err(err) = refuel {
                        failures.push(format!("Refuel: {}", err.message));
                    }
                    let message = failures.join("; ");
                    state.action_error = Some(message.clone());
                    error(ctx, message);
                }
            }
        }
        Operation::MaxSkills => {
            state.action_error = None;
            state.action_validation = None;
            match crate::gui::data::quick_actions::max_all_skills(ctx.data) {
                Ok(()) => {
                    let message =
                        "All six skills staged at maximum (ADR 63, others 6).".to_string();
                    state.action_message = Some(message.clone());
                    info(ctx, message);
                }
                Err(err) => {
                    let message = format!("Max All Skills failed: {}", err.message);
                    state.action_error = Some(message.clone());
                    error(ctx, message);
                }
            }
        }
    }
}

fn refresh_header(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match active_profile_path(ctx) {
        Err(message) => {
            state.header_error = Some(message.clone());
            error(ctx, message);
        }
        Ok(None) => {
            state.header = None;
            state.header_error = None;
        }
        Ok(Some(path)) => {
            let profiles = crate::gui::data::context::find_profiles(ctx.data);
            let game = match crate::gui::data::context::selected_game(ctx.data) {
                Ok(value) if !value.is_empty() => value,
                _ => "—".to_string(),
            };
            let active_save_label = ctx.active_save_label.trim().to_string();
            let profile_name = profiles
                .iter()
                .find(|profile| profile.path == path)
                .and_then(|profile| profile.name.clone())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| path.clone());
            match crate::gui::data::context::find_saves(ctx.data, path) {
                Err(err) => {
                    state.header_error = Some(err.message.clone());
                    error(ctx, err.message);
                }
                Ok(saves) => {
                    let save_label = if active_save_label.is_empty() {
                        "No save selected — choose Game, Profile and Save in the Context Column"
                            .to_string()
                    } else {
                        active_save_label
                    };
                    state.header = Some(HeaderView {
                        lines: vec![
                            format!("Game · {game}"),
                            format!("Profile · {profile_name}"),
                            format!("Save slot · {save_label}"),
                            format!("Profiles: {} · Saves: {}", profiles.len(), saves.len()),
                        ],
                    });
                    state.header_error = None;
                }
            }
        }
    }
    state.header_tried = true;
}

fn refresh_facts(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let Some(profile_path) = match active_profile_path(ctx) {
        Ok(path) => path,
        Err(message) => {
            state.facts_error = Some(message.clone());
            error(ctx, message);
            state.facts_tried = true;
            return;
        }
    } else {
        state.facts = None;
        state.facts_error = None;
        state.facts_tried = true;
        return;
    };

    let mut failures: Vec<String> = Vec::new();
    let save_data = crate::gui::data::context::read_all_save_data(ctx.data)
        .map_err(|err| err.message);
    let (money, xp, cities) = match save_data {
        Ok(data) => (data.money, data.xp, data.visited_cities),
        Err(message) => {
            failures.push(message);
            (None, None, None)
        }
    };
    let level = xp.map(|value| {
        crate::gui::data::context::calculate_level(ctx.data, value.max(0) as u64).level
    });

    let trucks = collect_count(
        crate::gui::data::context::all_trucks(ctx.data, profile_path.clone()),
        &mut failures,
    );
    let trailers = collect_count(
        crate::gui::data::context::all_trailers(ctx.data, profile_path.clone()),
        &mut failures,
    );
    let owned_garages = crate::gui::data::garages::list_all_garages(ctx.data)
        .map(|list| list.diagnostics.owned_garage_count.to_string())
        .map_err(|err| err.message);
    let owned_garages = collect_string(owned_garages, &mut failures);
    let drivers = crate::gui::data::garages::ai_driver_management(ctx.data)
        .map(|mgmt| mgmt.drivers.len().to_string())
        .map_err(|err| err.message);
    let drivers = collect_string(drivers, &mut failures);

    let tiles = vec![
        tile(
            "BALANCE",
            money.map(|value| format!("€{value}")),
            Route::save_data_economy,
            SubView::EconomyFields,
            theme::ACCENT,
        ),
        tile(
            "EXPERIENCE",
            xp.map(|value| format!("{value} XP")),
            Route::save_data_economy,
            SubView::EconomyFields,
            theme::TEXT,
        ),
        tile(
            "LEVEL",
            level.map(|value| value.to_string()),
            Route::save_data_economy,
            SubView::EconomyFields,
            theme::SUCCESS,
        ),
        tile(
            "VISITED CITIES",
            cities.map(|value| value.to_string()),
            Route::save_data_economy,
            SubView::EconomyFields,
            theme::TEXT,
        ),
        tile(
            "TRUCKS",
            trucks,
            Route::vehicles_fleet_inventory,
            SubView::FleetTrucks,
            theme::TEXT,
        ),
        tile(
            "TRAILERS",
            trailers,
            Route::vehicles_fleet_inventory,
            SubView::FleetTrailers,
            theme::TEXT,
        ),
        tile(
            "GARAGES OWNED",
            owned_garages,
            Route::operations_garages,
            SubView::GarageList,
            theme::TEXT,
        ),
        tile(
            "AI DRIVERS",
            drivers,
            Route::operations_ai_drivers,
            SubView::DriverGenerate,
            theme::TEXT,
        ),
    ];

    let setup = build_setup(ctx, &profile_path, &mut failures);

    if !failures.is_empty() {
        let message = failures.join("; ");
        state.facts_error = Some(message.clone());
        error(ctx, message);
    } else {
        state.facts_error = None;
    }
    state.facts = Some(FactsView {
        tiles,
        setup: Some(setup),
    });
    state.facts_tried = true;
}

fn build_setup(
    ctx: &mut ScreenCtx<'_>,
    profile_path: &str,
    failures: &mut Vec<String>,
) -> SetupView {
    let mut lines = Vec::new();
    match crate::gui::data::context::player_truck(ctx.data, profile_path.to_string()) {
        Ok(truck) => {
            let summary = match crate::gui::data::context::current_truck_summary(ctx.data) {
                Ok(value) => value,
                Err(err) => {
                    failures.push(err.message);
                    None
                }
            };
            let name = summary
                .as_ref()
                .and_then(|value| value.display_name.clone())
                .unwrap_or_else(|| format!("{} {}", truck.brand, truck.model));
            let plate = summary
                .as_ref()
                .and_then(|value| value.cleaned_plate.clone())
                .or_else(|| truck.license_plate.clone())
                .unwrap_or_else(|| "-".to_string());
            lines.push(format!(
                "Truck · {name} · Plate {plate} · {:.0} km",
                truck.odometer
            ));
            let worst_wear = truck
                .engine_wear
                .max(truck.transmission_wear)
                .max(truck.cabin_wear)
                .max(truck.chassis_wear);
            lines.push(format!(
                "Fuel {:.0}% · Worst wear {:.0}%",
                truck.fuel_relative.clamp(0.0, 1.0) * 100.0,
                worst_wear.clamp(0.0, 1.0) * 100.0
            ));
        }
        Err(err) => failures.push(err.message),
    }

    match crate::gui::data::context::player_trailer(ctx.data, profile_path.to_string()) {
        Ok(result) if result.has_trailer => {
            if let Some(trailer) = result.trailer {
                let plate = trailer
                    .display_license_plate
                    .or(trailer.license_plate)
                    .unwrap_or_else(|| "-".to_string());
                lines.push(format!(
                    "Trailer · {} · Plate {plate} · Cargo {:.0} kg",
                    trailer.body_type.as_deref().unwrap_or("Unknown"),
                    trailer.cargo_mass
                ));
            } else {
                lines.push("Trailer · Attached".to_string());
            }
        }
        Ok(_) => lines.push("Trailer · None attached".to_string()),
        Err(err) => failures.push(err.message),
    }

    match crate::gui::data::garages::list_all_garages(ctx.data) {
        Ok(list) => {
            let hq = list
                .headquarters_garage_id
                .as_deref()
                .and_then(|id| list.garages.iter().find(|garage| garage.garage_id == id))
                .and_then(|garage| {
                    garage
                        .city_name
                        .clone()
                        .or_else(|| garage.city_token.clone())
                        .or_else(|| Some(garage.garage_id.clone()))
                })
                .unwrap_or_else(|| "—".to_string());
            lines.push(format!("Headquarters · {hq}"));
        }
        Err(err) => failures.push(err.message),
    }

    if lines.is_empty() {
        lines.push("No truck, trailer or garage data available.".to_string());
    }
    SetupView { lines }
}

fn collect_count<T>(
    result: Result<Vec<T>, crate::gui::data::UiError>,
    failures: &mut Vec<String>,
) -> Option<String> {
    match result {
        Ok(items) => Some(items.len().to_string()),
        Err(err) => {
            failures.push(err.message);
            None
        }
    }
}

fn collect_string(
    result: Result<String, crate::gui::data::UiError>,
    failures: &mut Vec<String>,
) -> Option<String> {
    match result {
        Ok(value) => Some(value),
        Err(err) => {
            failures.push(err.message);
            None
        }
    }
}

fn tile(
    label: &str,
    value: Option<String>,
    route: Route,
    sub_view: SubView,
    color: Color32,
) -> TileView {
    TileView {
        label: label.to_string(),
        value: value.unwrap_or_else(|| "—".to_string()),
        route,
        sub_view,
        color,
    }
}

fn refresh_activity(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match crate::gui::data::system::user_logs_filtered(ctx.data, None, Some(5)) {
        Ok(entries) => {
            state.activity = entries
                .into_iter()
                .map(|entry| ActivityRow {
                    label: format!("{} · {}", entry.level, entry.module),
                    detail: entry.message,
                })
                .collect();
            state.activity_error = None;
        }
        Err(err) => {
            state.activity_error = Some(err.message.clone());
            error(ctx, err.message);
        }
    }
    state.activity_tried = true;
}

// ---------------------------------------------------------------- confirm dialog

fn render_confirm(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let Some(confirmation) = state.confirm.clone() else {
        return;
    };
    let mut message = confirmation.message.clone();
    if matches!(confirmation.action, Operation::MaxSkills)
        && crate::gui::data::session::is_dirty(ctx.data)
    {
        message.push_str("\n\nStaged edits are still pending; these skill values will be added to them.");
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
