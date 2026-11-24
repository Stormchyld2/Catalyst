//! Tauri commands for the authoritative Catalyst project filesystem root.

use std::path::PathBuf;

use tauri::State;
use uuid::Uuid;

use crate::project_root::{self as native_project_root, ProjectPaths};
use crate::project_discovery;
use crate::project_registry;
use crate::models::{PlanningStatus, Project, SceneStatus, SceneType};
use crate::write_projection::{self, WriteProjection};

use super::AppState;


#[tauri::command]
pub async fn create_file_project(
    root_path: String,
    title: String,
    project_type: String,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let root = PathBuf::from(root_path);
    let project = project_discovery::create_file_project(&root, &title, &project_type)?;
    project_registry::remember(&state.app_data_dir, project.id, &root)?;
    Ok(project)
}

#[tauri::command]
pub async fn open_project_folder(
    root_path: String,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    if root_path.trim().is_empty() {
        return Err("root_path must not be empty.".into());
    }

    // A folder's manifest is the sole source of project identity and metadata.
    // No SQLite read, merge, mirror or fallback is performed here.
    let root = PathBuf::from(&root_path);
    let project = project_discovery::project_from_root(&root)?;
    project_registry::remember(&state.app_data_dir, project.id, &root)?;
    Ok(project)
}

#[tauri::command]
pub async fn initialize_project_root(
    project_id: String,
    root_path: String,
    state: State<'_, AppState>,
) -> Result<ProjectPaths, String> {
    let project_id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    if root_path.trim().is_empty() {
        return Err("root_path must not be empty.".into());
    }

    let root = PathBuf::from(root_path);
    // Existing file-backed projects can be initialized idempotently. Projects
    // without a manifest must use the dedicated creation/migration workflow;
    // do not invent their identity or overwrite legacy content.
    let project = project_discovery::project_from_root(&root)?;
    if project.id != project_id {
        return Err("Selected folder belongs to a different Catalyst project.".into());
    }
    let paths = native_project_root::initialize_project_root(&root, &project)?;
    project_registry::remember(&state.app_data_dir, project_id, &root)?;
    Ok(paths)
}

/// Resolve only through the verified file-backed location registry.
/// A missing entry requires explicitly opening the project folder again.
fn resolve_registered_root(state: &AppState, id: Uuid) -> Result<Option<PathBuf>, String> {
    project_registry::lookup(&state.app_data_dir, id)
}

#[tauri::command]
pub async fn get_project_root(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    Ok(resolve_registered_root(&state, id)?
        .map(|root| root.to_string_lossy().into_owned()))
}

