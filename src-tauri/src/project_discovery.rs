//! Disposable, database-free discovery of existing LithoMark project folders.
//! The project.yaml in each folder is authoritative; this module never edits it.
use std::collections::HashMap;
use std::path::PathBuf;

use uuid::Uuid;

use crate::models::{Project, SourceType};
use crate::project_root::{read_project_manifest, ProjectManifest, PROJECT_MANIFEST_FILENAME};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredProject {
    pub root: PathBuf,
    pub manifest: ProjectManifest,
}

/// Inspect explicitly supplied project roots without creating or changing files.
/// A missing folder is skipped, but an invalid manifest is an error rather than
/// silently accepting a different book at a previously remembered path.
pub fn discover_project_roots(roots: &[PathBuf]) -> Result<Vec<DiscoveredProject>, String> {
    let mut projects: HashMap<Uuid, DiscoveredProject> = HashMap::new();
    for root in roots {
        if !root.exists() {
            continue;
        }
        if !root.is_absolute() {
            return Err(format!("Project root must be absolute: {}", root.display()));
        }
        if !root.join(PROJECT_MANIFEST_FILENAME).is_file() {
            return Err(format!("No project.yaml in {}", root.display()));
        }
        let canonical = root.canonicalize()
            .map_err(|e| format!("Cannot resolve {}: {e}", root.display()))?;
        let manifest = read_project_manifest(&canonical)?;
        let id = Uuid::parse_str(&manifest.project_id).map_err(|e| e.to_string())?;
        if let Some(previous) = projects.get(&id) {
            if previous.root != canonical {
                return Err(format!(
                    "Project identity {id} occurs in two different folders: {} and {}",
                    previous.root.display(), canonical.display()
                ));
            }
            continue;
        }
        projects.insert(id, DiscoveredProject { root: canonical, manifest });
    }
    let mut found: Vec<_> = projects.into_values().collect();
    found.sort_by(|a, b| a.root.cmp(&b.root));
    Ok(found)
}


/// Reconstruct the runtime project shell from the authoritative project.yaml.
/// Fields not represented in the manifest deliberately use neutral defaults;
/// transitional callers may merge legacy metadata after this step.
pub fn project_from_root(root: &PathBuf) -> Result<Project, String> {
    let canonical = root.canonicalize()
        .map_err(|e| format!("Cannot resolve {}: {e}", root.display()))?;
    let manifest = read_project_manifest(&canonical)?;
    let id = Uuid::parse_str(&manifest.project_id).map_err(|e| e.to_string())?;
    Ok(Project {
        id,
        name: manifest.title,
        source_type: SourceType::Blank,
        source_path: None,
        project_root: Some(canonical.to_string_lossy().into_owned()),
        created_at: manifest.created_at.clone(),
        modified_at: manifest.modified_at.unwrap_or(manifest.created_at),
        author_pen_name: manifest.author_pen_name,
        genre: manifest.genre,
        description: manifest.description,
        word_target: manifest.word_target,
        reference_types: manifest.reference_types.unwrap_or_else(Project::default_reference_types),
        project_type: manifest.project_type,
        target_page_count: manifest.target_page_count,
    })
}


/// Resolve a cached project path only after verifying its manifest identity.
/// A remembered path is a locator, never proof that the folder belongs to the
/// requested book (for example, after a folder has been replaced).
pub fn verified_project_root(root: &str, expected_id: Uuid) -> Result<PathBuf, String> {
    if root.trim().is_empty() {
        return Err("Project root is empty.".into());
    }
    let path = PathBuf::from(root);
    if !path.is_absolute() {
        return Err("Stored project root must be absolute.".into());
    }
    let project = project_from_root(&path)?;
    if project.id != expected_id {
        return Err(format!(
            "Project folder belongs to {}, not requested project {}.",
            project.id, expected_id
        ));
    }
    Ok(PathBuf::from(project.project_root.expect("validated project has root")))
}

/// Create a new standalone book at an explicitly selected empty folder.
/// Refuses existing content and never touches SQLite.
pub fn create_file_project(root: &std::path::Path, title: &str, project_type: &str) -> Result<Project, String> {
    if title.trim().is_empty() {
        return Err("Book title must not be empty.".into());
    }
    if project_type != "novel" && project_type != "screenplay" {
        return Err("Unsupported project type.".into());
    }
    if !root.is_absolute() {
        return Err("Book folder must be an absolute path.".into());
    }
    let manifest = ProjectManifest::new(title.trim(), project_type);
    crate::project_root::initialize_project_root_from_manifest(root, &manifest)?;
    project_from_root(&root.to_path_buf())
}

#[cfg(test)]
mod file_creation_tests {
    use super::*;

    #[test]
    fn creates_independent_books_without_sqlite() {
        let temp = tempfile::tempdir().unwrap();
        let first = create_file_project(&temp.path().join("first"), "First", "novel").unwrap();
        let second = create_file_project(&temp.path().join("second"), "Second", "screenplay").unwrap();
        assert_ne!(first.id, second.id);
        assert_eq!(first.name, "First");
        assert_eq!(second.project_type, "screenplay");
        assert!(temp.path().join("first").join("project.yaml").exists());
        assert!(temp.path().join("second").join("project.yaml").exists());
    }

