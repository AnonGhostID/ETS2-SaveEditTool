use rusqlite::Connection;
use crate::tauri::State;
use crate::features::auth::db as auth_db;
use crate::features::vtc::db;
use crate::features::vtc::models::{
    CareerSettings, CompanyMember, CompanyOverview, CompanyRoleOption, CompanySettings,
    CreateCompanyInput, UpdateCareerSettingsInput, UpdateCompanyProfileInput,
    UpdateCompanySettingsInput, UpdateUserProfileMetaInput, UpdateUserSettingsInput, UserProfile,
    UserSettings, UsernameAvailability, VtcRuntimeContext,
};
use crate::features::vtc::service;
use crate::shared::current_profile::snapshot_resolved_save_context;
use crate::state::{AppProfileState, AuthState};

fn open_connection(auth: &AuthState) -> Result<Connection, String> {
    let db_path = auth_db::default_db_path();
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| e.to_string())?;
    auth_db::ensure_tables(&conn)?;
    db::ensure_tables(&conn)?;
    service::ensure_local_company_bootstrap(&conn, auth)?;
    Ok(conn)
}

/// Read-only connection for commands that must not mutate state (D12c):
/// schema setup only (`auth_db::ensure_tables` is pure DDL), no admin
/// seeding and no company bootstrap.
fn open_read_connection() -> Result<Connection, String> {
    let db_path = auth_db::default_db_path();
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| e.to_string())?;
    auth_db::ensure_tables(&conn)?;
    Ok(conn)
}
pub fn get_current_user_profile(auth: State<'_, AuthState>) -> Result<UserProfile, String> {
    let conn = open_connection(auth.inner())?;
    service::get_current_user_profile(&conn, auth.inner())
}
pub fn get_vtc_runtime_context(
    auth: State<'_, AuthState>,
    profile: State<'_, AppProfileState>,
) -> Result<VtcRuntimeContext, String> {
    let conn = open_connection(auth.inner())?;
    let user = service::get_current_user_profile(&conn, auth.inner())?;
    let resolved_context = snapshot_resolved_save_context(profile.inner())?;
    let save_context = resolved_context.context;
    let save_session_status = if save_context.is_ready() {
        if resolved_context.profile_inferred || resolved_context.save_inferred {
            "inferred"
        } else {
            "linked"
        }
    } else if save_context.profile_reference.is_some() {
        "profile_only"
    } else {
        "missing"
    };

    Ok(VtcRuntimeContext {
        user_id: user.user_id,
        username: user.username,
        has_active_profile: save_context.profile_reference.is_some(),
        has_active_save: save_context.save_reference.is_some(),
        profile_reference: save_context.profile_reference,
        save_reference: save_context.save_reference,
        quicksave_reference: save_context.quicksave_reference,
        save_session_id: save_context.save_session_id,
        save_session_status: save_session_status.to_string(),
    })
}
pub fn update_user_language(
    language: String,
    auth: State<'_, AuthState>,
) -> Result<UserSettings, String> {
    let conn = open_connection(auth.inner())?;
    service::update_user_language(&conn, auth.inner(), language)
}
pub fn update_username(
    username: String,
    auth: State<'_, AuthState>,
) -> Result<UserProfile, String> {
    let mut conn = open_connection(auth.inner())?;
    service::update_username(&mut conn, auth.inner(), username)
}
pub fn check_username_availability(username: String) -> Result<UsernameAvailability, String> {
    let conn = open_read_connection()?;
    service::check_username_availability(&conn, username)
}
pub fn update_user_profile_meta(
    input: UpdateUserProfileMetaInput,
    auth: State<'_, AuthState>,
) -> Result<UserSettings, String> {
    let conn = open_connection(auth.inner())?;
    service::update_user_profile_meta(&conn, auth.inner(), input)
}
pub fn create_company(
    input: CreateCompanyInput,
    auth: State<'_, AuthState>,
) -> Result<CompanyOverview, String> {
    let mut conn = open_connection(auth.inner())?;
    service::create_company(&mut conn, auth.inner(), input)
}
pub fn get_company_overview(auth: State<'_, AuthState>) -> Result<CompanyOverview, String> {
    let conn = open_connection(auth.inner())?;
    service::get_company_overview(&conn, auth.inner())
}
pub fn update_company_profile(
    input: UpdateCompanyProfileInput,
    auth: State<'_, AuthState>,
) -> Result<CompanyOverview, String> {
    let conn = open_connection(auth.inner())?;
    service::update_company_profile(&conn, auth.inner(), input)
}
pub fn get_company_members(auth: State<'_, AuthState>) -> Result<Vec<CompanyMember>, String> {
    let conn = open_connection(auth.inner())?;
    service::get_company_members(&conn, auth.inner())
}
pub fn update_company_settings(
    input: UpdateCompanySettingsInput,
    auth: State<'_, AuthState>,
) -> Result<CompanySettings, String> {
    let conn = open_connection(auth.inner())?;
    service::update_company_settings(&conn, auth.inner(), input)
}
pub fn assign_member_role(
    user_id: i64,
    role_key: String,
    auth: State<'_, AuthState>,
) -> Result<CompanyMember, String> {
    let conn = open_connection(auth.inner())?;
    service::assign_member_role(&conn, auth.inner(), user_id, role_key)
}
pub fn change_member_role(
    member_id: i64,
    role_key: String,
    auth: State<'_, AuthState>,
) -> Result<CompanyMember, String> {
    let conn = open_connection(auth.inner())?;
    service::change_member_role(&conn, auth.inner(), member_id, role_key)
}
pub fn get_available_roles() -> Result<Vec<CompanyRoleOption>, String> {
    Ok(service::get_available_roles_static())
}
pub fn get_user_settings(auth: State<'_, AuthState>) -> Result<UserSettings, String> {
    let conn = open_connection(auth.inner())?;
    service::get_user_settings(&conn, auth.inner())
}
pub fn update_user_settings(
    input: UpdateUserSettingsInput,
    auth: State<'_, AuthState>,
) -> Result<UserSettings, String> {
    let conn = open_connection(auth.inner())?;
    service::update_user_settings(&conn, auth.inner(), input)
}
pub fn get_company_settings(auth: State<'_, AuthState>) -> Result<CompanySettings, String> {
    let conn = open_connection(auth.inner())?;
    service::get_company_settings(&conn, auth.inner())
}
pub fn get_career_settings(auth: State<'_, AuthState>) -> Result<CareerSettings, String> {
    let conn = open_connection(auth.inner())?;
    service::get_career_settings(&conn, auth.inner())
}
pub fn update_career_settings(
    input: UpdateCareerSettingsInput,
    auth: State<'_, AuthState>,
) -> Result<CareerSettings, String> {
    let conn = open_connection(auth.inner())?;
    service::update_career_settings(&conn, auth.inner(), input)
}
