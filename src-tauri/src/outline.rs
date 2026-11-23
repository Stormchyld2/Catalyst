//! File-backed Outline foundation. This is not yet wired to the Write projection.
//! Stable UUIDs are assigned once and retained across renames and reordering.
use std::collections::HashSet;
use std::path::Path;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::document::service::{read_document, save_document, DocumentIdentity, CatalystDocument};

const OUTLINE_PATH: &str = "Planning/Outline.md";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outline {
    pub version: u32,
    #[serde(default)]
    pub plotlines: Vec<Plotline>,
    #[serde(default)]
    pub chapters: Vec<OutlineChapter>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plotline {
    pub id: Uuid,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutlineChapter {
    pub id: Uuid,
    pub title: String,
    #[serde(default)]
    pub scenes: Vec<OutlineScene>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutlineScene {
    pub id: Uuid,
    pub title: String,
    #[serde(default)]
    pub synopsis: String,
    #[serde(default)]
    pub plotline_ids: Vec<Uuid>,
    #[serde(default)]
    pub beats: Vec<OutlineBeat>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutlineBeat {
    pub id: Uuid,
    pub content: String,
}

/// A read-only compatibility audit before connecting Outline edits to Write.
/// No IDs are rewritten and no manuscript files are touched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityAudit {
    pub shared_chapters: Vec<Uuid>,
    pub outline_only_chapters: Vec<Uuid>,
    pub write_only_chapters: Vec<Uuid>,
    pub shared_scenes: Vec<Uuid>,
    pub outline_only_scenes: Vec<Uuid>,
    pub write_only_scenes: Vec<Uuid>,
    /// The same scene ID is assigned to a different chapter in each view.
    pub conflicting_scene_parents: Vec<Uuid>,
}

impl IdentityAudit {
    pub fn safe_to_link(&self) -> bool {
        self.outline_only_chapters.is_empty()
            && self.write_only_chapters.is_empty()
            && self.outline_only_scenes.is_empty()
            && self.write_only_scenes.is_empty()
            && self.conflicting_scene_parents.is_empty()
    }
}

/// Compare persistent Outline IDs with the current Write projection without
/// guessing matches from titles or positional indexes. Legacy projects often
/// need an explicit migration before the two can safely share mutations.
pub fn audit_write_identity(
    outline: &Outline,
    write: &crate::write_projection::WriteProjection,
) -> IdentityAudit {
    use std::collections::{HashMap, HashSet};
    let outline_chapters: HashSet<Uuid> = outline.chapters.iter().map(|c| c.id).collect();
    let write_chapters: HashSet<Uuid> = write.chapters.iter().map(|c| c.id).collect();
    let outline_scenes: HashMap<Uuid, Uuid> = outline.chapters.iter()
        .flat_map(|chapter| chapter.scenes.iter().map(move |scene| (scene.id, chapter.id)))
        .collect();
    let write_scenes: HashMap<Uuid, Uuid> = write.scenes_by_chapter.values()
        .flat_map(|scenes| scenes.iter().map(|scene| (scene.id, scene.chapter_id)))
        .collect();
    let mut audit = IdentityAudit {
        shared_chapters: outline_chapters.intersection(&write_chapters).copied().collect(),
        outline_only_chapters: outline_chapters.difference(&write_chapters).copied().collect(),
        write_only_chapters: write_chapters.difference(&outline_chapters).copied().collect(),
        shared_scenes: outline_scenes.keys().filter(|id| write_scenes.contains_key(id)).copied().collect(),
        outline_only_scenes: outline_scenes.keys().filter(|id| !write_scenes.contains_key(id)).copied().collect(),
        write_only_scenes: write_scenes.keys().filter(|id| !outline_scenes.contains_key(id)).copied().collect(),
        conflicting_scene_parents: outline_scenes.iter()
            .filter_map(|(id, parent)| write_scenes.get(id).filter(|other| *other != parent).map(|_| *id))
            .collect(),
    };
    audit.shared_chapters.sort();
    audit.outline_only_chapters.sort();
    audit.write_only_chapters.sort();
    audit.shared_scenes.sort();
    audit.outline_only_scenes.sort();
    audit.write_only_scenes.sort();
    audit.conflicting_scene_parents.sort();
    audit
}

impl Default for Outline {
    fn default() -> Self {
        Self { version: 1, plotlines: vec![], chapters: vec![] }
    }
}

impl Outline {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 { return Err("Unsupported outline version.".into()); }
        let mut ids = HashSet::new();
        for plotline in &self.plotlines {
            if !ids.insert(plotline.id) { return Err("Duplicate outline ID.".into()); }
        }
        let plotline_ids: HashSet<_> = self.plotlines.iter().map(|p| p.id).collect();
        for chapter in &self.chapters {
            if !ids.insert(chapter.id) { return Err("Duplicate outline ID.".into()); }
            for scene in &chapter.scenes {
                if !ids.insert(scene.id) { return Err("Duplicate outline ID.".into()); }
                for id in &scene.plotline_ids {
                    if !plotline_ids.contains(id) { return Err("Scene references an unknown plotline.".into()); }
                }
                for beat in &scene.beats {
                    if !ids.insert(beat.id) { return Err("Duplicate outline ID.".into()); }
                }
            }
        }
        Ok(())
    }

    pub fn move_scene(&mut self, scene_id: Uuid, destination_chapter_id: Uuid, index: usize) -> Result<(), String> {
        if !self.chapters.iter().any(|c| c.id == destination_chapter_id) {
            return Err("Destination chapter does not exist.".into());
        }
        let source = self.chapters.iter().position(|c| c.scenes.iter().any(|s| s.id == scene_id))
            .ok_or_else(|| "Scene does not exist.".to_string())?;
        let position = self.chapters[source].scenes.iter().position(|s| s.id == scene_id).unwrap();
        let scene = self.chapters[source].scenes.remove(position);
        let destination = self.chapters.iter_mut().find(|c| c.id == destination_chapter_id).unwrap();
        destination.scenes.insert(index.min(destination.scenes.len()), scene);
        Ok(())
    }
}

/// Read the current Write projection and verify that an existing Outline can
/// share its identities. This is intentionally a gate, not a title-based
/// migration: an unsafe Outline cannot drive editor navigation or mutations.
pub fn checked_write_projection(
    project_root: &Path,
    project_id: Uuid,
) -> Result<crate::write_projection::WriteProjection, String> {
    let write = crate::write_projection::build_write_projection(project_root, project_id)?;
    if let Some((outline, _)) = load(project_root)? {
        let audit = audit_write_identity(&outline, &write);
        if !audit.safe_to_link() {
            return Err(format!(
                "Outline and Write identities differ ({} outline-only chapters, {} Write-only chapters, {} outline-only scenes, {} Write-only scenes, {} conflicting scene parents). Explicit migration is required before linking.",
                audit.outline_only_chapters.len(),
                audit.write_only_chapters.len(),
                audit.outline_only_scenes.len(),
                audit.write_only_scenes.len(),
                audit.conflicting_scene_parents.len(),
            ));
        }
    }
    Ok(write)
}

/// Create the first Outline from the current Write projection without
/// changing manuscript files or inventing new chapter/scene identities.
/// This operation is explicit: opening a project never seeds an outline.
pub fn initialize_from_write(
    project_root: &Path,
    write: &crate::write_projection::WriteProjection,
) -> Result<String, String> {
    if load(project_root)?.is_some() {
        return Err("Outline already exists; refusing to replace it.".into());
    }
    let mut seen = HashSet::new();
    let mut chapters = Vec::new();
    for chapter in &write.chapters {
        if !seen.insert(chapter.id) {
            return Err("Write projection contains duplicate chapter IDs.".into());
        }
        let mut scenes = Vec::new();
        if let Some(projected_scenes) = write.scenes_by_chapter.get(&chapter.id.to_string()) {
            for scene in projected_scenes {
                if scene.chapter_id != chapter.id {
                    return Err("Write scene has a conflicting parent chapter.".into());
                }
                if !seen.insert(scene.id) {
                    return Err("Write projection contains duplicate scene IDs.".into());
                }
                let mut beats = Vec::new();
                if let Some(projected_beats) = write.beats_by_scene.get(&scene.id.to_string()) {
                    for beat in projected_beats {
                        if beat.scene_id != scene.id {
                            return Err("Write beat has a conflicting parent scene.".into());
                        }
                        if !seen.insert(beat.id) {
                            return Err("Write projection contains duplicate beat IDs.".into());
                        }
                        beats.push(OutlineBeat { id: beat.id, content: beat.content.clone() });
                    }
                }
                scenes.push(OutlineScene {
                    id: scene.id,
                    title: scene.title.clone(),
                    synopsis: scene.synopsis.clone().unwrap_or_default(),
                    plotline_ids: vec![],
                    beats,
                });
            }
        }
        chapters.push(OutlineChapter { id: chapter.id, title: chapter.title.clone(), scenes });
    }
    save(project_root, &Outline { chapters, ..Default::default() }, None)
}

/// Returns None when the outline has not yet been created. Reading never creates files.
pub fn load(project_root: &Path) -> Result<Option<(Outline, String)>, String> {
    if !project_root.join(OUTLINE_PATH).exists() { return Ok(None); }
    let document = read_document(project_root, Path::new(OUTLINE_PATH))?;
    if document.identity.document_type != "outline" {
        return Err("Planning/Outline.md is not an outline document.".into());
    }
    let yaml = document.metadata.get("outline")
        .ok_or_else(|| "Outline metadata is missing.".to_string())?;
    let outline: Outline = serde_yaml::from_value(yaml.clone())
        .map_err(|e| format!("Invalid outline data: {e}"))?;
    outline.validate()?;
    Ok(Some((outline, document.source_hash)))
}

/// Uses the document service's staged write and optimistic conflict checking.
pub fn save(project_root: &Path, outline: &Outline, expected_hash: Option<&str>) -> Result<String, String> {
    outline.validate()?;
    let existing = load(project_root)?;
    if existing.is_some() && expected_hash.is_none() {
        return Err("An existing outline requires its source hash to prevent overwrites.".into());
    }
    if existing.is_none() && expected_hash.is_some() {
        return Err("Outline was removed or has not been created.".into());
    }
    let mut document = if existing.is_some() {
        read_document(project_root, Path::new(OUTLINE_PATH))?
    } else {
        CatalystDocument {
            identity: DocumentIdentity { document_type: "outline".into(), id: format!("out-{}", Uuid::new_v4()) },
            title: Some("Master Outline".into()),
            metadata: Default::default(),
            body: String::new(),
            relative_path: OUTLINE_PATH.into(),
            source_hash: String::new(),
        }
    };
    document.metadata.insert("outline".into(), serde_yaml::to_value(outline)
        .map_err(|e| format!("Could not serialize outline: {e}"))?);
    Ok(save_document(project_root, &document, expected_hash)?.source_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_projection_rejects_unmatched_outline_without_changing_files() {
        let root = std::env::temp_dir().join(format!("lithomark-outline-gate-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let project_id = Uuid::new_v4();
        let empty = checked_write_projection(&root, project_id).unwrap();
        assert!(empty.chapters.is_empty());

        let outline = Outline {
            chapters: vec![OutlineChapter {
                id: Uuid::new_v4(), title: "Unlinked chapter".into(), scenes: vec![],
            }],
            ..Default::default()
        };
        save(&root, &outline, None).unwrap();
        let before = std::fs::read(root.join(OUTLINE_PATH)).unwrap();
        let error = checked_write_projection(&root, project_id).unwrap_err();
        assert!(error.contains("Explicit migration is required"));
        assert_eq!(before, std::fs::read(root.join(OUTLINE_PATH)).unwrap());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn initialization_from_write_preserves_ids_and_rejects_overwrite() {
        let root = std::env::temp_dir().join(format!("lithomark-outline-seed-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let project_id = Uuid::new_v4();
        let chapter_id = Uuid::new_v4();
        let write = crate::write_projection::WriteProjection {
            chapters: vec![crate::models::Chapter {
                id: chapter_id, project_id, title: "Opening".into(),
                position: 0, source_id: None, archived: false, locked: false,
                is_part: false, synopsis: None,
                planning_status: crate::models::PlanningStatus::Flexible,
            }],
            scenes_by_chapter: Default::default(),
            beats_by_scene: Default::default(),
        };
        initialize_from_write(&root, &write).unwrap();
        let (outline, _) = load(&root).unwrap().unwrap();
        assert_eq!(outline.chapters[0].id, chapter_id);
        assert!(audit_write_identity(&outline, &write).safe_to_link());
        assert!(initialize_from_write(&root, &write).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn identity_audit_refuses_to_equate_matching_titles_with_different_ids() {
        let project_id = Uuid::new_v4();
        let outline_chapter_id = Uuid::new_v4();
        let write_chapter_id = Uuid::new_v4();
        let outline_scene_id = Uuid::new_v4();
        let write_scene_id = Uuid::new_v4();
        let outline = Outline {
            chapters: vec![OutlineChapter {
                id: outline_chapter_id, title: "Same title".into(),
                scenes: vec![OutlineScene {
                    id: outline_scene_id, title: "Same scene".into(),
                    synopsis: String::new(), plotline_ids: vec![], beats: vec![]
                }]
            }],
            ..Default::default()
        };
        let chapter = crate::models::Chapter {
            id: write_chapter_id, project_id, title: "Same title".into(),
            position: 0, source_id: None, archived: false, locked: false,
            is_part: false, synopsis: None,
            planning_status: crate::models::PlanningStatus::Flexible,
        };
        let write = crate::write_projection::WriteProjection {
            chapters: vec![chapter],
            scenes_by_chapter: Default::default(),
            beats_by_scene: Default::default(),
        };
        let audit = audit_write_identity(&outline, &write);
        assert!(!audit.safe_to_link());
        assert_eq!(audit.outline_only_chapters, vec![outline_chapter_id]);
        assert_eq!(audit.write_only_chapters, vec![write_chapter_id]);
        assert_eq!(audit.outline_only_scenes, vec![outline_scene_id]);
        assert!(!audit.write_only_scenes.contains(&write_scene_id));
    }

    #[test]
    fn scene_reorder_preserves_identity_and_beats() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let scene_id = Uuid::new_v4();
        let beat_id = Uuid::new_v4();
        let mut outline = Outline {
            chapters: vec![
                OutlineChapter { id: a, title: "One".into(), scenes: vec![OutlineScene {
                    id: scene_id, title: "Opening".into(), synopsis: String::new(),
                    plotline_ids: vec![], beats: vec![OutlineBeat { id: beat_id, content: "Clue".into() }]
                }] },
                OutlineChapter { id: b, title: "Two".into(), scenes: vec![] }
            ],
            ..Default::default()
        };
        outline.move_scene(scene_id, b, 0).unwrap();
        assert!(outline.chapters[0].scenes.is_empty());
        assert_eq!(outline.chapters[1].scenes[0].id, scene_id);
        assert_eq!(outline.chapters[1].scenes[0].beats[0].id, beat_id);
    }

    #[test]
    fn file_round_trip_preserves_ids_after_rename_and_move() {
        let root = std::env::temp_dir().join(format!("lithomark-outline-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let chapter_a = Uuid::new_v4();
        let chapter_b = Uuid::new_v4();
        let scene_id = Uuid::new_v4();
        let beat_id = Uuid::new_v4();
        let mut outline = Outline {
            chapters: vec![
                OutlineChapter { id: chapter_a, title: "Chapter One".into(), scenes: vec![
                    OutlineScene { id: scene_id, title: "First scene".into(), synopsis: "Notes".into(),
                        plotline_ids: vec![], beats: vec![OutlineBeat { id: beat_id, content: "Reveal".into() }] }
                ] },
                OutlineChapter { id: chapter_b, title: "Chapter Two".into(), scenes: vec![] },
            ],
            ..Default::default()
        };
        let hash = save(&root, &outline, None).unwrap();
        let (loaded, loaded_hash) = load(&root).unwrap().unwrap();
        assert_eq!(loaded, outline);
        assert_eq!(hash, loaded_hash);

        outline.chapters[0].title = "Renamed chapter".into();
        outline.chapters[0].scenes[0].title = "Renamed scene".into();
        outline.move_scene(scene_id, chapter_b, 0).unwrap();
        let new_hash = save(&root, &outline, Some(&hash)).unwrap();
        assert_ne!(hash, new_hash);
        let (reloaded, _) = load(&root).unwrap().unwrap();
        assert_eq!(reloaded.chapters[1].scenes[0].id, scene_id);
        assert_eq!(reloaded.chapters[1].scenes[0].beats[0].id, beat_id);
        assert_eq!(reloaded.chapters[0].title, "Renamed chapter");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn stale_save_is_rejected_without_overwriting() {
        let root = std::env::temp_dir().join(format!("lithomark-outline-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let first = Outline::default();
        let original_hash = save(&root, &first, None).unwrap();
        let mut changed = first.clone();
        changed.chapters.push(OutlineChapter { id: Uuid::new_v4(), title: "Added".into(), scenes: vec![] });
        let updated_hash = save(&root, &changed, Some(&original_hash)).unwrap();
        assert!(save(&root, &first, Some(&original_hash)).is_err());
        assert!(save(&root, &first, None).is_err());
        let (disk, hash) = load(&root).unwrap().unwrap();
        assert_eq!(disk, changed);
        assert_eq!(hash, updated_hash);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let id = Uuid::new_v4();
        let outline = Outline {
            plotlines: vec![Plotline { id, title: "A".into() }],
            chapters: vec![OutlineChapter { id, title: "B".into(), scenes: vec![] }],
            ..Default::default()
        };
        assert!(outline.validate().is_err());
    }

    #[test]
    fn missing_destination_does_not_remove_scene() {
        let chapter_id = Uuid::new_v4();
        let scene_id = Uuid::new_v4();
        let mut outline = Outline {
            chapters: vec![OutlineChapter { id: chapter_id, title: "One".into(), scenes: vec![OutlineScene {
                id: scene_id, title: "Scene".into(), synopsis: String::new(), plotline_ids: vec![], beats: vec![]
            }] }],
            ..Default::default()
        };
        assert!(outline.move_scene(scene_id, Uuid::new_v4(), 0).is_err());
        assert_eq!(outline.chapters[0].scenes[0].id, scene_id);
    }
}
