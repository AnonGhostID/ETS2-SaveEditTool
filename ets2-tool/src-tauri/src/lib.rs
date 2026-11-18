#![allow(unused_imports, unreachable_code, unused_assignments, dead_code)]
pub mod events;
pub mod models;
pub mod shared;
pub mod state;
pub mod tauri_compat;

pub use tauri_compat as tauri;
pub use tauri_compat::dialog as tauri_plugin_dialog;
