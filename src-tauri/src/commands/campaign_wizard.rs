//! Tauri boundary for the local-file New Campaign Wizard seeder.

use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

use super::AppState;
use crate::campaign_wizard::{self, NewCampaignInput, NewCampaignProject};
use crate::project_registry;

#[tauri::command]
pub async fn create_new_campaign(
    input: NewCampaignInput,
    state: State<'_, AppState>,
) -> Result<NewCampaignProject, String> {
    let created = campaign_wizard::create_campaign(&input)?;
    let id = Uuid::parse_str(&created.manifest.project_id)
        .map_err(|e| format!("Invalid new campaign identity: {e}"))?;
    // The campaign is already durable on disk. Register its verified location
    // so Home and reopen work without SQLite.
    project_registry::remember(&state.app_data_dir, id, &PathBuf::from(&created.paths.root))?;
    Ok(created)
}
