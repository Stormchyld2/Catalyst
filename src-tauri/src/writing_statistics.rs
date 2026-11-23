//! File-authoritative writing statistics for Catalyst projects.
//!
//! Durable daily history lives in an ordinary JSON file inside the project.
//! Manuscript totals are always derived from manuscript files, never stored here.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use chrono::{Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::write_projection::{self, WriteProjection};

const STATS_FILE: &str = "Manuscript/Writing Statistics.json";
const DEFAULT_DAILY_GOAL: i64 = 500;
const STREAK_WORDS: i64 = 100;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WritingHistory {
    #[serde(default = "default_daily_goal")]
    daily_goal: i64,
    #[serde(default)]
    session_words: i64,
    #[serde(default)]
    days: BTreeMap<String, i64>,
}

impl Default for WritingHistory {
    fn default() -> Self {
        Self { daily_goal: DEFAULT_DAILY_GOAL, session_words: 0, days: BTreeMap::new() }
    }
}

fn default_daily_goal() -> i64 { DEFAULT_DAILY_GOAL }

#[derive(Clone, Debug, Serialize)]
pub struct FileWritingStats {
    pub project_id: String,
    pub project_words: i64,
    pub chapter_words: HashMap<String, i64>,
    pub scene_words: HashMap<String, i64>,
    pub daily_goal: i64,
    pub today_words: i64,
    pub session_words: i64,
    pub streak: i64,
}

fn history_path(root: &Path) -> std::path::PathBuf { root.join(STATS_FILE) }

fn read_history(root: &Path) -> Result<WritingHistory, String> {
    let path = history_path(root);
    if !path.exists() { return Ok(WritingHistory::default()); }
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("Could not read {}: {e}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|e| format!("Could not parse {}: {e}", path.display()))
}

fn write_history(root: &Path, history: &WritingHistory) -> Result<(), String> {
    let path = history_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create {}: {e}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(history).map_err(|e| e.to_string())?;
    fs::write(&path, format!("{text}\n"))
        .map_err(|e| format!("Could not write {}: {e}", path.display()))
}

fn streak(history: &WritingHistory, today: NaiveDate) -> i64 {
    let today_words = history.days.get(&today.to_string()).copied().unwrap_or(0);
    let mut day = if today_words >= STREAK_WORDS { today } else { today - Duration::days(1) };
    let mut count = 0;
    loop {
        if history.days.get(&day.to_string()).copied().unwrap_or(0) < STREAK_WORDS { break; }
        count += 1;
        day -= Duration::days(1);
    }
    count
}

pub fn stats(root: &Path, project_id: Uuid, projection: &WriteProjection) -> Result<FileWritingStats, String> {
    let history = read_history(root)?;
    let (project_words, chapter_words, scene_words) = write_projection::projection_word_counts(projection);
    let today = Local::now().date_naive();
    Ok(FileWritingStats {
        project_id: project_id.to_string(),
        project_words,
        chapter_words,
        scene_words,
        daily_goal: history.daily_goal,
        today_words: history.days.get(&today.to_string()).copied().unwrap_or(0),
        session_words: history.session_words,
        streak: streak(&history, today),
    })
}

pub fn record_delta(root: &Path, delta: i64) -> Result<(), String> {
    if delta == 0 { return Ok(()); }
    let mut history = read_history(root)?;
    let key = Local::now().date_naive().to_string();
    let current = history.days.get(&key).copied().unwrap_or(0);
    history.days.insert(key, (current + delta).max(0));
    history.session_words = (history.session_words + delta).max(0);
    write_history(root, &history)
}

pub fn reset_session(root: &Path) -> Result<(), String> {
    let mut history = read_history(root)?;
    history.session_words = 0;
    write_history(root, &history)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streak_requires_one_hundred_words() {
        let mut history = WritingHistory::default();
        let today = NaiveDate::from_ymd_opt(2026, 10, 4).unwrap();
        history.days.insert(today.to_string(), 99);
        history.days.insert((today - Duration::days(1)).to_string(), 120);
        assert_eq!(streak(&history, today), 1);
        history.days.insert(today.to_string(), 100);
        assert_eq!(streak(&history, today), 2);
    }

    #[test]
    fn deletion_after_reset_never_makes_session_negative() {
        let dir = tempfile::tempdir().unwrap();
        reset_session(dir.path()).unwrap();
        record_delta(dir.path(), -25).unwrap();
        let history = read_history(dir.path()).unwrap();
        assert_eq!(history.session_words, 0);
        assert_eq!(history.days.values().copied().sum::<i64>(), 0);
    }
}
