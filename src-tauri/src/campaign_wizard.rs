//! File-authoritative New Campaign Wizard seeder.
//!
//! The campaign wizard is the tabletop counterpart of the New Book Foundation
//! (docs/architecture/016-NEW-BOOK-FOUNDATION.md) and follows the same rules:
//! setup state is temporary until Create Campaign; creation writes ordinary
//! Markdown/YAML files with stable Catalyst IDs inside a normal Catalyst
//! project root; those files are authoritative from that point forward. No
//! database rows are written.
//!
//! Both Quick Start and the ten-step Guided Setup call [`create_campaign`].
//! Quick Start simply sends fewer fields; the server fills defaults so a
//! Quick Start campaign is a complete, ordinary campaign folder.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::document::service::{create_document, save_document, CatalystDocument};
use crate::project_root::{initialize_project_root_from_manifest, ProjectManifest, ProjectPaths};

pub const CAMPAIGN_PROJECT_TYPE: &str = "campaign";

/// Campaign-specific folders created in addition to the shared Catalyst
/// project directories. They live beneath `Story Development` so the existing
/// Story Development wiki, file tree, search and Overview card work unchanged.
pub const CAMPAIGN_DIRECTORIES: &[&str] = &[
    "Story Development/Campaign",
    "Story Development/Adventures",
    "Story Development/Adventures/Arcs",
    "Story Development/Adventures/Quests & Hooks",
    "Story Development/Adventures/Encounters",
    "Story Development/Sessions",
    "Story Development/Sessions/Session Prep",
    "Story Development/Sessions/Session Logs",
    "Story Development/Characters/NPCs",
    "Story Development/Characters/Villains & Antagonists",
    "Story Development/Characters/Player Characters",
    "Story Development/Worldbuilding & Settings/Religions & Pantheons",
    "Story Development/Worldbuilding & Settings/History & Lore",
    "Story Development/Worldbuilding & Settings/Bestiary & Creatures",
    "Story Development/Worldbuilding & Settings/Magic & Technology",
    "Story Development/Worldbuilding & Settings/Languages",
    "Story Development/Game Master",
    "Story Development/Game Master/Templates",
    "Story Development/Game Master/Random Tables",
    "Story Development/Game Master/Handouts",
    "Assets/Handouts",
    "Assets/Tokens",
];

// ── Worldbuilding categories ────────────────────────────────────────────────

pub struct WorldCategoryDef {
    pub id: &'static str,
    pub label: &'static str,
    pub folder: &'static str,
    /// Core document type used for individual entries in this category.
    pub entry_type: &'static str,
    /// Default entity template recorded on each entry.
    pub entry_template: &'static str,
}

pub const WORLD_CATEGORIES: &[WorldCategoryDef] = &[
    WorldCategoryDef { id: "factions", label: "Factions & Governments", folder: "Governments & Factions", entry_type: "organization", entry_template: "faction-secret-society" },
    WorldCategoryDef { id: "organizations", label: "Organizations & Guilds", folder: "Organizations", entry_type: "organization", entry_template: "organization-profile" },
    WorldCategoryDef { id: "cultures", label: "Cultures & Peoples", folder: "Cultures & Societies", entry_type: "note", entry_template: "worldbuilding-profile" },
    WorldCategoryDef { id: "religions", label: "Religions & Pantheons", folder: "Religions & Pantheons", entry_type: "note", entry_template: "worldbuilding-profile" },
    WorldCategoryDef { id: "history", label: "History & Lore", folder: "History & Lore", entry_type: "event", entry_template: "event-profile" },
    WorldCategoryDef { id: "magic", label: "Magic & Technology", folder: "Magic & Technology", entry_type: "note", entry_template: "worldbuilding-profile" },
    WorldCategoryDef { id: "bestiary", label: "Bestiary & Creatures", folder: "Bestiary & Creatures", entry_type: "note", entry_template: "species-ancestry" },
    WorldCategoryDef { id: "items", label: "Items & Artifacts", folder: "Objects & Artifacts", entry_type: "object", entry_template: "object-item-profile" },
    WorldCategoryDef { id: "languages", label: "Languages", folder: "Languages", entry_type: "note", entry_template: "language-profile" },
];

fn world_category(id: &str) -> Option<&'static WorldCategoryDef> {
    WORLD_CATEGORIES.iter().find(|def| def.id == id)
}

// ── Game Master starter templates ───────────────────────────────────────────

pub struct GmTemplateDef {
    pub id: &'static str,
    pub title: &'static str,
    pub body: &'static str,
}

