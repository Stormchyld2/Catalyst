//! Campaign session dice history, stored as ordinary project files.
//!
//! ```text
//! <campaign root>/Sessions/Roll Logs/<session-id>/session.json   session metadata
//! <campaign root>/Sessions/Roll Logs/<session-id>/rolls.jsonl    one roll record per line
//!
//! One roll log per game session, in the campaign's top-level `Sessions`
//! folder that the New Campaign Wizard creates. Projects with the book layout
//! (not a campaign, no top-level `Sessions` folder) use
//! `Story Development/Sessions/Roll Logs/` instead.
//! ```
//!
//! The files are authoritative and readable without Catalyst. There is no
//! database. Records are validated before they are written: dice values must be
//! real faces and totals must add up, so history never holds an impossible roll.
//! See docs/architecture/022-DICE-RESULT-PROTOCOL.md.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// The campaign's top-level Sessions folder, as created by the New Campaign Wizard.
pub const CAMPAIGN_SESSIONS_DIR: &str = "Sessions";
/// Roll logs live in their own folder inside it, one subfolder per game session.
pub const ROLL_LOGS_DIR: &str = "Roll Logs";

/// Campaign projects (a top-level `Sessions` folder, a `campaign.yaml`, or
/// `project_type: campaign`) keep roll logs in `Sessions/Roll Logs`; book-layout projects keep
/// them in `Story Development/Sessions/Roll Logs`.
fn sessions_root(root: &Path) -> PathBuf {
    if is_campaign_layout(root) {
        root.join(CAMPAIGN_SESSIONS_DIR).join(ROLL_LOGS_DIR)
    } else {
        root.join("Story Development").join("Sessions").join(ROLL_LOGS_DIR)
    }
}

