//! Native LithoMark review-package IPC commands.
//!
//! These commands expose the new file-based review architecture without
//! replacing the legacy Kindling-compatible editorial command surface yet.

use std::path::PathBuf;

use crate::review::ledger::{read_distribution_ledger, DistributionRecord};
use crate::review::service::{
    create_review_package, CreateReviewPackageRequest, CreateReviewPackageResult,
};

type Result<T> = std::result::Result<T, String>;

#[tauri::command]
pub async fn create_native_review_package(
    request: CreateReviewPackageRequest,
) -> Result<CreateReviewPackageResult> {
    create_review_package(request)
}

#[tauri::command]
pub async fn read_native_review_distributions(
    ledger_dir: String,
) -> Result<Vec<DistributionRecord>> {
    if ledger_dir.trim().is_empty() {
        return Err("ledger_dir must not be empty.".into());
    }

    read_distribution_ledger(&PathBuf::from(ledger_dir))
}
