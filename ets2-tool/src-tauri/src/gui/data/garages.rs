//! Garage list/mutations and AI driver management (audit rows 54-64).

use crate::features::garages::models::{
    AiDriverGenerateRequest, AiDriverManagementResult, AiDriverMutationResult, AiDriverRemoveRequest,
    GarageBuyAllRequest, GarageBuyAllResult, GarageListResult, GarageMutationRequest, GarageMutationResult,
    GarageRelinquishEmptyRequest, GarageRelinquishEmptyResult, GarageResourceAssignmentRequest,
    GarageUpdateRequest,
};
use crate::features::trailer_change::cache::TrailerChangeSessionCache;
use crate::features::truck_change::cache::TruckChangeSessionCache;
use crate::state::{AppProfileState, AppState, DecryptCache, ProfileCache};

use super::{map_error, DataContext, UiError};

pub fn list_all_garages(ctx: &DataContext) -> Result<GarageListResult, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::garages::commands::get_all_garages(
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<AppState>(),
        ),
    ))
}

pub fn ai_driver_management(ctx: &DataContext) -> Result<AiDriverManagementResult, UiError> {
    map_error(crate::tauri::async_runtime::block_on(
        crate::features::garages::commands::get_ai_driver_management(
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<AppState>(),
        ),
    ))
}

pub fn generate_drivers(
    ctx: &DataContext,
    request: AiDriverGenerateRequest,
) -> Result<AiDriverMutationResult, UiError> {
    map_error(crate::features::garages::commands::generate_ai_drivers(
        request,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
        ctx.app.state::<TruckChangeSessionCache>(),
        ctx.app.state::<TrailerChangeSessionCache>(),
        ctx.app.state::<AppState>(),
    ))
}

pub fn purchase_garage(
    ctx: &DataContext,
    request: GarageMutationRequest,
) -> Result<GarageMutationResult, UiError> {
    map_error(crate::features::garages::commands::purchase_garage(
        request,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
        ctx.app.state::<TruckChangeSessionCache>(),
        ctx.app.state::<TrailerChangeSessionCache>(),
        ctx.app.state::<AppState>(),
    ))
}

pub fn upgrade_garage(
    ctx: &DataContext,
    request: GarageMutationRequest,
) -> Result<GarageMutationResult, UiError> {
    map_error(crate::features::garages::commands::upgrade_owned_garage(
        request,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
        ctx.app.state::<TruckChangeSessionCache>(),
        ctx.app.state::<TrailerChangeSessionCache>(),
        ctx.app.state::<AppState>(),
    ))
}

pub fn assign_random_resources(
    ctx: &DataContext,
    request: GarageResourceAssignmentRequest,
) -> Result<GarageMutationResult, UiError> {
    map_error(
        crate::features::garages::commands::assign_random_garage_resources(
            request,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<ProfileCache>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<TruckChangeSessionCache>(),
            ctx.app.state::<TrailerChangeSessionCache>(),
            ctx.app.state::<AppState>(),
        ),
    )
}

pub fn update_garage(
    ctx: &DataContext,
    request: GarageUpdateRequest,
) -> Result<GarageMutationResult, UiError> {
    map_error(crate::features::garages::commands::update_garage(
        request,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
        ctx.app.state::<TruckChangeSessionCache>(),
        ctx.app.state::<TrailerChangeSessionCache>(),
        ctx.app.state::<AppState>(),
    ))
}

pub fn sell_garage(
    ctx: &DataContext,
    request: GarageMutationRequest,
) -> Result<GarageMutationResult, UiError> {
    map_error(
        crate::features::garages::commands::relinquish_garage_ownership(
            request,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<ProfileCache>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<TruckChangeSessionCache>(),
            ctx.app.state::<TrailerChangeSessionCache>(),
            ctx.app.state::<AppState>(),
        ),
    )
}

pub fn sell_empty_garages(
    ctx: &DataContext,
    request: GarageRelinquishEmptyRequest,
) -> Result<GarageRelinquishEmptyResult, UiError> {
    map_error(
        crate::features::garages::commands::relinquish_empty_garages(
            request,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<ProfileCache>(),
            ctx.app.state::<DecryptCache>(),
            ctx.app.state::<TruckChangeSessionCache>(),
            ctx.app.state::<TrailerChangeSessionCache>(),
            ctx.app.state::<AppState>(),
        ),
    )
}

pub fn buy_all_garages(
    ctx: &DataContext,
    request: GarageBuyAllRequest,
) -> Result<GarageBuyAllResult, UiError> {
    map_error(crate::features::garages::commands::buy_all_garages(
        request,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
        ctx.app.state::<TruckChangeSessionCache>(),
        ctx.app.state::<TrailerChangeSessionCache>(),
        ctx.app.state::<AppState>(),
    ))
}

pub fn remove_drivers(
    ctx: &DataContext,
    request: AiDriverRemoveRequest,
) -> Result<AiDriverMutationResult, UiError> {
    map_error(crate::features::garages::commands::remove_ai_drivers(
        request,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
        ctx.app.state::<TruckChangeSessionCache>(),
        ctx.app.state::<TrailerChangeSessionCache>(),
        ctx.app.state::<AppState>(),
    ))
}
