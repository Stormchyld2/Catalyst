//! Campaign soundboard: file-authoritative audio library and pad layout.
//!
//! Audio lives in the campaign's own `Assets/Audio/` folder and the pad
//! layout is an ordinary JSON file next to it (`Assets/Audio/soundboard.json`).
//! There is no database, no app-data copy and no network access: deleting the
//! layout file only resets the board, and the audio files stay usable in any
//! other player.

use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::commands::staging::replace_file_atomically;

/// Campaign folder that holds soundboard audio, relative to the project root.
pub const AUDIO_DIR: &str = "Assets/Audio";
/// Layout file name inside [`AUDIO_DIR`].
pub const LAYOUT_FILE: &str = "soundboard.json";
pub const LAYOUT_FORMAT: &str = "catalyst-soundboard";
pub const LAYOUT_VERSION: u32 = 1;

/// Formats the bundled webview can decode on every desktop platform.
const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "ogg", "oga", "opus", "wav", "flac", "m4a", "aac", "webm",
];

/// Upper bound on the number of files listed, so a mis-pointed folder cannot
/// stall the UI.
const MAX_LISTED_FILES: usize = 5000;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AudioFile {
    /// Path relative to [`AUDIO_DIR`], always with forward slashes.
    pub path: String,
    pub name: String,
    pub size: u64,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Music,
    Ambient,
    #[default]
    Effects,
}

fn default_volume() -> f32 {
    1.0
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pad {
    pub id: String,
    pub label: String,
    /// Audio file relative to [`AUDIO_DIR`].
    pub file: String,
    #[serde(default)]
    pub channel: Channel,
    #[serde(default, rename = "loop")]
    pub looping: bool,
    #[serde(default = "default_volume")]
    pub volume: f32,
    /// Fields written by newer versions or by hand are kept on save.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChannelSettings {
    #[serde(default = "default_volume")]
    pub volume: f32,
}

impl Default for ChannelSettings {
    fn default() -> Self {
        Self { volume: 1.0 }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Channels {
    #[serde(default)]
    pub music: ChannelSettings,
    #[serde(default)]
    pub ambient: ChannelSettings,
    #[serde(default)]
    pub effects: ChannelSettings,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Layout {
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_master")]
    pub master_volume: f32,
    #[serde(default)]
    pub channels: Channels,
    #[serde(default)]
    pub pads: Vec<Pad>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

fn default_format() -> String {
    LAYOUT_FORMAT.into()
}
fn default_version() -> u32 {
    LAYOUT_VERSION
}
fn default_master() -> f32 {
    0.8
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            format: default_format(),
            version: LAYOUT_VERSION,
            master_volume: default_master(),
            channels: Channels::default(),
            pads: Vec::new(),
            extra: serde_json::Map::new(),
        }
    }
}

fn clamp_volume(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

impl Layout {
    /// Clamp volumes and drop pads that would point outside the audio folder.
    fn normalized(mut self) -> Result<Self, String> {
        if self.format != LAYOUT_FORMAT {
            return Err(format!(
                "Soundboard layout format '{}' is not recognized.",
                self.format
            ));
        }
        if self.version > LAYOUT_VERSION {
            return Err(format!(
                "Soundboard layout version {} is newer than this Catalyst supports ({}).",
                self.version, LAYOUT_VERSION
            ));
        }
        self.version = LAYOUT_VERSION;
        self.master_volume = clamp_volume(self.master_volume);
        for c in [
            &mut self.channels.music,
            &mut self.channels.ambient,
            &mut self.channels.effects,
        ] {
            c.volume = clamp_volume(c.volume);
        }
        let mut seen = std::collections::HashSet::new();
        for pad in &mut self.pads {
            pad.volume = clamp_volume(pad.volume);
            pad.file = pad.file.replace('\\', "/");
            if pad.file.starts_with("builtin:") {
                if !is_builtin_sound(&pad.file) {
                    return Err(format!("'{}' is not a built-in sound.", pad.file));
                }
            } else {
                validate_relative_audio_path(&pad.file)?;
            }
            if pad.id.trim().is_empty() {
                return Err("Every soundboard pad needs an id.".into());
            }
            if !seen.insert(pad.id.clone()) {
                return Err(format!("Duplicate soundboard pad id '{}'.", pad.id));
            }
        }
        Ok(self)
    }
}

fn require_absolute(root: &Path) -> Result<(), String> {
    if root.as_os_str().is_empty() || !root.is_absolute() {
        return Err("Project root must be an absolute path.".into());
    }
    Ok(())
}

pub fn audio_dir(root: &Path) -> PathBuf {
    root.join(AUDIO_DIR)
}

pub fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| AUDIO_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn audio_mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("mp3") => "audio/mpeg",
        Some("ogg") | Some("oga") => "audio/ogg",
        Some("opus") => "audio/ogg; codecs=opus",
        Some("wav") => "audio/wav",
        Some("flac") => "audio/flac",
        Some("m4a") | Some("aac") => "audio/mp4",
        Some("webm") => "audio/webm",
        _ => "application/octet-stream",
    }
}

/// Validate a path relative to [`AUDIO_DIR`]: plain components only, an
/// audio extension, and nothing that could climb out of the folder.
pub fn validate_relative_audio_path(relative: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(relative.replace('\\', "/"));
    if relative.trim().is_empty() || path.is_absolute() {
        return Err("Audio path must be relative to the campaign's Assets/Audio folder.".into());
    }
    for component in path.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err("Audio path must stay inside the campaign's Assets/Audio folder.".into());
        }
    }
    if !is_audio_file(&path) {
        return Err(format!("'{relative}' is not a supported audio file."));
    }
    Ok(path)
}

