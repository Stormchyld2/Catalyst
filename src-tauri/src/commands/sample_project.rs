//! Catalyst first-run sample project.
//!
//! The bundled sample is The Glass Meridian, a normal file-authoritative
//! Catalyst project. It is copied into a disposable local sample folder and
//! opened through the same project-root model used by user-created books.

use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{Project, SourceType};
use crate::project_root::read_project_manifest;
use uuid::Uuid;

const SAMPLE_DIR_NAME: &str = "Catalyst Sample - The Glass Meridian";

const TEXT_FILES: &[(&str, &str)] = &[
    ("project.yaml", include_str!("../../../docs/projects/The Glass Meridian/project.yaml")),
    ("Assets/README.md", include_str!("../../../docs/projects/The Glass Meridian/Assets/README.md")),
    ("Characters/Elias Venn.md", include_str!("../../../docs/projects/The Glass Meridian/Characters/Elias Venn.md")),
    ("Characters/Mara Vale.md", include_str!("../../../docs/projects/The Glass Meridian/Characters/Mara Vale.md")),
    ("Characters/Nia Corren.md", include_str!("../../../docs/projects/The Glass Meridian/Characters/Nia Corren.md")),
    ("Characters/Tomas Vale.md", include_str!("../../../docs/projects/The Glass Meridian/Characters/Tomas Vale.md")),
    ("Locations/East Reach.md", include_str!("../../../docs/projects/The Glass Meridian/Locations/East Reach.md")),
    ("Locations/Meridian Operations Center.md", include_str!("../../../docs/projects/The Glass Meridian/Locations/Meridian Operations Center.md")),
    ("Locations/Old Quay.md", include_str!("../../../docs/projects/The Glass Meridian/Locations/Old Quay.md")),
    ("Manuscript/Chapter 01 - The Missing Block.md", include_str!("../../../docs/projects/The Glass Meridian/Manuscript/Chapter 01 - The Missing Block.md")),
    ("Manuscript/Chapter 02 - The Old Meridian.md", include_str!("../../../docs/projects/The Glass Meridian/Manuscript/Chapter 02 - The Old Meridian.md")),
    ("Planning/Initial Outline.md", include_str!("../../../docs/projects/The Glass Meridian/Planning/Initial Outline.md")),
    ("Research/Flood Risk Maps and Public Decision-Making.md", include_str!("../../../docs/projects/The Glass Meridian/Research/Flood Risk Maps and Public Decision-Making.md")),
    ("Story Development/Brainstorming.md", include_str!("../../../docs/projects/The Glass Meridian/Story Development/Brainstorming.md")),
    ("Story Development/Foundation.md", include_str!("../../../docs/projects/The Glass Meridian/Story Development/Foundation.md")),
    ("Story Development/Style & Voice.md", include_str!("../../../docs/projects/The Glass Meridian/Story Development/Style & Voice.md")),
    ("Story Development/Synopsis.md", include_str!("../../../docs/projects/The Glass Meridian/Story Development/Synopsis.md")),
    ("Story Development/Themes.md", include_str!("../../../docs/projects/The Glass Meridian/Story Development/Themes.md")),
    ("Story Development/Worldbuilding/Lumen and the Meridian System.md", include_str!("../../../docs/projects/The Glass Meridian/Story Development/Worldbuilding/Lumen and the Meridian System.md")),
    ("Timeline/Nightwater Flood.md", include_str!("../../../docs/projects/The Glass Meridian/Timeline/Nightwater Flood.md")),
    ("Worldbuilding/Objects/Venn Survey Compass.md", include_str!("../../../docs/projects/The Glass Meridian/Worldbuilding/Objects/Venn Survey Compass.md")),
    ("Worldbuilding/Organizations/Helix Municipal Systems.md", include_str!("../../../docs/projects/The Glass Meridian/Worldbuilding/Organizations/Helix Municipal Systems.md")),
    ("Worldbuilding/Organizations/Lumen Civic Government.md", include_str!("../../../docs/projects/The Glass Meridian/Worldbuilding/Organizations/Lumen Civic Government.md")),
];

