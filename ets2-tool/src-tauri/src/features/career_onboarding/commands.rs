use rusqlite::Connection;
use crate::tauri::State;
use crate::features::auth::db as auth_db;
use crate::features::auth::service as auth_service;
use crate::features::career_onboarding::service::{self, CareerOnboardingState};
use crate::features::companies::db as company_db;
use crate::state::AuthState;
pub fn career_get_onboarding_state(
    auth: State<'_, AuthState>,
) -> Result<CareerOnboardingState, String> {
    let db_path = auth_db::default_db_path();
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| e.to_string())?;
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| e.to_string())?;
    auth_db::ensure_tables(&conn)?;
    company_db::ensure_tables(&conn)?;
    auth_service::seed_default_admin(&conn)?;

    service::get_onboarding_state(&conn, auth.inner())
}
