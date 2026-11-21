//! Operations sector screen: Live Telemetry, Dispatcher,
//! Logbook & Analytics, and VTC & Account.
//!
//! Navigation comes exclusively from `ScreenCtx` (the shell owns `route` and
//! `sub_view`). Backend access goes through `gui/data` wrappers only; anything
//! global (toasts, Issues, navigation, Save) is queued as a `ShellCommand`.

use std::time::{Duration, Instant};

use eframe::egui;

use crate::gui::components as comp;
use crate::gui::data::UiError;
use crate::gui::screens::{IssuesTab, Route, ScreenCtx, ShellCommand, SubView};
use crate::gui::theme;

// ------------------------------------------------------------- local DTO views

#[derive(Debug, Clone, Default)]
struct StatusView {
    games: String,
    bridge: String,
    plugin: String,
}

#[derive(Debug, Clone, Default)]
struct OverviewView {
    company: String,
    economy: String,
    reputation: String,
    fleet: String,
    trip: String,
    telemetry: String,
}

#[derive(Debug, Clone, Default)]
struct GenerationView {
    counts: String,
    timing: String,
    context: String,
    link: String,
}

#[derive(Debug, Clone, Default)]
struct JobDetailView {
    heading: String,
    summary: String,
    fields: Vec<(String, String)>,
    risk: Option<String>,
    bonus: Option<String>,
    last_error: Option<String>,
    status: String,
}

#[derive(Debug, Clone)]
struct JobRow {
    id: String,
    cells: Vec<String>,
    reward: i64,
    distance: f64,
    rate: f64,
    detail: JobDetailView,
}

#[derive(Debug, Clone, Default)]
struct ScanView {
    summary: String,
    note: String,
}

/// Build a table row plus the detail card from any dispatcher job DTO.
macro_rules! job_row {
    ($job:expr) => {{
        let status = format!(
            "{} ({})",
            job.status,
            job.ets2_job_link_status.as_deref().unwrap_or("-")
        );
        let detail = JobDetailView {
            heading: format!("{} → {}", job.origin_city, job.destination_city),
            summary: format!(
                "{} · {} ({:.0} kg) · {:.0} km · ~{} min",
                job.company_name,
                job.cargo_type,
                job.cargo_mass_kg,
                job.distance_km,
                job.estimated_duration_minutes
            ),
            fields: vec![
                (
                    "Reward".to_string(),
                    format!("€{} (profit est. €{})", job.total_reward, job.profit_estimate),
                ),
                (
                    "Rate".to_string(),
                    format!("€{:.2}/km", job.calculated_rate_per_km),
                ),
                (
                    "Urgency / Difficulty".to_string(),
                    format!("{} / {}", job.urgency_level, job.difficulty_level),
                ),
                (
                    "Equipment".to_string(),
                    format!(
                        "{}{}",
                        job.equipment_type_required,
                        job.trailer_type_required
                            .as_deref()
                            .map(|trailer| format!(" ({trailer})"))
                            .unwrap_or_default()
                    ),
                ),
                (
                    "Status".to_string(),
                    format!(
                        "{} ({})",
                        job.status,
                        job.ets2_job_link_status.as_deref().unwrap_or("-")
                    ),
                ),
                (
                    "Expires".to_string(),
                    job.expires_at_utc.as_deref().unwrap_or("-").to_string(),
                ),
            ],
            risk: job.risk_note.clone(),
            bonus: job.bonus_note.clone(),
            last_error: job.last_error_message.clone(),
            status: format!(
                "{} ({})",
                job.status,
                job.ets2_job_link_status.as_deref().unwrap_or("-")
            ),
        };
        JobRow {
            id: job.id.clone(),
            cells: vec![
                job.company_name.clone(),
                format!(
                    "{} ({}) → {} ({})",
                    job.origin_city,
                    job.origin_country,
                    job.destination_city,
                    job.destination_country
                ),
                format!("{} ({:.0} kg)", job.cargo_type, job.cargo_mass_kg),
                format!("{:.0} km", job.distance_km),
                format!("€{:.2}", job.calculated_rate_per_km),
                format!("€{}", job.total_reward),
                status.clone(),
            ],
            reward: job.total_reward,
            distance: job.distance_km,
            rate: job.calculated_rate_per_km,
            detail,
        }
    }};
}

// -------------------------------------------------------------- pending work

#[derive(Debug, Clone)]
enum Operation {
    RefreshLive,
    RefreshSnapshot,
    RefreshDispatcher,
    RefreshMarket,
    RefreshActive,
    RefreshLogbook,
    RefreshTrips,
    RefreshJobLog,
    RefreshStats,
    RefreshAnalytics,
    RefreshAccount,
    GenerateStandard,
    GenerateUniversal { force: bool },
    CleanupExpired,
    Accept { id: String },
    Cancel { id: String },
    AssignWrite { id: String },
    ScanHistory,
    ExportCsv,
    Login { email: String, password: String, remember: bool },
    Logout,
    CreateCompany { name: String, location: String },
}

#[derive(Debug, Clone)]
struct Confirm {
    title: &'static str,
    message: String,
    confirm_text: &'static str,
    action: Operation,
}

/// One persistent state instance per navigation tile.
#[derive(Default)]
pub struct ScreenState {
    pending: Option<Operation>,
    confirmed: Option<Operation>,
    confirm: Option<Confirm>,
    loading: Option<String>,
    validation: Option<String>,
    last_live_refresh: Option<Instant>,

    live_loaded: bool,
    dispatcher_loaded: bool,
    logbook_loaded: bool,
    account_loaded: bool,

    status: Option<StatusView>,
    status_error: Option<String>,
    overview: Option<OverviewView>,
    overview_error: Option<String>,
    snapshot: Option<String>,
    snapshot_error: Option<String>,

    dispatcher_overview: Option<String>,
    dispatcher_overview_error: Option<String>,
    generation: Option<GenerationView>,
    generation_error: Option<String>,
    market_jobs: Vec<JobRow>,
    market_loaded: bool,
    market_error: Option<String>,
    active_jobs: Vec<JobRow>,
    active_loaded: bool,
    active_error: Option<String>,
    job_source_active: bool,
    job_search: String,
    job_sort_label: String,
    job_sort_asc: bool,
    selected_job_id: Option<String>,
    force_universal: bool,
    job_action_error: Option<String>,
    job_action_message: Option<String>,

    trips: Vec<Vec<String>>,
    trips_loaded: bool,
    trips_error: Option<String>,
    job_log: Vec<Vec<String>>,
    job_log_loaded: bool,
    job_log_error: Option<String>,
    job_stats: Option<String>,
    job_stats_error: Option<String>,
    analytics: Option<String>,
    analytics_error: Option<String>,
    scan: Option<ScanView>,
    scan_error: Option<String>,
    export_message: Option<String>,
    export_error: Option<String>,

    user_profile: Option<String>,
    user_error: Option<String>,
    company: Option<String>,
    company_error: Option<String>,
    members: Vec<Vec<String>>,
    members_loaded: bool,
    members_error: Option<String>,
    auth_email: String,
    auth_password: String,
    auth_remember: bool,
    auth_message: Option<String>,
    new_company_name: String,
    new_company_location: String,
    company_message: Option<String>,
}

