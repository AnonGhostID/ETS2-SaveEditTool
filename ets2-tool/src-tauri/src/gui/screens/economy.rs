//! Save Data sector screen: Economy & Experience, Project Coaches,
//! Config Values and Custom Keys.
//!
//! Only foundation primitives and typed `gui/data/` wrappers are used.
//! Navigation, toasts, Issues and Save stay shell-owned via `ShellCommand`.
//! Every mutation stages in `save_session`; the Safety Spine owns backups,
//! Save and Undo.

use eframe::egui;

use crate::gui::components as comp;
use crate::gui::screens::{IssuesTab, Route, ScreenCtx, ShellCommand, SubView};
use crate::gui::theme;

// ------------------------------------------------------------ shell-owned view

fn normalized_sub_view(route: Route, view: SubView) -> SubView {
    match route {
        Route::save_data_economy => match view {
            SubView::EconomySkills
            | SubView::EconomyCustomReset
            | SubView::EconomyLevelPreview => view,
            _ => SubView::EconomyFields,
        },
        Route::save_data_coaches => match view {
            SubView::CoachesStats | SubView::CoachesJobHistory | SubView::CoachesStops => view,
            _ => SubView::CoachesOverview,
        },
        Route::save_data_custom_keys => match view {
            SubView::SaveConfigKeys => view,
            _ => SubView::BaseConfigKeys,
        },
        Route::save_data_config => SubView::ConfigValues,
        _ => SubView::None,
    }
}

fn navigate(route: Route, view: SubView, ctx: &mut ScreenCtx<'_>) {
    ctx.commands.push(ShellCommand::Navigate {
        route,
        sub_view: Some(view),
    });
}

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
    ui.add_space(6.0);
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

/// Validation is both inline and mirrored into the drawer's Validation tab
/// by the shell (`Validation: ` prefix, docs/components.md §1.3).
fn validation(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, format!("Validation: {message}"), false);
}

fn staging_note(ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("Edits stage until Save; backups are created by the backend.")
            .size(10.5)
            .color(theme::MUTED),
    );
}

// --------------------------------------------------------------- local views

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum StopFilter {
    #[default]
    All,
    Discovered,
    Undiscovered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigKey {
    Traffic,
    Developer,
    Console,
    Convoy,
    ParkingDoubles,
}

impl ConfigKey {
    fn label(self) -> &'static str {
        match self {
            Self::Traffic => "Traffic density (g_traffic)",
            Self::Developer => "Developer mode (g_developer)",
            Self::Console => "Developer console (g_console)",
            Self::Convoy => "Max convoy size (g_max_convoy_size)",
            Self::ParkingDoubles => "Double trailer parking (g_simple_parking_doubles)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyScope {
    Base,
    Save,
}

impl KeyScope {
    fn label(self) -> &'static str {
        match self {
            Self::Base => "Global config.cfg",
            Self::Save => "Profile config.cfg",
        }
    }
}

#[derive(Debug, Clone, Default)]
struct KeyRow {
    key: String,
    value: String,
    error: String,
}

#[derive(Debug, Clone, Copy)]
struct LevelView {
    level: u32,
    xp_into_level: u64,
    xp_needed: u64,
    percent: f32,
}

fn level_view(ctx: &ScreenCtx<'_>, xp: i64) -> LevelView {
    let progress = crate::gui::data::economy::level_progress(ctx.data, xp.max(0) as u64);
    LevelView {
        level: progress.level,
        xp_into_level: progress.xp_into_level,
        xp_needed: progress.xp_needed_for_next_level,
        percent: progress.progress_percent,
    }
}

#[derive(Debug, Clone)]
struct EconomyView {
    money: i64,
    xp: i64,
    visited_cities: i64,
    level: LevelView,
}

#[derive(Debug, Clone)]
struct ConfigView {
    traffic: i64,
    developer: i64,
    console: i64,
    convoy: i64,
    parking_doubles: i64,
    base_rows: Vec<(String, String)>,
    save_rows: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
struct CoachStop {
    id: String,
    name: String,
    discovered: Option<bool>,
    resolved: bool,
}

#[derive(Debug, Clone)]
struct CoachJob {
    route: String,
    bus: String,
    job_type: String,
    reward: String,
    xp: String,
    distance: String,
    passengers: String,
}

#[derive(Debug, Clone)]
struct CoachStat {
    label: String,
    value: String,
}

#[derive(Debug, Clone)]
struct CoachView {
    save_hash: String,
    bus_xp: Option<i64>,
    truck_xp: Option<i64>,
    level: Option<LevelView>,
    discovered_stops: usize,
    stop_count: usize,
    declared_stop_count: Option<String>,
    finished_jobs: Option<i64>,
    total_jobs: usize,
    total_reward: i64,
    passengers: Option<i64>,
    distance_km: Option<f64>,
    my_vehicles_mode: Option<String>,
    buses: Option<String>,
    current_job: Option<String>,
    warnings: Vec<String>,
    stats: Vec<CoachStat>,
    jobs: Vec<CoachJob>,
    stops: Vec<CoachStop>,
}

fn prettify_stat_field(field: &str) -> String {
    let rest = field.strip_prefix("bus_").unwrap_or(field);
    rest.split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn dash(value: Option<i64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "—".to_string())
}

fn km(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.0} km"))
        .unwrap_or_else(|| "—".to_string())
}

// ------------------------------------------------------------ deferred work

#[derive(Debug, Clone)]
enum Operation {
    RefreshAll,
    RefreshEconomy,
    RefreshSkills,
    RefreshConfig,
    RefreshCoaches,
    ApplyMoney(i64),
    ApplyExperience(i64),
    ApplyLevel(i64),
    ApplyReset {
        level: Option<u32>,
        xp: Option<u64>,
        money: Option<i64>,
    },
    ApplySkill {
        key: &'static str,
        value: i64,
    },
    MaxSkills,
    SetConfig {
        key: ConfigKey,
        value: i64,
    },
    SetCustomKey {
        scope: KeyScope,
        key: String,
        value: String,
    },
    ApplyBusXp {
        xp: i64,
        hash: String,
    },
}

#[derive(Debug, Clone)]
struct Confirm {
    title: String,
    message: String,
    action: Operation,
}

/// One persistent state instance; navigation never clears it.
#[derive(Default)]
pub struct ScreenState {
    pending: Option<Operation>,
    confirm: Option<Confirm>,
    loading: Option<&'static str>,
    initial_load_attempted: bool,

    // Economy & Experience.
    economy: Option<EconomyView>,
    economy_tried: bool,
    economy_error: Option<String>,
    economy_inputs_ready: bool,
    money_input: i64,
    xp_input: i64,
    level_input: i64,
    batch_level_edited_last: bool,
    economy_validation: String,
    economy_message: String,
    reset_validation: String,

    // Skills.
    skills_loaded: bool,
    skills_error: Option<String>,
    skill_adr: i64,
    skill_long_dist: i64,
    skill_heavy: i64,
    skill_fragile: i64,
    skill_urgent: i64,
    skill_mechanical: i64,
    skills_validation: String,
    skills_error_line: String,
    skills_message: String,

    // Config values.
    config: Option<ConfigView>,
    config_tried: bool,
    config_error: Option<String>,
    traffic_input: i64,
    convoy_input: i64,
    developer_input: bool,
    console_input: bool,
    parking_doubles_input: bool,
    config_inputs_ready: bool,
    config_row_error: String,
    config_message: String,

    // Custom config keys.
    base_rows: Vec<KeyRow>,
    save_rows: Vec<KeyRow>,
    base_key_input: String,
    base_value_input: String,
    save_key_input: String,
    save_value_input: String,
    custom_validation: String,
    custom_error_line: String,
    custom_message: String,

    // Project Coaches.
    coach: Option<CoachView>,
    coach_tried: bool,
    coach_error: Option<String>,
    bus_xp_input: i64,
    bus_xp_input_ready: bool,
    coach_validation: String,
    coach_error_line: String,
    coach_message: String,
    stop_search: String,
    stop_filter: StopFilter,
}

impl ScreenState {
    fn schedule(&mut self, operation: Operation, loading: &'static str) {
        self.pending = Some(operation);
        self.loading = Some(loading);
    }

    fn ask_confirm(&mut self, title: &str, message: String, operation: Operation) {
        self.confirm = Some(Confirm {
            title: title.to_string(),
            message,
            action: operation,
        });
    }

    fn skip_loading(&mut self) {
        self.loading = None;
    }
}

// ------------------------------------------------------------------ rendering

/// Exact screen contract. `route` / `sub_view` are shell-owned read-only fields.
pub fn render(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    if !state.initial_load_attempted && state.pending.is_none() {
        state.initial_load_attempted = true;
        state.schedule(Operation::RefreshAll, initial_loading(ctx.route));
    }
    if let Some(operation) = state.pending.take() {
        execute(operation, state, ctx);
    }

    ui.horizontal_wrapped(|ui| {
        if comp::primary_button(ui, "Refresh Save Data").clicked() {
            state.schedule(Operation::RefreshAll, initial_loading(ctx.route));
        }
        if comp::button(ui, "Issues & Logs").clicked() {
            ctx.commands
                .push(ShellCommand::OpenIssues(IssuesTab::Errors));
        }
        staging_note(ui);
    });
    ui.add_space(6.0);

    if let Some(message) = state.loading {
        comp::loading_state(ui, message);
    }

    let view = normalized_sub_view(ctx.route, ctx.sub_view);
    match ctx.route {
        Route::save_data_economy => match view {
            SubView::EconomySkills => render_skills(ui, state, ctx),
            SubView::EconomyCustomReset => render_custom_reset(ui, state, ctx),
            SubView::EconomyLevelPreview => render_level_preview(ui, state, ctx),
            _ => render_economy_fields(ui, state, ctx),
        },
        Route::save_data_coaches => render_coaches(ui, state, ctx, view),
        Route::save_data_config => render_config(ui, state, ctx),
        Route::save_data_custom_keys => render_custom_keys(ui, state, ctx, view),
        _ => render_economy_fields(ui, state, ctx),
    }

    render_confirm(ui, state, ctx);

    if state.pending.is_none() {
        state.skip_loading();
    }
}

fn initial_loading(route: Route) -> &'static str {
    match route {
        Route::save_data_coaches => "Loading bus career…",
        Route::save_data_config => "Loading config values…",
        Route::save_data_custom_keys => "Loading base/save config…",
        _ => "Loading save values…",
    }
}

