use std::path::Path;

use crate::shared::models::profile::ActiveSaveSelection;
use crate::shared::models::save_context::SaveContext;
use crate::state::AppProfileState;
use crate::tauri::State;

#[derive(Debug, Clone)]
pub struct ResolvedSaveContext {
    pub context: SaveContext,
    pub profile_inferred: bool,
    pub save_inferred: bool,
}

pub fn set_current_profile(state: State<'_, AppProfileState>, path: String) {
    // Poison-safe (AS8): recover the guard instead of panicking, so a
    // poisoned lock still applies the new selection.
    let mut guard = state
        .current_profile
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = Some(path);
}

pub fn clear_current_profile(state: State<'_, AppProfileState>) {
    let mut guard = state
        .current_profile
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = None;
}

pub fn get_current_profile(state: State<'_, AppProfileState>) -> Option<String> {
    state
        .current_profile
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

pub fn require_current_profile(state: State<'_, AppProfileState>) -> Result<String, String> {
    get_current_profile(state).ok_or_else(|| "Kein Profil geladen.".to_string())
}

pub fn require_current_save(state: State<'_, AppProfileState>) -> Result<String, String> {
    state
        .current_save
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
        .ok_or_else(|| "Kein Save geladen.".to_string())
}

pub fn snapshot_active_save_selection(
    state: &AppProfileState,
) -> Result<ActiveSaveSelection, String> {
    let profile_path = state
        .current_profile
        .lock()
        .map_err(|_| "AppProfileState current_profile lock poisoned".to_string())?
        .clone();
    let save_path = state
        .current_save
        .lock()
        .map_err(|_| "AppProfileState current_save lock poisoned".to_string())?
        .clone();

    Ok(ActiveSaveSelection {
        profile_path,
        save_path,
    })
}

pub fn snapshot_resolved_save_context(
    state: &AppProfileState,
) -> Result<ResolvedSaveContext, String> {
    let selection = snapshot_active_save_selection(state)?;

    let mut profile_path = normalize_existing_path(selection.profile_path);
    let save_path = normalize_existing_path(selection.save_path);
    let mut profile_inferred = false;

    // Deriving the profile from the stored save path is structural, not
    // a guess: the save lives inside its profile directory. It stays
    // tagged as inferred so callers can surface it (AS4).
    if profile_path.is_none() {
        if let Some(derived_profile) = save_path.as_deref().and_then(derive_profile_from_save_path)
        {
            profile_path = Some(derived_profile);
            profile_inferred = true;
        }
    }

    // NOTE (AS4): no mtime-based inference here anymore. When the stored
    // selection is missing or stale, the caller must ask for an explicit
    // profile/save selection instead of silently operating on whichever
    // profile/save happens to have the newest game.sii.
    let profile_path = match profile_path {
        Some(path) => path,
        None => {
            return Err(
                "No profile is selected. Please select a profile first.".to_string()
            );
        }
    };
    let save_path = match save_path {
        Some(path) => path,
        None => {
            return Err(
                "No save is selected for the current profile. Please select a save first."
                    .to_string(),
            );
        }
    };

    Ok(ResolvedSaveContext {
        context: SaveContext::from_paths(Some(profile_path), Some(save_path)),
        profile_inferred,
        save_inferred: false,
    })
}

pub fn snapshot_save_context(state: &AppProfileState) -> Result<SaveContext, String> {
    Ok(snapshot_resolved_save_context(state)?.context)
}

fn normalize_existing_path(path: Option<String>) -> Option<String> {
    let value = path?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let normalized = trimmed.replace('\\', "/");
    if Path::new(trimmed).exists() || Path::new(&normalized).exists() {
        Some(normalized)
    } else {
        None
    }
}

fn derive_profile_from_save_path(save_path: &str) -> Option<String> {
    let save_path = Path::new(save_path);
    let save_dir = if save_path.is_file() {
        save_path.parent()?
    } else {
        save_path
    };
    let save_root = save_dir.parent()?;
    if !save_root
        .file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("save"))
        .unwrap_or(false)
    {
        return None;
    }

    let profile_dir = save_root.parent()?;
    if !profile_dir.join("profile.sii").exists() {
        return None;
    }

    Some(profile_dir.display().to_string().replace('\\', "/"))
}
