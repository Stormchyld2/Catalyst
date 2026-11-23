//! Disposable recent-project location index. Never stores manuscript content.
//! Each location is verified against the book's own project.yaml on lookup.
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::project_discovery::verified_project_root;

const FILE_NAME: &str = "recent-projects.json";

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    version: u32,
    roots: BTreeMap<String, String>,
}

fn registry_path(app_data: &Path) -> PathBuf {
    app_data.join(FILE_NAME)
}

fn read(app_data: &Path) -> Result<Registry, String> {
    let path = registry_path(app_data);
    if !path.exists() {
        return Ok(Registry { version: 1, roots: BTreeMap::new() });
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("Cannot read recent projects: {e}"))?;
    let registry: Registry = serde_json::from_str(&raw)
        .map_err(|e| format!("Invalid recent projects index: {e}"))?;
    if registry.version != 1 {
        return Err(format!("Unsupported recent projects index version: {}", registry.version));
    }
    Ok(registry)
}

fn write(app_data: &Path, registry: &Registry) -> Result<(), String> {
    fs::create_dir_all(app_data).map_err(|e| format!("Cannot create application data directory: {e}"))?;
    let destination = registry_path(app_data);
    let staged = app_data.join(format!(".recent-projects-{}.tmp", Uuid::new_v4()));
    let bytes = serde_json::to_vec_pretty(registry).map_err(|e| e.to_string())?;
    fs::write(&staged, bytes).map_err(|e| format!("Cannot stage recent projects: {e}"))?;
    // On Windows rename cannot replace an existing destination. Keep the
    // existing index intact on failure rather than deleting it first.
    if destination.exists() {
        let backup = app_data.join(format!(".recent-projects-{}.bak", Uuid::new_v4()));
        if let Err(e) = fs::rename(&destination, &backup) {
            let _ = fs::remove_file(&staged);
            return Err(format!("Cannot stage previous recent projects index: {e}"));
        }
        match fs::rename(&staged, &destination) {
            Ok(()) => { let _ = fs::remove_file(&backup); Ok(()) }
            Err(e) => {
                let _ = fs::rename(&backup, &destination);
                let _ = fs::remove_file(&staged);
                Err(format!("Cannot replace recent projects index: {e}"))
            }
        }
    } else {
        fs::rename(&staged, &destination).map_err(|e| format!("Cannot save recent projects: {e}"))
    }
}

/// Record only a verified project identity and folder path.
pub fn remember(app_data: &Path, id: Uuid, root: &Path) -> Result<(), String> {
    let verified = verified_project_root(&root.to_string_lossy(), id)?;
    let mut registry = read(app_data)?;
    registry.roots.insert(id.to_string(), verified.to_string_lossy().into_owned());
    write(app_data, &registry)
}

/// Forget a book in the disposable recent-projects index only.
/// Never remove its project directory, manuscript, or local snapshots.
pub fn forget(app_data: &Path, id: Uuid) -> Result<(), String> {
    let mut registry = read(app_data)?;
    if registry.roots.remove(&id.to_string()).is_some() {
        write(app_data, &registry)?;
    }
    Ok(())
}

/// Missing or moved paths are not trusted; the caller can ask the user to reopen.
pub fn lookup(app_data: &Path, id: Uuid) -> Result<Option<PathBuf>, String> {
    let registry = read(app_data)?;
    let Some(root) = registry.roots.get(&id.to_string()) else { return Ok(None); };
    if !Path::new(root).exists() { return Ok(None); }
    verified_project_root(root, id).map(Some)
}

