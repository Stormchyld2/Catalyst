//! Native Catalyst project-root initialization.
//!
//! A project root is the authoritative filesystem boundary for a Catalyst
//! project. Project identity lives in project.yaml; runtime project services
//! must be able to reconstruct the project from the local folder alone.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::Project;

pub const PROJECT_FORMAT: &str = "catalyst-project";
/// Format marker written before the Catalyst rename. Projects that carry it
/// open normally and keep it; it is never rewritten silently.
pub const LEGACY_PROJECT_FORMAT: &str = "lithomark-project";
pub const PROJECT_FORMAT_VERSION: u32 = 1;
pub const PROJECT_MANIFEST_FILENAME: &str = "project.yaml";

const PROJECT_DIRECTORIES: &[&str] = &[
    "Manuscript",

    "Story Development",
    "Story Development/Foundation",

    "Story Development/Planning & Outline",
    "Story Development/Planning & Outline/Master Outline",
    "Story Development/Planning & Outline/Beat Sheets",
    "Story Development/Planning & Outline/Chapter Plans",
    "Story Development/Planning & Outline/Scene Plans",
    "Story Development/Planning & Outline/Plot Threads",
    "Story Development/Planning & Outline/Subplots",
    "Story Development/Planning & Outline/Reveal Plans",
    "Story Development/Planning & Outline/Foreshadowing & Payoffs",
    "Story Development/Planning & Outline/Brainstorming",

    "Story Development/Characters",
    "Story Development/Characters/Main Cast",
    "Story Development/Characters/Supporting Characters",
    "Story Development/Characters/Relationships",
    "Story Development/Characters/Character Arcs",

    "Story Development/Worldbuilding & Settings",
    "Story Development/Worldbuilding & Settings/Locations",
    "Story Development/Worldbuilding & Settings/Organizations",
    "Story Development/Worldbuilding & Settings/Governments & Factions",
    "Story Development/Worldbuilding & Settings/Cultures & Societies",
    "Story Development/Worldbuilding & Settings/Systems",
    "Story Development/Worldbuilding & Settings/Objects & Artifacts",
    "Story Development/Worldbuilding & Settings/Professions & Procedures",
    "Story Development/Worldbuilding & Settings/Setting Notes",

    "Story Development/Timeline",
    "Story Development/Timeline/Master Timeline",
    "Story Development/Timeline/Events",
    "Story Development/Timeline/Backstory",

    "Story Development/Research",
    "Story Development/Research/General",
    "Story Development/Research/People",
    "Story Development/Research/Places",
    "Story Development/Research/Professions",
    "Story Development/Research/Procedures",
    "Story Development/Research/Sources",

    "Notes",
    "Notes/Author Notes",
    "Notes/Continuity",
    "Notes/Questions",
    "Notes/Scratch",

    "Project Meta",
    "Project Meta/Writing Guides",
    "Project Meta/Workflow",
    "Project Meta/Change Control",
    "Project Meta/Project Documentation",

    "Reviews",
    "Reviews/Developmental",
    "Reviews/Beta",
    "Reviews/ARC",
    "Reviews/Rounds",
    "Reviews/Distribution",

    "Audiobook",

    "Assets",
    "Assets/Characters",
    "Assets/Locations",
    "Assets/Maps",
    "Assets/Objects",
    "Assets/Covers",
    "Assets/Other",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub format: String,
    pub format_version: u32,
    pub project_id: String,
    pub title: String,
    pub project_type: String,
    pub created_at: String,
    #[serde(default)]
    pub word_target: Option<i32>,
    #[serde(default)]
    pub author_pen_name: Option<String>,
    #[serde(default)]
    pub genre: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub reference_types: Option<Vec<String>>,
    #[serde(default)]
    pub target_page_count: Option<i32>,
    #[serde(default)]
    pub daily_writing_goal: Option<i64>,
    #[serde(default)]
    pub modified_at: Option<String>,
}