fn render_confirm(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    let Some(confirm) = state.confirm.clone() else {
        return;
    };
    if let Some(accepted) = comp::confirm_dialog(
        ui.ctx(),
        &confirm.title,
        &confirm.message,
        "Confirm",
        "Cancel",
    ) {
        state.confirm = None;
        if accepted {
            state.schedule(confirm.action, "Applying change…");
        } else {
            info(ctx, format!("{} cancelled.", confirm.title));
        }
    }
}

// ------------------------------------------------------------- operation runner

fn execute(operation: Operation, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match operation {
        Operation::RefreshAll => {
            refresh_economy(state, ctx);
            refresh_skills(state, ctx);
            refresh_config(state, ctx);
            refresh_coaches(state, ctx);
        }
        Operation::RefreshEconomy => refresh_economy(state, ctx),
        Operation::RefreshSkills => refresh_skills(state, ctx),
        Operation::RefreshConfig => refresh_config(state, ctx),
        Operation::RefreshCoaches => refresh_coaches(state, ctx),
        Operation::ApplyMoney(value) => {
            match crate::gui::data::economy::edit_money(ctx.data, value) {
                Ok(()) => {
                    state.economy_message = format!("Staged money €{value} — press Save to write.");
                    state.economy_validation.clear();
                    info(ctx, state.economy_message.clone());
                    refresh_economy(state, ctx);
                }
                Err(err) => {
                    state.economy_error_line = format!("Money update failed: {}", err.message);
                    error(ctx, state.economy_error_line.clone());
                }
            }
        }
        Operation::ApplyExperience(value) => {
            match crate::gui::data::economy::edit_experience(ctx.data, value) {
                Ok(()) => {
                    state.economy_message = format!("Staged truck XP {value} — press Save to write.");
                    state.economy_validation.clear();
                    info(ctx, state.economy_message.clone());
                    refresh_economy(state, ctx);
                }
                Err(err) => {
                    state.economy_error_line = format!("XP update failed: {}", err.message);
                    error(ctx, state.economy_error_line.clone());
                }
            }
        }
        Operation::ApplyLevel(level) => {
            match crate::gui::data::economy::edit_level(ctx.data, level) {
                Ok(()) => {
                    state.economy_message = format!("Staged level {level} — press Save to write.");
                    state.economy_validation.clear();
                    info(ctx, state.economy_message.clone());
                    refresh_economy(state, ctx);
                }
                Err(err) => {
                    state.economy_error_line = format!("Level update failed: {}", err.message);
                    error(ctx, state.economy_error_line.clone());
                }
            }
        }
        Operation::ApplyReset { level, xp, money } => {
            match crate::gui::data::economy::apply_custom_reset(ctx.data, level, xp, money) {
                Ok(result) => {
                    let what = match (result.applied_level, result.applied_xp) {
                        (Some(level), _) => format!("level {level}"),
                        (None, Some(xp)) => format!("XP {xp}"),
                        (None, None) => "money".to_string(),
                    };
                    state.reset_validation.clear();
                    state.economy_message = format!(
                        "Staged custom reset ({what}; backup {}). Press Save to write.",
                        result.undo_backup_id
                    );
                    info(ctx, state.economy_message.clone());
                    refresh_economy(state, ctx);
                }
                Err(err) => {
                    state.reset_validation = format!("Custom reset failed: {}", err.message);
                    error(ctx, state.reset_validation.clone());
                }
            }
        }
        Operation::ApplySkill { key, value } => {
            match crate::gui::data::economy::set_skill(ctx.data, key.to_string(), value) {
                Ok(()) => {
                    state.skills_validation.clear();
                    state.skills_error_line.clear();
                    state.skills_message =
                        format!("Staged skill `{key}` = {value} — press Save to write.");
                    info(ctx, state.skills_message.clone());
                }
                Err(err) => {
                    state.skills_error_line = format!("Skill `{key}` update failed: {}", err.message);
                    error(ctx, state.skills_error_line.clone());
                }
            }
        }
        Operation::MaxSkills => {
            let mut failures = Vec::new();
            for (key, max) in [
                ("adr", 63_i64),
                ("long_dist", 6),
                ("heavy", 6),
                ("fragile", 6),
                ("urgent", 6),
                ("mechanical", 6),
            ] {
                if let Err(err) = crate::gui::data::economy::set_skill(
                    ctx.data,
                    key.to_string(),
                    max,
                ) {
                    failures.push(format!("{key}: {}", err.message));
                }
            }
            state.skill_adr = 63;
            state.skill_long_dist = 6;
            state.skill_heavy = 6;
            state.skill_fragile = 6;
            state.skill_urgent = 6;
            state.skill_mechanical = 6;
            if failures.is_empty() {
                state.skills_error_line.clear();
                state.skills_message =
                    "Staged all six skills at maximum (ADR 63, others 6).".to_string();
                info(ctx, state.skills_message.clone());
            } else {
                state.skills_error_line = format!("Max All Skills failed for: {}", failures.join("; "));
                error(ctx, state.skills_error_line.clone());
            }
        }
        Operation::SetConfig { key, value } => {
            let result = match key {
                ConfigKey::Traffic => crate::gui::data::economy::set_traffic(ctx.data, value),
                ConfigKey::Developer => crate::gui::data::economy::set_developer(ctx.data, value),
                ConfigKey::Console => crate::gui::data::economy::set_console(ctx.data, value),
                ConfigKey::Convoy => crate::gui::data::economy::set_convoy(ctx.data, value),
                ConfigKey::ParkingDoubles => {
                    crate::gui::data::economy::set_parking_doubles(ctx.data, value)
                }
            };
            match result {
                Ok(()) => {
                    state.config_row_error.clear();
                    state.config_message =
                        format!("Staged {} = {value} — press Save to write.", key.label());
                    info(ctx, state.config_message.clone());
                    if let Some(view) = state.config.as_mut() {
                        match key {
                            ConfigKey::Traffic => view.traffic = value,
                            ConfigKey::Developer => view.developer = value,
                            ConfigKey::Console => view.console = value,
                            ConfigKey::Convoy => view.convoy = value,
                            ConfigKey::ParkingDoubles => view.parking_doubles = value,
                        }
                    }
                }
                Err(err) => {
                    state.config_row_error = format!("{} failed: {}", key.label(), err.message);
                    error(ctx, state.config_row_error.clone());
                    if let Some(view) = state.config.as_ref() {
                        match key {
                            ConfigKey::Traffic => state.traffic_input = view.traffic,
                            ConfigKey::Developer => state.developer_input = view.developer != 0,
                            ConfigKey::Console => state.console_input = view.console != 0,
                            ConfigKey::Convoy => state.convoy_input = view.convoy,
                            ConfigKey::ParkingDoubles => {
                                state.parking_doubles_input = view.parking_doubles != 0
                            }
                        }
                    }
                }
            }
        }
        Operation::SetCustomKey { scope, key, value } => {
            let result = match scope {
                KeyScope::Base => {
                    crate::gui::data::economy::set_base_config_key(ctx.data, key.clone(), value.clone())
                }
                KeyScope::Save => {
                    crate::gui::data::economy::set_save_config_key(ctx.data, key.clone(), value.clone())
                }
            };
            match result {
                Ok(()) => {
                    let rows = if scope == KeyScope::Base {
                        &mut state.base_rows
                    } else {
                        &mut state.save_rows
                    };
                    if let Some(row) = rows.iter_mut().find(|row| row.key == key) {
                        row.value = value.clone();
                        row.error.clear();
                    } else {
                        rows.push(KeyRow {
                            key: key.clone(),
                            value: value.clone(),
                            error: String::new(),
                        });
                    }
                    state.custom_validation.clear();
                    state.custom_error_line.clear();
                    state.custom_message = format!(
                        "Staged {} key `{key}` = `{value}` — press Save to write.",
                        scope.label()
                    );
                    info(ctx, state.custom_message.clone());
                }
                Err(err) => {
                    let message = format!("{} key `{key}` failed: {}", scope.label(), err.message);
                    let rows = if scope == KeyScope::Base {
                        &mut state.base_rows
                    } else {
                        &mut state.save_rows
                    };
                    if let Some(row) = rows.iter_mut().find(|row| row.key == key) {
                        row.error = message.clone();
                    } else {
                        rows.push(KeyRow {
                            key: key.clone(),
                            value: value.clone(),
                            error: message.clone(),
                        });
                    }
                    state.custom_error_line = message.clone();
                    error(ctx, message);
                }
            }
        }
        Operation::ApplyBusXp { xp, hash } => {
            match crate::gui::data::coaches::apply_bus_xp(ctx.data, xp, hash) {
                Ok(()) => {
                    state.coach_error_line.clear();
                    state.coach_validation.clear();
                    state.coach_message =
                        format!("Staged bus XP {xp} — press Save to write.");
                    info(ctx, state.coach_message.clone());
                    refresh_coaches(state, ctx);
                }
                Err(err) => {
                    state.coach_error_line = format!("Bus XP update failed: {}", err.message);
                    error(ctx, state.coach_error_line.clone());
                }
            }
        }
    }
}

