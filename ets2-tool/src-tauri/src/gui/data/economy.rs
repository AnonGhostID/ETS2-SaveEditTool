//! Save-editor economy, skills, config values and custom keys
//! (audit rows 135-148).

use crate::features::save_editor::commands::{
    ApplyCustomResetValuesResultDto, KeyValuePayload,
};
use crate::features::save_editor::coaches::CoachesSnapshot;
use crate::state::{AppProfileState, DecryptCache, ProfileCache};
use crate::xp::command::LevelProgress;

use super::{map_error, DataContext, UiError};

/// Audit row 135: coaches snapshot refresh from the Save Editor screen.
pub fn refresh_coaches_snapshot(ctx: &DataContext) -> Result<CoachesSnapshot, UiError> {
    map_error(crate::features::save_editor::commands::read_project_coaches(
        ctx.app.state::<AppProfileState>(),
    ))
}

pub fn edit_money(ctx: &DataContext, value: i64) -> Result<(), UiError> {
    map_error(crate::features::save_editor::commands::edit_player_money(
        value,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
    ))
}

pub fn edit_experience(ctx: &DataContext, value: i64) -> Result<(), UiError> {
    map_error(crate::features::save_editor::commands::edit_player_experience(
        value,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
    ))
}

pub fn edit_level(ctx: &DataContext, level: i64) -> Result<(), UiError> {
    map_error(crate::features::save_editor::commands::edit_level(
        level,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
    ))
}

pub fn apply_custom_reset(
    ctx: &DataContext,
    level: Option<u32>,
    xp: Option<u64>,
    money: Option<i64>,
) -> Result<ApplyCustomResetValuesResultDto, UiError> {
    map_error(
        crate::features::save_editor::commands::apply_custom_reset_values(
            level,
            xp,
            money,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<ProfileCache>(),
            ctx.app.state::<DecryptCache>(),
        ),
    )
}

pub fn set_skill(ctx: &DataContext, skill: String, value: i64) -> Result<(), UiError> {
    map_error(crate::features::save_editor::commands::edit_skill_value(
        skill,
        value,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
    ))
}

pub fn set_traffic(ctx: &DataContext, value: i64) -> Result<(), UiError> {
    map_error(crate::features::save_editor::commands::edit_traffic_value(
        value,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
    ))
}

pub fn set_developer(ctx: &DataContext, value: i64) -> Result<(), UiError> {
    map_error(crate::features::save_editor::commands::edit_developer_value(
        value,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
    ))
}

pub fn set_console(ctx: &DataContext, value: i64) -> Result<(), UiError> {
    map_error(crate::features::save_editor::commands::edit_console_value(
        value,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
    ))
}

pub fn set_parking_doubles(ctx: &DataContext, value: i64) -> Result<(), UiError> {
    map_error(
        crate::features::save_editor::commands::edit_parking_doubles_value(
            value,
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<ProfileCache>(),
            ctx.app.state::<DecryptCache>(),
        ),
    )
}

pub fn set_convoy(ctx: &DataContext, value: i64) -> Result<(), UiError> {
    map_error(crate::features::save_editor::commands::edit_convoy_value(
        value,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
    ))
}

pub fn set_base_config_key(
    ctx: &DataContext,
    key: String,
    value: String,
) -> Result<(), UiError> {
    map_error(crate::features::save_editor::commands::edit_config_value(
        KeyValuePayload { key, value },
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
    ))
}

pub fn set_save_config_key(
    ctx: &DataContext,
    key: String,
    value: String,
) -> Result<(), UiError> {
    map_error(
        crate::features::save_editor::commands::edit_save_config_value(
            KeyValuePayload { key, value },
            ctx.app.state::<AppProfileState>(),
            ctx.app.state::<ProfileCache>(),
            ctx.app.state::<DecryptCache>(),
        ),
    )
}

/// Audit row 148: level preview helper.
pub fn level_progress(ctx: &DataContext, total_xp: u64) -> LevelProgress {
    super::context::calculate_level(ctx, total_xp)
}
