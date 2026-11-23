//! File-authoritative Write workspace projection.
//!
//! This module reads ordinary Catalyst project files and derives the chapter /
//! scene / beat structure consumed by the existing Write UI. It owns no content
//! and does not write to the transitional SQLite cache.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::document::service::{read_document, save_document, DocumentIdentity, CatalystDocument};
use crate::entity_templates::EntityTemplate;
use crate::models::{
    Beat, Chapter, EditorMode, PlanningStatus, Scene, SceneStatus, SceneType,
};

const MANUSCRIPT_ROOT: &str = "Manuscript";
const WRITE_METADATA_FILE: &str = "Manuscript/Write Metadata.yaml";
const MASTER_OUTLINE_ROOT: &str = "Story Development/Planning & Outline/Master Outline";
const PLANNING_ROOTS: &[&str] = &[
    "Story Development/Planning & Outline/Master Outline",
    "Story Development/Planning & Outline/Beat Sheets",
    "Planning",
];

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct SceneMetadataOverride {
    scene_type: Option<SceneType>,
    scene_status: Option<SceneStatus>,
    planning_status: Option<PlanningStatus>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct WriteMetadata {
    #[serde(default)]
    scenes: HashMap<String, SceneMetadataOverride>,
    #[serde(default)]
    chapter_order: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct WriteProjection {
    pub chapters: Vec<Chapter>,
    pub scenes_by_chapter: HashMap<String, Vec<Scene>>,
    pub beats_by_scene: HashMap<String, Vec<Beat>>,
}

#[derive(Clone, Debug)]
struct PlanScene {
    title: String,
    notes: String,
    beats: Vec<String>,
}

#[derive(Clone, Debug)]
struct PlanChapter {
    title: String,
    scenes: Vec<PlanScene>,
}

#[derive(Clone, Debug)]
struct ManuscriptChapter {
    document: CatalystDocument,
    position: i32,
}

#[derive(Clone, Debug)]
struct ManuscriptScene {
    document: CatalystDocument,
}

pub fn build_write_projection(project_root: &Path, project_id: Uuid) -> Result<WriteProjection, String> {
    if !project_root.is_absolute() {
        return Err("Project root must be absolute.".into());
    }

    let manuscript = collect_manuscript_chapters(project_root)?;
    let mut manuscript_scenes = collect_manuscript_scenes(project_root)?;
    let planning = collect_planning_structure(project_root)?;
    let write_metadata = read_write_metadata(project_root)?;

    let mut plan_by_key = HashMap::<String, PlanChapter>::new();
    let mut plan_order = Vec::<String>::new();
    for chapter in planning {
        let key = normalize_structure_title(&chapter.title);
        if !plan_by_key.contains_key(&key) {
            plan_order.push(key.clone());
        }
        plan_by_key
            .entry(key)
            .and_modify(|existing| merge_plan_chapter(existing, &chapter))
            .or_insert(chapter);
    }

    let mut manuscript_by_key = HashMap::<String, ManuscriptChapter>::new();
    for chapter in manuscript {
        let title = chapter
            .document
            .title
            .clone()
            .unwrap_or_else(|| chapter.document.relative_path.file_stem().unwrap_or_default().to_string_lossy().into_owned());
        let key = normalize_structure_title(&title);
        if let Some(previous) = manuscript_by_key.insert(key, chapter.clone()) {
            return Err(format!(
                "Two manuscript chapters share the same normalized title: {} and {}. Refusing to silently discard a chapter.",
                previous.document.relative_path.display(),
                chapter.document.relative_path.display()
            ));
        }
    }

    let mut ordered_keys = manuscript_by_key
        .iter()
        .map(|(key, chapter)| (chapter.position, key.clone()))
        .collect::<Vec<_>>();
    ordered_keys.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut keys = ordered_keys.into_iter().map(|(_, key)| key).collect::<Vec<_>>();
    for key in plan_order {
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    // Persist chapter ordering independently of filenames, prose, and stable identities.
    if !write_metadata.chapter_order.is_empty() {
        let rank = write_metadata.chapter_order.iter().enumerate()
            .map(|(index, id)| (id.as_str(), index))
            .collect::<HashMap<_, _>>();
        keys.sort_by_key(|key| {
            let id = manuscript_by_key.get(key)
                .and_then(|chapter| uuid_from_stable_id(&chapter.document.identity.id))
                .unwrap_or_else(|| deterministic_uuid(&project_id.to_string(), &format!("chapter:{key}")));
            rank.get(id.to_string().as_str()).copied().unwrap_or(usize::MAX)
        });
    }

    let mut chapters = Vec::new();
    let mut scenes_by_chapter = HashMap::new();
    let mut beats_by_scene = HashMap::new();

    for (chapter_index, key) in keys.into_iter().enumerate() {
        let manuscript = manuscript_by_key.remove(&key);
        let plan = plan_by_key.remove(&key);

        let title = manuscript
            .as_ref()
            .and_then(|value| value.document.title.clone())
            .or_else(|| plan.as_ref().map(|value| value.title.clone()))
            .unwrap_or_else(|| format!("Chapter {}", chapter_index + 1));

        let chapter_id = manuscript
            .as_ref()
            .and_then(|value| uuid_from_stable_id(&value.document.identity.id))
            .unwrap_or_else(|| deterministic_uuid(&project_id.to_string(), &format!("chapter:{key}")));

        let synopsis = manuscript
            .as_ref()
            .and_then(|value| value.document.metadata.get("synopsis"))
            .and_then(|value| value.as_str())
            .map(str::to_string)
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                plan.as_ref()
                    .and_then(|value| value.scenes.first())
                    .map(|scene| scene.notes.clone())
                    .filter(|value| !value.trim().is_empty())
            });

        chapters.push(Chapter {
            id: chapter_id,
            project_id,
            title: title.clone(),
            position: chapter_index as i32,
            source_id: Some(match manuscript.as_ref() {
                Some(value) if value.document.metadata.get("scene_authority")
                    .and_then(|value| value.as_str()) == Some("individual_files") =>
                    format!("lithomark:scene-files:{}", value.document.relative_path.display()),
                Some(value) => format!(
                    "lithomark:file:{}",
                    value.document.relative_path.to_string_lossy().replace('\\', "/")
                ),
                None => format!("lithomark:planned:{key}"),
            }),
            archived: false,
            locked: false,
            is_part: false,
            synopsis,
            planning_status: if manuscript.is_some() {
                PlanningStatus::Fixed
            } else {
                PlanningStatus::Flexible
            },
        });

        let plan_scenes = plan.map(|value| value.scenes).unwrap_or_default();
        let manuscript_segments = manuscript
            .as_ref()
            .map(|value| split_manuscript_scenes(&value.document.body))
            .unwrap_or_default();
        let scene_authority = manuscript.as_ref()
            .and_then(|value| value.document.metadata.get("scene_authority"))
            .and_then(|value| value.as_str()) == Some("individual_files");
        let authority_ids = if scene_authority {
            let chapter_document = &manuscript.as_ref().unwrap().document;
            let ids: Vec<Uuid> = serde_yaml::from_value(
                chapter_document.metadata.get("authoritative_scene_ids")
                    .cloned().ok_or_else(|| "Missing authoritative scene IDs.".to_string())?
            ).map_err(|e| format!("Invalid authoritative scene IDs: {e}"))?;
            Some(ids)
        } else { None };

        let scene_count = authority_ids.as_ref().map(|ids| ids.len())
            .unwrap_or_else(|| plan_scenes.len().max(manuscript_segments.len()).max(1));
        // After verified migration, the original chapter body remains as a
        // non-authoritative recovery copy. Individual scene files own prose.
        if let Some(ids) = authority_ids.as_ref() {
            if ids.len() != scene_count || ids.iter().collect::<std::collections::HashSet<_>>().len() != ids.len() {
                return Err("Authoritative scene IDs must uniquely match the legacy scene count.".into());
            }
        }
        let mut scenes = Vec::new();

        for scene_index in 0..scene_count {
            let planned = plan_scenes.get(scene_index);
            // Resolve the same legacy identity used by the editor and scene saver.
            // Previously this lookup used an empty title when no plan existed,
            // while the editor generated a chapter/scene fallback title. That
            // mismatch made saved scene documents disappear from the projection.
            let scene_title = authority_ids.as_ref()
                .and_then(|ids| ids.get(scene_index))
                .and_then(|id| manuscript_scenes.get(id))
                .and_then(|scene| scene.document.title.clone())
                .or_else(|| planned.map(|value| value.title.clone()))
                .unwrap_or_else(|| {
                    if scene_count == 1 {
                        title.clone()
                    } else {
                        format!("Scene {}", scene_index + 1)
                    }
                });
            let scene_key = format!("{}:scene:{}:{}", key, scene_index, normalize_structure_title(&scene_title));
            let scene_id = if let Some(ids) = authority_ids.as_ref() {
                *ids.get(scene_index).ok_or_else(|| "Authoritative scene count mismatch.".to_string())?
            } else {
                deterministic_uuid(&chapter_id.to_string(), &scene_key)
            };
            let materialized = manuscript_scenes.remove(&scene_id);
            if scene_authority && materialized.is_none() {
                return Err(format!("Authoritative scene file missing for {scene_id}."));
            }
            // Once a chapter manuscript exists it is the prose authority. Older
            // materialized scene files remain on disk for non-destructive recovery,
            // but they must not override edits made in the chapter manuscript.
            let prose_markdown = if manuscript.is_some() && !scene_authority {
                manuscript_segments.get(scene_index).cloned().unwrap_or_default()
            } else {
                materialized
                    .as_ref()
                    .map(|value| value.document.body.trim().to_string())
                    .unwrap_or_default()
            };
            let drafted = !prose_markdown.trim().is_empty();


            let notes = planned
                .map(|value| value.notes.trim().to_string())
                .filter(|value| !value.is_empty());

            let metadata = write_metadata.scenes.get(&scene_id.to_string());
            scenes.push(Scene {
                id: scene_id,
                chapter_id,
                title: scene_title,
                synopsis: notes,
                prose: if drafted {
                    Some(markdown_to_editor_html(&prose_markdown))
                } else {
                    Some(String::new())
                },
                position: scene_index as i32,
                source_id: Some(if drafted {
                    format!("lithomark:drafted:{scene_index}")
                } else {
                    format!("lithomark:planned:{scene_index}")
                }),
                archived: false,
                locked: false,
                scene_type: metadata.and_then(|value| value.scene_type.clone()).unwrap_or(SceneType::Normal),
                scene_status: metadata.and_then(|value| value.scene_status.clone()).unwrap_or(SceneStatus::Draft),
                planning_status: metadata
                    .and_then(|value| value.planning_status.clone())
                    .unwrap_or(if drafted { PlanningStatus::Fixed } else { PlanningStatus::Flexible }),
                // File-backed chapters currently store prose by scene section.
                // Page mode maps cleanly to that representation while planning
                // beats remain available to the projection for later beat-mode use.
                editor_mode: EditorMode::Page,
            });

            let mut beat_values = Vec::new();
            if let Some(planned) = planned {
                for (beat_index, content) in planned.beats.iter().enumerate() {
                    beat_values.push(Beat {
                        id: deterministic_uuid(
                            &scene_id.to_string(),
                            &format!("beat:{beat_index}:{}", normalize_structure_title(content)),
                        ),
                        scene_id,
                        content: content.clone(),
                        prose: None,
                        position: beat_index as i32,
                        source_id: Some("lithomark:planning".into()),
                    });
                }
            }
            beats_by_scene.insert(scene_id.to_string(), beat_values);
        }

        scenes_by_chapter.insert(chapter_id.to_string(), scenes);
    }

    let mut projection = WriteProjection {
        chapters,
        scenes_by_chapter,
        beats_by_scene,
    };
    // Outline is allowed to drive labels and ordering only after every
    // chapter/scene identity and parent has been verified. This preserves
    // existing manuscript prose, source paths, and metadata by UUID.
    // Unmatched legacy projects retain their original Write projection.
    if let Some((outline, _)) = crate::outline::load(project_root)? {
        apply_compatible_outline(&mut projection, &outline);
    }
    Ok(projection)
}

/// Apply Outline structure only when it is a one-to-one identity match.
/// Never infer identity from a title, ordinal, or filename.
fn apply_compatible_outline(
    projection: &mut WriteProjection,
    outline: &crate::outline::Outline,
) -> bool {
    let audit = crate::outline::audit_write_identity(outline, projection);
    // Parent changes are allowed only when the complete set of chapter and
    // scene UUIDs matches. Chapter manuscript sections are positional prose
    // authority, so moving their scenes would silently attach the wrong text.
    // Refuse that projection until a separate, explicit file migration exists.
    if !audit.outline_only_chapters.is_empty()
        || !audit.write_only_chapters.is_empty()
        || !audit.outline_only_scenes.is_empty()
        || !audit.write_only_scenes.is_empty()
        || (!audit.conflicting_scene_parents.is_empty()
            && projection.chapters.iter().any(|chapter|
                chapter.source_id.as_deref().is_some_and(|source| source.starts_with("lithomark:file:"))))
    {
        return false;
    }
    let mut scenes_by_id = projection.scenes_by_chapter.values()
        .flat_map(|scenes| scenes.iter().cloned())
        .map(|scene| (scene.id, scene))
        .collect::<HashMap<_, _>>();
    let mut chapters_by_id = projection.chapters.drain(..)
        .map(|chapter| (chapter.id, chapter))
        .collect::<HashMap<_, _>>();
    let mut ordered = Vec::with_capacity(outline.chapters.len());
    for (chapter_position, outline_chapter) in outline.chapters.iter().enumerate() {
        let mut chapter = chapters_by_id.remove(&outline_chapter.id)
            .expect("verified chapter identity");
        chapter.title = outline_chapter.title.clone();
        chapter.position = chapter_position as i32;
        let mut scenes = Vec::with_capacity(outline_chapter.scenes.len());
        for (position, outline_scene) in outline_chapter.scenes.iter().enumerate() {
            let mut scene = scenes_by_id.remove(&outline_scene.id)
                .expect("verified scene identity");
            scene.chapter_id = chapter.id;
            scene.title = outline_scene.title.clone();
            scene.synopsis = Some(outline_scene.synopsis.clone());
            scene.position = position as i32;
            scenes.push(scene);
            // Outline beat IDs are authoritative after a compatible link.
            projection.beats_by_scene.insert(outline_scene.id.to_string(),
                outline_scene.beats.iter().enumerate().map(|(position, beat)| Beat {
                    id: beat.id,
                    scene_id: outline_scene.id,
                    content: beat.content.clone(),
                    prose: None,
                    position: position as i32,
                    source_id: Some("lithomark:outline".into()),
                }).collect());
        }
        projection.scenes_by_chapter.insert(chapter.id.to_string(), scenes);
        ordered.push(chapter);
    }
    projection.chapters = ordered;
    true
}

pub fn projection_word_counts(
    projection: &WriteProjection,
) -> (i64, HashMap<String, i64>, HashMap<String, i64>) {
    let mut total = 0_i64;
    let mut chapter_words = HashMap::new();
    let mut scene_words = HashMap::new();
    for chapter in &projection.chapters {
        let mut chapter_total = 0_i64;
        for scene in projection
            .scenes_by_chapter
            .get(&chapter.id.to_string())
            .into_iter()
            .flatten()
        {
            let count = crate::db::writing::count_words(scene.prose.as_deref().unwrap_or(""));
            scene_words.insert(scene.id.to_string(), count);
            chapter_total += count;
        }
        chapter_words.insert(chapter.id.to_string(), chapter_total);
        total += chapter_total;
    }
    (total, chapter_words, scene_words)
}


pub fn save_chapter_prose(
    project_root: &Path,
    project_id: Uuid,
    chapter_id: Uuid,
    prose_html: &str,
) -> Result<WriteProjection, String> {
    let projection = build_write_projection(project_root, project_id)?;
    let chapter = projection
        .chapters
        .iter()
        .find(|chapter| chapter.id == chapter_id)
        .ok_or_else(|| "Chapter was not found in the authoritative Write projection.".to_string())?;

    // Scene-owned chapters must be edited through their scene files.
    // Writing a chapter-wide body here would introduce a second prose authority.
    if chapter.source_id.as_deref().is_some_and(|source| source.starts_with("lithomark:scene-files:")) {
        return Err("This chapter uses individual scene files. Edit its scenes instead of replacing the chapter manuscript.".into());
    }

    let prose_markdown = html_to_markdown_rs::convert(prose_html, None)
        .map_err(|e| format!("Could not convert chapter prose to Markdown: {e}"))?
        .content
        .unwrap_or_default();
    let markdown = if prose_markdown.trim().is_empty() {
        format!("# {}\n", chapter.title)
    } else {
        format!("# {}\n\n{}\n", chapter.title, prose_markdown.trim())
    };

    let relative_path = chapter
        .source_id
        .as_deref()
        .and_then(|source| source.strip_prefix("lithomark:file:"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(format!(
                "Manuscript/Chapter {:02} - {}.md",
                chapter.position + 1,
                safe_path_segment(&chapter.title),
            ))
        });

    if project_root.join(&relative_path).exists() {
        let mut document = read_document(project_root, &relative_path)?;
        if document.identity.id != format!("ch-{chapter_id}") {
            return Err("Existing chapter manuscript has a different stable identity.".into());
        }
        document.body = markdown;
        let expected = document.source_hash.clone();
        save_document(project_root, &document, Some(&expected))?;
    } else {
        let mut metadata = std::collections::BTreeMap::new();
        metadata.insert("position".into(), serde_yaml::Value::Number(chapter.position.into()));
        let document = CatalystDocument {
            identity: DocumentIdentity {
                document_type: "chapter".into(),
                id: format!("ch-{chapter_id}"),
            },
            title: Some(chapter.title.clone()),
            metadata,
            body: markdown,
            relative_path,
            source_hash: String::new(),
        };
        save_document(project_root, &document, None)?;
    }

    build_write_projection(project_root, project_id)
}

/// Stage a lossless, non-authoritative extraction of a positional chapter.
/// The original chapter remains the prose authority until a separate,
/// verified cutover. Existing scene files are never overwritten.
pub fn stage_legacy_chapter_scenes(
    project_root: &Path,
    project_id: Uuid,
    chapter_id: Uuid,
) -> Result<Vec<std::path::PathBuf>, String> {
    let projection = build_write_projection(project_root, project_id)?;
    let chapter = projection.chapters.iter().find(|chapter| chapter.id == chapter_id)
        .ok_or_else(|| "Chapter not found.".to_string())?;
    let source = chapter.source_id.as_deref()
        .and_then(|source| source.strip_prefix("lithomark:file:"))
        .ok_or_else(|| "Only file-backed legacy chapters can be staged.".to_string())?;
    let source_document = read_document(project_root, Path::new(source))?;
    let segments = split_manuscript_scenes(&source_document.body);
    let scenes = projection.scenes_by_chapter.get(&chapter_id.to_string())
        .ok_or_else(|| "Chapter has no projected scenes.".to_string())?;
    if segments.is_empty() || segments.len() != scenes.len() {
        return Err("Cannot safely stage: chapter prose segments and scene identities do not match.".into());
    }
    // Preflight the entire batch before writing any files. A retry must not
    // silently overwrite previous staged prose.
    let existing = collect_manuscript_scenes(project_root)?;
    let mut pending = Vec::new();
    for (index, (scene, segment)) in scenes.iter().zip(segments.iter()).enumerate() {
        if existing.contains_key(&scene.id) {
            return Err(format!("Scene {} already has a manuscript file; refusing to overwrite.", scene.id));
        }
        let relative = std::path::PathBuf::from(format!(
            "Manuscript/Scenes/{}.md", scene.id
        ));
        if project_root.join(&relative).exists() {
            return Err(format!("Destination already exists: {}", relative.display()));
        }
        let mut metadata = std::collections::BTreeMap::new();
        metadata.insert("chapter_id".into(), serde_yaml::Value::String(chapter_id.to_string()));
        metadata.insert("chapter_title".into(), serde_yaml::Value::String(chapter.title.clone()));
        metadata.insert("position".into(), serde_yaml::Value::Number((index as i32).into()));
        metadata.insert("staged_from".into(), serde_yaml::Value::String(source.to_string()));
        metadata.insert("source_hash".into(), serde_yaml::Value::String(source_document.source_hash.clone()));
        pending.push(CatalystDocument {
            identity: DocumentIdentity { document_type: "scene".into(), id: format!("scn-{}", scene.id) },
            title: Some(scene.title.clone()),
            metadata,
            body: segment.clone(),
            relative_path: relative,
            source_hash: String::new(),
        });
    }
    let mut written = Vec::new();
    for document in pending {
        // The chapter remains untouched and authoritative. If a later write
        // fails, previously staged files are recoverable, not hidden.
        save_document(project_root, &document, None)?;
        let verified = read_document(project_root, &document.relative_path)?;
        if verified.identity.id != document.identity.id
            || !migration_body_matches(&verified.body, &document.body)
        {
            return Err(format!("Staged scene verification failed: {}", document.relative_path.display()));
        }
        written.push(document.relative_path);
    }
    Ok(written)
}

/// Verify a staged legacy chapter before any authority handoff.
/// This function is read-only: it never edits or deletes manuscript files.
pub fn verify_staged_legacy_chapter(
    project_root: &Path,
    project_id: Uuid,
    chapter_id: Uuid,
) -> Result<Vec<Uuid>, String> {
    let projection = build_write_projection(project_root, project_id)?;
    let chapter = projection.chapters.iter().find(|chapter| chapter.id == chapter_id)
        .ok_or_else(|| "Chapter not found.".to_string())?;
    let source = chapter.source_id.as_deref()
        .and_then(|source| source.strip_prefix("lithomark:file:"))
        .ok_or_else(|| "Only legacy file-backed chapters can be verified.".to_string())?;
    let source_document = read_document(project_root, Path::new(source))?;
    let segments = split_manuscript_scenes(&source_document.body);
    let scenes = projection.scenes_by_chapter.get(&chapter_id.to_string())
        .ok_or_else(|| "Chapter has no scenes.".to_string())?;
    if segments.is_empty() || segments.len() != scenes.len() {
        return Err("Staged scene count does not match the original chapter.".into());
    }
    let materialized = collect_manuscript_scenes(project_root)?;
    let mut verified = Vec::with_capacity(scenes.len());
    for (position, (scene, segment)) in scenes.iter().zip(segments.iter()).enumerate() {
        let document = &materialized.get(&scene.id)
            .ok_or_else(|| format!("Staged scene {} is missing.", scene.id))?.document;
        let recorded_hash = document.metadata.get("source_hash").and_then(|v| v.as_str());
        let recorded_path = document.metadata.get("staged_from").and_then(|v| v.as_str());
        let recorded_parent = document.metadata.get("chapter_id").and_then(|v| v.as_str());
        let recorded_position = document.metadata.get("position").and_then(|v| v.as_i64());
        if recorded_hash != Some(source_document.source_hash.as_str())
            || recorded_path != Some(source)
            || recorded_parent != Some(chapter_id.to_string().as_str())
            || recorded_position != Some(position as i64)
            || !migration_body_matches(&document.body, segment)
        {
            return Err(format!("Staged scene {} does not match its original chapter.", scene.id));
        }
        verified.push(scene.id);
    }
    Ok(verified)
}

/// Explicitly activate previously verified scene files. The chapter file is
/// retained verbatim except for its frontmatter authority marker.
pub fn activate_staged_legacy_chapter(
    project_root: &Path,
    project_id: Uuid,
    chapter_id: Uuid,
) -> Result<(), String> {
    let scene_ids = verify_staged_legacy_chapter(project_root, project_id, chapter_id)?;
    let projection = build_write_projection(project_root, project_id)?;
    let chapter = projection.chapters.iter().find(|chapter| chapter.id == chapter_id)
        .ok_or_else(|| "Chapter not found.".to_string())?;
    let source = chapter.source_id.as_deref()
        .and_then(|value| value.strip_prefix("lithomark:file:"))
        .ok_or_else(|| "Chapter is not a legacy manuscript.".to_string())?;
    let mut document = read_document(project_root, Path::new(source))?;
    if document.metadata.contains_key("scene_authority") {
        return Err("Chapter authority was already selected.".into());
    }
    document.metadata.insert("scene_authority".into(), serde_yaml::Value::String("individual_files".into()));
    document.metadata.insert("authoritative_scene_ids".into(),
        serde_yaml::to_value(&scene_ids).map_err(|e| e.to_string())?);
    let expected = document.source_hash.clone();
    save_document(project_root, &document, Some(&expected))?;
    Ok(())
}

fn migration_body_matches(saved: &str, expected: &str) -> bool {
    // Document Service preserves Markdown content but canonicalizes line endings
    // and the number of terminal newlines. Compare using only those same
    // serialization rules so significant spaces/tabs remain protected.
    let normalize = |value: &str| {
        value
            .replace("\r\n", "\n")
            .replace('\r', "\n")
            .trim_end_matches('\n')
            .to_string()
    };
    normalize(saved) == normalize(expected)
}

pub fn save_scene_prose(
    project_root: &Path,
    project_id: Uuid,
    scene_id: Uuid,
    prose_html: &str,
) -> Result<WriteProjection, String> {
    let projection = build_write_projection(project_root, project_id)?;
    let (chapter, scene) = projection
        .chapters
        .iter()
        .find_map(|chapter| {
            projection
                .scenes_by_chapter
                .get(&chapter.id.to_string())
                .and_then(|scenes| scenes.iter().find(|scene| scene.id == scene_id))
                .map(|scene| (chapter, scene))
        })
        .ok_or_else(|| "Scene was not found in the authoritative Write projection.".to_string())?;

    let markdown = html_to_markdown_rs::convert(prose_html, None)
        .map_err(|e| format!("Could not convert scene prose to Markdown: {e}"))?
        .content
        .unwrap_or_default();

    // A scene's UUID, not its title or ordinal, identifies an existing
    // manuscript file. Reuse its actual path after renames or reordering
    // rather than creating a second file at a newly computed path.
    let existing_scenes = collect_manuscript_scenes(project_root)?;
    let relative_path = existing_scenes
        .get(&scene_id)
        .map(|existing| existing.document.relative_path.clone())
        .unwrap_or_else(|| std::path::PathBuf::from(format!(
            "Manuscript/Scenes/{:02} - {}/{:02} - {}.md",
            chapter.position + 1,
            safe_path_segment(&chapter.title),
            scene.position + 1,
            safe_path_segment(&scene.title),
        )));

    if project_root.join(&relative_path).exists() {
        let mut document = read_document(project_root, &relative_path)?;
        if document.identity.id != format!("scn-{scene_id}") {
            return Err("Existing scene manuscript has a different stable identity.".into());
        }
        document.body = markdown;
        // Persist the projected parent and order on the existing UUID-owned
        // scene file. Never move the file by a title-derived path.
        document.title = Some(scene.title.clone());
        document.metadata.insert("chapter_id".into(), serde_yaml::Value::String(chapter.id.to_string()));
        document.metadata.insert("chapter_title".into(), serde_yaml::Value::String(chapter.title.clone()));
        document.metadata.insert("position".into(), serde_yaml::Value::Number(scene.position.into()));
        let expected = document.source_hash.clone();
        save_document(project_root, &document, Some(&expected))?;
    } else {
        let mut metadata = std::collections::BTreeMap::new();
        metadata.insert("chapter_id".into(), serde_yaml::Value::String(chapter.id.to_string()));
        metadata.insert("chapter_title".into(), serde_yaml::Value::String(chapter.title.clone()));
        metadata.insert("position".into(), serde_yaml::Value::Number(scene.position.into()));
        let document = CatalystDocument {
            identity: DocumentIdentity {
                document_type: "scene".into(),
                id: format!("scn-{scene_id}"),
            },
            title: Some(scene.title.clone()),
            metadata,
            body: markdown,
            relative_path,
            source_hash: String::new(),
        };
        save_document(project_root, &document, None)?;
    }

    build_write_projection(project_root, project_id)
}

fn safe_path_segment(value: &str) -> String {
    let cleaned = value
        .chars()
        .map(|ch| if matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') { '-' } else { ch })
        .collect::<String>();
    let cleaned = cleaned.trim().trim_end_matches(['.', ' ']);
    if cleaned.is_empty() { "Untitled".into() } else { cleaned.to_string() }
}

pub fn update_scene_metadata(
    project_root: &Path,
    scene_id: Uuid,
    scene_type: Option<SceneType>,
    scene_status: Option<SceneStatus>,
    planning_status: Option<PlanningStatus>,
) -> Result<(), String> {
    let mut metadata = read_write_metadata(project_root)?;
    let entry = metadata.scenes.entry(scene_id.to_string()).or_default();
    if let Some(value) = scene_type {
        entry.scene_type = Some(value);
    }
    if let Some(value) = scene_status {
        entry.scene_status = Some(value);
    }
    if let Some(value) = planning_status {
        entry.planning_status = Some(value);
    }
    write_write_metadata(project_root, &metadata)
}

fn manuscript_template(template_id: &str) -> Result<EntityTemplate, String> {
    crate::entity_templates::built_in_template(template_id)
        .filter(|template| template.manifest.scope == "manuscript")
        .ok_or_else(|| format!("Built-in manuscript template '{template_id}' is unavailable."))
}

fn template_metadata(template: &EntityTemplate) -> std::collections::BTreeMap<String, serde_yaml::Value> {
    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert("template_id".into(), serde_yaml::Value::String(template.manifest.id.clone()));
    if let Some(subtype) = &template.manifest.subtype {
        metadata.insert("subtype".into(), serde_yaml::Value::String(subtype.clone()));
    }
    for field in &template.manifest.fields {
        let value = match field.field_type.as_str() {
            "list" => serde_yaml::Value::Sequence(Vec::new()),
            "boolean" => serde_yaml::Value::Bool(false),
            "number" => serde_yaml::Value::Null,
            _ => serde_yaml::Value::String(String::new()),
        };
        metadata.entry(field.key.clone()).or_insert(value);
    }
    metadata
}

pub fn append_planned_chapter(
    project_root: &Path,
    project_id: Uuid,
    title: &str,
    synopsis: &str,
) -> Result<(), String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("Chapter title must not be empty.".into());
    }

    let existing = build_write_projection(project_root, project_id)?;
    // Manuscript is authoritative. A new chapter must not require or mutate
    // a separate Master Outline document.
    if existing.chapters.iter().any(|chapter| chapter.title.eq_ignore_ascii_case(title)) {
        return Err("A chapter with this title already exists.".into());
    }
    let synopsis = synopsis.trim();

    // A chapter created from Write is immediately a real manuscript chapter.
    // The outline remains its planning source; the manuscript file is authoritative prose.
    let position = existing.chapters.len() as i32;
    // New canonical chapter identities must survive title changes and must not be recycled.
    let chapter_id = Uuid::new_v4();
    let relative_path = std::path::PathBuf::from(format!(
        "Manuscript/Chapter {:02} - {}.md",
        position + 1,
        safe_path_segment(title),
    ));
    if project_root.join(&relative_path).exists() {
        return Err("A manuscript file already exists at the new chapter path.".into());
    }
    {
        let template = manuscript_template("manuscript-chapter")?;
        let mut metadata = template_metadata(&template);
        metadata.insert("position".into(), serde_yaml::Value::Number(position.into()));
        metadata.insert("chapter_number".into(), serde_yaml::Value::String((position + 1).to_string()));
        metadata.insert("status".into(), serde_yaml::Value::String("draft".into()));
        if !synopsis.is_empty() {
            metadata.insert("synopsis".into(), serde_yaml::Value::String(synopsis.to_string()));
        }
        let manuscript = CatalystDocument {
            identity: DocumentIdentity {
                document_type: "chapter".into(),
                id: format!("ch-{chapter_id}"),
            },
            title: Some(title.to_string()),
            metadata,
            body: format!("# {title}\n"),
            relative_path,
            source_hash: String::new(),
        };
        save_document(project_root, &manuscript, None)?;
    }
    // Explicitly append the new chapter even when legacy manuscript positions have gaps.
    let mut metadata = read_write_metadata(project_root)?;
    let mut order = existing.chapters.iter().map(|chapter| chapter.id.to_string()).collect::<Vec<_>>();
    order.push(chapter_id.to_string());
    metadata.chapter_order = order;
    write_write_metadata(project_root, &metadata)?;
    Ok(())
}

