//! Tauri boundary for the campaign soundboard. All state lives in the
//! campaign folder (`Assets/Audio/`); see [`crate::soundboard`].

use std::path::PathBuf;

use tauri::ipc::Response;

use crate::soundboard::{self, AudioFile, Layout};

#[tauri::command]
pub async fn list_soundboard_audio(project_root: String) -> Result<Vec<AudioFile>, String> {
    soundboard::list_audio_files(&PathBuf::from(project_root))
}

/// Raw audio bytes, sent as a binary IPC payload rather than a JSON array so
/// long music tracks stay cheap to load.
#[tauri::command]
pub async fn read_soundboard_audio(project_root: String, path: String) -> Result<Response, String> {
    let bytes = soundboard::read_audio_file(&PathBuf::from(project_root), &path)?;
    Ok(Response::new(bytes))
}

#[tauri::command]
pub async fn load_soundboard_layout(project_root: String) -> Result<Option<Layout>, String> {
    soundboard::load_layout(&PathBuf::from(project_root))
}

#[tauri::command]
pub async fn save_soundboard_layout(
    project_root: String,
    layout: Layout,
) -> Result<Layout, String> {
    soundboard::save_layout(&PathBuf::from(project_root), layout)
}

#[tauri::command]
pub async fn import_soundboard_audio(
    project_root: String,
    source_paths: Vec<String>,
) -> Result<Vec<String>, String> {
    let sources: Vec<PathBuf> = source_paths.into_iter().map(PathBuf::from).collect();
    soundboard::import_audio_files(&PathBuf::from(project_root), &sources)
}