// --------------------------------------------------------------------- loaders

fn refresh_economy(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    state.economy_tried = true;
    match crate::gui::data::context::read_all_save_data(ctx.data) {
        Ok(data) => {
            let money = data.money.unwrap_or(0);
            let xp = data.xp.unwrap_or(0);
            let level = level_view(ctx, xp);
            state.economy_error = None;
            state.economy = Some(EconomyView {
                money,
                xp,
                visited_cities: data.visited_cities.unwrap_or(0),
                level,
            });
            if !state.economy_inputs_ready {
                state.money_input = money;
                state.xp_input = xp;
                state.level_input = level.level as i64;
                // The XP editors use the true truck XP from game.sii, not
                // the displayed total (info.sii = truck + bus).
                match crate::gui::data::context::try_read_truck_xp(ctx.data) {
                    Ok(Some(truck_xp)) => {
                        state.xp_input = truck_xp;
                        state.level_input =
                            crate::gui::data::economy::level_progress(
                                ctx.data,
                                truck_xp.max(0) as u64,
                            )
                            .level as i64;
                    }
                    Ok(None) => {}
                    Err(err) => error(ctx, format!("Truck XP unreadable: {}", err.message)),
                }
                state.economy_inputs_ready = true;
            }
        }
        Err(err) => {
            state.economy = None;
            state.economy_error = Some(err.message.clone());
            error(ctx, format!("Save data read failed: {}", err.message));
        }
    }
}

fn refresh_skills(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match crate::gui::data::context::quicksave_game_info(ctx.data) {
        Ok(info) => {
            state.skills_error = None;
            state.skills_loaded = true;
            if !state.economy_inputs_ready {
                // Economy inputs are initialised by refresh_economy; avoid a
                // second read of the same save here.
                state.skill_adr = info.adr.unwrap_or(0);
                state.skill_long_dist = info.long_dist.unwrap_or(0);
                state.skill_heavy = info.heavy.unwrap_or(0);
                state.skill_fragile = info.fragile.unwrap_or(0);
                state.skill_urgent = info.urgent.unwrap_or(0);
                state.skill_mechanical = info.mechanical.unwrap_or(0);
            } else {
                state.skill_adr = info.adr.unwrap_or(state.skill_adr);
                state.skill_long_dist = info.long_dist.unwrap_or(state.skill_long_dist);
                state.skill_heavy = info.heavy.unwrap_or(state.skill_heavy);
                state.skill_fragile = info.fragile.unwrap_or(state.skill_fragile);
                state.skill_urgent = info.urgent.unwrap_or(state.skill_urgent);
                state.skill_mechanical = info.mechanical.unwrap_or(state.skill_mechanical);
            }
        }
        Err(err) => {
            state.skills_loaded = false;
            state.skills_error = Some(err.message.clone());
            error(ctx, format!("Skill values unreadable: {}", err.message));
        }
    }
}

