//! Tauri boundary for the local-file New Book Foundation seeder.

use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

use crate::new_book_foundation::{
    self, NewBookFoundationInput, NewBookProject,
};
use crate::project_registry;
use super::AppState;

#[tauri::command]
pub async fn create_new_book_foundation(
    input: NewBookFoundationInput,
    state: State<'_, AppState>,
) -> Result<NewBookProject, String> {
    let created = new_book_foundation::create_new_book(&input)?;
    let id = Uuid::parse_str(&created.manifest.project_id)
        .map_err(|e| format!("Invalid new book identity: {e}"))?;
    // The book is already durable on disk. Register its verified location
    // so the Home screen and subsequent reopen work without SQLite.
    project_registry::remember(
        &state.app_data_dir,
        id,
        &PathBuf::from(&created.paths.root),
    )?;
    Ok(created)
}
