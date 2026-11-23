//! Tauri boundary for the local entity-template registry.

use tauri::{AppHandle, Manager};

use crate::entity_templates::{self, EntityTemplateRegistry, SaveUserEntityTemplateInput};

#[tauri::command]
pub async fn get_entity_template_registry(
    app: AppHandle,
) -> Result<EntityTemplateRegistry, String> {
    let templates_dir = templates_dir(&app)?;
    entity_templates::load_registry(&templates_dir)
}


fn templates_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not resolve Catalyst app data directory: {e}"))?;
    Ok(data_dir.join("Templates"))
}

#[tauri::command]
pub async fn save_user_entity_template(
    app: AppHandle,
    input: SaveUserEntityTemplateInput,
) -> Result<EntityTemplateRegistry, String> {
    let templates_dir = templates_dir(&app)?;
    entity_templates::save_user_template(&templates_dir, input)
}

#[tauri::command]
pub async fn delete_user_entity_template(
    app: AppHandle,
    relative_path: String,
) -> Result<EntityTemplateRegistry, String> {
    let templates_dir = templates_dir(&app)?;
    entity_templates::delete_user_template(&templates_dir, &relative_path)
}

#[derive(serde::Deserialize)]
pub struct CreateFromTemplateInput {
    pub project_root: String,
    pub template_id: String,
    pub destination_folder: String,
    pub title: String,
    #[serde(default)]
    pub fields: std::collections::BTreeMap<String, serde_json::Value>,
}

