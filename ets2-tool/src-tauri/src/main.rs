#![allow(unused_imports, unreachable_code, unused_assignments, dead_code)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use crate::state::AuthState;
use crate::state::{AppProfileState, AppState, DecryptCache, EtsDbState, ProfileCache};
use crate::state::{CareerState, HubState};
use crate::tauri::AppHandle;

mod commands;
mod db;
mod events;
mod features;
mod gui;
mod models;
mod shared;
mod state;
pub mod tauri_compat;
mod xp;

pub use tauri_compat as tauri;
pub use tauri_compat::dialog as tauri_plugin_dialog;

fn initialize_backend() -> AppHandle {
    crate::shared::logs::diagnostic::install_panic_hook();
    let app = AppHandle::new();

    let diagnostic_root = app
        .path()
        .app_local_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir().join("com.xlieferant.saveedittool"))
        .join("support");
    let diagnostic_runtime = shared::logs::diagnostic::DiagnosticRuntime::start(
        diagnostic_root,
        env!("CARGO_PKG_VERSION").to_string(),
    );

    crate::dev_log!("[app] starting native desktop runtime");
    let _ = crate::shared::user_log::user_log_info("App", "Application start (native GUI)");

    let sqlite_pool = tauri::async_runtime::block_on(db::sqlite::init_sqlite())
        .expect("failed to initialize central sqlite pool");
    let sqlite_path = db::sqlite::app_db_path();
    if let Err(error) = db::sqlite::validate_sqlite_extension(&sqlite_path) {
        panic!("{}", error);
    }

    app.manage(DecryptCache::default());
    app.manage(AppProfileState::default());
    app.manage(ProfileCache::default());
    app.manage(features::truck_change::cache::TruckChangeSessionCache::default());
    app.manage(features::trailer_change::cache::TrailerChangeSessionCache::default());
    app.manage(HubState::default());
    app.manage(CareerState::default());
    app.manage(AuthState::default());
    app.manage(EtsDbState {
        pool: sqlite_pool.clone(),
    });
    app.manage(AppState {
        sqlite: sqlite_pool.clone(),
        sqlite_path: sqlite_path.clone(),
        garage_mutation_lock: std::sync::Mutex::new(()),
    });
    app.manage(state::DiagnosticsState(diagnostic_runtime));

    let career = app.state::<CareerState>();
    let runtime = career.runtime.clone();
    let auth = app.state::<AuthState>();

    match features::language::translator::initialize_translator(&app) {
        Ok(()) => crate::dev_log!("[language] translator initialized"),
        Err(error) => crate::dev_log!("[language] translator init failed: {}", error),
    }

    let db_path = db::sqlite::app_db_path();
    match features::career::db::init_logbook(&db_path) {
        Ok(()) => {
            if let Ok(mut guard) = runtime.db_path.lock() {
                *guard = Some(db_path);
            }

            if let Ok(mut conn) = rusqlite::Connection::open(db::sqlite::app_db_path()) {
                let _ = conn.execute_batch("PRAGMA foreign_keys = ON;");
                let _ = conn.busy_timeout(std::time::Duration::from_secs(5));
                if let Err(error) = features::auth::db::ensure_tables(&conn) {
                    crate::dev_log!("[auth] ensure tables failed: {}", error);
                } else if let Err(error) = features::vtc::db::ensure_tables(&conn) {
                    crate::dev_log!("[vtc] ensure tables failed: {}", error);
                } else if let Err(error) = features::auth::service::seed_default_admin(&conn) {
                    crate::dev_log!("[auth] seed admin failed: {}", error);
                } else if let Err(error) =
                    features::auth::service::restore_persisted_session(&conn, auth.inner())
                {
                    crate::dev_log!("[auth] restore session failed: {}", error);
                } else {
                    let _ = features::vtc::service::ensure_local_company_bootstrap(
                        &conn,
                        auth.inner(),
                    );
                }

                if let Err(error) = shared::ets2data::import::ensure_tables(&conn) {
                    crate::dev_log!("[ets2data] ensure tables failed: {}", error);
                } else {
                    let company_count: i64 = conn
                        .query_row("SELECT COUNT(*) FROM ets2_companies", [], |row| row.get(0))
                        .unwrap_or(0);
                    if company_count == 0 {
                        let _ = shared::ets2data::import::import_datasets(
                            None,
                            &mut conn,
                            &shared::ets2data::default_repo_root(),
                            false,
                        );
                    }
                }
            }
        }
        Err(error) => {
            crate::dev_log!("[career] setup db init failed: {}", error);
        }
    }

    if let Ok(mode) = features::hub::config::load_mode() {
        let hub = app.state::<HubState>();
        if let Ok(mut guard) = hub.mode.write() {
            *guard = mode;
        }
    }

    for game in [
        features::career::plugin_installer::ScsGame::Ets2,
        features::career::plugin_installer::ScsGame::Ats,
    ] {
        // NT4: surface installer failures instead of discarding them; a
        // missing plugin means no live telemetry for that game.
        if let Err(error) = features::career::plugin_installer::ensure_plugin_files(&app, game) {
            crate::dev_log!(
                "[career] plugin install failed for {:?}: {}",
                game,
                error
            );
        }
    }

    features::career::service::start_background(app.clone(), runtime);
    let ets_db = app.state::<EtsDbState>();
    features::telemetry::scs_shared_mem::start(app.clone(), ets_db.pool.clone());

    let _ = crate::shared::user_log::user_log_info("App", "Native backend setup completed");
    app
}

fn main() {
    // Explicit save: game.sii edits are staged in memory until the user
    // presses Save in the top bar. Tests and the data-builder binary
    // never reach this entry point, so staging stays off for them.
    crate::shared::save_session::set_staging_enabled(true);

    let app = initialize_backend();
    let app_for_shutdown = app.clone();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("ETS2 / ATS Save-Edit Tool")
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([960.0, 640.0]),
        ..Default::default()
    };

    let run_result = eframe::run_native(
        "Save-Edit Tool",
        native_options,
        Box::new(move |cc| Ok(Box::new(gui::Ets2NativeApp::new(cc, app)))),
    );

    if let Some(state) = app_for_shutdown.try_state::<state::DiagnosticsState>() {
        state.0.shutdown();
    }

    if let Err(error) = run_result {
        eprintln!("Native GUI exited with error: {error}");
    }
}