pub const GM_TEMPLATES: &[GmTemplateDef] = &[
    GmTemplateDef { id: "session-prep", title: "Session Prep", body: "## Recap\n\n## Strong Start\n\n## Potential Scenes\n\n- \n\n## Secrets & Clues\n\n- \n\n## NPCs\n\n## Locations\n\n## Encounters\n\n## Rewards\n" },
    GmTemplateDef { id: "session-log", title: "Session Log", body: "## Date & Attendance\n\n## Summary\n\n## Key Events\n\n- \n\n## Loot & Rewards\n\n## Open Threads\n\n## Notes for Next Session\n" },
    GmTemplateDef { id: "npc-card", title: "NPC Quick Card", body: "## Appearance\n\n## Personality & Mannerisms\n\n## Wants\n\n## Secret\n\n## Voice / Quote\n\n## Stats Reference\n" },
    GmTemplateDef { id: "location-card", title: "Location Card", body: "## First Impression\n\n## Notable Features\n\n## Inhabitants\n\n## Hooks & Secrets\n\n## Connections\n" },
    GmTemplateDef { id: "faction-card", title: "Faction Card", body: "## Goal\n\n## Methods\n\n## Leadership\n\n## Resources\n\n## Allies & Enemies\n\n## Clock / Progress\n" },
    GmTemplateDef { id: "quest-hook", title: "Quest / Adventure Hook", body: "## Hook\n\n## Patron\n\n## Objective\n\n## Complications\n\n## Rewards\n\n## Consequences of Failure\n" },
    GmTemplateDef { id: "encounter", title: "Encounter", body: "## Setup\n\n## Opposition\n\n## Terrain & Features\n\n## Tactics\n\n## Scaling\n\n## Outcome & Rewards\n" },
    GmTemplateDef { id: "loot-table", title: "Treasure & Loot", body: "| Roll | Reward |\n| --- | --- |\n| 1 | |\n| 2 | |\n| 3 | |\n| 4 | |\n| 5 | |\n| 6 | |\n" },
    GmTemplateDef { id: "random-table", title: "Random Table", body: "| d20 | Result |\n| --- | --- |\n| 1 | |\n| 2 | |\n| 3 | |\n| 4 | |\n| 5 | |\n| 6 | |\n| 7 | |\n| 8 | |\n| 9 | |\n| 10 | |\n| 11 | |\n| 12 | |\n| 13 | |\n| 14 | |\n| 15 | |\n| 16 | |\n| 17 | |\n| 18 | |\n| 19 | |\n| 20 | |\n" },
    GmTemplateDef { id: "handout", title: "Player Handout", body: "## Handout Text\n\n## When to Reveal\n\n## GM-Only Notes\n" },
];

fn gm_template(id: &str) -> Option<&'static GmTemplateDef> {
    GM_TEMPLATES.iter().find(|def| def.id == id)
}

// ── Campaign starter structures ─────────────────────────────────────────────

pub struct StarterDef {
    pub id: &'static str,
    pub name: &'static str,
    /// (title, body) arcs seeded into Adventures/Arcs.
    pub arcs: &'static [(&'static str, &'static str)],
}

pub const STARTERS: &[StarterDef] = &[
    StarterDef { id: "blank", name: "Blank Campaign", arcs: &[] },
    StarterDef {
        id: "adventure-path",
        name: "Adventure Path",
        arcs: &[
            ("Arc 1 — The Call", "## Premise\n\n## Key Locations\n\n## Key NPCs\n\n## Milestones\n\n## Climax\n"),
            ("Arc 2 — Rising Threat", "## Premise\n\n## Key Locations\n\n## Key NPCs\n\n## Milestones\n\n## Climax\n"),
            ("Arc 3 — The Reckoning", "## Premise\n\n## Key Locations\n\n## Key NPCs\n\n## Milestones\n\n## Climax\n"),
        ],
    },
    StarterDef {
        id: "sandbox",
        name: "Sandbox / Hexcrawl",
        arcs: &[("Region Overview", "## Starting Hub\n\n## Regions\n\n## Faction Clocks\n\n## Rumors\n\n- \n\n## Points of Interest\n")],
    },
    StarterDef {
        id: "mystery",
        name: "Mystery & Intrigue",
        arcs: &[("The Central Mystery", "## What Really Happened\n\n## Suspects\n\n## Clues (three per conclusion)\n\n- \n\n## Red Herrings\n\n## Revelation\n")],
    },
    StarterDef {
        id: "dungeon",
        name: "Dungeon Delve",
        arcs: &[("The Dungeon", "## History\n\n## Entrances\n\n## Levels\n\n## Inhabitants\n\n## Treasure\n\n## Wandering Monsters\n")],
    },
    StarterDef {
        id: "one-shot",
        name: "One-Shot",
        arcs: &[("One-Shot Adventure", "## Hook\n\n## Scene 1 — Opening\n\n## Scene 2 — Complication\n\n## Scene 3 — Climax\n\n## Wrap-Up\n")],
    },
];

fn starter(id: &str) -> Option<&'static StarterDef> {
    STARTERS.iter().find(|def| def.id == id)
}

