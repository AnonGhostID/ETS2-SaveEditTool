//! Mod manager, presets, sandboxes and conflict analysis wrappers
//! (audit rows 98-118).

use crate::features::mod_profile_manager::models::{
    ApplySandboxResult, ModPreset, ModProfileManagerState, PresetCompareResult, SandboxCollection,
};
use crate::features::save_analysis::models::ModConflictAnalysisReport;
use crate::state::{AppProfileState, DecryptCache};

use super::{map_error, DataContext, UiError};

pub fn load_manager_state(
    ctx: &DataContext,
    game: Option<String>,
) -> Result<ModProfileManagerState, UiError> {
    map_error(
        crate::features::mod_profile_manager::commands::load_mod_profile_manager_state(
            ctx.app.clone(),
            ctx.app.state::<AppProfileState>(),
            game,
        ),
    )
}

pub fn open_steam_console(_ctx: &DataContext) -> Result<(), UiError> {
    map_error(crate::features::mod_profile_manager::commands::open_steam_console())
}

pub fn scan_mods_deep(
    ctx: &DataContext,
    game: Option<String>,
) -> Result<ModProfileManagerState, UiError> {
    map_error(crate::features::mod_profile_manager::commands::scan_mods_deep(
        ctx.app.clone(),
        ctx.app.state::<AppProfileState>(),
        game,
    ))
}

pub fn scan_mods_light(
    ctx: &DataContext,
    game: Option<String>,
) -> Result<ModProfileManagerState, UiError> {
    map_error(
        crate::features::mod_profile_manager::commands::scan_mods_light(
            ctx.app.clone(),
            ctx.app.state::<AppProfileState>(),
            game,
        ),
    )
}

pub fn delete_preset(ctx: &DataContext, preset_id: String) -> Result<(), UiError> {
    map_error(
        crate::features::mod_profile_manager::commands::delete_mod_preset(
            ctx.app.clone(),
            preset_id,
        ),
    )
}

pub fn delete_sandbox(ctx: &DataContext, sandbox_id: String) -> Result<SandboxCollection, UiError> {
    map_error(
        crate::features::mod_profile_manager::commands::delete_mod_sandbox(
            ctx.app.clone(),
            sandbox_id,
        ),
    )
}

pub fn list_presets(ctx: &DataContext, game: Option<String>) -> Result<Vec<ModPreset>, UiError> {
    map_error(crate::features::mod_profile_manager::commands::list_mod_presets(
        ctx.app.clone(),
        game,
    ))
}

pub fn choose_workshop_directory(
    ctx: &DataContext,
    game: String,
) -> Result<Option<String>, UiError> {
    map_error(
        crate::features::mod_profile_manager::commands::select_manual_workshop_directory(
            ctx.app.clone(),
            game,
        ),
    )
}

pub fn clear_workshop_directory(ctx: &DataContext, game: String) -> Result<(), UiError> {
    map_error(
        crate::features::mod_profile_manager::commands::clear_manual_workshop_directory(
            ctx.app.clone(),
            game,
        ),
    )
}

pub fn open_workshop_page(_ctx: &DataContext, mod_id: String) -> Result<(), UiError> {
    map_error(
        crate::features::mod_profile_manager::commands::open_workshop_page(mod_id),
    )
}

pub fn import_preset(ctx: &DataContext) -> Result<Option<ModPreset>, UiError> {
    map_error(crate::features::mod_profile_manager::commands::import_mod_preset(
        ctx.app.clone(),
    ))
}

pub fn create_preset(
    ctx: &DataContext,
    name: String,
    game: String,
    notes: Option<String>,
    preset_label: Option<String>,
) -> Result<ModPreset, UiError> {
    map_error(crate::features::mod_profile_manager::commands::create_mod_preset(
        ctx.app.clone(),
        ctx.app.state::<AppProfileState>(),
        name,
        game,
        notes,
        preset_label,
    ))
}

pub fn compare_preset(
    ctx: &DataContext,
    preset_id: String,
    game: String,
) -> Result<PresetCompareResult, UiError> {
    map_error(
        crate::features::mod_profile_manager::commands::compare_mod_preset(
            ctx.app.clone(),
            ctx.app.state::<AppProfileState>(),
            preset_id,
            game,
        ),
    )
}

pub fn export_preset(ctx: &DataContext, preset_id: String) -> Result<String, UiError> {
    map_error(crate::features::mod_profile_manager::commands::export_mod_preset(
        ctx.app.clone(),
        preset_id,
    ))
}

pub fn load_sandboxes(ctx: &DataContext) -> Result<SandboxCollection, UiError> {
    map_error(crate::features::mod_profile_manager::commands::load_mod_sandboxes(
        ctx.app.clone(),
    ))
}

pub fn create_sandbox(
    ctx: &DataContext,
    title: String,
    description: String,
) -> Result<SandboxCollection, UiError> {
    map_error(
        crate::features::mod_profile_manager::commands::create_mod_sandbox(
            ctx.app.clone(),
            title,
            description,
        ),
    )
}

pub fn apply_sandbox(
    ctx: &DataContext,
    sandbox_id: String,
    force_clear: bool,
) -> Result<ApplySandboxResult, UiError> {
    map_error(
        crate::features::mod_profile_manager::commands::apply_sandbox_to_active_profile(
            ctx.app.clone(),
            ctx.app.state::<AppProfileState>(),
            sandbox_id,
            force_clear,
        ),
    )
}

pub fn add_mod_to_sandbox(
    ctx: &DataContext,
    sandbox_id: String,
    workshop_input: String,
    manual_fallback: Option<bool>,
) -> Result<SandboxCollection, UiError> {
    map_error(
        crate::features::mod_profile_manager::commands::add_mod_to_sandbox(
            ctx.app.clone(),
            sandbox_id,
            workshop_input,
            manual_fallback,
        ),
    )
}

pub fn analyze_conflicts(ctx: &DataContext) -> Result<ModConflictAnalysisReport, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::save_analysis::commands::analyze_mod_conflict_diagnostics(
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
        ),
    ))
}

pub fn analyze_conflicts_deep(ctx: &DataContext) -> Result<ModConflictAnalysisReport, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::save_analysis::commands::analyze_mod_conflict_diagnostics_deep(
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
        ),
    ))
}

pub fn export_conflict_report(
    ctx: &DataContext,
    report: ModConflictAnalysisReport,
    formatted: Option<bool>,
) -> Result<Option<String>, UiError> {
    map_error(
        crate::features::save_analysis::commands::export_mod_conflict_diagnostics_report(
            ctx.app.clone(),
            report,
            formatted,
        ),
    )
}
