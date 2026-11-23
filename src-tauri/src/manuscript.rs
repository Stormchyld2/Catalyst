//! File-authoritative manuscript structure.
//!
//! Chapters and scenes are normal Catalyst documents. Structure is expressed
//! with stable IDs in YAML metadata, never inferred from filenames or SQLite.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::document::service::{read_document, save_document, CatalystDocument};

const CHAPTER_TYPE: &str = "chapter";
const SCENE_TYPE: &str = "scene";
const ORDER_KEY: &str = "order";
const PARENT_CHAPTER_KEY: &str = "parent_chapter";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChapterStructure {
    pub chapter_id: String,
    pub scene_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureIssueKind {
    DuplicateStableId,
    MissingScene,
    MissingChapter,
    ParentMismatch,
    SceneListedMultipleTimes,
    OrphanScene,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructureIssue {
    pub kind: StructureIssueKind,
    pub stable_id: String,
    pub message: String,
    pub paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructureIntegrityReport {
    pub chapters: usize,
    pub scenes: usize,
    pub issues: Vec<StructureIssue>,
}

impl StructureIntegrityReport {
    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }
}

pub fn validate_manuscript_structure(project_root: &Path) -> Result<StructureIntegrityReport, String> {
    if !project_root.is_absolute() {
        return Err("Project root must be absolute.".into());
    }

    let manuscript_root = project_root.join("Manuscript");
    if !manuscript_root.exists() {
        return Ok(StructureIntegrityReport::default());
    }

    let mut documents = Vec::new();
    collect_markdown_paths(project_root, &manuscript_root, &mut documents)?;
    let mut by_id: BTreeMap<String, Vec<CatalystDocument>> = BTreeMap::new();
    for relative_path in documents {
        let document = read_document(project_root, &relative_path)?;
        if matches!(document.identity.document_type.as_str(), CHAPTER_TYPE | SCENE_TYPE) {
            by_id.entry(document.identity.id.clone()).or_default().push(document);
        }
    }

    let mut report = StructureIntegrityReport::default();
    for (id, docs) in &by_id {
        if docs.len() > 1 {
            report.issues.push(StructureIssue {
                kind: StructureIssueKind::DuplicateStableId,
                stable_id: id.clone(),
                message: format!("Stable ID '{id}' appears in multiple authoritative manuscript files."),
                paths: docs.iter().map(|doc| doc.relative_path.clone()).collect(),
            });
        }
        for doc in docs {
            match doc.identity.document_type.as_str() {
                CHAPTER_TYPE => report.chapters += 1,
                SCENE_TYPE => report.scenes += 1,
                _ => {}
            }
        }
    }

    let unique: BTreeMap<&str, &CatalystDocument> = by_id
        .iter()
        .filter_map(|(id, docs)| (docs.len() == 1).then_some((id.as_str(), &docs[0])))
        .collect();
    let mut listed_by: BTreeMap<String, Vec<&CatalystDocument>> = BTreeMap::new();

    for chapter in unique.values().filter(|doc| doc.identity.document_type == CHAPTER_TYPE) {
        let structure = chapter_structure(chapter)?;
        for scene_id in structure.scene_ids {
            match unique.get(scene_id.as_str()) {
                Some(scene) if scene.identity.document_type == SCENE_TYPE => {
                    listed_by.entry(scene_id.clone()).or_default().push(chapter);
                    if scene_parent(scene)?.as_deref() != Some(chapter.identity.id.as_str()) {
                        report.issues.push(StructureIssue {
                            kind: StructureIssueKind::ParentMismatch,
                            stable_id: scene_id,
                            message: format!(
                                "Scene parent does not match chapter '{}' that lists it.",
                                chapter.identity.id
                            ),
                            paths: vec![chapter.relative_path.clone(), scene.relative_path.clone()],
                        });
                    }
                }
                _ => report.issues.push(StructureIssue {
                    kind: StructureIssueKind::MissingScene,
                    stable_id: scene_id.clone(),
                    message: format!("Chapter '{}' references missing scene '{scene_id}'.", chapter.identity.id),
                    paths: vec![chapter.relative_path.clone()],
                }),
            }
        }
    }

    for (scene_id, chapters) in &listed_by {
        if chapters.len() > 1 {
            report.issues.push(StructureIssue {
                kind: StructureIssueKind::SceneListedMultipleTimes,
                stable_id: scene_id.clone(),
                message: format!("Scene '{scene_id}' is listed by more than one chapter."),
                paths: chapters.iter().map(|chapter| chapter.relative_path.clone()).collect(),
            });
        }
    }

    for scene in unique.values().filter(|doc| doc.identity.document_type == SCENE_TYPE) {
        match scene_parent(scene)? {
            Some(parent_id) => match unique.get(parent_id.as_str()) {
                Some(parent) if parent.identity.document_type == CHAPTER_TYPE => {
                    let structure = chapter_structure(parent)?;
                    if !structure.scene_ids.iter().any(|id| id == &scene.identity.id) {
                        report.issues.push(StructureIssue {
                            kind: StructureIssueKind::ParentMismatch,
                            stable_id: scene.identity.id.clone(),
                            message: format!(
                                "Scene points to chapter '{parent_id}', but that chapter does not list the scene."
                            ),
                            paths: vec![scene.relative_path.clone(), parent.relative_path.clone()],
                        });
                    }
                }
                _ => report.issues.push(StructureIssue {
                    kind: StructureIssueKind::MissingChapter,
                    stable_id: scene.identity.id.clone(),
                    message: format!("Scene points to missing chapter '{parent_id}'."),
                    paths: vec![scene.relative_path.clone()],
                }),
            },
            None if !listed_by.contains_key(&scene.identity.id) => report.issues.push(StructureIssue {
                kind: StructureIssueKind::OrphanScene,
                stable_id: scene.identity.id.clone(),
                message: "Scene has no parent chapter and is not listed by any chapter.".into(),
                paths: vec![scene.relative_path.clone()],
            }),
            None => {}
        }
    }

    Ok(report)
}

fn collect_markdown_paths(
    project_root: &Path,
    directory: &Path,
    output: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for entry in fs::read_dir(directory)
        .map_err(|e| format!("Could not scan manuscript directory '{}': {e}", directory.display()))?
    {
        let entry = entry.map_err(|e| format!("Could not read manuscript directory entry: {e}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_paths(project_root, &path, output)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
            output.push(
                path.strip_prefix(project_root)
                    .map_err(|_| "Manuscript path escaped the project root.".to_string())?
                    .to_path_buf(),
            );
        }
    }
    Ok(())
}

pub fn chapter_structure(document: &CatalystDocument) -> Result<ChapterStructure, String> {
    require_type(document, CHAPTER_TYPE)?;
    let scene_ids = string_list(document.metadata.get(ORDER_KEY), ORDER_KEY)?;
    for id in &scene_ids {
        validate_prefixed_uuid(id, "scn-")?;
    }
    ensure_unique(&scene_ids, "chapter scene order")?;

    Ok(ChapterStructure {
        chapter_id: document.identity.id.clone(),
        scene_ids,
    })
}

pub fn scene_parent(document: &CatalystDocument) -> Result<Option<String>, String> {
    require_type(document, SCENE_TYPE)?;
    let Some(value) = document.metadata.get(PARENT_CHAPTER_KEY) else {
        return Ok(None);
    };
    let id = value
        .as_str()
        .ok_or_else(|| format!("'{PARENT_CHAPTER_KEY}' must be a stable chapter ID string."))?;
    validate_prefixed_uuid(id, "ch-")?;
    Ok(Some(id.to_string()))
}

pub fn set_chapter_order(
    project_root: &Path,
    chapter: &mut CatalystDocument,
    scene_ids: Vec<String>,
) -> Result<(), String> {
    require_type(chapter, CHAPTER_TYPE)?;
    for id in &scene_ids {
        validate_prefixed_uuid(id, "scn-")?;
    }
    ensure_unique(&scene_ids, "chapter scene order")?;

    chapter.metadata.insert(
        ORDER_KEY.into(),
        Value::Sequence(scene_ids.into_iter().map(Value::String).collect()),
    );
    persist(project_root, chapter)
}

pub fn set_scene_parent(
    project_root: &Path,
    scene: &mut CatalystDocument,
    chapter_id: Option<&str>,
) -> Result<(), String> {
    require_type(scene, SCENE_TYPE)?;
    match chapter_id {
        Some(id) => {
            validate_prefixed_uuid(id, "ch-")?;
            scene
                .metadata
                .insert(PARENT_CHAPTER_KEY.into(), Value::String(id.to_string()));
        }
        None => {
            scene.metadata.remove(PARENT_CHAPTER_KEY);
        }
    }
    persist(project_root, scene)
}

/// Move a scene between chapter structures while preserving its stable identity.
///
/// All three documents are validated before the first write. Each write still
/// uses the Document Service's optimistic conflict check. A later transactional
/// multi-document mutation layer may strengthen crash recovery for this
/// operation; authoritative structure remains recoverable from the files.
pub fn move_scene(
    project_root: &Path,
    scene_path: &Path,
    from_chapter_path: &Path,
    to_chapter_path: &Path,
    target_index: usize,
) -> Result<(), String> {
    if from_chapter_path == to_chapter_path {
        return reorder_scene(project_root, from_chapter_path, scene_path, target_index);
    }

    let mut scene = read_document(project_root, scene_path)?;
    let mut from = read_document(project_root, from_chapter_path)?;
    let mut to = read_document(project_root, to_chapter_path)?;

    require_type(&scene, SCENE_TYPE)?;
    let mut from_structure = chapter_structure(&from)?;
    let mut to_structure = chapter_structure(&to)?;

    let current_parent = scene_parent(&scene)?;
    if current_parent.as_deref() != Some(from.identity.id.as_str()) {
        return Err("Scene parent does not match the source chapter.".into());
    }

    let scene_id = scene.identity.id.clone();
    let old_len = from_structure.scene_ids.len();
    from_structure.scene_ids.retain(|id| id != &scene_id);
    if from_structure.scene_ids.len() == old_len {
        return Err("Source chapter does not contain the scene in its explicit order.".into());
    }
    if to_structure.scene_ids.iter().any(|id| id == &scene_id) {
        return Err("Destination chapter already contains the scene ID.".into());
    }
    if target_index > to_structure.scene_ids.len() {
        return Err("Target scene index is outside the destination chapter order.".into());
    }
    to_structure.scene_ids.insert(target_index, scene_id);

    from.metadata.insert(
        ORDER_KEY.into(),
        Value::Sequence(from_structure.scene_ids.into_iter().map(Value::String).collect()),
    );
    to.metadata.insert(
        ORDER_KEY.into(),
        Value::Sequence(to_structure.scene_ids.into_iter().map(Value::String).collect()),
    );
    scene.metadata.insert(
        PARENT_CHAPTER_KEY.into(),
        Value::String(to.identity.id.clone()),
    );

    persist(project_root, &mut from)?;
    persist(project_root, &mut to)?;
    persist(project_root, &mut scene)?;
    Ok(())
}

pub fn reorder_scene(
    project_root: &Path,
    chapter_path: &Path,
    scene_path: &Path,
    target_index: usize,
) -> Result<(), String> {
    let scene = read_document(project_root, scene_path)?;
    let mut chapter = read_document(project_root, chapter_path)?;
    require_type(&scene, SCENE_TYPE)?;
    let mut structure = chapter_structure(&chapter)?;

    if scene_parent(&scene)?.as_deref() != Some(chapter.identity.id.as_str()) {
        return Err("Scene parent does not match the chapter being reordered.".into());
    }

    let scene_id = scene.identity.id;
    let old_len = structure.scene_ids.len();
    structure.scene_ids.retain(|id| id != &scene_id);
    if structure.scene_ids.len() == old_len {
        return Err("Chapter does not contain the scene in its explicit order.".into());
    }
    if target_index > structure.scene_ids.len() {
        return Err("Target scene index is outside the chapter order.".into());
    }
    structure.scene_ids.insert(target_index, scene_id);
    set_chapter_order(project_root, &mut chapter, structure.scene_ids)
}

fn persist(project_root: &Path, document: &mut CatalystDocument) -> Result<(), String> {
    let result = save_document(project_root, document, Some(&document.source_hash))?;
    document.source_hash = result.source_hash;
    Ok(())
}

fn require_type(document: &CatalystDocument, expected: &str) -> Result<(), String> {
    if document.identity.document_type != expected {
        return Err(format!(
            "Expected a {expected} document, found '{}'.",
            document.identity.document_type
        ));
    }
    Ok(())
}

fn string_list(value: Option<&Value>, key: &str) -> Result<Vec<String>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let sequence = value
        .as_sequence()
        .ok_or_else(|| format!("'{key}' must be a sequence of stable IDs."))?;
    sequence
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("'{key}' entries must be stable ID strings."))
        })
        .collect()
}