/// The active profile path is needed for `read_save_config`. It is read from
/// the existing share-context data wrapper (which reports the backend's
/// active profile), with the remembered last profile as a fallback.
fn active_profile_path(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) -> Option<String> {
    match crate::gui::data::safety::share_context(ctx.data, None) {
        Ok(context) => {
            if let Some(path) = context.profile_path {
                return Some(path);
            }
            if let Some(err) = context.path_resolution_error {
                state.config_row_error = err.clone();
                error(ctx, err);
            }
        }
        Err(err) => error(ctx, format!("Active profile unreadable: {}", err.message)),
    }
    match crate::gui::data::context::read_last_profile(ctx.data) {
        Ok(path) => path,
        Err(err) => {
            error(ctx, format!("Last profile unreadable: {}", err.message));
            None
        }
    }
}

fn refresh_config(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    state.config_tried = true;
    let base = crate::gui::data::context::read_base_config(ctx.data);
    let base = match base {
        Ok(base) => base,
        Err(err) => {
            state.config = None;
            state.config_error = Some(err.message.clone());
            error(ctx, format!("Config read failed: {}", err.message));
            return;
        }
    };

    let save = active_profile_path(state, ctx).and_then(|path| {
        match crate::gui::data::context::read_save_config(ctx.data, path) {
            Ok(save) => Some(save),
            Err(err) => {
                state.config_row_error = format!("Profile config read failed: {}", err.message);
                error(ctx, state.config_row_error.clone());
                None
            }
        }
    });

    let traffic = base.traffic.unwrap_or(1);
    let developer = base.developer.unwrap_or(0);
    let console = base.console.unwrap_or(0);
    let convoy = base.max_convoy_size.unwrap_or(8);
    let parking_value = save.as_ref().and_then(|save| save.factor_parking_doubles);
    let parking_doubles = parking_value.unwrap_or(0);

    let optional = |value: Option<i64>| {
        value
            .map(|value| value.to_string())
            .unwrap_or_else(|| "—".to_string())
    };
    let view = ConfigView {
        traffic,
        developer,
        console,
        convoy,
        parking_doubles,
        base_rows: vec![
            ("g_traffic".to_string(), optional(base.traffic)),
            ("g_developer".to_string(), optional(base.developer)),
            ("g_console".to_string(), optional(base.console)),
            (
                "g_max_convoy_size".to_string(),
                optional(base.max_convoy_size),
            ),
        ],
        save_rows: vec![(
            "g_simple_parking_doubles".to_string(),
            optional(parking_value),
        )],
    };

    state.config = Some(view);
    state.config_error = None;
    if !state.config_inputs_ready {
        state.traffic_input = traffic;
        state.convoy_input = convoy;
        state.developer_input = developer != 0;
        state.console_input = console != 0;
        state.parking_doubles_input = parking_doubles != 0;
        state.config_inputs_ready = true;
    }
}

fn refresh_coaches(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    state.coach_tried = true;
    match crate::gui::data::coaches::snapshot(ctx.data) {
        Ok(snapshot) => {
            let level = snapshot.bus_xp.map(|xp| {
                let progress = crate::gui::data::coaches::bus_level(ctx.data, xp.max(0) as u64);
                LevelView {
                    level: progress.level,
                    xp_into_level: progress.xp_into_level,
                    xp_needed: progress.xp_needed_for_next_level,
                    percent: progress.progress_percent,
                }
            });

            let stats = snapshot
                .statistics
                .iter()
                .filter_map(|(field, value)| {
                    value.as_ref().map(|value| CoachStat {
                        label: prettify_stat_field(field),
                        value: value.clone(),
                    })
                })
                .collect();

            let jobs = snapshot
                .job_logs
                .iter()
                .flat_map(|log| log.entries_detail.iter())
                .map(|entry| {
                    let route = match (&entry.source_stop, &entry.target_stop) {
                        (Some(from), Some(to)) => format!(
                            "{} → {}",
                            crate::gui::data::coaches::display_job_stop_name(ctx.data, from),
                            crate::gui::data::coaches::display_job_stop_name(ctx.data, to)
                        ),
                        _ => "—".to_string(),
                    };
                    CoachJob {
                        route,
                        bus: entry
                            .assigned_bus
                            .as_deref()
                            .map(|id| crate::gui::data::coaches::display_bus_name(ctx.data, id))
                            .unwrap_or_else(|| "—".to_string()),
                        job_type: entry.job_type.clone().unwrap_or_else(|| "—".to_string()),
                        reward: entry
                            .reward
                            .map(|value| format!("€{value}"))
                            .unwrap_or_else(|| "—".to_string()),
                        xp: dash(entry.exp_delta),
                        distance: entry
                            .travelled_distance
                            .map(|value| format!("{value:.0} km"))
                            .unwrap_or_else(|| "—".to_string()),
                        passengers: dash(entry.total_passengers),
                    }
                })
                .collect();

            let stops = snapshot
                .stops
                .iter()
                .map(|stop| CoachStop {
                    id: stop.id.clone(),
                    name: crate::gui::data::coaches::display_stop_name(ctx.data, &stop.id),
                    discovered: stop.discovered,
                    resolved: stop.resolved,
                })
                .collect();

            let view = CoachView {
                save_hash: snapshot.save_hash,
                bus_xp: snapshot.bus_xp,
                truck_xp: snapshot.truck_xp,
                level,
                discovered_stops: snapshot.discovered_stops,
                stop_count: snapshot.stops.len(),
                declared_stop_count: snapshot.declared_stop_count,
                finished_jobs: snapshot.finished_job_count,
                total_jobs: snapshot.total_bus_jobs,
                total_reward: snapshot.total_bus_reward,
                passengers: snapshot.total_passengers_count,
                distance_km: snapshot.total_distance_km,
                my_vehicles_mode: snapshot.my_vehicles_mode,
                buses: snapshot.buses,
                current_job: snapshot.current_bus_job,
                warnings: snapshot.warnings,
                stats,
                jobs,
                stops,
            };

            if !state.bus_xp_input_ready {
                state.bus_xp_input = view.bus_xp.unwrap_or(0);
                state.bus_xp_input_ready = true;
            }
            state.coach = Some(view);
            state.coach_error = None;
        }
        Err(err) => {
            state.coach = None;
            state.coach_error = Some(err.message.clone());
            error(ctx, format!("Coaches load failed: {}", err.message));
        }
    }
}

// -------------------------------------------------------------- data-view UI

fn data_error(ui: &mut egui::Ui, error: &str, loading: &'static str, state: &mut ScreenState) {
    if comp::error_state(
        ui,
        &format!("{error} Retry to reload this view."),
        Some("Retry"),
    ) {
        state.schedule(Operation::RefreshAll, loading);
    }
}

