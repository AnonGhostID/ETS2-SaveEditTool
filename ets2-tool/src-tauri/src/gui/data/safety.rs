//! Profile clone/rename/copy, sharing, save health and backups
//! (audit rows 31 and 119-134).

use crate::features::backup::models::{
    BackupRestorePreviewDto, BackupRestoreResultDto, BackupVersionDto,
};
use crate::features::health_monitor::models::{SaveHealthFixResultDto, SaveHealthReportDto};
use crate::features::profile_sharing::models::{
    ProfileShareContext, ProfileShareExportResult, ProfileShareImportPreview,
    ProfileShareImportResult,
};
use crate::models::clone_profiles_info::CloneTargetStatus;
use crate::state::{AppProfileState, DecryptCache, ProfileCache};

use super::{map_error, DataContext, UiError};

pub fn validate_clone_target(
    _ctx: &DataContext,
    source_profile: String,
    new_name: String,
) -> Result<CloneTargetStatus, UiError> {
    map_error(crate::features::profile_clone::commands::validate_clone_target(
        source_profile,
        new_name,
    ))
}

pub fn clone_profile(
    _ctx: &DataContext,
    source_profile: String,
    new_name: String,
    backup: bool,
    replace_hex: bool,
    replace_text: bool,
) -> Result<String, UiError> {
    map_error(crate::features::profile_clone::commands::clone_profile_command(
        source_profile,
        new_name,
        backup,
        replace_hex,
        replace_text,
    ))
}

pub fn rename_profile(ctx: &DataContext, new_name: String) -> Result<String, UiError> {
    map_error(crate::features::profile_rename::commands::profile_rename(
        new_name,
        ctx.app.state::<AppProfileState>(),
    ))
}

pub fn copy_controls(
    _ctx: &DataContext,
    source_profile_path: String,
    target_profile_path: String,
) -> Result<String, UiError> {
    map_error(
        crate::features::profile_controls::commands::copy_profile_controls(
            source_profile_path,
            target_profile_path,
        ),
    )
}

pub fn copy_mods(ctx: &DataContext, target_profile_path: String) -> Result<String, UiError> {
    map_error(crate::features::profile_move_mods::commands::copy_mods_to_profile(
        target_profile_path,
        ctx.app.state::<AppProfileState>(),
    ))
}

pub fn share_context(
    ctx: &DataContext,
    profile_path: Option<String>,
) -> Result<ProfileShareContext, UiError> {
    map_error(
        crate::features::profile_sharing::commands::get_profile_share_context(
            profile_path,
            ctx.app.state::<AppProfileState>(),
        ),
    )
}

pub fn pick_export_directory(ctx: &DataContext) -> Result<Option<String>, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::profile_sharing::commands::pick_shared_profile_export_directory(
            ctx.app.clone(),
            ctx.app.state::<AppProfileState>(),
        ),
    ))
}

pub fn export_shared_profile(
    ctx: &DataContext,
    profile_path: String,
    export_dir_override: Option<String>,
) -> Result<ProfileShareExportResult, UiError> {
    map_error(
        crate::features::profile_sharing::commands::export_shared_profile(
            profile_path,
            export_dir_override,
            ctx.app.state::<AppProfileState>(),
        ),
    )
}

pub fn pick_import_archive(ctx: &DataContext) -> Result<Option<String>, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::profile_sharing::commands::pick_shared_profile_import_archive(
            ctx.app.clone(),
            ctx.app.state::<AppProfileState>(),
        ),
    ))
}

pub fn inspect_archive(
    ctx: &DataContext,
    archive_path: String,
    profile_name_override: Option<String>,
) -> Result<ProfileShareImportPreview, UiError> {
    map_error(
        crate::features::profile_sharing::commands::inspect_shared_profile_archive(
            archive_path,
            profile_name_override,
            ctx.app.state::<AppProfileState>(),
        ),
    )
}

pub fn import_shared_profile(
    ctx: &DataContext,
    archive_path: String,
    profile_name_override: Option<String>,
) -> Result<ProfileShareImportResult, UiError> {
    map_error(
        crate::features::profile_sharing::commands::import_shared_profile(
            archive_path,
            profile_name_override,
            ctx.app.state::<AppProfileState>(),
        ),
    )
}

/// Audit rows 31 and 130 share this backend call.
pub fn scan_save_health(ctx: &DataContext) -> Result<SaveHealthReportDto, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::health_monitor::commands::get_active_save_health(
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
        ),
    ))
}

pub fn apply_health_fix(
    ctx: &DataContext,
    fix_id: String,
    confirmed: bool,
) -> Result<SaveHealthFixResultDto, UiError> {
    map_error(crate::features::health_monitor::commands::apply_save_health_fix(
        fix_id,
        confirmed,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
    ))
}

pub fn list_backups(ctx: &DataContext) -> Result<Vec<BackupVersionDto>, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::backup::commands::list_active_save_backups(
            ctx.app.state::<AppProfileState>(),
        ),
    ))
}

pub fn preview_backup(
    ctx: &DataContext,
    backup_id: String,
) -> Result<BackupRestorePreviewDto, UiError> {
    map_error(crate::features::backup::commands::preview_backup_restore(
        backup_id,
        ctx.app.state::<AppProfileState>(),
    ))
}

pub fn restore_backup(
    ctx: &DataContext,
    backup_id: String,
    confirmed: bool,
) -> Result<BackupRestoreResultDto, UiError> {
    map_error(crate::features::backup::commands::restore_backup(
        backup_id,
        confirmed,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
    ))
}
