//! Gate B canonical relationship and fact contracts.
//!
//! These records represent explicitly authored facts, not derived wiki links.
//! Persistence and chronology projections are intentionally separate follow-up work.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidityInterval {
    /// Inclusive story-time boundary; ISO date for the initial contract.
    pub valid_from: Option<String>,
    /// Inclusive story-time boundary; ISO date for the initial contract.
    pub valid_to: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonRelationship {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: String,
    pub directed: bool,
    pub validity: ValidityInterval,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonFact {
    pub id: String,
    pub subject_id: String,
    pub fact_type: String,
    pub value: serde_yaml::Value,
    pub validity: ValidityInterval,
    pub status: String,
}

fn valid_id(id: &str, prefix: &str) -> bool {
    id.strip_prefix(prefix)
        .and_then(|value| Uuid::parse_str(value).ok())
        .is_some_and(|uuid| uuid.get_version_num() == 4)
}

fn valid_entity_reference(id: &str) -> bool {
    ["chr-", "loc-", "org-", "obj-", "evt-"]
        .iter()
        .any(|prefix| valid_id(id, prefix))
}

fn valid_interval(interval: &ValidityInterval) -> Result<(), String> {
    // ISO calendar dates sort chronologically when strictly formatted.
    let check = |date: &str| {
        let bytes = date.as_bytes();
        bytes.len() == 10
            && bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes.iter().enumerate().all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
            && chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok()
    };
    for date in [interval.valid_from.as_deref(), interval.valid_to.as_deref()].into_iter().flatten() {
        if !check(date) {
            return Err("Validity boundaries must be ISO calendar dates (YYYY-MM-DD).".into());
        }
    }
    if let (Some(from), Some(to)) = (&interval.valid_from, &interval.valid_to) {
        if from > to {
            return Err("Validity interval ends before it begins.".into());
        }
    }
    Ok(())
}

impl CanonRelationship {
    pub fn validate(&self) -> Result<(), String> {
        if !valid_id(&self.id, "rel-") {
            return Err("Relationship ID must be a rel-prefixed UUID v4.".into());
        }
        if !valid_entity_reference(&self.source_id) || !valid_entity_reference(&self.target_id) {
            return Err("Relationship endpoints must reference stable entity IDs.".into());
        }
        if self.relationship_type.trim().is_empty() {
            return Err("Relationship type is required.".into());
        }
        valid_interval(&self.validity)
    }
}

impl CanonFact {
    pub fn validate(&self) -> Result<(), String> {
        if !valid_id(&self.id, "fact-") {
            return Err("Canon fact ID must be a fact-prefixed UUID v4.".into());
        }
        if !valid_entity_reference(&self.subject_id) && !valid_id(&self.subject_id, "rel-") {
            return Err("Canon fact subject must reference a stable entity or relationship ID.".into());
        }
        if self.fact_type.trim().is_empty() || self.status.trim().is_empty() {
            return Err("Canon fact type and status are required.".into());
        }
        valid_interval(&self.validity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(prefix: &str) -> String {
        format!("{prefix}{}", Uuid::new_v4())
    }

    fn interval() -> ValidityInterval {
        ValidityInterval { valid_from: Some("2024-01-01".into()), valid_to: Some("2025-01-01".into()) }
    }

    #[test]
    fn explicit_relationship_round_trips_without_changing_identity() {
        let relation = CanonRelationship {
            id: id("rel-"), source_id: id("chr-"), target_id: id("org-"),
            relationship_type: "member".into(), directed: true, validity: interval(),
        };
        relation.validate().unwrap();
        let serialized = serde_yaml::to_string(&relation).unwrap();
        let reopened: CanonRelationship = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(reopened, relation);
    }

    #[test]
    fn rejects_invalid_relationship_ids_and_backward_dates() {
        let mut relation = CanonRelationship {
            id: id("rel-"), source_id: id("chr-"), target_id: id("org-"),
            relationship_type: "member".into(), directed: true, validity: interval(),
        };
        relation.target_id = "org-not-a-uuid".into();
        assert!(relation.validate().is_err());
        relation.target_id = id("org-");
        relation.validity.valid_to = Some("2023-12-31".into());
        assert!(relation.validate().is_err());
        relation.validity.valid_to = Some("2024-02-30".into());
        assert!(relation.validate().is_err());
    }

    #[test]
    fn explicit_canon_fact_round_trips_and_rejects_invalid_subject() {
        let mut fact = CanonFact {
            id: id("fact-"), subject_id: id("chr-"), fact_type: "rank".into(),
            value: serde_yaml::Value::String("captain".into()),
            validity: interval(), status: "confirmed".into(),
        };
        fact.validate().unwrap();
        let reopened: CanonFact = serde_yaml::from_str(&serde_yaml::to_string(&fact).unwrap()).unwrap();
        assert_eq!(reopened, fact);
        fact.subject_id = "character-title-not-identity".into();
        assert!(fact.validate().is_err());
    }
}
