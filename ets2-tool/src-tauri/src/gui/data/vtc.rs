//! VTC / account and authentication wrappers (audit rows 76-78, 88-90).

use crate::features::auth::models::AuthLoginResult;
use crate::features::vtc::models::{CompanyMember, CompanyOverview, CreateCompanyInput, UserProfile};
use crate::state::AuthState;

use super::{map_error, DataContext, UiError};

pub fn current_user_profile(ctx: &DataContext) -> Result<UserProfile, UiError> {
    map_error(crate::features::vtc::commands::get_current_user_profile(
        ctx.app.state::<AuthState>(),
    ))
}

pub fn company_overview(ctx: &DataContext) -> Result<CompanyOverview, UiError> {
    map_error(crate::features::vtc::commands::get_company_overview(
        ctx.app.state::<AuthState>(),
    ))
}

pub fn company_members(ctx: &DataContext) -> Result<Vec<CompanyMember>, UiError> {
    map_error(crate::features::vtc::commands::get_company_members(
        ctx.app.state::<AuthState>(),
    ))
}

pub fn logout(ctx: &DataContext) -> Result<(), UiError> {
    map_error(crate::features::auth::commands::auth_logout(
        ctx.app.state::<AuthState>(),
    ))
}

pub fn login(
    ctx: &DataContext,
    email: String,
    password: String,
    remember_me: bool,
) -> Result<AuthLoginResult, UiError> {
    map_error(crate::features::auth::commands::auth_login(
        email,
        password,
        remember_me,
        ctx.app.state::<AuthState>(),
    ))
}

pub fn create_company(
    ctx: &DataContext,
    input: CreateCompanyInput,
) -> Result<CompanyOverview, UiError> {
    map_error(crate::features::vtc::commands::create_company(
        input,
        ctx.app.state::<AuthState>(),
    ))
}
