//! One-time import of safe app settings written under the pre-Catalyst identifier.
//!
//! Tauri derives the app data directory from the bundle identifier, which the
//! rebrand changed from `com.lithomark.writer` to `com.catalyst.writer`.
//! Catalyst is an independent product, so it deliberately does NOT adopt the
//! LithoMark database, remembered project list, snapshots or templates: doing so
//! would point Catalyst at LithoMark's project folders and let it write to them.
//!
//! Only `settings.json` (author name and contact details, no paths) is imported,
//! once, when Catalyst has no settings of its own. The legacy folder is only read.

use std::fs;
use std::io;
use std::path::Path;

use crate::models::AppSettings;

/// Bundle identifier the app used before it was renamed Catalyst.
pub const LEGACY_IDENTIFIER: &str = "com.lithomark.writer";

const SETTINGS_FILE: &str = "settings.json";

/// Whether startup should look for legacy settings at all. Visual QA runs and
/// debug runs with a `CATALYST_DATA_DIR` override use scratch storage and must
/// never read from or write to the user's real app data.
pub fn should_import(qa_background: bool, data_dir_overridden: bool) -> bool {
    !qa_background && !data_dir_overridden
}

/// Copies the legacy `settings.json` into `app_data_dir` when Catalyst has no
/// settings file yet and the legacy file is valid. Returns whether it copied.
pub fn import_legacy_settings(app_data_dir: &Path) -> io::Result<bool> {
    let target = app_data_dir.join(SETTINGS_FILE);
    if target.exists() {
        return Ok(false);
    }
    let Some(parent) = app_data_dir.parent() else {
        return Ok(false);
    };
    let legacy_dir = parent.join(LEGACY_IDENTIFIER);
    if legacy_dir == app_data_dir {
        return Ok(false);
    }
    let legacy = legacy_dir.join(SETTINGS_FILE);
    if !legacy.is_file() {
        return Ok(false);
    }
    let contents = fs::read(&legacy)?;
    // Never carry an unreadable file over; Catalyst would refuse to load it.
    if serde_json::from_slice::<AppSettings>(&contents).is_err() {
        return Ok(false);
    }

    fs::create_dir_all(app_data_dir)?;
    let staging = app_data_dir.join(format!("{SETTINGS_FILE}.importing"));
    fs::write(&staging, &contents)?;
    if let Err(error) = fs::rename(&staging, &target) {
        let _ = fs::remove_file(&staging);
        return Err(error);
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SETTINGS: &[u8] = br#"{"author_name":"A. Writer","contact_email":"a@example.com"}"#;

    fn seed_legacy(root: &Path, settings: &[u8]) {
        let legacy = root.join(LEGACY_IDENTIFIER);
        fs::create_dir_all(legacy.join("snapshots/p1")).unwrap();
        fs::write(legacy.join("kindling.db"), b"db").unwrap();
        fs::write(legacy.join("project-registry.json"), b"{}").unwrap();
        fs::write(legacy.join("snapshots/p1/s.json"), b"snap").unwrap();
        fs::write(legacy.join(SETTINGS_FILE), settings).unwrap();
    }

    #[test]
    fn imports_only_settings_and_leaves_projects_behind() {
        let temp = tempfile::tempdir().unwrap();
        seed_legacy(temp.path(), SETTINGS);
        let target = temp.path().join("com.catalyst.writer");

        assert!(import_legacy_settings(&target).unwrap());

        assert_eq!(fs::read(target.join(SETTINGS_FILE)).unwrap(), SETTINGS);
        let mut names: Vec<_> = fs::read_dir(&target)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        assert_eq!(names, vec![SETTINGS_FILE.to_string()]);
        // The legacy folder is left exactly as it was.
        let legacy = temp.path().join(LEGACY_IDENTIFIER);
        assert_eq!(fs::read(legacy.join(SETTINGS_FILE)).unwrap(), SETTINGS);
        assert_eq!(fs::read(legacy.join("kindling.db")).unwrap(), b"db");
    }

    #[test]
    fn keeps_existing_catalyst_settings() {
        let temp = tempfile::tempdir().unwrap();
        seed_legacy(temp.path(), SETTINGS);
        let target = temp.path().join("com.catalyst.writer");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join(SETTINGS_FILE), b"{}").unwrap();

        assert!(!import_legacy_settings(&target).unwrap());
        assert_eq!(fs::read(target.join(SETTINGS_FILE)).unwrap(), b"{}");
    }

    #[test]
    fn skips_invalid_legacy_settings() {
        let temp = tempfile::tempdir().unwrap();
        seed_legacy(temp.path(), b"not json");
        let target = temp.path().join("com.catalyst.writer");

        assert!(!import_legacy_settings(&target).unwrap());
        assert!(!target.exists());
    }

    #[test]
    fn does_nothing_without_legacy_settings() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("com.catalyst.writer");

        assert!(!import_legacy_settings(&target).unwrap());
        assert!(!target.exists());
    }

    #[test]
    fn never_imports_into_overridden_or_qa_storage() {
        assert!(should_import(false, false));
        assert!(!should_import(false, true));
        assert!(!should_import(true, false));
        assert!(!should_import(true, true));
    }
}