// ------------------------------------------------------------------ feedback

fn toast(ctx: &mut ScreenCtx<'_>, message: String, is_error: bool) {
    ctx.commands.push(ShellCommand::Toast { message, is_error });
}

fn error(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, message, true);
}

fn info(ctx: &mut ScreenCtx<'_>, message: String) {
    toast(ctx, message, false);
}

/// Errors go to a toast, the Errors drawer and the inline error card.
fn record_error(ctx: &mut ScreenCtx<'_>, slot: &mut Option<String>, err: UiError) {
    let message = err.message;
    if slot.as_deref() != Some(message.as_str()) {
        error(ctx, message.clone());
        ctx.commands
            .push(ShellCommand::OpenIssues(IssuesTab::Errors));
    }
    *slot = Some(message);
}

fn set_validation(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>, message: String) {
    state.validation = Some(message.clone());
    toast(ctx, format!("Validation: {message}"), false);
}

fn navigate(route: Route, sub_view: SubView, ctx: &mut ScreenCtx<'_>) {
    ctx.commands.push(ShellCommand::Navigate {
        route,
        sub_view: Some(sub_view),
    });
}

fn schedule(state: &mut ScreenState, op: Operation, label: &'static str) {
    state.loading = Some(label.to_string());
    state.pending = Some(op);
}

fn confirm_action(
    state: &mut ScreenState,
    title: &'static str,
    message: String,
    confirm_text: &'static str,
    action: Operation,
) {
    state.confirm = Some(Confirm {
        title,
        message,
        confirm_text,
        action,
    });
}

fn bool_label(value: bool) -> &'static str {
    if value {
        "Yes"
    } else {
        "No"
    }
}

fn optional(value: Option<&str>) -> String {
    value
        .filter(|value| !value.is_empty())
        .unwrap_or("-")
        .to_string()
}

// ------------------------------------------------------------------ execution

fn refresh_live(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match crate::gui::data::career::status(ctx.data) {
        Ok(status) => {
            state.status = Some(StatusView {
                games: format!(
                    "ETS2 running: {} | ATS running: {} | Active game: {}",
                    bool_label(status.ets2_running),
                    bool_label(status.ats_running),
                    optional(status.active_game.as_deref())
                ),
                bridge: format!(
                    "Telemetry active: {} | Bridge connected: {}",
                    bool_label(status.telemetry_running),
                    bool_label(status.bridge_connected)
                ),
                plugin: bool_label(status.plugin_installed).to_string(),
            });
            state.status_error = None;
        }
        Err(err) => record_error(ctx, &mut state.status_error, err),
    }
    match crate::gui::data::career::plugin_status(ctx.data) {
        Ok(installed) => {
            if let Some(view) = state.status.as_mut() {
                view.plugin = bool_label(installed).to_string();
            }
            state.plugin_error = None;
        }
        Err(err) => record_error(ctx, &mut state.plugin_error, err),
    }
    match crate::gui::data::career::overview(ctx.data) {
        Ok(overview) => {
            state.overview = Some(OverviewView {
                company: overview.economy.company_name.clone(),
                economy: format!(
                    "Cash: €{} | Debt: €{} | Rate: €{:.2}/km | Diesel: €{:.2}/L",
                    overview.bank.cash_balance,
                    overview.bank.debt_balance,
                    overview.economy.price_per_km,
                    overview.economy.diesel_price_per_liter
                ),
                reputation: format!(
                    "{} (Lv {} — {} pts) | Completed jobs: {}",
                    overview.reputation.label,
                    overview.reputation.level,
                    overview.reputation.xp_points,
                    overview.reputation.completed_jobs
                ),
                fleet: format!(
                    "{} assets ({} trucks, {} trailers, avg cond {:.1}%)",
                    overview.fleet_overview.total_assets,
                    overview.fleet_overview.trucks,
                    overview.fleet_overview.trailers,
                    overview.fleet_overview.avg_condition
                ),
                trip: overview
                    .active_trip
                    .as_ref()
                    .map(|trip| {
                        format!(
                            "Active trip #{}: {} → {} ({}, {:.1} km, avg {:.1} km/h)",
                            trip.trip_id,
                            trip.origin,
                            trip.destination,
                            trip.cargo,
                            trip.distance_km,
                            trip.avg_speed_kph
                        )
                    })
                    .unwrap_or_default(),
                telemetry: overview
                    .last_telemetry
                    .as_ref()
                    .map(|tel| {
                        format!(
                            "{:.1} km/h | {:.0} RPM | Gear {} | Fuel {:.1}/{:.1} L | Engine {} | Paused: {}",
                            tel.speed_kph,
                            tel.rpm,
                            tel.gear,
                            tel.fuel_liters,
                            tel.fuel_capacity_liters,
                            if tel.engine_on { "ON" } else { "OFF" },
                            bool_label(tel.paused)
                        )
                    })
                    .unwrap_or_default(),
            });
            state.overview_error = None;
        }
        Err(err) => record_error(ctx, &mut state.overview_error, err),
    }
}

fn refresh_snapshot(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match crate::gui::data::career::snapshot_refresh_active_save(ctx.data) {
        Ok(snapshot) => {
            state.snapshot = Some(format!(
                "Snapshot {} captured {} · checksum {} · {} depots · {} visited cities · {} cargo tokens",
                snapshot.save_session_id,
                snapshot.captured_at_utc,
                snapshot.checksum,
                snapshot.depots.len(),
                snapshot.visited_city_tokens.len(),
                snapshot.transported_cargo_tokens.len()
            ));
            state.snapshot_error = None;
            info(ctx, "Active save snapshot refreshed.".to_string());
        }
        Err(err) => record_error(ctx, &mut state.snapshot_error, err),
    }
}

fn refresh_dispatcher(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match crate::gui::data::career::dispatcher_overview(ctx.data) {
        Ok(view) => {
            state.dispatcher_overview = Some(format!(
                "Open market jobs: {} | Active jobs: {} | Open offers: {} | Accepted contracts: {}",
                view.open_market_jobs,
                view.active_jobs,
                view.open_offers,
                view.accepted_contracts
            ));
            state.dispatcher_overview_error = None;
        }
        Err(err) => record_error(ctx, &mut state.dispatcher_overview_error, err),
    }
    match crate::gui::data::career::generation_status(ctx.data) {
        Ok(status) => {
            state.generation = Some(GenerationView {
                counts: format!(
                    "Open jobs: {} | Generated: {} | Max open jobs: {}",
                    status.open_total_jobs, status.open_generated_jobs, status.max_open_jobs
                ),
                timing: format!(
                    "Interval: {} min | Last generated: {} | Last cleanup: {} | Next: {}",
                    status.interval_minutes,
                    optional(status.last_generated_at_utc.as_deref()),
                    optional(status.last_cleanup_at_utc.as_deref()),
                    optional(status.next_generation_at_utc.as_deref())
                ),
                context: format!("{:?}", status.current_context),
                link: bool_label(status.save_link_active).to_string(),
            });
            state.generation_error = None;
        }
        Err(err) => record_error(ctx, &mut state.generation_error, err),
    }
    match crate::gui::data::career::market_jobs(ctx.data, None) {
        Ok(jobs) => {
            state.market_jobs = jobs.iter().map(|job| job_row!(job)).collect();
            state.market_loaded = true;
            state.market_error = None;
        }
        Err(err) => record_error(ctx, &mut state.market_error, err),
    }
    match crate::gui::data::career::active_jobs(ctx.data) {
        Ok(jobs) => {
            state.active_jobs = jobs.iter().map(|job| job_row!(job)).collect();
            state.active_loaded = true;
            state.active_error = None;
        }
        Err(err) => record_error(ctx, &mut state.active_error, err),
    }
}

