//! Read-only continuity diagnostics from explicitly authored canon records.
//! Diagnostics are potential conflicts, never automatic corrections.
use std::collections::BTreeSet;
use std::path::Path;

use crate::canon_contract::{CanonFact, CanonRelationship, ValidityInterval};
use crate::canon_event::{read_event_canon, EventCanon};
use crate::canon_store::{list_facts, list_relationships};
use crate::story_development::build_index;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContinuityWarning {
    pub rule_id: String,
    pub severity: String,
    pub subject_id: String,
    pub record_ids: Vec<String>,
    pub message: String,
}

fn active_on(interval: &ValidityInterval, date: &str) -> bool {
    interval.valid_from.as_deref().is_none_or(|from| from <= date)
        && interval.valid_to.as_deref().is_none_or(|to| date <= to)
}

fn overlaps(a: &ValidityInterval, b: &ValidityInterval) -> bool {
    let a_starts_before_b_ends = match (&a.valid_from, &b.valid_to) {
        (Some(start), Some(end)) => start <= end,
        _ => true,
    };
    let b_starts_before_a_ends = match (&b.valid_from, &a.valid_to) {
        (Some(start), Some(end)) => start <= end,
        _ => true,
    };
    a_starts_before_b_ends && b_starts_before_a_ends
}

/// Only inherently single-valued facts are checked. Other fact types may
/// legitimately have multiple concurrent values, and are not presumed exclusive.
fn exclusive_type(kind: &str) -> bool {
    matches!(kind, "location" | "rank" | "birth" | "death")
}

pub fn analyze(facts: &[CanonFact], relationships: &[CanonRelationship], known_entity_ids: Option<&BTreeSet<String>>) -> Result<Vec<ContinuityWarning>, String> {
    analyze_with_events(facts, relationships, &[], known_entity_ids)
}

/// Read-only diagnostics across Canon facts and explicitly authored event effects.
pub fn analyze_with_events(facts: &[CanonFact], relationships: &[CanonRelationship], events: &[EventCanon], known_entity_ids: Option<&BTreeSet<String>>) -> Result<Vec<ContinuityWarning>, String> {
    for fact in facts { fact.validate()?; }
    for event in events { event.validate()?; }
    for relation in relationships { relation.validate()?; }
    let mut warnings = Vec::new();
    let relation_ids: BTreeSet<&str> = relationships.iter().map(|r| r.id.as_str()).collect();
    let mut ordered: Vec<_> = facts.iter().filter(|fact| fact.status == "confirmed" && exclusive_type(&fact.fact_type)).collect();
    ordered.sort_by(|a, b| a.id.cmp(&b.id));
    for (i, left) in ordered.iter().enumerate() {
        for right in ordered.iter().skip(i + 1) {
            if left.subject_id == right.subject_id && left.fact_type == right.fact_type
                && left.value != right.value && overlaps(&left.validity, &right.validity)
            {
                let mut record_ids = vec![left.id.clone(), right.id.clone()];
                record_ids.sort();
                warnings.push(ContinuityWarning {
                    rule_id: format!("continuity.canon.{}.overlap", left.fact_type),
                    severity: "warning".into(),
                    subject_id: left.subject_id.clone(),
                    record_ids,
                    message: format!("Conflicting confirmed {} facts have overlapping validity.", left.fact_type),
                });
            }
        }
    }
    for event in events {
        for effect in &event.effects {
            if effect.status != "confirmed" || !exclusive_type(&effect.fact_type) { continue; }
            for fact in facts {
                if fact.status == "confirmed"
                    && fact.subject_id == effect.subject_id
                    && fact.fact_type == effect.fact_type
                    && active_on(&fact.validity, &effect.effective_on)
                    && fact.value != effect.value
                {
                    let mut record_ids = vec![event.event_id.clone(), fact.id.clone()];
                    record_ids.sort();
                    warnings.push(ContinuityWarning {
                        rule_id: format!("continuity.event.{}.conflict", effect.fact_type),
                        severity: "warning".into(),
                        subject_id: effect.subject_id.clone(),
                        record_ids,
                        message: format!("Confirmed event effect conflicts with an established {} fact on its effective date.", effect.fact_type),
                    });
                }
            }
        }
    }
    if let Some(known) = known_entity_ids {
        for fact in facts {
            // A relationship is a valid subject; only check entity subjects here.
            if !(if fact.subject_id.starts_with("rel-") { relation_ids.contains(fact.subject_id.as_str()) } else { known.contains(&fact.subject_id) }) {
                warnings.push(ContinuityWarning {
                    rule_id: "continuity.reference.missing".into(), severity: "warning".into(),
                    subject_id: fact.subject_id.clone(), record_ids: vec![fact.id.clone()],
                    message: "Canon fact refers to an entity not present in the supplied entity index.".into(),
                });
            }
        }
        for relation in relationships {
            for endpoint in [&relation.source_id, &relation.target_id] {
                if !known.contains(endpoint) {
                    warnings.push(ContinuityWarning {
                        rule_id: "continuity.reference.missing".into(), severity: "warning".into(),
                        subject_id: endpoint.clone(), record_ids: vec![relation.id.clone()],
                        message: "Relationship endpoint is not present in the supplied entity index.".into(),
                    });
                }
            }
        }
    }
    warnings.sort_by(|a, b| (&a.rule_id, &a.subject_id, &a.record_ids).cmp(&(&b.rule_id, &b.subject_id, &b.record_ids)));
    Ok(warnings)
}

