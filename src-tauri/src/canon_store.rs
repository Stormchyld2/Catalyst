//! File-authoritative Gate B storage. Uses the existing guarded Document Service.
//! Each explicit record is one Markdown document with YAML frontmatter.
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Serialize};

use crate::canon_contract::{CanonFact, CanonRelationship};
use crate::document::service::{read_document, save_document, DocumentIdentity, CatalystDocument};

const FACT_DIR: &str = "Story Development/Canon/Facts";
const REL_DIR: &str = "Story Development/Canon/Relationships";

pub struct StoredRecord<T> {
    pub record: T,
    pub source_hash: String,
}

fn record_path(folder: &str, id: &str) -> PathBuf {
    Path::new(folder).join(format!("{id}.md"))
}

fn scan<T: DeserializeOwned>(root: &Path, folder: &str, kind: &str) -> Result<Vec<StoredRecord<T>>, String> {
    let directory = root.join(folder);
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut records = Vec::new();
    let mut ids = BTreeSet::new();
    for item in fs::read_dir(&directory).map_err(|e| format!("Cannot read {}: {e}", directory.display()))? {
        let entry = item.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        if !path.is_file() {
            return Err(format!("Non-file canon entry: {}", path.display()));
        }
        let relative = Path::new(folder).join(entry.file_name());
        let doc = read_document(root, &relative)?;
        if doc.identity.document_type != kind {
            return Err(format!("Unexpected canon document type at {}", relative.display()));
        }
        if !ids.insert(doc.identity.id.clone()) {
            return Err(format!("Duplicate canon ID: {}", doc.identity.id));
        }
        let value = doc.metadata.get("record")
            .ok_or_else(|| format!("Missing canon record: {}", relative.display()))?;
        let record: T = serde_yaml::from_value(value.clone())
            .map_err(|e| format!("Invalid canon record at {}: {e}", relative.display()))?;
        let embedded_id = value.as_mapping().and_then(|map| map.get(serde_yaml::Value::String("id".into()))).and_then(serde_yaml::Value::as_str);
        if embedded_id != Some(doc.identity.id.as_str()) || relative != record_path(folder, &doc.identity.id) {
            return Err(format!("Canon record identity/path mismatch at {}", relative.display()));
        }
        records.push(StoredRecord { record, source_hash: doc.source_hash });
    }
    Ok(records)
}

fn save<T: Serialize>(
    root: &Path, folder: &str, kind: &str, id: &str, record: &T, expected_hash: Option<&str>,
) -> Result<String, String> {
    if !root.is_absolute() || !root.is_dir() {
        return Err("An existing absolute project root is required.".into());
    }
    let relative = record_path(folder, id);
    let path = root.join(&relative);
    // Validate uniqueness across all files before creating or changing anything.
    let directory = root.join(folder);
    if directory.exists() {
        for item in fs::read_dir(&directory).map_err(|e| e.to_string())? {
            let entry = item.map_err(|e| e.to_string())?;
            if entry.path().extension().and_then(|v| v.to_str()) != Some("md") { continue; }
            let other_relative = Path::new(folder).join(entry.file_name());
            let other = read_document(root, &other_relative)?;
            if other.identity.id == id && other_relative != relative {
                return Err(format!("Duplicate canon ID: {id}"));
            }
        }
    }
    match expected_hash {
        None if path.exists() => return Err("Canon record already exists; supply its current hash to update.".into()),
        Some(_) if !path.exists() => return Err("Canon record missing; cannot update.".into()),
        _ => {}
    }
    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert("record".to_string(), serde_yaml::to_value(record).map_err(|e| e.to_string())?);
    let doc = CatalystDocument {
        identity: DocumentIdentity { document_type: kind.into(), id: id.into() },
        title: None, metadata, body: String::new(), relative_path: relative,
        source_hash: String::new(),
    };
    save_document(root, &doc, expected_hash).map(|saved| saved.source_hash)
}

