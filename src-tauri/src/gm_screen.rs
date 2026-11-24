//! GM Screen layout (doc 034): one small JSON file in the campaign,
//! `Game Master/gm-screen.json`. It holds only the screen's own state (which
//! panels show, which pad tag the soundboard panel uses); pads, rolls and
//! records stay in their own files. Unknown fields are kept as written.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::commands::staging::replace_file_atomically;

pub const LAYOUT_FILE: &str = "Game Master/gm-screen.json";
pub const LAYOUT_FORMAT: &str = "catalyst-gm-screen";
pub const LAYOUT_VERSION: u64 = 1;

pub fn layout_path(root: &Path) -> PathBuf {
    root.join(LAYOUT_FILE)
}

fn require_root(root: &Path) -> Result<(), String> {
    if root.as_os_str().is_empty() || !root.is_absolute() || !root.is_dir() {
        return Err("Project root must be an existing absolute path.".into());
    }
    Ok(())
}

fn validate(layout: &Value) -> Result<(), String> {
    let format = layout.get("format").and_then(Value::as_str);
    if format != Some(LAYOUT_FORMAT) {
        return Err(format!("{LAYOUT_FILE} is not a GM Screen layout."));
    }
    let version = layout.get("version").and_then(Value::as_u64).unwrap_or(0);
    if version == 0 || version > LAYOUT_VERSION {
        return Err(format!(
            "{LAYOUT_FILE} version {version} is not supported by this Catalyst ({LAYOUT_VERSION})."
        ));
    }
    if !layout.get("layouts").is_some_and(Value::is_object) {
        return Err(format!("{LAYOUT_FILE} has no layouts."));
    }
    Ok(())
}

/// A campaign without the file gets `null`; the UI then uses its default layout.
pub fn load_layout(root: &Path) -> Result<Value, String> {
    require_root(root)?;
    match fs::read_to_string(layout_path(root)) {
        Ok(text) => {
            let layout: Value = serde_json::from_str(&text)
                .map_err(|e| format!("{LAYOUT_FILE} could not be read: {e}"))?;
            validate(&layout)?;
            Ok(layout)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Null),
        Err(e) => Err(format!("Could not read {LAYOUT_FILE}: {e}")),
    }
}

/// Atomically write the layout. A file on disk from a newer Catalyst is
/// refused rather than overwritten.
pub fn save_layout(root: &Path, layout: Value) -> Result<(), String> {
    require_root(root)?;
    validate(&layout)?;
    let path = layout_path(root);
    if let Ok(text) = fs::read_to_string(&path) {
        let existing: Value = serde_json::from_str(&text).unwrap_or(json!(null));
        if existing.get("version").and_then(Value::as_u64).unwrap_or(0) > LAYOUT_VERSION {
            return Err(format!("{LAYOUT_FILE} was written by a newer Catalyst; not overwritten."));
        }
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("Could not create Game Master folder: {e}"))?;
    }
    let mut text = serde_json::to_string_pretty(&layout).map_err(|e| e.to_string())?;
    text.push('\n');
    replace_file_atomically(&path, |file| {
        file.write_all(text.as_bytes())
            .map_err(|e| format!("Could not write {LAYOUT_FILE}: {e}"))
    })
}

#[tauri::command]
pub async fn load_gm_screen(project_root: String) -> Result<Value, String> {
    load_layout(&PathBuf::from(project_root))
}

#[tauri::command]
pub async fn save_gm_screen(project_root: String, layout: Value) -> Result<(), String> {
    save_layout(&PathBuf::from(project_root), layout)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Value {
        json!({
            "format": LAYOUT_FORMAT,
            "version": 1,
            "active_layout": "default",
            "layouts": { "default": { "panels": [{ "id": "soundboard", "visible": true, "pad_tag": "Combat" }] } },
            "future_field": { "kept": true }
        })
    }

    #[test]
    fn missing_file_loads_as_null() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load_layout(dir.path()).unwrap(), Value::Null);
    }

    #[test]
    fn layout_round_trips_and_keeps_unknown_fields() {
        let dir = tempfile::tempdir().unwrap();
        save_layout(dir.path(), sample()).unwrap();
        assert!(dir.path().join("Game Master").is_dir());
        assert_eq!(load_layout(dir.path()).unwrap(), sample());
    }

    #[test]
    fn wrong_format_and_newer_versions_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let mut bad = sample();
        bad["format"] = json!("something-else");
        assert!(save_layout(dir.path(), bad).is_err());

        fs::create_dir_all(dir.path().join("Game Master")).unwrap();
        let mut newer = sample();
        newer["version"] = json!(2);
        fs::write(layout_path(dir.path()), newer.to_string()).unwrap();
        assert!(load_layout(dir.path()).is_err());
        assert!(save_layout(dir.path(), sample()).is_err());
        let on_disk: Value = serde_json::from_str(&fs::read_to_string(layout_path(dir.path())).unwrap()).unwrap();
        assert_eq!(on_disk["version"], json!(2));
    }
}
