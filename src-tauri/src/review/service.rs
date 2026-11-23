use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::assembly::{assemble_review_copy, AssembledReviewCopy, ReviewCopyMetadata};
use super::integrity::manuscript_sha256;
use super::ledger::record_distribution;
use super::model::{
    ReviewDistribution, ReviewManifest, ReviewProject, ReviewRecipient,
};
use super::package::write_review_package;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewSourceInput {
    pub source_id: String,
    pub relative_path: String,
    pub manuscript_markdown: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateReviewPackageRequest {
    pub output_dir: PathBuf,
    pub ledger_dir: PathBuf,
    pub package_reference: String,
    pub review_id: String,
    pub round_id: String,
    pub project: ReviewProject,
    pub reviewer: ReviewRecipient,
    pub distribution: ReviewDistribution,
    pub sources: Vec<ReviewSourceInput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CreateReviewPackageResult {
    pub package_path: PathBuf,
    pub ledger_record_path: PathBuf,
    pub manifest: ReviewManifest,
}

/// Creates one complete native Catalyst review package and records its
/// distribution in the author's durable ledger.
///
/// This function is the authoritative orchestration boundary for new
/// Catalyst review exports:
///
/// manuscript sources
/// -> recipient-specific Markdown copies
/// -> integrity inventory
/// -> manifest
/// -> package filesystem write
/// -> durable distribution record
///
/// If the package write succeeds but the ledger write fails, the newly-created
/// package is removed so callers never observe a successful export without a
/// corresponding author-side distribution record.
pub fn create_review_package(
    request: CreateReviewPackageRequest,
) -> Result<CreateReviewPackageResult, String> {
    validate_request(&request)?;

    let copies = assemble_sources(&request)?;
    let files = copies
        .iter()
        .map(|copy| copy.integrity.clone())
        .collect::<Vec<_>>();
    let manuscript_hash = manuscript_sha256(&files)?;

    let manifest = ReviewManifest::new(
        request.review_id.clone(),
        request.round_id.clone(),
        request.project.clone(),
        request.reviewer.clone(),
        request.distribution.clone(),
        manuscript_hash,
        files,
    );

    manifest.validate()?;

    let package_path = write_review_package(
        &request.output_dir,
        &manifest,
        &copies,
    )?;

    let ledger_record_path = match record_distribution(
        &request.ledger_dir,
        &manifest,
        &request.package_reference,
    ) {
        Ok(path) => path,
        Err(error) => {
            let rollback_error = rollback_new_package(&package_path).err();
            return match rollback_error {
                Some(rollback) => Err(format!(
                    "Failed to record review distribution: {error}.                      The package was created but rollback also failed: {rollback}"
                )),
                None => Err(format!(
                    "Failed to record review distribution: {error}.                      The newly-created package was rolled back."
                )),
            };
        }
    };

    Ok(CreateReviewPackageResult {
        package_path,
        ledger_record_path,
        manifest,
    })
}

fn validate_request(request: &CreateReviewPackageRequest) -> Result<(), String> {
    if request.review_id.trim().is_empty() {
        return Err("review_id must not be empty.".into());
    }
    if request.round_id.trim().is_empty() {
        return Err("round_id must not be empty.".into());
    }
    if request.package_reference.trim().is_empty() {
        return Err("package_reference must not be empty.".into());
    }
    if request.sources.is_empty() {
        return Err("A review package must contain at least one manuscript source.".into());
    }

    Ok(())
}

fn assemble_sources(
    request: &CreateReviewPackageRequest,
) -> Result<Vec<AssembledReviewCopy>, String> {
    let mut copies = Vec::with_capacity(request.sources.len());

    for source in &request.sources {
        let metadata = ReviewCopyMetadata::new(
            request.review_id.clone(),
            request.round_id.clone(),
            request.project.id.clone(),
            source.source_id.clone(),
            request.reviewer.id.clone(),
            request.reviewer.name.clone(),
            request.distribution.created_at.clone(),
            request.distribution.purpose.clone(),
        );

        copies.push(assemble_review_copy(
            &source.relative_path,
            &metadata,
            &source.manuscript_markdown,
        )?);
    }

    Ok(copies)
}

fn rollback_new_package(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    fs::remove_dir_all(path).map_err(|error| {
        format!(
            "Failed to remove newly-created review package '{}': {error}",
            path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::ledger::read_distribution_ledger;

    fn request(root: &Path, review_id: &str) -> CreateReviewPackageRequest {
        CreateReviewPackageRequest {
            output_dir: root.join(format!("{review_id}-package")),
            ledger_dir: root.join("distribution-ledger"),
            package_reference: format!("Reviews/{review_id}-package"),
            review_id: review_id.into(),
            round_id: "round-1".into(),
            project: ReviewProject {
                id: "project-1".into(),
                title: "Test Novel".into(),
                author: Some("Author Name".into()),
            },
            reviewer: ReviewRecipient {
                id: "reviewer-1".into(),
                name: "Jane Smith".into(),
                email: Some("jane@example.com".into()),
            },
            distribution: ReviewDistribution {
                purpose: "beta-review".into(),
                created_at: "2026-09-29T14:00:00Z".into(),
            },
            sources: vec![
                ReviewSourceInput {
                    source_id: "scene-1".into(),
                    relative_path: "Manuscript/Chapter 01/Scene 01.md".into(),
                    manuscript_markdown: "# Scene One\n\nFirst scene.".into(),
                },
                ReviewSourceInput {
                    source_id: "scene-2".into(),
                    relative_path: "Manuscript/Chapter 01/Scene 02.md".into(),
                    manuscript_markdown: "# Scene Two\n\nSecond scene.".into(),
                },
            ],
        }
    }

    #[test]
    fn creates_complete_package_and_distribution_record() {
        let temp = tempfile::tempdir().unwrap();
        let req = request(temp.path(), "review-1");

        let result = create_review_package(req.clone()).unwrap();

        assert_eq!(result.package_path, req.output_dir);
        assert!(result.package_path.join("review.yaml").exists());
        assert!(result
            .package_path
            .join("Manuscript/Chapter 01/Scene 01.md")
            .exists());
        assert!(result
            .package_path
            .join("Manuscript/Chapter 01/Scene 02.md")
            .exists());
        assert!(result.ledger_record_path.exists());
        assert_eq!(result.manifest.review_id, "review-1");
        assert_eq!(result.manifest.integrity.files.len(), 2);

        let ledger = read_distribution_ledger(&req.ledger_dir).unwrap();
        assert_eq!(ledger.len(), 1);
        assert_eq!(ledger[0].review_id, "review-1");
        assert_eq!(ledger[0].source_count, 2);
        assert_eq!(ledger[0].package_reference, "Reviews/review-1-package");
    }

    #[test]
    fn rolls_back_new_package_when_ledger_write_fails() {
        let temp = tempfile::tempdir().unwrap();

        let first = request(temp.path(), "review-1");
        create_review_package(first).unwrap();

        let mut second = request(temp.path(), "review-1");
        second.output_dir = temp.path().join("second-package");
        second.package_reference = "Reviews/second-package".into();

        let error = create_review_package(second.clone()).unwrap_err();

        assert!(error.contains("Failed to record review distribution"));
        assert!(error.contains("rolled back"));
        assert!(!second.output_dir.exists());

        let ledger = read_distribution_ledger(&second.ledger_dir).unwrap();
        assert_eq!(ledger.len(), 1);
    }

    #[test]
    fn rejects_duplicate_source_ids_before_package_write() {
        let temp = tempfile::tempdir().unwrap();
        let mut req = request(temp.path(), "review-1");
        req.sources[1].source_id = req.sources[0].source_id.clone();

        assert!(create_review_package(req.clone()).is_err());
        assert!(!req.output_dir.exists());
    }

    #[test]
    fn rejects_duplicate_output_paths_before_package_write() {
        let temp = tempfile::tempdir().unwrap();
        let mut req = request(temp.path(), "review-1");
        req.sources[1].relative_path = req.sources[0].relative_path.clone();

        assert!(create_review_package(req.clone()).is_err());
        assert!(!req.output_dir.exists());
    }

    #[test]
    fn rejects_empty_source_inventory() {
        let temp = tempfile::tempdir().unwrap();
        let mut req = request(temp.path(), "review-1");
        req.sources.clear();

        assert!(create_review_package(req.clone()).is_err());
        assert!(!req.output_dir.exists());
    }

    #[test]
    fn rejects_existing_package_target_without_touching_ledger() {
        let temp = tempfile::tempdir().unwrap();
        let req = request(temp.path(), "review-1");

        fs::create_dir_all(&req.output_dir).unwrap();

        assert!(create_review_package(req.clone()).is_err());

        let ledger = read_distribution_ledger(&req.ledger_dir).unwrap();
        assert!(ledger.is_empty());
    }
}
