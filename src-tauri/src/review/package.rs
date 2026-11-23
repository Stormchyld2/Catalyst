use std::fs;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use super::assembly::AssembledReviewCopy;
use super::integrity::{manuscript_sha256, normalize_relative_path};
use super::model::ReviewManifest;
use super::yaml::to_yaml;

pub const REVIEW_MANIFEST_FILENAME: &str = "review.yaml";

/// Writes a complete LithoMark review package to a new directory.
///
/// The package is assembled in a sibling staging directory and renamed into
/// place only after every file has been written successfully. Existing target
/// paths are never overwritten.
pub fn write_review_package(
    output_dir: &Path,
    manifest: &ReviewManifest,
    copies: &[AssembledReviewCopy],
) -> Result<PathBuf, String> {
    manifest.validate()?;

    if copies.is_empty() {
        return Err("Cannot write a review package without review copies.".into());
    }

    if output_dir.exists() {
        return Err(format!(
            "Review package target already exists: {}",
            output_dir.display()
        ));
    }

    let parent = output_dir.parent().ok_or_else(|| {
        format!(
            "Review package target must have a parent directory: {}",
            output_dir.display()
        )
    })?;

    if !parent.exists() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "Failed to create review package parent directory '{}': {error}",
                parent.display()
            )
        })?;
    }

    let expected_files: Vec<_> = copies.iter().map(|copy| copy.integrity.clone()).collect();
    let expected_manuscript_hash = manuscript_sha256(&expected_files)?;

    if expected_manuscript_hash != manifest.integrity.manuscript_sha256.to_ascii_lowercase() {
        return Err(
            "Review manifest manuscript integrity does not match the assembled review copies."
                .into(),
        );
    }

    if manifest.integrity.files != expected_files {
        return Err(
            "Review manifest source inventory does not exactly match the assembled review copies."
                .into(),
        );
    }

    for copy in copies {
        let normalized = normalize_relative_path(&copy.integrity.path)?;
        if normalized.eq_ignore_ascii_case(REVIEW_MANIFEST_FILENAME) {
            return Err(format!(
                "'{REVIEW_MANIFEST_FILENAME}' is reserved for the LithoMark review manifest."
            ));
        }
    }

    let target_name = output_dir
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| "Review package target must have a valid directory name.".to_string())?;

    let staging = parent.join(format!(
        ".{target_name}.lithomark-staging-{}",
        Uuid::new_v4()
    ));

    fs::create_dir(&staging).map_err(|error| {
        format!(
            "Failed to create review package staging directory '{}': {error}",
            staging.display()
        )
    })?;

    let write_result = write_staging_package(&staging, manifest, copies);

    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    if let Err(error) = fs::rename(&staging, output_dir) {
        let _ = fs::remove_dir_all(&staging);
        return Err(format!(
            "Failed to finalize review package '{}': {error}",
            output_dir.display()
        ));
    }

    Ok(output_dir.to_path_buf())
}

fn write_staging_package(
    staging: &Path,
    manifest: &ReviewManifest,
    copies: &[AssembledReviewCopy],
) -> Result<(), String> {
    for copy in copies {
        let relative = normalize_relative_path(&copy.integrity.path)?;
        let destination = join_canonical_relative(staging, &relative);

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "Failed to create review package directory '{}': {error}",
                    parent.display()
                )
            })?;
        }

        fs::write(&destination, copy.markdown.as_bytes()).map_err(|error| {
            format!(
                "Failed to write review copy '{}': {error}",
                destination.display()
            )
        })?;
    }

    let yaml = to_yaml(manifest)?;
    let manifest_path = staging.join(REVIEW_MANIFEST_FILENAME);

    fs::write(&manifest_path, yaml.as_bytes()).map_err(|error| {
        format!(
            "Failed to write review manifest '{}': {error}",
            manifest_path.display()
        )
    })?;

    Ok(())
}

