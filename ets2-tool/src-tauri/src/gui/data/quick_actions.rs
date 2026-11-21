//! Dashboard quick actions (audit rows 28-32). All mutations delegate to the
//! owning module wrappers; no backend logic is reimplemented here.

use crate::features::career::dispatcher::DispatcherGenerationStatus;
use crate::features::health_monitor::models::SaveHealthReportDto;

use super::{ui_error, DataContext, UiError};

pub fn repair_truck(ctx: &DataContext) -> Result<(), UiError> {
    super::vehicles::repair_truck(ctx)
}

pub fn refuel_truck(ctx: &DataContext) -> Result<(), UiError> {
    super::vehicles::refuel_truck(ctx)
}

/// Six skill writes in one user action. Every backend failure is reported in
/// the returned error message; none is silently dropped.
pub fn max_all_skills(ctx: &DataContext) -> Result<(), UiError> {
    let mut failures: Vec<String> = Vec::new();
    for (skill, max) in [
        ("adr", 63),
        ("long_dist", 6),
        ("heavy", 6),
        ("fragile", 6),
        ("urgent", 6),
        ("mechanical", 6),
    ] {
        if let Err(error) = super::economy::set_skill(ctx, skill.to_string(), max) {
            failures.push(format!("{skill}: {}", error.message));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(ui_error(failures.join("; ")))
    }
}

pub fn scan_save_health(ctx: &DataContext) -> Result<SaveHealthReportDto, UiError> {
    super::safety::scan_save_health(ctx)
}

pub fn generate_market_jobs(ctx: &DataContext) -> Result<DispatcherGenerationStatus, UiError> {
    super::career::generate_jobs(ctx)
}
