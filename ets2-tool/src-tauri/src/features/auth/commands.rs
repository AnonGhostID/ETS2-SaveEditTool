use rusqlite::Connection;
use crate::tauri::State;
use crate::features::auth::db;
use crate::features::auth::models::{
    AuthAccountOverview, AuthAdminDbOverview, AuthLoginResult, AuthRegisterResult, PublicUser,
};
use crate::features::auth::service;
use crate::features::vtc;
use crate::state::AuthState;

fn open_connection(auth: &AuthState) -> Result<Connection, String> {
    let db_path = db::default_db_path();
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    db::ensure_tables(&conn)?;
    vtc::db::ensure_tables(&conn)?;
    service::seed_default_admin(&conn)?;
    vtc::service::ensure_local_company_bootstrap(&conn, auth)?;
    Ok(conn)
}
pub fn auth_seed_default_admin() -> Result<(), String> {
    let db_path = db::default_db_path();
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    db::ensure_tables(&conn)?;
    vtc::db::ensure_tables(&conn)?;
    service::seed_default_admin(&conn)?;
    Ok(())
}
pub fn auth_register(
    username: String,
    email: String,
    password: String,
    password_confirm: String,
    consent_privacy: bool,
    consent_terms: bool,
    remember_me: bool,
    auth: State<'_, AuthState>,
) -> Result<AuthRegisterResult, String> {
    let conn = open_connection(auth.inner())?;

    let (user, remember_me) = service::register_local(
        &conn,
        auth.inner(),
        username,
        email,
        password,
        password_confirm,
        consent_privacy,
        consent_terms,
        remember_me,
    )?;

    Ok(AuthRegisterResult { user, remember_me })
}
pub fn auth_login(
    email: String,
    password: String,
    remember_me: bool,
    auth: State<'_, AuthState>,
) -> Result<AuthLoginResult, String> {
    let conn = open_connection(auth.inner())?;

    let (user, remember_me) =
        service::login_local(&conn, auth.inner(), email, password, remember_me)?;
    Ok(AuthLoginResult { user, remember_me })
}
pub fn auth_logout(auth: State<'_, AuthState>) -> Result<(), String> {
    service::logout_local(auth.inner())
}
pub fn auth_get_current_user(auth: State<'_, AuthState>) -> Result<Option<PublicUser>, String> {
    let conn = open_connection(auth.inner())?;
    service::get_current_user(&conn, auth.inner())
}
pub fn auth_restore_session(auth: State<'_, AuthState>) -> Result<(), String> {
    let conn = open_connection(auth.inner())?;
    service::restore_persisted_session(&conn, auth.inner())?;
    vtc::service::ensure_local_company_bootstrap(&conn, auth.inner())
}
pub fn auth_get_account_overview(
    auth: State<'_, AuthState>,
) -> Result<AuthAccountOverview, String> {
    let conn = open_connection(auth.inner())?;
    service::get_account_overview(&conn, auth.inner())
}
pub fn auth_generate_recovery_codes(auth: State<'_, AuthState>) -> Result<Vec<String>, String> {
    let conn = open_connection(auth.inner())?;
    service::generate_recovery_codes(&conn, auth.inner(), 5)
}
pub fn auth_reset_password_with_recovery_code(
    email: String,
    recovery_code: String,
    new_password: String,
    new_password_confirm: String,
) -> Result<(), String> {
    let auth = AuthState::default();
    let conn = open_connection(&auth)?;
    service::reset_password_with_recovery_code(
        &conn,
        email,
        recovery_code,
        new_password,
        new_password_confirm,
    )
}
pub fn auth_admin_get_db_overview(
    auth: State<'_, AuthState>,
) -> Result<AuthAdminDbOverview, String> {
    let conn = open_connection(auth.inner())?;
    service::admin_get_db_overview(&conn, auth.inner())
}
