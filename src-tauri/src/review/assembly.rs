use serde::{Deserialize, Serialize};

use super::integrity::source_integrity;
use super::model::ReviewSourceIntegrity;

pub const REVIEW_COPY_FORMAT: &str = "lithomark-review-copy";
pub const REVIEW_COPY_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewCopyMetadata {
    pub format: String,
    pub format_version: u32,
    pub review_id: String,
    pub round_id: String,
    pub project_id: String,
    pub source_id: String,
    pub reviewer_id: String,
    pub reviewer_name: String,
    pub created_at: String,
    pub purpose: String,
}

impl ReviewCopyMetadata {
    pub fn new(review_id: String, round_id: String, project_id: String, source_id: String, reviewer_id: String, reviewer_name: String, created_at: String, purpose: String) -> Self {
        Self { format: REVIEW_COPY_FORMAT.into(), format_version: REVIEW_COPY_FORMAT_VERSION, review_id, round_id, project_id, source_id, reviewer_id, reviewer_name, created_at, purpose }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.format != REVIEW_COPY_FORMAT { return Err(format!("Unsupported review-copy format '{}'. Expected '{}'.", self.format, REVIEW_COPY_FORMAT)); }
        if self.format_version != REVIEW_COPY_FORMAT_VERSION { return Err(format!("Unsupported Catalyst review-copy format version {}.", self.format_version)); }
        for (field, value) in [
            ("review_id", self.review_id.as_str()), ("round_id", self.round_id.as_str()),
            ("project_id", self.project_id.as_str()), ("source_id", self.source_id.as_str()),
            ("reviewer_id", self.reviewer_id.as_str()), ("reviewer_name", self.reviewer_name.as_str()),
            ("created_at", self.created_at.as_str()), ("purpose", self.purpose.as_str()),
        ] {
            if value.trim().is_empty() { return Err(format!("{field} must not be empty.")); }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssembledReviewCopy {
    pub markdown: String,
    pub integrity: ReviewSourceIntegrity,
}

/// Builds the exact Markdown review copy written into a portable Catalyst
/// review package. Integrity hashes include frontmatter and the visible notice.
pub fn assemble_review_copy(relative_path: &str, metadata: &ReviewCopyMetadata, manuscript_markdown: &str) -> Result<AssembledReviewCopy, String> {
    metadata.validate()?;
    if manuscript_markdown.trim().is_empty() { return Err("Review-copy manuscript Markdown must not be empty.".into()); }

    let yaml = serialize_metadata(metadata)?;
    let manuscript = normalize_newlines(manuscript_markdown);
    let mut markdown = String::new();
    markdown.push_str("---\n");
    markdown.push_str(&yaml);
    markdown.push_str("---\n\n");
    markdown.push_str("> **Catalyst Review Copy**\n");
    markdown.push_str("> Prepared for **");
    markdown.push_str(&escape_notice_text(&metadata.reviewer_name));
    markdown.push_str("**. Review ID: ");
    markdown.push_str(&metadata.review_id);
    markdown.push_str(" · Round: ");
    markdown.push_str(&metadata.round_id);
    markdown.push_str("\n\n");
    markdown.push_str(&manuscript);
    if !markdown.ends_with('\n') { markdown.push('\n'); }

    let integrity = source_integrity(metadata.source_id.clone(), relative_path, markdown.as_bytes())?;
    Ok(AssembledReviewCopy { markdown, integrity })
}

fn serialize_metadata(metadata: &ReviewCopyMetadata) -> Result<String, String> {
    let mut yaml = serde_yaml::to_string(metadata)
        .map_err(|error| format!("Failed to serialize review-copy metadata: {error}"))?
        .replace("\r\n", "\n");
    while yaml.ends_with("\n\n") { yaml.pop(); }
    if !yaml.ends_with('\n') { yaml.push('\n'); }
    Ok(yaml)
}

fn normalize_newlines(value: &str) -> String { value.replace("\r\n", "\n").replace('\r', "\n") }
fn escape_notice_text(value: &str) -> String { value.replace('\\', "\\\\").replace('*', "\\*") }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::integrity::sha256_hex;

    fn metadata() -> ReviewCopyMetadata {
        ReviewCopyMetadata::new("review-1".into(), "round-1".into(), "project-1".into(), "scene-1".into(), "reviewer-1".into(), "Jane Smith".into(), "2026-09-29T13:00:00Z".into(), "beta-review".into())
    }

    #[test]
    fn assembles_frontmatter_notice_and_manuscript() {
        let copy = assemble_review_copy("Manuscript/Chapter 01/Scene 01.md", &metadata(), "# Scene One\n\nThe door opened.").unwrap();
        assert!(copy.markdown.starts_with("---\nformat: lithomark-review-copy\n"));
        assert!(copy.markdown.contains("reviewer_name: Jane Smith"));
        assert!(copy.markdown.contains("> **Catalyst Review Copy**"));
        assert!(copy.markdown.contains("Prepared for **Jane Smith**"));
        assert!(copy.markdown.contains("# Scene One\n\nThe door opened."));
        assert!(copy.markdown.ends_with('\n'));
    }

    #[test]
    fn integrity_hashes_exact_completed_markdown_bytes() {
        let copy = assemble_review_copy("Manuscript/Chapter 01/Scene 01.md", &metadata(), "# Scene One\n").unwrap();
        assert_eq!(copy.integrity.sha256, sha256_hex(copy.markdown.as_bytes()));
        assert_eq!(copy.integrity.source_id, "scene-1");
        assert_eq!(copy.integrity.path, "Manuscript/Chapter 01/Scene 01.md");
    }

    #[test]
    fn normalizes_windows_newlines_before_hashing() {
        let windows = assemble_review_copy("Manuscript\\Chapter 01\\Scene 01.md", &metadata(), "# Scene One\r\n\r\nText.\r\n").unwrap();
        let unix = assemble_review_copy("Manuscript/Chapter 01/Scene 01.md", &metadata(), "# Scene One\n\nText.\n").unwrap();
        assert_eq!(windows.markdown, unix.markdown);
        assert_eq!(windows.integrity.sha256, unix.integrity.sha256);
    }

    #[test]
    fn rejects_empty_manuscript() {
        assert!(assemble_review_copy("Manuscript/Scene.md", &metadata(), " \n ").is_err());
    }

    #[test]
    fn rejects_invalid_metadata() {
        let mut invalid = metadata();
        invalid.reviewer_name.clear();
        assert!(assemble_review_copy("Manuscript/Scene.md", &invalid, "# Scene").is_err());
    }

    #[test]
    fn rejects_unsafe_output_path() {
        assert!(assemble_review_copy("../outside.md", &metadata(), "# Scene").is_err());
    }

    #[test]
    fn escapes_markdown_emphasis_in_visible_recipient_name() {
        let mut meta = metadata();
        meta.reviewer_name = "Jane *Editor*".into();
        let copy = assemble_review_copy("Manuscript/Scene.md", &meta, "# Scene").unwrap();
        assert!(copy.markdown.contains("Prepared for **Jane \\*Editor\\***"));
    }
}
