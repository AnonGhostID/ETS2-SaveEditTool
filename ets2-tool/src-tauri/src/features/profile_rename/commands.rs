use crate::dev_log;
use crate::shared::current_profile::require_current_profile;
use crate::shared::decrypt::decrypt_if_needed;
use crate::shared::hex_float::text_to_hex;
use crate::state::AppProfileState;
use std::fs;
use std::path::Path;
pub fn profile_rename(
    new_name: String,
    profile_state: crate::tauri::State<'_, AppProfileState>,
) -> Result<String, String> {
    // 1. Get current profile path
    let profile_path_str = require_current_profile(profile_state.clone())?;
    let profile_path = Path::new(&profile_path_str);

    if !profile_path.exists() {
        return Err("Profilpfad existiert nicht".into());
    }

    dev_log!("Starte Profil-Umbenennung für: {}", profile_path.display());

    // 2. Find profile.sii
    let profile_sii = profile_path.join("profile.sii");
    if !profile_sii.exists() {
        return Err("profile.sii nicht gefunden".into());
    }

    // 3. Compute the target folder and check for conflicts BEFORE writing
    // anything (P4): the old order wrote profile.sii first, so renaming
    // to an already-used hex folder left the old folder's profile.sii
    // claiming the new name even though the rename was refused.
    let parent = profile_path.parent().ok_or("Kein Elternordner gefunden")?;
    let new_dir_name = text_to_hex(&new_name);
    let new_profile_path = parent.join(&new_dir_name);

    if new_profile_path.exists() && new_profile_path != profile_path {
        return Err(format!(
            "Ein Profilordner mit dem Namen '{}' existiert bereits.",
            new_dir_name
        ));
    }

    // 4. Decrypt & Read
    let content = decrypt_if_needed(&profile_sii).map_err(|e| e.to_string())?;

    // 5. Replace Name in Content
    let new_content = change_profile_name_in_sii(&content, &new_name)?;

    // 6. Write back profile.sii
    fs::write(&profile_sii, new_content)
        .map_err(|e| format!("Fehler beim Schreiben von profile.sii: {}", e))?;

    // 7. Rename Folder (Input Text -> Hex)
    if new_profile_path != profile_path {
        fs::rename(profile_path, &new_profile_path)
            .map_err(|e| format!("Fehler beim Umbenennen des Ordners: {}", e))?;

        // 8. Update State with new path
        *profile_state.current_profile.lock().unwrap() =
            Some(new_profile_path.to_string_lossy().to_string());

        // P4: the current save lives under the profile folder, so it
        // must move with the rename instead of pointing at the old path
        // until a full re-selection. Best-effort: a poisoned lock must
        // not fail a rename that already succeeded.
        if let Ok(mut current_save) = profile_state.current_save.lock() {
            if let Some(save) = current_save.as_ref() {
                if let Ok(relative) = Path::new(save).strip_prefix(profile_path) {
                    *current_save = Some(
                        new_profile_path
                            .join(relative)
                            .to_string_lossy()
                            .to_string(),
                    );
                }
            }
        }

        dev_log!(
            "Profil erfolgreich umbenannt nach: {}",
            new_profile_path.display()
        );
        Ok(new_profile_path.to_string_lossy().to_string())
    } else {
        Ok(profile_path_str)
    }
}

fn change_profile_name_in_sii(content: &str, new_name: &str) -> Result<String, String> {
    let mut result = String::new();
    let mut found = false;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("profile_name:") {
            // Replace the line with the new name
            result.push_str(&format!(
                " profile_name: \"{}\"",
                escape_sii_string(new_name)
            ));
            found = true;
        } else {
            result.push_str(line);
        }
        result.push_str("\r\n");
    }

    if !found {
        return Err("Konnte 'profile_name' in profile.sii nicht finden.".into());
    }
    Ok(result)
}

/// P9: escape a user-supplied value before interpolating it into a
/// quoted SII string, so `"`, `\` or a newline in the name cannot break
/// out of the value and corrupt the file.
fn escape_sii_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\r' | '\n' => escaped.push(' '),
            other => escaped.push(other),
        }
    }
    escaped
}