fn render_economy_fields(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    sub_view_nav(
        ui,
        ctx,
        Route::save_data_economy,
        SubView::EconomyFields,
        &[
            (SubView::EconomyFields, "Fields"),
            (SubView::EconomySkills, "Skills"),
            (SubView::EconomyCustomReset, "Custom reset"),
            (SubView::EconomyLevelPreview, "Level preview"),
        ],
    );

    if state.loading.is_some() && !state.economy_tried {
        comp::loading_state(ui, "Loading save values…");
        return;
    }
    if let Some(message) = state.economy_error.clone() {
        data_error(ui, &message, "Loading save values…", state);
        return;
    }
    let Some(view) = state.economy.clone() else {
        comp::empty_state(
            ui,
            "No save data loaded",
            "Choose Game, Profile and Save in the Context Column, then Refresh Save Data.",
        );
        return;
    };

    comp::section_card(ui, "Player economy & experience", |ui| {
        ui.horizontal_wrapped(|ui| {
            comp::compact_stat_tile(ui, "Money", &format!("€{}", view.money), theme::SUCCESS);
            comp::compact_stat_tile(ui, "Total XP", &view.xp.to_string(), theme::ACCENT);
            comp::compact_stat_tile(
                ui,
                "Level",
                &view.level.level.to_string(),
                theme::ACCENT,
            );
            comp::compact_stat_tile(
                ui,
                "Visited cities",
                &view.visited_cities.to_string(),
                theme::MUTED,
            );
        });
    });
    ui.add_space(6.0);

    comp::section_card(ui, "Money", |ui| {
        ui.horizontal(|ui| {
            ui.label("Money (€):");
            ui.add(
                egui::DragValue::new(&mut state.money_input)
                    .speed(10_000)
                    .range(0..=999_999_999_999_i64),
            );
            if comp::button(ui, "Apply Money").clicked() {
                let message = money_validation(state.money_input);
                if let Some(message) = message {
                    validation_economy(state, ctx, message);
                } else {
                    state.schedule(
                        Operation::ApplyMoney(state.money_input),
                        "Staging money…",
                    );
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            for (label, amount) in [
                ("+100k", 100_000_i64),
                ("+1M", 1_000_000_i64),
                ("Set 10M", 10_000_000_i64),
                ("Set 100M", 100_000_000_i64),
            ] {
                if comp::button(ui, label).clicked() {
                    state.money_input = if label.starts_with('+') {
                        state.money_input.saturating_add(amount)
                    } else {
                        amount
                    };
                }
            }
        });
    });
    ui.add_space(6.0);

    comp::section_card(ui, "Experience & level", |ui| {
        ui.horizontal(|ui| {
            ui.label("Truck XP:");
            if ui
                .add(
                    egui::DragValue::new(&mut state.xp_input)
                        .speed(1_000)
                        .range(0..=999_999_999_i64),
                )
                .changed()
            {
                state.batch_level_edited_last = false;
            }
            if comp::button(ui, "Apply XP").clicked() {
                if let Some(message) = xp_validation(state.xp_input) {
                    validation_economy(state, ctx, message);
                } else {
                    state.schedule(
                        Operation::ApplyExperience(state.xp_input),
                        "Staging XP…",
                    );
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Target level:");
            if ui
                .add(egui::DragValue::new(&mut state.level_input).range(0..=256))
                .changed()
            {
                state.batch_level_edited_last = true;
            }
            if comp::button(ui, "Apply Level").clicked() {
                if let Some(message) = level_validation(state.level_input) {
                    validation_economy(state, ctx, message);
                } else {
                    state.schedule(
                        Operation::ApplyLevel(state.level_input),
                        "Staging level…",
                    );
                }
            }
        });
        ui.horizontal(|ui| {
            if comp::button(ui, "Apply custom reset (money + XP / level)").clicked() {
                let message = money_validation(state.money_input)
                    .or_else(|| xp_validation(state.xp_input))
                    .or_else(|| level_validation(state.level_input));
                if let Some(message) = message {
                    state.reset_validation = message.clone();
                    validation(ctx, message);
                } else {
                    let (level, xp) = if state.batch_level_edited_last {
                        (Some(state.level_input.max(0) as u32), None)
                    } else {
                        (None, Some(state.xp_input.max(0) as u64))
                    };
                    state.reset_validation.clear();
                    state.schedule(
                        Operation::ApplyReset {
                            level,
                            xp,
                            money: Some(state.money_input),
                        },
                        "Staging custom reset…",
                    );
                }
            }
            staging_note(ui);
        });
    });

    comp::validation_message(ui, &state.economy_validation);
    comp::validation_message(ui, &state.reset_validation);
    comp::validation_message(ui, &state.economy_error_line);
    inline_message(ui, &state.economy_message);
}

fn render_skills(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    sub_view_nav(
        ui,
        ctx,
        Route::save_data_economy,
        SubView::EconomySkills,
        &[
            (SubView::EconomyFields, "Fields"),
            (SubView::EconomySkills, "Skills"),
            (SubView::EconomyCustomReset, "Custom reset"),
            (SubView::EconomyLevelPreview, "Level preview"),
        ],
    );

    if state.loading.is_some() && !state.skills_loaded {
        comp::loading_state(ui, "Loading save values…");
        return;
    }
    if let Some(message) = state.skills_error.clone() {
        data_error(ui, &message, "Loading save values…", state);
        return;
    }
    if !state.skills_loaded {
        comp::empty_state(
            ui,
            "No save data loaded",
            "Skill values were not found in this save. Refresh Save Data to retry.",
        );
        return;
    }

    comp::section_card(ui, "Player skills (ADR 0–63, others 0–6)", |ui| {
        let skills: [(&str, &mut i64, i64); 6] = [
            ("ADR (hazardous)", &mut state.skill_adr, 63),
            ("Long distance", &mut state.skill_long_dist, 6),
            ("Heavy cargo", &mut state.skill_heavy, 6),
            ("Fragile cargo", &mut state.skill_fragile, 6),
            ("Just-in-time", &mut state.skill_urgent, 6),
            ("Eco driving", &mut state.skill_mechanical, 6),
        ];
        let mut save: Option<(&'static str, i64)> = None;
        for (label, value, max) in skills {
            ui.horizontal(|ui| {
                ui.label(label);
                ui.add(egui::Slider::new(&mut *value, 0..=max));
                if comp::button(ui, "Apply").clicked() {
                    let key = match label {
                        "ADR (hazardous)" => "adr",
                        "Long distance" => "long_dist",
                        "Heavy cargo" => "heavy",
                        "Fragile cargo" => "fragile",
                        "Just-in-time" => "urgent",
                        _ => "mechanical",
                    };
                    if *value < 0 || *value > max {
                        let message = format!("{key} must be between 0 and {max}.");
                        state.skills_validation = message.clone();
                        validation(ctx, message);
                    } else {
                        save = Some((key, *value));
                    }
                }
            });
        }
        if let Some((key, value)) = save {
            state.skills_validation.clear();
            state.schedule(
                Operation::ApplySkill { key, value },
                "Staging skill…",
            );
        }
        if comp::button(ui, "Max All Skills (ADR 63, others 6)").clicked() {
            state.ask_confirm(
                "Max all skills",
                "Stage ADR 63 and the other five skills at 6? Six staged writes will be queued; the backend creates a backup when you press Save in the Safety Spine.".to_string(),
                Operation::MaxSkills,
            );
        }
    });
    ui.add_space(6.0);
    comp::validation_message(ui, &state.skills_validation);
    comp::validation_message(ui, &state.skills_error_line);
    inline_message(ui, &state.skills_message);
}

fn render_custom_reset(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    sub_view_nav(
        ui,
        ctx,
        Route::save_data_economy,
        SubView::EconomyCustomReset,
        &[
            (SubView::EconomyFields, "Fields"),
            (SubView::EconomySkills, "Skills"),
            (SubView::EconomyCustomReset, "Custom reset"),
            (SubView::EconomyLevelPreview, "Level preview"),
        ],
    );

    if state.loading.is_some() && !state.economy_tried {
        comp::loading_state(ui, "Loading save values…");
        return;
    }
    if let Some(message) = state.economy_error.clone() {
        data_error(ui, &message, "Loading save values…", state);
        return;
    }
    if state.economy.is_none() {
        comp::empty_state(
            ui,
            "No save data loaded",
            "Custom reset values need an active save.",
        );
        return;
    }

    comp::section_card(ui, "Custom reset values", |ui| {
        ui.label(
            "The batch applies money plus the last-edited of XP or target level; \
             the backend reports the level actually written.",
        );
        ui.horizontal(|ui| {
            ui.label("Reset money (€):");
            ui.add(
                egui::DragValue::new(&mut state.money_input)
                    .speed(10_000)
                    .range(0..=999_999_999_999_i64),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Reset truck XP:");
            if ui
                .add(
                    egui::DragValue::new(&mut state.xp_input)
                        .speed(1_000)
                        .range(0..=999_999_999_i64),
                )
                .changed()
            {
                state.batch_level_edited_last = false;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Reset target level:");
            if ui
                .add(egui::DragValue::new(&mut state.level_input).range(0..=256))
                .changed()
            {
                state.batch_level_edited_last = true;
            }
        });
        ui.label(format!(
            "Last edited: {}",
            if state.batch_level_edited_last {
                "target level"
            } else {
                "truck XP"
            }
        ));
        if comp::button(ui, "Apply custom reset").clicked() {
            if let Some(message) = money_validation(state.money_input)
                .or_else(|| xp_validation(state.xp_input))
                .or_else(|| level_validation(state.level_input))
            {
                state.reset_validation = message.clone();
                validation(ctx, message);
            } else {
                state.reset_validation.clear();
                let (level, xp) = if state.batch_level_edited_last {
                    (Some(state.level_input.max(0) as u32), None)
                } else {
                    (None, Some(state.xp_input.max(0) as u64))
                };
                state.schedule(
                    Operation::ApplyReset {
                        level,
                        xp,
                        money: Some(state.money_input),
                    },
                    "Staging custom reset…",
                );
            }
        }
        staging_note(ui);
    });
    comp::validation_message(ui, &state.reset_validation);
    comp::validation_message(ui, &state.economy_error_line);
    inline_message(ui, &state.economy_message);
}

fn render_level_preview(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    sub_view_nav(
        ui,
        ctx,
        Route::save_data_economy,
        SubView::EconomyLevelPreview,
        &[
            (SubView::EconomyFields, "Fields"),
            (SubView::EconomySkills, "Skills"),
            (SubView::EconomyCustomReset, "Custom reset"),
            (SubView::EconomyLevelPreview, "Level preview"),
        ],
    );

    if state.loading.is_some() && !state.economy_tried {
        comp::loading_state(ui, "Loading save values…");
        return;
    }
    if let Some(message) = state.economy_error.clone() {
        data_error(ui, &message, "Loading save values…", state);
        return;
    }
    if state.economy.is_none() {
        comp::empty_state(
            ui,
            "No save data loaded",
            "Level preview becomes available once the active save is read.",
        );
        return;
    }

    comp::section_card(ui, "Level preview", |ui| {
        let preview = level_view(ctx, state.xp_input);
        let next = (preview.level + 1).min(256);
        ui.label(format!("Preview for {} XP", state.xp_input.max(0)));
        ui.add(
            egui::ProgressBar::new(preview.percent / 100.0).text(format!(
                "Level {} — {} / {} XP to level {}",
                preview.level,
                preview.xp_into_level,
                preview.xp_into_level + preview.xp_needed,
                next
            )),
        );
        if preview.xp_needed == 0 {
            ui.label("Maximum level reached.");
        }
        ui.separator();
        ui.label(format!(
            "Currently staged XP: {} · current level: {}",
            state.xp_input, state.level_input
        ));
        comp::validation_message(ui, &state.economy_validation);
    });
}

fn render_coaches(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
    view: SubView,
) {
    sub_view_nav(
        ui,
        ctx,
        Route::save_data_coaches,
        view,
        &[
            (SubView::CoachesOverview, "Overview"),
            (SubView::CoachesStats, "Statistics"),
            (SubView::CoachesJobHistory, "Job history"),
            (SubView::CoachesStops, "Bus stops"),
        ],
    );
    if comp::button(ui, "Reload coaches data").clicked() {
        state.schedule(Operation::RefreshCoaches, "Loading bus career…");
    }

    if state.loading.is_some() && !state.coach_tried {
        comp::loading_state(ui, "Loading bus career…");
        return;
    }
    if let Some(message) = state.coach_error.clone() {
        data_error(ui, &message, "Loading bus career…", state);
        return;
    }
    let Some(coach) = state.coach.clone() else {
        comp::empty_state(
            ui,
            "No bus career data",
            "Load Coaches Data from this tile or Refresh Save Data.",
        );
        return;
    };

    if !coach.warnings.is_empty() {
        comp::section_card(ui, "Read warnings", |ui| {
            for warning in &coach.warnings {
                ui.horizontal(|ui| {
                    comp::status_dot(ui, theme::WARNING);
                    ui.label(warning);
                });
            }
        });
        ui.add_space(6.0);
    }

    match view {
        SubView::CoachesStats => render_coach_stats(ui, &coach),
        SubView::CoachesJobHistory => render_coach_jobs(ui, &coach),
        SubView::CoachesStops => render_coach_stops(ui, state, ctx, &coach),
        _ => render_coach_overview(ui, state, ctx, &coach),
    }
}

fn render_coach_overview(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
    coach: &CoachView,
) {
    comp::section_card(ui, "Bus career", |ui| {
        ui.horizontal_wrapped(|ui| {
            comp::compact_stat_tile(
                ui,
                "Bus level",
                &coach
                    .level
                    .map(|level| level.level.to_string())
                    .unwrap_or_else(|| "—".to_string()),
                theme::ACCENT,
            );
            comp::compact_stat_tile(ui, "Bus XP", &dash(coach.bus_xp), theme::TEXT);
            comp::compact_stat_tile(
                ui,
                "Discovered stops",
                &format!("{}/{}", coach.discovered_stops, coach.stop_count),
                theme::SUCCESS,
            );
            comp::compact_stat_tile(
                ui,
                "Jobs completed",
                &dash(coach.finished_jobs),
                theme::TEXT,
            );
            comp::compact_stat_tile(
                ui,
                "Total earned",
                &format!("€{}", coach.total_reward),
                theme::SUCCESS,
            );
            comp::compact_stat_tile(ui, "Passengers", &dash(coach.passengers), theme::TEXT);
            comp::compact_stat_tile(ui, "Distance driven", &km(coach.distance_km), theme::TEXT);
        });
        if let Some(level) = coach.level {
            ui.add(
                egui::ProgressBar::new(level.percent / 100.0).text(format!(
                    "{} / {} XP to level {}",
                    level.xp_into_level,
                    level.xp_into_level + level.xp_needed,
                    (level.level + 1).min(256)
                )),
            );
            if level.xp_needed == 0 {
                ui.label("Maximum level reached.");
            }
        }
    });
    ui.add_space(6.0);

    comp::section_card(ui, "Career & XP", |ui| {
        ui.label(format!(
            "Mode: {} · owned buses: {} · current job: {}",
            coach.my_vehicles_mode.as_deref().unwrap_or("unknown"),
            coach.buses.as_deref().unwrap_or("—"),
            coach.current_job.as_deref().unwrap_or("none")
        ));
        if let Some(declared) = &coach.declared_stop_count {
            ui.label(format!("Declared stop count: {declared}"));
        }
        ui.horizontal(|ui| {
            ui.label("Bus XP:");
            ui.add(
                egui::DragValue::new(&mut state.bus_xp_input)
                    .speed(100)
                    .range(0..=9_007_199_254_740_991_i64),
            );
            if comp::button(ui, "Apply").clicked() {
                apply_bus_xp(ui, state, ctx, coach);
            }
            if let Some(truck_xp) = coach.truck_xp {
                if comp::button(ui, "Set bus XP = truck XP").clicked() {
                    state.bus_xp_input = truck_xp;
                    apply_bus_xp(ui, state, ctx, coach);
                }
            }
        });
        if coach.bus_xp.is_none() {
            ui.label("This save has no bus career yet; bus XP cannot be edited.");
        } else {
            ui.label("Only bus XP changes. Bus levels are derived from XP.");
        }
        comp::validation_message(ui, &state.coach_validation);
        comp::validation_message(ui, &state.coach_error_line);
        inline_message(ui, &state.coach_message);
    });
}

fn apply_bus_xp(
    _ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
    coach: &CoachView,
) {
    if coach.bus_xp.is_none() {
        let message = "This save has no bus career; bus XP cannot be edited.".to_string();
        state.coach_validation = message.clone();
        validation(ctx, message);
        return;
    }
    if state.bus_xp_input < 0 {
        let message = "Bus XP must be a non-negative number.".to_string();
        state.coach_validation = message.clone();
        validation(ctx, message);
        return;
    }
    let operation = Operation::ApplyBusXp {
        xp: state.bus_xp_input,
        hash: coach.save_hash.clone(),
    };
    // Warnings mean the save was not read unambiguously; require an
    // explicit confirmation before staging a write against that hash.
    if coach.warnings.is_empty() {
        state.schedule(operation, "Staging bus XP…");
    } else {
        state.ask_confirm(
            "Apply bus XP to an ambiguous save",
            format!(
                "The snapshot carries {} read warning(s) and will be written against save hash {}. Stage bus XP {} anyway?",
                coach.warnings.len(),
                coach.save_hash,
                state.bus_xp_input
            ),
            operation,
        );
    }
}

fn render_coach_stats(ui: &mut egui::Ui, coach: &CoachView) {
    comp::section_card(ui, "Bus statistics", |ui| {
        if coach.stats.is_empty() {
            ui.label(
                "This save does not store per-career bus statistics yet \
                 (distance, job counts, passengers, play time).",
            );
            return;
        }
        let rows: Vec<Vec<String>> = coach
            .stats
            .iter()
            .map(|stat| vec![stat.label.clone(), stat.value.clone()])
            .collect();
        comp::table(
            ui,
            "coaches_statistics",
            &["Statistic", "Value"],
            &rows,
            None,
        );
    });
}

fn render_coach_jobs(ui: &mut egui::Ui, coach: &CoachView) {
    comp::section_card(ui, "Bus job history", |ui| {
        if coach.jobs.is_empty() {
            ui.label("No bus jobs logged in this save yet.");
            return;
        }
        let rows: Vec<Vec<String>> = coach
            .jobs
            .iter()
            .map(|job| {
                vec![
                    job.route.clone(),
                    job.bus.clone(),
                    job.job_type.clone(),
                    job.reward.clone(),
                    job.xp.clone(),
                    job.distance.clone(),
                    job.passengers.clone(),
                ]
            })
            .collect();
        comp::table(
            ui,
            "coaches_job_history",
            &[
                "Route",
                "Bus",
                "Type",
                "Reward",
                "XP",
                "Distance",
                "Passengers",
            ],
            &rows,
            None,
        );
    });
    ui.label(format!("{} jobs logged in total.", coach.total_jobs));
}

fn render_coach_stops(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    _ctx: &mut ScreenCtx<'_>,
    coach: &CoachView,
) {
    comp::section_card(ui, "Bus stops", |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label("Search:");
            comp::text_input(ui, &mut state.stop_search, "Stop name or id");
            if let Some(filter) = comp::tabs(
                ui,
                state.stop_filter,
                &[
                    (StopFilter::All, "All"),
                    (StopFilter::Discovered, "Discovered"),
                    (StopFilter::Undiscovered, "Undiscovered"),
                ],
            ) {
                state.stop_filter = filter;
            }
        });
        let query = state.stop_search.trim().to_lowercase();
        let filtered: Vec<&CoachStop> = coach
            .stops
            .iter()
            .filter(|stop| match state.stop_filter {
                StopFilter::All => true,
                StopFilter::Discovered => stop.discovered == Some(true),
                StopFilter::Undiscovered => stop.discovered != Some(true),
            })
            .filter(|stop| {
                query.is_empty()
                    || stop.name.to_lowercase().contains(&query)
                    || stop.id.to_lowercase().contains(&query)
            })
            .collect();
        ui.label(format!(
            "Showing {} of {} stops ({} discovered)",
            filtered.len(),
            coach.stop_count,
            coach.discovered_stops
        ));
        if filtered.is_empty() {
            comp::empty_state(
                ui,
                "No matching bus stops",
                "Change the search text or stop filter.",
            );
            return;
        }
        let row_height = ui.text_style_height(&egui::TextStyle::Body) + 6.0;
        egui::ScrollArea::vertical()
            .id_salt("coaches_stops_rows")
            .max_height(360.0)
            .show_rows(ui, row_height, filtered.len(), |ui, range| {
                for stop in &filtered[range] {
                    ui.horizontal(|ui| {
                        let color = match stop.discovered {
                            Some(true) => theme::SUCCESS,
                            Some(false) => theme::FAINT,
                            None => theme::WARNING,
                        };
                        comp::status_dot(ui, color);
                        ui.label(&stop.name);
                        ui.label(
                            egui::RichText::new(&stop.id)
                                .small()
                                .color(theme::MUTED),
                        );
                        if !stop.resolved {
                            ui.label(
                                egui::RichText::new("unresolved")
                                    .small()
                                    .color(theme::WARNING),
                            );
                        }
                    });
                }
            });
    });
}

fn render_config(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    if state.loading.is_some() && !state.config_tried {
        comp::loading_state(ui, "Loading config values…");
        return;
    }
    if let Some(message) = state.config_error.clone() {
        data_error(ui, &message, "Loading config values…", state);
        return;
    }
    let Some(view) = state.config.clone() else {
        comp::empty_state(
            ui,
            "No config values found",
            "Refresh Save Data to read g_traffic, g_developer, g_console, convoy and parking values.",
        );
        return;
    };

    comp::section_card(ui, "Game configuration (config.cfg)", |ui| {
        ui.horizontal(|ui| {
            ui.label("Traffic density (0–10):");
            ui.add(egui::Slider::new(&mut state.traffic_input, 0..=10));
            if comp::button(ui, "Apply").clicked() {
                if !(0..=10).contains(&state.traffic_input) {
                    validation_config(state, ctx, "Traffic density must be between 0 and 10.");
                } else {
                    state.schedule(
                        Operation::SetConfig {
                            key: ConfigKey::Traffic,
                            value: state.traffic_input,
                        },
                        "Staging traffic value…",
                    );
                }
            }
            ui.label(format!("Current: {}", view.traffic));
        });
        ui.horizontal(|ui| {
            if comp::toggle(
                ui,
                "cfg_developer",
                "Developer mode (g_developer)",
                &mut state.developer_input,
            )
            .changed()
            {
                state.schedule(
                    Operation::SetConfig {
                        key: ConfigKey::Developer,
                        value: i64::from(state.developer_input),
                    },
                    "Staging developer mode…",
                );
            }
        });
        ui.horizontal(|ui| {
            if comp::toggle(
                ui,
                "cfg_console",
                "Developer console (g_console)",
                &mut state.console_input,
            )
            .changed()
            {
                state.schedule(
                    Operation::SetConfig {
                        key: ConfigKey::Console,
                        value: i64::from(state.console_input),
                    },
                    "Staging developer console…",
                );
            }
        });
        ui.horizontal(|ui| {
            if comp::toggle(
                ui,
                "cfg_parking_doubles",
                "Double trailer parking (g_simple_parking_doubles)",
                &mut state.parking_doubles_input,
            )
            .changed()
            {
                state.schedule(
                    Operation::SetConfig {
                        key: ConfigKey::ParkingDoubles,
                        value: i64::from(state.parking_doubles_input),
                    },
                    "Staging parking doubles…",
                );
            }
        });
        ui.horizontal(|ui| {
            ui.label("Max convoy size (1–128):");
            ui.add(egui::Slider::new(&mut state.convoy_input, 1..=128));
            if comp::button(ui, "Apply").clicked() {
                if !(1..=128).contains(&state.convoy_input) {
                    validation_config(state, ctx, "Convoy size must be between 1 and 128.");
                } else {
                    state.schedule(
                        Operation::SetConfig {
                            key: ConfigKey::Convoy,
                            value: state.convoy_input,
                        },
                        "Staging convoy size…",
                    );
                }
            }
            ui.label(format!("Current: {}", view.convoy));
        });
    });
    ui.add_space(6.0);
    comp::validation_message(ui, &state.config_row_error);
    inline_message(ui, &state.config_message);
}

fn render_custom_keys(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
    view: SubView,
) {
    let scope = if view == SubView::SaveConfigKeys {
        KeyScope::Save
    } else {
        KeyScope::Base
    };
    sub_view_nav(
        ui,
        ctx,
        Route::save_data_custom_keys,
        if scope == KeyScope::Base {
            SubView::BaseConfigKeys
        } else {
            SubView::SaveConfigKeys
        },
        &[
            (SubView::BaseConfigKeys, "Base config keys"),
            (SubView::SaveConfigKeys, "Save config keys"),
        ],
    );

    if state.loading.is_some() && !state.config_tried {
        comp::loading_state(ui, "Loading base/save config…");
        return;
    }
    if let Some(message) = state.config_error.clone() {
        data_error(ui, &message, "Loading base/save config…", state);
        return;
    }
    if state.config.is_none() {
        comp::empty_state(
            ui,
            "No custom keys yet — add a key",
            "Refresh Save Data to read config keys, then add a key/value pair below.",
        );
        return;
    }

    let mut key_input = if scope == KeyScope::Base {
        state.base_key_input.clone()
    } else {
        state.save_key_input.clone()
    };
    let mut value_input = if scope == KeyScope::Base {
        state.base_value_input.clone()
    } else {
        state.save_value_input.clone()
    };

    let title = format!("{} — key/value rows", scope.label());
    comp::section_card(ui, &title, |ui| {
        let known: Vec<(String, String)> = if scope == KeyScope::Base {
            state
                .config
                .as_ref()
                .map(|config| config.base_rows.clone())
                .unwrap_or_default()
        } else {
            state
                .config
                .as_ref()
                .map(|config| config.save_rows.clone())
                .unwrap_or_default()
        };
        let custom: Vec<KeyRow> = if scope == KeyScope::Base {
            state.base_rows.clone()
        } else {
            state.save_rows.clone()
        };
        let mut rows: Vec<Vec<String>> = known
            .into_iter()
            .map(|(key, value)| vec![key, value, "config".to_string()])
            .collect();
        rows.extend(custom.into_iter().map(|row| {
            vec![
                row.key,
                row.value,
                if row.error.is_empty() {
                    "staged".to_string()
                } else {
                    row.error
                },
            ]
        }));
        if rows.is_empty() {
            comp::empty_state(
                ui,
                "No keys in this config",
                "Add a key/value pair below and press Set.",
            );
        } else {
            comp::table(
                ui,
                if scope == KeyScope::Base {
                    "base_config_keys_table"
                } else {
                    "save_config_keys_table"
                },
                &["Key", "Value", "Source / status"],
                &rows,
                None,
            );
        }

        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("Key:");
            comp::text_input(ui, &mut key_input, "g_example");
            ui.label("Value:");
            comp::text_input(ui, &mut value_input, "value");
            if comp::button(ui, "Set").clicked() {
                let key = key_input.trim().to_string();
                let value = value_input.trim().to_string();
                if key.is_empty() {
                    let message = "A config key cannot be empty.".to_string();
                    state.custom_validation = message.clone();
                    validation(ctx, message);
                } else {
                    let duplicate = state
                        .config
                        .as_ref()
                        .map(|config| {
                            if scope == KeyScope::Base {
                                config.base_rows.iter().any(|(existing, _)| existing == &key)
                            } else {
                                config.save_rows.iter().any(|(existing, _)| existing == &key)
                            }
                        })
                        .unwrap_or(false)
                        || (if scope == KeyScope::Base {
                            &state.base_rows
                        } else {
                            &state.save_rows
                        })
                        .iter()
                        .any(|row| row.key == key);
                    if duplicate {
                        let message = format!(
                            "Key `{key}` already exists in {}. Confirm the overwrite to stage a new value.",
                            scope.label()
                        );
                        state.custom_validation = message.clone();
                        validation(ctx, message.clone());
                        state.ask_confirm(
                            "Overwrite config key",
                            message,
                            Operation::SetCustomKey { scope, key, value },
                        );
                    } else {
                        state.custom_validation.clear();
                        state.schedule(
                            Operation::SetCustomKey { scope, key, value },
                            "Staging config key…",
                        );
                    }
                }
            }
        });
        staging_note(ui);
    });
    if scope == KeyScope::Base {
        state.base_key_input = key_input;
        state.base_value_input = value_input;
    } else {
        state.save_key_input = key_input;
        state.save_value_input = value_input;
    }
    comp::validation_message(ui, &state.custom_validation);
    comp::validation_message(ui, &state.custom_error_line);
    inline_message(ui, &state.custom_message);
}

// ----------------------------------------------------------------- validation

fn money_validation(value: i64) -> Option<String> {
    if !(0..=999_999_999_999).contains(&value) {
        Some("Money must be between 0 and 999999999999.".to_string())
    } else {
        None
    }
}

fn xp_validation(value: i64) -> Option<String> {
    if !(0..=999_999_999).contains(&value) {
        Some("XP must be between 0 and 999999999.".to_string())
    } else {
        None
    }
}

fn level_validation(value: i64) -> Option<String> {
    if !(0..=256).contains(&value) {
        Some("Level must be between 0 and 256.".to_string())
    } else {
        None
    }
}

fn validation_economy(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>, message: String) {
    state.economy_validation = message.clone();
    validation(ctx, message);
}

fn validation_config(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>, message: &str) {
    state.config_row_error = message.to_string();
    validation(ctx, message.to_string());
}

fn inline_message(ui: &mut egui::Ui, message: &str) {
    if !message.is_empty() {
        ui.label(egui::RichText::new(message).color(theme::SUCCESS));
    }
}
