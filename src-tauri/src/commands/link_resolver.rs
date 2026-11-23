//! Tauri boundary for the file-derived Catalyst project link resolver.

use std::path::PathBuf;

use crate::link_resolver::{self, Backlink, LinkResolution};

#[tauri::command]
pub async fn resolve_project_link(
    project_root: String,
    query: String,
) -> Result<LinkResolution, String> {
    let index = link_resolver::build_project_link_index(&PathBuf::from(project_root))?;
    Ok(index.resolve(&query))
}

#[tauri::command]
pub async fn get_project_backlinks(
    project_root: String,
    target_id: String,
) -> Result<Vec<Backlink>, String> {
    let index = link_resolver::build_project_link_index(&PathBuf::from(project_root))?;
    Ok(index.backlinks(&target_id))
}
