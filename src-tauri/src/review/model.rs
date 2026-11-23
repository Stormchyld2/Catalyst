use serde::{Deserialize, Serialize};

pub const REVIEW_FORMAT: &str = "lithomark-review";
pub const REVIEW_FORMAT_VERSION: u32 = 1;
pub const INTEGRITY_ALGORITHM: &str = "sha256";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewManifest {
    pub format: String,
    pub format_version: u32,
    pub review_id: String,
    pub round_id: String,
    pub project: ReviewProject,
    pub reviewer: ReviewRecipient,
    pub distribution: ReviewDistribution,
    pub integrity: ReviewIntegrity,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewProject {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewRecipient {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewDistribution {
    pub purpose: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewIntegrity {
    pub algorithm: String,
    pub manuscript_sha256: String,
    pub files: Vec<ReviewSourceIntegrity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewSourceIntegrity {
    pub source_id: String,
    pub path: String,
    pub sha256: String,
}

impl ReviewManifest {
    pub fn new(
        review_id: String,
        round_id: String,
        project: ReviewProject,
        reviewer: ReviewRecipient,
        distribution: ReviewDistribution,
        manuscript_sha256: String,
        files: Vec<ReviewSourceIntegrity>,
    ) -> Self {
        Self {
            format: REVIEW_FORMAT.to_string(),
            format_version: REVIEW_FORMAT_VERSION,
            review_id,
            round_id,
            project,
            reviewer,
            distribution,
            integrity: ReviewIntegrity {
                algorithm: INTEGRITY_ALGORITHM.to_string(),
                manuscript_sha256,
                files,
            },
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.format != REVIEW_FORMAT {
            return Err(format!(
                "Unsupported review format '{}'. Expected '{}'.",
                self.format, REVIEW_FORMAT
            ));
        }

        if self.format_version != REVIEW_FORMAT_VERSION {
            return Err(format!(
                "Unsupported LithoMark review format version {}.",
                self.format_version
            ));
        }

        require_non_empty("review_id", &self.review_id)?;
        require_non_empty("round_id", &self.round_id)?;
        require_non_empty("project.id", &self.project.id)?;
        require_non_empty("project.title", &self.project.title)?;
        require_non_empty("reviewer.id", &self.reviewer.id)?;
        require_non_empty("reviewer.name", &self.reviewer.name)?;
        require_non_empty("distribution.purpose", &self.distribution.purpose)?;
        require_non_empty("distribution.created_at", &self.distribution.created_at)?;

        if self.integrity.algorithm != INTEGRITY_ALGORITHM {
            return Err(format!(
                "Unsupported review integrity algorithm '{}'. Expected '{}'.",
                self.integrity.algorithm, INTEGRITY_ALGORITHM
            ));
        }

        require_sha256("integrity.manuscript_sha256", &self.integrity.manuscript_sha256)?;

        if self.integrity.files.is_empty() {
            return Err("Review manifest must contain at least one manuscript source.".into());
        }

        for file in &self.integrity.files {
            require_non_empty("integrity.files[].source_id", &file.source_id)?;
            require_non_empty("integrity.files[].path", &file.path)?;
            require_sha256("integrity.files[].sha256", &file.sha256)?;
        }

        Ok(())
    }
}

fn require_non_empty(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{field} must not be empty."))
    } else {
        Ok(())
    }
}

fn require_sha256(field: &str, value: &str) -> Result<(), String> {
    let value = value.trim();

    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{field} must be a 64-character SHA-256 hexadecimal value."));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_hash(character: char) -> String {
        std::iter::repeat_n(character, 64).collect()
    }

    fn valid_manifest() -> ReviewManifest {
        ReviewManifest::new(
            "rev-550e8400-e29b-41d4-a716-446655440002".into(),
            "rnd-550e8400-e29b-41d4-a716-446655440003".into(),
            ReviewProject {
                id: "prj-550e8400-e29b-41d4-a716-446655440004".into(),
                title: "All That Remains".into(),
                author: Some("Jason Williams".into()),
            },
            ReviewRecipient {
                id: "rvr-550e8400-e29b-41d4-a716-446655440005".into(),
                name: "Jane Smith".into(),
                email: Some("jane@example.com".into()),
            },
            ReviewDistribution {
                purpose: "beta-review".into(),
                created_at: "2026-09-27T22:00:00Z".into(),
            },
            valid_hash('a'),
            vec![ReviewSourceIntegrity {
                source_id: "scn-550e8400-e29b-41d4-a716-446655440000".into(),
                path: "Manuscript/Chapter 01/Scene 01.md".into(),
                sha256: valid_hash('b'),
            }],
        )
    }

    #[test]
    fn native_manifest_validates() {
        assert!(valid_manifest().validate().is_ok());
    }

    #[test]
    fn rejects_wrong_format() {
        let mut manifest = valid_manifest();
        manifest.format = "kindling-editorial".into();

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn rejects_wrong_version() {
        let mut manifest = valid_manifest();
        manifest.format_version = 2;

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn rejects_missing_recipient_name() {
        let mut manifest = valid_manifest();
        manifest.reviewer.name.clear();

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn rejects_invalid_manuscript_hash() {
        let mut manifest = valid_manifest();
        manifest.integrity.manuscript_sha256 = "not-a-sha256".into();

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn rejects_empty_source_inventory() {
        let mut manifest = valid_manifest();
        manifest.integrity.files.clear();

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn rejects_invalid_source_hash() {
        let mut manifest = valid_manifest();
        manifest.integrity.files[0].sha256 = "bad".into();

        assert!(manifest.validate().is_err());
    }
}