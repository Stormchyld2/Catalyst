//! Tauri boundary for the file-authoritative Story Development index.

use std::path::PathBuf;

use crate::document::service::read_document;
use crate::story_development::{self, StoryDevelopmentIndex};
use serde::Serialize;


#[tauri::command]
pub async fn get_story_development_index(
    project_root: String,
) -> Result<StoryDevelopmentIndex, String> {
    story_development::build_index(&PathBuf::from(project_root))
}


#[derive(Clone, Debug, Serialize)]
pub struct StoryDevelopmentMetadataField {
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct StoryDevelopmentAsset {
    pub relative_path: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize)]
pub struct StoryDevelopmentDocument {
    pub id: String,
    pub document_type: String,
    pub title: Option<String>,
    pub relative_path: String,
    pub body: String,
    /// Complete authoritative Markdown file, including YAML frontmatter.
    pub source: String,
    pub metadata_yaml: String,
    pub metadata: Vec<StoryDevelopmentMetadataField>,
    pub source_hash: String,
}


fn document_response(
    document: crate::document::service::LithoMarkDocument,
) -> Result<StoryDevelopmentDocument, String> {
    let source = crate::document::service::serialize_document(&document)?;
    let metadata_yaml = serde_yaml::to_string(&document.metadata)
        .map_err(|e| format!("Could not serialize document metadata: {e}"))?;
    let metadata = document
        .metadata
        .iter()
        .map(|(key, value)| {
            serde_json::to_value(value)
                .map(|value| StoryDevelopmentMetadataField {
                    key: key.clone(),
                    value,
                })
                .map_err(|e| format!("Could not serialize metadata field '{key}': {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(StoryDevelopmentDocument {
        id: document.identity.id,
        document_type: document.identity.document_type,
        title: document.title,
        relative_path: document.relative_path.to_string_lossy().replace('\\', "/"),
        body: document.body,
        source,
        metadata_yaml,
        metadata,
        source_hash: document.source_hash,
    })
}

#[tauri::command]
pub async fn get_story_development_document(
    project_root: String,
    relative_path: String,
) -> Result<StoryDevelopmentDocument, String> {
    let document = read_document(&PathBuf::from(project_root), &PathBuf::from(&relative_path))?;
    document_response(document)
}

#[tauri::command]
pub async fn update_story_development_metadata(
    project_root: String,
    relative_path: String,
    key: String,
    value: serde_json::Value,
    expected_source_hash: String,
) -> Result<StoryDevelopmentDocument, String> {
    let yaml_value = serde_yaml::to_value(value)
        .map_err(|e| format!("Could not convert metadata value: {e}"))?;
    let document = story_development::update_metadata_field(
        &PathBuf::from(project_root),
        &PathBuf::from(relative_path),
        &key,
        yaml_value,
        &expected_source_hash,
    )?;
    document_response(document)
}

#[tauri::command]
pub async fn update_story_development_source(
    project_root: String,
    relative_path: String,
    source: String,
    expected_source_hash: String,
) -> Result<StoryDevelopmentDocument, String> {
    let root = PathBuf::from(&project_root);
    let relative = PathBuf::from(&relative_path);
    let current = read_document(&root, &relative)?;
    if current.source_hash != expected_source_hash {
        return Err("Document changed on disk after it was loaded.".into());
    }

    // Parse and validate the complete source before the authoritative file is
    // touched. Invalid YAML, reserved identity changes, or malformed
    // frontmatter therefore cannot corrupt the saved document.
    let mut parsed = crate::document::service::parse_document(&relative, &source)?;
    if parsed.identity.id != current.identity.id {
        return Err("Stable document ID cannot be changed in Edit view.".into());
    }
    parsed.source_hash = current.source_hash.clone();
    crate::document::service::save_document(&root, &parsed, Some(&expected_source_hash))?;
    let saved = read_document(&root, &relative)?;
    document_response(saved)
}

#[tauri::command]
pub async fn update_story_development_body(
    project_root: String,
    relative_path: String,
    body: String,
    expected_source_hash: String,
) -> Result<StoryDevelopmentDocument, String> {
    let document = story_development::update_document_body(
        &PathBuf::from(project_root),
        &PathBuf::from(relative_path),
        &body,
        &expected_source_hash,
    )?;
    document_response(document)
}

#[tauri::command]
pub async fn move_story_development_document(
    project_root: String,
    relative_path: String,
    destination_path: String,
    expected_source_hash: String,
) -> Result<StoryDevelopmentDocument, String> {
    let document = crate::document::service::move_document(
        &PathBuf::from(project_root),
        &PathBuf::from(relative_path),
        &PathBuf::from(destination_path),
        &expected_source_hash,
    )?;
    document_response(document)
}

#[tauri::command]
pub async fn delete_story_development_document(
    project_root: String,
    relative_path: String,
    expected_source_hash: String,
) -> Result<String, String> {
    let identity = crate::document::service::delete_document(
        &PathBuf::from(project_root),
        &PathBuf::from(relative_path),
        &expected_source_hash,
    )?;
    Ok(identity.id)
}

#[derive(Clone, Debug, Serialize)]
pub struct TrashedStoryDocument {
    pub entry_id: String,
    pub original_path: String,
    pub document_id: String,
    pub source_hash: String,
}

#[tauri::command]
pub async fn list_story_development_trash(project_root: String) -> Result<Vec<TrashedStoryDocument>, String> {
    crate::document::service::list_trashed_documents(&PathBuf::from(project_root))
        .map(|items| items.into_iter().map(|item| TrashedStoryDocument {
            entry_id: item.entry_id,
            original_path: item.original_relative_path.to_string_lossy().replace('\\', "/"),
            document_id: item.identity.id,
            source_hash: item.source_hash,
        }).collect())
}

#[tauri::command]
pub async fn restore_story_development_trash(
    project_root: String,
    entry_id: String,
    expected_source_hash: String,
) -> Result<StoryDevelopmentDocument, String> {
    let restored = crate::document::service::restore_trashed_document(
        &PathBuf::from(project_root), &entry_id, &expected_source_hash,
    )?;
    document_response(restored)
}

#[tauri::command]
pub async fn get_story_development_asset(
    project_root: String,
    relative_path: String,
) -> Result<StoryDevelopmentAsset, String> {
    let relative = PathBuf::from(&relative_path);
    let bytes = story_development::read_story_asset(&PathBuf::from(project_root), &relative)?;
    let mime_type = story_development::image_mime_type(&relative)?.to_string();
    Ok(StoryDevelopmentAsset {
        relative_path: relative_path.replace('\\', "/"),
        mime_type,
        bytes,
    })
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveryCandidate {
    pub artifact_path: String,
    pub original_path: Option<String>,
    pub kind: String,
    pub valid_document: bool,
}

#[tauri::command]
pub async fn list_document_recovery_candidates(
    project_root: String,
) -> Result<Vec<RecoveryCandidate>, String> {
    crate::document::service::discover_recovery_artifacts(&PathBuf::from(project_root))
        .map(|artifacts| artifacts.into_iter().map(|artifact| RecoveryCandidate {
            artifact_path: artifact.relative_path.to_string_lossy().replace('\\', "/"),
            original_path: artifact.original_relative_path
                .map(|path| path.to_string_lossy().replace('\\', "/")),
            kind: artifact.kind,
            valid_document: artifact.valid_document,
        }).collect())
}

/// Preview is strictly read-only. Re-discover the artifact to reject arbitrary
/// project paths, including regular manuscript documents.
#[tauri::command]
pub async fn preview_document_recovery_candidate(
    project_root: String,
    artifact_path: String,
) -> Result<String, String> {
    let root = PathBuf::from(project_root);
    let candidate = crate::document::service::discover_recovery_artifacts(&root)?
        .into_iter()
        .find(|item| item.relative_path.to_string_lossy().replace('\\', "/") == artifact_path)
        .ok_or_else(|| "Recovery artifact was not found.".to_string())?;
    if !candidate.valid_document {
        return Err("Recovery artifact is not a valid LithoMark document.".into());
    }
    let path = root.join(&candidate.relative_path);
    std::fs::read_to_string(&path)
        .map_err(|e| format!("Could not preview recovery artifact: {e}"))
}

#[derive(Clone, Debug, Serialize)]
pub struct RecoveryRestoreReview {
    pub artifact_hash: String,
    pub destination_hash: String,
    pub original_path: String,
}

/// Read-only preflight: hashes are rechecked by the service at restore time.
#[tauri::command]
pub async fn review_document_recovery_restore(
    project_root: String,
    artifact_path: String,
) -> Result<RecoveryRestoreReview, String> {
    use sha2::{Digest, Sha256};
    let root = PathBuf::from(project_root);
    let candidate = crate::document::service::discover_recovery_artifacts(&root)?
        .into_iter()
        .find(|item| item.relative_path.to_string_lossy().replace('\\', "/") == artifact_path)
        .ok_or_else(|| "Recovery artifact was not found.".to_string())?;
    if !candidate.valid_document {
        return Err("Recovery artifact is not a valid LithoMark document.".into());
    }
    let original = candidate.original_relative_path
        .ok_or_else(|| "Recovery file has no known original destination.".to_string())?;
    let artifact_bytes = std::fs::read(root.join(&candidate.relative_path))
        .map_err(|e| format!("Could not inspect recovery artifact: {e}"))?;
    let destination = read_document(&root, &original)?;
    let original_bytes = std::fs::read(root.join(&original))
        .map_err(|e| format!("Could not inspect original document: {e}"))?;
    let recovered_text = std::str::from_utf8(&artifact_bytes)
        .map_err(|e| format!("Recovery file is not UTF-8: {e}"))?;
    let recovered = crate::document::service::parse_document(&original, recovered_text)?;
    if recovered.identity != destination.identity {
        return Err("Recovery document identity differs from the original.".into());
    }
    Ok(RecoveryRestoreReview {
        artifact_hash: Sha256::digest(&artifact_bytes).iter().map(|byte| format!("{byte:02x}")).collect(),
        destination_hash: Sha256::digest(&original_bytes).iter().map(|byte| format!("{byte:02x}")).collect(),
        original_path: original.to_string_lossy().replace('\\', "/"),
    })
}

#[tauri::command]
pub async fn restore_document_recovery_candidate(
    project_root: String,
    artifact_path: String,
    expected_artifact_hash: String,
    expected_destination_hash: String,
) -> Result<String, String> {
    let root = PathBuf::from(project_root);
    crate::document::service::restore_recovery_artifact(
        &root,
        &PathBuf::from(artifact_path),
        &expected_artifact_hash,
        &expected_destination_hash,
    ).map(|backup| backup.to_string_lossy().replace('\\', "/"))
}