pub fn reorder_file_chapters(
    project_root: &Path,
    project_id: Uuid,
    chapter_ids: &[String],
) -> Result<WriteProjection, String> {
    let projection = build_write_projection(project_root, project_id)?;
    let existing = projection.chapters.iter().map(|chapter| chapter.id.to_string()).collect::<Vec<_>>();
    if chapter_ids.len() != existing.len()
        || chapter_ids.iter().collect::<std::collections::HashSet<_>>().len() != existing.len()
        || !existing.iter().all(|id| chapter_ids.contains(id))
    {
        return Err("Chapter reorder must include every existing chapter exactly once.".into());
    }
    let mut metadata = read_write_metadata(project_root)?;
    metadata.chapter_order = chapter_ids.to_vec();
    write_write_metadata(project_root, &metadata)?;
    build_write_projection(project_root, project_id)
}

pub fn append_planned_scene(
    project_root: &Path,
    chapter_title: &str,
    scene_title: &str,
    synopsis: &str,
    pov: &str,
    location: &str,
    purpose: &str,
    reveal: &str,
) -> Result<(), String> {
    let scene_title = scene_title.trim();
    if scene_title.is_empty() {
        return Err("Scene title must not be empty.".into());
    }
    let chapters = collect_manuscript_chapters(project_root)?;
    let matches = chapters.iter()
        .filter(|chapter| chapter.document.title.as_deref()
            .is_some_and(|title| title.eq_ignore_ascii_case(chapter_title.trim())))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err("Scene creation requires exactly one canonical manuscript chapter with this title.".into());
    }
    let chapter = &matches[0].document;
    let chapter_id = uuid_from_stable_id(&chapter.identity.id)
        .ok_or_else(|| "Chapter has an invalid stable ID.".to_string())?;
    let existing = collect_manuscript_scenes(project_root)?;
    let mut scene_ids: Vec<Uuid> = if chapter.metadata.get("scene_authority")
        .and_then(|value| value.as_str()) == Some("individual_files") {
        serde_yaml::from_value(chapter.metadata.get("authoritative_scene_ids")
            .cloned().ok_or_else(|| "Missing authoritative scene IDs.".to_string())?)
            .map_err(|e| format!("Invalid authoritative scene IDs: {e}"))?
    } else {
        // A legacy chapter with prose cannot be converted implicitly: the
        // existing positional scenes must first undergo verified migration.
        if !split_manuscript_scenes(&chapter.body).is_empty() {
            return Err("This chapter contains legacy prose. Migrate its scenes before adding file-authoritative scenes.".into());
        }
        if existing.values().any(|scene| scene.document.metadata.get("chapter_id")
            .and_then(|value| value.as_str()) == Some(chapter_id.to_string().as_str())) {
            return Err("Unlinked scene files already exist for this chapter; refusing implicit migration.".into());
        }
        Vec::new()
    };
    // Do not append to a corrupt manifest or attach another chapter's scene.
    let unique: std::collections::HashSet<_> = scene_ids.iter().collect();
    if unique.len() != scene_ids.len() {
        return Err("Chapter contains duplicate authoritative scene IDs.".into());
    }
    for id in &scene_ids {
        let scene = existing.get(id)
            .ok_or_else(|| format!("Authoritative scene file {id} is missing."))?;
        if scene.document.metadata.get("chapter_id").and_then(|value| value.as_str())
            != Some(chapter_id.to_string().as_str()) {
            return Err(format!("Authoritative scene {id} belongs to another chapter."));
        }
    }
    if scene_ids.iter().any(|id| existing.get(id).is_some_and(|scene|
        scene.document.title.as_deref().is_some_and(|title| title.eq_ignore_ascii_case(scene_title)))) {
        return Err("A scene with this title already exists in the chapter.".into());
    }
    let scene_id = Uuid::new_v4();
    let relative_path = std::path::PathBuf::from(format!("Manuscript/Scenes/{scene_id}.md"));
    if project_root.join(&relative_path).exists() {
        return Err("A scene file already exists at the generated path.".into());
    }
    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert("chapter_id".into(), serde_yaml::Value::String(chapter_id.to_string()));
    metadata.insert("chapter_title".into(), serde_yaml::Value::String(chapter_title.trim().to_string()));
    metadata.insert("position".into(), serde_yaml::Value::Number((scene_ids.len() as i32).into()));
    for (key, value) in [("synopsis", synopsis), ("pov", pov), ("location", location),
        ("purpose", purpose), ("reveal", reveal)] {
        if !value.trim().is_empty() {
            metadata.insert(key.into(), serde_yaml::Value::String(value.trim().to_string()));
        }
    }
    let scene = CatalystDocument {
        identity: DocumentIdentity { document_type: "scene".into(), id: format!("scn-{scene_id}") },
        title: Some(scene_title.to_string()),
        metadata,
        body: String::new(),
        relative_path: relative_path.clone(),
        source_hash: String::new(),
    };
    save_document(project_root, &scene, None)?;
    scene_ids.push(scene_id);
    let mut updated_chapter = chapter.clone();
    updated_chapter.metadata.insert("scene_authority".into(),
        serde_yaml::Value::String("individual_files".into()));
    updated_chapter.metadata.insert("authoritative_scene_ids".into(),
        serde_yaml::to_value(&scene_ids).map_err(|e| e.to_string())?);
    if let Err(error) = save_document(project_root, &updated_chapter, Some(&chapter.source_hash)) {
        // Leave no orphaned scene file when the chapter's optimistic write fails.
        let _ = fs::remove_file(project_root.join(&relative_path));
        return Err(error);
    }
    Ok(())
}