#[tauri::command]
pub async fn get_project_paths(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<ProjectPaths, String> {
    let id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let root = resolve_registered_root(&state, id)?
        .ok_or_else(|| "Project does not have a registered Catalyst project root yet.".to_string())?;
    let project = project_discovery::project_from_root(&root)?;
    native_project_root::resolve_project_paths(&root, &project)
}


#[tauri::command]
pub async fn get_write_projection(
    project_root: String,
    project_id: String,
) -> Result<WriteProjection, String> {
    if project_root.trim().is_empty() {
        return Err("project_root must not be empty.".into());
    }
    let project_id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    write_projection::build_write_projection(&PathBuf::from(project_root), project_id)
}


#[derive(serde::Deserialize)]
pub struct FileSceneMetadataUpdate {
    pub scene_type: Option<SceneType>,
    pub scene_status: Option<SceneStatus>,
    pub planning_status: Option<PlanningStatus>,
}

#[tauri::command]
pub async fn update_file_scene_metadata(
    project_root: String,
    scene_id: String,
    metadata: FileSceneMetadataUpdate,
) -> Result<(), String> {
    let scene_id = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    write_projection::update_scene_metadata(
        &PathBuf::from(project_root),
        scene_id,
        metadata.scene_type,
        metadata.scene_status,
        metadata.planning_status,
    )
}

#[tauri::command]
pub async fn create_file_chapter(
    project_root: String,
    project_id: String,
    title: String,
    synopsis: String,
) -> Result<WriteProjection, String> {
    let project_id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let root = PathBuf::from(project_root);
    write_projection::append_planned_chapter(&root, project_id, &title, &synopsis)?;
    write_projection::build_write_projection(&root, project_id)
}

#[tauri::command]
pub async fn reorder_file_chapters(
    project_root: String,
    project_id: String,
    chapter_ids: Vec<String>,
) -> Result<WriteProjection, String> {
    let project_id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    write_projection::reorder_file_chapters(&PathBuf::from(project_root), project_id, &chapter_ids)
}

#[tauri::command]
pub async fn create_file_scene(
    project_root: String,
    project_id: String,
    chapter_title: String,
    title: String,
    synopsis: String,
    pov: String,
    location: String,
    purpose: String,
    reveal: String,
) -> Result<WriteProjection, String> {
    let project_id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let root = PathBuf::from(project_root);
    write_projection::append_planned_scene(
        &root,
        &chapter_title,
        &title,
        &synopsis,
        &pov,
        &location,
        &purpose,
        &reveal,
    )?;
    write_projection::build_write_projection(&root, project_id)
}

#[tauri::command]
pub async fn save_file_chapter_prose(
    project_root: String,
    project_id: String,
    chapter_id: String,
    prose_html: String,
) -> Result<WriteProjection, String> {
    let project_id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let chapter_id = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let root = PathBuf::from(project_root);
    let before_projection = write_projection::build_write_projection(&root, project_id)?;
    let (before, _, _) = write_projection::projection_word_counts(&before_projection);
    let projection = write_projection::save_chapter_prose(&root, project_id, chapter_id, &prose_html)?;
    let (after, _, _) = write_projection::projection_word_counts(&projection);
    crate::writing_statistics::record_delta(&root, after - before)?;
    Ok(projection)
}

#[tauri::command]
pub async fn save_file_scene_prose(
    project_root: String,
    project_id: String,
    scene_id: String,
    prose_html: String,
) -> Result<WriteProjection, String> {
    let project_id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let scene_id = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    write_projection::save_scene_prose(
        &PathBuf::from(project_root),
        project_id,
        scene_id,
        &prose_html,
    )
}


#[tauri::command]
pub async fn get_file_writing_stats(
    project_root: String,
    project_id: String,
) -> Result<crate::writing_statistics::FileWritingStats, String> {
    let project_id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let root = PathBuf::from(project_root);
    let projection = write_projection::build_write_projection(&root, project_id)?;
    crate::writing_statistics::stats(&root, project_id, &projection)
}

#[tauri::command]
pub async fn reset_file_writing_session(project_root: String) -> Result<(), String> {
    crate::writing_statistics::reset_session(&PathBuf::from(project_root))
}


/// Resolve an Outline operation against the registered project identity rather
/// than accepting an arbitrary filesystem path from the frontend.
fn registered_outline_root(state: &AppState, project_id: Uuid) -> Result<PathBuf, String> {
    let root = resolve_registered_root(state, project_id)?
        .ok_or_else(|| "Open the Catalyst project folder before editing its outline.".to_string())?;
    let project = project_discovery::project_from_root(&root)?;
    if project.id != project_id {
        return Err("Registered folder belongs to a different Catalyst project.".into());
    }
    Ok(root)
}

#[derive(serde::Serialize)]
pub struct FileOutlineSnapshot {
    pub outline: crate::outline::Outline,
    pub source_hash: String,
}

#[tauri::command]
pub async fn get_file_outline(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Option<FileOutlineSnapshot>, String> {
    let id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let root = registered_outline_root(&state, id)?;
    Ok(crate::outline::load(&root)?.map(|(outline, source_hash)| FileOutlineSnapshot {
        outline,
        source_hash,
    }))
}

#[tauri::command]
pub async fn initialize_file_outline(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<FileOutlineSnapshot, String> {
    let id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let root = registered_outline_root(&state, id)?;
    let write = write_projection::build_write_projection(&root, id)?;
    crate::outline::initialize_from_write(&root, &write)?;
    let (outline, source_hash) = crate::outline::load(&root)?
        .ok_or_else(|| "Outline initialization did not produce a file.".to_string())?;
    Ok(FileOutlineSnapshot { outline, source_hash })
}

#[tauri::command]
pub async fn save_file_outline(
    project_id: String,
    outline: crate::outline::Outline,
    expected_hash: String,
    state: State<'_, AppState>,
) -> Result<FileOutlineSnapshot, String> {
    let id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let root = registered_outline_root(&state, id)?;
    if expected_hash.trim().is_empty() {
        return Err("An existing Outline source hash is required to save changes.".into());
    }
    // Refuse structural mutations when the Write identities are not a safe
    // one-to-one match. Cross-chapter moves of manuscript-backed scenes require
    // a separate prose-preserving file migration.
    let write = write_projection::build_write_projection(&root, id)?;
    let audit = crate::outline::audit_write_identity(&outline, &write);
    if !audit.safe_to_link() {
        return Err("Outline edits conflict with Write chapter/scene identities or parents; manuscript migration is required.".into());
    }
    let source_hash = crate::outline::save(&root, &outline, Some(&expected_hash))?;
    Ok(FileOutlineSnapshot { outline, source_hash })
}