fn ensure_unique(values: &[String], label: &str) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(format!("Duplicate stable ID '{value}' in {label}."));
        }
    }
    Ok(())
}

fn validate_prefixed_uuid(id: &str, prefix: &str) -> Result<(), String> {
    let uuid = id
        .strip_prefix(prefix)
        .ok_or_else(|| format!("Stable ID '{id}' must use prefix '{prefix}'."))?;
    let parsed = uuid::Uuid::parse_str(uuid)
        .map_err(|_| format!("Stable ID '{id}' must contain a valid UUID."))?;
    if id != format!("{prefix}{}", parsed.hyphenated()) {
        return Err(format!("Stable ID '{id}' must use canonical lowercase form."));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::service::{create_document, read_document};

    fn root() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn setup() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
        let temp = root();
        let chapter_a = PathBuf::from("Manuscript/Chapter A.md");
        let chapter_b = PathBuf::from("Manuscript/Chapter B.md");
        let scene = PathBuf::from("Manuscript/Scene.md");
        create_document(temp.path(), &chapter_a, "chapter", Some("A"), "").unwrap();
        create_document(temp.path(), &chapter_b, "chapter", Some("B"), "").unwrap();
        create_document(temp.path(), &scene, "scene", Some("Scene"), "Text").unwrap();
        (temp, chapter_a, chapter_b, scene)
    }

    #[test]
    fn chapter_order_is_explicit_and_round_trips() {
        let (temp, chapter_path, _, scene_path) = setup();
        let scene = read_document(temp.path(), &scene_path).unwrap();
        let mut chapter = read_document(temp.path(), &chapter_path).unwrap();
        set_chapter_order(temp.path(), &mut chapter, vec![scene.identity.id.clone()]).unwrap();

        let loaded = read_document(temp.path(), &chapter_path).unwrap();
        assert_eq!(chapter_structure(&loaded).unwrap().scene_ids, vec![scene.identity.id]);
    }

    #[test]
    fn scene_parent_is_stable_id_based() {
        let (temp, chapter_path, _, scene_path) = setup();
        let chapter = read_document(temp.path(), &chapter_path).unwrap();
        let mut scene = read_document(temp.path(), &scene_path).unwrap();
        set_scene_parent(temp.path(), &mut scene, Some(&chapter.identity.id)).unwrap();

        let loaded = read_document(temp.path(), &scene_path).unwrap();
        assert_eq!(scene_parent(&loaded).unwrap().as_deref(), Some(chapter.identity.id.as_str()));
    }

    #[test]
    fn duplicate_scene_ids_in_order_are_rejected() {
        let (temp, chapter_path, _, scene_path) = setup();
        let scene = read_document(temp.path(), &scene_path).unwrap();
        let mut chapter = read_document(temp.path(), &chapter_path).unwrap();
        let error = set_chapter_order(
            temp.path(),
            &mut chapter,
            vec![scene.identity.id.clone(), scene.identity.id],
        )
        .unwrap_err();
        assert!(error.contains("Duplicate"));
    }

    #[test]
    fn moving_scene_preserves_identity_and_updates_both_chapters() {
        let (temp, chapter_a_path, chapter_b_path, scene_path) = setup();
        let mut a = read_document(temp.path(), &chapter_a_path).unwrap();
        let b = read_document(temp.path(), &chapter_b_path).unwrap();
        let mut scene = read_document(temp.path(), &scene_path).unwrap();
        let scene_id = scene.identity.id.clone();

        set_chapter_order(temp.path(), &mut a, vec![scene_id.clone()]).unwrap();
        set_scene_parent(temp.path(), &mut scene, Some(&a.identity.id)).unwrap();

        move_scene(temp.path(), &scene_path, &chapter_a_path, &chapter_b_path, 0).unwrap();

        let a = read_document(temp.path(), &chapter_a_path).unwrap();
        let b_after = read_document(temp.path(), &chapter_b_path).unwrap();
        let scene_after = read_document(temp.path(), &scene_path).unwrap();
        assert!(chapter_structure(&a).unwrap().scene_ids.is_empty());
        assert_eq!(chapter_structure(&b_after).unwrap().scene_ids, vec![scene_id.clone()]);
        assert_eq!(scene_after.identity.id, scene_id);
        assert_eq!(scene_parent(&scene_after).unwrap().as_deref(), Some(b.identity.id.as_str()));
    }

    #[test]
    fn reorder_preserves_scene_identity() {
        let temp = root();
        let chapter_path = PathBuf::from("Manuscript/Chapter.md");
        let first_path = PathBuf::from("Manuscript/First.md");
        let second_path = PathBuf::from("Manuscript/Second.md");
        let mut chapter = create_document(temp.path(), &chapter_path, "chapter", Some("Chapter"), "").unwrap();
        let mut first = create_document(temp.path(), &first_path, "scene", Some("First"), "").unwrap();
        let mut second = create_document(temp.path(), &second_path, "scene", Some("Second"), "").unwrap();

        set_chapter_order(
            temp.path(),
            &mut chapter,
            vec![first.identity.id.clone(), second.identity.id.clone()],
        )
        .unwrap();
        set_scene_parent(temp.path(), &mut first, Some(&chapter.identity.id)).unwrap();
        set_scene_parent(temp.path(), &mut second, Some(&chapter.identity.id)).unwrap();

        reorder_scene(temp.path(), &chapter_path, &second_path, 0).unwrap();
        let loaded = read_document(temp.path(), &chapter_path).unwrap();
        assert_eq!(
            chapter_structure(&loaded).unwrap().scene_ids,
            vec![second.identity.id.clone(), first.identity.id.clone()]
        );
        assert_eq!(read_document(temp.path(), &second_path).unwrap().identity.id, second.identity.id);
    }

    #[test]
    fn mismatched_parent_is_reported_not_silently_repaired() {
        let (temp, chapter_a_path, chapter_b_path, scene_path) = setup();
        let a = read_document(temp.path(), &chapter_a_path).unwrap();
        let b = read_document(temp.path(), &chapter_b_path).unwrap();
        let mut scene = read_document(temp.path(), &scene_path).unwrap();
        set_scene_parent(temp.path(), &mut scene, Some(&b.identity.id)).unwrap();

        let error = reorder_scene(temp.path(), &chapter_a_path, &scene_path, 0).unwrap_err();
        assert!(error.contains("parent"));
        assert_eq!(scene_parent(&read_document(temp.path(), &scene_path).unwrap()).unwrap().as_deref(), Some(b.identity.id.as_str()));
        assert_ne!(a.identity.id, b.identity.id);
    }

    #[test]
    fn integrity_report_is_clean_for_consistent_structure() {
        let (temp, chapter_path, _, scene_path) = setup();
        let mut chapter = read_document(temp.path(), &chapter_path).unwrap();
        let mut scene = read_document(temp.path(), &scene_path).unwrap();
        set_chapter_order(temp.path(), &mut chapter, vec![scene.identity.id.clone()]).unwrap();
        set_scene_parent(temp.path(), &mut scene, Some(&chapter.identity.id)).unwrap();

        // The unused second chapter is valid; only unparented scenes are orphans.
        let report = validate_manuscript_structure(temp.path()).unwrap();
        assert!(report.is_clean(), "{:?}", report.issues);
    }

    #[test]
    fn integrity_report_finds_missing_scene_reference() {
        let temp = root();
        let chapter_path = PathBuf::from("Manuscript/Chapter.md");
        let mut chapter = create_document(temp.path(), &chapter_path, "chapter", Some("Chapter"), "").unwrap();
        let missing = format!("scn-{}", uuid::Uuid::new_v4());
        set_chapter_order(temp.path(), &mut chapter, vec![missing.clone()]).unwrap();

        let report = validate_manuscript_structure(temp.path()).unwrap();
        assert!(report.issues.iter().any(|issue| {
            issue.kind == StructureIssueKind::MissingScene && issue.stable_id == missing
        }));
    }

    #[test]
    fn integrity_report_finds_orphan_scene() {
        let temp = root();
        let scene = create_document(
            temp.path(),
            Path::new("Manuscript/Orphan.md"),
            "scene",
            Some("Orphan"),
            "",
        )
        .unwrap();

        let report = validate_manuscript_structure(temp.path()).unwrap();
        assert!(report.issues.iter().any(|issue| {
            issue.kind == StructureIssueKind::OrphanScene && issue.stable_id == scene.identity.id
        }));
    }

    #[test]
    fn integrity_report_finds_scene_listed_by_multiple_chapters() {
        let (temp, chapter_a_path, chapter_b_path, scene_path) = setup();
        let mut a = read_document(temp.path(), &chapter_a_path).unwrap();
        let mut b = read_document(temp.path(), &chapter_b_path).unwrap();
        let mut scene = read_document(temp.path(), &scene_path).unwrap();
        set_chapter_order(temp.path(), &mut a, vec![scene.identity.id.clone()]).unwrap();
        set_chapter_order(temp.path(), &mut b, vec![scene.identity.id.clone()]).unwrap();
        set_scene_parent(temp.path(), &mut scene, Some(&a.identity.id)).unwrap();

        let report = validate_manuscript_structure(temp.path()).unwrap();
        assert!(report.issues.iter().any(|issue| {
            issue.kind == StructureIssueKind::SceneListedMultipleTimes
                && issue.stable_id == scene.identity.id
        }));
    }

}