fn join_canonical_relative(root: &Path, relative: &str) -> PathBuf {
    relative
        .split('/')
        .fold(root.to_path_buf(), |path, part| path.join(part))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::assembly::{assemble_review_copy, ReviewCopyMetadata};
    use crate::review::integrity::manuscript_sha256;
    use crate::review::model::{
        ReviewDistribution, ReviewProject, ReviewRecipient, ReviewSourceIntegrity,
    };

    fn metadata(source_id: &str) -> ReviewCopyMetadata {
        ReviewCopyMetadata::new(
            "review-1".into(),
            "round-1".into(),
            "project-1".into(),
            source_id.into(),
            "reviewer-1".into(),
            "Jane Smith".into(),
            "2026-09-29T13:00:00Z".into(),
            "beta-review".into(),
        )
    }

    fn manifest(files: Vec<ReviewSourceIntegrity>) -> ReviewManifest {
        let manuscript_hash = manuscript_sha256(&files).unwrap();

        ReviewManifest::new(
            "review-1".into(),
            "round-1".into(),
            ReviewProject {
                id: "project-1".into(),
                title: "Test Novel".into(),
                author: Some("Author Name".into()),
            },
            ReviewRecipient {
                id: "reviewer-1".into(),
                name: "Jane Smith".into(),
                email: None,
            },
            ReviewDistribution {
                purpose: "beta-review".into(),
                created_at: "2026-09-29T13:00:00Z".into(),
            },
            manuscript_hash,
            files,
        )
    }

    #[test]
    fn writes_complete_package_with_manifest_and_nested_markdown() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("review-package");

        let copy = assemble_review_copy(
            "Manuscript/Chapter 01/Scene 01.md",
            &metadata("scene-1"),
            "# Scene One\n\nText.",
        )
        .unwrap();

        let review_manifest = manifest(vec![copy.integrity.clone()]);

        let written = write_review_package(&target, &review_manifest, &[copy.clone()]).unwrap();

        assert_eq!(written, target);
        assert_eq!(
            fs::read_to_string(target.join("Manuscript/Chapter 01/Scene 01.md")).unwrap(),
            copy.markdown
        );

        let manifest_text = fs::read_to_string(target.join(REVIEW_MANIFEST_FILENAME)).unwrap();
        assert!(manifest_text.contains("format: lithomark-review"));
        assert!(manifest_text.contains("review_id: review-1"));
    }

    #[test]
    fn refuses_to_overwrite_existing_target() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("review-package");
        fs::create_dir(&target).unwrap();

        let copy = assemble_review_copy(
            "Manuscript/Scene.md",
            &metadata("scene-1"),
            "# Scene",
        )
        .unwrap();
        let review_manifest = manifest(vec![copy.integrity.clone()]);

        assert!(write_review_package(&target, &review_manifest, &[copy]).is_err());
    }

    #[test]
    fn rejects_manifest_hash_mismatch() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("review-package");

        let copy = assemble_review_copy(
            "Manuscript/Scene.md",
            &metadata("scene-1"),
            "# Scene",
        )
        .unwrap();

        let mut review_manifest = manifest(vec![copy.integrity.clone()]);
        review_manifest.integrity.manuscript_sha256 = "a".repeat(64);

        assert!(write_review_package(&target, &review_manifest, &[copy]).is_err());
        assert!(!target.exists());
    }

    #[test]
    fn rejects_manifest_inventory_mismatch() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("review-package");

        let copy = assemble_review_copy(
            "Manuscript/Scene.md",
            &metadata("scene-1"),
            "# Scene",
        )
        .unwrap();

        let mut wrong_file = copy.integrity.clone();
        wrong_file.source_id = "different-source".into();
        let review_manifest = manifest(vec![wrong_file]);

        assert!(write_review_package(&target, &review_manifest, &[copy]).is_err());
        assert!(!target.exists());
    }

    #[test]
    fn rejects_reserved_manifest_output_path() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("review-package");

        let copy = assemble_review_copy(
            REVIEW_MANIFEST_FILENAME,
            &metadata("scene-1"),
            "# Scene",
        )
        .unwrap();
        let review_manifest = manifest(vec![copy.integrity.clone()]);

        assert!(write_review_package(&target, &review_manifest, &[copy]).is_err());
        assert!(!target.exists());
    }
}