fn refresh_logbook(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match crate::gui::data::career::list_trips(ctx.data) {
        Ok(trips) => {
            state.trips = trips
                .iter()
                .map(|trip| {
                    vec![
                        format!("#{}", trip.id),
                        format!(
                            "{} → {}",
                            optional(trip.origin.as_deref()),
                            optional(trip.destination.as_deref())
                        ),
                        optional(trip.cargo.as_deref()),
                        format!("{:.1} km", trip.distance_km),
                        format!("{:.1} / {:.1} km/h", trip.avg_speed_kph, trip.max_speed_kph),
                        format!("€{}", trip.income.unwrap_or(0)),
                        trip.status.clone(),
                    ]
                })
                .collect();
            state.trips_loaded = true;
            state.trips_error = None;
        }
        Err(err) => record_error(ctx, &mut state.trips_error, err),
    }
    match crate::gui::data::career::job_log(ctx.data) {
        Ok(log) => {
            state.job_log = log
                .iter()
                .map(|entry| {
                    vec![
                        entry.job_id.clone(),
                        format!(
                            "{} → {}",
                            entry.origin_city, entry.destination_city
                        ),
                        entry.cargo.clone(),
                        format!("{:.1} km", entry.planned_distance_km),
                        format!("€{}", entry.income),
                        entry.status.clone(),
                        format!("{:.1}%", entry.cargo_damage),
                    ]
                })
                .collect();
            state.job_log_loaded = true;
            state.job_log_error = None;
        }
        Err(err) => record_error(ctx, &mut state.job_log_error, err),
    }
    match crate::gui::data::career::job_stats(ctx.data) {
        Ok(stats) => {
            state.job_stats = Some(format!(
                "Jobs: {} | Completed: {} | Failed: {} | Cancelled: {} | Abandoned: {} | Income: €{} | Avg distance: {:.1} km | Success: {:.1}%",
                stats.total_jobs,
                stats.completed_jobs,
                stats.failed_jobs,
                stats.cancelled_jobs,
                stats.abandoned_jobs,
                stats.total_income,
                stats.average_distance_km,
                stats.success_rate
            ));
            state.job_stats_error = None;
        }
        Err(err) => record_error(ctx, &mut state.job_stats_error, err),
    }
    match crate::gui::data::career::analytics_summary(ctx.data, None) {
        Ok(summary) => {
            state.analytics = Some(format!(
                "Total jobs: {} | Revenue: €{} | Costs: €{} | Net profit: €{} | Distance: {:.1} km | Avg profit/job: €{:.2} | Damaged deliveries: {} | Partial data: {}",
                summary.total_jobs,
                summary.total_revenue.unwrap_or(0),
                summary.total_costs.unwrap_or(0),
                summary.total_profit.unwrap_or(0),
                summary.total_distance_km.unwrap_or(0.0),
                summary.average_profit_per_job.unwrap_or(0.0),
                summary.damaged_deliveries,
                bool_label(summary.partial_data)
            ));
            state.analytics_error = None;
        }
        Err(err) => record_error(ctx, &mut state.analytics_error, err),
    }
}

fn refresh_account(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match crate::gui::data::vtc::current_user_profile(ctx.data) {
        Ok(profile) => {
            state.user_profile = Some(format!(
                "Signed in as: {} (User ID #{}) | Role: {} | Language: {}",
                profile.username,
                profile.user_id,
                optional(profile.company_role.as_deref()),
                profile.language
            ));
            state.user_error = None;
        }
        Err(err) => record_error(ctx, &mut state.user_error, err),
    }
    match crate::gui::data::vtc::company_overview(ctx.data) {
        Ok(company) => {
            state.company = Some(format!(
                "Company: {} (ID #{}) | HQ: {} | Members: {} | Visibility: {}",
                company.name,
                company.id,
                company.location,
                company.members_count,
                if company.public_visibility {
                    "Public"
                } else {
                    "Private"
                }
            ));
            state.company_error = None;
        }
        Err(err) => record_error(ctx, &mut state.company_error, err),
    }
    match crate::gui::data::vtc::company_members(ctx.data) {
        Ok(members) => {
            state.members = members
                .iter()
                .map(|member| {
                    vec![
                        member.username.clone(),
                        member.role_key.clone(),
                        member.joined_at.clone(),
                        optional(member.notes.as_deref()),
                    ]
                })
                .collect();
            state.members_loaded = true;
            state.members_error = None;
        }
        Err(err) => record_error(ctx, &mut state.members_error, err),
    }
}