fn master_outline_path(project_root: &Path) -> Result<std::path::PathBuf, String> {
    let root = project_root.join(MASTER_OUTLINE_ROOT);
    if !root.is_dir() {
        return Err("This project does not have a canonical Master Outline yet.".into());
    }
    let mut candidates = WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && entry.path().extension().and_then(|v| v.to_str()) == Some("md"))
        .map(|entry| entry.into_path())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.into_iter().next().ok_or_else(|| "The Master Outline folder is empty.".into())
}

fn read_write_metadata(project_root: &Path) -> Result<WriteMetadata, String> {
    let path = project_root.join(WRITE_METADATA_FILE);
    if !path.exists() {
        return Ok(WriteMetadata::default());
    }
    let raw = fs::read_to_string(&path)
        .map_err(|e| format!("Could not read Write metadata '{}': {e}", path.display()))?;
    serde_yaml::from_str(&raw).map_err(|e| format!("Invalid Write metadata YAML: {e}"))
}

fn write_write_metadata(project_root: &Path, metadata: &WriteMetadata) -> Result<(), String> {
    let path = project_root.join(WRITE_METADATA_FILE);
    let parent = path.parent().ok_or_else(|| "Write metadata path has no parent.".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("Could not create Manuscript directory: {e}"))?;
    let raw = serde_yaml::to_string(metadata).map_err(|e| format!("Could not serialize Write metadata: {e}"))?;
    fs::write(&path, raw).map_err(|e| format!("Could not save Write metadata '{}': {e}", path.display()))
}


