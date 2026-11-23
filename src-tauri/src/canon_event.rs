//! Explicit event participation and proposed state effects.
//! Events remain authoritative Story Development documents (evt- IDs).
//! This module never applies effects or changes facts automatically.
use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use uuid::Uuid;

use crate::canon_contract::CanonFact;
use crate::document::service::{read_document, save_document};

const EVENT_CANON_METADATA_KEY: &str = "event_canon";

fn valid_id(id: &str, prefix: &str) -> bool {
    id.strip_prefix(prefix).and_then(|raw| Uuid::parse_str(raw).ok())
        .is_some_and(|value| value.get_version_num() == 4)
}

fn valid_entity(id: &str) -> bool {
    ["chr-", "loc-", "org-", "obj-", "evt-"].iter().any(|prefix| valid_id(id, prefix))
}

/// Structured event annotation attached to an existing event document.
/// No additional event identity or persistence authority is created.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventCanon {
    pub event_id: String,
    pub participants: Vec<String>,
    pub effects: Vec<EventEffect>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventEffect {
    pub subject_id: String,
    pub fact_type: String,
    pub value: Value,
    /// Exact story date; no inference from manuscript position.
    pub effective_on: String,
    /// This is a proposal until explicitly recorded as a confirmed Canon fact.
    pub status: String,
}

impl EventCanon {
    pub fn validate(&self) -> Result<(), String> {
        if !valid_id(&self.event_id, "evt-") {
            return Err("Event annotation requires an evt-prefixed UUID v4.".into());
        }
        let mut seen = BTreeSet::new();
        for id in &self.participants {
            if !valid_entity(id) {
                return Err("Event participant must have a stable entity ID.".into());
            }
            if !seen.insert(id) {
                return Err("Duplicate event participant.".into());
            }
        }
        for effect in &self.effects {
            if !valid_entity(&effect.subject_id) || effect.fact_type.trim().is_empty() {
                return Err("Event effect requires a stable subject and fact type.".into());
            }
            let date = &effect.effective_on;
            if date.len() != 10 || chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .map(|parsed| parsed.format("%Y-%m-%d").to_string() != *date)
                .unwrap_or(true)
            {
                return Err("Event effect requires an exact YYYY-MM-DD story date.".into());
            }
            if !matches!(effect.status.as_str(), "planned" | "provisional" | "confirmed") {
                return Err("Event effect must have planned, provisional, or confirmed status.".into());
            }
        }
        Ok(())
    }

    /// Returns proposed canon values for author review. Does not synthesize
    /// CanonFact IDs, change existing facts, or silently confirm event effects.
    pub fn confirmed_effects_at(&self, date: &str) -> Result<Vec<&EventEffect>, String> {
        self.validate()?;
        if chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err() {
            return Err("Invalid story date.".into());
        }
        Ok(self.effects.iter().filter(|e| e.status == "confirmed" && e.effective_on.as_str() <= date).collect())
    }

    /// Checks whether an explicit confirmed fact agrees with a confirmed event effect.
    /// A mismatch is a candidate warning, not a correction.
    pub fn conflicts_with(&self, fact: &CanonFact) -> Result<bool, String> {
        self.validate()?;
        fact.validate()?;
        Ok(fact.status == "confirmed" && self.effects.iter().any(|effect| {
            effect.status == "confirmed"
                && effect.subject_id == fact.subject_id
                && effect.fact_type == fact.fact_type
                && fact.validity.valid_from.as_deref().is_none_or(|from| from <= effect.effective_on.as_str())
                && fact.validity.valid_to.as_deref().is_none_or(|to| to >= effect.effective_on.as_str())
                && effect.value != fact.value
        }))
    }
}


/// Reopens the structured Canon annotation attached to an authoritative event
/// document. The document's own evt- identity remains the only event identity.
pub fn read_event_canon(project_root: &Path, relative_path: &Path) -> Result<Option<EventCanon>, String> {
    let document = read_document(project_root, relative_path)?;
    if !valid_id(&document.identity.id, "evt-") {
        return Err("Event Canon annotations require an existing evt-prefixed document.".into());
    }
    let Some(value) = document.metadata.get(EVENT_CANON_METADATA_KEY) else { return Ok(None); };
    let annotation: EventCanon = serde_yaml::from_value(value.clone())
        .map_err(|error| format!("Invalid event Canon annotation: {error}"))?;
    annotation.validate()?;
    if annotation.event_id != document.identity.id {
        return Err("Event Canon annotation identity does not match its event document.".into());
    }
    Ok(Some(annotation))
}

