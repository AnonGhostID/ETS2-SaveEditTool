use crate::models::clone_profiles_info::CloneOptions;
use crate::shared::{decrypt::decrypt_if_needed, hex_float};
use regex::Regex;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::CompressionMethod;
use zip::{ZipWriter, write::FileOptions};

/// Hauptlogik
pub fn clone_profile(
    source: &Path,
    new_name: &str,
    options: CloneOptions,
) -> Result<PathBuf, String> {
    if !source.exists() {
        return Err("Quellprofil existiert nicht".to_string());
    }

    let parent = source
        .parent()
        .ok_or("Kein Parent-Verzeichnis".to_string())?;
    // P5: honor the clone options instead of always hex-renaming the
    // folder and always rewriting the internal name.
    let target_dir = if options.replace_hex {
        let new_hex = hex_float::text_to_hex(new_name);
        let target_dir = parent.join(&new_hex);
        if target_dir.exists() {
            return Err("Zielprofil existiert bereits".to_string());
        }
        target_dir
    } else {
        // Keep the source folder identifier; the clone still needs a
        // distinct directory, so append a suffix (numbered if taken).
        let base = source
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| hex_float::text_to_hex(new_name));
        let mut candidate = parent.join(format!("{}_copy", base));
        let mut suffix = 2;
        while candidate.exists() {
            candidate = parent.join(format!("{}_copy{}", base, suffix));
            suffix += 1;
        }
        candidate
    };

    // 1. ZIP-Backup
    if options.backup {
        create_zip_backup(source, parent)?;
    }

    // 2. Profilname aus profile.sii lesen und neuen Inhalt vorbereiten.
    // Only when replace_text is set; otherwise the clone keeps the
    // source's internal profile name untouched.
    let updated_profile_sii = if options.replace_text {
        let source_profile_sii = source.join("profile.sii");
        if !source_profile_sii.exists() {
            return Err("profile.sii nicht gefunden".to_string());
        }
        let decrypted =
            decrypt_if_needed(&source_profile_sii).map_err(|e| e.to_string())?;
        Some(change_profile_name_in_sii(&decrypted, new_name)?)
    } else {
        None
    };

    // 3. Profil kopieren und profile.sii aktualisieren
    if let Err(e) = copy_dir_recursive(source, &target_dir) {
        let _ = fs::remove_dir_all(&target_dir);
        return Err(e);
    }

    if let Some(updated_profile_sii) = updated_profile_sii {
        let target_profile_sii = target_dir.join("profile.sii");
        if !target_profile_sii.exists() {
            let _ = fs::remove_dir_all(&target_dir);
            return Err("profile.sii nicht gefunden".to_string());
        }

        if let Err(e) = fs::write(&target_profile_sii, updated_profile_sii) {
            let _ = fs::remove_dir_all(&target_dir);
            return Err(e.to_string());
        }
    }

    Ok(target_dir)
}

fn create_zip_backup(source: &Path, parent: &Path) -> Result<(), String> {
    let backup_root = parent.join("Save Edit Tool Profile Backups");
    fs::create_dir_all(&backup_root).map_err(|e| e.to_string())?;

    let profile_name = source.file_name().unwrap().to_string_lossy();
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let zip_path = backup_root.join(format!("{}_{}.zip", profile_name, timestamp));

    let file = File::create(zip_path).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let options: FileOptions<()> =
        FileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in WalkDir::new(source) {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = path
            .strip_prefix(source)
            .map_err(|e| e.to_string())?
            .to_string_lossy();

        if path.is_file() {
            zip.start_file(name, options).map_err(|e| e.to_string())?;
            let data = fs::read(path).map_err(|e| e.to_string())?;
            zip.write_all(&data).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

/// Rekursives Kopieren
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    for entry in WalkDir::new(src) {
        let entry = entry.map_err(|e| e.to_string())?;
        let rel = entry.path().strip_prefix(src).map_err(|e| e.to_string())?;
        let dest = dst.join(rel);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
        } else {
            fs::copy(entry.path(), &dest).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn change_profile_name_in_sii(content: &str, new_name: &str) -> Result<String, String> {
    let re = Regex::new(r#"(?m)^(?P<indent>\s*)profile_name\s*:\s*"?[^"\r\n]*"?"#)
        .map_err(|e| e.to_string())?;

    if !re.is_match(content) {
        return Err("Profilname konnte nicht gelesen werden".to_string());
    }

    Ok(re
        .replace(content, |caps: &regex::Captures| {
            format!(
                "{}profile_name: \"{}\"",
                &caps["indent"],
                escape_sii_string(new_name)
            )
        })
        .to_string())
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