fn collect_manuscript_chapters(project_root: &Path) -> Result<Vec<ManuscriptChapter>, String> {
    let root = project_root.join(MANUSCRIPT_ROOT);
    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let mut chapters = Vec::new();
    for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() || entry.path().extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(project_root)
            .map_err(|_| "Manuscript path escaped project root.".to_string())?;
        let document = match read_document(project_root, relative) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if document.identity.document_type != "chapter" {
            continue;
        }
        let position = document
            .metadata
            .get("position")
            .and_then(|value| value.as_i64())
            .map(|value| value as i32)
            .unwrap_or(i32::MAX);
        chapters.push(ManuscriptChapter { document, position });
    }
    chapters.sort_by(|a, b| {
        a.position.cmp(&b.position).then_with(|| a.document.relative_path.cmp(&b.document.relative_path))
    });
    Ok(chapters)
}


fn collect_manuscript_scenes(project_root: &Path) -> Result<HashMap<Uuid, ManuscriptScene>, String> {
    let root = project_root.join(MANUSCRIPT_ROOT);
    if !root.is_dir() {
        return Ok(HashMap::new());
    }
    let mut scenes: HashMap<Uuid, ManuscriptScene> = HashMap::new();
    for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() || entry.path().extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let relative = entry.path().strip_prefix(project_root).map_err(|_| "Manuscript path escaped project root.".to_string())?;
        let document = match read_document(project_root, relative) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if document.identity.document_type != "scene" {
            continue;
        }
        if let Some(id) = uuid_from_stable_id(&document.identity.id) {
            if let Some(previous) = scenes.get(&id) {
                return Err(format!(
                    "Duplicate scene identity {id} in {} and {}. Resolve the duplicate before editing or migrating manuscript files.",
                    previous.document.relative_path.display(),
                    document.relative_path.display(),
                ));
            }
            scenes.insert(id, ManuscriptScene { document });
        }
    }
    Ok(scenes)
}

