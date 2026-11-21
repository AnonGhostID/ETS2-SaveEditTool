//! Language, game/profile/save selection and save snapshot wrappers
//! (audit rows 1-26 and 33).

use crate::features::language::commands::LanguageInfo;
use crate::features::save_editor::commands::UndoStatusDto;
use crate::models::global_config_info::BaseGameConfig;
use crate::models::profile_info::ProfileInfo;
use crate::models::quicksave_game_info::{CurrentTruckSummary, GameDataQuicksave};
use crate::models::save_game_config::SaveGameConfig;
use crate::models::save_game_data::SaveGameData;
use crate::models::save_info::SaveInfo;
use crate::models::trailers::{ParsedTrailer, PlayerTrailerResult};
use crate::models::trucks::ParsedTruck;
use crate::state::{AppProfileState, DecryptCache, ProfileCache};
use crate::xp::command::LevelProgress;

use super::{map_error, ui_error, DataContext, UiError};

pub fn available_languages(ctx: &DataContext) -> Vec<LanguageInfo> {
    crate::features::language::commands::get_available_languages_command(ctx.app.clone())
}

pub fn current_language(ctx: &DataContext) -> String {
    crate::features::language::commands::get_current_language_command(ctx.app.clone())
}

pub fn set_game(ctx: &DataContext, game: String) -> Result<(), UiError> {
    map_error(crate::features::profile_manager::commands::set_selected_game(
        game,
        ctx.app.state::<AppProfileState>(),
    ))
    .map(|_| ())
}

/// Active game from the context column selector.
pub fn selected_game(ctx: &DataContext) -> Result<String, UiError> {
    let state = ctx.app.state::<AppProfileState>();
    state
        .selected_game
        .lock()
        .map(|game| game.clone())
        .map_err(|_| ui_error("AppProfileState selected_game lock poisoned"))
}

pub fn find_profiles(ctx: &DataContext) -> Vec<ProfileInfo> {
    crate::features::profile_manager::commands::find_ets2_profiles(
        ctx.app.state::<AppProfileState>(),
    )
}

/// Active profile path from `AppProfileState::current_profile`.
pub fn selected_profile_path(ctx: &DataContext) -> Result<String, UiError> {
    let state = ctx.app.state::<AppProfileState>();
    state
        .current_profile
        .lock()
        .map_err(|_| ui_error("AppProfileState current_profile lock poisoned"))?
        .clone()
        .ok_or_else(|| ui_error("No profile selected"))
}

pub fn read_last_profile(ctx: &DataContext) -> Result<Option<String>, UiError> {
    map_error(crate::features::profile_manager::commands::read_last_profile(
        ctx.app.clone(),
    ))
}

pub fn save_last_profile(ctx: &DataContext, profile_path: String) -> Result<(), UiError> {
    map_error(crate::features::profile_manager::commands::save_last_profile(
        ctx.app.clone(),
        profile_path,
    ))
    .map(|_| ())
}

pub fn set_active_profile(ctx: &DataContext, profile_path: String) -> Result<(), UiError> {
    map_error(crate::features::profile_manager::commands::set_active_profile(
        profile_path,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<DecryptCache>(),
        ctx.app.state::<ProfileCache>(),
    ))
}

pub fn find_saves(_ctx: &DataContext, profile_path: String) -> Result<Vec<SaveInfo>, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::profile_manager::commands::find_profile_saves(profile_path),
    ))
}

pub fn load_profile(
    ctx: &DataContext,
    profile_path: String,
    save_path: Option<String>,
) -> Result<(), UiError> {
    map_error(crate::features::profile_manager::commands::load_profile(
        profile_path,
        save_path,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<DecryptCache>(),
        ctx.app.state::<ProfileCache>(),
    ))
    .map(|_| ())
}

pub fn set_current_save(ctx: &DataContext, save_path: String) -> Result<(), UiError> {
    map_error(crate::features::profile_manager::commands::set_current_save(
        save_path,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<DecryptCache>(),
        ctx.app.state::<ProfileCache>(),
    ))
}

pub fn undo_status(ctx: &DataContext) -> Result<UndoStatusDto, UiError> {
    map_error(crate::features::save_editor::commands::get_undo_status(
        ctx.app.state::<AppProfileState>(),
    ))
}

pub fn read_base_config(ctx: &DataContext) -> Result<BaseGameConfig, UiError> {
    map_error(crate::features::settings::game_config::read_base_config(
        ctx.app.state::<ProfileCache>(),
    ))
}

pub fn read_save_config(ctx: &DataContext, profile_path: String) -> Result<SaveGameConfig, UiError> {
    map_error(crate::features::settings::game_config::read_save_config(
        &profile_path,
        ctx.app.state::<ProfileCache>(),
    ))
}

pub fn read_all_save_data(ctx: &DataContext) -> Result<SaveGameData, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::save_analysis::reader::read_all_save_data(
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<ProfileCache>(),
            ctx.app.state::<DecryptCache>(),
        ),
    ))
}

pub fn try_read_truck_xp(ctx: &DataContext) -> Result<Option<i64>, UiError> {
    map_error(crate::features::save_editor::commands::try_read_truck_xp(
        ctx.app.state::<AppProfileState>(),
    ))
}

pub fn quicksave_game_info(ctx: &DataContext) -> Result<GameDataQuicksave, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::save_analysis::quicksave::quicksave_game_info(
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<ProfileCache>(),
            ctx.app.state::<DecryptCache>(),
        ),
    ))
}

pub fn current_truck_summary(ctx: &DataContext) -> Result<Option<CurrentTruckSummary>, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::save_analysis::quicksave::get_current_truck_summary(
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<ProfileCache>(),
            ctx.app.state::<DecryptCache>(),
        ),
    ))
}

pub fn all_trucks(ctx: &DataContext, profile_path: String) -> Result<Vec<ParsedTruck>, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::vehicles::trucks::get_all_trucks(
            profile_path,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<ProfileCache>(),
            ctx.app.state::<DecryptCache>(),
        ),
    ))
}

pub fn player_truck(ctx: &DataContext, profile_path: String) -> Result<ParsedTruck, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::vehicles::trucks::get_player_truck(
            profile_path,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<ProfileCache>(),
            ctx.app.state::<DecryptCache>(),
        ),
    ))
}

pub fn all_trailers(ctx: &DataContext, profile_path: String) -> Result<Vec<ParsedTrailer>, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::vehicles::trailers::get_all_trailers(
            profile_path,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<ProfileCache>(),
            ctx.app.state::<DecryptCache>(),
        ),
    ))
}

pub fn player_trailer(
    ctx: &DataContext,
    profile_path: String,
) -> Result<PlayerTrailerResult, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::vehicles::trailers::get_player_trailer(
            profile_path,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<ProfileCache>(),
            ctx.app.state::<DecryptCache>(),
        ),
    ))
}

/// Pure level helper (audit rows 33, 97, 148). Kept in the data layer so
/// screens never import backend modules.
pub fn calculate_level(_ctx: &DataContext, total_xp: u64) -> LevelProgress {
    crate::xp::command::calculate_level(total_xp)
}

