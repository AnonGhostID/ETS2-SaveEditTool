//! Typed, synchronous data layer for the new GUI shell and screens.
//!
//! Every backend access lives in this module tree. Screens and the shell only
//! call the free functions here; they never import `crate::features`,
//! `crate::commands` or `crate::tauri::async_runtime`.

pub mod career;
pub mod coaches;
pub mod context;
pub mod economy;
pub mod garages;
pub mod library;
pub mod loading;
pub mod quick_actions;
pub mod safety;
pub mod session;
pub mod system;
pub mod vehicles;
pub mod vtc;

use crate::tauri::AppHandle;

/// A backend error mapped into a UI-displayable message. Never swallowed:
/// every `Result<T, String>` / `Result<T, AppError>` from the backend becomes
/// `Result<T, UiError>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiError {
    pub message: String,
}

impl std::fmt::Display for UiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for UiError {}

/// Handle to the backend state registry, passed as the first argument of every
/// data wrapper.
#[derive(Clone)]
pub struct DataContext {
    pub app: AppHandle,
}

impl DataContext {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

#[allow(dead_code)]
pub(crate) fn ui_error(message: impl Into<String>) -> UiError {
    UiError {
        message: message.into(),
    }
}

/// Map the backend's `Result<T, String>` into `Result<T, UiError>`.
#[allow(dead_code)]
pub(crate) fn map_error<T>(result: Result<T, String>) -> Result<T, UiError> {
    result.map_err(ui_error)
}