fn collect_planning_structure(project_root: &Path) -> Result<Vec<PlanChapter>, String> {
    let mut output = Vec::new();

    for root_name in PLANNING_ROOTS {
        let root = project_root.join(root_name);
        if !root.exists() {
            continue;
        }
        let walker = if root.is_file() {
            WalkDir::new(root.parent().unwrap_or(project_root)).max_depth(1)
        } else {
            WalkDir::new(&root)
        };

        for entry in walker.into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() || entry.path().extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            if root.is_file() && entry.path() != root {
                continue;
            }
            let relative = entry
                .path()
                .strip_prefix(project_root)
                .map_err(|_| "Planning path escaped project root.".to_string())?;
            let document = match read_document(project_root, relative) {
                Ok(value) => value,
                Err(_) => continue,
            };
            let path_text = relative.to_string_lossy().to_ascii_lowercase();
            let beat_sheet = path_text.contains("beat");
            let parsed = parse_heading_plan(&document.body, beat_sheet);
            for chapter in parsed {
                if !chapter.scenes.is_empty() {
                    output.push(chapter);
                }
            }
        }
    }

    Ok(output)
}

fn parse_heading_plan(body: &str, beat_sheet: bool) -> Vec<PlanChapter> {
    let mut chapters = Vec::<PlanChapter>::new();
    let mut current_chapter: Option<PlanChapter> = None;
    let mut current_scene: Option<PlanScene> = None;

    let finish_scene = |chapter: &mut Option<PlanChapter>, scene: &mut Option<PlanScene>| {
        if let (Some(chapter), Some(scene)) = (chapter.as_mut(), scene.take()) {
            chapter.scenes.push(scene);
        }
    };
    let finish_chapter = |chapters: &mut Vec<PlanChapter>, chapter: &mut Option<PlanChapter>| {
        if let Some(chapter) = chapter.take() {
            chapters.push(chapter);
        }
    };

    for raw_line in body.replace("\r\n", "\n").replace('\r', "\n").lines() {
        let line = raw_line.trim_end();

        if let Some(title) = line.strip_prefix("## ") {
            finish_scene(&mut current_chapter, &mut current_scene);
            finish_chapter(&mut chapters, &mut current_chapter);
            current_chapter = Some(PlanChapter {
                title: title.trim().to_string(),
                scenes: Vec::new(),
            });
            continue;
        }

        if let Some(title) = line.strip_prefix("### ") {
            finish_scene(&mut current_chapter, &mut current_scene);
            if current_chapter.is_none() {
                current_chapter = Some(PlanChapter {
                    title: "Planned Chapter".into(),
                    scenes: Vec::new(),
                });
            }
            current_scene = Some(PlanScene {
                title: title.trim().to_string(),
                notes: String::new(),
                beats: Vec::new(),
            });
            continue;
        }

        let Some(scene) = current_scene.as_mut() else {
            continue;
        };

        if let Some(content) = line.strip_prefix("#### ") {
            let content = content.trim();
            if !content.is_empty() {
                scene.beats.push(content.to_string());
            }
            continue;
        }

        let trimmed = line.trim();
        if !trimmed.is_empty() {
            if !scene.notes.is_empty() {
                scene.notes.push('\n');
            }
            scene.notes.push_str(trimmed);

            if beat_sheet {
                let beat = trimmed
                    .strip_prefix("- ")
                    .or_else(|| trimmed.strip_prefix("* "))
                    .or_else(|| trimmed.strip_prefix("+ "));
                if let Some(beat) = beat.map(str::trim).filter(|value| !value.is_empty()) {
                    scene.beats.push(beat.to_string());
                }
            }
        }
    }

    finish_scene(&mut current_chapter, &mut current_scene);
    finish_chapter(&mut chapters, &mut current_chapter);
    chapters
}

fn merge_plan_chapter(existing: &mut PlanChapter, incoming: &PlanChapter) {
    for scene in &incoming.scenes {
        let key = normalize_structure_title(&scene.title);
        if let Some(target) = existing
            .scenes
            .iter_mut()
            .find(|candidate| normalize_structure_title(&candidate.title) == key)
        {
            if target.notes.trim().is_empty() && !scene.notes.trim().is_empty() {
                target.notes = scene.notes.clone();
            }
            for beat in &scene.beats {
                if !target.beats.contains(beat) {
                    target.beats.push(beat.clone());
                }
            }
        } else {
            existing.scenes.push(scene.clone());
        }
    }
}