/// Loads canon files fresh; entity existence checking is deferred until a
/// canonical cross-module entity index is supplied.
pub fn analyze_project(root: &Path) -> Result<Vec<ContinuityWarning>, String> {
    let facts: Vec<_> = list_facts(root)?.into_iter().map(|entry| entry.record).collect();
    let relationships: Vec<_> = list_relationships(root)?.into_iter().map(|entry| entry.record).collect();
    let index = build_index(root)?;
    if !index.issues.is_empty() {
        return Err(format!("Story Development index has {} unreadable documents.", index.issues.len()));
    }
    let mut events = Vec::new();
    for entry in &index.entries {
        if entry.entry_kind == "document" && matches!(entry.document_type.as_str(), "event" | "timeline-event") {
            if let Some(event) = read_event_canon(root, Path::new(&entry.relative_path))? {
                events.push(event);
            }
        }
    }
    let known: BTreeSet<String> = index.entries.into_iter()
        .filter(|e| e.entry_kind == "document" && matches!(e.document_type.as_str(), "character" | "location" | "organization" | "object" | "item" | "event" | "timeline-event"))
        .map(|e| e.id).collect();
    analyze_with_events(&facts, &relationships, &events, Some(&known))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canon_store::save_fact;
    use crate::canon_event::{save_event_canon, EventEffect};
    use crate::document::service::create_document;
    use uuid::Uuid;
    fn id(prefix: &str) -> String { format!("{prefix}{}", Uuid::new_v4()) }
    fn interval(from: Option<&str>, to: Option<&str>) -> ValidityInterval {
        ValidityInterval { valid_from: from.map(str::to_owned), valid_to: to.map(str::to_owned) }
    }
    fn fact(subject: &str, kind: &str, value: &str, from: Option<&str>, to: Option<&str>) -> CanonFact {
        CanonFact { id: id("fact-"), subject_id: subject.into(), fact_type: kind.into(),
            value: serde_yaml::Value::String(value.into()), validity: interval(from, to),
            status: "confirmed".into() }
    }

    #[test]
    fn conflicting_location_overlap_warns_without_mutating_files() {
        let root = tempfile::tempdir().unwrap();
        let subject = id("chr-");
        let a = fact(&subject, "location", "A", Some("2025-01-01"), Some("2025-03-01"));
        let b = fact(&subject, "location", "B", Some("2025-02-01"), Some("2025-04-01"));
        save_fact(root.path(), &a, None).unwrap();
        save_fact(root.path(), &b, None).unwrap();
        let warnings = analyze_project(root.path()).unwrap();
        assert_eq!(warnings.len(), 3);
        let overlap: Vec<_> = warnings.iter()
            .filter(|warning| warning.rule_id == "continuity.canon.location.overlap")
            .collect();
        assert_eq!(overlap.len(), 1);
        assert_eq!(overlap[0].record_ids.len(), 2);
        let missing: Vec<_> = warnings.iter()
            .filter(|warning| warning.rule_id == "continuity.reference.missing")
            .collect();
        assert_eq!(missing.len(), 2);
        assert_eq!(analyze_project(root.path()).unwrap(), warnings);
    }

    #[test]
    fn nonoverlapping_and_unconfirmed_facts_do_not_warn() {
        let subject = id("chr-");
        let a = fact(&subject, "rank", "captain", None, Some("2024-12-31"));
        let b = fact(&subject, "rank", "major", Some("2025-01-01"), None);
        let mut planned = fact(&subject, "rank", "general", None, None);
        planned.status = "planned".into();
        assert!(analyze(&[a, b, planned], &[], None).unwrap().is_empty());
    }

    #[test]
    fn multi_valued_knowledge_does_not_trigger_false_positive() {
        let subject = id("chr-");
        let a = fact(&subject, "knowledge", "secret A", None, None);
        let b = fact(&subject, "knowledge", "secret B", None, None);
        assert!(analyze(&[a, b], &[], None).unwrap().is_empty());
    }

    #[test]
    fn actual_character_document_resolves_fact_reference() {
        use crate::document::service::create_document;
        let root = tempfile::tempdir().unwrap();
        let path = Path::new("Story Development/Characters/Test.md");
        let character = create_document(root.path(), path, "character", Some("Test"), "").unwrap();
        save_fact(root.path(), &fact(&character.identity.id, "rank", "captain", None, None), None).unwrap();
        assert!(analyze_project(root.path()).unwrap().is_empty());
    }

    #[test]
    fn moving_character_keeps_canon_reference_valid() {
        use crate::document::service::{create_document, move_document};
        let root = tempfile::tempdir().unwrap();
        let original = Path::new("Story Development/Characters/Test.md");
        let renamed = Path::new("Story Development/Characters/Renamed.md");
        let character = create_document(root.path(), original, "character", Some("Test"), "").unwrap();
        save_fact(root.path(), &fact(&character.identity.id, "rank", "captain", None, None), None).unwrap();
        let moved = move_document(root.path(), original, renamed, &character.source_hash).unwrap();
        assert_eq!(moved.identity.id, character.identity.id);
        assert!(analyze_project(root.path()).unwrap().is_empty());
    }

    #[test]
    fn nonexistent_character_produces_warning_without_rewriting_fact() {
        let root = tempfile::tempdir().unwrap();
        let missing = id("chr-");
        let record = fact(&missing, "rank", "captain", None, None);
        save_fact(root.path(), &record, None).unwrap();
        let before = std::fs::read_to_string(root.path().join("Story Development/Canon/Facts").join(format!("{}.md", record.id))).unwrap();
        let warnings = analyze_project(root.path()).unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_id, "continuity.reference.missing");
        let after = std::fs::read_to_string(root.path().join("Story Development/Canon/Facts").join(format!("{}.md", record.id))).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn missing_references_are_warnings_when_entity_index_is_provided() {
        let subject = id("chr-");
        let a = fact(&subject, "location", "A", None, None);
        let known = BTreeSet::new();
        let warnings = analyze(&[a], &[], Some(&known)).unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_id, "continuity.reference.missing");
    }
    #[test]
    fn confirmed_event_conflict_warns_without_changing_canon() {
        let person = id("chr-");
        let established = fact(&person, "rank", "major", Some("2025-01-01"), None);
        let event = EventCanon { event_id: id("evt-"), participants: vec![person.clone()],
            effects: vec![EventEffect { subject_id: person.clone(), fact_type: "rank".into(),
                value: serde_yaml::Value::String("captain".into()), effective_on: "2025-02-01".into(),
                status: "confirmed".into() }] };
        let warnings = analyze_with_events(&[established.clone()], &[], &[event.clone()], None).unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_id, "continuity.event.rank.conflict");
        assert_eq!(warnings[0].record_ids.len(), 2);
        assert_eq!(established.value, serde_yaml::Value::String("major".into()));
        assert_eq!(event.effects[0].value, serde_yaml::Value::String("captain".into()));
    }

    #[test]
    fn unconfirmed_and_outside_validity_event_effects_do_not_warn() {
        let person = id("chr-");
        let established = fact(&person, "rank", "major", Some("2025-01-01"), Some("2025-01-31"));
        let mut event = EventCanon { event_id: id("evt-"), participants: vec![person.clone()],
            effects: vec![EventEffect { subject_id: person, fact_type: "rank".into(),
                value: serde_yaml::Value::String("captain".into()), effective_on: "2025-02-01".into(),
                status: "confirmed".into() }] };
        assert!(analyze_with_events(&[established.clone()], &[], &[event.clone()], None).unwrap().is_empty());
        event.effects[0].effective_on = "2025-01-15".into();
        event.effects[0].status = "planned".into();
        assert!(analyze_with_events(&[established], &[], &[event], None).unwrap().is_empty());
    }

    #[test]
    fn project_diagnostics_read_persisted_event_and_preserve_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let character_path = Path::new("Story Development/Characters/Person.md");
        let person = create_document(&root, character_path, "character", Some("Person"), "").unwrap();
        let event_path = Path::new("Story Development/Events/Promotion.md");
        let document = create_document(&root, event_path, "event", Some("Promotion"), "Author notes.").unwrap();
        let established = fact(&person.identity.id, "rank", "major", Some("2025-01-01"), None);
        save_fact(&root, &established, None).unwrap();
        let event = EventCanon { event_id: document.identity.id.clone(), participants: vec![person.identity.id],
            effects: vec![EventEffect { subject_id: established.subject_id.clone(), fact_type: "rank".into(),
                value: serde_yaml::Value::String("captain".into()), effective_on: "2025-02-01".into(),
                status: "confirmed".into() }] };
        save_event_canon(&root, event_path, &event, &document.source_hash).unwrap();
        let fact_path = root.join("Story Development/Canon/Facts").join(format!("{}.md", established.id));
        let fact_before = std::fs::read(&fact_path).unwrap();
        let event_before = std::fs::read(root.join(event_path)).unwrap();
        let warnings = analyze_project(&root).unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_id, "continuity.event.rank.conflict");
        assert_eq!(std::fs::read(&fact_path).unwrap(), fact_before);
        assert_eq!(std::fs::read(root.join(event_path)).unwrap(), event_before);
    }

}
