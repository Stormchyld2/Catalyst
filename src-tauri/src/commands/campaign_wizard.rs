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

#[derive(Clone, Debug, serde::Serialize)]
pub struct CampaignDashboardSummary {
    pub ruleset: String,
    pub edition: Option<String>,
    pub starting_level: Option<String>,
    pub party_name: Option<String>,
    pub expected_players: Option<String>,
    pub play_mode: String,
    pub session_schedule: Option<String>,
    pub sessions_planned: usize,
    pub sessions_logged: usize,
    pub adventure_records: usize,
    pub encounters: usize,
    pub npcs: usize,
    pub world_entries: usize,
    pub latest_session_plan: Option<String>,
}

fn markdown_files(root: &std::path::Path, folder: &str) -> Result<Vec<std::path::PathBuf>, String> {
    let base = root.join(folder);
    if !base.exists() { return Ok(Vec::new()); }
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(base).follow_links(false).max_depth(12) {
        let entry = entry.map_err(|e| format!("Cannot read campaign folder {folder}: {e}"))?;
        if entry.file_type().is_file()
            && entry.path().extension().and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("md")).unwrap_or(false) {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

/// Read actual campaign YAML and file-backed records without manuscript metrics.
#[tauri::command]
pub async fn get_campaign_dashboard_summary(
    project_root: String,
) -> Result<CampaignDashboardSummary, String> {
    let root = std::path::PathBuf::from(project_root);
    if !root.is_absolute() { return Err("Campaign path must be absolute.".into()); }
    let campaign = campaign_wizard::read_campaign_manifest(&root)?;
    let plans = markdown_files(&root, "Sessions/Session Plans")?;
    let latest_session_plan = plans.last()
        .and_then(|p| p.file_stem().and_then(|s| s.to_str()))
        .map(str::to_string);
    Ok(CampaignDashboardSummary {
        ruleset: campaign.rules.custom_system_name
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| campaign.rules.pack_id.clone()),
        edition: campaign.rules.edition,
        starting_level: campaign.rules.starting_level,
        party_name: campaign.party.name,
        expected_players: campaign.party.expected_players,
        play_mode: campaign.play.mode,
        session_schedule: campaign.play.schedule,
        sessions_planned: plans.len(),
        sessions_logged: markdown_files(&root, "Sessions/Session Logs")?.len(),
        adventure_records: markdown_files(&root, "Adventures")?.len(),
        encounters: markdown_files(&root, "Encounters")?.len(),
        npcs: markdown_files(&root, "Characters/Non-Player Characters")?.len(),
        world_entries: markdown_files(&root, "Worldbuilding")?.len(),
        latest_session_plan,
    })
}