/// Persists participation/effects inside an existing event Markdown document.
/// Title, body, unrelated/unknown metadata, path, and stable identity are retained.
/// Document Service performs the guarded atomic save and stale-hash check.
pub fn save_event_canon(
    project_root: &Path,
    relative_path: &Path,
    annotation: &EventCanon,
    expected_source_hash: &str,
) -> Result<EventCanon, String> {
    annotation.validate()?;
    let mut document = read_document(project_root, relative_path)?;
    if !valid_id(&document.identity.id, "evt-") {
        return Err("Event Canon annotations require an existing evt-prefixed document.".into());
    }
    if annotation.event_id != document.identity.id {
        return Err("Event Canon annotation identity does not match its event document.".into());
    }
    let value = serde_yaml::to_value(annotation)
        .map_err(|error| format!("Could not serialize event Canon annotation: {error}"))?;
    document.metadata.insert(EVENT_CANON_METADATA_KEY.into(), value);
    save_document(project_root, &document, Some(expected_source_hash))?;
    read_event_canon(project_root, relative_path)?
        .ok_or_else(|| "Event Canon annotation was not persisted.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canon_contract::ValidityInterval;
    use crate::document::service::{create_document, move_document, read_document};
    use std::fs;
    use std::path::Path;
    fn id(prefix: &str) -> String { format!("{prefix}{}", Uuid::new_v4()) }
    fn event() -> EventCanon {
        let character = id("chr-");
        EventCanon { event_id: id("evt-"), participants: vec![character.clone()],
            effects: vec![EventEffect { subject_id: character, fact_type: "rank".into(),
                value: Value::String("captain".into()), effective_on: "2025-02-01".into(),
                status: "confirmed".into() }] }
    }
    #[test]
    fn annotations_round_trip_without_new_event_identity() {
        let record = event();
        record.validate().unwrap();
        let reopened: EventCanon = serde_yaml::from_str(&serde_yaml::to_string(&record).unwrap()).unwrap();
        assert_eq!(record, reopened);
    }
    #[test]
    fn effects_respect_dates_and_confirmation() {
        let mut record = event();
        assert!(record.confirmed_effects_at("2025-01-31").unwrap().is_empty());
        assert_eq!(record.confirmed_effects_at("2025-02-01").unwrap().len(), 1);
        record.effects[0].status = "planned".into();
        assert!(record.confirmed_effects_at("2025-03-01").unwrap().is_empty());
    }
    #[test]
    fn mismatched_confirmed_fact_is_detected_without_rewriting() {
        let record = event();
        let fact = CanonFact { id: id("fact-"), subject_id: record.participants[0].clone(),
            fact_type: "rank".into(), value: Value::String("major".into()),
            validity: ValidityInterval { valid_from: Some("2025-01-01".into()), valid_to: None },
            status: "confirmed".into() };
        assert!(record.conflicts_with(&fact).unwrap());
        assert_eq!(fact.value, Value::String("major".into()));
    }
    #[test]
    fn invalid_event_and_participants_are_rejected() {
        let mut record = event();
        record.event_id = "evt-not-uuid".into();
        assert!(record.validate().is_err());
        record.event_id = id("evt-");
        record.participants.push(record.participants[0].clone());
        assert!(record.validate().is_err());
    }
    #[test]
    fn event_annotations_persist_in_existing_document_without_rewriting_content() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = Path::new("Story Development/Events/Founding.md");
        let mut document = create_document(&root, path, "event", Some("The Founding"), "Writer-owned event notes.").unwrap();
        document.metadata.insert("future_plugin".into(), Value::String("preserve-me".into()));
        crate::document::service::save_document(&root, &document, Some(&document.source_hash)).unwrap();
        document = read_document(&root, path).unwrap();
        let mut annotation = event();
        annotation.event_id = document.identity.id.clone();
        assert_eq!(save_event_canon(&root, path, &annotation, &document.source_hash).unwrap(), annotation);
        let reopened = read_document(&root, path).unwrap();
        assert_eq!(reopened.identity.id, annotation.event_id);
        assert_eq!(reopened.title.as_deref(), Some("The Founding"));
        assert_eq!(reopened.body, "Writer-owned event notes.\n");
        assert_eq!(reopened.metadata.get("future_plugin"), Some(&Value::String("preserve-me".into())));
        assert_eq!(read_event_canon(&root, path).unwrap(), Some(annotation));
    }

    #[test]
    fn event_annotation_save_rejects_stale_hash_without_overwriting_external_change() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = Path::new("Story Development/Events/Stale.md");
        let document = create_document(&root, path, "event", Some("Stale Event"), "Original.").unwrap();
        let stale_hash = document.source_hash.clone();
        fs::write(root.join(path), fs::read_to_string(root.join(path)).unwrap() + "\nExternal edit.\n").unwrap();
        let mut annotation = event();
        annotation.event_id = document.identity.id;
        assert!(save_event_canon(&root, path, &annotation, &stale_hash).is_err());
        let raw = fs::read_to_string(root.join(path)).unwrap();
        assert!(raw.contains("External edit."));
        assert!(!raw.contains(EVENT_CANON_METADATA_KEY));
    }

    #[test]
    fn invalid_event_annotation_is_rejected_without_modifying_document() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = Path::new("Story Development/Events/Invalid.md");
        let document = create_document(&root, path, "event", Some("Invalid Event"), "Keep me.").unwrap();
        let before = fs::read(root.join(path)).unwrap();
        let mut annotation = event();
        annotation.event_id = document.identity.id;
        annotation.participants.push(annotation.participants[0].clone());
        assert!(save_event_canon(&root, path, &annotation, &document.source_hash).is_err());
        assert_eq!(fs::read(root.join(path)).unwrap(), before);
    }

    #[test]
    fn event_annotation_survives_document_rename_and_move_by_stable_identity() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let original = Path::new("Story Development/Events/Old Name.md");
        let moved = Path::new("Story Development/Timeline/New Name.md");
        let document = create_document(&root, original, "event", Some("Renamed Event"), "Body.").unwrap();
        let mut annotation = event();
        annotation.event_id = document.identity.id.clone();
        save_event_canon(&root, original, &annotation, &document.source_hash).unwrap();
        let annotated = read_document(&root, original).unwrap();
        let moved_document = move_document(&root, original, moved, &annotated.source_hash).unwrap();
        assert_eq!(moved_document.identity.id, document.identity.id);
        assert_eq!(read_event_canon(&root, moved).unwrap(), Some(annotation));
    }

    #[test]
    fn mismatched_event_document_identity_is_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = Path::new("Story Development/Events/Mismatch.md");
        let document = create_document(&root, path, "event", Some("Mismatch"), "Body.").unwrap();
        let annotation = event();
        assert_ne!(annotation.event_id, document.identity.id);
        assert!(save_event_canon(&root, path, &annotation, &document.source_hash).is_err());
        assert!(read_event_canon(&root, path).unwrap().is_none());
    }

}
