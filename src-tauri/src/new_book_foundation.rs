//! File-authoritative New Book Foundation seeder.
//!
//! Setup state is temporary. Pressing Create Book materializes ordinary local
//! LithoMark project files and project.yaml; those files are authoritative from
//! that point forward.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::document::service::{create_document, save_document};
use crate::project_root::{
    initialize_project_root_from_manifest, ProjectManifest, ProjectPaths,
};

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FoundationCharacter {
    pub name: String,
    #[serde(default)]
    pub template_id: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FoundationLocation {
    pub name: String,
    #[serde(default)]
    pub template_id: String,
    #[serde(default)]
    pub location_type: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FoundationWorldbuilding {
    #[serde(default)]
    pub template_id: String,
    #[serde(default)]
    pub setting: String,
    #[serde(default)]
    pub time_period: String,
    #[serde(default)]
    pub organizations: Vec<String>,
    #[serde(default)]
    pub factions: Vec<String>,
    #[serde(default)]
    pub systems: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FoundationStyleVoice {
    #[serde(default)]
    pub pov: String,
    #[serde(default)]
    pub tense: String,
    #[serde(default)]
    pub tone: String,
    #[serde(default)]
    pub style_notes: String,
    #[serde(default)]
    pub voice_notes: String,
    #[serde(default)]
    pub influences: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FoundationThemes {
    #[serde(default)]
    pub themes: Vec<String>,
    #[serde(default)]
    pub central_conflict: String,
    #[serde(default)]
    pub stakes: String,
    #[serde(default)]
    pub story_intent: String,
    #[serde(default)]
    pub motifs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewBookFoundationInput {
    pub project_root: String,
    pub title: String,
    #[serde(default = "default_project_type")]
    pub project_type: String,
    #[serde(default)]
    pub primary_genre: String,
    #[serde(default)]
    pub subgenres: Vec<String>,
    #[serde(default)]
    pub target_audience: String,
    pub target_length: Option<u32>,
    #[serde(default)]
    pub series: String,
    #[serde(default)]
    pub characters: Vec<FoundationCharacter>,
    #[serde(default)]
    pub locations: Vec<FoundationLocation>,
    #[serde(default)]
    pub worldbuilding: FoundationWorldbuilding,
    #[serde(default)]
    pub style_voice: FoundationStyleVoice,
    #[serde(default)]
    pub themes: FoundationThemes,
    #[serde(default)]
    pub brainstorming: String,
    #[serde(default)]
    pub synopsis: String,
    #[serde(default)]
    pub manuscript_template_id: String,
}

fn default_project_type() -> String {
    "novel".into()
}

#[derive(Clone, Debug, Serialize)]
pub struct NewBookProject {
    pub manifest: ProjectManifest,
    pub paths: ProjectPaths,
}

pub fn create_new_book(input: &NewBookFoundationInput) -> Result<NewBookProject, String> {
    let root = PathBuf::from(input.project_root.trim());
    if !root.is_absolute() {
        return Err("New Book destination must be an absolute local folder path.".into());
    }

    let title = input.title.trim();
    if title.is_empty() {
        return Err("Book title must not be empty.".into());
    }

    let project_type = input.project_type.trim();
    if project_type.is_empty() {
        return Err("Project type must not be empty.".into());
    }

    validate_entities(&input.characters, &input.locations)?;

    let world = &input.worldbuilding;
    if [world.setting.as_str(), world.time_period.as_str(), world.systems.as_str(), world.notes.as_str()]
        .iter()
        .all(|value| value.trim().is_empty())
        && world.organizations.iter().all(|value| value.trim().is_empty())
        && world.factions.iter().all(|value| value.trim().is_empty())
    {
        return Err("At least one worldbuilding detail is required to create a book.".into());
    }

    let manifest = ProjectManifest::new(title, project_type);
    let paths = initialize_project_root_from_manifest(&root, &manifest)?;

    seed_manuscript_structure(&root, &input.manuscript_template_id)?;
    seed_foundation(&root, input)?;

    Ok(NewBookProject { manifest, paths })
}

// Legacy book creation remains readable for older files, but Catalyst no
// longer ships the five novel outlining presets. Do not include removed JSON.
fn seed_manuscript_structure(_root: &Path, template_id: &str) -> Result<(), String> {
    match template_id.trim() {
        "" | "decide-later" => Ok(()),
        retired => Err(format!(
            "Novel outline preset '{retired}' was retired from Catalyst; choose an RPG campaign template instead."
        )),
    }
}

fn validate_entities(
    characters: &[FoundationCharacter],
    locations: &[FoundationLocation],
) -> Result<(), String> {
    if characters.iter().any(|character| character.name.trim().is_empty()) {
        return Err("Character names must not be empty.".into());
    }
    if locations.iter().any(|location| location.name.trim().is_empty()) {
        return Err("Location names must not be empty.".into());
    }
    Ok(())
}

fn seed_foundation(root: &Path, input: &NewBookFoundationInput) -> Result<(), String> {
    let mut foundation = create_document(
        root,
        Path::new("Story Development/Foundation/Foundation.md"),
        "note",
        Some("Foundation"),
        "",
    )?;
    insert_string(&mut foundation.metadata, "primary_genre", &input.primary_genre);
    insert_strings(&mut foundation.metadata, "subgenres", &input.subgenres)?;
    insert_string(&mut foundation.metadata, "target_audience", &input.target_audience);
    if let Some(target_length) = input.target_length {
        foundation
            .metadata
            .insert("target_length".into(), Value::Number(target_length.into()));
    }
    insert_string(&mut foundation.metadata, "series", &input.series);
    insert_string(
        &mut foundation.metadata,
        "manuscript_structure_template",
        &input.manuscript_template_id,
    );
    save_document(root, &foundation, Some(&foundation.source_hash))?;

    create_document(
        root,
        Path::new("Story Development/Planning & Outline/Brainstorming/Brainstorming.md"),
        "note",
        Some("Brainstorming"),
        &input.brainstorming,
    )?;
    create_document(
        root,
        Path::new("Story Development/Foundation/Synopsis.md"),
        "note",
        Some("Synopsis"),
        &input.synopsis,
    )?;

    let mut style = create_document(
        root,
        Path::new("Story Development/Foundation/Style & Voice.md"),
        "note",
        Some("Style & Voice"),
        "",
    )?;
    insert_string(&mut style.metadata, "pov", &input.style_voice.pov);
    insert_string(&mut style.metadata, "tense", &input.style_voice.tense);
    insert_string(&mut style.metadata, "tone", &input.style_voice.tone);
    insert_string(&mut style.metadata, "style_notes", &input.style_voice.style_notes);
    insert_string(&mut style.metadata, "voice_notes", &input.style_voice.voice_notes);
    insert_string(&mut style.metadata, "influences", &input.style_voice.influences);
    save_document(root, &style, Some(&style.source_hash))?;

    let mut themes = create_document(
        root,
        Path::new("Story Development/Foundation/Themes.md"),
        "note",
        Some("Themes & Core Ideas"),
        "",
    )?;
    insert_strings(&mut themes.metadata, "themes", &input.themes.themes)?;
    insert_string(
        &mut themes.metadata,
        "central_conflict",
        &input.themes.central_conflict,
    );
    insert_string(&mut themes.metadata, "stakes", &input.themes.stakes);
    insert_string(
        &mut themes.metadata,
        "story_intent",
        &input.themes.story_intent,
    );
    insert_strings(&mut themes.metadata, "motifs", &input.themes.motifs)?;
    save_document(root, &themes, Some(&themes.source_hash))?;

    let mut world = create_document(
        root,
        Path::new("Story Development/Worldbuilding & Settings/Overview.md"),
        "note",
        Some("Worldbuilding"),
        &input.worldbuilding.notes,
    )?;
    insert_string(&mut world.metadata, "template_id", &input.worldbuilding.template_id);
    insert_string(&mut world.metadata, "setting", &input.worldbuilding.setting);
    insert_string(
        &mut world.metadata,
        "time_period",
        &input.worldbuilding.time_period,
    );
    insert_strings(
        &mut world.metadata,
        "organizations",
        &input.worldbuilding.organizations,
    )?;
    insert_strings(&mut world.metadata, "factions", &input.worldbuilding.factions)?;
    insert_string(&mut world.metadata, "systems", &input.worldbuilding.systems);
    save_document(root, &world, Some(&world.source_hash))?;

    seed_characters(root, &input.characters)?;
    seed_locations(root, &input.locations)?;
    Ok(())
}

fn seed_characters(root: &Path, characters: &[FoundationCharacter]) -> Result<(), String> {
    let mut used = BTreeMap::<String, usize>::new();
    for character in characters {
        let relative = unique_entity_path("Story Development/Characters/Main Cast", &character.name, &mut used);
        let mut document = create_document(
            root,
            &relative,
            "character",
            Some(character.name.trim()),
            &character.description,
        )?;
        insert_string(&mut document.metadata, "template_id", &character.template_id);
        insert_string(&mut document.metadata, "role", &character.role);
        insert_strings(&mut document.metadata, "aliases", &character.aliases)?;
        save_document(root, &document, Some(&document.source_hash))?;
    }
    Ok(())
}

fn seed_locations(root: &Path, locations: &[FoundationLocation]) -> Result<(), String> {
    let mut used = BTreeMap::<String, usize>::new();
    for location in locations {
        let relative = unique_entity_path("Story Development/Worldbuilding & Settings/Locations", &location.name, &mut used);
        let mut document = create_document(
            root,
            &relative,
            "location",
            Some(location.name.trim()),
            &location.description,
        )?;
        insert_string(&mut document.metadata, "template_id", &location.template_id);
        insert_string(
            &mut document.metadata,
            "location_type",
            &location.location_type,
        );
        insert_strings(&mut document.metadata, "aliases", &location.aliases)?;
        save_document(root, &document, Some(&document.source_hash))?;
    }
    Ok(())
}

fn unique_entity_path(
    directory: &str,
    name: &str,
    used: &mut BTreeMap<String, usize>,
) -> PathBuf {
    let stem = safe_filename(name);
    let count = used.entry(stem.clone()).or_insert(0);
    *count += 1;
    let filename = if *count == 1 {
        format!("{stem}.md")
    } else {
        format!("{stem} {}.md", *count)
    };
    Path::new(directory).join(filename)
}

fn safe_filename(value: &str) -> String {
    let mut out = String::new();
    for ch in value.trim().chars() {
        if ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') {
            out.push('-');
        } else {
            out.push(ch);
        }
    }
    let out = out
        .trim()
        .trim_end_matches(|ch| ch == '.' || ch == ' ')
        .trim();
    if out.is_empty() {
        "Untitled".into()
    } else {
        out.to_string()
    }
}

fn insert_string(metadata: &mut BTreeMap<String, Value>, key: &str, value: &str) {
    let value = value.trim();
    if !value.is_empty() {
        metadata.insert(key.into(), Value::String(value.into()));
    }
}

fn insert_strings(
    metadata: &mut BTreeMap<String, Value>,
    key: &str,
    values: &[String],
) -> Result<(), String> {
    let values = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| Value::String(value.to_string()))
        .collect::<Vec<_>>();
    if !values.is_empty() {
        metadata.insert(key.into(), Value::Sequence(values));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::service::read_document;
    use crate::project_root::read_project_manifest;

    fn input(root: &Path) -> NewBookFoundationInput {
        NewBookFoundationInput {
            project_root: root.to_string_lossy().into_owned(),
            title: "Test Novel".into(),
            project_type: "novel".into(),
            primary_genre: "Mystery".into(),
            subgenres: vec!["Procedural".into()],
            target_audience: "Adult".into(),
            target_length: Some(90_000),
            series: String::new(),
            characters: vec![FoundationCharacter {
                name: "Test Character".into(),
                template_id: "character-profile".into(),
                role: "protagonist".into(),
                aliases: vec!["TC".into()],
                description: "Synthetic character fixture.".into(),
            }],
            locations: vec![FoundationLocation {
                name: "Test Location".into(),
                template_id: "location-profile".into(),
                location_type: "city".into(),
                aliases: vec![],
                description: "Synthetic location fixture.".into(),
            }],
            worldbuilding: FoundationWorldbuilding {
                template_id: "worldbuilding-profile".into(),
                setting: "Synthetic setting".into(),
                time_period: "Present".into(),
                organizations: vec!["Test Organization".into()],
                factions: vec![],
                systems: "Synthetic rules.".into(),
                notes: "World notes.".into(),
            },
            style_voice: FoundationStyleVoice {
                pov: "Third person limited".into(),
                tense: "Past".into(),
                tone: "Grounded".into(),
                style_notes: "Clean prose.".into(),
                voice_notes: "Distinct voices.".into(),
                influences: String::new(),
            },
            themes: FoundationThemes {
                themes: vec!["Trust".into()],
                central_conflict: "Synthetic conflict.".into(),
                stakes: "Synthetic stakes.".into(),
                story_intent: "Test the seeder.".into(),
                motifs: vec!["Keys".into()],
            },
            brainstorming: "Initial ideas.".into(),
            synopsis: "Synthetic synopsis.".into(),
            manuscript_template_id: "bundled-three-act".into(),
        }
    }

    #[test]
    fn creates_authoritative_local_new_book_foundation() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Test Novel");

        let created = create_new_book(&input(&root)).unwrap();

        assert_eq!(created.manifest.title, "Test Novel");
        assert_eq!(read_project_manifest(&root).unwrap(), created.manifest);
        assert!(root.join("Story Development/Foundation/Foundation.md").is_file());
        assert!(root.join("Story Development/Planning & Outline/Brainstorming/Brainstorming.md").is_file());
        assert!(root.join("Story Development/Foundation/Synopsis.md").is_file());
        assert!(root.join("Story Development/Foundation/Style & Voice.md").is_file());
        assert!(root.join("Story Development/Foundation/Themes.md").is_file());
        assert!(root.join("Story Development/Worldbuilding & Settings/Overview.md").is_file());
        assert!(root.join("Story Development/Characters/Main Cast/Test Character.md").is_file());
        assert!(root.join("Story Development/Worldbuilding & Settings/Locations/Test Location.md").is_file());
        assert!(root.join("Manuscript/Act I — Setup").is_dir());
        assert!(root.join("Manuscript/Act II — Confrontation").is_dir());
        assert!(root.join("Manuscript/Act III — Resolution").is_dir());
        assert!(root.join("Story Development/Planning & Outline/Master Outline").is_dir());
        assert!(root.join("Story Development/Planning & Outline/Beat Sheets").is_dir());
        assert!(root.join("Story Development/Characters/Relationships").is_dir());
        assert!(root.join("Story Development/Characters/Character Arcs").is_dir());
        assert!(root.join("Story Development/Worldbuilding & Settings/Organizations").is_dir());
        assert!(root.join("Story Development/Worldbuilding & Settings/Governments & Factions").is_dir());
        assert!(root.join("Story Development/Timeline/Events").is_dir());
        assert!(root.join("Story Development/Research/Sources").is_dir());
        assert!(root.join("Project Meta/Change Control").is_dir());
        assert!(root.join("Reviews/Developmental").is_dir());
        assert!(root.join("Assets/Characters").is_dir());
        assert!(root.join("Assets/Maps").is_dir());

        let character =
            read_document(&root, Path::new("Story Development/Characters/Main Cast/Test Character.md")).unwrap();
        assert_eq!(
            character.metadata.get("role"),
            Some(&Value::String("protagonist".into()))
        );
        assert_eq!(
            character.metadata.get("template_id"),
            Some(&Value::String("character-profile".into()))
        );
        let world =
            read_document(&root, Path::new("Story Development/Worldbuilding & Settings/Overview.md")).unwrap();
        assert_eq!(
            world.metadata.get("template_id"),
            Some(&Value::String("worldbuilding-profile".into()))
        );
        assert_eq!(character.body, "Synthetic character fixture.\n");
    }

    #[test]
    fn new_book_wiki_handoff_rebuilds_from_canonical_files_and_preserves_ids() {
        use crate::story_development::build_index;
        use std::collections::HashSet;

        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Wiki Handoff");
        let created = create_new_book(&input(&root)).unwrap();
        assert_eq!(read_project_manifest(&root).unwrap(), created.manifest);

        let index = build_index(&root).unwrap();
        assert!(index.issues.is_empty(), "Wiki indexing issues: {:?}", index.issues);
        let expected = [
            "Story Development/Foundation/Foundation.md",
            "Story Development/Foundation/Synopsis.md",
            "Story Development/Foundation/Style & Voice.md",
            "Story Development/Foundation/Themes.md",
            "Story Development/Planning & Outline/Brainstorming/Brainstorming.md",
            "Story Development/Worldbuilding & Settings/Overview.md",
            "Story Development/Characters/Main Cast/Test Character.md",
            "Story Development/Worldbuilding & Settings/Locations/Test Location.md",
        ];
        let mut ids = HashSet::new();
        for path in expected {
            let entry = index.entries.iter().find(|entry| entry.relative_path == path)
                .unwrap_or_else(|| panic!("Missing seeded Wiki document: {path}"));
            let reopened = read_document(&root, Path::new(path)).unwrap();
            assert_eq!(entry.id, reopened.identity.id);
            assert!(ids.insert(entry.id.clone()), "Duplicate stable document ID: {}", entry.id);
        }
        assert_eq!(index.entries.len(), expected.len());
        let world = read_document(
            &root, Path::new("Story Development/Worldbuilding & Settings/Overview.md"),
        ).unwrap();
        assert_eq!(world.title.as_deref(), Some("Worldbuilding"));
        assert_eq!(world.body, "World notes.\n");
        assert_eq!(world.metadata.get("setting"), Some(&Value::String("Synthetic setting".into())));
        assert!(!root.join("Story Bible").exists(), "New books must not seed a second Story Bible authority");
    }

    #[test]
    fn reopened_new_book_wiki_preserves_saved_prose_and_document_identity() {
        use crate::document::service::{read_document, save_document};
        use crate::story_development::build_index;

        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Reopen Wiki");
        let created = create_new_book(&input(&root)).unwrap();
        let path = Path::new("Story Development/Characters/Main Cast/Test Character.md");

        let mut document = read_document(&root, path).unwrap();
        let original_id = document.identity.id.clone();
        let original_hash = document.source_hash.clone();
        document.body = "Edited character biography.\n".into();
        save_document(&root, &document, Some(&original_hash)).unwrap();

        // Reconstruct from disk, rather than retaining an in-memory document or index.
        let reopened_manifest = read_project_manifest(&root).unwrap();
        assert_eq!(reopened_manifest, created.manifest);
        let reopened = read_document(&root, path).unwrap();
        assert_eq!(reopened.identity.id, original_id);
        assert_eq!(reopened.body, "Edited character biography.\n");

        let rebuilt = build_index(&root).unwrap();
        assert!(rebuilt.issues.is_empty(), "Reopened Wiki issues: {:?}", rebuilt.issues);
        let entry = rebuilt.entries.iter().find(|entry| entry.relative_path == path.to_string_lossy())
            .expect("Saved character should remain discoverable after reopen");
        assert_eq!(entry.id, original_id);
        assert_eq!(rebuilt.entries.len(), 8);
        assert!(!root.join("Story Bible").exists());
    }

    #[test]
    fn rejects_empty_worldbuilding_before_creating_project() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("missing-worldbuilding");
        let mut request = input(&root);
        request.worldbuilding.setting.clear();
        request.worldbuilding.time_period.clear();
        request.worldbuilding.organizations.clear();
        request.worldbuilding.factions.clear();
        request.worldbuilding.systems.clear();
        request.worldbuilding.notes.clear();
        let error = create_new_book(&request).unwrap_err();
        assert!(error.contains("worldbuilding"));
        assert!(!root.exists());
    }

    #[test]
    fn rejects_relative_destination_before_writing() {
        let error = create_new_book(&input(Path::new("relative/book"))).unwrap_err();
        assert!(error.contains("absolute"));
    }

    #[test]
    fn duplicate_entity_names_get_distinct_normal_files() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Test Novel");
        let mut request = input(&root);
        request.characters.push(FoundationCharacter {
            name: "Test Character".into(),
            template_id: "character-profile".into(),
            role: "supporting".into(),
            aliases: vec![],
            description: String::new(),
        });

        create_new_book(&request).unwrap();

        assert!(root.join("Story Development/Characters/Main Cast/Test Character.md").is_file());
        assert!(root.join("Story Development/Characters/Main Cast/Test Character 2.md").is_file());
    }

    #[test]
    fn unsafe_filename_characters_are_sanitized_without_changing_title() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Test Novel");
        let mut request = input(&root);
        request.locations = vec![FoundationLocation {
            name: "Test: Location?".into(),
            template_id: "location-profile".into(),
            location_type: "site".into(),
            aliases: vec![],
            description: String::new(),
        }];

        create_new_book(&request).unwrap();

        let path = Path::new("Story Development/Worldbuilding & Settings/Locations/Test- Location-.md");
        assert!(root.join(path).is_file());
        let location = read_document(&root, path).unwrap();
        assert_eq!(location.title.as_deref(), Some("Test: Location?"));
    }

    #[test]
    fn decide_later_keeps_manuscript_folder_empty() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Test Novel");
        let mut request = input(&root);
        request.manuscript_template_id = "decide-later".into();

        create_new_book(&request).unwrap();

        let entries = std::fs::read_dir(root.join("Manuscript")).unwrap().count();
        assert_eq!(entries, 0);
    }

    #[test]
    fn selected_manuscript_template_controls_manuscript_folders() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Test Novel");
        let mut request = input(&root);
        request.manuscript_template_id = "bundled-story-circle".into();

        create_new_book(&request).unwrap();

        assert!(root.join("Manuscript/Act I — You - Need - Go - Search").is_dir());
        assert!(root.join("Manuscript/Act II — Find - Take - Return - Change").is_dir());
        assert!(!root.join("Manuscript/Act III — Resolution").exists());
    }

}