fn execute(op: Operation, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    match op {
        Operation::RefreshLive => refresh_live(state, ctx),
        Operation::RefreshSnapshot => refresh_snapshot(state, ctx),
        Operation::RefreshDispatcher
        | Operation::RefreshMarket
        | Operation::RefreshActive => refresh_dispatcher(state, ctx),
        Operation::RefreshLogbook
        | Operation::RefreshTrips
        | Operation::RefreshJobLog
        | Operation::RefreshStats
        | Operation::RefreshAnalytics => refresh_logbook(state, ctx),
        Operation::RefreshAccount => refresh_account(state, ctx),
        Operation::GenerateStandard => {
            match crate::gui::data::career::generate_jobs(ctx.data) {
                Ok(status) => {
                    state.generation = Some(GenerationView {
                        counts: format!(
                            "Open jobs: {} | Generated: {} | Max open jobs: {}",
                            status.open_total_jobs,
                            status.open_generated_jobs,
                            status.max_open_jobs
                        ),
                        timing: format!(
                            "Interval: {} min | Last generated: {} | Next: {}",
                            status.interval_minutes,
                            optional(status.last_generated_at_utc.as_deref()),
                            optional(status.next_generation_at_utc.as_deref())
                        ),
                        context: format!("{:?}", status.current_context),
                        link: bool_label(status.save_link_active).to_string(),
                    });
                    state.generation_error = None;
                    info(
                        ctx,
                        format!(
                            "Market jobs: {} open ({} generated).",
                            status.open_total_jobs, status.open_generated_jobs
                        ),
                    );
                    schedule(state, Operation::RefreshDispatcher, "Refreshing dispatcher…");
                }
                Err(err) => record_error(ctx, &mut state.generation_error, err),
            }
        }
        Operation::GenerateUniversal { force } => {
            match crate::gui::data::career::generate_universal_jobs(ctx.data, Some(force), None) {
                Ok(status) => {
                    let message = format!(
                        "Market jobs: {} open ({} generated).",
                        status.open_total_jobs, status.open_generated_jobs
                    );
                    state.generation = Some(GenerationView {
                        counts: format!(
                            "Open jobs: {} | Generated: {} | Max open jobs: {}",
                            status.open_total_jobs,
                            status.open_generated_jobs,
                            status.max_open_jobs
                        ),
                        timing: format!(
                            "Interval: {} min | Last generated: {} | Next: {}",
                            status.interval_minutes,
                            optional(status.last_generated_at_utc.as_deref()),
                            optional(status.next_generation_at_utc.as_deref())
                        ),
                        context: format!("{:?}", status.current_context),
                        link: bool_label(status.save_link_active).to_string(),
                    });
                    state.generation_error = None;
                    info(ctx, message);
                    schedule(state, Operation::RefreshDispatcher, "Refreshing dispatcher…");
                }
                Err(err) => record_error(ctx, &mut state.generation_error, err),
            }
        }
        Operation::CleanupExpired => {
            match crate::gui::data::career::cleanup_expired_jobs(ctx.data) {
                Ok(status) => {
                    state.generation = Some(GenerationView {
                        counts: format!(
                            "Open jobs: {} | Generated: {} | Max open jobs: {}",
                            status.open_total_jobs,
                            status.open_generated_jobs,
                            status.max_open_jobs
                        ),
                        timing: format!(
                            "Interval: {} min | Last cleanup: {}",
                            status.interval_minutes,
                            optional(status.last_cleanup_at_utc.as_deref())
                        ),
                        context: format!("{:?}", status.current_context),
                        link: bool_label(status.save_link_active).to_string(),
                    });
                    state.generation_error = None;
                    info(ctx, "Ran expired-job cleanup.".to_string());
                    schedule(state, Operation::RefreshDispatcher, "Refreshing dispatcher…");
                }
                Err(err) => record_error(ctx, &mut state.generation_error, err),
            }
        }
        Operation::Accept { id } => {
            match crate::gui::data::career::accept_job(ctx.data, id.clone()) {
                Ok(details) => {
                    state.selected_job_id = Some(id.clone());
                    state.job_action_message =
                        Some(format!("Accepted job {}.", details.job.id));
                    state.job_action_error = None;
                    info(ctx, format!("Accepted job {id}."));
                    schedule(state, Operation::RefreshDispatcher, "Refreshing dispatcher…");
                }
                Err(err) => record_error(ctx, &mut state.job_action_error, err),
            }
        }
        Operation::Cancel { id } => {
            match crate::gui::data::career::cancel_job(ctx.data, id.clone()) {
                Ok(details) => {
                    state.job_action_message =
                        Some(format!("Cancelled job {}.", details.job.id));
                    state.job_action_error = None;
                    info(ctx, format!("Cancelled job {id}."));
                    schedule(state, Operation::RefreshDispatcher, "Refreshing dispatcher…");
                }
                Err(err) => record_error(ctx, &mut state.job_action_error, err),
            }
        }
        Operation::AssignWrite { id } => {
            match crate::gui::data::career::assign_prepare_write(ctx.data, id.clone(), true) {
                Ok(dto) => {
                    let staged = crate::gui::data::session::is_staging_enabled(ctx.data);
                    let job_id = dto.job.job.id.clone();
                    let message = if dto.write_applied && staged {
                        format!(
                            "Job {job_id} staged in quicksave — press Save in the Safety Spine to write it to disk."
                        )
                    } else if dto.write_applied {
                        format!("Job {job_id} written to quicksave.")
                    } else {
                        format!(
                            "Job {job_id} was NOT written to quicksave (status: {}).",
                            dto.dispatcher_status
                        )
                    };
                    state.job_action_message = Some(message.clone());
                    if !dto.write_applied {
                        let err = UiError {
                            message: message.clone(),
                        };
                        record_error(ctx, &mut state.job_action_error, err);
                    } else {
                        state.job_action_error = None;
                        info(ctx, message);
                    }
                    schedule(state, Operation::RefreshDispatcher, "Refreshing dispatcher…");
                }
                Err(err) => record_error(ctx, &mut state.job_action_error, err),
            }
        }
        Operation::ScanHistory => {
            match crate::gui::data::career::scan_profile_job_history(ctx.data, None) {
                Ok(result) => {
                    state.scan = Some(ScanView {
                        summary: format!(
                            "Scanned {} save(s): {} jobs detected, {} inserted, {} updated.",
                            result.scanned_saves,
                            result.detected_jobs,
                            result.inserted_jobs,
                            result.updated_jobs
                        ),
                        note: result.note,
                    });
                    state.scan_error = None;
                    info(ctx, "Profile job history scan finished.".to_string());
                    schedule(state, Operation::RefreshLogbook, "Refreshing logbook…");
                }
                Err(err) => record_error(ctx, &mut state.scan_error, err),
            }
        }
        Operation::ExportCsv => {
            match crate::gui::data::career::export_analytics_csv(ctx.data, None) {
                Ok(Some(path)) => {
                    state.export_message = Some(format!("Exported analytics CSV to {path}"));
                    state.export_error = None;
                    info(ctx, state.export_message.clone().unwrap_or_default());
                }
                Ok(None) => {
                    state.export_message = Some("Analytics CSV export was cancelled.".to_string());
                }
                Err(err) => record_error(ctx, &mut state.export_error, err),
            }
        }
        Operation::Login { email, password, remember } => {
            match crate::gui::data::vtc::login(ctx.data, email, password, remember) {
                Ok(result) => {
                    state.auth_message = Some(format!("Signed in as {}.", result.user.username));
                    state.auth_email.clear();
                    state.auth_password.clear();
                    state.validation = None;
                    info(ctx, state.auth_message.clone().unwrap_or_default());
                    schedule(state, Operation::RefreshAccount, "Refreshing account…");
                }
                Err(err) => record_error(ctx, &mut state.user_error, err),
            }
        }
        Operation::Logout => {
            match crate::gui::data::vtc::logout(ctx.data) {
                Ok(()) => {
                    state.auth_message = Some("Signed out.".to_string());
                    state.validation = None;
                    info(ctx, "Signed out.".to_string());
                    schedule(state, Operation::RefreshAccount, "Refreshing account…");
                }
                Err(err) => record_error(ctx, &mut state.user_error, err),
            }
        }
        Operation::CreateCompany { name, location } => {
            let language = match crate::gui::data::context::current_language(ctx.data) {
                Ok(language) => language,
                Err(err) => {
                    record_error(ctx, &mut state.company_error, err);
                    "en".to_string()
                }
            };
            let game = match crate::gui::data::career::status(ctx.data) {
                Ok(status) => status.active_game.unwrap_or_else(|| "ets2".to_string()),
                Err(err) => {
                    record_error(ctx, &mut state.status_error, err);
                    "ets2".to_string()
                }
            };
            // `CreateCompanyInput` is deserializable; constructing it through
            // the typed `vtc::create_company` wrapper keeps the screen free
            // of direct backend-type imports.
            let input = serde_json::from_value(serde_json::json!({
                "name": name,
                "location": location,
                "language": language,
                "game": game,
                "description": null,
                "logoPath": null,
                "headerPath": null,
                "slogan": null,
                "accentColor": null,
                "publicVisibility": true
            }));
            let input = match input {
                Ok(input) => input,
                Err(err) => {
                    record_error(
                        ctx,
                        &mut state.company_error,
                        UiError {
                            message: format!("Invalid company details: {err}"),
                        },
                    );
                    return;
                }
            };
            match crate::gui::data::vtc::create_company(ctx.data, input) {
                Ok(_) => {
                    state.new_company_name.clear();
                    state.new_company_location.clear();
                    state.validation = None;
                    state.company_message = Some("Created local VTC company.".to_string());
                    info(ctx, "Created local VTC company.".to_string());
                    schedule(state, Operation::RefreshAccount, "Refreshing account…");
                }
                Err(err) => record_error(ctx, &mut state.company_error, err),
            }
        }
    }
}

