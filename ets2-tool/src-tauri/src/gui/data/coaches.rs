//! Project Coaches / Bus Career wrappers (audit rows 92-97).

use crate::features::save_editor::coaches::CoachesSnapshot;
use crate::state::{AppProfileState, DecryptCache, ProfileCache};
use crate::xp::command::LevelProgress;

use super::{map_error, DataContext, UiError};

pub fn snapshot(ctx: &DataContext) -> Result<CoachesSnapshot, UiError> {
    map_error(crate::features::save_editor::commands::read_project_coaches(
        ctx.app.state::<AppProfileState>(),
    ))
}

pub fn apply_bus_xp(
    ctx: &DataContext,
    xp: i64,
    expected_save_hash: String,
) -> Result<(), UiError> {
    map_error(crate::features::save_editor::commands::edit_career_xp(
        xp,
        crate::features::save_editor::coaches::Career::Bus,
        expected_save_hash,
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
    ))
}

pub fn display_job_stop_name(_ctx: &DataContext, raw: &str) -> String {
    crate::features::save_editor::coaches::display_job_stop_name(raw)
}

pub fn display_stop_name(_ctx: &DataContext, id: &str) -> String {
    crate::features::save_editor::coaches::display_stop_name(id)
}

pub fn display_bus_name(_ctx: &DataContext, id: &str) -> String {
    crate::features::save_editor::coaches::display_bus_name(id)
}

pub fn bus_level(ctx: &DataContext, total_xp: u64) -> LevelProgress {
    super::context::calculate_level(ctx, total_xp)
}