fn split_manuscript_scenes(body: &str) -> Vec<String> {
    let normalized = body.replace("\r\n", "\n").replace('\r', "\n");
    normalized
        .split("\n---\n")
        .map(strip_leading_chapter_heading)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn strip_leading_chapter_heading(value: &str) -> String {
    let mut lines = value.lines();
    let first = lines.next().unwrap_or_default();
    if first.trim_start().starts_with("# ") {
        lines.collect::<Vec<_>>().join("\n")
    } else {
        value.to_string()
    }
}

fn markdown_to_editor_html(markdown: &str) -> String {
    markdown
        .split("\n\n")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            if let Some(value) = part.strip_prefix("### ") {
                format!("<h3>{}</h3>", escape_html(value))
            } else if let Some(value) = part.strip_prefix("## ") {
                format!("<h2>{}</h2>", escape_html(value))
            } else if let Some(value) = part.strip_prefix("# ") {
                format!("<h1>{}</h1>", escape_html(value))
            } else {
                format!("<p>{}</p>", escape_html(part).replace('\n', "<br>"))
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn normalize_structure_title(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn uuid_from_stable_id(value: &str) -> Option<Uuid> {
    value
        .rsplit_once('-')
        .and_then(|_| value.get(value.len().saturating_sub(36)..))
        .and_then(|uuid| Uuid::parse_str(uuid).ok())
}

fn deterministic_uuid(namespace: &str, key: &str) -> Uuid {
    let mut digest = Sha256::new();
    digest.update(namespace.as_bytes());
    digest.update(b"\0");
    digest.update(key.as_bytes());
    let hash = digest.finalize();
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    // RFC 4122 variant with version 5-style deterministic semantics.
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_scene_creation_without_master_outline() {
        let root = std::env::temp_dir().join(format!("lithomark-scene-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let chapter_id = Uuid::new_v4();
        let chapter = CatalystDocument {
            identity: DocumentIdentity { document_type: "chapter".into(), id: format!("ch-{chapter_id}") },
            title: Some("Opening".into()),
            metadata: Default::default(),
            body: String::new(),
            relative_path: "Manuscript/Opening.md".into(),
            source_hash: String::new(),
        };
        save_document(&root, &chapter, None).unwrap();
        append_planned_scene(&root, "Opening", "Arrival", "", "", "", "", "").unwrap();
        let scenes = collect_manuscript_scenes(&root).unwrap();
        assert_eq!(scenes.len(), 1);
        let saved = read_document(&root, Path::new("Manuscript/Opening.md")).unwrap();
        assert_eq!(saved.metadata.get("scene_authority").and_then(|v| v.as_str()), Some("individual_files"));
        let ids: Vec<Uuid> = serde_yaml::from_value(saved.metadata["authoritative_scene_ids"].clone()).unwrap();
        assert_eq!(ids.len(), 1);
        assert!(scenes.contains_key(&ids[0]));
        assert!(append_planned_scene(&root, "Opening", "Arrival", "", "", "", "", "").is_err());
        std::fs::remove_dir_all(root).unwrap();
    }


    #[test]
    fn outline_cross_chapter_move_preserves_scene_prose_and_identity() {
        let project_id = Uuid::new_v4();
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        let scene_id = Uuid::new_v4();
        let mut projection = WriteProjection {
            chapters: vec![first, second].into_iter().enumerate().map(|(position, id)| Chapter {
                id, project_id, title: format!("Chapter {position}"),
                position: position as i32, source_id: Some("lithomark:planned".into()),
                archived: false, locked: false, is_part: false,
                synopsis: None, planning_status: PlanningStatus::Flexible,
            }).collect(),
            scenes_by_chapter: HashMap::new(),
            beats_by_scene: HashMap::new(),
        };
        projection.scenes_by_chapter.insert(first.to_string(), vec![Scene {
            id: scene_id, chapter_id: first, title: "Original".into(),
            synopsis: None, prose: Some("<p>Keep this prose</p>".into()),
            position: 0, source_id: None, archived: false, locked: false,
            scene_type: SceneType::Normal, scene_status: SceneStatus::Draft,
            planning_status: PlanningStatus::Fixed, editor_mode: EditorMode::Page,
        }]);
        projection.scenes_by_chapter.insert(second.to_string(), vec![]);
        let outline = crate::outline::Outline {
            chapters: vec![
                crate::outline::OutlineChapter { id: first, title: "First".into(), scenes: vec![] },
                crate::outline::OutlineChapter { id: second, title: "Second".into(), scenes: vec![
                    crate::outline::OutlineScene { id: scene_id, title: "Moved".into(),
                        synopsis: String::new(), plotline_ids: vec![], beats: vec![] },
                ] },
            ],
            ..Default::default()
        };
        assert!(apply_compatible_outline(&mut projection, &outline));
        assert!(projection.scenes_by_chapter[&first.to_string()].is_empty());
        let moved = &projection.scenes_by_chapter[&second.to_string()][0];
        assert_eq!(moved.id, scene_id);
        assert_eq!(moved.chapter_id, second);
        assert_eq!(moved.prose.as_deref(), Some("<p>Keep this prose</p>"));
    }

    #[test]
    fn outline_cross_chapter_move_refuses_positional_chapter_manuscript() {
        let project_id = Uuid::new_v4();
        let chapter_id = Uuid::new_v4();
        let destination = Uuid::new_v4();
        let scene_id = Uuid::new_v4();
        let mut projection = WriteProjection {
            chapters: vec![chapter_id, destination].into_iter().enumerate().map(|(position, id)| Chapter {
                id, project_id, title: "Chapter".into(), position: position as i32,
                source_id: Some("lithomark:file:Manuscript/Chapter.md".into()),
                archived: false, locked: false, is_part: false,
                synopsis: None, planning_status: PlanningStatus::Fixed,
            }).collect(),
            scenes_by_chapter: HashMap::new(), beats_by_scene: HashMap::new(),
        };
        projection.scenes_by_chapter.insert(chapter_id.to_string(), vec![Scene {
            id: scene_id, chapter_id, title: "Scene".into(), synopsis: None,
            prose: Some("<p>Chapter prose</p>".into()), position: 0,
            source_id: None, archived: false, locked: false,
            scene_type: SceneType::Normal, scene_status: SceneStatus::Draft,
            planning_status: PlanningStatus::Fixed, editor_mode: EditorMode::Page,
        }]);
        projection.scenes_by_chapter.insert(destination.to_string(), vec![]);
        let outline = crate::outline::Outline {
            chapters: vec![
                crate::outline::OutlineChapter { id: chapter_id, title: "Chapter".into(), scenes: vec![] },
                crate::outline::OutlineChapter { id: destination, title: "Destination".into(),
                    scenes: vec![crate::outline::OutlineScene { id: scene_id, title: "Scene".into(),
                        synopsis: String::new(), plotline_ids: vec![], beats: vec![] }] },
            ],
            ..Default::default()
        };
        assert!(!apply_compatible_outline(&mut projection, &outline));
        assert_eq!(projection.scenes_by_chapter[&chapter_id.to_string()][0].chapter_id, chapter_id);
    }

    #[test]
    fn compatible_outline_rename_and_reorder_keep_prose_and_scene_ids() {
        let project_id = Uuid::new_v4();
        let chapter_id = Uuid::new_v4();
        let scene_a = Uuid::new_v4();
        let scene_b = Uuid::new_v4();
        let mut projection = WriteProjection {
            chapters: vec![Chapter {
                id: chapter_id, project_id, title: "Old chapter".into(),
                position: 0, source_id: None, archived: false, locked: false,
                is_part: false, synopsis: None, planning_status: PlanningStatus::Flexible,
            }],
            scenes_by_chapter: HashMap::new(),
            beats_by_scene: HashMap::new(),
        };
        let make_scene = |id, title: &str, prose: &str, position| Scene {
            id, chapter_id, title: title.into(), synopsis: None,
            prose: Some(prose.into()), position, source_id: None,
            archived: false, locked: false, scene_type: SceneType::Normal,
            scene_status: SceneStatus::Draft, planning_status: PlanningStatus::Fixed,
            editor_mode: EditorMode::Page,
        };
        projection.scenes_by_chapter.insert(chapter_id.to_string(), vec![
            make_scene(scene_a, "First", "<p>Alpha prose</p>", 0),
            make_scene(scene_b, "Second", "<p>Beta prose</p>", 1),
        ]);
        let make_outline_scene = |id, title: &str| crate::outline::OutlineScene {
            id, title: title.into(), synopsis: String::new(),
            plotline_ids: vec![], beats: vec![],
        };
        let outline = crate::outline::Outline {
            chapters: vec![crate::outline::OutlineChapter {
                id: chapter_id, title: "Renamed chapter".into(),
                scenes: vec![
                    make_outline_scene(scene_b, "Renamed second"),
                    make_outline_scene(scene_a, "Renamed first"),
                ],
            }],
            ..Default::default()
        };
        assert!(apply_compatible_outline(&mut projection, &outline));
        assert_eq!(projection.chapters[0].title, "Renamed chapter");
        let scenes = &projection.scenes_by_chapter[&chapter_id.to_string()];
        assert_eq!(scenes[0].id, scene_b);
        assert_eq!(scenes[0].title, "Renamed second");
        assert_eq!(scenes[0].prose.as_deref(), Some("<p>Beta prose</p>"));
        assert_eq!(scenes[1].id, scene_a);
        assert_eq!(scenes[1].prose.as_deref(), Some("<p>Alpha prose</p>"));
    }

    #[test]
    fn activated_scene_files_drive_prose_without_modifying_legacy_chapter_body() {
        let temp = tempfile::tempdir().unwrap();
        let project_id = Uuid::new_v4();
        let chapter_id = Uuid::new_v4();
        let source_path = Path::new("Manuscript/Chapter One.md");
        let source = CatalystDocument {
            identity: DocumentIdentity { document_type: "chapter".into(), id: format!("ch-{chapter_id}") },
            title: Some("Chapter One".into()),
            metadata: Default::default(),
            body: "# Chapter One\n\nOriginal scene prose.".into(),
            relative_path: source_path.to_path_buf(),
            source_hash: String::new(),
        };
        save_document(temp.path(), &source, None).unwrap();
        let original_body = read_document(temp.path(), source_path).unwrap().body;
        let staged = stage_legacy_chapter_scenes(temp.path(), project_id, chapter_id).unwrap();
        let before = build_write_projection(temp.path(), project_id).unwrap();
        let scene_id = before.scenes_by_chapter[&chapter_id.to_string()][0].id;
        activate_staged_legacy_chapter(temp.path(), project_id, chapter_id).unwrap();
        assert!(activate_staged_legacy_chapter(temp.path(), project_id, chapter_id).is_err());
        let reopened = build_write_projection(temp.path(), project_id).unwrap();
        assert_eq!(reopened.scenes_by_chapter[&chapter_id.to_string()][0].id, scene_id);
        assert!(reopened.scenes_by_chapter[&chapter_id.to_string()][0]
            .prose.as_deref().unwrap().contains("Original scene prose."));
        assert_eq!(read_document(temp.path(), source_path).unwrap().body, original_body);

        let mut edited = read_document(temp.path(), &staged[0]).unwrap();
        edited.body = "Updated independent scene prose.".into();
        let hash = edited.source_hash.clone();
        save_document(temp.path(), &edited, Some(&hash)).unwrap();
        let reopened = build_write_projection(temp.path(), project_id).unwrap();
        assert!(reopened.scenes_by_chapter[&chapter_id.to_string()][0]
            .prose.as_deref().unwrap().contains("Updated independent scene prose."));
        assert_eq!(read_document(temp.path(), source_path).unwrap().body, original_body);

        std::fs::remove_file(temp.path().join(&staged[0])).unwrap();
        assert!(build_write_projection(temp.path(), project_id).is_err());
        assert_eq!(read_document(temp.path(), source_path).unwrap().body, original_body);
    }

    #[test]
    fn activated_scene_rename_via_outline_keeps_stable_identity_and_prose() {
        let temp = tempfile::tempdir().unwrap();
        let project_id = Uuid::new_v4();
        let chapter_id = Uuid::new_v4();
        let chapter_path = Path::new("Manuscript/Chapter One.md");
        let chapter = CatalystDocument {
            identity: DocumentIdentity { document_type: "chapter".into(), id: format!("ch-{chapter_id}") },
            title: Some("Chapter One".into()),
            metadata: Default::default(),
            body: "# Chapter One\n\nKeep this prose.".into(),
            relative_path: chapter_path.to_path_buf(),
            source_hash: String::new(),
        };
        save_document(temp.path(), &chapter, None).unwrap();
        let staged = stage_legacy_chapter_scenes(temp.path(), project_id, chapter_id).unwrap();
        activate_staged_legacy_chapter(temp.path(), project_id, chapter_id).unwrap();
        let before = build_write_projection(temp.path(), project_id).unwrap();
        let original = &before.scenes_by_chapter[&chapter_id.to_string()][0];
        let scene_id = original.id;
        let original_prose = original.prose.clone();
        crate::outline::initialize_from_write(temp.path(), &before).unwrap();
        let (mut outline, hash) = crate::outline::load(temp.path()).unwrap().unwrap();
        outline.chapters[0].scenes[0].title = "A Completely Different Scene Name".into();
        crate::outline::save(temp.path(), &outline, Some(&hash)).unwrap();
        let reopened = build_write_projection(temp.path(), project_id).unwrap();
        let renamed = &reopened.scenes_by_chapter[&chapter_id.to_string()][0];
        assert_eq!(renamed.id, scene_id);
        assert_eq!(renamed.title, "A Completely Different Scene Name");
        assert_eq!(renamed.prose, original_prose);
        assert!(temp.path().join(&staged[0]).exists());
        assert!(read_document(temp.path(), chapter_path).unwrap().body.contains("Keep this prose."));
    }

    #[test]
    fn activated_scene_reorder_survives_outline_save_and_reopen() {
        let temp = tempfile::tempdir().unwrap();
        let project_id = Uuid::new_v4();
        let chapter_id = Uuid::new_v4();
        let chapter_path = Path::new("Manuscript/Chapter One.md");
        let chapter = CatalystDocument {
            identity: DocumentIdentity { document_type: "chapter".into(), id: format!("ch-{chapter_id}") },
            title: Some("Chapter One".into()),
            metadata: Default::default(),
            body: "# Chapter One\n\nFirst passage.\n\n---\n\nSecond passage.".into(),
            relative_path: chapter_path.to_path_buf(),
            source_hash: String::new(),
        };
        save_document(temp.path(), &chapter, None).unwrap();
        let staged = stage_legacy_chapter_scenes(temp.path(), project_id, chapter_id).unwrap();
        assert_eq!(staged.len(), 2);
        activate_staged_legacy_chapter(temp.path(), project_id, chapter_id).unwrap();
        let before = build_write_projection(temp.path(), project_id).unwrap();
        let original = &before.scenes_by_chapter[&chapter_id.to_string()];
        let first_id = original[0].id;
        let second_id = original[1].id;
        let first_prose = original[0].prose.clone();
        let second_prose = original[1].prose.clone();
        let legacy_body = read_document(temp.path(), chapter_path).unwrap().body;
        crate::outline::initialize_from_write(temp.path(), &before).unwrap();
        let (mut outline, hash) = crate::outline::load(temp.path()).unwrap().unwrap();
        outline.move_scene(second_id, chapter_id, 0).unwrap();
        crate::outline::save(temp.path(), &outline, Some(&hash)).unwrap();

        let reopened = build_write_projection(temp.path(), project_id).unwrap();
        let scenes = &reopened.scenes_by_chapter[&chapter_id.to_string()];
        assert_eq!((scenes[0].id, scenes[1].id), (second_id, first_id));
        assert_eq!((scenes[0].position, scenes[1].position), (0, 1));
        assert_eq!(scenes[0].prose, second_prose);
        assert_eq!(scenes[1].prose, first_prose);
        assert_eq!(read_document(temp.path(), chapter_path).unwrap().body, legacy_body);
        assert!(staged.iter().all(|path| temp.path().join(path).exists()));
        assert_eq!(collect_manuscript_scenes(temp.path()).unwrap().len(), 2);
    }

    #[test]
    fn activated_cross_chapter_move_survives_reopen_and_scene_save() {
        let temp = tempfile::tempdir().unwrap();
        let project_id = Uuid::new_v4();
        let first_id = Uuid::new_v4();
        let second_id = Uuid::new_v4();
        let first_path = Path::new("Manuscript/Chapter One.md");
        let second_path = Path::new("Manuscript/Chapter Two.md");
        for (id, path, title, prose) in [
            (first_id, first_path, "Chapter One", "First chapter prose."),
            (second_id, second_path, "Chapter Two", "Second chapter prose."),
        ] {
            save_document(temp.path(), &CatalystDocument {
                identity: DocumentIdentity { document_type: "chapter".into(), id: format!("ch-{id}") },
                title: Some(title.into()),
                metadata: Default::default(),
                body: format!("# {title}\\n\\n{prose}").replace("\\n", "\n"),
                relative_path: path.to_path_buf(),
                source_hash: String::new(),
            }, None).unwrap();
        }
        let first_files = stage_legacy_chapter_scenes(temp.path(), project_id, first_id).unwrap();
        let second_files = stage_legacy_chapter_scenes(temp.path(), project_id, second_id).unwrap();
        activate_staged_legacy_chapter(temp.path(), project_id, first_id).unwrap();
        activate_staged_legacy_chapter(temp.path(), project_id, second_id).unwrap();
        let first_body = read_document(temp.path(), first_path).unwrap().body;
        let second_body = read_document(temp.path(), second_path).unwrap().body;
        let before = build_write_projection(temp.path(), project_id).unwrap();
        let moving = before.scenes_by_chapter[&first_id.to_string()][0].clone();
        crate::outline::initialize_from_write(temp.path(), &before).unwrap();
        let (mut outline, hash) = crate::outline::load(temp.path()).unwrap().unwrap();
        outline.move_scene(moving.id, second_id, 0).unwrap();
        crate::outline::save(temp.path(), &outline, Some(&hash)).unwrap();
        let reopened = build_write_projection(temp.path(), project_id).unwrap();
        assert!(reopened.scenes_by_chapter[&first_id.to_string()].is_empty());
        let moved = &reopened.scenes_by_chapter[&second_id.to_string()][0];
        assert_eq!(moved.id, moving.id);
        assert_eq!(moved.prose, moving.prose);
        assert_eq!(moved.chapter_id, second_id);
        assert_eq!(moved.position, 0);
        let saved = save_scene_prose(temp.path(), project_id, moving.id, "<p>Edited after move.</p>").unwrap();
        assert_eq!(saved.scenes_by_chapter[&second_id.to_string()][0].id, moving.id);
        assert!(saved.scenes_by_chapter[&second_id.to_string()][0].prose.as_deref().is_some_and(|prose| prose.contains("Edited after move.")));
        let document = read_document(temp.path(), &first_files[0]).unwrap();
        assert_eq!(document.identity.id, format!("scn-{}", moving.id));
        assert_eq!(document.metadata["chapter_id"].as_str(), Some(second_id.to_string().as_str()));
        assert_eq!(document.metadata["position"].as_i64(), Some(0));
        assert_eq!(read_document(temp.path(), first_path).unwrap().body, first_body);
        assert_eq!(read_document(temp.path(), second_path).unwrap().body, second_body);
        assert!(second_files[0].exists() || temp.path().join(&second_files[0]).exists());
        assert_eq!(collect_manuscript_scenes(temp.path()).unwrap().len(), 2);
    }

    #[test]
    fn staged_legacy_verification_rejects_changed_source_and_scene() {
        let temp = tempfile::tempdir().unwrap();
        let project_id = Uuid::new_v4();
        let chapter_id = Uuid::new_v4();
        let chapter_path = Path::new("Manuscript/Chapter One.md");
        let chapter = CatalystDocument {
            identity: DocumentIdentity { document_type: "chapter".into(), id: format!("ch-{chapter_id}") },
            title: Some("Chapter One".into()),
            metadata: Default::default(),
            body: "# Chapter One\n\nFirst passage.".into(),
            relative_path: chapter_path.to_path_buf(),
            source_hash: String::new(),
        };
        save_document(temp.path(), &chapter, None).unwrap();
        let staged = stage_legacy_chapter_scenes(temp.path(), project_id, chapter_id).unwrap();
        assert_eq!(verify_staged_legacy_chapter(temp.path(), project_id, chapter_id).unwrap().len(), 1);
        let mut changed_scene = read_document(temp.path(), &staged[0]).unwrap();
        changed_scene.body = "Tampered scene.".into();
        let hash = changed_scene.source_hash.clone();
        save_document(temp.path(), &changed_scene, Some(&hash)).unwrap();
        assert!(verify_staged_legacy_chapter(temp.path(), project_id, chapter_id).is_err());
        let mut restored = read_document(temp.path(), &staged[0]).unwrap();
        restored.body = "First passage.".into();
        let hash = restored.source_hash.clone();
        save_document(temp.path(), &restored, Some(&hash)).unwrap();
        assert_eq!(verify_staged_legacy_chapter(temp.path(), project_id, chapter_id).unwrap().len(), 1);
        let mut changed_chapter = read_document(temp.path(), chapter_path).unwrap();
        changed_chapter.body.push_str("\nAdditional prose.");
        let hash = changed_chapter.source_hash.clone();
        save_document(temp.path(), &changed_chapter, Some(&hash)).unwrap();
        assert!(verify_staged_legacy_chapter(temp.path(), project_id, chapter_id).is_err());
    }

    #[test]
    fn staging_legacy_chapter_preserves_source_and_verifies_scene_files() {
        let temp = tempfile::tempdir().unwrap();
        let project_id = Uuid::new_v4();
        let chapter_id = Uuid::new_v4();
        let original = "# Chapter One\n\nFirst passage.\n\n---\n\nSecond passage.";
        let document = CatalystDocument {
            identity: DocumentIdentity { document_type: "chapter".into(), id: format!("ch-{chapter_id}") },
            title: Some("Chapter One".into()),
            metadata: Default::default(),
            body: original.into(),
            relative_path: Path::new("Manuscript/Chapter One.md").to_path_buf(),
            source_hash: String::new(),
        };
        save_document(temp.path(), &document, None).unwrap();
        let source_path = Path::new("Manuscript/Chapter One.md");
        let source_before = std::fs::read(temp.path().join(source_path)).unwrap();
        let paths = stage_legacy_chapter_scenes(temp.path(), project_id, chapter_id).unwrap();
        assert_eq!(paths.len(), 2);
        assert_eq!(std::fs::read(temp.path().join(source_path)).unwrap(), source_before);
        assert!(read_document(temp.path(), &paths[0]).unwrap().body.contains("First passage"));
        assert!(read_document(temp.path(), &paths[1]).unwrap().body.contains("Second passage"));
        assert!(stage_legacy_chapter_scenes(temp.path(), project_id, chapter_id).is_err());
        let reopened = build_write_projection(temp.path(), project_id).unwrap();
        assert!(reopened.scenes_by_chapter[&chapter_id.to_string()][0]
            .prose.as_deref().unwrap().contains("First passage"));
    }

    #[test]
    fn duplicate_scene_identity_refuses_projection_and_preserves_both_files() {
        let temp = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();
        for (name, prose) in [("One.md", "First original"), ("Two.md", "Second original")] {
            let document = CatalystDocument {
                identity: DocumentIdentity {
                    document_type: "scene".into(),
                    id: format!("scn-{id}"),
                },
                title: Some(name.into()),
                metadata: Default::default(),
                body: prose.into(),
                relative_path: Path::new("Manuscript/Scenes").join(name),
                source_hash: String::new(),
            };
            save_document(temp.path(), &document, None).unwrap();
        }
        let error = build_write_projection(temp.path(), Uuid::new_v4()).unwrap_err();
        assert!(error.contains("Duplicate scene identity"));
        assert!(error.contains("One.md"));
        assert!(error.contains("Two.md"));
        assert!(read_document(temp.path(), Path::new("Manuscript/Scenes/One.md")).unwrap().body.contains("First original"));
        assert!(read_document(temp.path(), Path::new("Manuscript/Scenes/Two.md")).unwrap().body.contains("Second original"));
    }

    #[test]
    fn cross_chapter_move_survives_outline_save_and_project_reopen() {
        let temp = tempfile::tempdir().unwrap();
        let project_id = Uuid::new_v4();
        let planning = CatalystDocument {
            identity: DocumentIdentity {
                document_type: "note".into(),
                id: format!("note-{}", Uuid::new_v4()),
            },
            title: Some("Legacy plan".into()),
            metadata: Default::default(),
            body: "## First chapter\n### Moving scene\n## Second chapter\n### Staying scene\n".into(),
            relative_path: Path::new("Planning/Legacy.md").to_path_buf(),
            source_hash: String::new(),
        };
        save_document(temp.path(), &planning, None).unwrap();
        let initial = build_write_projection(temp.path(), project_id).unwrap();
        assert_eq!(initial.chapters.len(), 2);
        let first_id = initial.chapters[0].id;
        let second_id = initial.chapters[1].id;
        let moving_id = initial.scenes_by_chapter[&first_id.to_string()][0].id;
        save_scene_prose(temp.path(), project_id, moving_id, "<p>Prose survives relocation.</p>").unwrap();
        let original_file = collect_manuscript_scenes(temp.path()).unwrap();
        let original_path = original_file[&moving_id].document.relative_path.clone();

        crate::outline::initialize_from_write(temp.path(), &build_write_projection(temp.path(), project_id).unwrap()).unwrap();
        let (mut outline, hash) = crate::outline::load(temp.path()).unwrap().unwrap();
        outline.move_scene(moving_id, second_id, 1).unwrap();
        crate::outline::save(temp.path(), &outline, Some(&hash)).unwrap();

        // Reconstruct from disk, as a fresh app session would.
        let reopened = build_write_projection(temp.path(), project_id).unwrap();
        assert!(reopened.scenes_by_chapter[&first_id.to_string()].is_empty());
        let moved = reopened.scenes_by_chapter[&second_id.to_string()]
            .iter().find(|scene| scene.id == moving_id).unwrap();
        assert_eq!(moved.chapter_id, second_id);
        assert!(moved.prose.as_deref().unwrap().contains("Prose survives relocation"));

        // Saving from the new parent updates the same manuscript file.
        save_scene_prose(temp.path(), project_id, moving_id, "<p>Prose survives relocation.</p>").unwrap();
        let persisted = read_document(temp.path(), &original_path).unwrap();
        assert_eq!(persisted.identity.id, format!("scn-{moving_id}"));
        assert_eq!(persisted.metadata["chapter_id"].as_str(), Some(second_id.to_string().as_str()));
        assert_eq!(persisted.metadata["position"].as_i64(), Some(1));
        assert!(persisted.body.contains("Prose survives relocation"));
        assert_eq!(collect_manuscript_scenes(temp.path()).unwrap().len(), 1);
        let reopened_again = build_write_projection(temp.path(), project_id).unwrap();
        assert_eq!(reopened_again.scenes_by_chapter[&second_id.to_string()][1].id, moving_id);
    }

    #[test]
    fn scene_save_updates_parent_metadata_without_changing_identity_or_path() {
        let temp = tempfile::tempdir().unwrap();
        let project_id = Uuid::new_v4();
        let planning = CatalystDocument {
            identity: DocumentIdentity {
                document_type: "note".into(),
                id: format!("note-{}", Uuid::new_v4()),
            },
            title: Some("Planning".into()),
            metadata: Default::default(),
            body: "## Chapter One\n### Opening\n".into(),
            relative_path: Path::new("Planning/Legacy.md").to_path_buf(),
            source_hash: String::new(),
        };
        save_document(temp.path(), &planning, None).unwrap();
        let initial = build_write_projection(temp.path(), project_id).unwrap();
        let chapter = &initial.chapters[0];
        let scene = &initial.scenes_by_chapter[&chapter.id.to_string()][0];
        let scene_id = scene.id;
        save_scene_prose(temp.path(), project_id, scene_id, "<p>Preserved prose</p>").unwrap();
        let before = collect_manuscript_scenes(temp.path()).unwrap();
        let path = before[&scene_id].document.relative_path.clone();
        let identity = before[&scene_id].document.identity.clone();
        // Simulate metadata from an earlier position; saving must reconcile
        // it with the current UUID-backed projection.
        let mut stale = read_document(temp.path(), &path).unwrap();
        stale.metadata.insert("chapter_id".into(), serde_yaml::Value::String(Uuid::new_v4().to_string()));
        stale.metadata.insert("position".into(), serde_yaml::Value::Number(99.into()));
        let hash = stale.source_hash.clone();
        save_document(temp.path(), &stale, Some(&hash)).unwrap();

        save_scene_prose(temp.path(), project_id, scene_id, "<p>Preserved prose</p>").unwrap();
        let updated = read_document(temp.path(), &path).unwrap();
        assert_eq!(updated.identity, identity);
        assert_eq!(updated.metadata["chapter_id"].as_str(), Some(chapter.id.to_string().as_str()));
        assert_eq!(updated.metadata["position"].as_i64(), Some(0));
        assert!(updated.body.contains("Preserved prose"));
        assert_eq!(collect_manuscript_scenes(temp.path()).unwrap().len(), 1);
    }

    #[test]
    fn scene_save_reuses_existing_path_by_stable_id() {
        let temp = tempfile::tempdir().unwrap();
        let project_id = Uuid::new_v4();
        let first = build_write_projection(temp.path(), project_id).unwrap();
        // A planning-only chapter provides a stable projected scene identity.
        let planning_path = temp.path().join("Planning/Legacy.md");
        fs::create_dir_all(planning_path.parent().unwrap()).unwrap();
        let planning = CatalystDocument {
            identity: DocumentIdentity {
                document_type: "note".into(),
                id: format!("note-{}", Uuid::new_v4()),
            },
            title: Some("Legacy".into()),
            metadata: Default::default(),
            body: "## Chapter One\n### Opening\n".into(),
            relative_path: Path::new("Planning/Legacy.md").to_path_buf(),
            source_hash: String::new(),
        };
        save_document(temp.path(), &planning, None).unwrap();
        let projection = build_write_projection(temp.path(), project_id).unwrap();
        assert!(first.chapters.is_empty());
        let chapter = &projection.chapters[0];
        let scene = &projection.scenes_by_chapter[&chapter.id.to_string()][0];
        let scene_id = scene.id;
        save_scene_prose(temp.path(), project_id, scene_id, "<p>Original prose.</p>").unwrap();
        let original = collect_manuscript_scenes(temp.path()).unwrap();
        let original_path = original[&scene_id].document.relative_path.clone();
        let renamed_path = Path::new("Manuscript/Scenes/Renamed folder/Retitled scene.md");
        fs::create_dir_all(temp.path().join(renamed_path).parent().unwrap()).unwrap();
        fs::rename(temp.path().join(&original_path), temp.path().join(renamed_path)).unwrap();

        save_scene_prose(temp.path(), project_id, scene_id, "<p>Updated prose.</p>").unwrap();
        let saved = collect_manuscript_scenes(temp.path()).unwrap();
        assert_eq!(saved[&scene_id].document.relative_path, renamed_path);
        assert!(saved[&scene_id].document.body.contains("Updated prose."));
        assert!(!temp.path().join(&original_path).exists());
    }

    // Real chapter content from the checked-in Glass Meridian project, copied
    // into an isolated filesystem. The source manuscript is never modified.
    #[test]
    fn saves_and_reopens_real_glass_meridian_chapter_prose() {
        let temp = tempfile::tempdir().expect("temporary project directory");
        let relative = Path::new("Manuscript/Chapter 01 - The Missing Block.md");
        let source = include_str!("../../docs/projects/The Glass Meridian/Manuscript/Chapter 01 - The Missing Block.md");
        let path = temp.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, source).unwrap();

        let project_id = Uuid::new_v4();
        let before = build_write_projection(temp.path(), project_id).unwrap();
        assert_eq!(before.chapters.len(), 1);
        let chapter_id = before.chapters[0].id;
        let identity_before = read_document(temp.path(), relative).unwrap().identity;

        // This is an actual passage from the fixture chapter, in the HTML
        // representation sent by the editor to the native save command.
        let prose = "<p>Mara Vale found Bellwether Row at 8:17 on a Tuesday morning because a road refused to meet the water.</p><p>She zoomed in.</p>";
        let after = save_chapter_prose(temp.path(), project_id, chapter_id, prose)
            .expect("chapter prose must save to the real Markdown file");
        let disk = fs::read_to_string(&path).unwrap();
        assert!(disk.contains("Mara Vale found Bellwether Row at 8:17"));
        assert!(disk.contains("She zoomed in."));
        assert_eq!(read_document(temp.path(), relative).unwrap().identity, identity_before);
        assert_eq!(after.chapters[0].id, chapter_id);

        let reopened = build_write_projection(temp.path(), project_id).unwrap();
        let scenes = reopened.scenes_by_chapter.get(&chapter_id.to_string()).unwrap();
        assert!(scenes.iter().any(|scene| scene.prose.as_deref().unwrap_or("").contains("She zoomed in.")));
    }

    #[test]
    fn parses_outline_chapters_and_scenes() {
        let body = "# Outline\n\n## Chapter 1 — Arrival\n\n### Scene 1 — Door\n\nOpen the door.\n\n### Scene 2 — Hall\n\nCross the hall.";
        let chapters = parse_heading_plan(body, false);
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].scenes.len(), 2);
        assert_eq!(chapters[0].scenes[0].title, "Scene 1 — Door");
    }

    #[test]
    fn beat_sheet_bullets_attach_to_the_current_scene() {
        let body = "## Chapter 1\n### Scene 1\n- Door opens\n- Alarm sounds";
        let chapters = parse_heading_plan(body, true);
        assert_eq!(chapters[0].scenes[0].beats, vec!["Door opens", "Alarm sounds"]);
    }

    #[test]
    fn manuscript_sections_split_on_thematic_breaks() {
        let body = "# Chapter 1\n\nFirst scene.\n\n---\n\nSecond scene.";
        assert_eq!(
            split_manuscript_scenes(body),
            vec!["First scene.", "Second scene."]
        );
    }

    #[test]
    fn materialized_scene_without_plan_uses_editor_identity() {
        let temp = tempfile::tempdir().unwrap();
        let project_id = Uuid::new_v4();
        let chapter_id = Uuid::new_v4();
        let scene_id = deterministic_uuid(
            &chapter_id.to_string(),
            "chapter one:scene:0:chapter one",
        );
        let chapter = CatalystDocument {
            identity: DocumentIdentity { document_type: "chapter".into(), id: format!("ch-{chapter_id}") },
            title: Some("Chapter One".into()),
            metadata: Default::default(),
            body: String::new(),
            relative_path: "Manuscript/Chapter One.md".into(),
            source_hash: String::new(),
        };
        save_document(temp.path(), &chapter, None).unwrap();
        let scene = CatalystDocument {
            identity: DocumentIdentity { document_type: "scene".into(), id: format!("scn-{scene_id}") },
            title: Some("Chapter One".into()),
            metadata: Default::default(),
            body: "Preserved scene prose.".into(),
            relative_path: "Manuscript/Scenes/Chapter One/Scene One.md".into(),
            source_hash: String::new(),
        };
        save_document(temp.path(), &scene, None).unwrap();
        let projection = build_write_projection(temp.path(), project_id).unwrap();
        let projected = &projection.scenes_by_chapter[&chapter_id.to_string()][0];
        assert_eq!(projected.id, scene_id);
        // The chapter manuscript is the authority when present, so the scene
        // file must not silently override chapter prose.
        assert_eq!(projected.prose.as_deref(), Some(""));
    }

    #[test]
    fn deterministic_ids_are_stable() {
        assert_eq!(
            deterministic_uuid("project", "scene"),
            deterministic_uuid("project", "scene")
        );
        assert_ne!(
            deterministic_uuid("project", "scene"),
            deterministic_uuid("project", "other")
        );
    }
}