/// Create an ordinary project-owned Markdown file using a registered template.
/// Existing filenames are never overwritten; legacy category folders are reused.
#[tauri::command]
pub async fn create_story_document_from_template(
    app: AppHandle,
    input: CreateFromTemplateInput,
) -> Result<crate::commands::StoryDevelopmentDocument, String> {
    use std::path::PathBuf;
    let root = PathBuf::from(&input.project_root);
    if !root.is_absolute() || !root.is_dir() {
        return Err("Select a valid local project folder.".into());
    }
    let title = input.title.trim();
    if title.is_empty() || title == "." || title == ".."
        || title.chars().any(|c| c.is_control() || r#"<>:"/\\|?*"#.contains(c))
        || title.ends_with('.') || title.ends_with(' ') {
        return Err("Document title contains invalid filename characters.".into());
    }
    let registry = entity_templates::load_registry(&templates_dir(&app)?)?;
    let template = registry.templates.iter().find(|t| t.manifest.id == input.template_id);
    if template.is_none() && input.template_id != "empty" {
        return Err("Selected template is no longer available.".into());
    }
    let base = std::path::PathBuf::from(&input.destination_folder);
    let allowed = ["Story Development", "Characters", "Locations", "Worldbuilding", "Organizations", "Items", "General", "Timeline", "Research", "Planning", "Story Bible", "Story"];
    let parts: Vec<_> = base.components().collect();
    if parts.is_empty() || !parts.iter().all(|c| matches!(c, std::path::Component::Normal(_)))
        || !allowed.iter().any(|v| base.starts_with(v)) {
        return Err("Choose a valid Story Development destination.".into());
    }
    let mut cursor = root.clone();
    for component in &parts {
        cursor.push(component.as_os_str());
        if cursor.exists() {
            let meta = std::fs::symlink_metadata(&cursor).map_err(|e| e.to_string())?;
            if meta.file_type().is_symlink() || !meta.is_dir() {
                return Err("Destination contains a link or non-folder.".into());
            }
        }
    }
    let relative = base.join(format!("{title}.md"));
    let destination = root.join(&relative);
    std::fs::create_dir_all(root.join(&base))
        .map_err(|e| format!("Could not create document folder: {e}"))?;
    if destination.exists() {
        return Err("A document with this name already exists in the destination folder.".into());
    }
    // Validate submitted values against the template schema; unknown fields are
    // rejected rather than silently being dropped.
    let mut metadata = std::collections::BTreeMap::new();
    if let Some(template) = template {
    metadata.insert("template_id".to_string(), serde_yaml::Value::String(template.manifest.id.clone()));
    if let Some(subtype) = &template.manifest.subtype {
        metadata.insert("subtype".to_string(), serde_yaml::Value::String(subtype.clone()));
    }
    for (key, value) in &input.fields {
        if !template.manifest.fields.iter().any(|f| &f.key == key) {
            return Err(format!("Field '{key}' is not part of this template."));
        }
        metadata.insert(key.clone(), serde_yaml::to_value(value)
            .map_err(|e| format!("Invalid value for '{key}': {e}"))?);
    }
    }
    let document_type = template.map(|t| t.manifest.document_type.as_str()).unwrap_or("note");
    let document = crate::document::service::CatalystDocument {
        identity: crate::document::service::DocumentIdentity {
            document_type: document_type.to_string(),
            id: crate::document::service::generate_stable_id(document_type)?,
        },
        title: Some(title.to_string()),
        metadata,
        body: template.map(|t| t.body.clone()).unwrap_or_default(),
        relative_path: relative.clone(),
        source_hash: String::new(),
    };
    // Save into a unique reserved filename: do not race with another creator.
    let reserved = std::fs::OpenOptions::new().write(true).create_new(true).open(&destination)
        .map_err(|e| format!("Could not reserve new document filename: {e}"))?;
    drop(reserved);
    let saved = crate::document::service::save_document(&root, &document, None);
    if let Err(error) = saved {
        // Only remove the zero-byte reservation, never a populated document.
        if std::fs::metadata(&destination).map(|m| m.len() == 0).unwrap_or(false) {
            let _ = std::fs::remove_file(&destination);
        }
        return Err(error);
    }
    crate::commands::get_story_development_document(
        input.project_root,
        relative.to_string_lossy().replace('\\', "/"),
    ).await
}

#[tauri::command]
pub async fn get_story_development_folders(project_root: String) -> Result<Vec<String>, String> {
    let root = std::path::PathBuf::from(project_root);
    if !root.is_absolute() || !root.is_dir() { return Err("Invalid project root.".into()); }
    let roots = ["Story Development", "Story Bible", "Story", "Characters", "Locations",
        "Worldbuilding", "Timeline", "Research", "Planning", "Organizations", "Items", "General"];
    let mut folders = Vec::new();
    for name in roots {
        let path = root.join(name);
        if !path.is_dir() || path.is_symlink() { continue; }
        let mut pending = vec![std::path::PathBuf::from(name)];
        while let Some(relative) = pending.pop() {
            if folders.len() >= 2000 { return Err("Too many story folders.".into()); }
            folders.push(relative.to_string_lossy().replace('\\', "/"));
            for child in std::fs::read_dir(root.join(&relative)).map_err(|e| e.to_string())? {
                let child = child.map_err(|e| e.to_string())?;
                if child.file_type().map_err(|e| e.to_string())?.is_dir() {
                    pending.push(relative.join(child.file_name()));
                }
            }
        }
    }
    folders.sort();
    folders.dedup();
    if folders.is_empty() { folders.push("Story Development".into()); }
    Ok(folders)
}

#[tauri::command]
pub async fn create_story_development_folder(
    project_root: String,
    parent_folder: String,
    name: String,
) -> Result<String, String> {
    let root = std::path::PathBuf::from(project_root);
    if !root.is_absolute() || !root.is_dir() { return Err("Invalid project root.".into()); }
    let name = name.trim();
    if name.is_empty() || name == "." || name == ".." || name.ends_with('.')
        || name.ends_with(' ') || name.chars().any(|c| c.is_control() || r#"<>:"/\\|?*"#.contains(c)) {
        return Err("Enter a valid folder name.".into());
    }
    let parent = std::path::PathBuf::from(&parent_folder);
    let allowed = ["Story Development", "Story Bible", "Story", "Characters", "Locations",
        "Worldbuilding", "Timeline", "Research", "Planning", "Organizations", "Items", "General"];
    if !parent.components().all(|c| matches!(c, std::path::Component::Normal(_)))
        || !allowed.iter().any(|v| parent.starts_with(v)) {
        return Err("Select a Story Development parent folder.".into());
    }
    let mut cursor = root.clone();
    for component in parent.components() {
        cursor.push(component.as_os_str());
        if cursor.exists() {
            let meta = std::fs::symlink_metadata(&cursor).map_err(|e| e.to_string())?;
            if meta.file_type().is_symlink() || !meta.is_dir() {
                return Err("Folder path contains a link or non-folder.".into());
            }
        }
    }
    let relative = parent.join(name);
    let absolute = root.join(&relative);
    if absolute.exists() { return Err("That folder already exists.".into()); }
    std::fs::create_dir_all(&absolute).map_err(|e| format!("Could not create folder: {e}"))?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

#[derive(serde::Serialize)]
pub struct TemplateApplyPreview {
    pub template_name: String,
    pub existing_body: String,
    pub proposed_body: String,
    pub added_headings: Vec<String>,
}

fn empty_template_field_value(field_type: &str) -> serde_yaml::Value {
    match field_type {
        "list" => serde_yaml::Value::Sequence(Vec::new()),
        "boolean" => serde_yaml::Value::Bool(false),
        "number" => serde_yaml::Value::Null,
        _ => serde_yaml::Value::String(String::new()),
    }
}

fn initialize_template_metadata(
    metadata: &mut std::collections::BTreeMap<String, serde_yaml::Value>,
    template: &entity_templates::EntityTemplate,
) {
    metadata.insert("template_id".into(), serde_yaml::Value::String(template.manifest.id.clone()));
    if let Some(subtype) = &template.manifest.subtype {
        metadata.entry("subtype".into())
            .or_insert_with(|| serde_yaml::Value::String(subtype.clone()));
    }
    for field in &template.manifest.fields {
        metadata.entry(field.key.clone())
            .or_insert_with(|| empty_template_field_value(&field.field_type));
    }
}

fn merge_template_sections(existing: &str, template_body: &str) -> (String, Vec<String>) {
    fn heading(line: &str) -> Option<String> {
        let trimmed = line.trim();
        let count = trimmed.bytes().take_while(|b| *b == b'#').count();
        if count == 0 || count > 6 || !trimmed.as_bytes().get(count).is_some_and(|b| *b == b' ') {
            return None;
        }
        Some(trimmed[count..].trim().to_lowercase())
    }
    let mut seen: std::collections::BTreeSet<String> =
        existing.lines().filter_map(heading).collect();
    let mut missing = Vec::new();
    let mut added = Vec::new();
    let mut include = false;
    for line in template_body.lines() {
        if let Some(name) = heading(line) {
            include = seen.insert(name.clone());
            if include {
                added.push(line.trim().trim_start_matches('#').trim().to_string());
                missing.push(line.to_string());
            }
        } else if include {
            missing.push(line.to_string());
        }
    }
    if missing.is_empty() { return (existing.to_string(), added); }
    let mut body = existing.trim_end().to_string();
    if !body.is_empty() { body.push_str("\n\n"); }
    body.push_str(missing.join("\n").trim_end());
    (body, added)
}

#[tauri::command]
pub async fn preview_story_document_template(
    app: AppHandle,
    project_root: String,
    relative_path: String,
    template_id: String,
) -> Result<TemplateApplyPreview, String> {
    let registry = entity_templates::load_registry(&templates_dir(&app)?)?;
    let template = registry.templates.iter().find(|t| t.manifest.id == template_id)
        .ok_or_else(|| "Template not found.".to_string())?;
    let doc = crate::document::service::read_document(
        std::path::Path::new(&project_root), std::path::Path::new(&relative_path)
    )?;
    let (proposed_body, added_headings) = merge_template_sections(&doc.body, &template.body);
    Ok(TemplateApplyPreview {
        template_name: template.manifest.name.clone(),
        existing_body: doc.body,
        proposed_body,
        added_headings,
    })
}

#[tauri::command]
pub async fn apply_story_document_template(
    app: AppHandle,
    project_root: String,
    relative_path: String,
    template_id: String,
    expected_source_hash: String,
) -> Result<crate::commands::StoryDevelopmentDocument, String> {
    let registry = entity_templates::load_registry(&templates_dir(&app)?)?;
    let template = registry.templates.iter().find(|t| t.manifest.id == template_id)
        .ok_or_else(|| "Template not found.".to_string())?;
    let root = std::path::Path::new(&project_root);
    let mut doc = crate::document::service::read_document(root, std::path::Path::new(&relative_path))?;
    if doc.source_hash != expected_source_hash {
        return Err("Document changed since preview. Reopen the preview before applying.".into());
    }
    let (body, _) = merge_template_sections(&doc.body, &template.body);
    doc.body = body;
    initialize_template_metadata(&mut doc.metadata, template);
    crate::document::service::save_document(root, &doc, Some(&expected_source_hash))?;
    crate::commands::get_story_development_document(project_root, relative_path).await
}

#[cfg(test)]
mod template_assignment_tests {
    use super::merge_template_sections;
    #[test]
    fn initializes_missing_template_fields_without_overwriting_existing_values() {
        let temp = tempfile::tempdir().unwrap();
        let template = crate::entity_templates::load_registry(&temp.path().join("Templates"))
            .unwrap()
            .templates
            .into_iter()
            .find(|template| template.manifest.id == "character-profile")
            .unwrap();
        let mut metadata = std::collections::BTreeMap::new();
        metadata.insert("occupation".into(), serde_yaml::Value::String("Detective".into()));
        super::initialize_template_metadata(&mut metadata, &template);
        assert_eq!(metadata.get("template_id").and_then(serde_yaml::Value::as_str), Some("character-profile"));
        assert_eq!(metadata.get("occupation").and_then(serde_yaml::Value::as_str), Some("Detective"));
        assert_eq!(metadata.get("image").and_then(serde_yaml::Value::as_str), Some(""));
        assert!(matches!(metadata.get("aliases"), Some(serde_yaml::Value::Sequence(values)) if values.is_empty()));
        assert!(matches!(metadata.get("tags"), Some(serde_yaml::Value::Sequence(values)) if values.is_empty()));
    }

    #[test]
    fn preserves_existing_content_and_avoids_duplicate_sections() {
        let original = "# Summary\n\nMy original words.\n\n## Custom Notes\n\nKeep me.";
        let template = "# Summary\n\nPlaceholder\n\n## Background\n\n## Custom Notes\n\nPlaceholder";
        let (merged, added) = merge_template_sections(original, template);
        assert!(merged.contains("My original words."));
        assert!(merged.contains("Keep me."));
        assert!(!merged.contains("Placeholder"));
        assert_eq!(added, vec!["Background"]);
        let (again, second_added) = merge_template_sections(&merged, template);
        assert_eq!(merged, again);
        assert!(second_added.is_empty());
    }
}
