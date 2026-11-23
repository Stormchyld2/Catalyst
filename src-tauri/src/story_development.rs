//! File-authoritative Story Development index.
//!
//! The index is a projection over ordinary LithoMark Markdown files. It owns no
//! content and can be rebuilt at any time by scanning the project tree.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_yaml::Value;

use crate::document::service::{read_document, save_document, LithoMarkDocument};

// New LithoMark projects write story-domain documents beneath this canonical root.
// Legacy roots remain readable so existing Kindling/early-LithoMark projects can be
// opened without destructive migration.
const CANONICAL_STORY_DEVELOPMENT_ROOT: &str = "Story Development";
const LEGACY_STORY_DEVELOPMENT_ROOTS: &[&str] = &[
    "Story Bible",
    "Story",
    "Characters",
    "Locations",
    "Worldbuilding",
    "Timeline",
    "Research",
    "Planning",
];
const SHARED_PROJECT_ASSET_ROOTS: &[&str] = &["Assets", "Images"];


const STORY_IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "svg"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StoryDevelopmentEntry {
    pub id: String,
    pub entry_kind: String,
    pub document_type: String,
    pub title: String,
    pub relative_path: String,
    pub section: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StoryDevelopmentIssue {
    pub relative_path: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct StoryDevelopmentIndex {
    pub entries: Vec<StoryDevelopmentEntry>,
    pub issues: Vec<StoryDevelopmentIssue>,
}

pub fn build_index(project_root: &Path) -> Result<StoryDevelopmentIndex, String> {
    if !project_root.is_absolute() {
        return Err("Project root must be absolute.".into());
    }

    let mut content_paths = Vec::new();
    for root_name in story_development_read_roots() {
        let absolute = project_root.join(root_name);
        if absolute.is_dir() {
            collect_story_files(project_root, &absolute, &mut content_paths)?;
        }
    }
    content_paths.sort();

    let mut index = StoryDevelopmentIndex::default();

    for relative_path in content_paths {
        if is_story_image(&relative_path) {
            let title = relative_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("Image")
                .to_string();
            let display_path = relative_path.to_string_lossy().replace('\\', "/");
            index.entries.push(StoryDevelopmentEntry {
                id: format!("asset:{display_path}"),
                entry_kind: "image".into(),
                document_type: "image".into(),
                title,
                section: section_for(&relative_path),
                relative_path: display_path,
            });
            continue;
        }

        match read_document(project_root, &relative_path) {
            Ok(document) => {
                let title = document
                    .title
                    .clone()
                    .or_else(|| {
                        relative_path
                            .file_stem()
                            .and_then(|value| value.to_str())
                            .map(str::to_string)
                    })
                    .unwrap_or_else(|| "Untitled".into());

                index.entries.push(StoryDevelopmentEntry {
                    id: document.identity.id,
                    entry_kind: "document".into(),
                    document_type: document.identity.document_type,
                    title,
                    section: section_for(&relative_path),
                    relative_path: relative_path.to_string_lossy().replace('\\', "/"),
                });
            }
            Err(message) => index.issues.push(StoryDevelopmentIssue {
                relative_path: relative_path.to_string_lossy().replace('\\', "/"),
                message,
            }),
        }
    }

    index.entries.sort_by(|a, b| {
        a.section
            .cmp(&b.section)
            .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
            .then_with(|| a.relative_path.cmp(&b.relative_path))
    });

    Ok(index)
}


pub fn update_metadata_field(
    project_root: &Path,
    relative_path: &Path,
    key: &str,
    value: Value,
    expected_source_hash: &str,
) -> Result<LithoMarkDocument, String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("Metadata key must not be empty.".into());
    }
    if matches!(key, "lithomark" | "title") {
        return Err(format!("Metadata field '{key}' is reserved."));
    }

    let mut document = read_document(project_root, relative_path)?;
    document.metadata.insert(key.to_string(), value);
    save_document(project_root, &document, Some(expected_source_hash))?;
    read_document(project_root, relative_path)
}