/// Built-in sounds ship with the app (`static/soundboard/`) and are referenced
/// as `builtin:<name>.ogg`; they are never read from the campaign folder.
pub fn is_builtin_sound(file: &str) -> bool {
    file.strip_prefix("builtin:").is_some_and(|name| {
        name.ends_with(".ogg")
            && name.len() > 4
            && name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.')
    })
}

/// Resolve a pad's file to a real path inside the campaign's audio folder,
/// refusing symlinks that point elsewhere.
pub fn resolve_audio_file(root: &Path, relative: &str) -> Result<PathBuf, String> {
    require_absolute(root)?;
    let rel = validate_relative_audio_path(relative)?;
    let dir = audio_dir(root);
    let full = dir.join(&rel);
    let canonical_dir = dir
        .canonicalize()
        .map_err(|_| "This campaign has no Assets/Audio folder yet.".to_string())?;
    let canonical = full
        .canonicalize()
        .map_err(|_| format!("Audio file '{relative}' is missing from Assets/Audio."))?;
    if !canonical.starts_with(&canonical_dir) || !canonical.is_file() {
        return Err("Audio path must stay inside the campaign's Assets/Audio folder.".into());
    }
    Ok(canonical)
}

pub fn read_audio_file(root: &Path, relative: &str) -> Result<Vec<u8>, String> {
    let path = resolve_audio_file(root, relative)?;
    fs::read(&path).map_err(|e| format!("Could not read audio file '{relative}': {e}"))
}

/// List the audio files under `Assets/Audio/`, sorted by path. A missing
/// folder is an empty library, not an error.
pub fn list_audio_files(root: &Path) -> Result<Vec<AudioFile>, String> {
    require_absolute(root)?;
    let dir = audio_dir(root);
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Ok(out);
    }
    let mut stack = vec![dir.clone()];
    while let Some(current) = stack.pop() {
        let entries = match fs::read_dir(&current) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                stack.push(path);
            } else if file_type.is_file() && is_audio_file(&path) {
                let Ok(relative) = path.strip_prefix(&dir) else {
                    continue;
                };
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                out.push(AudioFile {
                    path: relative.to_string_lossy().replace('\\', "/"),
                    name,
                    size,
                });
                if out.len() >= MAX_LISTED_FILES {
                    break;
                }
            }
        }
    }
    out.sort_by_key(|f| f.path.to_lowercase());
    Ok(out)
}

pub fn layout_path(root: &Path) -> PathBuf {
    audio_dir(root).join(LAYOUT_FILE)
}