// ------------------------------------------------------------ view validation

/// Shell-owned sub-view validation; an invalid deep link falls back to the
/// tile default instead of panicking (spec §1.3 rule 3).
fn normalized_sub_view(route: Route, view: SubView) -> SubView {
    match route {
        Route::operations_live_telemetry => match view {
            SubView::TelemetrySnapshot => view,
            _ => SubView::TelemetryLive,
        },
        Route::operations_dispatcher => match view {
            SubView::DispatcherJobDetail | SubView::DispatcherGeneration => view,
            _ => SubView::DispatcherJobs,
        },
        Route::operations_logbook_analytics => match view {
            SubView::LogbookJobLog | SubView::LogbookStats | SubView::LogbookAnalytics => view,
            _ => SubView::LogbookTrips,
        },
        Route::operations_vtc_account => match view {
            SubView::VtcCompany | SubView::VtcMembers => view,
            _ => SubView::VtcSignIn,
        },
        _ => SubView::None,
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


// ------------------------------------------------------------------ rendering

/// Exact screen contract: one persistent state instance and one render fn.
pub fn render(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    ui.ctx().request_repaint_after(Duration::from_millis(500));
    if state.job_sort_label.is_empty() {
        state.job_sort_label = "Reward".to_string();
    }

    if let Some(op) = state.pending.take() {
        execute(op, state, ctx);
    }
    if let Some(op) = state.confirmed.take() {
        execute(op, state, ctx);
    }

    if state.pending.is_none() {
        match ctx.route {
            Route::operations_live_telemetry => {
                let due = state
                    .last_live_refresh
                    .map_or(true, |instant| instant.elapsed() >= Duration::from_millis(500));
                if due {
                    state.last_live_refresh = Some(Instant::now());
                    schedule(state, Operation::RefreshLive, "Refreshing live telemetry…");
                }
            }
            Route::operations_dispatcher => {
                if !state.dispatcher_loaded {
                    state.dispatcher_loaded = true;
                    schedule(state, Operation::RefreshDispatcher, "Loading dispatcher…");
                }
            }
            Route::operations_logbook_analytics => {
                if !state.logbook_loaded {
                    state.logbook_loaded = true;
                    schedule(state, Operation::RefreshLogbook, "Loading logbook…");
                }
            }
            Route::operations_vtc_account => {
                if !state.account_loaded {
                    state.account_loaded = true;
                    schedule(state, Operation::RefreshAccount, "Loading account…");
                }
            }
            _ => {}
        }
    }

    ui.horizontal(|ui| {
        if comp::button(ui, "Open Issues").clicked() {
            ctx.commands
                .push(ShellCommand::OpenIssues(IssuesTab::Errors));
        }
        ui.label(
            egui::RichText::new("Dispatcher and career edits stage until Save; the backend creates a backup when Save runs.")
                .size(10.5)
                .color(theme::MUTED),
        );
    });
    comp::validation_message(ui, state.validation.as_deref().unwrap_or(""));

    if let Some(label) = state.loading.clone() {
        comp::loading_state(ui, &label);
    }

    let view = normalized_sub_view(ctx.route, ctx.sub_view);
    match ctx.route {
        Route::operations_live_telemetry => render_telemetry(ui, state, ctx, view),
        Route::operations_dispatcher => render_dispatcher(ui, state, ctx, view),
        Route::operations_logbook_analytics => render_logbook(ui, state, ctx, view),
        Route::operations_vtc_account => render_vtc(ui, state, ctx, view),
        _ => render_telemetry(ui, state, ctx, SubView::TelemetryLive),
    }

    render_confirm(ui, state);

    if state.pending.is_none() && state.confirmed.is_none() {
        state.loading = None;
    }
}

fn render_confirm(ui: &mut egui::Ui, state: &mut ScreenState) {
    let Some(confirm) = state.confirm.take() else {
        return;
    };
    if let Some(accepted) = comp::confirm_dialog(
        ui.ctx(),
        confirm.title,
        &confirm.message,
        confirm.confirm_text,
        "Cancel",
    ) {
        if accepted {
            state.confirmed = Some(confirm.action);
        }
    } else {
        state.confirm = Some(confirm);
    }
}

// -------------------------------------------------------------- live telemetry

fn render_telemetry(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
    view: SubView,
) {
    sub_view_nav(
        ui,
        ctx,
        Route::operations_live_telemetry,
        view,
        &[
            (SubView::TelemetryLive, "Live"),
            (SubView::TelemetrySnapshot, "Save snapshot"),
        ],
    );
    if view == SubView::TelemetrySnapshot {
        render_snapshot(ui, state, ctx);
        return;
    }

    comp::section_card(ui, "SCS Telemetry Bridge", |ui| {
        match &state.status {
            Some(status) => {
                ui.label(&status.games);
                ui.label(&status.bridge);
                ui.label(format!("SDK plugin installed: {}", status.plugin));
            }
            None if state.status_error.is_none() && state.loading.is_none() => {
                comp::empty_state(ui, "No bridge status yet", "The 500 ms refresh will populate it.");
            }
            None => {}
        }
        if let Some(message) = state.status_error.clone() {
            if comp::error_state(ui, &message, Some("Retry")) {
                schedule(state, Operation::RefreshLive, "Refreshing live telemetry…");
            }
        }
        if let Some(message) = state.plugin_error.clone() {
            if comp::error_state(ui, &message, Some("Retry")) {
                schedule(state, Operation::RefreshLive, "Refreshing plugin status…");
            }
        }
    });

    comp::section_card(ui, "Live telemetry sample", |ui| {
        match state.overview.as_ref().and_then(|overview| {
            if overview.telemetry.is_empty() {
                None
            } else {
                Some(overview.telemetry.as_str())
            }
        }) {
            Some(telemetry) => {
                ui.label(
                    egui::RichText::new(telemetry)
                        .size(14.0)
                        .strong()
                        .color(theme::SUCCESS),
                );
            }
            None if state.overview_error.is_none() && state.loading.is_none() => {
                comp::empty_state(
                    ui,
                    "No live telemetry sample received",
                    "Start ETS2/ATS on Windows with the SCS SDK plugin active.",
                );
            }
            None => {}
        }
        if let Some(message) = state.overview_error.clone() {
            if comp::error_state(ui, &message, Some("Retry")) {
                schedule(state, Operation::RefreshLive, "Refreshing overview…");
            }
        }
    });

    comp::section_card(ui, "Company economy & bank", |ui| {
        match &state.overview {
            Some(overview) => {
                ui.label(&overview.company);
                ui.label(&overview.economy);
                ui.label(&overview.reputation);
                ui.label(&overview.fleet);
                if !overview.trip.is_empty() {
                    ui.label(
                        egui::RichText::new(&overview.trip)
                            .color(theme::WARNING),
                    );
                }
            }
            None if state.overview_error.is_none() && state.loading.is_none() => {
                comp::empty_state(ui, "No career overview", "The 500 ms refresh will populate it.");
            }
            None => {}
        }
        if let Some(message) = state.overview_error.clone() {
            if comp::error_state(ui, &message, Some("Retry")) {
                schedule(state, Operation::RefreshLive, "Refreshing overview…");
            }
        }
        if comp::button(ui, "Refresh career now").clicked() {
            schedule(state, Operation::RefreshLive, "Refreshing live telemetry…");
        }
    });
}

fn render_snapshot(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Active save snapshot", |ui| {
        if let Some(snapshot) = &state.snapshot {
            ui.label(snapshot);
        } else if state.snapshot_error.is_none() && state.loading.is_none() {
            comp::empty_state(
                ui,
                "No snapshot loaded",
                "Refresh the active save snapshot to synchronise depots and cities.",
            );
        }
        if let Some(message) = state.snapshot_error.clone() {
            if comp::error_state(ui, &message, Some("Retry")) {
                schedule(state, Operation::RefreshSnapshot, "Refreshing snapshot…");
            }
        }
        if comp::button(ui, "Refresh active save snapshot").clicked() {
            schedule(state, Operation::RefreshSnapshot, "Refreshing snapshot…");
        }
    });
    let _ = ctx;
}

