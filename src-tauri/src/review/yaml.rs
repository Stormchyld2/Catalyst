use super::model::ReviewManifest;

/// Serializes a validated Catalyst review manifest to deterministic YAML.
///
/// The returned string is the YAML payload only. Markdown frontmatter
/// delimiters are added by the review-copy assembler, not here.
pub fn to_yaml(manifest: &ReviewManifest) -> Result<String, String> {
    manifest.validate()?;

    let mut yaml = serde_yaml::to_string(manifest)
        .map_err(|error| format!("Failed to serialize Catalyst review manifest: {error}"))?;

    // Keep package output stable across platforms and ensure exactly one
    // trailing newline before a caller adds a closing frontmatter delimiter.
    yaml = yaml.replace("\r\n", "\n");
    while yaml.ends_with("\n\n") {
        yaml.pop();
    }
    if !yaml.ends_with('\n') {
        yaml.push('\n');
    }

    Ok(yaml)
}

/// Deserializes and validates a Catalyst review manifest from YAML.
///
/// Validation after deserialization is deliberate: syntactically valid YAML
/// is not automatically a valid or supported Catalyst review package.
pub fn from_yaml(yaml: &str) -> Result<ReviewManifest, String> {
    if yaml.trim().is_empty() {
        return Err("Review manifest YAML must not be empty.".into());
    }

    let manifest: ReviewManifest = serde_yaml::from_str(yaml)
        .map_err(|error| format!("Failed to deserialize Catalyst review manifest: {error}"))?;

    manifest.validate()?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::model::{
        ReviewDistribution, ReviewProject, ReviewRecipient, ReviewSourceIntegrity,
    };

    fn hash(character: char) -> String {
        std::iter::repeat_n(character, 64).collect()
    }

    fn manifest() -> ReviewManifest {
        ReviewManifest::new(
            "rev-1".into(),
            "round-1".into(),
            ReviewProject {
                id: "project-1".into(),
                title: "All That Remains".into(),
                author: Some("Jason Williams".into()),
            },
            ReviewRecipient {
                id: "reviewer-1".into(),
                name: "Jane Smith".into(),
                email: Some("jane@example.com".into()),
            },
            ReviewDistribution {
                purpose: "beta-review".into(),
                created_at: "2026-09-29T13:00:00Z".into(),
            },
            hash('a'),
            vec![ReviewSourceIntegrity {
                source_id: "scene-1".into(),
                path: "Manuscript/Chapter 01/Scene 01.md".into(),
                sha256: hash('b'),
            }],
        )
    }

    #[test]
    fn manifest_round_trips_through_yaml() {
        let original = manifest();
        let yaml = to_yaml(&original).unwrap();
        let decoded = from_yaml(&yaml).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn serialized_yaml_uses_catalyst_format() {
        let yaml = to_yaml(&manifest()).unwrap();

        assert!(yaml.contains("format: catalyst-review"));
        assert!(!yaml.contains("kindling-editorial"));
    }

    #[test]
    fn rejects_empty_yaml() {
        assert!(from_yaml("   \n").is_err());
    }

    #[test]
    fn rejects_unsupported_format_after_deserialization() {
        let yaml = to_yaml(&manifest())
            .unwrap()
            .replace("format: catalyst-review", "format: kindling-editorial");

        assert!(from_yaml(&yaml).is_err());
    }

    #[test]
    fn rejects_invalid_manifest_before_serialization() {
        let mut invalid = manifest();
        invalid.review_id.clear();

        assert!(to_yaml(&invalid).is_err());
    }
}