pub fn update_document_body(
    project_root: &Path,
    relative_path: &Path,
    body: &str,
    expected_source_hash: &str,
) -> Result<LithoMarkDocument, String> {
    let mut document = read_document(project_root, relative_path)?;
    document.body = body.to_string();
    save_document(project_root, &document, Some(expected_source_hash))?;
    read_document(project_root, relative_path)
}

pub fn read_story_asset(project_root: &Path, image_reference: &Path) -> Result<Vec<u8>, String> {
    let relative_path = resolve_story_image_reference(project_root, image_reference)?;
    let path = project_root.join(&relative_path);
    fs::read(&path).map_err(|e| format!("Could not read image '{}': {e}", path.display()))
}

pub fn resolve_story_image_reference(
    project_root: &Path,
    image_reference: &Path,
) -> Result<PathBuf, String> {
    if !project_root.is_absolute() {
        return Err("Project root must be absolute.".into());
    }
    if image_reference.as_os_str().is_empty() || image_reference.is_absolute() {
        return Err("Asset reference must be a non-empty project-relative path or filename.".into());
    }
    if !is_story_image(image_reference) {
        return Err("Story Development asset is not a supported image type.".into());
    }

    for component in image_reference.components() {
        if !matches!(component, std::path::Component::Normal(_)) {
            return Err("Asset reference must not escape the project root.".into());
        }
    }

    let component_count = image_reference.components().count();
    if component_count > 1 {
        let path = project_root.join(image_reference);
        if !path.is_file() {
            return Err(format!("Image '{}' does not exist.", image_reference.display()));
        }
        return Ok(image_reference.to_path_buf());
    }

    let requested_name = image_reference
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Image filename is not valid UTF-8.".to_string())?
        .to_ascii_lowercase();

    let mut matches = Vec::new();
    for root_name in story_development_read_roots() {
        let absolute = project_root.join(root_name);
        if absolute.is_dir() {
            collect_matching_images(project_root, &absolute, &requested_name, &mut matches)?;
        }
    }

    matches.sort();
    matches.dedup();

    match matches.len() {
        0 => Err(format!("Image '{}' was not found in the project.", image_reference.display())),
        1 => Ok(matches.remove(0)),
        _ => Err(format!(
            "Image reference '{}' is ambiguous. Use a project-root-relative path. Matches: {}",
            image_reference.display(),
            matches
                .iter()
                .map(|path| path.to_string_lossy().replace('\\', "/"))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

fn story_development_read_roots() -> impl Iterator<Item = &'static str> {
    std::iter::once(CANONICAL_STORY_DEVELOPMENT_ROOT)
        .chain(LEGACY_STORY_DEVELOPMENT_ROOTS.iter().copied())
        .chain(SHARED_PROJECT_ASSET_ROOTS.iter().copied())
}

fn collect_matching_images(
    project_root: &Path,
    directory: &Path,
    requested_name: &str,
    output: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|e| format!("Could not read '{}': {e}", directory.display()))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Could not read project directory entry: {e}"))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Could not inspect '{}': {e}", path.display()))?;

        if file_type.is_dir() {
            collect_matching_images(project_root, &path, requested_name, output)?;
        } else if file_type.is_file() {
            let relative = path
                .strip_prefix(project_root)
                .map_err(|_| "Story Development path escaped the project root.".to_string())?;
            let filename_matches = relative
                .file_name()
                .and_then(|value| value.to_str())
                .map(|value| value.to_ascii_lowercase() == requested_name)
                .unwrap_or(false);
            if filename_matches && is_story_image(relative) {
                output.push(relative.to_path_buf());
            }
        }
    }

    Ok(())
}

pub fn image_mime_type(relative_path: &Path) -> Result<&'static str, String> {
    match relative_path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => Ok("image/png"),
        Some("jpg") | Some("jpeg") => Ok("image/jpeg"),
        Some("webp") => Ok("image/webp"),
        Some("gif") => Ok("image/gif"),
        Some("svg") => Ok("image/svg+xml"),
        _ => Err("Unsupported Story Development image type.".into()),
    }
}