// ------------------------------------------------------------------ dispatcher

fn sorted_jobs(state: &ScreenState) -> Vec<JobRow> {
    let needle = state.job_search.trim().to_ascii_lowercase();
    let source = if state.job_source_active {
        &state.active_jobs
    } else {
        &state.market_jobs
    };
    let mut jobs: Vec<JobRow> = source
        .iter()
        .filter(|job| {
            if needle.is_empty() {
                return true;
            }
            job.cells
                .iter()
                .any(|cell| cell.to_ascii_lowercase().contains(&needle))
        })
        .cloned()
        .collect();
    jobs.sort_by(|left, right| {
        let ordering = match state.job_sort_label.as_str() {
            "Distance" => left
                .distance
                .partial_cmp(&right.distance)
                .unwrap_or(std::cmp::Ordering::Equal),
            "Rate/km" => left
                .rate
                .partial_cmp(&right.rate)
                .unwrap_or(std::cmp::Ordering::Equal),
            _ => left.reward.cmp(&right.reward),
        };
        if state.job_sort_asc {
            ordering
        } else {
            ordering.reverse()
        }
    });
    jobs
}

fn render_dispatcher(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
    view: SubView,
) {
    sub_view_nav(
        ui,
        ctx,
        Route::operations_dispatcher,
        view,
        &[
            (SubView::DispatcherJobs, "Jobs"),
            (SubView::DispatcherGeneration, "Generation"),
            (SubView::DispatcherJobDetail, "Job detail"),
        ],
    );
    match view {
        SubView::DispatcherGeneration => {
            render_generation(ui, state, ctx);
        }
        SubView::DispatcherJobDetail => {
            render_job_detail(ui, state, ctx);
        }
        _ => render_jobs(ui, state, ctx),
    }
}

fn render_generation(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Dispatcher overview", |ui| {
        match &state.dispatcher_overview {
            Some(overview) => {
                ui.label(overview);
            }
            None if state.dispatcher_overview_error.is_none() && state.loading.is_none() => {
                comp::empty_state(ui, "No dispatcher overview", "Refresh to load counts.");
            }
            None => {}
        }
        if let Some(message) = state.dispatcher_overview_error.clone() {
            if comp::error_state(ui, &message, Some("Retry")) {
                schedule(state, Operation::RefreshDispatcher, "Refreshing dispatcher…");
            }
        }
    });

    comp::section_card(ui, "Generation status", |ui| {
        match &state.generation {
            Some(generation) => {
                ui.label(
                    egui::RichText::new(&generation.counts)
                        .size(14.0)
                        .strong(),
                );
                ui.label(&generation.timing);
                ui.label(format!("Save link active: {}", generation.link));
                ui.label(
                    egui::RichText::new(&generation.context)
                        .size(10.5)
                        .color(theme::MUTED),
                );
            }
            None if state.generation_error.is_none() && state.loading.is_none() => {
                comp::empty_state(ui, "No generation status", "Refresh to load generation state.");
            }
            None => {}
        }
        if let Some(message) = state.generation_error.clone() {
            if comp::error_state(ui, &message, Some("Retry")) {
                schedule(state, Operation::RefreshDispatcher, "Refreshing dispatcher…");
            }
        }
    });

    comp::section_card(ui, "Generate jobs", |ui| {
        comp::toggle(
            ui,
            "career_force_universal",
            "Force save-synced universal generation",
            &mut state.force_universal,
        );
        if comp::button(ui, "Generate market jobs").clicked() {
            schedule(state, Operation::GenerateStandard, "Generating market jobs…");
        }
        if comp::button(ui, "Generate universal jobs").clicked() {
            let force = state.force_universal;
            schedule(
                state,
                Operation::GenerateUniversal { force },
                "Generating universal jobs…",
            );
        }
        if comp::danger_button(ui, "Cleanup expired jobs").clicked() {
            confirm_action(
                state,
                "Cleanup expired jobs",
                "Remove expired dispatcher jobs from the active save context? This change is staged until Save.",
                "Cleanup",
                Operation::CleanupExpired,
            );
        }
        comp::validation_message(ui, state.validation.as_deref().unwrap_or(""));
    });
    let _ = ctx;
}

fn render_jobs(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Market and active jobs", |ui| {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(!state.job_source_active, "Market jobs")
                .clicked()
            {
                state.job_source_active = false;
                state.selected_job_id = None;
            }
            if ui
                .selectable_label(state.job_source_active, "Active jobs")
                .clicked()
            {
                state.job_source_active = true;
                state.selected_job_id = None;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Search:");
            comp::text_input(ui, &mut state.job_search, "company, city or cargo");
            let options = [
                "Reward".to_string(),
                "Distance".to_string(),
                "Rate/km".to_string(),
            ];
            if comp::select(ui, "career_job_sort", &mut state.job_sort_label, &options) {
                state.validation = None;
            }
            comp::toggle(ui, "career_job_sort_asc", "Ascending", &mut state.job_sort_asc);
        });

        let error = if state.job_source_active {
            state.active_error.clone()
        } else {
            state.market_error.clone()
        };
        let source_loaded = if state.job_source_active {
            state.active_loaded
        } else {
            state.market_loaded
        };
        let retry = if state.job_source_active {
            Operation::RefreshActive
        } else {
            Operation::RefreshMarket
        };

        if let Some(message) = error {
            if comp::error_state(ui, &message, Some("Retry")) {
                schedule(state, retry, "Refreshing jobs…");
            }
        } else {
            let jobs = sorted_jobs(state);
            if jobs.is_empty() && source_loaded && state.loading.is_none() {
                comp::empty_state(
                    ui,
                    "No jobs match",
                    "Try another search, or generate new market jobs.",
                );
            } else if !jobs.is_empty() {
                let rows: Vec<Vec<String>> = jobs.iter().map(|job| job.cells.clone()).collect();
                let selected = state
                    .selected_job_id
                    .as_ref()
                    .and_then(|id| jobs.iter().position(|job| &job.id == id));
                if let Some(index) = comp::table(
                    ui,
                    "career_dispatcher_jobs",
                    &[
                        "Company", "Route", "Cargo", "Distance", "Rate/km", "Reward", "Status",
                    ],
                    &rows,
                    selected,
                ) {
                    state.selected_job_id = Some(jobs[index].id.clone());
                }
            }
        }
        if comp::button(ui, "Refresh jobs").clicked() {
            schedule(state, Operation::RefreshDispatcher, "Refreshing jobs…");
        }
        comp::validation_message(ui, state.validation.as_deref().unwrap_or(""));
    });
    render_job_detail(ui, state, ctx);
}

