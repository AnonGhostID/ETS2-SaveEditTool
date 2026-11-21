//! Data location, SQLite/datasets, logs and diagnostics wrappers
//! (audit rows 24-26 and 149-159).

use crate::db::sqlite::{SqliteInfoDto, SqliteTableCountsDto};
use crate::shared::ets2data::models::Ets2DataImportSummary;
use crate::shared::paths::Ets2DataLocationDto;
use crate::shared::user_log::{UserLogEntry, UserLogStatus};
use crate::state::{
    AppProfileState, AppState, CareerState, DecryptCache, DiagnosticsState, ProfileCache,
};

use super::{map_error, DataContext, UiError};

pub fn data_location(_ctx: &DataContext) -> Result<Ets2DataLocationDto, UiError> {
    map_error(crate::features::settings::ets2_paths::get_ets2_data_location())
}

pub fn sqlite_info(ctx: &DataContext) -> Result<SqliteInfoDto, UiError> {
    map_error(crate::tauri::async_runtime::block_on(crate::commands::get_sqlite_info(
        ctx.app.state::<AppState>(),
    )))
}

pub fn sqlite_table_counts(ctx: &DataContext) -> Result<SqliteTableCountsDto, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::commands::get_sqlite_table_counts(
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<AppState>(),
        ),
    ))
}

pub fn log_status(_ctx: &DataContext) -> Result<UserLogStatus, UiError> {
    map_error(crate::features::logging::commands::get_log_status())
}

/// Default, unfiltered log view used by the shell's Issues drawer.
pub fn user_logs(_ctx: &DataContext) -> Result<Vec<UserLogEntry>, UiError> {
    map_error(crate::features::logging::commands::get_user_logs(None, None))
}

/// Filtered log view used by the Logs & Diagnostics tile.
pub fn user_logs_filtered(
    _ctx: &DataContext,
    level_filter: Option<String>,
    max_lines: Option<u32>,
) -> Result<Vec<UserLogEntry>, UiError> {
    map_error(crate::features::logging::commands::get_user_logs(
        level_filter,
        max_lines,
    ))
}

pub fn log_preview(ctx: &DataContext) -> Result<String, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::logging::diagnostics::get_log_preview(
            ctx.app.state::<DiagnosticsState>(),
        ),
    ))
}

pub fn choose_data_location(
    ctx: &DataContext,
    dialog_title: Option<String>,
) -> Result<Option<Ets2DataLocationDto>, UiError> {
    map_error(crate::features::settings::ets2_paths::choose_ets2_data_location(
        ctx.app.clone(),
        dialog_title,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<DecryptCache>(),
        ctx.app.state::<ProfileCache>(),
    ))
}

pub fn reset_data_location(ctx: &DataContext) -> Result<Ets2DataLocationDto, UiError> {
    map_error(
        crate::features::settings::ets2_paths::reset_ets2_data_location(
            ctx.app.clone(),
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<ProfileCache>(),
        ),
    )
}

pub fn set_data_location_mode(
    ctx: &DataContext,
    mode: String,
    custom_homedir: Option<String>,
) -> Result<Ets2DataLocationDto, UiError> {
    map_error(
        crate::features::settings::ets2_paths::set_ets2_data_location_mode(
            ctx.app.clone(),
            mode,
            custom_homedir,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<ProfileCache>(),
        ),
    )
}

pub fn import_datasets(ctx: &DataContext, force: bool) -> Result<Ets2DataImportSummary, UiError> {
    map_error(crate::commands::data_import_ets2_datasets(
        force,
        ctx.app.clone(),
        ctx.app.state::<CareerState>(),
    ))
}

pub fn export_user_logs(ctx: &DataContext) -> Result<String, UiError> {
    map_error(crate::features::logging::commands::export_user_logs(
        ctx.app.clone(),
        ctx.app.state::<AppProfileState>(),
    ))
}

pub fn export_logs_bundle(ctx: &DataContext) -> Result<Option<String>, UiError> {
    map_error(crate::features::logging::commands::export_logs_bundle(
        ctx.app.clone(),
        ctx.app.state::<AppProfileState>(),
    ))
}

pub fn export_log_file(ctx: &DataContext) -> Result<Option<String>, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::logging::diagnostics::export_log_file(
            ctx.app.clone(),
            ctx.app.state::<DiagnosticsState>(),
        ),
    ))
}

pub fn clear_user_logs(_ctx: &DataContext) -> Result<Option<String>, UiError> {
    map_error(crate::features::logging::commands::clear_user_logs())
}
