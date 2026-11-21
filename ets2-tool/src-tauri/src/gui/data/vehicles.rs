//! Vehicle reader/editor, switcher and powertrain wrappers
//! (audit rows 20-23 and 35-53).

use crate::features::trailer_change::cache::TrailerChangeSessionCache;
use crate::features::trailer_change::models::{
    ApplyTrailerChangeResult, TrailerChangePreview, TrailerChangeSession, TrailerSwitchList,
};
use crate::features::truck_change::cache::TruckChangeSessionCache;
use crate::features::truck_change::models::{
    ApplyTruckChangeResult, PowertrainCatalog, TruckChangePreview, TruckChangeSession,
    TruckPowertrainPreview, TruckSwitchList,
};
use crate::models::trailers::{ParsedTrailer, PlayerTrailerResult};
use crate::models::trucks::ParsedTruck;
use crate::state::{AppProfileState, DecryptCache, ProfileCache};

use super::{map_error, DataContext, UiError};

// ---------------------------------------------------------------------------
// Readers (audit rows 20-23; delegates to the shared context readers)
// ---------------------------------------------------------------------------

pub fn all_trucks(ctx: &DataContext, profile_path: String) -> Result<Vec<ParsedTruck>, UiError> {
    super::context::all_trucks(ctx, profile_path)
}

pub fn player_truck(ctx: &DataContext, profile_path: String) -> Result<ParsedTruck, UiError> {
    super::context::player_truck(ctx, profile_path)
}

pub fn all_trailers(ctx: &DataContext, profile_path: String) -> Result<Vec<ParsedTrailer>, UiError> {
    super::context::all_trailers(ctx, profile_path)
}

pub fn player_trailer(
    ctx: &DataContext,
    profile_path: String,
) -> Result<PlayerTrailerResult, UiError> {
    super::context::player_trailer(ctx, profile_path)
}

// ---------------------------------------------------------------------------
// Player truck / trailer edits (audit rows 35-43)
// ---------------------------------------------------------------------------

pub fn repair_truck(ctx: &DataContext) -> Result<(), UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::vehicles::editor::repair_player_truck(
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<ProfileCache>(),
        ),
    ))
}

pub fn refuel_truck(ctx: &DataContext) -> Result<(), UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::vehicles::editor::refuel_player_truck(
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<ProfileCache>(),
        ),
    ))
}

pub fn set_fuel(ctx: &DataContext, level: f32) -> Result<(), UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::vehicles::editor::set_player_truck_fuel(
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<ProfileCache>(),
            level,
        ),
    ))
}

pub fn set_wear(ctx: &DataContext, wear_type: String, level: f32) -> Result<(), UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::vehicles::editor::set_player_truck_wear(
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<ProfileCache>(),
            wear_type,
            level,
        ),
    ))
}

pub fn set_plate(ctx: &DataContext, plate: String) -> Result<(), UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::vehicles::editor::set_player_truck_license_plate(
            plate,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<ProfileCache>(),
        ),
    ))
}

pub fn set_odometer(ctx: &DataContext, value: i64) -> Result<(), UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::vehicles::editor::edit_truck_odometer(
            value,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<ProfileCache>(),
        ),
    ))
}

pub fn repair_trailer(ctx: &DataContext) -> Result<(), UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::vehicles::editor::repair_player_trailer(
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<ProfileCache>(),
        ),
    ))
}

pub fn set_trailer_plate(ctx: &DataContext, plate: String) -> Result<(), UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::vehicles::editor::set_player_trailer_license_plate(
            plate,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<ProfileCache>(),
        ),
    ))
}

pub fn set_cargo_mass(ctx: &DataContext, mass: f32) -> Result<(), UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::vehicles::editor::set_player_trailer_cargo_mass(
            mass,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<ProfileCache>(),
        ),
    ))
}

// ---------------------------------------------------------------------------
// Truck switcher (audit rows 44-47)
// ---------------------------------------------------------------------------

pub fn init_truck_switch(
    ctx: &DataContext,
    save_path: Option<String>,
) -> Result<TruckChangeSession, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::truck_change::commands::initialize_truck_change_session(
            save_path,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<TruckChangeSessionCache>(),
        ),
    ))
}

pub fn list_trucks_for_switch(
    ctx: &DataContext,
    save_path: Option<String>,
) -> Result<TruckSwitchList, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::truck_change::commands::list_owned_trucks_for_switch(
            save_path,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
        ),
    ))
}

pub fn preview_truck_switch(
    ctx: &DataContext,
    save_path: Option<String>,
    target_truck_id: String,
    expected_file_hash: String,
) -> Result<TruckChangePreview, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::truck_change::commands::preview_active_truck_switch(
            save_path,
            target_truck_id,
            expected_file_hash,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
        ),
    ))
}

pub fn apply_truck_switch(
    ctx: &DataContext,
    save_path: Option<String>,
    target_truck_id: String,
    expected_file_hash: String,
    create_persistent_backup: Option<bool>,
) -> Result<ApplyTruckChangeResult, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::truck_change::commands::apply_active_truck_switch(
            save_path,
            target_truck_id,
            expected_file_hash,
            create_persistent_backup,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<ProfileCache>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<TruckChangeSessionCache>(),
        ),
    ))
}

// ---------------------------------------------------------------------------
// Trailer switcher (audit rows 48-51)
// ---------------------------------------------------------------------------

pub fn init_trailer_switch(
    ctx: &DataContext,
    save_path: Option<String>,
) -> Result<TrailerChangeSession, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::trailer_change::commands::initialize_trailer_change_session(
            save_path,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<TrailerChangeSessionCache>(),
        ),
    ))
}

pub fn list_trailers_for_switch(
    ctx: &DataContext,
    save_path: Option<String>,
) -> Result<TrailerSwitchList, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::trailer_change::commands::list_owned_trailers_for_switch(
            save_path,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
        ),
    ))
}

pub fn preview_trailer_switch(
    ctx: &DataContext,
    save_path: Option<String>,
    target_trailer_id: String,
    expected_file_hash: String,
) -> Result<TrailerChangePreview, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::trailer_change::commands::preview_active_trailer_switch(
            save_path,
            target_trailer_id,
            expected_file_hash,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
        ),
    ))
}

pub fn apply_trailer_switch(
    ctx: &DataContext,
    save_path: Option<String>,
    target_trailer_id: String,
    expected_file_hash: String,
    create_persistent_backup: Option<bool>,
) -> Result<ApplyTrailerChangeResult, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::trailer_change::commands::apply_active_trailer_switch(
            save_path,
            target_trailer_id,
            expected_file_hash,
            create_persistent_backup,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<ProfileCache>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<TrailerChangeSessionCache>(),
        ),
    ))
}

// ---------------------------------------------------------------------------
// Powertrain catalog / preview (audit rows 52-53)
// ---------------------------------------------------------------------------

pub fn powertrain_catalog(
    ctx: &DataContext,
    game: Option<String>,
    game_version: Option<String>,
) -> Result<PowertrainCatalog, UiError> {
    let _ = ctx;
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::truck_change::commands::get_official_powertrain_catalog(
            game,
            game_version,
        ),
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn preview_powertrain(
    ctx: &DataContext,
    save_path: Option<String>,
    truck_id: String,
    engine_data_path: Option<String>,
    transmission_data_path: Option<String>,
    game: Option<String>,
    game_version: Option<String>,
) -> Result<TruckPowertrainPreview, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::truck_change::commands::preview_truck_powertrain_change(
            save_path,
            truck_id,
            engine_data_path,
            transmission_data_path,
            game,
            game_version,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
        ),
    ))
}
