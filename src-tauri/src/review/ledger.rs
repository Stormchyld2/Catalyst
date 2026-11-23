use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::integrity::sha256_hex;
use super::model::ReviewManifest;

pub const DISTRIBUTION_RECORD_FORMAT: &str = "lithomark-review-distribution-record";
pub const DISTRIBUTION_RECORD_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistributionRecord {
    pub format: String,
    pub format_version: u32,
    pub review_id: String,
    pub round_id: String,
    pub project_id: String,
    pub project_title: String,
    pub reviewer_id: String,
    pub reviewer_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer_email: Option<String>,
    pub purpose: String,
    pub created_at: String,
    pub manuscript_sha256: String,
    pub source_count: usize,
    pub package_reference: String,
}

impl DistributionRecord {
    pub fn from_manifest(
        manifest: &ReviewManifest,
        package_reference: String,
    ) -> Result<Self, String> {
        manifest.validate()?;

        if package_reference.trim().is_empty() {
            return Err("Distribution package reference must not be empty.".into());
        }

        let record = Self {
            format: DISTRIBUTION_RECORD_FORMAT.into(),
            format_version: DISTRIBUTION_RECORD_FORMAT_VERSION,
            review_id: manifest.review_id.clone(),
            round_id: manifest.round_id.clone(),
            project_id: manifest.project.id.clone(),
            project_title: manifest.project.title.clone(),
            reviewer_id: manifest.reviewer.id.clone(),
            reviewer_name: manifest.reviewer.name.clone(),
            reviewer_email: manifest.reviewer.email.clone(),
            purpose: manifest.distribution.purpose.clone(),
            created_at: manifest.distribution.created_at.clone(),
            manuscript_sha256: manifest.integrity.manuscript_sha256.to_ascii_lowercase(),
            source_count: manifest.integrity.files.len(),
            package_reference,
        };

        record.validate()?;
        Ok(record)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.format != DISTRIBUTION_RECORD_FORMAT {
            return Err(format!(
                "Unsupported distribution record format '{}'. Expected '{}'.",
                self.format, DISTRIBUTION_RECORD_FORMAT
            ));
        }

        if self.format_version != DISTRIBUTION_RECORD_FORMAT_VERSION {
            return Err(format!(
                "Unsupported LithoMark distribution record version {}.",
                self.format_version
            ));
        }

        for (field, value) in [
            ("review_id", self.review_id.as_str()),
            ("round_id", self.round_id.as_str()),
            ("project_id", self.project_id.as_str()),
            ("project_title", self.project_title.as_str()),
            ("reviewer_id", self.reviewer_id.as_str()),
            ("reviewer_name", self.reviewer_name.as_str()),
            ("purpose", self.purpose.as_str()),
            ("created_at", self.created_at.as_str()),
            ("package_reference", self.package_reference.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("{field} must not be empty."));
            }
        }

        if self.manuscript_sha256.len() != 64
            || !self
                .manuscript_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err("manuscript_sha256 must be a 64-character SHA-256 hexadecimal value.".into());
        }

        if self.source_count == 0 {
            return Err("source_count must be greater than zero.".into());
        }

        Ok(())
    }
}

/// Records one immutable author-side distribution entry.
///
/// Each distribution is stored as its own YAML file. This deliberately avoids
/// a single mutable ledger file becoming the sole source of truth.
pub fn record_distribution(
    ledger_dir: &Path,
    manifest: &ReviewManifest,
    package_reference: &str,
) -> Result<PathBuf, String> {
    let record = DistributionRecord::from_manifest(manifest, package_reference.to_string())?;

    fs::create_dir_all(ledger_dir).map_err(|error| {
        format!(
            "Failed to create review distribution ledger '{}': {error}",
            ledger_dir.display()
        )
    })?;

    let filename = record_filename(&record.review_id);
    let final_path = ledger_dir.join(filename);

    if final_path.exists() {
        return Err(format!(
            "Distribution record already exists for review '{}'.",
            record.review_id
        ));
    }

    let yaml = serialize_record(&record)?;
    let staging = ledger_dir.join(format!(".distribution-{}.tmp", Uuid::new_v4()));

    fs::write(&staging, yaml.as_bytes()).map_err(|error| {
        format!(
            "Failed to write distribution record staging file '{}': {error}",
            staging.display()
        )
    })?;

    if let Err(error) = fs::rename(&staging, &final_path) {
        let _ = fs::remove_file(&staging);
        return Err(format!(
            "Failed to finalize distribution record '{}': {error}",
            final_path.display()
        ));
    }

    Ok(final_path)
}

