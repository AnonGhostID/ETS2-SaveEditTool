//! Career status, telemetry, dispatcher, logbook and analytics wrappers
//! (audit rows 65-75 and 79-87).

use crate::features::career::analytics::{AnalyticsFilters, AnalyticsScanResult, AnalyticsSummary};
use crate::features::career::commands::DispatcherAssignPrepareWriteDto;
use crate::features::career::dispatcher::{
    DispatcherGenerationConfigInput, DispatcherGenerationStatus, DispatcherJobFilter,
    DispatcherMarketJob, DispatcherOverview, DispatcherJobDetails,
};
use crate::features::career::job_log::{JobLogEntry, JobStats};
use crate::features::career::logbook::TripSummary;
use crate::features::career::overview::CareerOverview;
use crate::features::ets2save::snapshot::SaveSnapshotDto;
use crate::features::hub::events::CareerStatus;
use crate::state::{AppProfileState, CareerState, EtsDbState};

use super::{map_error, ui_error, DataContext, UiError};

pub fn status(ctx: &DataContext) -> Result<CareerStatus, UiError> {
    map_error(crate::features::career::commands::career_get_status(
        ctx.app.state::<CareerState>(),
    ))
}

pub fn plugin_status(ctx: &DataContext) -> Result<bool, UiError> {
    map_error(crate::features::career::commands::get_plugin_status(
        ctx.app.state::<AppProfileState>(),
    ))
}

pub fn overview(ctx: &DataContext) -> Result<CareerOverview, UiError> {
    map_error(crate::features::career::commands::career_get_overview(
        ctx.app.state::<CareerState>(),
    ))
}

pub fn dispatcher_overview(ctx: &DataContext) -> Result<DispatcherOverview, UiError> {
    map_error(
        crate::features::career::commands::dispatcher_get_dispatcher_overview(
            ctx.app.state::<CareerState>(),
            ctx.app.state::<AppProfileState>(),
        ),
    )
}

pub fn generation_status(ctx: &DataContext) -> Result<DispatcherGenerationStatus, UiError> {
    map_error(
        crate::features::career::commands::dispatcher_get_generation_status(
            ctx.app.state::<CareerState>(),
            ctx.app.state::<AppProfileState>(),
        ),
    )
}

pub fn market_jobs(
    ctx: &DataContext,
    filter: Option<DispatcherJobFilter>,
) -> Result<Vec<DispatcherMarketJob>, UiError> {
    map_error(crate::features::career::commands::dispatcher_get_market_jobs(
        filter,
        ctx.app.state::<CareerState>(),
        ctx.app.state::<AppProfileState>(),
    ))
}

pub fn active_jobs(ctx: &DataContext) -> Result<Vec<DispatcherMarketJob>, UiError> {
    map_error(crate::features::career::commands::dispatcher_get_active_jobs(
        ctx.app.state::<CareerState>(),
        ctx.app.state::<AppProfileState>(),
    ))
}

pub fn list_trips(ctx: &DataContext) -> Result<Vec<TripSummary>, UiError> {
    map_error(crate::features::career::commands::career_list_trips(
        ctx.app.state::<CareerState>(),
    ))
}

pub fn job_log(ctx: &DataContext) -> Result<Vec<JobLogEntry>, UiError> {
    map_error(crate::features::career::commands::career_get_job_log(
        ctx.app.state::<CareerState>(),
    ))
}

pub fn job_stats(ctx: &DataContext) -> Result<JobStats, UiError> {
    map_error(crate::features::career::commands::career_get_job_stats(
        ctx.app.state::<CareerState>(),
    ))
}

pub fn analytics_summary(
    ctx: &DataContext,
    filters: Option<AnalyticsFilters>,
) -> Result<AnalyticsSummary, UiError> {
    map_error(
        crate::features::career::commands::career_get_analytics_summary(
            filters,
            ctx.app.state::<CareerState>(),
        ),
    )
}

pub fn generate_jobs(ctx: &DataContext) -> Result<DispatcherGenerationStatus, UiError> {
    map_error(crate::features::career::commands::dispatcher_generate_jobs(
        ctx.app.state::<CareerState>(),
        ctx.app.state::<AppProfileState>(),
    ))
}

pub fn generate_universal_jobs(
    ctx: &DataContext,
    force: Option<bool>,
    config: Option<DispatcherGenerationConfigInput>,
) -> Result<DispatcherGenerationStatus, UiError> {
    map_error(
        crate::features::career::commands::dispatcher_generate_universal_jobs(
            force,
            config,
            ctx.app.state::<CareerState>(),
            ctx.app.state::<AppProfileState>(),
        ),
    )
}

pub fn snapshot_refresh_active_save(ctx: &DataContext) -> Result<SaveSnapshotDto, UiError> {
    crate::tauri::async_runtime::block_on(
        crate::commands::ets_snapshot_refresh_active_save(
            ctx.app.clone(),
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<EtsDbState>(),
        ),
    )
    .map_err(|error| ui_error(error.to_string()))
}

pub fn cleanup_expired_jobs(ctx: &DataContext) -> Result<DispatcherGenerationStatus, UiError> {
    map_error(
        crate::features::career::commands::dispatcher_cleanup_expired_jobs(
            ctx.app.state::<CareerState>(),
            ctx.app.state::<AppProfileState>(),
        ),
    )
}

pub fn accept_job(ctx: &DataContext, job_id: String) -> Result<DispatcherJobDetails, UiError> {
    map_error(crate::features::career::commands::dispatcher_accept_job(
        job_id,
        ctx.app.state::<CareerState>(),
        ctx.app.state::<AppProfileState>(),
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn assign_prepare_write(
    ctx: &DataContext,
    job_id: String,
    auto_write: bool,
) -> Result<DispatcherAssignPrepareWriteDto, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::career::commands::dispatcher_assign_and_prepare_and_write(
            job_id,
            auto_write,
            ctx.app.clone(),
            ctx.app.state::<CareerState>(),
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<EtsDbState>(),
        ),
    ))
}

pub fn cancel_job(ctx: &DataContext, job_id: String) -> Result<DispatcherJobDetails, UiError> {
    map_error(crate::features::career::commands::dispatcher_cancel_job(
        job_id,
        ctx.app.state::<CareerState>(),
        ctx.app.state::<AppProfileState>(),
    ))
}

pub fn scan_profile_job_history(
    ctx: &DataContext,
    profile_path: Option<String>,
) -> Result<AnalyticsScanResult, UiError> {
    map_error(
        crate::features::career::commands::career_scan_profile_job_history(
            profile_path,
            ctx.app.state::<CareerState>(),
            ctx.app.state::<AppProfileState>(),
        ),
    )
}

pub fn export_analytics_csv(
    ctx: &DataContext,
    filters: Option<AnalyticsFilters>,
) -> Result<Option<String>, UiError> {
    map_error(
        crate::features::career::commands::career_export_analytics_csv(
            ctx.app.clone(),
            filters,
            ctx.app.state::<CareerState>(),
        ),
    )
}