/// Read the board layout. `None` means the campaign has never saved one, so
/// the UI starts it with the built-in pads.
pub fn load_layout(root: &Path) -> Result<Option<Layout>, String> {
    require_absolute(root)?;
    let path = layout_path(root);
    match fs::read_to_string(&path) {
        Ok(text) => {
            let layout: Layout = serde_json::from_str(&text)
                .map_err(|e| format!("Assets/Audio/{LAYOUT_FILE} could not be read: {e}"))?;
            layout.normalized().map(Some)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Could not read Assets/Audio/{LAYOUT_FILE}: {e}")),
    }
}

/// Validate and atomically write the board layout, creating `Assets/Audio/`
/// if needed. Returns the layout as saved.
pub fn save_layout(root: &Path, layout: Layout) -> Result<Layout, String> {
    require_absolute(root)?;
    if !root.is_dir() {
        return Err("Project root does not exist.".into());
    }
    let layout = layout.normalized()?;
    let dir = audio_dir(root);
    fs::create_dir_all(&dir).map_err(|e| format!("Could not create Assets/Audio: {e}"))?;
    let mut text = serde_json::to_string_pretty(&layout).map_err(|e| e.to_string())?;
    text.push('\n');
    replace_file_atomically(&layout_path(root), |file| {
        file.write_all(text.as_bytes())
            .map_err(|e| format!("Could not write soundboard layout: {e}"))
    })?;
    Ok(layout)
}

/// Copy audio files chosen by the user into `Assets/Audio/`. Existing files
/// are never overwritten: a clash gets a " (2)", " (3)"… suffix. Returns the
/// imported files' paths relative to the audio folder.
pub fn import_audio_files(root: &Path, sources: &[PathBuf]) -> Result<Vec<String>, String> {
    require_absolute(root)?;
    if !root.is_dir() {
        return Err("Project root does not exist.".into());
    }
    let dir = audio_dir(root);
    fs::create_dir_all(&dir).map_err(|e| format!("Could not create Assets/Audio: {e}"))?;
    let mut imported = Vec::new();
    for source in sources {
        if !source.is_file() || !is_audio_file(source) {
            return Err(format!(
                "'{}' is not a supported audio file.",
                source.display()
            ));
        }
        let stem = source
            .file_stem()
            .and_then(|s| s.to_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("Sound")
            .to_string();
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp3")
            .to_ascii_lowercase();
        let mut name = format!("{stem}.{ext}");
        let mut n = 2;
        while dir.join(&name).exists() {
            name = format!("{stem} ({n}).{ext}");
            n += 1;
        }
        let target = dir.join(&name);
        // create_new guards against a race with another writer.
        let mut out = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
            .map_err(|e| format!("Could not create '{}': {e}", target.display()))?;
        let mut input = fs::File::open(source)
            .map_err(|e| format!("Could not read '{}': {e}", source.display()))?;
        if let Err(e) = std::io::copy(&mut input, &mut out).and_then(|_| out.sync_all()) {
            drop(out);
            let _ = fs::remove_file(&target);
            return Err(format!("Could not copy '{}': {e}", source.display()));
        }
        imported.push(name);
    }
    Ok(imported)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn pad(id: &str, file: &str) -> Pad {
        Pad {
            id: id.into(),
            label: id.into(),
            file: file.into(),
            channel: Channel::Effects,
            looping: false,
            volume: 1.0,
            extra: serde_json::Map::new(),
        }
    }

    #[test]
    fn missing_folder_is_an_empty_board() {
        let dir = root();
        assert!(list_audio_files(dir.path()).unwrap().is_empty());
        assert_eq!(load_layout(dir.path()).unwrap(), None);
    }

    #[test]
    fn lists_audio_recursively_and_skips_other_files() {
        let dir = root();
        let audio = audio_dir(dir.path());
        fs::create_dir_all(audio.join("Music")).unwrap();
        fs::write(audio.join("Music/Tavern.OGG"), b"x").unwrap();
        fs::write(audio.join("door.mp3"), b"xy").unwrap();
        fs::write(audio.join("notes.txt"), b"x").unwrap();
        fs::write(audio.join(LAYOUT_FILE), b"{}").unwrap();
        let files = list_audio_files(dir.path()).unwrap();
        let paths: Vec<_> = files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, vec!["door.mp3", "Music/Tavern.OGG"]);
        assert_eq!(files[0].size, 2);
    }

    #[test]
    fn layout_round_trips_through_the_campaign_file() {
        let dir = root();
        let mut layout = Layout::default();
        let mut p = pad("pad-1", "Music/Tavern.ogg");
        p.channel = Channel::Music;
        p.looping = true;
        p.volume = 3.0;
        layout.pads.push(p);
        let saved = save_layout(dir.path(), layout).unwrap();
        assert_eq!(saved.pads[0].volume, 1.0);
        let text = fs::read_to_string(layout_path(dir.path())).unwrap();
        assert!(text.contains("\"loop\": true"));
        assert!(text.contains("\"channel\": \"music\""));
        assert_eq!(load_layout(dir.path()).unwrap(), Some(saved));
    }

    #[test]
    fn layout_rejects_escaping_paths_and_duplicate_ids() {
        let dir = root();
        let mut layout = Layout::default();
        layout.pads.push(pad("pad-1", "../secret.mp3"));
        assert!(save_layout(dir.path(), layout).is_err());

        let mut layout = Layout::default();
        layout.pads.push(pad("pad-1", "a.mp3"));
        layout.pads.push(pad("pad-1", "b.mp3"));
        assert!(save_layout(dir.path(), layout).is_err());
        assert!(!layout_path(dir.path()).exists());
    }

    #[test]
    fn builtin_sounds_are_accepted_but_never_read_from_disk() {
        let dir = root();
        let mut layout = Layout::default();
        layout.pads.push(pad("pad-1", "builtin:rain-storm.ogg"));
        assert!(save_layout(dir.path(), layout).is_ok());
        assert!(read_audio_file(dir.path(), "builtin:rain-storm.ogg").is_err());
        for bad in [
            "builtin:../x.ogg",
            "builtin:a/b.ogg",
            "builtin:Rain.ogg",
            "builtin:.ogg",
        ] {
            let mut layout = Layout::default();
            layout.pads.push(pad("pad-1", bad));
            assert!(save_layout(dir.path(), layout).is_err(), "{bad}");
        }
    }

    #[test]
    fn unknown_layout_fields_survive_a_save() {
        let dir = root();
        fs::create_dir_all(audio_dir(dir.path())).unwrap();
        fs::write(
            layout_path(dir.path()),
            r#"{"format":"catalyst-soundboard","version":1,"scene":"storm","pads":[{"id":"pad-1","label":"Rain","file":"rain.mp3","color":"blue"}]}"#,
        )
        .unwrap();
        let layout = load_layout(dir.path()).unwrap().unwrap();
        save_layout(dir.path(), layout).unwrap();
        let text = fs::read_to_string(layout_path(dir.path())).unwrap();
        assert!(text.contains("\"scene\": \"storm\""));
        assert!(text.contains("\"color\": \"blue\""));
    }

    #[test]
    fn layout_from_newer_version_is_refused() {
        let dir = root();
        fs::create_dir_all(audio_dir(dir.path())).unwrap();
        fs::write(
            layout_path(dir.path()),
            r#"{"format":"catalyst-soundboard","version":99,"pads":[]}"#,
        )
        .unwrap();
        assert!(load_layout(dir.path()).is_err());
    }

    #[test]
    fn reading_audio_stays_inside_the_audio_folder() {
        let dir = root();
        let audio = audio_dir(dir.path());
        fs::create_dir_all(&audio).unwrap();
        fs::write(audio.join("door.mp3"), b"abc").unwrap();
        fs::write(dir.path().join("outside.mp3"), b"no").unwrap();
        assert_eq!(read_audio_file(dir.path(), "door.mp3").unwrap(), b"abc");
        assert!(read_audio_file(dir.path(), "../outside.mp3").is_err());
        assert!(read_audio_file(dir.path(), "/etc/passwd.mp3").is_err());
        assert!(read_audio_file(dir.path(), "missing.mp3").is_err());
        assert!(read_audio_file(dir.path(), "project.yaml").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_out_of_the_audio_folder_are_refused() {
        let dir = root();
        let audio = audio_dir(dir.path());
        fs::create_dir_all(&audio).unwrap();
        fs::write(dir.path().join("outside.mp3"), b"no").unwrap();
        std::os::unix::fs::symlink(dir.path().join("outside.mp3"), audio.join("link.mp3")).unwrap();
        assert!(read_audio_file(dir.path(), "link.mp3").is_err());
        assert!(list_audio_files(dir.path()).unwrap().is_empty());
    }

    #[test]
    fn import_copies_without_overwriting() {
        let dir = root();
        let src = root();
        let source = src.path().join("Rain.wav");
        fs::write(&source, b"rain").unwrap();
        let first = import_audio_files(dir.path(), &[source.clone()]).unwrap();
        let second = import_audio_files(dir.path(), &[source.clone()]).unwrap();
        assert_eq!(first, vec!["Rain.wav"]);
        assert_eq!(second, vec!["Rain (2).wav"]);
        assert_eq!(
            fs::read(audio_dir(dir.path()).join("Rain.wav")).unwrap(),
            b"rain"
        );
        let bad = src.path().join("notes.txt");
        fs::write(&bad, b"x").unwrap();
        assert!(import_audio_files(dir.path(), &[bad]).is_err());
    }
}