fn collect_story_files(
    project_root: &Path,
    directory: &Path,
    output: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|e| format!("Could not read '{}': {e}", directory.display()))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Could not read project directory entry: {e}"))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Could not inspect '{}': {e}", path.display()))?;

        if file_type.is_dir() {
            collect_story_files(project_root, &path, output)?;
        } else if file_type.is_file() {
            let relative = path
                .strip_prefix(project_root)
                .map_err(|_| "Story Development path escaped the project root.".to_string())?;
            let is_markdown = path.extension().and_then(|v| v.to_str()) == Some("md");
            let is_support_markdown = relative
                .file_name()
                .and_then(|value| value.to_str())
                .map(|value| value.eq_ignore_ascii_case("README.md"))
                .unwrap_or(false);
            if (is_markdown && !is_support_markdown) || is_story_image(relative) {
                output.push(relative.to_path_buf());
            }
        }
    }

    Ok(())
}

fn is_story_image(relative_path: &Path) -> bool {
    relative_path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| {
            let lower = value.to_ascii_lowercase();
            STORY_IMAGE_EXTENSIONS.contains(&lower.as_str())
        })
        .unwrap_or(false)
}

fn section_for(relative_path: &Path) -> String {
    relative_path
        .components()
        .next()
        .and_then(|component| component.as_os_str().to_str())
        .unwrap_or("Story Development")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::service::create_document;

    #[test]
    fn built_in_and_custom_template_assignments_survive_reopen_with_unknown_metadata() {
        use crate::entity_templates::load_registry;

        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let templates = root.join("Templates");
        fs::create_dir_all(&templates).unwrap();
        fs::write(
            templates.join("Custom.md"),
            "---\nlithomark_template:\n  id: custom-character-test\n  name: Custom Character Test\n  category: character\n  document_type: character\n  version: 1\n  fields:\n    - key: role\n      label: Role\n      type: text\n---\n",
        ).unwrap();
        let registry = load_registry(&templates).unwrap();
        assert!(registry.issues.is_empty());
        for (path, template_id) in [
            ("Characters/Built In.md", "character-profile"),
            ("Characters/Custom.md", "custom-character-test"),
        ] {
            assert!(registry.templates.iter().any(|template| template.manifest.id == template_id));
            let relative = Path::new(path);
            let document = create_document(&root, relative, "character", Some("Test Character"), "Body unchanged.").unwrap();
            let document = update_metadata_field(
                &root, relative, "unrecognized_extension",
                Value::Mapping(serde_yaml::Mapping::from_iter([
                    (Value::String("nested".into()), Value::String("preserve me".into())),
                ])),
                &document.source_hash,
            ).unwrap();
            let document = update_metadata_field(
                &root, relative, "template_id", Value::String(template_id.into()), &document.source_hash,
            ).unwrap();
            let reopened = read_document(&root, relative).unwrap();
            assert_eq!(reopened.identity.id, document.identity.id);
            assert_eq!(reopened.body, "Body unchanged.\n");
            assert_eq!(reopened.metadata.get("template_id"), Some(&Value::String(template_id.into())));
            assert_eq!(reopened.metadata.get("unrecognized_extension"), document.metadata.get("unrecognized_extension"));
        }
    }

    #[test]
    fn indexes_file_authoritative_story_development_documents() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();

        let character = create_document(
            &root,
            Path::new("Characters/Test Character.md"),
            "character",
            Some("Test Character"),
            "Synthetic character fixture.",
        )
        .unwrap();
        let location = create_document(
            &root,
            Path::new("Locations/Test Location.md"),
            "location",
            Some("Test Location"),
            "Synthetic location fixture.",
        )
        .unwrap();
        create_document(
            &root,
            Path::new("Story Development/Synopsis.md"),
            "note",
            Some("Synopsis"),
            "Story summary.",
        )
        .unwrap();

        let index = build_index(&root).unwrap();

        assert_eq!(index.entries.len(), 3);
        assert!(index.issues.is_empty());
        assert!(index.entries.iter().any(|entry| {
            entry.id == character.identity.id
                && entry.document_type == "character"
                && entry.section == "Characters"
        }));
        assert!(index.entries.iter().any(|entry| {
            entry.id == location.identity.id
                && entry.relative_path == "Locations/Test Location.md"
        }));
    }

    #[test]
    fn ignores_readme_support_markdown_without_lithomark_frontmatter() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        fs::create_dir_all(root.join("Assets")).unwrap();
        fs::write(
            root.join("Assets/README.md"),
            "# Assets\n\nStore project images in this folder.\n",
        )
        .unwrap();

        let index = build_index(&root).unwrap();

        assert!(index.issues.is_empty());
        assert!(!index
            .entries
            .iter()
            .any(|entry| entry.relative_path == "Assets/README.md"));
    }

    #[test]
    fn indexes_supported_local_images() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        fs::create_dir_all(root.join("Assets/Characters")).unwrap();
        fs::write(root.join("Assets/Characters/Test Portrait.png"), [0u8, 1, 2]).unwrap();

        let index = build_index(&root).unwrap();
        let image = index
            .entries
            .iter()
            .find(|entry| entry.relative_path == "Assets/Characters/Test Portrait.png")
            .unwrap();

        assert_eq!(image.entry_kind, "image");
        assert_eq!(image.document_type, "image");
        assert_eq!(image.section, "Assets");
    }

    #[test]
    fn reads_supported_image_bytes_and_rejects_traversal() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        fs::create_dir_all(root.join("Images")).unwrap();
        fs::write(root.join("Images/Test.png"), [7u8, 8, 9]).unwrap();

        assert_eq!(
            read_story_asset(&root, Path::new("Images/Test.png")).unwrap(),
            vec![7u8, 8, 9]
        );
        assert!(read_story_asset(&root, Path::new("../Test.png")).is_err());
    }

    #[test]
    fn resolves_unique_image_by_filename_anywhere_in_story_roots() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        fs::create_dir_all(root.join("Assets/Locations")).unwrap();
        fs::write(root.join("Assets/Locations/old-quay.png"), [1u8, 2, 3]).unwrap();

        assert_eq!(
            resolve_story_image_reference(&root, Path::new("old-quay.png")).unwrap(),
            PathBuf::from("Assets/Locations/old-quay.png")
        );
        assert_eq!(
            read_story_asset(&root, Path::new("old-quay.png")).unwrap(),
            vec![1u8, 2, 3]
        );
    }

    #[test]
    fn filename_lookup_is_case_insensitive() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        fs::create_dir_all(root.join("Assets/Characters")).unwrap();
        fs::write(root.join("Assets/Characters/Mara-Vale.PNG"), [4u8, 5, 6]).unwrap();

        assert_eq!(
            resolve_story_image_reference(&root, Path::new("mara-vale.png")).unwrap(),
            PathBuf::from("Assets/Characters/Mara-Vale.PNG")
        );
    }

    #[test]
    fn ambiguous_filename_requires_explicit_project_relative_path() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        fs::create_dir_all(root.join("Assets/Characters")).unwrap();
        fs::create_dir_all(root.join("Images/Archive")).unwrap();
        fs::write(root.join("Assets/Characters/portrait.png"), [1u8]).unwrap();
        fs::write(root.join("Images/Archive/portrait.png"), [2u8]).unwrap();

        let error = resolve_story_image_reference(&root, Path::new("portrait.png")).unwrap_err();
        assert!(error.contains("ambiguous"));
        assert!(error.contains("Assets/Characters/portrait.png"));
        assert!(error.contains("Images/Archive/portrait.png"));

        assert_eq!(
            read_story_asset(&root, Path::new("Assets/Characters/portrait.png")).unwrap(),
            vec![1u8]
        );
    }

    #[test]
    fn missing_optional_story_development_directories_are_not_errors() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();

        let index = build_index(&root).unwrap();

        assert!(index.entries.is_empty());
        assert!(index.issues.is_empty());
    }

    #[test]
    fn reports_invalid_markdown_documents_without_hiding_valid_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();

        create_document(
            &root,
            Path::new("Characters/Valid.md"),
            "character",
            Some("Valid"),
            "Good.",
        )
        .unwrap();

        fs::create_dir_all(root.join("Locations")).unwrap();
        fs::write(root.join("Locations/Broken.md"), "---\nlithomark: [broken\n---\n").unwrap();

        let index = build_index(&root).unwrap();

        assert_eq!(index.entries.len(), 1);
        assert_eq!(index.issues.len(), 1);
        assert_eq!(index.issues[0].relative_path, "Locations/Broken.md");
        assert!(index.issues[0].message.contains("Invalid document YAML"));
    }


    #[test]
    fn updates_metadata_through_document_service_and_preserves_body() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = Path::new("Characters/Test Character.md");
        let created = create_document(
            &root,
            path,
            "character",
            Some("Test Character"),
            "Fixture body.",
        )
        .unwrap();

        let updated = update_metadata_field(
            &root,
            path,
            "status",
            Value::String("active".into()),
            &created.source_hash,
        )
        .unwrap();

        assert_eq!(
            updated.metadata.get("status"),
            Some(&Value::String("active".into()))
        );
        assert_eq!(updated.body, "Fixture body.\n");
        assert_ne!(updated.source_hash, created.source_hash);
    }

    #[test]
    fn metadata_update_rejects_stale_hash() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = Path::new("Notes/Test.md");
        let created =
            create_document(&root, path, "note", Some("Test"), "Original.").unwrap();

        fs::write(
            root.join(path),
            fs::read_to_string(root.join(path))
                .unwrap()
                .replace("Original.", "External."),
        )
        .unwrap();

        let error = update_metadata_field(
            &root,
            path,
            "status",
            Value::String("active".into()),
            &created.source_hash,
        )
        .unwrap_err();

        assert!(error.contains("conflict"));
    }

    #[test]
    fn metadata_update_rejects_reserved_fields() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = Path::new("Notes/Test.md");
        let created = create_document(&root, path, "note", Some("Test"), "Body.").unwrap();

        let error = update_metadata_field(
            &root,
            path,
            "title",
            Value::String("Replacement".into()),
            &created.source_hash,
        )
        .unwrap_err();

        assert!(error.contains("reserved"));
    }

    #[test]
    fn updates_document_body_through_document_service() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = Path::new("Notes/Test.md");
        let created = create_document(&root, path, "note", Some("Test"), "Original body.").unwrap();

        let updated = update_document_body(
            &root,
            path,
            "# Revised\n\nUpdated body.",
            &created.source_hash,
        )
        .unwrap();

        assert_eq!(updated.body, "# Revised\n\nUpdated body.\n");
        assert_eq!(updated.title.as_deref(), Some("Test"));
        assert_ne!(updated.source_hash, created.source_hash);
    }

    #[test]
    fn body_update_rejects_stale_hash() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = Path::new("Notes/Test.md");
        let created = create_document(&root, path, "note", Some("Test"), "Original.").unwrap();

        fs::write(
            root.join(path),
            fs::read_to_string(root.join(path))
                .unwrap()
                .replace("Original.", "External."),
        )
        .unwrap();

        let error = update_document_body(
            &root,
            path,
            "LithoMark edit.",
            &created.source_hash,
        )
        .unwrap_err();

        assert!(error.contains("conflict"));
        assert!(fs::read_to_string(root.join(path)).unwrap().contains("External."));
    }

    #[test]
    fn rejects_relative_project_roots() {
        let error = build_index(Path::new("relative/project")).unwrap_err();
        assert!(error.contains("absolute"));
    }
}