fn is_campaign_layout(root: &Path) -> bool {
    if root.join(CAMPAIGN_SESSIONS_DIR).is_dir() || root.join("campaign.yaml").is_file() {
        return true;
    }
    crate::project_root::read_project_manifest(root)
        .map(|manifest| manifest.project_type == "campaign")
        .unwrap_or(false)
}
const SESSION_FILE: &str = "session.json";
const ROLLS_FILE: &str = "rolls.jsonl";
const SESSION_PREFIX: &str = "session-";
const ROLL_PREFIX: &str = "roll-";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiceSession {
    pub id: String,
    pub title: String,
    pub started_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DieResult {
    #[serde(rename = "type")]
    pub die_type: String,
    pub value: i64,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub dropped: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiceRollRecord {
    pub id: String,
    pub session_id: Option<String>,
    pub expression: String,
    pub kind: String,
    pub dice: Vec<DieResult>,
    pub modifier: i64,
    pub natural: i64,
    pub total: i64,
    pub visibility: String,
    pub source: String,
    pub resolution: String,
    pub timestamp: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub critical: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub npc_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encounter_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiceSessionHistory {
    pub session: DiceSession,
    pub rolls: Vec<DiceRollRecord>,
    /// Lines that could not be read back (hand edits, partial writes).
    pub skipped_lines: usize,
}

fn sides(die_type: &str) -> Option<i64> {
    match die_type {
        "d4" => Some(4),
        "d6" => Some(6),
        "d8" => Some(8),
        "d10" => Some(10),
        "d12" => Some(12),
        "d20" => Some(20),
        "d100" => Some(100),
        _ => None,
    }
}

fn prefixed_uuid(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .and_then(|rest| Uuid::parse_str(rest).ok())
        .is_some_and(|id| id.get_version_num() == 4 && value.len() == prefix.len() + 36)
}

fn one_of(field: &str, value: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(format!("Invalid roll {field}: {value}"))
    }
}

/// Check that a record describes a roll that could actually have happened.
pub fn validate_record(record: &DiceRollRecord, session_id: &str) -> Result<(), String> {
    if !prefixed_uuid(&record.id, ROLL_PREFIX) {
        return Err(format!("Invalid roll id: {}", record.id));
    }
    if record.session_id.as_deref() != Some(session_id) {
        return Err("Roll belongs to a different session.".into());
    }
    if record.dice.is_empty() {
        return Err("A roll needs at least one die.".into());
    }
    for die in &record.dice {
        let max = sides(&die.die_type).ok_or_else(|| format!("Unknown die: {}", die.die_type))?;
        if die.value < 1 || die.value > max {
            return Err(format!("Invalid {} value: {}", die.die_type, die.value));
        }
    }
    let natural: i64 = record.dice.iter().filter(|d| !d.dropped).map(|d| d.value).sum();
    if natural != record.natural || natural + record.modifier != record.total {
        return Err("Roll total does not match its dice.".into());
    }
    one_of("kind", &record.kind, &["sum", "advantage", "disadvantage"])?;
    one_of("visibility", &record.visibility, &["public", "gm-private", "player-private"])?;
    one_of("source", &record.source, &["dice-tray", "gm-console", "encounter", "character", "campaign"])?;
    one_of("resolution", &record.resolution, &["physics", "random"])?;
    if let Some(critical) = &record.critical {
        one_of("critical", critical, &["success", "failure"])?;
    }
    DateTime::parse_from_rfc3339(&record.timestamp).map_err(|_| "Invalid roll timestamp.".to_string())?;
    Ok(())
}

fn project_root(raw: &str) -> Result<PathBuf, String> {
    let root = PathBuf::from(raw.trim());
    if !root.is_absolute() {
        return Err("Project folder must be an absolute local path.".into());
    }
    if !root.is_dir() {
        return Err(format!("Project folder '{}' does not exist.", root.display()));
    }
    Ok(root)
}

fn session_dir(root: &Path, session_id: &str) -> Result<PathBuf, String> {
    // The id becomes a folder name, so it must be exactly a prefixed UUID.
    if !prefixed_uuid(session_id, SESSION_PREFIX) {
        return Err(format!("Invalid session id: {session_id}"));
    }
    Ok(sessions_root(root).join(session_id))
}

fn read_session(dir: &Path) -> Result<DiceSession, String> {
    let path = dir.join(SESSION_FILE);
    let text = fs::read_to_string(&path).map_err(|e| format!("Could not read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("Could not parse {}: {e}", path.display()))
}

pub fn start_session(root: &Path, title: &str, now: DateTime<Utc>) -> Result<DiceSession, String> {
    let title = title.trim();
    let session = DiceSession {
        id: format!("{SESSION_PREFIX}{}", Uuid::new_v4()),
        title: if title.is_empty() { format!("Session {}", now.format("%Y-%m-%d")) } else { title.to_string() },
        started_at: now.to_rfc3339(),
    };
    let dir = session_dir(root, &session.id)?;
    fs::create_dir_all(&dir).map_err(|e| format!("Could not create {}: {e}", dir.display()))?;
    let text = serde_json::to_string_pretty(&session).map_err(|e| e.to_string())?;
    let temp = dir.join(format!(".{SESSION_FILE}.tmp"));
    fs::write(&temp, format!("{text}\n")).map_err(|e| format!("Could not write {}: {e}", temp.display()))?;
    fs::rename(&temp, dir.join(SESSION_FILE)).map_err(|e| format!("Could not save session: {e}"))?;
    Ok(session)
}

pub fn append_roll(root: &Path, session_id: &str, record: &DiceRollRecord) -> Result<(), String> {
    let dir = session_dir(root, session_id)?;
    read_session(&dir)?;
    validate_record(record, session_id)?;
    let line = serde_json::to_string(record).map_err(|e| e.to_string())?;
    let path = dir.join(ROLLS_FILE);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Could not open {}: {e}", path.display()))?;
    file.write_all(format!("{line}\n").as_bytes())
        .and_then(|_| file.sync_data())
        .map_err(|e| format!("Could not save roll to {}: {e}", path.display()))
}

pub fn read_history(root: &Path, session_id: &str) -> Result<DiceSessionHistory, String> {
    let dir = session_dir(root, session_id)?;
    let session = read_session(&dir)?;
    let path = dir.join(ROLLS_FILE);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("Could not read {}: {e}", path.display())),
    };
    let mut rolls = Vec::new();
    let mut skipped_lines = 0;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        match serde_json::from_str::<DiceRollRecord>(line) {
            Ok(record) if validate_record(&record, session_id).is_ok() => rolls.push(record),
            _ => skipped_lines += 1,
        }
    }
    Ok(DiceSessionHistory { session, rolls, skipped_lines })
}

pub fn list_sessions(root: &Path) -> Result<Vec<DiceSession>, String> {
    let dir = sessions_root(root);
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("Could not read {}: {e}", dir.display())),
    };
    let mut sessions: Vec<DiceSession> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| read_session(&entry.path()).ok())
        .filter(|session| prefixed_uuid(&session.id, SESSION_PREFIX))
        .collect();
    sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    Ok(sessions)
}

