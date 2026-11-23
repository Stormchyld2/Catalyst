//! One-time adoption of app data written under the pre-Catalyst bundle identifier.
//!
//! Tauri derives the app data directory from the bundle identifier. The rebrand
//! changed the identifier from `com.lithomark.writer` to `com.catalyst.writer`,
//! which would otherwise leave the remembered project list, settings, snapshots,
//! entity templates and the rebuildable `kindling.db` behind in the old folder.
//!
//! Adoption is non-destructive: the legacy folder is only read, never changed or
//! removed, and nothing happens once the Catalyst folder exists.

use std::fs;
use std::io;
use std::path::Path;

/// Bundle identifier the app used before it was renamed Catalyst.
pub const LEGACY_IDENTIFIER: &str = "com.lithomark.writer";

/// Copies the sibling legacy app data folder into `app_data_dir` when
/// `app_data_dir` does not exist yet. Returns whether anything was copied.
///
/// The copy is staged next to the destination and renamed into place, so an
/// interrupted copy never leaves a half-populated Catalyst folder behind.
pub fn adopt_legacy_app_data(app_data_dir: &Path) -> io::Result<bool> {
    if app_data_dir.exists() {
        return Ok(false);
    }
    let (Some(parent), Some(name)) = (app_data_dir.parent(), app_data_dir.file_name()) else {
        return Ok(false);
    };
    let legacy = parent.join(LEGACY_IDENTIFIER);
    if legacy == app_data_dir || !legacy.is_dir() {
        return Ok(false);
    }

    let mut staging_name = name.to_os_string();
    staging_name.push(".adopting");
    let staging = parent.join(staging_name);
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }
    if let Err(error) = copy_dir(&legacy, &staging).and_then(|_| fs::rename(&staging, app_data_dir)) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    Ok(true)
}

fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let target = to.join(entry.file_name());
        if kind.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), &target)?;
        }
        // Symlinks and other special entries are not app-written data; skip them.
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed_legacy(root: &Path) {
        let legacy = root.join(LEGACY_IDENTIFIER);
        fs::create_dir_all(legacy.join("snapshots/p1")).unwrap();
        fs::write(legacy.join("kindling.db"), b"db").unwrap();
        fs::write(legacy.join("settings.json"), b"{}").unwrap();
        fs::write(legacy.join("snapshots/p1/s.json"), b"snap").unwrap();
    }

    #[test]
    fn copies_legacy_folder_when_catalyst_folder_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        seed_legacy(temp.path());
        let target = temp.path().join("com.catalyst.writer");

        assert!(adopt_legacy_app_data(&target).unwrap());

        assert_eq!(fs::read(target.join("kindling.db")).unwrap(), b"db");
        assert_eq!(fs::read(target.join("settings.json")).unwrap(), b"{}");
        assert_eq!(fs::read(target.join("snapshots/p1/s.json")).unwrap(), b"snap");
        // The legacy folder is left exactly as it was.
        assert_eq!(fs::read(temp.path().join(LEGACY_IDENTIFIER).join("kindling.db")).unwrap(), b"db");
        assert!(!temp.path().join("com.catalyst.writer.adopting").exists());
    }

    #[test]
    fn leaves_existing_catalyst_folder_alone() {
        let temp = tempfile::tempdir().unwrap();
        seed_legacy(temp.path());
        let target = temp.path().join("com.catalyst.writer");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("settings.json"), b"catalyst").unwrap();

        assert!(!adopt_legacy_app_data(&target).unwrap());

        assert_eq!(fs::read(target.join("settings.json")).unwrap(), b"catalyst");
        assert!(!target.join("kindling.db").exists());
    }

    #[test]
    fn does_nothing_without_legacy_folder() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("com.catalyst.writer");

        assert!(!adopt_legacy_app_data(&target).unwrap());
        assert!(!target.exists());
    }
}