fn render_job_detail(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
) {
    let detail = state
        .selected_job_id
        .as_ref()
        .and_then(|id| {
            state
                .market_jobs
                .iter()
                .chain(state.active_jobs.iter())
                .find(|job| &job.id == id)
                .map(|job| job.detail.clone())
        });

    comp::section_card(ui, "Job detail", |ui| {
        match detail {
            Some(detail) => {
                ui.label(
                    egui::RichText::new(&detail.heading)
                        .size(16.0)
                        .strong(),
                );
                ui.label(&detail.summary);
                ui.add_space(5.0);
                egui::Grid::new("career_job_detail_grid")
                    .num_columns(2)
                    .spacing([20.0, 4.0])
                    .show(ui, |ui| {
                        for (label, value) in &detail.fields {
                            ui.label(label);
                            ui.label(value);
                            ui.end_row();
                        }
                    });
                if let Some(note) = &detail.risk {
                    ui.label(
                        egui::RichText::new(format!("Risk: {note}"))
                            .size(11.5)
                            .color(theme::WARNING),
                    );
                }
                if let Some(note) = &detail.bonus {
                    ui.label(
                        egui::RichText::new(format!("Bonus: {note}"))
                            .size(11.5)
                            .color(theme::SUCCESS),
                    );
                }
                if let Some(message) = &detail.last_error {
                    ui.label(
                        egui::RichText::new(format!("Last error: {message}"))
                            .size(11.5)
                            .color(theme::DANGER),
                    );
                }
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    if comp::button(ui, "Accept job").clicked() {
                        accept_selected(state, ctx);
                    }
                    if comp::button(ui, "Write to quicksave").clicked() {
                        write_selected(state);
                    }
                    if comp::danger_button(ui, "Cancel job").clicked() {
                        cancel_selected(state);
                    }
                    if crate::gui::data::session::is_staging_enabled(ctx.data)
                        && comp::button(ui, "Save staged edits").clicked()
                    {
                        ctx.commands.push(ShellCommand::Save);
                    }
                });
                if let Some(message) = &state.job_action_message {
                    ui.label(
                        egui::RichText::new(message)
                            .size(11.5)
                            .color(theme::SUCCESS),
                    );
                }
                if let Some(message) = state.job_action_error.clone() {
                    if comp::error_state(ui, &message, None) {
                        // Failure is already mirrored to Issues by record_error.
                    }
                }
            }
            None if state.loading.is_none() => {
                comp::empty_state(
                    ui,
                    "No job selected",
                    "Select a market or active job row to see its details and actions.",
                );
            }
            None => {}
        }
    });
}

fn selected_job_id(state: &ScreenState) -> Option<String> {
    state.selected_job_id.clone()
}

fn accept_selected(state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    if let Some(id) = selected_job_id(state) {
        schedule(state, Operation::Accept { id }, "Accepting job…");
    } else {
        set_validation(state, ctx, "Select a job before accepting it.".to_string());
    }
}

fn write_selected(state: &mut ScreenState) {
    if let Some(id) = selected_job_id(state) {
        confirm_action(
            state,
            "Write job to quicksave",
            "Assign, prepare and write this job to the active quicksave? With staging enabled the result stays in memory until you press Save.",
            "Write job",
            Operation::AssignWrite { id },
        );
    }
}

fn cancel_selected(state: &mut ScreenState) {
    if let Some(id) = selected_job_id(state) {
        confirm_action(
            state,
            "Cancel dispatcher job",
            "Cancel this dispatcher job? This changes the job record and is staged until Save.",
            "Cancel job",
            Operation::Cancel { id },
        );
    }
}

// ----------------------------------------------------------- logbook/analytics

fn render_logbook(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
    view: SubView,
) {
    sub_view_nav(
        ui,
        ctx,
        Route::operations_logbook_analytics,
        view,
        &[
            (SubView::LogbookTrips, "Trips"),
            (SubView::LogbookJobLog, "Job log"),
            (SubView::LogbookStats, "Stats"),
            (SubView::LogbookAnalytics, "Analytics"),
        ],
    );
    match view {
        SubView::LogbookJobLog => {
            comp::section_card(ui, "Job log", |ui| {
                if let Some(message) = state.job_log_error.clone() {
                    if comp::error_state(ui, &message, Some("Retry")) {
                        schedule(state, Operation::RefreshJobLog, "Refreshing job log…");
                    }
                } else if state.job_log.is_empty() && state.job_log_loaded && state.loading.is_none() {
                    comp::empty_state(ui, "No job log entries", "Completed jobs will appear here.");
                } else if !state.job_log.is_empty() {
                    comp::table(
                        ui,
                        "career_job_log",
                        &[
                            "Job", "Route", "Cargo", "Distance", "Income", "Status", "Damage",
                        ],
                        &state.job_log,
                        None,
                    );
                }
            });
        }
        SubView::LogbookStats => {
            comp::section_card(ui, "Job statistics", |ui| {
                match &state.job_stats {
                    Some(stats) => {
                        ui.label(stats);
                    }
                    None if state.job_stats_error.is_none() && state.loading.is_none() => {
                        comp::empty_state(ui, "No job statistics", "Refresh to load statistics.");
                    }
                    None => {}
                }
                if let Some(message) = state.job_stats_error.clone() {
                    if comp::error_state(ui, &message, Some("Retry")) {
                        schedule(state, Operation::RefreshStats, "Refreshing statistics…");
                    }
                }
            });
        }
        SubView::LogbookAnalytics => {
            comp::section_card(ui, "Analytics", |ui| {
                match &state.analytics {
                    Some(analytics) => {
                        ui.label(analytics);
                    }
                    None if state.analytics_error.is_none() && state.loading.is_none() => {
                        comp::empty_state(ui, "No analytics summary", "Refresh to load analytics.");
                    }
                    None => {}
                }
                if let Some(message) = state.analytics_error.clone() {
                    if comp::error_state(ui, &message, Some("Retry")) {
                        schedule(state, Operation::RefreshAnalytics, "Refreshing analytics…");
                    }
                }
                if comp::button(ui, "Scan profile save job history").clicked() {
                    schedule(state, Operation::ScanHistory, "Scanning profile job history…");
                }
                if comp::button(ui, "Export analytics CSV").clicked() {
                    schedule(state, Operation::ExportCsv, "Exporting analytics CSV…");
                }
                if let Some(scan) = &state.scan {
                    ui.label(&scan.summary);
                    if !scan.note.trim().is_empty() {
                        ui.label(
                            egui::RichText::new(&scan.note)
                                .size(11.0)
                                .color(theme::MUTED),
                        );
                    }
                }
                if let Some(message) = state.scan_error.clone() {
                    if comp::error_state(ui, &message, Some("Retry")) {
                        schedule(state, Operation::ScanHistory, "Retrying history scan…");
                    }
                }
                if let Some(message) = &state.export_message {
                    ui.label(
                        egui::RichText::new(message)
                            .size(11.5)
                            .color(theme::SUCCESS),
                    );
                }
                if let Some(message) = state.export_error.clone() {
                    if comp::error_state(ui, &message, Some("Retry")) {
                        schedule(state, Operation::ExportCsv, "Retrying CSV export…");
                    }
                }
                comp::validation_message(ui, state.validation.as_deref().unwrap_or(""));
            });
        }
        _ => {
            comp::section_card(ui, "Recorded trips", |ui| {
                if let Some(message) = state.trips_error.clone() {
                    if comp::error_state(ui, &message, Some("Retry")) {
                        schedule(state, Operation::RefreshTrips, "Refreshing trips…");
                    }
                } else if state.trips.is_empty() && state.trips_loaded && state.loading.is_none() {
                    comp::empty_state(ui, "No recorded trips", "Driven trips will appear here.");
                } else if !state.trips.is_empty() {
                    comp::table(
                        ui,
                        "career_trips",
                        &[
                            "Trip", "Route", "Cargo", "Distance", "Avg/Max", "Income", "Status",
                        ],
                        &state.trips,
                        None,
                    );
                }
            });
        }
    }
}

