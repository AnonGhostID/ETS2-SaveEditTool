//! Staged-edit state, Save / Discard / Undo wrappers (audit rows 3, 5, 27, 34, 91).

use crate::features::save_editor::commands::{
    discard_pending_changes, save_pending_changes, undo_last_save_change,
};
use crate::shared::save_session;
use crate::state::{AppProfileState, DecryptCache, ProfileCache};

use super::{map_error, DataContext, UiError};

/// Whether any staged (unflushed) edit exists.
pub fn is_dirty(_ctx: &DataContext) -> bool {
    save_session::is_dirty()
}

/// Staged paths as display strings for the Safety Spine gauge.
pub fn dirty_paths(_ctx: &DataContext) -> Vec<String> {
    save_session::dirty_paths()
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect()
}

pub fn is_staging_enabled(_ctx: &DataContext) -> bool {
    save_session::is_staging_enabled()
}

/// Flush staged edits: backend creates one pre-flush backup, then writes.
/// `Ok(paths)` are the paths actually written.
pub fn flush(ctx: &DataContext) -> Result<Vec<String>, UiError> {
    map_error(save_pending_changes(
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
    ))
}

/// Drop staged edits without touching disk files.
pub fn discard(ctx: &DataContext) -> Result<(), UiError> {
    map_error(discard_pending_changes(
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
    ))
}

/// Undo the last save change by restoring its backup. The confirm dialog is
/// owned by the shell; this wrapper only marshals state and errors.
pub fn undo_last(ctx: &DataContext) -> Result<(), UiError> {
    map_error(undo_last_save_change(
        ctx.app.state::<AppProfileState>(),
        ctx.app.state::<ProfileCache>(),
        ctx.app.state::<DecryptCache>(),
    ))
    .map(|_| ())
}