// ── Input ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CampaignTable {
    #[serde(default)]
    pub player_count: Option<u32>,
    #[serde(default)]
    pub starting_level: String,
    #[serde(default)]
    pub session_length: String,
    #[serde(default)]
    pub cadence: String,
    #[serde(default)]
    pub play_mode: String,
    #[serde(default)]
    pub safety_tools: Vec<String>,
    #[serde(default)]
    pub lines: Vec<String>,
    #[serde(default)]
    pub veils: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CampaignWorld {
    #[serde(default)]
    pub setting: String,
    #[serde(default)]
    pub era: String,
    #[serde(default)]
    pub magic_level: String,
    #[serde(default)]
    pub technology_level: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CampaignWorldCategory {
    pub id: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub entries: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CampaignLocation {
    pub name: String,
    #[serde(default)]
    pub location_type: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CampaignNpc {
    pub name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub affiliation: String,
    #[serde(default)]
    pub description: String,
    /// Villains and antagonists are filed separately for quick GM access.
    #[serde(default)]
    pub antagonist: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CampaignStory {
    #[serde(default)]
    pub central_conflict: String,
    #[serde(default)]
    pub main_threat: String,
    #[serde(default)]
    pub themes: Vec<String>,
    #[serde(default)]
    pub hooks: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CampaignFirstSession {
    #[serde(default)]
    pub session_zero_notes: String,
    #[serde(default)]
    pub opening_scene: String,
    #[serde(default)]
    pub session_one_notes: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewCampaignInput {
    pub project_root: String,
    pub name: String,
    /// "quick-start" or "guided"; recorded for provenance only.
    #[serde(default = "default_setup_mode")]
    pub setup_mode: String,
    #[serde(default)]
    pub game_system: String,
    #[serde(default)]
    pub edition: String,
    #[serde(default)]
    pub genre: String,
    #[serde(default)]
    pub tone: String,
    #[serde(default)]
    pub pitch: String,
    #[serde(default)]
    pub table: CampaignTable,
    #[serde(default)]
    pub world: CampaignWorld,
    #[serde(default)]
    pub world_categories: Vec<CampaignWorldCategory>,
    #[serde(default)]
    pub locations: Vec<CampaignLocation>,
    #[serde(default)]
    pub npcs: Vec<CampaignNpc>,
    #[serde(default)]
    pub story: CampaignStory,
    #[serde(default)]
    pub first_session: CampaignFirstSession,
    #[serde(default = "default_starter")]
    pub starter_id: String,
    /// GM starter templates to copy into Game Master/Templates. `None`
    /// (Quick Start) selects all of them.
    #[serde(default)]
    pub gm_template_ids: Option<Vec<String>>,
    #[serde(default)]
    pub house_rules: String,
}

fn default_setup_mode() -> String {
    "guided".into()
}

fn default_starter() -> String {
    "blank".into()
}

#[derive(Clone, Debug, Serialize)]
pub struct NewCampaignProject {
    pub manifest: ProjectManifest,
    pub paths: ProjectPaths,
    /// Project-relative path of the Campaign Overview document to open first.
    pub overview_path: String,
    /// Stable ID of the Campaign Overview document.
    pub overview_id: String,
}

// ── Creation ────────────────────────────────────────────────────────────────

pub fn create_campaign(input: &NewCampaignInput) -> Result<NewCampaignProject, String> {
    let root = PathBuf::from(input.project_root.trim());
    if !root.is_absolute() {
        return Err("Campaign destination must be an absolute local folder path.".into());
    }
    let name = input.name.trim();
    if name.is_empty() {
        return Err("Campaign name must not be empty.".into());
    }
    validate(input)?;

    // Refuse a non-empty destination before writing anything. The shared
    // initializer would accept an existing Catalyst project; a new campaign
    // must never seed files into somebody else's project.
    if root.exists() {
        if !root.is_dir() {
            return Err("Campaign destination points to a file, not a folder.".into());
        }
        let has_entries = fs::read_dir(&root)
            .map_err(|e| format!("Could not inspect campaign destination: {e}"))?
            .next()
            .is_some();
        if has_entries {
            return Err("Choose an empty folder for the new campaign.".into());
        }
    }

    let mut manifest = ProjectManifest::new(name, CAMPAIGN_PROJECT_TYPE);
    manifest.genre = non_empty(&input.genre);
    manifest.description = non_empty(&input.pitch);
    let paths = initialize_project_root_from_manifest(&root, &manifest)?;
    for relative in CAMPAIGN_DIRECTORIES {
        fs::create_dir_all(root.join(relative))
            .map_err(|e| format!("Could not create campaign folder '{relative}': {e}"))?;
    }

    let overview = seed_campaign(&root, &manifest, input)?;

    Ok(NewCampaignProject {
        manifest,
        paths,
        overview_path: overview.relative_path.to_string_lossy().replace('\\', "/"),
        overview_id: overview.identity.id,
    })
}

fn validate(input: &NewCampaignInput) -> Result<(), String> {
    if input.locations.iter().any(|l| l.name.trim().is_empty()) {
        return Err("Location names must not be empty.".into());
    }
    if input.npcs.iter().any(|n| n.name.trim().is_empty()) {
        return Err("NPC names must not be empty.".into());
    }
    if starter(input.starter_id.trim()).is_none() {
        return Err(format!("Unknown campaign starter '{}'.", input.starter_id));
    }
    for category in &input.world_categories {
        if world_category(category.id.trim()).is_none() {
            return Err(format!("Unknown worldbuilding category '{}'.", category.id));
        }
    }
    if let Some(ids) = &input.gm_template_ids {
        for id in ids {
            if gm_template(id.trim()).is_none() {
                return Err(format!("Unknown Game Master template '{id}'."));
            }
        }
    }
    if let Some(count) = input.table.player_count {
        if count == 0 || count > 50 {
            return Err("Player count must be between 1 and 50.".into());
        }
    }
    Ok(())
}

fn seed_campaign(
    root: &Path,
    manifest: &ProjectManifest,
    input: &NewCampaignInput,
) -> Result<CatalystDocument, String> {
    let starter = starter(input.starter_id.trim()).expect("validated starter");

    // Campaign Overview: the human-facing metadata document.
    let mut overview = create_document(
        root,
        Path::new("Story Development/Campaign/Campaign Overview.md"),
        "note",
        Some(input.name.trim()),
        &input.pitch,
    )?;
    {
        let m = &mut overview.metadata;
        insert_string(m, "campaign_kind", "overview");
        insert_string(m, "campaign_id", &manifest.project_id);
        insert_string(m, "setup_mode", &input.setup_mode);
        insert_string(m, "game_system", &input.game_system);
        insert_string(m, "edition", &input.edition);
        insert_string(m, "genre", &input.genre);
        insert_string(m, "tone", &input.tone);
        insert_string(m, "starter", starter.id);
        if let Some(count) = input.table.player_count {
            m.insert("player_count".into(), Value::Number(count.into()));
        }
        insert_string(m, "starting_level", &input.table.starting_level);
        insert_string(m, "session_length", &input.table.session_length);
        insert_string(m, "cadence", &input.table.cadence);
        insert_string(m, "play_mode", &input.table.play_mode);
        insert_strings(m, "themes", &input.story.themes);
        insert_string(m, "central_conflict", &input.story.central_conflict);
        insert_string(m, "main_threat", &input.story.main_threat);
        insert_string(m, "status", "planning");
    }
    save_document(root, &overview, Some(&overview.source_hash))?;

    // Table & safety.
    let mut table = create_document(
        root,
        Path::new("Story Development/Campaign/Table & Safety.md"),
        "note",
        Some("Table & Safety"),
        "## Expectations\n\n## Scheduling\n\n## Safety Notes\n",
    )?;
    insert_string(&mut table.metadata, "campaign_kind", "table");
    insert_strings(&mut table.metadata, "safety_tools", &input.table.safety_tools);
    insert_strings(&mut table.metadata, "lines", &input.table.lines);
    insert_strings(&mut table.metadata, "veils", &input.table.veils);
    save_document(root, &table, Some(&table.source_hash))?;

    // House rules.
    let rules_body = if input.house_rules.trim().is_empty() {
        "## Rules Variants\n\n## Character Creation\n\n## Rests, Death & Downtime\n"
    } else {
        input.house_rules.as_str()
    };
    let mut rules = create_document(
        root,
        Path::new("Story Development/Campaign/House Rules.md"),
        "note",
        Some("House Rules"),
        rules_body,
    )?;
    insert_string(&mut rules.metadata, "campaign_kind", "house-rules");
    save_document(root, &rules, Some(&rules.source_hash))?;

    // World overview (always seeded so the world has a home).
    let world_body = if input.world.notes.trim().is_empty() {
        "## The World at a Glance\n\n## What Everyone Knows\n\n## What Few Know\n"
    } else {
        input.world.notes.as_str()
    };
    let mut world = create_document(
        root,
        Path::new("Story Development/Worldbuilding & Settings/Overview.md"),
        "note",
        Some("World Overview"),
        world_body,
    )?;
    insert_string(&mut world.metadata, "template_id", "worldbuilding-profile");
    insert_string(&mut world.metadata, "campaign_kind", "world");
    insert_string(&mut world.metadata, "setting", &input.world.setting);
    insert_string(&mut world.metadata, "era", &input.world.era);
    insert_string(&mut world.metadata, "magic_level", &input.world.magic_level);
    insert_string(&mut world.metadata, "technology_level", &input.world.technology_level);
    save_document(root, &world, Some(&world.source_hash))?;

    seed_world_categories(root, &input.world_categories)?;
    seed_locations(root, &input.locations)?;
    seed_npcs(root, &input.npcs)?;
    seed_player_characters_placeholder(root)?;
    seed_story(root, starter, &input.story)?;
    seed_sessions(root, &input.first_session)?;
    seed_gm_templates(root, input.gm_template_ids.as_deref())?;

    Ok(overview)
}

fn seed_world_categories(root: &Path, categories: &[CampaignWorldCategory]) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    for category in categories {
        let def = world_category(category.id.trim()).expect("validated category");
        if !seen.insert(def.id) {
            continue;
        }
        let folder = format!("Story Development/Worldbuilding & Settings/{}", def.folder);
        let notes = category.notes.trim();
        let mut overview = create_document(
            root,
            &Path::new(&folder).join(format!("{} Overview.md", safe_filename(def.label))),
            "note",
            Some(def.label),
            if notes.is_empty() { "## Overview\n\n## Details\n" } else { notes },
        )?;
        insert_string(&mut overview.metadata, "campaign_kind", "world-category");
        insert_string(&mut overview.metadata, "category", def.id);
        insert_string(&mut overview.metadata, "template_id", "worldbuilding-profile");
        save_document(root, &overview, Some(&overview.source_hash))?;

        let mut used = BTreeMap::new();
        // The overview occupies its filename; reserve it so entries never collide.
        used.insert(format!("{} Overview", safe_filename(def.label)), 1);
        for entry in category.entries.iter().map(|e| e.trim()).filter(|e| !e.is_empty()) {
            let relative = unique_entity_path(&folder, entry, &mut used);
            let mut document = create_document(root, &relative, def.entry_type, Some(entry), "")?;
            insert_string(&mut document.metadata, "template_id", def.entry_template);
            insert_string(&mut document.metadata, "category", def.id);
            save_document(root, &document, Some(&document.source_hash))?;
        }
    }
    Ok(())
}

fn seed_locations(root: &Path, locations: &[CampaignLocation]) -> Result<(), String> {
    let mut used = BTreeMap::new();
    for location in locations {
        let relative = unique_entity_path(
            "Story Development/Worldbuilding & Settings/Locations",
            &location.name,
            &mut used,
        );
        let mut document = create_document(root, &relative, "location", Some(location.name.trim()), &location.description)?;
        insert_string(&mut document.metadata, "template_id", "location-profile");
        insert_string(&mut document.metadata, "location_type", &location.location_type);
        save_document(root, &document, Some(&document.source_hash))?;
    }
    Ok(())
}

fn seed_npcs(root: &Path, npcs: &[CampaignNpc]) -> Result<(), String> {
    let mut used = BTreeMap::new();
    for npc in npcs {
        let folder = if npc.antagonist {
            "Story Development/Characters/Villains & Antagonists"
        } else {
            "Story Development/Characters/NPCs"
        };
        let relative = unique_entity_path(folder, &npc.name, &mut used);
        let mut document = create_document(root, &relative, "character", Some(npc.name.trim()), &npc.description)?;
        insert_string(&mut document.metadata, "template_id", "character-profile");
        insert_string(&mut document.metadata, "character_kind", if npc.antagonist { "antagonist" } else { "npc" });
        insert_string(&mut document.metadata, "role", &npc.role);
        insert_string(&mut document.metadata, "affiliation", &npc.affiliation);
        save_document(root, &document, Some(&document.source_hash))?;
    }
    Ok(())
}

/// Player characters belong to the upcoming Character Studio. Until it ships
/// the folder carries a plain README so the GM knows where PCs will live. A
/// README without Catalyst frontmatter is ignored by the Story Development
/// index, so it never shows up as a broken document.
fn seed_player_characters_placeholder(root: &Path) -> Result<(), String> {
    let path = root.join("Story Development/Characters/Player Characters/README.md");
    fs::write(
        &path,
        "# Player Characters\n\nCharacter Studio (coming soon) will create player characters here.\n\
         Until then you can add PCs as ordinary character documents in this folder.\n",
    )
    .map_err(|e| format!("Could not write Player Characters placeholder: {e}"))
}

fn seed_story(root: &Path, starter: &StarterDef, story: &CampaignStory) -> Result<(), String> {
    for (title, body) in starter.arcs {
        let relative = Path::new("Story Development/Adventures/Arcs").join(format!("{}.md", safe_filename(title)));
        let mut arc = create_document(root, &relative, "plot-thread", Some(title), body)?;
        insert_string(&mut arc.metadata, "campaign_kind", "arc");
        insert_string(&mut arc.metadata, "status", "planned");
        save_document(root, &arc, Some(&arc.source_hash))?;
    }

    let mut used = BTreeMap::new();
    for hook in story.hooks.iter().map(|h| h.trim()).filter(|h| !h.is_empty()) {
        let title = truncate_title(hook);
        let relative = unique_entity_path("Story Development/Adventures/Quests & Hooks", &title, &mut used);
        let mut doc = create_document(root, &relative, "plot-thread", Some(&title), hook)?;
        insert_string(&mut doc.metadata, "campaign_kind", "hook");
        insert_string(&mut doc.metadata, "status", "available");
        save_document(root, &doc, Some(&doc.source_hash))?;
    }
    Ok(())
}

fn seed_sessions(root: &Path, first: &CampaignFirstSession) -> Result<(), String> {
    let zero_body = if first.session_zero_notes.trim().is_empty() {
        "## Pitch & Expectations\n\n## Character Creation\n\n## Party Ties\n\n## Safety Tools\n\n## Scheduling\n"
    } else {
        first.session_zero_notes.as_str()
    };
    let mut zero = create_document(
        root,
        Path::new("Story Development/Sessions/Session Prep/Session 00 — Session Zero.md"),
        "note",
        Some("Session 0 — Session Zero"),
        zero_body,
    )?;
    insert_string(&mut zero.metadata, "campaign_kind", "session-prep");
    zero.metadata.insert("session_number".into(), Value::Number(0.into()));
    insert_string(&mut zero.metadata, "status", "planned");
    save_document(root, &zero, Some(&zero.source_hash))?;

    let mut body = String::from("## Strong Start\n\n");
    if !first.opening_scene.trim().is_empty() {
        body.push_str(first.opening_scene.trim());
        body.push_str("\n\n");
    }
    body.push_str("## Potential Scenes\n\n## Secrets & Clues\n\n## Notes\n\n");
    if !first.session_one_notes.trim().is_empty() {
        body.push_str(first.session_one_notes.trim());
        body.push('\n');
    }
    let mut one = create_document(
        root,
        Path::new("Story Development/Sessions/Session Prep/Session 01.md"),
        "note",
        Some("Session 1"),
        &body,
    )?;
    insert_string(&mut one.metadata, "campaign_kind", "session-prep");
    one.metadata.insert("session_number".into(), Value::Number(1.into()));
    insert_string(&mut one.metadata, "status", "planned");
    save_document(root, &one, Some(&one.source_hash))?;
    Ok(())
}

fn seed_gm_templates(root: &Path, ids: Option<&[String]>) -> Result<(), String> {
    let selected: Vec<&GmTemplateDef> = match ids {
        None => GM_TEMPLATES.iter().collect(),
        Some(ids) => GM_TEMPLATES
            .iter()
            .filter(|def| ids.iter().any(|id| id.trim() == def.id))
            .collect(),
    };
    for def in selected {
        let relative = Path::new("Story Development/Game Master/Templates").join(format!("{}.md", safe_filename(def.title)));
        let mut doc = create_document(root, &relative, "template", Some(def.title), def.body)?;
        insert_string(&mut doc.metadata, "campaign_kind", "gm-template");
        insert_string(&mut doc.metadata, "gm_template_id", def.id);
        save_document(root, &doc, Some(&doc.source_hash))?;
    }
    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn truncate_title(text: &str) -> String {
    let first_line = text.lines().next().unwrap_or("").trim();
    let mut title: String = first_line.chars().take(60).collect();
    if first_line.chars().count() > 60 {
        title = title.trim_end().to_string();
        title.push('…');
    }
    if title.is_empty() {
        "Hook".into()
    } else {
        title
    }
}

fn non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn unique_entity_path(directory: &str, name: &str, used: &mut BTreeMap<String, usize>) -> PathBuf {
    let stem = safe_filename(name);
    let count = used.entry(stem.to_lowercase()).or_insert(0);
    *count += 1;
    let filename = if *count == 1 { format!("{stem}.md") } else { format!("{stem} {}.md", *count) };
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
    let out = out.trim().trim_end_matches(['.', ' ']).trim();
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

fn insert_strings(metadata: &mut BTreeMap<String, Value>, key: &str, values: &[String]) {
    let values: Vec<Value> = values
        .iter()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .map(|v| Value::String(v.into()))
        .collect();
    if !values.is_empty() {
        metadata.insert(key.into(), Value::Sequence(values));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::service::read_document;
    use crate::project_discovery::project_from_root;
    use crate::project_root::read_project_manifest;
    use crate::story_development::build_index;
    use std::collections::HashSet;

    fn quick_start(root: &Path) -> NewCampaignInput {
        serde_json::from_value(serde_json::json!({
            "project_root": root.to_string_lossy(),
            "name": "Ashes of Varn",
            "setup_mode": "quick-start",
            "game_system": "Fifth Edition Fantasy",
            "pitch": "A border kingdom burns.",
            "starter_id": "adventure-path",
        }))
        .unwrap()
    }

    fn guided(root: &Path) -> NewCampaignInput {
        serde_json::from_value(serde_json::json!({
            "project_root": root.to_string_lossy(),
            "name": "The Drowned Crown",
            "setup_mode": "guided",
            "game_system": "Generic d20",
            "edition": "2nd",
            "genre": "Nautical fantasy",
            "tone": "Swashbuckling",
            "pitch": "Pirates hunt a sunken throne.",
            "table": {
                "player_count": 4, "starting_level": "3", "session_length": "4 hours",
                "cadence": "Weekly", "play_mode": "In person",
                "safety_tools": ["Lines & Veils", "X-Card"], "lines": ["Harm to children"], "veils": []
            },
            "world": { "setting": "The Shattered Isles", "era": "Age of Sail", "magic_level": "Low", "notes": "Islands drift." },
            "world_categories": [
                { "id": "factions", "notes": "Pirate lords vie.", "entries": ["The Tide Court", "Iron Navy", "The Tide Court"] },
                { "id": "religions", "notes": "", "entries": [] },
                { "id": "items", "notes": "", "entries": ["Crown of Depths"] }
            ],
            "locations": [ { "name": "Port: Saltmere?", "location_type": "port town", "description": "Rough harbour." } ],
            "npcs": [
                { "name": "Captain Mara", "role": "patron", "affiliation": "Iron Navy", "description": "Stern." },
                { "name": "The Drowned King", "role": "villain", "antagonist": true }
            ],
            "story": { "central_conflict": "Who claims the crown", "main_threat": "The Drowned King", "themes": ["Greed"], "hooks": ["A map washes ashore", "A navy press gang"] },
            "first_session": { "session_zero_notes": "Discuss tone.", "opening_scene": "A storm at sea.", "session_one_notes": "" },
            "starter_id": "sandbox",
            "gm_template_ids": ["session-prep", "npc-card"],
            "house_rules": "Flanking grants advantage."
        }))
        .unwrap()
    }

    #[test]
    fn quick_start_creates_complete_campaign_folder() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Ashes of Varn");
        let created = create_campaign(&quick_start(&root)).unwrap();

        let manifest = read_project_manifest(&root).unwrap();
        assert_eq!(manifest, created.manifest);
        assert_eq!(manifest.project_type, "campaign");
        assert_eq!(manifest.description.as_deref(), Some("A border kingdom burns."));
        for dir in CAMPAIGN_DIRECTORIES {
            assert!(root.join(dir).is_dir(), "missing campaign folder {dir}");
        }
        // Shared Catalyst folders remain so existing modules keep working.
        assert!(root.join("Manuscript").is_dir());
        assert!(root.join("Story Development/Characters/Main Cast").is_dir());

        assert_eq!(created.overview_path, "Story Development/Campaign/Campaign Overview.md");
        let overview = read_document(&root, Path::new(&created.overview_path)).unwrap();
        assert_eq!(overview.identity.id, created.overview_id);
        assert!(overview.identity.id.starts_with("note-"));
        assert_eq!(overview.title.as_deref(), Some("Ashes of Varn"));
        assert_eq!(overview.metadata.get("campaign_id"), Some(&Value::String(manifest.project_id.clone())));
        assert_eq!(overview.metadata.get("setup_mode"), Some(&Value::String("quick-start".into())));
        assert_eq!(overview.metadata.get("game_system"), Some(&Value::String("Fifth Edition Fantasy".into())));

        assert!(root.join("Story Development/Adventures/Arcs/Arc 1 — The Call.md").is_file());
        assert!(root.join("Story Development/Sessions/Session Prep/Session 00 — Session Zero.md").is_file());
        assert!(root.join("Story Development/Sessions/Session Prep/Session 01.md").is_file());
        // Quick Start installs every GM starter template.
        let templates = fs::read_dir(root.join("Story Development/Game Master/Templates")).unwrap().count();
        assert_eq!(templates, GM_TEMPLATES.len());
        assert!(root.join("Story Development/Characters/Player Characters/README.md").is_file());

        let index = build_index(&root).unwrap();
        assert!(index.issues.is_empty(), "index issues: {:?}", index.issues);
    }

    #[test]
    fn guided_setup_seeds_entities_categories_and_metadata() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Drowned Crown");
        create_campaign(&guided(&root)).unwrap();

        let overview = read_document(&root, Path::new("Story Development/Campaign/Campaign Overview.md")).unwrap();
        assert_eq!(overview.metadata.get("player_count"), Some(&Value::Number(4.into())));
        assert_eq!(overview.metadata.get("starter"), Some(&Value::String("sandbox".into())));
        let table = read_document(&root, Path::new("Story Development/Campaign/Table & Safety.md")).unwrap();
        assert_eq!(
            table.metadata.get("safety_tools"),
            Some(&Value::Sequence(vec![Value::String("Lines & Veils".into()), Value::String("X-Card".into())]))
        );
        let rules = read_document(&root, Path::new("Story Development/Campaign/House Rules.md")).unwrap();
        assert_eq!(rules.body, "Flanking grants advantage.\n");

        let wb = "Story Development/Worldbuilding & Settings";
        assert!(root.join(format!("{wb}/Governments & Factions/Factions & Governments Overview.md")).is_file());
        let court = read_document(&root, Path::new(&format!("{wb}/Governments & Factions/The Tide Court.md"))).unwrap();
        assert!(court.identity.id.starts_with("org-"));
        assert_eq!(court.metadata.get("template_id"), Some(&Value::String("faction-secret-society".into())));
        assert!(root.join(format!("{wb}/Governments & Factions/The Tide Court 2.md")).is_file());
        assert!(root.join(format!("{wb}/Religions & Pantheons/Religions & Pantheons Overview.md")).is_file());
        let crown = read_document(&root, Path::new(&format!("{wb}/Objects & Artifacts/Crown of Depths.md"))).unwrap();
        assert!(crown.identity.id.starts_with("obj-"));
        // Unselected categories are not seeded with files.
        assert_eq!(fs::read_dir(root.join(format!("{wb}/Bestiary & Creatures"))).unwrap().count(), 0);

        let port = read_document(&root, Path::new(&format!("{wb}/Locations/Port- Saltmere-.md"))).unwrap();
        assert_eq!(port.title.as_deref(), Some("Port: Saltmere?"));
        assert!(port.identity.id.starts_with("loc-"));

        let mara = read_document(&root, Path::new("Story Development/Characters/NPCs/Captain Mara.md")).unwrap();
        assert!(mara.identity.id.starts_with("chr-"));
        assert_eq!(mara.metadata.get("affiliation"), Some(&Value::String("Iron Navy".into())));
        assert!(root.join("Story Development/Characters/Villains & Antagonists/The Drowned King.md").is_file());

        assert!(root.join("Story Development/Adventures/Arcs/Region Overview.md").is_file());
        assert!(root.join("Story Development/Adventures/Quests & Hooks/A map washes ashore.md").is_file());
        let one = read_document(&root, Path::new("Story Development/Sessions/Session Prep/Session 01.md")).unwrap();
        assert!(one.body.contains("A storm at sea."));

        let templates: HashSet<_> = fs::read_dir(root.join("Story Development/Game Master/Templates"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(templates, HashSet::from(["Session Prep.md".to_string(), "NPC Quick Card.md".to_string()]));
    }

    #[test]
    fn every_seeded_document_has_a_unique_stable_id_that_survives_reopen() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Stable");
        let created = create_campaign(&guided(&root)).unwrap();

        let index = build_index(&root).unwrap();
        assert!(index.issues.is_empty(), "index issues: {:?}", index.issues);
        assert!(index.entries.len() >= 20);
        let mut ids = HashSet::new();
        for entry in &index.entries {
            assert!(ids.insert(entry.id.clone()), "duplicate id {}", entry.id);
            let doc = read_document(&root, Path::new(&entry.relative_path)).unwrap();
            assert_eq!(doc.identity.id, entry.id);
        }

        // Edit, save, and reopen purely from disk.
        let path = Path::new("Story Development/Characters/NPCs/Captain Mara.md");
        let mut doc = read_document(&root, path).unwrap();
        let id = doc.identity.id.clone();
        let hash = doc.source_hash.clone();
        doc.body = "Edited by the GM.\n".into();
        save_document(&root, &doc, Some(&hash)).unwrap();

        let reopened = project_from_root(&root).unwrap();
        assert_eq!(reopened.id.to_string(), created.manifest.project_id);
        assert_eq!(reopened.project_type, "campaign");
        let again = read_document(&root, path).unwrap();
        assert_eq!(again.identity.id, id);
        assert_eq!(again.body, "Edited by the GM.\n");
        let rebuilt = build_index(&root).unwrap();
        assert_eq!(rebuilt.entries, index.entries);
    }

    #[test]
    fn refuses_non_empty_or_existing_project_destination() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Taken");
        create_campaign(&quick_start(&root)).unwrap();
        let before = read_project_manifest(&root).unwrap();
        let error = create_campaign(&quick_start(&root)).unwrap_err();
        assert!(error.contains("empty folder"), "{error}");
        assert_eq!(read_project_manifest(&root).unwrap(), before);

        let other = parent.path().join("Has files");
        fs::create_dir_all(&other).unwrap();
        fs::write(other.join("notes.txt"), "keep me").unwrap();
        assert!(create_campaign(&quick_start(&other)).is_err());
        assert!(!other.join("project.yaml").exists());
    }

    #[test]
    fn validation_rejects_bad_input_before_writing() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Bad");

        let mut input = quick_start(&root);
        input.name = "  ".into();
        assert!(create_campaign(&input).unwrap_err().contains("name"));

        let mut input = quick_start(&root);
        input.npcs.push(CampaignNpc::default());
        assert!(create_campaign(&input).unwrap_err().contains("NPC"));

        let mut input = quick_start(&root);
        input.starter_id = "nope".into();
        assert!(create_campaign(&input).is_err());

        let mut input = quick_start(&root);
        input.world_categories.push(CampaignWorldCategory { id: "nope".into(), ..Default::default() });
        assert!(create_campaign(&input).is_err());

        let mut input = quick_start(&root);
        input.gm_template_ids = Some(vec!["nope".into()]);
        assert!(create_campaign(&input).is_err());

        let mut input = quick_start(&root);
        input.table.player_count = Some(0);
        assert!(create_campaign(&input).is_err());

        assert!(!root.exists());
        assert!(create_campaign(&quick_start(Path::new("relative/campaign"))).unwrap_err().contains("absolute"));
    }

    #[test]
    fn existing_novel_projects_still_open_alongside_campaigns() {
        let parent = tempfile::tempdir().unwrap();
        let novel_root = parent.path().join("Novel");
        let manifest = ProjectManifest::new("Old Book", "novel");
        initialize_project_root_from_manifest(&novel_root, &manifest).unwrap();
        let campaign_root = parent.path().join("Campaign");
        create_campaign(&quick_start(&campaign_root)).unwrap();

        let novel = project_from_root(&novel_root).unwrap();
        assert_eq!(novel.project_type, "novel");
        assert!(!novel_root.join("Story Development/Campaign").exists());
        assert!(build_index(&novel_root).unwrap().issues.is_empty());
    }
}