#[tauri::command]
pub async fn start_dice_session(project_root: String, title: String) -> Result<DiceSession, String> {
    start_session(&self::project_root(&project_root)?, &title, Utc::now())
}

#[tauri::command]
pub async fn append_dice_roll(
    project_root: String,
    session_id: String,
    record: DiceRollRecord,
) -> Result<(), String> {
    append_roll(&self::project_root(&project_root)?, &session_id, &record)
}

#[tauri::command]
pub async fn get_dice_session_history(
    project_root: String,
    session_id: String,
) -> Result<DiceSessionHistory, String> {
    read_history(&self::project_root(&project_root)?, &session_id)
}

#[tauri::command]
pub async fn list_dice_sessions(project_root: String) -> Result<Vec<DiceSession>, String> {
    list_sessions(&self::project_root(&project_root)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(session_id: &str, values: &[i64]) -> DiceRollRecord {
        let natural = values.iter().sum();
        DiceRollRecord {
            id: format!("{ROLL_PREFIX}{}", Uuid::new_v4()),
            session_id: Some(session_id.into()),
            expression: format!("{}d20+2", values.len()),
            kind: "sum".into(),
            dice: values.iter().map(|&value| DieResult { die_type: "d20".into(), value, dropped: false }).collect(),
            modifier: 2,
            natural,
            total: natural + 2,
            visibility: "public".into(),
            source: "dice-tray".into(),
            resolution: "physics".into(),
            timestamp: "2026-11-23T12:00:00Z".into(),
            critical: None,
            player_id: None,
            character_id: None,
            npc_id: None,
            encounter_id: None,
            purpose: None,
            label: Some("Mira — Stealth".into()),
        }
    }

    #[test]
    fn campaigns_keep_one_roll_log_per_session_in_their_sessions_folder() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("Sessions").join("Roll Logs")).unwrap();
        let session = start_session(dir.path(), "Session one", Utc::now()).unwrap();
        append_roll(dir.path(), &session.id, &record(&session.id, &[14, 7])).unwrap();
        let log = dir.path().join("Sessions").join("Roll Logs").join(&session.id).join(ROLLS_FILE);
        assert_eq!(fs::read_to_string(log).unwrap().lines().count(), 1);
        assert!(!dir.path().join("Story Development").exists());
        assert_eq!(list_sessions(dir.path()).unwrap().len(), 1);
    }

    #[test]
    fn campaign_manifest_selects_the_campaign_layout_before_the_folder_exists() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = crate::project_root::ProjectManifest::new("Ashes", "campaign");
        fs::write(
            dir.path().join(crate::project_root::PROJECT_MANIFEST_FILENAME),
            serde_yaml::to_string(&manifest).unwrap(),
        )
        .unwrap();
        let session = start_session(dir.path(), "Session one", Utc::now()).unwrap();
        assert!(dir.path().join("Sessions").join("Roll Logs").join(&session.id).is_dir());
    }

    #[test]
    fn campaign_yaml_selects_the_campaign_layout() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("campaign.yaml"), "title: Ashes\n").unwrap();
        let session = start_session(dir.path(), "Session one", Utc::now()).unwrap();
        assert!(dir.path().join("Sessions").join("Roll Logs").join(&session.id).is_dir());
    }

    #[test]
    fn session_rolls_round_trip_through_plain_files() {
        let dir = tempfile::tempdir().unwrap();
        let session = start_session(dir.path(), "Night one", Utc::now()).unwrap();
        append_roll(dir.path(), &session.id, &record(&session.id, &[14, 7])).unwrap();
        append_roll(dir.path(), &session.id, &record(&session.id, &[20])).unwrap();

        let folder = sessions_root(dir.path()).join(&session.id);
        // Without a campaign layout, roll logs use the book layout's Sessions folder.
        assert_eq!(
            folder,
            dir.path().join("Story Development").join("Sessions").join("Roll Logs").join(&session.id)
        );
        let lines = fs::read_to_string(folder.join(ROLLS_FILE)).unwrap();
        assert_eq!(lines.lines().count(), 2);
        assert!(lines.contains("\"expression\":\"2d20+2\""));

        let history = read_history(dir.path(), &session.id).unwrap();
        assert_eq!(history.session.title, "Night one");
        assert_eq!(history.rolls.len(), 2);
        assert_eq!(history.rolls[0].total, 23);
        assert_eq!(history.rolls[0].label.as_deref(), Some("Mira — Stealth"));
        assert_eq!(history.skipped_lines, 0);
        assert_eq!(list_sessions(dir.path()).unwrap(), vec![session]);
    }

    #[test]
    fn advantage_totals_use_only_kept_dice() {
        let dir = tempfile::tempdir().unwrap();
        let session = start_session(dir.path(), "", Utc::now()).unwrap();
        let mut adv = record(&session.id, &[15, 4]);
        adv.kind = "advantage".into();
        adv.dice[1].dropped = true;
        adv.natural = 15;
        adv.total = 17;
        append_roll(dir.path(), &session.id, &adv).unwrap();
        assert!(read_history(dir.path(), &session.id).unwrap().rolls[0].dice[1].dropped);
    }

    #[test]
    fn impossible_rolls_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let session = start_session(dir.path(), "", Utc::now()).unwrap();
        let mut bad_face = record(&session.id, &[21]);
        bad_face.natural = 21;
        bad_face.total = 23;
        assert!(append_roll(dir.path(), &session.id, &bad_face).is_err());

        let mut bad_total = record(&session.id, &[3]);
        bad_total.total = 99;
        assert!(append_roll(dir.path(), &session.id, &bad_total).is_err());

        let mut wrong_session = record(&session.id, &[3]);
        wrong_session.session_id = Some(format!("{SESSION_PREFIX}{}", Uuid::new_v4()));
        assert!(append_roll(dir.path(), &session.id, &wrong_session).is_err());
        assert!(!sessions_root(dir.path()).join(&session.id).join(ROLLS_FILE).exists());
    }

    #[test]
    fn session_ids_cannot_escape_the_sessions_folder() {
        let dir = tempfile::tempdir().unwrap();
        for id in ["../escape", "session-../../x", "session-not-a-uuid", ""] {
            assert!(append_roll(dir.path(), id, &record(id, &[3])).is_err(), "{id}");
            assert!(read_history(dir.path(), id).is_err(), "{id}");
        }
    }

    #[test]
    fn rolls_need_an_existing_session() {
        let dir = tempfile::tempdir().unwrap();
        let id = format!("{SESSION_PREFIX}{}", Uuid::new_v4());
        assert!(append_roll(dir.path(), &id, &record(&id, &[3])).is_err());
    }

    #[test]
    fn unreadable_lines_are_skipped_and_counted() {
        let dir = tempfile::tempdir().unwrap();
        let session = start_session(dir.path(), "", Utc::now()).unwrap();
        append_roll(dir.path(), &session.id, &record(&session.id, &[5])).unwrap();
        let path = sessions_root(dir.path()).join(&session.id).join(ROLLS_FILE);
        let mut file = OpenOptions::new().append(true).open(&path).unwrap();
        writeln!(file, "{{not json").unwrap();
        let history = read_history(dir.path(), &session.id).unwrap();
        assert_eq!(history.rolls.len(), 1);
        assert_eq!(history.skipped_lines, 1);
    }

    #[test]
    fn missing_sessions_folder_lists_nothing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(list_sessions(dir.path()).unwrap().is_empty());
    }
}