pub fn read_distribution_ledger(ledger_dir: &Path) -> Result<Vec<DistributionRecord>, String> {
    if !ledger_dir.exists() {
        return Ok(Vec::new());
    }

    if !ledger_dir.is_dir() {
        return Err(format!(
            "Review distribution ledger path is not a directory: {}",
            ledger_dir.display()
        ));
    }

    let mut records = Vec::new();

    for entry in fs::read_dir(ledger_dir).map_err(|error| {
        format!(
            "Failed to read review distribution ledger '{}': {error}",
            ledger_dir.display()
        )
    })? {
        let entry = entry.map_err(|error| format!("Failed to read ledger entry: {error}"))?;
        let path = entry.path();

        if path.extension().and_then(|extension| extension.to_str()) != Some("yaml") {
            continue;
        }

        let yaml = fs::read_to_string(&path).map_err(|error| {
            format!(
                "Failed to read distribution record '{}': {error}",
                path.display()
            )
        })?;

        let record: DistributionRecord = serde_yaml::from_str(&yaml).map_err(|error| {
            format!(
                "Failed to parse distribution record '{}': {error}",
                path.display()
            )
        })?;

        record.validate().map_err(|error| {
            format!(
                "Invalid distribution record '{}': {error}",
                path.display()
            )
        })?;

        records.push(record);
    }

    records.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then_with(|| left.review_id.cmp(&right.review_id))
    });

    Ok(records)
}

fn serialize_record(record: &DistributionRecord) -> Result<String, String> {
    record.validate()?;

    let mut yaml = serde_yaml::to_string(record)
        .map_err(|error| format!("Failed to serialize distribution record: {error}"))?
        .replace("\r\n", "\n");

    while yaml.ends_with("\n\n") {
        yaml.pop();
    }
    if !yaml.ends_with('\n') {
        yaml.push('\n');
    }

    Ok(yaml)
}

fn record_filename(review_id: &str) -> String {
    format!("{}.yaml", sha256_hex(review_id.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::model::{
        ReviewDistribution, ReviewProject, ReviewRecipient, ReviewSourceIntegrity,
    };

    fn valid_hash(character: char) -> String {
        std::iter::repeat_n(character, 64).collect()
    }

    fn manifest(review_id: &str, created_at: &str) -> ReviewManifest {
        ReviewManifest::new(
            review_id.into(),
            "round-1".into(),
            ReviewProject {
                id: "project-1".into(),
                title: "Test Novel".into(),
                author: Some("Author Name".into()),
            },
            ReviewRecipient {
                id: "reviewer-1".into(),
                name: "Jane Smith".into(),
                email: Some("jane@example.com".into()),
            },
            ReviewDistribution {
                purpose: "beta-review".into(),
                created_at: created_at.into(),
            },
            valid_hash('a'),
            vec![ReviewSourceIntegrity {
                source_id: "scene-1".into(),
                path: "Manuscript/Scene.md".into(),
                sha256: valid_hash('b'),
            }],
        )
    }

    #[test]
    fn records_distribution_as_immutable_yaml() {
        let temp = tempfile::tempdir().unwrap();
        let ledger = temp.path().join("ledger");
        let review = manifest("review-1", "2026-09-29T13:00:00Z");

        let path = record_distribution(&ledger, &review, "Exports/review-1").unwrap();

        assert!(path.exists());
        let yaml = fs::read_to_string(path).unwrap();
        assert!(yaml.contains("format: lithomark-review-distribution-record"));
        assert!(yaml.contains("review_id: review-1"));
        assert!(yaml.contains("package_reference: Exports/review-1"));
    }

    #[test]
    fn reads_and_sorts_distribution_records() {
        let temp = tempfile::tempdir().unwrap();
        let ledger = temp.path().join("ledger");

        record_distribution(
            &ledger,
            &manifest("review-2", "2026-09-30T13:00:00Z"),
            "Exports/review-2",
        )
        .unwrap();
        record_distribution(
            &ledger,
            &manifest("review-1", "2026-09-29T13:00:00Z"),
            "Exports/review-1",
        )
        .unwrap();

        let records = read_distribution_ledger(&ledger).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].review_id, "review-1");
        assert_eq!(records[1].review_id, "review-2");
    }

    #[test]
    fn refuses_duplicate_review_record() {
        let temp = tempfile::tempdir().unwrap();
        let ledger = temp.path().join("ledger");
        let review = manifest("review-1", "2026-09-29T13:00:00Z");

        record_distribution(&ledger, &review, "Exports/review-1").unwrap();

        assert!(record_distribution(&ledger, &review, "Exports/other").is_err());
    }

    #[test]
    fn missing_ledger_reads_as_empty() {
        let temp = tempfile::tempdir().unwrap();
        let records = read_distribution_ledger(&temp.path().join("missing")).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn rejects_empty_package_reference() {
        let temp = tempfile::tempdir().unwrap();
        let review = manifest("review-1", "2026-09-29T13:00:00Z");

        assert!(record_distribution(temp.path(), &review, "   ").is_err());
    }

    #[test]
    fn record_filename_is_safe_and_deterministic() {
        let first = record_filename("../unsafe/review");
        let second = record_filename("../unsafe/review");

        assert_eq!(first, second);
        assert!(first.ends_with(".yaml"));
        assert!(!first.contains('/'));
        assert!(!first.contains(".."));
    }
}