/// Enumerate registered books without consulting a database. Missing folders
/// are omitted; mismatched identities and invalid manifests remain errors.
pub fn list_projects(app_data: &Path) -> Result<Vec<crate::models::Project>, String> {
    let registry = read(app_data)?;
    let mut projects = Vec::new();
    for (id, root) in registry.roots {
        let id = Uuid::parse_str(&id).map_err(|e| format!("Invalid registry identity: {e}"))?;
        if !Path::new(&root).exists() {
            continue;
        }
        let verified = verified_project_root(&root, id)?;
        projects.push(crate::project_discovery::project_from_root(&verified)?);
    }
    projects.sort_by(|a, b| b.modified_at.cmp(&a.modified_at).then_with(|| a.name.cmp(&b.name)));
    Ok(projects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_root::{initialize_project_root_from_manifest, ProjectManifest};

    #[test]
    fn forgetting_a_book_preserves_its_manuscript_and_other_books() {
        let temp = tempfile::tempdir().unwrap();
        let app = temp.path().join("app");
        let mut ids = Vec::new();
        for name in ["First", "Second"] {
            let root = temp.path().join(name);
            let manifest = ProjectManifest::new(name, "novel");
            let id = Uuid::parse_str(&manifest.project_id).unwrap();
            initialize_project_root_from_manifest(&root, &manifest).unwrap();
            let manuscript = root.join("Manuscript").join("chapter.md");
            fs::create_dir_all(manuscript.parent().unwrap()).unwrap();
            fs::write(&manuscript, "Original chapter").unwrap();
            remember(&app, id, &root).unwrap();
            ids.push((id, root, manuscript));
        }
        forget(&app, ids[0].0).unwrap();
        assert!(lookup(&app, ids[0].0).unwrap().is_none());
        assert!(lookup(&app, ids[1].0).unwrap().is_some());
        assert_eq!(fs::read_to_string(&ids[0].2).unwrap(), "Original chapter");
        assert!(ids[0].1.join("project.yaml").exists());
        forget(&app, ids[0].0).unwrap();
        remember(&app, ids[0].0, &ids[0].1).unwrap();
        assert!(lookup(&app, ids[0].0).unwrap().is_some());
    }

    #[test]
    fn remembers_two_books_and_reopens_without_sqlite() {
        let temp = tempfile::tempdir().unwrap();
        let app = temp.path().join("app");
        for name in ["A", "B"] {
            let root = temp.path().join(name);
            let manifest = ProjectManifest::new(name, "novel");
            let id = Uuid::parse_str(&manifest.project_id).unwrap();
            initialize_project_root_from_manifest(&root, &manifest).unwrap();
            remember(&app, id, &root).unwrap();
            assert_eq!(lookup(&app, id).unwrap(), Some(root.canonicalize().unwrap()));
        }
        assert_eq!(read(&app).unwrap().roots.len(), 2);
    }

    #[test]
    fn refuses_wrong_book_and_does_not_change_registry() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("Book");
        let app = temp.path().join("app");
        let manifest = ProjectManifest::new("Book", "novel");
        initialize_project_root_from_manifest(&root, &manifest).unwrap();
        assert!(remember(&app, Uuid::new_v4(), &root).is_err());
        assert!(!registry_path(&app).exists());
    }

    #[test]
    fn missing_moved_folder_is_not_assumed_to_be_valid() {
        let temp = tempfile::tempdir().unwrap();
        let app = temp.path().join("app");
        let root = temp.path().join("Original");
        let moved = temp.path().join("Moved");
        let manifest = ProjectManifest::new("Book", "novel");
        let id = Uuid::parse_str(&manifest.project_id).unwrap();
        initialize_project_root_from_manifest(&root, &manifest).unwrap();
        remember(&app, id, &root).unwrap();
        fs::rename(&root, &moved).unwrap();
        assert_eq!(lookup(&app, id).unwrap(), None);
        remember(&app, id, &moved).unwrap();
        assert_eq!(lookup(&app, id).unwrap(), Some(moved.canonicalize().unwrap()));
    }

    #[test]
    fn corrupt_index_fails_without_overwriting_it() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(registry_path(temp.path()), "{invalid").unwrap();
        assert!(lookup(temp.path(), Uuid::new_v4()).is_err());
        assert_eq!(fs::read_to_string(registry_path(temp.path())).unwrap(), "{invalid");
    }
}