pub fn list_facts(root: &Path) -> Result<Vec<StoredRecord<CanonFact>>, String> {
    let records = scan::<CanonFact>(root, FACT_DIR, "canon-fact")?;
    for item in &records { item.record.validate()?; }
    Ok(records)
}

pub fn list_relationships(root: &Path) -> Result<Vec<StoredRecord<CanonRelationship>>, String> {
    let records = scan::<CanonRelationship>(root, REL_DIR, "relationship")?;
    for item in &records { item.record.validate()?; }
    Ok(records)
}

pub fn save_fact(root: &Path, fact: &CanonFact, expected_hash: Option<&str>) -> Result<String, String> {
    fact.validate()?;
    save(root, FACT_DIR, "canon-fact", &fact.id, fact, expected_hash)
}

pub fn save_relationship(root: &Path, relation: &CanonRelationship, expected_hash: Option<&str>) -> Result<String, String> {
    relation.validate()?;
    save(root, REL_DIR, "relationship", &relation.id, relation, expected_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canon_contract::ValidityInterval;
    use uuid::Uuid;

    fn id(prefix: &str) -> String { format!("{prefix}{}", Uuid::new_v4()) }
    fn validity() -> ValidityInterval { ValidityInterval { valid_from: None, valid_to: None } }
    fn fact() -> CanonFact {
        CanonFact { id: id("fact-"), subject_id: id("chr-"), fact_type: "occupation".into(),
            value: serde_yaml::Value::String("editor".into()), validity: validity(), status: "confirmed".into() }
    }
    fn relationship() -> CanonRelationship {
        CanonRelationship { id: id("rel-"), source_id: id("chr-"), target_id: id("org-"),
            relationship_type: "member".into(), directed: true, validity: validity() }
    }

    #[test]
    fn records_survive_reopen_and_keep_stable_ids() {
        let temp = tempfile::tempdir().unwrap();
        let f = fact();
        let r = relationship();
        save_fact(temp.path(), &f, None).unwrap();
        save_relationship(temp.path(), &r, None).unwrap();
        assert_eq!(list_facts(temp.path()).unwrap()[0].record, f);
        assert_eq!(list_relationships(temp.path()).unwrap()[0].record, r);
    }

    #[test]
    fn guarded_update_rejects_stale_hash_and_preserves_newer_data() {
        let temp = tempfile::tempdir().unwrap();
        let mut f = fact();
        let old_hash = save_fact(temp.path(), &f, None).unwrap();
        f.value = serde_yaml::Value::String("director".into());
        save_fact(temp.path(), &f, Some(&old_hash)).unwrap();
        f.value = serde_yaml::Value::String("intern".into());
        assert!(save_fact(temp.path(), &f, Some(&old_hash)).is_err());
        assert_eq!(list_facts(temp.path()).unwrap()[0].record.value,
            serde_yaml::Value::String("director".into()));
    }

    #[test]
    fn duplicate_ids_and_create_overwrite_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let f = fact();
        save_fact(temp.path(), &f, None).unwrap();
        assert!(save_fact(temp.path(), &f, None).is_err());
        let original = temp.path().join(record_path(FACT_DIR, &f.id));
        let duplicate = temp.path().join(FACT_DIR).join("duplicate.md");
        fs::copy(&original, &duplicate).unwrap();
        assert!(list_facts(temp.path()).is_err());
        assert!(save_fact(temp.path(), &f, Some("bogus")).is_err());
    }

    #[test]
    fn invalid_files_are_reported_without_rewrite() {
        let temp = tempfile::tempdir().unwrap();
        let f = fact();
        save_fact(temp.path(), &f, None).unwrap();
        let path = temp.path().join(record_path(FACT_DIR, &f.id));
        fs::write(&path, "not valid frontmatter").unwrap();
        assert!(list_facts(temp.path()).is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "not valid frontmatter");
    }
}