    #[test]
    fn refuses_nonempty_unowned_folder_without_changing_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("book");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("manuscript.md"), "keep me").unwrap();
        assert!(create_file_project(&root, "New", "novel").is_err());
        assert_eq!(std::fs::read_to_string(root.join("manuscript.md")).unwrap(), "keep me");
        assert!(!root.join("project.yaml").exists());
    }

    #[test]
    fn refuses_recreating_existing_book() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("book");
        let original = create_file_project(&root, "Original", "novel").unwrap();
        assert!(create_file_project(&root, "Replacement", "novel").is_err());
        assert_eq!(project_from_root(&root).unwrap().id, original.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_root::{initialize_project_root_from_manifest, ProjectManifest};

    #[test]
    fn discovers_multiple_independent_books_without_sqlite() {
        let temp = tempfile::tempdir().unwrap();
        let a = temp.path().join("A");
        let b = temp.path().join("B");
        let first = ProjectManifest::new("First Book", "novel");
        let second = ProjectManifest::new("Second Book", "novel");
        initialize_project_root_from_manifest(&a, &first).unwrap();
        initialize_project_root_from_manifest(&b, &second).unwrap();
        let found = discover_project_roots(&[b.clone(), a.clone(), a]).unwrap();
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].manifest, first);
        assert_eq!(found[1].manifest, second);
    }

    #[test]
    fn missing_folder_is_skipped_but_unowned_folder_is_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("missing");
        assert!(discover_project_roots(&[missing]).unwrap().is_empty());
        assert!(discover_project_roots(&[temp.path().to_path_buf()]).is_err());
    }

    #[test]
    fn duplicate_identity_in_different_folders_is_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let a = temp.path().join("A");
        let b = temp.path().join("B");
        let manifest = ProjectManifest::new("Same Identity", "novel");
        initialize_project_root_from_manifest(&a, &manifest).unwrap();
        initialize_project_root_from_manifest(&b, &manifest).unwrap();
        assert!(discover_project_roots(&[a, b]).unwrap_err().contains("two different folders"));
    }

    #[test]
    fn restores_optional_settings_from_manifest_without_touching_manuscript() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("Settings Book");
        let mut manifest = ProjectManifest::new("Settings Book", "novel");
        manifest.author_pen_name = Some("J. Writer".into());
        manifest.genre = Some("Thriller".into());
        manifest.description = Some("A synopsis".into());
        manifest.reference_types = Some(vec!["character".into()]);
        manifest.target_page_count = Some(350);
        manifest.modified_at = Some("2026-10-08T12:00:00Z".into());
        initialize_project_root_from_manifest(&root, &manifest).unwrap();
        std::fs::write(root.join("Manuscript").join("chapter.md"), "Original chapter").unwrap();
        let loaded = project_from_root(&root).unwrap();
        assert_eq!(loaded.author_pen_name.as_deref(), Some("J. Writer"));
        assert_eq!(loaded.genre.as_deref(), Some("Thriller"));
        assert_eq!(loaded.description.as_deref(), Some("A synopsis"));
        assert_eq!(loaded.reference_types, vec!["character"]);
        assert_eq!(loaded.target_page_count, Some(350));
        assert_eq!(loaded.modified_at, "2026-10-08T12:00:00Z");
        assert_eq!(std::fs::read_to_string(root.join("Manuscript").join("chapter.md")).unwrap(), "Original chapter");
    }

    #[test]
    fn reconstructs_runtime_project_from_manifest_without_database() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("Book");
        let mut manifest = ProjectManifest::new("Manifest Book", "novel");
        manifest.word_target = Some(90_000);
        initialize_project_root_from_manifest(&root, &manifest).unwrap();

        let project = project_from_root(&root).unwrap();
        assert_eq!(project.id.to_string(), manifest.project_id);
        assert_eq!(project.name, "Manifest Book");
        assert_eq!(project.project_type, "novel");
        assert_eq!(project.word_target, Some(90_000));
        assert_eq!(project.project_root.as_deref(), Some(root.canonicalize().unwrap().to_string_lossy().as_ref()));
    }

    #[test]
    fn cached_root_must_match_requested_project_identity() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("Book");
        let manifest = ProjectManifest::new("Book", "novel");
        initialize_project_root_from_manifest(&root, &manifest).unwrap();
        let id = Uuid::parse_str(&manifest.project_id).unwrap();
        assert_eq!(verified_project_root(root.to_str().unwrap(), id).unwrap(), root.canonicalize().unwrap());
        assert!(verified_project_root(root.to_str().unwrap(), Uuid::new_v4())
            .unwrap_err().contains("belongs to"));
    }

    #[test]
    fn moving_a_book_preserves_its_identity() {
        let temp = tempfile::tempdir().unwrap();
        let original = temp.path().join("Before");
        let moved = temp.path().join("After");
        let manifest = ProjectManifest::new("Portable Book", "novel");
        initialize_project_root_from_manifest(&original, &manifest).unwrap();
        std::fs::rename(&original, &moved).unwrap();
        let found = discover_project_roots(&[original, moved.clone()]).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].root, moved.canonicalize().unwrap());
        assert_eq!(found[0].manifest.project_id, manifest.project_id);
    }
}