// ------------------------------------------------------------------- VTC/account

fn render_vtc(
    ui: &mut egui::Ui,
    state: &mut ScreenState,
    ctx: &mut ScreenCtx<'_>,
    view: SubView,
) {
    sub_view_nav(
        ui,
        ctx,
        Route::operations_vtc_account,
        view,
        &[
            (SubView::VtcSignIn, "Sign in"),
            (SubView::VtcCompany, "Company"),
            (SubView::VtcMembers, "Members"),
        ],
    );
    match view {
        SubView::VtcCompany => render_company(ui, state, ctx),
        SubView::VtcMembers => render_members(ui, state),
        _ => render_sign_in(ui, state, ctx),
    }
}

fn render_sign_in(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Local account & session", |ui| {
        match &state.user_profile {
            Some(profile) => {
                ui.label(profile);
                if comp::danger_button(ui, "Sign Out").clicked() {
                    confirm_action(
                        state,
                        "Sign out",
                        "Sign out of the local VTC account?",
                        "Sign out",
                        Operation::Logout,
                    );
                }
            }
            None if state.user_error.is_none() && state.loading.is_none() => {
                comp::empty_state(
                    ui,
                    "Not signed in",
                    "Sign in to the local VTC database below.",
                );
            }
            None => {}
        }
        if let Some(message) = state.user_error.clone() {
            if comp::error_state(ui, &message, Some("Retry")) {
                schedule(state, Operation::RefreshAccount, "Refreshing account…");
            }
        }
        if state.user_profile.is_none() {
            comp::form_field(ui, "Email", |ui| {
                comp::text_input(ui, &mut state.auth_email, "name@example.com");
            });
            comp::form_field(ui, "Password", |ui| {
                comp::text_input(ui, &mut state.auth_password, "Password");
            });
            comp::toggle(ui, "career_auth_remember", "Remember me", &mut state.auth_remember);
            if comp::button(ui, "Sign In").clicked() {
                if state.auth_email.trim().is_empty() || state.auth_password.is_empty() {
                    set_validation(
                        state,
                        ctx,
                        "Email and password are required to sign in.".to_string(),
                    );
                } else {
                    let email = state.auth_email.trim().to_string();
                    let password = state.auth_password.clone();
                    let remember = state.auth_remember;
                    schedule(
                        state,
                        Operation::Login { email, password, remember },
                        "Signing in…",
                    );
                }
            }
        }
        if let Some(message) = &state.auth_message {
            ui.label(
                egui::RichText::new(message)
                    .size(11.5)
                    .color(theme::SUCCESS),
            );
        }
        comp::validation_message(ui, state.validation.as_deref().unwrap_or(""));
    });
}

fn render_company(ui: &mut egui::Ui, state: &mut ScreenState, ctx: &mut ScreenCtx<'_>) {
    comp::section_card(ui, "Virtual trucking company", |ui| {
        match &state.company {
            Some(company) => {
                ui.label(company);
            }
            None if state.company_error.is_none() && state.loading.is_none() => {
                comp::empty_state(
                    ui,
                    "No VTC company",
                    "Create a local VTC company with the form below.",
                );
            }
            None => {}
        }
        if let Some(message) = state.company_error.clone() {
            if comp::error_state(ui, &message, Some("Retry")) {
                schedule(state, Operation::RefreshAccount, "Refreshing company…");
            }
        }
        if state.company.is_none() {
            comp::form_field(ui, "Company name", |ui| {
                comp::text_input(ui, &mut state.new_company_name, "Company name");
            });
            comp::form_field(ui, "HQ city", |ui| {
                comp::text_input(ui, &mut state.new_company_location, "HQ city");
            });
            if comp::button(ui, "Create VTC company").clicked() {
                if state.new_company_name.trim().is_empty()
                    || state.new_company_location.trim().is_empty()
                {
                    set_validation(
                        state,
                        ctx,
                        "Company name and HQ city are required.".to_string(),
                    );
                } else {
                    let name = state.new_company_name.trim().to_string();
                    let location = state.new_company_location.trim().to_string();
                    schedule(
                        state,
                        Operation::CreateCompany { name, location },
                        "Creating company…",
                    );
                }
            }
        }
        if let Some(message) = &state.company_message {
            ui.label(
                egui::RichText::new(message)
                    .size(11.5)
                    .color(theme::SUCCESS),
            );
        }
        comp::validation_message(ui, state.validation.as_deref().unwrap_or(""));
    });
}

fn render_members(ui: &mut egui::Ui, state: &mut ScreenState) {
    comp::section_card(ui, "Company members", |ui| {
        if let Some(message) = state.members_error.clone() {
            if comp::error_state(ui, &message, Some("Retry")) {
                schedule(state, Operation::RefreshAccount, "Refreshing members…");
            }
        } else if state.members.is_empty() && state.members_loaded && state.loading.is_none() {
            comp::empty_state(ui, "No company members", "Members appear here after joining.");
        } else if !state.members.is_empty() {
            comp::table(
                ui,
                "career_company_members",
                &["Member", "Role", "Joined", "Notes"],
                &state.members,
                None,
            );
        }
    });
}