const BINARY_FILES: &[(&str, &[u8])] = &[
    ("Assets/Characters/elias-venn.png", include_bytes!("../../../docs/projects/The Glass Meridian/Assets/Characters/elias-venn.png")),
    ("Assets/Characters/mara-vale.png", include_bytes!("../../../docs/projects/The Glass Meridian/Assets/Characters/mara-vale.png")),
    ("Assets/Characters/nia-corren.png", include_bytes!("../../../docs/projects/The Glass Meridian/Assets/Characters/nia-corren.png")),
    ("Assets/Characters/tomas-vale.png", include_bytes!("../../../docs/projects/The Glass Meridian/Assets/Characters/tomas-vale.png")),
    ("Assets/Characters/venn-compass.png", include_bytes!("../../../docs/projects/The Glass Meridian/Assets/Characters/venn-compass.png")),
    ("Assets/Locations/east-reach.png", include_bytes!("../../../docs/projects/The Glass Meridian/Assets/Locations/east-reach.png")),
    ("Assets/Locations/meridian-operations-center.png", include_bytes!("../../../docs/projects/The Glass Meridian/Assets/Locations/meridian-operations-center.png")),
    ("Assets/Locations/old-quay.png", include_bytes!("../../../docs/projects/The Glass Meridian/Assets/Locations/old-quay.png")),
];

#[tauri::command]
pub async fn create_sample_project() -> Result<Project, String> {
    let root = std::env::temp_dir().join(SAMPLE_DIR_NAME);
    seed_glass_meridian_sample(&root)
}

fn seed_glass_meridian_sample(root: &Path) -> Result<Project, String> {
    if root.exists() {
        fs::remove_dir_all(root)
            .map_err(|e| format!("Could not refresh Catalyst sample project '{}': {e}", root.display()))?;
    }
    fs::create_dir_all(root)
        .map_err(|e| format!("Could not create Catalyst sample project '{}': {e}", root.display()))?;

    for (relative, contents) in TEXT_FILES {
        write_sample_file(root, relative, contents.as_bytes())?;
    }
    for (relative, contents) in BINARY_FILES {
        write_sample_file(root, relative, contents)?;
    }

    let manifest = read_project_manifest(root)?;
    let id = Uuid::parse_str(&manifest.project_id)
        .map_err(|e| format!("Bundled sample project has an invalid project id: {e}"))?;

    Ok(Project {
        id,
        name: manifest.title,
        source_type: SourceType::Blank,
        source_path: None,
        project_root: Some(root.to_string_lossy().into_owned()),
        created_at: manifest.created_at.clone(),
        modified_at: manifest.created_at,
        author_pen_name: None,
        genre: Some("Speculative Mystery".into()),
        description: Some(
            "A cartographic analyst discovers that Lumen's trusted flood-prediction network is quietly erasing vulnerable neighborhoods from public maps."
                .into(),
        ),
        word_target: Some(88_000),
        reference_types: Project::default_reference_types(),
        project_type: manifest.project_type,
        target_page_count: None,
    })
}

fn write_sample_file(root: &Path, relative: &str, contents: &[u8]) -> Result<(), String> {
    let path = root.join(PathBuf::from(relative));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Could not create sample directory '{}': {e}", parent.display()))?;
    }
    fs::write(&path, contents)
        .map_err(|e| format!("Could not write sample file '{}': {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_is_the_file_authoritative_glass_meridian_project() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("The Glass Meridian");

        let project = seed_glass_meridian_sample(&root).unwrap();

        assert_eq!(project.name, "The Glass Meridian");
        assert_eq!(project.genre.as_deref(), Some("Speculative Mystery"));
        assert_eq!(project.word_target, Some(88_000));
        assert_eq!(project.project_root.as_deref(), root.to_str());
        assert!(root.join("project.yaml").is_file());
        assert!(root.join("Characters/Mara Vale.md").is_file());
        assert!(root.join("Locations/Old Quay.md").is_file());
        assert!(root.join("Manuscript/Chapter 01 - The Missing Block.md").is_file());
        assert!(root.join("Story Development/Synopsis.md").is_file());
        assert!(root.join("Assets/Characters/mara-vale.png").is_file());
        assert!(root.join("Assets/Locations/old-quay.png").is_file());
    }

    #[test]
    fn recreating_sample_replaces_previous_copy() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("The Glass Meridian");

        seed_glass_meridian_sample(&root).unwrap();
        fs::create_dir_all(root.join("Notes")).unwrap();
        fs::write(root.join("Notes/local-change.md"), "temporary").unwrap();
        seed_glass_meridian_sample(&root).unwrap();

        assert!(!root.join("Notes/local-change.md").exists());
        assert!(root.join("Characters/Nia Corren.md").is_file());
    }
}
