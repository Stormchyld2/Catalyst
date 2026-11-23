//! Deterministic, read-only story-time projections over authoritative canon files.
//! Missing information is unknown; overlapping contradictory facts remain visible.
use std::path::Path;
use serde_yaml::Value;
use crate::canon_contract::{CanonFact, CanonRelationship, ValidityInterval};
use crate::canon_event::{read_event_canon, EventCanon};
use crate::canon_store::{list_facts, list_relationships};
use crate::story_development::build_index;

#[derive(Clone, Debug, PartialEq)]
pub struct StateFact {
    pub fact_id: String,
    pub fact_type: String,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StateRelationship {
    pub relationship_id: String,
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: String,
    pub directed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StateEventEffect {
    pub event_id: String,
    pub fact_type: String,
    pub value: Value,
    pub effective_on: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntityState {
    pub entity_id: String,
    pub story_date: String,
    pub facts: Vec<StateFact>,
    pub relationships: Vec<StateRelationship>,
    /// Confirmed, explicitly dated effects from authoritative event documents.
    /// These remain distinct from Canon facts and never overwrite them.
    pub event_effects: Vec<StateEventEffect>,
}

fn validate_story_date(date: &str) -> Result<(), String> {
    let bytes = date.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-'
        || !bytes.iter().enumerate().all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
        || chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err()
    {
        return Err("Story date must be a valid YYYY-MM-DD calendar date.".into());
    }
    Ok(())
}

fn active(interval: &ValidityInterval, date: &str) -> bool {
    interval.valid_from.as_deref().is_none_or(|from| from <= date)
        && interval.valid_to.as_deref().is_none_or(|to| date <= to)
}

/// Pure projection. Input order does not affect output order or membership.
/// Confirmed facts only: planned/provisional records are not established state.
pub fn reconstruct(entity_id: &str, story_date: &str, facts: &[CanonFact], relationships: &[CanonRelationship]) -> Result<EntityState, String> {
    reconstruct_with_events(entity_id, story_date, facts, relationships, &[])
}

/// Pure story-time projection including explicit event effects. Confirmed effects
/// become visible on their effective date and afterward, but remain separate from
/// Canon facts so no transition silently rewrites author-established truth.
pub fn reconstruct_with_events(
    entity_id: &str,
    story_date: &str,
    facts: &[CanonFact],
    relationships: &[CanonRelationship],
    events: &[EventCanon],
) -> Result<EntityState, String> {
    validate_story_date(story_date)?;
    let mut selected_facts = Vec::new();
    for fact in facts {
        fact.validate()?;
        if fact.subject_id == entity_id && fact.status == "confirmed" && active(&fact.validity, story_date) {
            selected_facts.push(StateFact { fact_id: fact.id.clone(), fact_type: fact.fact_type.clone(), value: fact.value.clone() });
        }
    }
    selected_facts.sort_by(|a, b| (&a.fact_type, &a.fact_id).cmp(&(&b.fact_type, &b.fact_id)));

    let mut selected_relationships = Vec::new();
    for relation in relationships {
        relation.validate()?;
        if (relation.source_id == entity_id || relation.target_id == entity_id)
            && active(&relation.validity, story_date)
        {
            selected_relationships.push(StateRelationship {
                relationship_id: relation.id.clone(), source_id: relation.source_id.clone(),
                target_id: relation.target_id.clone(), relationship_type: relation.relationship_type.clone(),
                directed: relation.directed,
            });
        }
    }
    selected_relationships.sort_by(|a, b| a.relationship_id.cmp(&b.relationship_id));

    let mut event_effects = Vec::new();
    for event in events {
        event.validate()?;
        for effect in &event.effects {
            if effect.status == "confirmed"
                && effect.subject_id == entity_id
                && effect.effective_on.as_str() <= story_date
            {
                event_effects.push(StateEventEffect {
                    event_id: event.event_id.clone(),
                    fact_type: effect.fact_type.clone(),
                    value: effect.value.clone(),
                    effective_on: effect.effective_on.clone(),
                });
            }
        }
    }
    event_effects.sort_by(|a, b| {
        (&a.effective_on, &a.event_id, &a.fact_type)
            .cmp(&(&b.effective_on, &b.event_id, &b.fact_type))
    });

    Ok(EntityState {
        entity_id: entity_id.into(),
        story_date: story_date.into(),
        facts: selected_facts,
        relationships: selected_relationships,
        event_effects,
    })
}

/// Always reloads from canonical Markdown files. No cache, database, or prose inference.
pub fn reconstruct_from_project(root: &Path, entity_id: &str, story_date: &str) -> Result<EntityState, String> {
    validate_story_date(story_date)?;
    let facts: Vec<_> = list_facts(root)?.into_iter().map(|entry| entry.record).collect();
    let relationships: Vec<_> = list_relationships(root)?.into_iter().map(|entry| entry.record).collect();

    let index = build_index(root)?;
    let mut events = Vec::new();
    for entry in index.entries.into_iter().filter(|entry| {
        entry.entry_kind == "document"
            && matches!(entry.document_type.as_str(), "event" | "timeline-event")
    }) {
        if let Some(annotation) = read_event_canon(root, Path::new(&entry.relative_path))? {
            events.push(annotation);
        }
    }

    reconstruct_with_events(entity_id, story_date, &facts, &relationships, &events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canon_event::{save_event_canon, EventEffect};
    use crate::canon_store::{save_fact, save_relationship};
    use crate::document::service::create_document;
    use uuid::Uuid;

    fn id(prefix: &str) -> String { format!("{prefix}{}", Uuid::new_v4()) }
    fn interval(from: Option<&str>, to: Option<&str>) -> ValidityInterval {
        ValidityInterval { valid_from: from.map(str::to_owned), valid_to: to.map(str::to_owned) }
    }
    #[test]
    fn point_in_time_membership_and_rank_change_without_manuscript_order() {
        let person = id("chr-");
        let organization = id("org-");
        let membership = CanonRelationship { id: id("rel-"), source_id: person.clone(),
            target_id: organization.clone(), relationship_type: "member".into(), directed: true,
            validity: interval(Some("2022-01-01"), Some("2025-12-31")) };
        let captain = CanonFact { id: id("fact-"), subject_id: person.clone(),
            fact_type: "rank".into(), value: Value::String("captain".into()),
            validity: interval(Some("2024-01-01"), None), status: "confirmed".into() };
        let planned = CanonFact { id: id("fact-"), subject_id: person.clone(),
            fact_type: "rank".into(), value: Value::String("general".into()),
            validity: interval(None, None), status: "planned".into() };
        let temp = tempfile::tempdir().unwrap();
        save_relationship(temp.path(), &membership, None).unwrap();
        save_fact(temp.path(), &captain, None).unwrap();
        save_fact(temp.path(), &planned, None).unwrap();
        let early = reconstruct_from_project(temp.path(), &person, "2023-01-01").unwrap();
        assert_eq!(early.relationships.len(), 1);
        assert!(early.facts.is_empty());
        let middle = reconstruct_from_project(temp.path(), &person, "2024-05-01").unwrap();
        assert_eq!(middle.relationships.len(), 1);
        assert_eq!(middle.facts.len(), 1);
        assert_eq!(middle.facts[0].value, Value::String("captain".into()));
        let late = reconstruct_from_project(temp.path(), &person, "2026-01-01").unwrap();
        assert!(late.relationships.is_empty());
        assert_eq!(late.facts.len(), 1);
        assert_eq!(reconstruct_from_project(temp.path(), &organization, "2024-05-01").unwrap().relationships.len(), 1);
    }

    #[test]
    fn overlapping_confirmed_facts_remain_visible_instead_of_silent_resolution() {
        let person = id("chr-");
        let make = |value: &str| CanonFact { id: id("fact-"), subject_id: person.clone(),
            fact_type: "location".into(), value: Value::String(value.into()),
            validity: interval(None, None), status: "confirmed".into() };
        let state = reconstruct(&person, "2025-01-01", &[make("A"), make("B")], &[]).unwrap();
        assert_eq!(state.facts.len(), 2);
    }

    #[test]
    fn invalid_story_date_is_rejected() {
        assert!(reconstruct("any", "2025-02-30", &[], &[]).is_err());
        assert!(reconstruct("any", "2025-01-01", &[], &[]).is_ok());
    }
    #[test]
    fn confirmed_event_effects_appear_only_at_or_after_their_story_date() {
        let person = id("chr-");
        let event = EventCanon {
            event_id: id("evt-"),
            participants: vec![person.clone()],
            effects: vec![EventEffect {
                subject_id: person.clone(),
                fact_type: "rank".into(),
                value: Value::String("captain".into()),
                effective_on: "2025-02-01".into(),
                status: "confirmed".into(),
            }],
        };
        let before = reconstruct_with_events(&person, "2025-01-31", &[], &[], &[event.clone()]).unwrap();
        assert!(before.event_effects.is_empty());
        let on_date = reconstruct_with_events(&person, "2025-02-01", &[], &[], &[event.clone()]).unwrap();
        assert_eq!(on_date.event_effects.len(), 1);
        assert_eq!(on_date.event_effects[0].value, Value::String("captain".into()));
        let after = reconstruct_with_events(&person, "2025-03-01", &[], &[], &[event]).unwrap();
        assert_eq!(after.event_effects.len(), 1);
    }

    #[test]
    fn planned_event_effects_are_not_established_historical_state() {
        let person = id("chr-");
        let event = EventCanon {
            event_id: id("evt-"),
            participants: vec![person.clone()],
            effects: vec![EventEffect {
                subject_id: person.clone(),
                fact_type: "rank".into(),
                value: Value::String("general".into()),
                effective_on: "2025-02-01".into(),
                status: "planned".into(),
            }],
        };
        let state = reconstruct_with_events(&person, "2025-03-01", &[], &[], &[event]).unwrap();
        assert!(state.event_effects.is_empty());
    }

    #[test]
    fn event_effects_do_not_replace_conflicting_confirmed_canon_facts() {
        let person = id("chr-");
        let fact = CanonFact {
            id: id("fact-"),
            subject_id: person.clone(),
            fact_type: "rank".into(),
            value: Value::String("major".into()),
            validity: interval(Some("2025-01-01"), None),
            status: "confirmed".into(),
        };
        let event = EventCanon {
            event_id: id("evt-"),
            participants: vec![person.clone()],
            effects: vec![EventEffect {
                subject_id: person.clone(),
                fact_type: "rank".into(),
                value: Value::String("captain".into()),
                effective_on: "2025-02-01".into(),
                status: "confirmed".into(),
            }],
        };
        let state = reconstruct_with_events(&person, "2025-03-01", &[fact], &[], &[event]).unwrap();
        assert_eq!(state.facts.len(), 1);
        assert_eq!(state.facts[0].value, Value::String("major".into()));
        assert_eq!(state.event_effects.len(), 1);
        assert_eq!(state.event_effects[0].value, Value::String("captain".into()));
    }

    #[test]
    fn project_reconstruction_reads_persisted_event_effects_from_existing_event_document() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let person = id("chr-");
        let path = Path::new("Story Development/Events/Promotion.md");
        let document = create_document(&root, path, "event", Some("Promotion"), "Writer notes.").unwrap();
        let event = EventCanon {
            event_id: document.identity.id.clone(),
            participants: vec![person.clone()],
            effects: vec![EventEffect {
                subject_id: person.clone(),
                fact_type: "rank".into(),
                value: Value::String("captain".into()),
                effective_on: "2025-02-01".into(),
                status: "confirmed".into(),
            }],
        };
        save_event_canon(&root, path, &event, &document.source_hash).unwrap();

        let before = reconstruct_from_project(&root, &person, "2025-01-31").unwrap();
        assert!(before.event_effects.is_empty());
        let after = reconstruct_from_project(&root, &person, "2025-02-01").unwrap();
        assert_eq!(after.event_effects.len(), 1);
        assert_eq!(after.event_effects[0].event_id, document.identity.id);
        assert_eq!(after.event_effects[0].value, Value::String("captain".into()));
    }

}