impl ProjectManifest {
    pub fn new(title: impl Into<String>, project_type: impl Into<String>) -> Self {
        Self {
            format: PROJECT_FORMAT.to_string(),
            format_version: PROJECT_FORMAT_VERSION,
            project_id: Uuid::new_v4().to_string(),
            title: title.into(),
            project_type: project_type.into(),
            created_at: Utc::now().to_rfc3339(),
            word_target: None,
            author_pen_name: None,
            genre: None,
            description: None,
            reference_types: None,
            target_page_count: None,
            daily_writing_goal: None,
            modified_at: None,
        }
    }

    pub fn from_project(project: &Project) -> Self {
        Self {
            format: PROJECT_FORMAT.to_string(),
            format_version: PROJECT_FORMAT_VERSION,
            project_id: project.id.to_string(),
            title: project.name.clone(),
            project_type: project.project_type.clone(),
            created_at: project.created_at.clone(),
            word_target: project.word_target,
            author_pen_name: project.author_pen_name.clone(),
            genre: project.genre.clone(),
            description: project.description.clone(),
            reference_types: Some(project.reference_types.clone()),
            target_page_count: project.target_page_count,
            daily_writing_goal: None,
            modified_at: Some(project.modified_at.clone()),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.format != PROJECT_FORMAT && self.format != LEGACY_PROJECT_FORMAT {
            return Err(format!(
                "Unsupported project format '{}'. Expected '{}'.",
                self.format, PROJECT_FORMAT
            ));
        }
        if self.format_version != PROJECT_FORMAT_VERSION {
            return Err(format!(
                "Unsupported project format version {}. Expected {}.",
                self.format_version, PROJECT_FORMAT_VERSION
            ));
        }
        Uuid::parse_str(&self.project_id)
            .map_err(|_| "project.yaml contains an invalid project_id.".to_string())?;
        if self.title.trim().is_empty() {
            return Err("project.yaml title must not be empty.".into());
        }
        if self.project_type.trim().is_empty() {
            return Err("project.yaml project_type must not be empty.".into());
        }
        if self.created_at.trim().is_empty() {
            return Err("project.yaml created_at must not be empty.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProjectPaths {
    pub root: String,
    pub project_manifest: String,
    pub manuscript: String,
    pub story_development: String,
    pub story_worldbuilding: String,
    pub characters: String,
    pub locations: String,
    pub timeline: String,
    pub research: String,
    pub notes: String,
    pub planning: String,
    pub reviews: String,
    pub review_rounds: String,
    pub review_distribution: String,
    pub audiobook: String,
    pub assets: String,
}

impl ProjectPaths {
    pub fn from_root(root: &Path) -> Self {
        let s = |path: PathBuf| path.to_string_lossy().into_owned();
        Self {
            root: s(root.to_path_buf()),
            project_manifest: s(root.join(PROJECT_MANIFEST_FILENAME)),
            manuscript: s(root.join("Manuscript")),
            story_development: s(root.join("Story Development")),
            story_worldbuilding: s(root.join("Story Development").join("Worldbuilding & Settings")),
            characters: s(root.join("Story Development").join("Characters")),
            locations: s(root.join("Story Development").join("Worldbuilding & Settings").join("Locations")),
            timeline: s(root.join("Story Development").join("Timeline")),
            research: s(root.join("Story Development").join("Research")),
            notes: s(root.join("Notes")),
            planning: s(root.join("Story Development").join("Planning & Outline")),
            reviews: s(root.join("Reviews")),
            review_rounds: s(root.join("Reviews").join("Rounds")),
            review_distribution: s(root.join("Reviews").join("Distribution")),
            audiobook: s(root.join("Audiobook")),
            assets: s(root.join("Assets")),
        }
    }
}

pub fn initialize_project_root(root: &Path, project: &Project) -> Result<ProjectPaths, String> {
    initialize_project_root_from_manifest(root, &ProjectManifest::from_project(project))
}

pub fn initialize_project_root_from_manifest(
    root: &Path,
    manifest: &ProjectManifest,
) -> Result<ProjectPaths, String> {
    initialize_project_root_with_directories(root, manifest, PROJECT_DIRECTORIES)
}

/// Initialize a project root with an explicit directory layout. Book projects
/// use the shared Catalyst layout; campaigns supply their own tabletop layout.
pub fn initialize_project_root_with_directories(
    root: &Path,
    manifest: &ProjectManifest,
    directories: &[&str],
) -> Result<ProjectPaths, String> {
    manifest.validate()?;

    if root.as_os_str().is_empty() {
        return Err("Project root must not be empty.".into());
    }
    if !root.is_absolute() {
        return Err("Project root must be an absolute path.".into());
    }

    if root.exists() && !root.is_dir() {
        return Err("Project root points to a file, not a directory.".into());
    }

    let manifest_path = root.join(PROJECT_MANIFEST_FILENAME);
    if manifest_path.exists() {
        validate_existing_manifest_for_manifest(&manifest_path, manifest)?;
    } else if root.exists() && directory_has_entries(root)? {
        return Err(
            "Selected project root is not empty and has no Catalyst project.yaml.".into(),
        );
    }

    fs::create_dir_all(root)
        .map_err(|e| format!("Could not create project root '{}': {e}", root.display()))?;

    for relative in directories {
        fs::create_dir_all(root.join(relative)).map_err(|e| {
            format!(
                "Could not create Catalyst project directory '{}': {e}",
                relative
            )
        })?;
    }

    if !manifest_path.exists() {
        write_manifest_atomically(&manifest_path, manifest)?;
    }

    Ok(ProjectPaths::from_root(root))
}

pub fn read_project_manifest(root: &Path) -> Result<ProjectManifest, String> {
    if !root.is_absolute() {
        return Err("Project root must be an absolute path.".into());
    }
    if !root.is_dir() {
        return Err(format!(
            "Project root '{}' does not exist or is not a directory.",
            root.display()
        ));
    }

    let path = root.join(PROJECT_MANIFEST_FILENAME);
    let raw = fs::read_to_string(&path)
        .map_err(|e| format!("Could not read '{}': {e}", path.display()))?;
    let manifest: ProjectManifest =
        serde_yaml::from_str(&raw).map_err(|e| format!("Invalid project.yaml: {e}"))?;
    manifest.validate()?;
    Ok(manifest)
}

/// Replace only the verified project's manifest; never write manuscript files.
pub fn save_project_manifest(root: &Path, expected_id: Uuid, updated: &ProjectManifest) -> Result<(), String> {
    let current = read_project_manifest(root)?;
    if current.project_id != expected_id.to_string() || updated.project_id != current.project_id {
        return Err("Project identity mismatch; settings were not saved.".into());
    }
    if updated.created_at != current.created_at || updated.format != current.format ||
        updated.format_version != current.format_version {
        return Err("Project identity metadata cannot be changed in settings.".into());
    }
    updated.validate()?;
    let path = root.join(PROJECT_MANIFEST_FILENAME);
    let raw = fs::read_to_string(&path).map_err(|e| format!("Cannot read project.yaml: {e}"))?;
    // Preserve any additional YAML fields owned by other modules.
    let mut document: serde_yaml::Value =
        serde_yaml::from_str(&raw).map_err(|e| format!("Invalid project.yaml: {e}"))?;
    let map = document.as_mapping_mut().ok_or("project.yaml must contain a mapping.")?;
    let updated_value = serde_yaml::to_value(updated).map_err(|e| e.to_string())?;
    let updated_map = updated_value.as_mapping().ok_or("Invalid updated project manifest.")?;
    for (key, value) in updated_map {
        map.insert(key.clone(), value.clone());
    }
    let serialized = serde_yaml::to_string(&document).map_err(|e| e.to_string())?;
    let staged = root.join(format!(".project-settings-{}.tmp", Uuid::new_v4()));
    fs::write(&staged, serialized).map_err(|e| format!("Cannot stage project settings: {e}"))?;
    // Windows does not allow rename over an existing file. Restore the original
    // if the replacement fails; never remove the manuscript or other files.
    let backup = root.join(format!(".project-settings-{}.bak", Uuid::new_v4()));
    if let Err(e) = fs::rename(&path, &backup) {
        let _ = fs::remove_file(&staged);
        return Err(format!("Cannot back up project settings: {e}"));
    }
    match fs::rename(&staged, &path) {
        Ok(()) => { let _ = fs::remove_file(&backup); Ok(()) }
        Err(e) => {
            let restore = fs::rename(&backup, &path);
            let _ = fs::remove_file(&staged);
            match restore {
                Ok(()) => Err(format!("Cannot save project settings: {e}")),
                Err(re) => Err(format!("Cannot save project settings: {e}; original manifest remains at {} (restore error: {re})", backup.display())),
            }
        }
    }
}

pub fn resolve_project_paths(root: &Path, project: &Project) -> Result<ProjectPaths, String> {
    if !root.is_absolute() {
        return Err("Stored project root is not an absolute path.".into());
    }
    if !root.is_dir() {
        return Err(format!(
            "Project root '{}' does not exist or is not a directory.",
            root.display()
        ));
    }
    validate_existing_manifest(&root.join(PROJECT_MANIFEST_FILENAME), project)?;
    Ok(ProjectPaths::from_root(root))
}

fn directory_has_entries(root: &Path) -> Result<bool, String> {
    let mut entries = fs::read_dir(root)
        .map_err(|e| format!("Could not inspect project root '{}': {e}", root.display()))?;
    Ok(entries.next().is_some())
}

fn validate_existing_manifest(path: &Path, project: &Project) -> Result<(), String> {
    validate_existing_manifest_for_manifest(path, &ProjectManifest::from_project(project))
}

fn validate_existing_manifest_for_manifest(
    path: &Path,
    expected: &ProjectManifest,
) -> Result<(), String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("Could not read '{}': {e}", path.display()))?;
    let manifest: ProjectManifest = serde_yaml::from_str(&raw)
        .map_err(|e| format!("Invalid project.yaml: {e}"))?;
    manifest.validate()?;

    if manifest.project_id != expected.project_id {
        return Err(format!(
            "Selected folder belongs to a different Catalyst project ({}).",
            manifest.project_id
        ));
    }
    Ok(())
}

fn write_manifest_atomically(path: &Path, manifest: &ProjectManifest) -> Result<(), String> {
    manifest.validate()?;
    let mut yaml =
        serde_yaml::to_string(manifest).map_err(|e| format!("Could not serialize project.yaml: {e}"))?;
    yaml = yaml.replace("\r\n", "\n");
    if !yaml.ends_with('\n') {
        yaml.push('\n');
    }

    let parent = path
        .parent()
        .ok_or_else(|| "project.yaml has no parent directory.".to_string())?;
    let temp = parent.join(format!(".project.yaml.tmp-{}", Uuid::new_v4()));

    fs::write(&temp, yaml.as_bytes())
        .map_err(|e| format!("Could not stage project.yaml: {e}"))?;
    match fs::rename(&temp, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = fs::remove_file(&temp);
            Err(format!("Could not finalize project.yaml: {e}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SourceType;

    fn project() -> Project {
        Project::new("Test Novel".into(), SourceType::Blank, None)
    }

    #[test]
    fn saves_settings_and_preserves_manuscript_and_unknown_yaml() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("Book");
        let original = ProjectManifest::new("Book", "novel");
        initialize_project_root_from_manifest(&root, &original).unwrap();
        let chapter = root.join("Manuscript/chapter.md");
        fs::write(&chapter, "Do not change this chapter").unwrap();
        let path = root.join(PROJECT_MANIFEST_FILENAME);
        let raw = fs::read_to_string(&path).unwrap();
        fs::write(&path, format!("{raw}custom_setting: preserved\\n")).unwrap();
        let mut updated = read_project_manifest(&root).unwrap();
        updated.genre = Some("Mystery".into());
        updated.word_target = Some(85_000);
        save_project_manifest(&root, Uuid::parse_str(&original.project_id).unwrap(), &updated).unwrap();
        let reopened = read_project_manifest(&root).unwrap();
        assert_eq!(reopened.genre.as_deref(), Some("Mystery"));
        assert_eq!(reopened.word_target, Some(85_000));
        assert!(fs::read_to_string(&path).unwrap().contains("custom_setting: preserved"));
        assert_eq!(fs::read_to_string(&chapter).unwrap(), "Do not change this chapter");
    }

    #[test]
    fn refuses_wrong_project_identity_without_changing_manifest() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("Book");
        let manifest = ProjectManifest::new("Book", "novel");
        initialize_project_root_from_manifest(&root, &manifest).unwrap();
        let path = root.join(PROJECT_MANIFEST_FILENAME);
        let before = fs::read(&path).unwrap();
        let mut updated = manifest.clone();
        updated.genre = Some("Thriller".into());
        assert!(save_project_manifest(&root, Uuid::new_v4(), &updated).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
    }

    #[test]
    fn initializes_canonical_project_tree_and_manifest() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Novel");
        let project = project();

        let paths = initialize_project_root(&root, &project).unwrap();

        assert!(root.join("project.yaml").is_file());
        for relative in PROJECT_DIRECTORIES {
            assert!(root.join(relative).is_dir(), "{relative}");
        }
        assert_eq!(
            PathBuf::from(&paths.review_distribution),
            root.join("Reviews").join("Distribution")
        );

        let raw = fs::read_to_string(root.join("project.yaml")).unwrap();
        let manifest: ProjectManifest = serde_yaml::from_str(&raw).unwrap();
        assert_eq!(manifest.project_id, project.id.to_string());
        assert_eq!(manifest.format, PROJECT_FORMAT);
    }

    #[test]
    fn initialization_is_idempotent_for_same_project() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Novel");
        let project = project();

        initialize_project_root(&root, &project).unwrap();
        fs::write(root.join("Notes").join("keep.md"), "keep").unwrap();
        initialize_project_root(&root, &project).unwrap();

        assert_eq!(fs::read_to_string(root.join("Notes/keep.md")).unwrap(), "keep");
    }

    #[test]
    fn refuses_nonempty_unowned_directory() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Existing");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("other.txt"), "data").unwrap();

        let error = initialize_project_root(&root, &project()).unwrap_err();
        assert!(error.contains("not empty"));
        assert!(!root.join("project.yaml").exists());
    }

    #[test]
    fn refuses_directory_owned_by_another_project() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Novel");
        let first = project();
        let second = project();

        initialize_project_root(&root, &first).unwrap();
        let error = initialize_project_root(&root, &second).unwrap_err();

        assert!(error.contains("different Catalyst project"));
    }

    #[test]
    fn creates_and_reads_manifest_without_database_project_model() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Local Novel");
        let manifest = ProjectManifest::new("Local Novel", "novel");

        initialize_project_root_from_manifest(&root, &manifest).unwrap();
        let loaded = read_project_manifest(&root).unwrap();

        assert_eq!(loaded, manifest);
        assert!(root.join("Story Development").is_dir());
        assert!(root.join("Story Development/Planning & Outline").is_dir());
        assert!(root.join("Story Development/Characters/Main Cast").is_dir());
        assert!(root.join("Story Development/Worldbuilding & Settings/Locations").is_dir());
    }

    #[test]
    fn resolve_requires_matching_project_manifest() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Novel");
        let project = project();
        initialize_project_root(&root, &project).unwrap();

        let paths = resolve_project_paths(&root, &project).unwrap();
        assert_eq!(paths.root, root.to_string_lossy());
    }
}
