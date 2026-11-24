//! File-authoritative New Campaign Wizard seeder.
//!
//! Implements docs/architecture/020-CAMPAIGN-WIZARD-ARCHITECTURE.md (approved
//! baseline). The wizard is a temporary seeder: nothing is written until
//! Create, and afterwards the generated files are authoritative. Quick Start
//! and Guided Setup produce the same manifests and the same full directory
//! tree; Quick Start just supplies fewer optional values.
//!
//! Every seeded record is created the same way the wiki's New Document command
//! creates one: it carries a registered entity `template_id`, the template's
//! subtype, every template field (including the image field) and the template's
//! Markdown scaffold, so it renders in the wiki's right-hand Overview panel.
//!
//! Creation is staged: the campaign is built in a unique sibling directory on
//! the same filesystem, verified, then published with a rename, so a partially
//! initialised campaign is never visible at the destination.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use uuid::Uuid;

use crate::document::service::{create_document, save_document, CatalystDocument};
use crate::entity_templates::built_in_template;
use crate::project_root::{
    initialize_project_root_with_directories, read_project_manifest, ProjectManifest, ProjectPaths,
};

pub const CAMPAIGN_PROJECT_TYPE: &str = "campaign";
pub const CAMPAIGN_MANIFEST_FILENAME: &str = "campaign.yaml";
pub const CAMPAIGN_SCHEMA_VERSION: u32 = 1;
pub const CAMPAIGN_STATE_PATH: &str = "Sessions/Campaign State/";

/// The approved physical campaign layout (020 §4), created for both Quick
/// Start and Guided Setup even when empty. One documented addition:
/// `Sessions/Roll Logs`, where the global dice roller keeps roll history.
pub const CAMPAIGN_DIRECTORIES: &[&str] = &[
    "Worldbuilding",
    "Worldbuilding/Geography & Climate",
    "Worldbuilding/Governments & Politics",
    "Worldbuilding/Kingdoms & Nations",
    "Worldbuilding/Settlements & Locations",
    "Worldbuilding/History & Timelines",
    "Worldbuilding/Lore & Legends",
    "Worldbuilding/Cultures & Species",
    "Worldbuilding/Religions & Deities",
    "Worldbuilding/Magic & Supernatural",
    "Worldbuilding/Languages",
    "Worldbuilding/Economy & Trade",
    "Worldbuilding/Technology & Crafting",
    "Worldbuilding/Organizations & Factions",
    "Worldbuilding/Laws & Customs",
    "Worldbuilding/Natural World",
    "Characters",
    "Characters/Player Characters",
    "Characters/Non-Player Characters",
    "Characters/Character Templates",
    "Characters/Portraits & Tokens",
    "Creatures & Monsters",
    "Creatures & Monsters/Bestiary",
    "Creatures & Monsters/Custom Creatures",
    "Creatures & Monsters/Encounter Groups",
    "Adventures",
    "Adventures/Main Campaign",
    "Adventures/Side Quests",
    "Adventures/Quest Chains",
    "Adventures/Scenes",
    "Adventures/Puzzles & Traps",
    "Encounters",
    "Encounters/Combat",
    "Encounters/Social",
    "Encounters/Exploration",
    "Encounters/Random Tables",
    "Maps",
    "Maps/World",
    "Maps/Regional",
    "Maps/Settlements",
    "Maps/Dungeons",
    "Maps/Battle Maps",
    "Items & Equipment",
    "Items & Equipment/Weapons & Armor",
    "Items & Equipment/Magic Items",
    "Items & Equipment/Consumables",
    "Items & Equipment/Treasure & Loot",
    "Items & Equipment/Crafting",
    "Rules & Mechanics",
    "Rules & Mechanics/Rules Packs",
    "Rules & Mechanics/House Rules",
    "Rules & Mechanics/Conditions",
    "Rules & Mechanics/Spells & Abilities",
    "Sessions",
    "Sessions/Session Plans",
    "Sessions/Session Logs",
    "Sessions/Recaps",
    "Sessions/Campaign State",
    "Sessions/Roll Logs",
    "Game Master",
    "Game Master/Secrets & Clues",
    "Game Master/GM Notes",
    "Game Master/Random Tables",
    "Game Master/Handouts",
    "Game Master/Screen References",
    "Game Master/Guides",
    "Assets",
    "Assets/Images",
    "Assets/Audio",
    "Assets/Map Textures",
    "Assets/Tokens",
    "Exports & Print",
    "Exports & Print/Character Sheets",
    "Exports & Print/Maps",
    "Exports & Print/Handouts",
    "Exports & Print/Campaign Reports",
];

/// Campaign top-level folders that hold wiki documents. The Story Development
/// index, folder picker and New Document/folder commands accept these in
/// addition to the book roots (the documented adapter from 020 §4), so the
/// inherited wiki works for campaigns while book projects keep their layout.
/// `Assets` is already read as a shared image root; `Exports & Print` only
/// holds generated output.
pub const CAMPAIGN_ROOTS: &[&str] = &[
    "Worldbuilding",
    "Characters",
    "Creatures & Monsters",
    "Adventures",
    "Encounters",
    "Maps",
    "Items & Equipment",
    "Rules & Mechanics",
    "Sessions",
    "Game Master",
];

/// Rules packs installed with Catalyst. No D&D-only assumptions; a licensed
/// 5e-compatible pack may be added once content rights are verified.
pub const RULES_PACKS: &[(&str, &str)] = &[("system-neutral", "1"), ("custom-homebrew", "1")];

fn rules_pack_version(id: &str) -> Option<&'static str> {
    RULES_PACKS
        .iter()
        .find(|(pack, _)| *pack == id)
        .map(|(_, v)| *v)
}

/// Worldbuilding families (020 §6): wizard id, folder, entity template.
pub const WORLD_CATEGORIES: &[(&str, &str, &str)] = &[
    ("geography", "Geography & Climate", "geography-landmark"),
    ("government", "Governments & Politics", "government-profile"),
    ("nations", "Kingdoms & Nations", "region-territory"),
    ("settlements", "Settlements & Locations", "settlement-city"),
    ("history", "History & Timelines", "event-profile"),
    ("lore", "Lore & Legends", "worldbuilding-profile"),
    ("cultures", "Cultures & Species", "species-ancestry"),
    ("religions", "Religions & Deities", "worldbuilding-profile"),
    ("magic", "Magic & Supernatural", "worldbuilding-profile"),
    ("languages", "Languages", "language-profile"),
    ("economy", "Economy & Trade", "worldbuilding-profile"),
    (
        "technology",
        "Technology & Crafting",
        "worldbuilding-profile",
    ),
    (
        "factions",
        "Organizations & Factions",
        "faction-secret-society",
    ),
    ("laws", "Laws & Customs", "worldbuilding-profile"),
    ("natural", "Natural World", "worldbuilding-profile"),
];

fn world_category(id: &str) -> Option<(&'static str, &'static str, &'static str)> {
    WORLD_CATEGORIES
        .iter()
        .copied()
        .find(|(cid, _, _)| *cid == id)
}

// ── Input ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct CampaignBasics {
    pub subtitle: String,
    pub premise: String,
    pub genre: String,
    pub tone: String,
    pub gm_name: String,
    pub start_date: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct CampaignRules {
    pub pack_id: String,
    pub custom_system_name: String,
    pub advancement: String,
    pub starting_level: String,
    pub difficulty: String,
    pub house_rules: String,
    /// Edition or ruleset offered by the selected pack.
    pub edition: String,
    /// The pack's progression terminology: level, rank, tier, experience or other.
    pub progression_type: String,
    /// "standard", "modified" or "custom" (empty when unanswered).
    pub house_rules_mode: String,
    /// Character-creation restrictions, stored for future Character Studio
    /// validation. Catalyst does not enforce them itself.
    pub allowed_content: Vec<String>,
    pub creation_notes: String,
}

impl Default for CampaignRules {
    fn default() -> Self {
        Self {
            pack_id: "system-neutral".into(),
            custom_system_name: String::new(),
            advancement: String::new(),
            starting_level: String::new(),
            difficulty: String::new(),
            house_rules: String::new(),
            edition: String::new(),
            progression_type: String::new(),
            house_rules_mode: String::new(),
            allowed_content: Vec::new(),
            creation_notes: String::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct CampaignWorld {
    /// "new" or "blank". Linked shared and imported worlds are not available
    /// yet and are rejected rather than half-supported.
    pub mode: String,
    pub name: String,
    pub era: String,
    pub calendar: String,
    pub geography: String,
    /// What makes the setting distinctive.
    pub notes: String,
    /// "original", "published", "adapted" or "undecided". Published settings
    /// are recorded by name only; no setting content is imported.
    pub origin: String,
    pub scale: String,
}

impl Default for CampaignWorld {
    fn default() -> Self {
        Self {
            mode: "blank".into(),
            name: String::new(),
            era: String::new(),
            calendar: String::new(),
            geography: String::new(),
            notes: String::new(),
            origin: String::new(),
            scale: String::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct WorldCategoryInput {
    pub id: String,
    /// Plain names (older payloads). Each becomes a record.
    pub names: Vec<String>,
    pub notes: String,
    /// The category's structured questions and the GM's answers.
    pub answers: Vec<CategoryAnswer>,
    /// Named records with details (a settlement's purpose and location, a
    /// nation's ruler). The description is Markdown composed by the wizard.
    pub entries: Vec<NamedEntryInput>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct CategoryAnswer {
    pub question: String,
    pub answer: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct PlayerInput {
    pub player_name: String,
    pub character_name: String,
    /// A one-line character concept until Character Studio exists.
    pub concept: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct PartyInput {
    pub name: String,
    pub starting_progression: String,
    pub players: Vec<PlayerInput>,
    /// Expected number of players (free text so "4-5" works).
    pub expected_players: String,
    /// "together", "first-session" or "mixed" (empty when unanswered).
    pub origin: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct NpcInput {
    pub name: String,
    /// "ally", "neutral" or "villain".
    pub disposition: String,
    pub role: String,
    pub motivation: String,
    pub description: String,
}

/// A named record with an optional kind (creature type, faction goal,
/// location type) and description.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct NamedEntryInput {
    pub name: String,
    pub kind: String,
    pub description: String,
    /// Creature threat level, or a faction's attitude toward the party.
    pub detail: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct StoryInput {
    /// "sandbox", "episodic", "linear" or "mixed".
    pub structure: String,
    /// "one-shot", "short", "medium", "long" or "ongoing".
    pub length: String,
    pub central_conflict: String,
    pub starting_adventure: String,
    pub starting_adventure_summary: String,
    pub arcs: Vec<String>,
    pub side_quests: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct LocationsInput {
    pub starting_location: String,
    pub entries: Vec<NamedEntryInput>,
    pub map_references: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct SessionInput {
    pub first_session_title: String,
    pub first_session_notes: String,
    pub gm_notes: String,
    /// "physical", "hybrid" or "digital".
    pub play_mode: String,
    /// "physical" or "digital".
    pub dice: String,
    pub player_display: bool,
    pub schedule: String,
    pub session_length: String,
    /// The first session is a Session Zero.
    pub session_zero: bool,
}

impl Default for SessionInput {
    fn default() -> Self {
        Self {
            first_session_title: String::new(),
            first_session_notes: String::new(),
            gm_notes: String::new(),
            play_mode: "physical".into(),
            dice: "physical".into(),
            player_display: false,
            schedule: String::new(),
            session_length: String::new(),
            session_zero: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewCampaignInput {
    pub project_root: String,
    pub name: String,
    #[serde(default = "default_setup_mode")]
    pub setup_mode: String,
    #[serde(default)]
    pub basics: CampaignBasics,
    #[serde(default)]
    pub rules: CampaignRules,
    #[serde(default)]
    pub world: CampaignWorld,
    #[serde(default)]
    pub worldbuilding: Vec<WorldCategoryInput>,
    #[serde(default)]
    pub party: PartyInput,
    #[serde(default)]
    pub npcs: Vec<NpcInput>,
    #[serde(default)]
    pub creatures: Vec<NamedEntryInput>,
    #[serde(default)]
    pub factions: Vec<NamedEntryInput>,
    #[serde(default)]
    pub story: StoryInput,
    #[serde(default)]
    pub locations: LocationsInput,
    #[serde(default)]
    pub session: SessionInput,
}

fn default_setup_mode() -> String {
    "quick-start".into()
}

// ── campaign.yaml ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignRulesManifest {
    pub pack_id: String,
    pub pack_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_system_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advancement: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starting_level: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progression_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub house_rules_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_creation: Option<CharacterCreationManifest>,
}

/// Restrictions for a future Character Studio to validate against.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterCreationManifest {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_content: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignWorldManifest {
    pub mode: String,
    pub reference_id: Option<String>,
    pub local_state_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setting_origin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignPlayManifest {
    pub mode: String,
    pub dice: String,
    pub player_display: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_length: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignPartyManifest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starting_progression: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub players: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_players: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

/// RPG-only campaign metadata. Identity (title, timestamps) stays in
/// project.yaml; campaign.yaml repeats only project_id so the pair can be
/// validated as belonging together.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignManifest {
    pub schema_version: u32,
    pub project_id: String,
    pub setup_mode: String,
    pub rules: CampaignRulesManifest,
    pub world: CampaignWorldManifest,
    pub play: CampaignPlayManifest,
    #[serde(default)]
    pub party: CampaignPartyManifest,
    pub starting_session_id: Option<String>,
}

/// Read campaign.yaml and verify it belongs to the project.yaml beside it.
pub fn read_campaign_manifest(root: &Path) -> Result<CampaignManifest, String> {
    let project = read_project_manifest(root)?;
    let path = root.join(CAMPAIGN_MANIFEST_FILENAME);
    let raw = fs::read_to_string(&path)
        .map_err(|e| format!("Could not read '{}': {e}", path.display()))?;
    let manifest: CampaignManifest =
        serde_yaml::from_str(&raw).map_err(|e| format!("Invalid campaign.yaml: {e}"))?;
    if manifest.schema_version != CAMPAIGN_SCHEMA_VERSION {
        return Err(format!(
            "Unsupported campaign.yaml schema_version {}. Expected {}.",
            manifest.schema_version, CAMPAIGN_SCHEMA_VERSION
        ));
    }
    if manifest.project_id != project.project_id {
        return Err("campaign.yaml project_id does not match project.yaml.".into());
    }
    if project.project_type != CAMPAIGN_PROJECT_TYPE {
        return Err("project.yaml does not identify a campaign.".into());
    }
    if rules_pack_version(&manifest.rules.pack_id).is_none() {
        return Err(format!("Unknown rules pack '{}'.", manifest.rules.pack_id));
    }
    Ok(manifest)
}

// ── Output ──────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize)]
pub struct NewCampaignProject {
    pub manifest: ProjectManifest,
    pub campaign: CampaignManifest,
    pub paths: ProjectPaths,
    /// Project-relative path of the Campaign Overview document.
    pub overview_path: String,
    pub overview_id: String,
}

/// Agenda added to a first session marked as Session Zero.
const SESSION_ZERO_AGENDA: &str = "## Session Zero Agenda\n\n- [ ] Pitch the campaign: genre, tone and premise\n- [ ] Agree on lines and veils, and how anyone can pause a scene\n- [ ] Confirm schedule, session length and what happens when someone is absent\n- [ ] Explain house rules and character-creation restrictions\n- [ ] Build characters together and tie each one to the group\n- [ ] Decide how the party knows each other";

pub const OVERVIEW_PATH: &str = "Adventures/Main Campaign/Campaign Overview.md";

// ── Creation pipeline (020 §10) ─────────────────────────────────────────────

pub fn create_campaign(input: &NewCampaignInput) -> Result<NewCampaignProject, String> {
    let root = PathBuf::from(input.project_root.trim());
    validate(input, &root)?;

    let parent = root
        .parent()
        .ok_or("Campaign destination has no parent folder.")?;
    let leaf = root
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or("Campaign destination folder name is not valid.")?;
    let staging = parent.join(format!(
        ".{leaf}.catalyst-staging-{}",
        Uuid::new_v4().simple()
    ));

    let (manifest, campaign, overview_id) = match build_in_staging(&staging, input) {
        Ok(result) => result,
        Err(error) => {
            // Only the staging directory belongs to this attempt.
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
    };

    if let Err(error) = publish(&staging, &root) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    Ok(NewCampaignProject {
        manifest,
        campaign,
        paths: ProjectPaths::from_root(&root),
        overview_path: OVERVIEW_PATH.into(),
        overview_id,
    })
}

fn validate(input: &NewCampaignInput, root: &Path) -> Result<(), String> {
    if !root.is_absolute() {
        return Err("Campaign destination must be an absolute local folder path.".into());
    }
    if root.components().any(|c| {
        matches!(
            c,
            std::path::Component::ParentDir | std::path::Component::CurDir
        )
    }) {
        return Err("Campaign destination must not contain '.' or '..' segments.".into());
    }
    if input.name.trim().is_empty() {
        return Err("Campaign name must not be empty.".into());
    }
    match root.parent() {
        Some(parent) if parent.is_dir() => {}
        _ => return Err("The folder that will contain the campaign does not exist.".into()),
    }
    if let Ok(meta) = fs::symlink_metadata(root) {
        if meta.file_type().is_symlink() {
            return Err("Campaign destination must not be a link.".into());
        }
        if !meta.is_dir() {
            return Err("Campaign destination points to a file, not a folder.".into());
        }
        if fs::read_dir(root)
            .map_err(|e| format!("Could not inspect campaign destination: {e}"))?
            .next()
            .is_some()
        {
            return Err("Choose an empty or new folder for the campaign.".into());
        }
    }
    if rules_pack_version(input.rules.pack_id.trim()).is_none() {
        return Err(format!("Unknown rules pack '{}'.", input.rules.pack_id));
    }
    if input.rules.pack_id.trim() == "custom-homebrew"
        && input.rules.custom_system_name.trim().is_empty()
    {
        return Err("Name your custom or homebrew system.".into());
    }
    if !matches!(input.setup_mode.trim(), "quick-start" | "guided" | "blank") {
        return Err(format!("Unknown setup mode '{}'.", input.setup_mode));
    }
    if !matches!(input.world.mode.trim(), "new" | "blank") {
        return Err("Only a new or blank world is available in this version.".into());
    }
    if !matches!(input.world.origin.trim(), "" | "original" | "published" | "adapted" | "undecided") {
        return Err(format!("Unknown setting origin '{}'.", input.world.origin));
    }
    if !matches!(input.rules.house_rules_mode.trim(), "" | "standard" | "modified" | "custom") {
        return Err(format!("Unknown house rules choice '{}'.", input.rules.house_rules_mode));
    }
    for category in &input.worldbuilding {
        if world_category(category.id.trim()).is_none() {
            return Err(format!("Unknown worldbuilding category '{}'.", category.id));
        }
    }
    if !matches!(input.party.origin.trim(), "" | "together" | "first-session" | "mixed") {
        return Err(format!("Unknown party start '{}'.", input.party.origin));
    }
    if !matches!(input.story.structure.trim(), "" | "sandbox" | "episodic" | "linear" | "mixed") {
        return Err(format!("Unknown campaign structure '{}'.", input.story.structure));
    }
    if !matches!(input.story.length.trim(), "" | "one-shot" | "short" | "medium" | "long" | "ongoing") {
        return Err(format!("Unknown campaign length '{}'.", input.story.length));
    }
    if input.npcs.iter().any(|n| n.name.trim().is_empty()) {
        return Err("NPC names must not be empty.".into());
    }
    if input
        .worldbuilding
        .iter()
        .any(|c| c.entries.iter().any(|e| e.name.trim().is_empty()))
    {
        return Err("Worldbuilding entries need a name.".into());
    }
    for (label, list) in [
        ("Creature", &input.creatures),
        ("Faction", &input.factions),
        ("Location", &input.locations.entries),
    ] {
        if list.iter().any(|e| e.name.trim().is_empty()) {
            return Err(format!("{label} names must not be empty."));
        }
    }
    if !matches!(
        input.session.play_mode.trim(),
        "physical" | "hybrid" | "digital"
    ) {
        return Err("Tabletop style must be physical, hybrid or digital.".into());
    }
    if !matches!(input.session.dice.trim(), "physical" | "digital") {
        return Err("Dice preference must be physical or digital.".into());
    }
    Ok(())
}

fn build_in_staging(
    staging: &Path,
    input: &NewCampaignInput,
) -> Result<(ProjectManifest, CampaignManifest, String), String> {
    let manifest = ProjectManifest::new(input.name.trim(), CAMPAIGN_PROJECT_TYPE);
    initialize_project_root_with_directories(staging, &manifest, CAMPAIGN_DIRECTORIES)?;
    seed_player_characters_placeholder(staging)?;

    let seeded = seed_documents(staging, input)?;

    let pack_id = input.rules.pack_id.trim().to_string();
    let campaign = CampaignManifest {
        schema_version: CAMPAIGN_SCHEMA_VERSION,
        project_id: manifest.project_id.clone(),
        setup_mode: input.setup_mode.trim().into(),
        rules: CampaignRulesManifest {
            pack_version: rules_pack_version(&pack_id).expect("validated").into(),
            pack_id,
            custom_system_name: non_empty(&input.rules.custom_system_name),
            advancement: non_empty(&input.rules.advancement),
            starting_level: non_empty(&input.rules.starting_level),
            difficulty: non_empty(&input.rules.difficulty),
            edition: non_empty(&input.rules.edition),
            progression_type: non_empty(&input.rules.progression_type),
            house_rules_mode: non_empty(&input.rules.house_rules_mode),
            character_creation: {
                let allowed_content: Vec<String> = input
                    .rules
                    .allowed_content
                    .iter()
                    .map(|c| c.trim().to_string())
                    .filter(|c| !c.is_empty())
                    .collect();
                let notes = non_empty(&input.rules.creation_notes);
                (!allowed_content.is_empty() || notes.is_some())
                    .then_some(CharacterCreationManifest { allowed_content, notes })
            },
        },
        world: CampaignWorldManifest {
            mode: input.world.mode.trim().into(),
            reference_id: None,
            local_state_path: CAMPAIGN_STATE_PATH.into(),
            setting_origin: non_empty(&input.world.origin),
            scale: non_empty(&input.world.scale),
        },
        play: CampaignPlayManifest {
            mode: input.session.play_mode.trim().into(),
            dice: input.session.dice.trim().into(),
            player_display: input.session.player_display,
            schedule: non_empty(&input.session.schedule),
            session_length: non_empty(&input.session.session_length),
        },
        party: CampaignPartyManifest {
            name: non_empty(&input.party.name),
            starting_progression: non_empty(&input.party.starting_progression),
            players: input
                .party
                .players
                .iter()
                .map(|p| p.player_name.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect(),
            expected_players: non_empty(&input.party.expected_players),
            origin: non_empty(&input.party.origin),
        },
        starting_session_id: seeded.first_session_id,
    };
    let yaml = serde_yaml::to_string(&campaign)
        .map_err(|e| format!("Could not serialize campaign.yaml: {e}"))?;
    fs::write(staging.join(CAMPAIGN_MANIFEST_FILENAME), yaml)
        .map_err(|e| format!("Could not write campaign.yaml: {e}"))?;

    verify(staging, &campaign)?;
    Ok((manifest, campaign, seeded.overview_id))
}

/// Validate the staged campaign before it becomes visible (020 §10 step 5).
fn verify(staging: &Path, expected: &CampaignManifest) -> Result<(), String> {
    for dir in CAMPAIGN_DIRECTORIES {
        if !staging.join(dir).is_dir() {
            return Err(format!("Campaign folder '{dir}' was not created."));
        }
    }
    if &read_campaign_manifest(staging)? != expected {
        return Err("campaign.yaml did not round-trip.".into());
    }
    let index = crate::story_development::build_index(staging)?;
    if let Some(issue) = index.issues.first() {
        return Err(format!(
            "Seeded document '{}' is invalid: {}",
            issue.relative_path, issue.message
        ));
    }
    let mut ids = HashSet::new();
    for entry in index.entries.iter().filter(|e| e.entry_kind == "document") {
        if !ids.insert(entry.id.as_str()) {
            return Err(format!(
                "Duplicate stable ID {} in seeded campaign.",
                entry.id
            ));
        }
    }
    Ok(())
}

fn publish(staging: &Path, root: &Path) -> Result<(), String> {
    // An existing destination was verified empty. Windows cannot rename over a
    // directory, so remove the empty folder first and restore it on failure.
    let removed_empty = if root.exists() {
        fs::remove_dir(root).map_err(|e| format!("Could not prepare campaign destination: {e}"))?;
        true
    } else {
        false
    };
    if let Err(e) = fs::rename(staging, root) {
        if removed_empty {
            let _ = fs::create_dir(root);
        }
        return Err(format!("Could not publish the new campaign: {e}"));
    }
    Ok(())
}

// ── Seeding ─────────────────────────────────────────────────────────────────

struct Seeded {
    overview_id: String,
    first_session_id: Option<String>,
}

/// Tracks used filenames per folder so records from different steps that land
/// in the same folder never collide.
#[derive(Default)]
struct Seeder {
    used: BTreeMap<String, BTreeMap<String, usize>>,
}

impl Seeder {
    /// Create one record from a registered entity template, exactly as the
    /// wiki's New Document command would, then apply the wizard's values.
    /// Every seeded record is GM-only until the GM reveals it (020 §8).
    fn record(
        &mut self,
        root: &Path,
        folder: &str,
        title: &str,
        template_id: &str,
        prose: &str,
        values: &[(&str, Value)],
    ) -> Result<CatalystDocument, String> {
        let template = built_in_template(template_id)
            .ok_or_else(|| format!("Built-in template '{template_id}' is missing."))?;
        let used = self.used.entry(folder.to_string()).or_default();
        let relative = unique_entity_path(folder, title, used);

        let prose = prose.trim();
        let scaffold = template.body.trim();
        let body = if template_id == "campaign-overview" && !prose.is_empty() {
            // The wizard has already filled the master-reference sections.
            // Do not append a second blank scaffold or create duplicate facts.
            prose.to_string()
        } else {
            match (prose.is_empty(), scaffold.is_empty()) {
                (true, _) => scaffold.to_string(),
                (false, true) => prose.to_string(),
                (false, false) => format!("{prose}\n\n{scaffold}"),
            }
        };
        let mut doc = create_document(
            root,
            &relative,
            &template.manifest.document_type,
            Some(title.trim()),
            &body,
        )?;
        let m = &mut doc.metadata;
        m.insert(
            "template_id".into(),
            Value::String(template.manifest.id.clone()),
        );
        if let Some(subtype) = &template.manifest.subtype {
            m.insert("subtype".into(), Value::String(subtype.clone()));
        }
        for field in &template.manifest.fields {
            m.entry(field.key.clone())
                .or_insert_with(|| empty_field_value(&field.field_type));
        }
        m.insert("visibility".into(), Value::String("gm-only".into()));
        for (key, value) in values {
            let blank = matches!(value, Value::String(s) if s.trim().is_empty());
            if !blank {
                m.insert((*key).into(), value.clone());
            }
        }
        save_document(root, &doc, Some(&doc.source_hash))?;
        Ok(doc)
    }
}

/// Copy the offline GM Reference Library (020 §9) into the campaign so the GM
/// can edit it. Every entry is original Catalyst content.
fn seed_gm_library(root: &Path, seeder: &mut Seeder) -> Result<(), String> {
    use crate::gm_library::{GUIDES, GUIDES_FOLDER, LIBRARY_LICENSE, LIBRARY_SOURCE, TABLES, TABLES_FOLDER};
    for guide in GUIDES {
        seeder.record(root, GUIDES_FOLDER, guide.title, "gm-guide", guide.body, &[
            ("guide_topic", s(guide.topic)),
            ("source", s(LIBRARY_SOURCE)),
            ("license", s(LIBRARY_LICENSE)),
        ])?;
    }
    for table in TABLES {
        seeder.record(root, TABLES_FOLDER, table.title, "random-table", &table.body(), &[
            ("die", s(&table.die())),
            ("table_category", s(table.category)),
            ("source", s(LIBRARY_SOURCE)),
            ("license", s(LIBRARY_LICENSE)),
        ])?;
    }
    Ok(())
}

fn empty_field_value(field_type: &str) -> Value {
    match field_type {
        "list" => Value::Sequence(Vec::new()),
        "boolean" => Value::Bool(false),
        "number" => Value::Null,
        _ => Value::String(String::new()),
    }
}

fn s(value: &str) -> Value {
    Value::String(value.trim().to_string())
}

fn seed_documents(root: &Path, input: &NewCampaignInput) -> Result<Seeded, String> {
    let mut seeder = Seeder::default();
    let b = &input.basics;
    let story = &input.story;

    // The campaign title and identity live in project.yaml. The overview is
    // an editable narrative master reference, not a second rules/config store.
    let w = &input.world;
    let party_name = input.party.name.trim();
    let players = input.party.players.iter()
        .filter_map(|p| {
            let name = p.character_name.trim();
            if name.is_empty() { None } else { Some(format!("- {name}")) }
        }).collect::<Vec<_>>();
    let npc_names = input.npcs.iter().filter_map(|n| {
        let name = n.name.trim();
        if name.is_empty() { None } else { Some(format!("- {name}")) }
    }).collect::<Vec<_>>();
    let faction_names = input.factions.iter().filter_map(|n| {
        let name = n.name.trim();
        if name.is_empty() { None } else { Some(format!("- {name}")) }
    }).collect::<Vec<_>>();
    let overview_body = format!(
        "## Campaign Summary\n\n{}\n\n## World and Setting\n\n{}\n\n{}\n\n## Party\n\n{}\n\n{}\n\n## Main Story Arc\n\n{}\n\n## Active Quests\n\n{}\n\n## Important NPCs and Factions\n\n### NPCs\n\n{}\n\n### Factions\n\n{}\n\n## Campaign Timeline\n\nUpdate after actual sessions; see Sessions/Session Logs and Sessions/Campaign State.\n\n## Player-Facing Pitch\n\n{}\n\n## GM-Only Notes\n\nAdd private GM notes here. Visibility metadata is descriptive and must not be treated as enforced sharing permissions.\n\n## Related Records\n\n- Worldbuilding/ (lore and locations)\n- Characters/ (PC placeholders and NPC profiles)\n- Adventures/ (scenes and quests)\n- Sessions/ (plans, logs and campaign state)\n- Rules & Mechanics/ (system references and house rules)\n\nEdit the source records rather than duplicating their content here.",
        b.premise.trim(),
        w.name.trim(),
        w.notes.trim(),
        if party_name.is_empty() { "Party not named" } else { party_name },
        players.join("\n"),
        story.central_conflict.trim(),
        story.side_quests.join("\n"),
        npc_names.join("\n"),
        faction_names.join("\n"),
        b.subtitle.trim()
    );
    let overview = seeder.record(
        root,
        "Adventures/Main Campaign",
        "Campaign Overview",
        "campaign-overview",
        &overview_body,
        &[
            ("subtitle", s(&b.subtitle)),
            ("genre", s(&b.genre)),
            ("tone", s(&b.tone)),
            ("gm_name", s(&b.gm_name)),
            ("start_date", s(&b.start_date)),
            ("central_conflict", s(&story.central_conflict)),
            ("world_name", s(&w.name)),
            ("party_name", s(&input.party.name)),
            ("ruleset_id", s(&input.rules.pack_id)),
            ("edition_id", s(&input.rules.edition)),
            ("campaign_structure", s(&story.structure)),
            ("campaign_length", s(&story.length)),
            ("status", s("planning")),
        ],
    )?;
    debug_assert_eq!(overview.relative_path, Path::new(OVERVIEW_PATH));

    let rules = &input.rules;
    if !rules.house_rules.trim().is_empty() {
        let system = if rules.pack_id == "custom-homebrew" {
            rules.custom_system_name.trim()
        } else {
            "System-Neutral"
        };
        seeder.record(
            root,
            "Rules & Mechanics/House Rules",
            "House Rules",
            "house-rules",
            &rules.house_rules,
            &[("rules_system", s(system))],
        )?;
    }

    let w = &input.world;
    if [&w.name, &w.era, &w.calendar, &w.geography, &w.notes, &w.origin, &w.scale]
        .iter()
        .any(|v| !v.trim().is_empty())
    {
        let title = if w.name.trim().is_empty() {
            "World Overview"
        } else {
            w.name.trim()
        };
        seeder.record(
            root,
            "Worldbuilding",
            title,
            "worldbuilding-profile",
            &w.notes,
            &[
                ("setting", s(&w.name)),
                ("time_period", s(&w.era)),
                ("calendar", s(&w.calendar)),
                ("geography", s(&w.geography)),
                ("setting_origin", s(&w.origin)),
                ("scale", s(&w.scale)),
            ],
        )?;
    }

    let mut seen = BTreeSet::new();
    for category in &input.worldbuilding {
        let (id, folder, template) = world_category(category.id.trim()).expect("validated");
        if !seen.insert(id) {
            continue;
        }
        let label = folder;
        let folder = format!("Worldbuilding/{folder}");
        let mut overview: Vec<String> = category
            .answers
            .iter()
            .filter(|a| !a.answer.trim().is_empty() && !a.question.trim().is_empty())
            .map(|a| format!("## {}\n\n{}", a.question.trim(), a.answer.trim()))
            .collect();
        if !category.notes.trim().is_empty() {
            overview.push(format!("## Notes\n\n{}", category.notes.trim()));
        }
        if !overview.is_empty() {
            seeder.record(
                root,
                &folder,
                &format!("{label} Overview"),
                "worldbuilding-profile",
                &overview.join("\n\n"),
                &[],
            )?;
        }
        for entry in &category.entries {
            seeder.record(root, &folder, entry.name.trim(), template, &entry.description, &[])?;
        }
        for name in category
            .names
            .iter()
            .map(|n| n.trim())
            .filter(|n| !n.is_empty())
        {
            seeder.record(root, &folder, name, template, "", &[])?;
        }
    }

    for player in &input.party.players {
        let character = player.character_name.trim();
        if character.is_empty() {
            continue;
        }
        // A minimal placeholder, not a character sheet (020 §7).
        seeder.record(
            root,
            "Characters/Player Characters",
            character,
            "player-character-placeholder",
            &player.concept,
            &[
                ("player", s(&player.player_name)),
                ("status", s("placeholder")),
                ("character_record", s("Pending Character Studio")),
            ],
        )?;
    }

    for npc in &input.npcs {
        let disposition = match npc.disposition.trim() {
            d @ ("ally" | "neutral" | "rival" | "villain" | "unknown") => d,
            _ => "neutral",
        };
        seeder.record(
            root,
            "Characters/Non-Player Characters",
            &npc.name,
            "npc-profile",
            &npc.description,
            &[
                ("disposition", s(disposition)),
                ("role", s(&npc.role)),
                ("motivation", s(&npc.motivation)),
            ],
        )?;
    }

    for creature in &input.creatures {
        seeder.record(
            root,
            "Creatures & Monsters/Custom Creatures",
            &creature.name,
            "creature-profile",
            &creature.description,
            &[("creature_type", s(&creature.kind)), ("threat", s(&creature.detail))],
        )?;
    }

    for faction in &input.factions {
        seeder.record(
            root,
            "Worldbuilding/Organizations & Factions",
            &faction.name,
            "faction-secret-society",
            &faction.description,
            &[("agenda", s(&faction.kind)), ("attitude", s(&faction.detail))],
        )?;
    }

    if !story.starting_adventure.trim().is_empty() {
        seeder.record(
            root,
            "Adventures/Main Campaign",
            &story.starting_adventure,
            "quest-adventure",
            &story.starting_adventure_summary,
            &[
                ("quest_type", s("Starting adventure")),
                ("status", s("planned")),
            ],
        )?;
    }
    for arc in story
        .arcs
        .iter()
        .map(|a| a.trim())
        .filter(|a| !a.is_empty())
    {
        seeder.record(
            root,
            "Adventures/Main Campaign",
            &short_title(arc),
            "quest-adventure",
            arc,
            &[("quest_type", s("Story arc")), ("status", s("planned"))],
        )?;
    }
    for quest in story
        .side_quests
        .iter()
        .map(|q| q.trim())
        .filter(|q| !q.is_empty())
    {
        seeder.record(
            root,
            "Adventures/Side Quests",
            &short_title(quest),
            "quest-adventure",
            quest,
            &[("quest_type", s("Side quest")), ("status", s("available"))],
        )?;
    }

    let loc = &input.locations;
    let starting = loc.starting_location.trim();
    let mut starting_seeded = false;
    for entry in &loc.entries {
        let is_start = !starting.is_empty() && entry.name.trim().eq_ignore_ascii_case(starting);
        starting_seeded |= is_start;
        let mut values = vec![("location_type", s(&entry.kind))];
        if is_start {
            values.push(("starting_location", Value::Bool(true)));
        }
        seeder.record(
            root,
            "Worldbuilding/Settlements & Locations",
            &entry.name,
            "location-profile",
            &entry.description,
            &values,
        )?;
    }
    if !starting.is_empty() && !starting_seeded {
        seeder.record(
            root,
            "Worldbuilding/Settlements & Locations",
            starting,
            "location-profile",
            "",
            &[("starting_location", Value::Bool(true))],
        )?;
    }
    let maps: Vec<&str> = loc
        .map_references
        .iter()
        .map(|m| m.trim())
        .filter(|m| !m.is_empty())
        .collect();
    if !maps.is_empty() {
        // References only: Map Studio has not generated these maps.
        let body = maps
            .iter()
            .map(|m| format!("- {m}"))
            .collect::<Vec<_>>()
            .join("\n");
        seeder.record(root, "Maps", "Map References", "map-reference", &body, &[])?;
    }

    let session = &input.session;
    let mut first_session_id = None;
    if !session.first_session_title.trim().is_empty()
        || !session.first_session_notes.trim().is_empty()
        || session.session_zero
    {
        let title = match session.first_session_title.trim() {
            "" if session.session_zero => "Session Zero",
            "" => "Session 1",
            t => t,
        };
        let mut prose = session.first_session_notes.trim().to_string();
        if session.session_zero {
            if !prose.is_empty() {
                prose.push_str("\n\n");
            }
            prose.push_str(SESSION_ZERO_AGENDA);
        }
        let doc = seeder.record(
            root,
            "Sessions/Session Plans",
            title,
            "session-plan",
            &prose,
            &[
                ("session_number", Value::Number(1.into())),
                ("status", s("planned")),
            ],
        )?;
        first_session_id = Some(doc.identity.id);
    }
    if !session.gm_notes.trim().is_empty() {
        seeder.record(
            root,
            "Game Master/GM Notes",
            "GM Notes",
            "gm-notes",
            &session.gm_notes,
            &[],
        )?;
    }

    // A Blank Campaign gets only the required anchor record.
    if input.setup_mode.trim() != "blank" {
        seed_gm_library(root, &mut seeder)?;
    }

    Ok(Seeded {
        overview_id: overview.identity.id,
        first_session_id,
    })
}

/// Player characters belong to the upcoming Character Studio; the README says
/// so explicitly (020 §7). A README without Catalyst frontmatter is ignored by
/// the Story Development index.
fn seed_player_characters_placeholder(root: &Path) -> Result<(), String> {
    fs::write(
        root.join("Characters/Player Characters/README.md"),
        "# Player Characters\n\nCharacter Studio is not available yet. Player characters entered in the\n\
         New Campaign Wizard are minimal placeholder records, not character sheets.\n",
    )
    .map_err(|e| format!("Could not write Player Characters placeholder: {e}"))
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn short_title(text: &str) -> String {
    let first = text.lines().next().unwrap_or("").trim();
    let mut title: String = first.chars().take(60).collect();
    if first.chars().count() > 60 {
        title = title.trim_end().to_string();
        title.push('…');
    }
    if title.is_empty() {
        "Untitled".into()
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
    let out = out.trim().trim_end_matches(['.', ' ']).trim();
    if out.is_empty() {
        "Untitled".into()
    } else {
        out.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::service::read_document;
    use crate::project_discovery::project_from_root;
    use crate::story_development::build_index;

    fn quick_start(root: &Path) -> NewCampaignInput {
        serde_json::from_value(serde_json::json!({
            "project_root": root.to_string_lossy(),
            "name": "Ashes of Varn",
            "setup_mode": "quick-start",
            "basics": { "premise": "A border kingdom burns." },
        }))
        .unwrap()
    }

    fn guided(root: &Path) -> NewCampaignInput {
        serde_json::from_value(serde_json::json!({
            "project_root": root.to_string_lossy(),
            "name": "The Drowned Crown",
            "setup_mode": "guided",
            "basics": { "subtitle": "Book One", "premise": "Pirates hunt a sunken throne.", "genre": "Nautical fantasy", "tone": "Swashbuckling", "gm_name": "GM", "start_date": "Spring 1022" },
            "rules": { "pack_id": "custom-homebrew", "custom_system_name": "Tides d6", "advancement": "Milestone", "starting_level": "3", "difficulty": "Standard", "house_rules": "Flanking grants advantage." },
            "world": { "mode": "new", "name": "The Shattered Isles", "era": "Age of Sail", "calendar": "Tidal reckoning", "geography": "Archipelago", "notes": "Islands drift." },
            "worldbuilding": [
                { "id": "factions", "names": ["The Tide Court", "The Tide Court"], "notes": "Pirate lords vie." },
                { "id": "religions", "names": [], "notes": "" },
                { "id": "magic", "names": ["Brine-binding"], "notes": "" }
            ],
            "party": { "name": "The Gullwings", "starting_progression": "Level 3", "players": [ { "player_name": "Sam", "character_name": "Kestrel" }, { "player_name": "Ari", "character_name": "" } ] },
            "npcs": [
                { "name": "Captain Mara", "disposition": "ally", "role": "patron", "motivation": "Revenge", "description": "Stern." },
                { "name": "The Drowned King", "disposition": "villain" }
            ],
            "creatures": [ { "name": "Reef Wraith", "kind": "undead", "description": "Cold." } ],
            "factions": [ { "name": "Iron Navy", "kind": "Order at sea", "description": "" } ],
            "story": { "central_conflict": "Who claims the crown", "starting_adventure": "Wreck of the Gull", "starting_adventure_summary": "A storm.", "arcs": ["The Map"], "side_quests": ["Lost cargo"] },
            "locations": { "starting_location": "Saltmere", "entries": [ { "name": "Port: Saltmere?", "kind": "port", "description": "Rough harbour." } ], "map_references": ["Assets/Images/isles.png"] },
            "session": { "first_session_title": "Storm at Sea", "first_session_notes": "Open in a storm.", "gm_notes": "Mara lies.", "play_mode": "hybrid", "dice": "digital", "player_display": true }
        }))
        .unwrap()
    }

    fn md_files(root: &Path, dir: &str) -> Vec<String> {
        let mut names: Vec<_> = fs::read_dir(root.join(dir))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".md"))
            .collect();
        names.sort();
        names
    }

    fn meta<'a>(doc: &'a CatalystDocument, key: &str) -> Option<&'a Value> {
        doc.metadata.get(key)
    }

    #[test]
    fn quick_start_creates_manifests_and_the_complete_approved_tree() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Ashes of Varn");
        let created = create_campaign(&quick_start(&root)).unwrap();

        let project = read_project_manifest(&root).unwrap();
        assert_eq!(project, created.manifest);
        assert_eq!(project.project_type, "campaign");
        let campaign = read_campaign_manifest(&root).unwrap();
        assert_eq!(campaign, created.campaign);
        assert_eq!(campaign.project_id, project.project_id);
        assert_eq!(campaign.rules.pack_id, "system-neutral");
        assert_eq!(campaign.world.local_state_path, "Sessions/Campaign State/");
        assert_eq!(campaign.setup_mode, "quick-start");
        assert_eq!(campaign.starting_session_id, None);

        for dir in CAMPAIGN_DIRECTORIES {
            assert!(root.join(dir).is_dir(), "missing campaign folder {dir}");
        }
        assert!(root.join("Sessions/Roll Logs").is_dir());
        for book in [
            "Manuscript",
            "Audiobook",
            "Reviews",
            "Project Meta",
            "Notes",
            "Story Development",
        ] {
            assert!(!root.join(book).exists(), "campaign must not create {book}");
        }
        let siblings: Vec<_> = fs::read_dir(parent.path()).unwrap().collect();
        assert_eq!(siblings.len(), 1, "staging residue next to the campaign");

        // Quick Start seeds only the overview plus the GM Reference Library:
        // no invented facts or blank lore files.
        let index = build_index(&root).unwrap();
        assert!(index.issues.is_empty(), "{:?}", index.issues);
        let library: Vec<_> = index
            .entries
            .iter()
            .filter(|e| e.entry_kind == "document" && e.relative_path.starts_with("Game Master/"))
            .collect();
        assert_eq!(library.len(), crate::gm_library::GUIDES.len() + crate::gm_library::TABLES.len());
        let table = read_document(&root, Path::new("Game Master/Random Tables/Weather.md")).unwrap();
        assert_eq!(table.metadata.get("template_id").and_then(|v| v.as_str()), Some("random-table"));
        assert_eq!(table.metadata.get("die").and_then(|v| v.as_str()), Some("d8"));
        assert!(table.body.contains("| 8 | Snowfall"));
        let docs: Vec<_> = index
            .entries
            .iter()
            .filter(|e| e.entry_kind == "document" && !e.relative_path.starts_with("Game Master/"))
            .collect();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].relative_path, OVERVIEW_PATH);
        assert_eq!(docs[0].id, created.overview_id);
        assert_eq!(docs[0].section, "Adventures");
        let overview = read_document(&root, Path::new(OVERVIEW_PATH)).unwrap();
        assert!(overview
            .body
            .starts_with("A border kingdom burns.\n\n## Premise"));
        assert!(root
            .join("Characters/Player Characters/README.md")
            .is_file());
    }

    #[test]
    fn seeded_records_follow_the_wiki_template_contract() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Contract");
        create_campaign(&guided(&root)).unwrap();

        let index = build_index(&root).unwrap();
        for entry in index.entries.iter().filter(|e| e.entry_kind == "document") {
            let doc = read_document(&root, Path::new(&entry.relative_path)).unwrap();
            let id = match meta(&doc, "template_id") {
                Some(Value::String(id)) => id.clone(),
                other => panic!("{} has no template_id: {other:?}", entry.relative_path),
            };
            let template =
                built_in_template(&id).unwrap_or_else(|| panic!("unregistered template {id}"));
            assert_eq!(
                doc.identity.document_type, template.manifest.document_type,
                "{}",
                entry.relative_path
            );
            assert_eq!(
                meta(&doc, "subtype").and_then(Value::as_str),
                template.manifest.subtype.as_deref()
            );
            // Every field (including the image field) exists, so the Overview
            // panel can render and edit it.
            assert!(
                doc.metadata.contains_key(&template.manifest.image_field),
                "{} lacks image",
                entry.relative_path
            );
            for field in &template.manifest.fields {
                assert!(
                    doc.metadata.contains_key(&field.key),
                    "{} lacks {}",
                    entry.relative_path,
                    field.key
                );
            }
            assert_eq!(
                meta(&doc, "visibility"),
                Some(&Value::String("gm-only".into()))
            );
        }

        let npc = read_document(
            &root,
            Path::new("Characters/Non-Player Characters/Captain Mara.md"),
        )
        .unwrap();
        assert_eq!(
            meta(&npc, "template_id").and_then(Value::as_str),
            Some("npc-profile")
        );
        assert_eq!(
            meta(&npc, "motivation").and_then(Value::as_str),
            Some("Revenge")
        );
        assert!(npc.body.starts_with("Stern.\n\n## Appearance"));
    }

    #[test]
    fn guided_setup_places_each_entity_in_its_approved_folder() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Drowned Crown");
        let created = create_campaign(&guided(&root)).unwrap();

        let c = &created.campaign;
        assert_eq!(c.rules.pack_id, "custom-homebrew");
        assert_eq!(c.rules.custom_system_name.as_deref(), Some("Tides d6"));
        assert_eq!(c.rules.starting_level.as_deref(), Some("3"));
        assert_eq!(c.world.mode, "new");
        assert_eq!(
            c.play,
            CampaignPlayManifest {
                mode: "hybrid".into(),
                dice: "digital".into(),
                player_display: true,
                schedule: None,
                session_length: None,
            }
        );
        assert_eq!(c.party.players, vec!["Sam".to_string(), "Ari".to_string()]);
        let session =
            read_document(&root, Path::new("Sessions/Session Plans/Storm at Sea.md")).unwrap();
        assert_eq!(
            c.starting_session_id.as_deref(),
            Some(session.identity.id.as_str())
        );
        assert_eq!(
            meta(&session, "session_number"),
            Some(&Value::Number(1.into()))
        );

        assert_eq!(
            md_files(&root, "Worldbuilding/Organizations & Factions"),
            vec![
                "Iron Navy.md",
                "Organizations & Factions Overview.md",
                "The Tide Court 2.md",
                "The Tide Court.md"
            ]
        );
        let court = read_document(
            &root,
            Path::new("Worldbuilding/Organizations & Factions/The Tide Court.md"),
        )
        .unwrap();
        assert!(court.identity.id.starts_with("org-"));
        assert!(md_files(&root, "Worldbuilding/Religions & Deities").is_empty());
        assert_eq!(
            md_files(&root, "Worldbuilding/Magic & Supernatural"),
            vec!["Brine-binding.md"]
        );
        assert!(root.join("Worldbuilding/The Shattered Isles.md").is_file());

        let pc =
            read_document(&root, Path::new("Characters/Player Characters/Kestrel.md")).unwrap();
        assert_eq!(
            meta(&pc, "status").and_then(Value::as_str),
            Some("placeholder")
        );
        assert_eq!(meta(&pc, "player").and_then(Value::as_str), Some("Sam"));
        let villain = read_document(
            &root,
            Path::new("Characters/Non-Player Characters/The Drowned King.md"),
        )
        .unwrap();
        assert!(villain.identity.id.starts_with("chr-"));
        assert_eq!(
            meta(&villain, "disposition").and_then(Value::as_str),
            Some("villain")
        );
        assert!(root
            .join("Creatures & Monsters/Custom Creatures/Reef Wraith.md")
            .is_file());
        assert!(root
            .join("Adventures/Main Campaign/Wreck of the Gull.md")
            .is_file());
        assert!(root.join("Adventures/Main Campaign/The Map.md").is_file());
        assert!(root.join("Adventures/Side Quests/Lost cargo.md").is_file());

        let start = read_document(
            &root,
            Path::new("Worldbuilding/Settlements & Locations/Saltmere.md"),
        )
        .unwrap();
        assert_eq!(meta(&start, "starting_location"), Some(&Value::Bool(true)));
        let port = read_document(
            &root,
            Path::new("Worldbuilding/Settlements & Locations/Port- Saltmere-.md"),
        )
        .unwrap();
        assert_eq!(port.title.as_deref(), Some("Port: Saltmere?"));

        assert!(root.join("Maps/Map References.md").is_file());
        assert!(root
            .join("Rules & Mechanics/House Rules/House Rules.md")
            .is_file());
        assert!(root.join("Game Master/GM Notes/GM Notes.md").is_file());
    }

    #[test]
    fn stable_ids_are_unique_and_survive_edit_move_and_reopen() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Stable");
        let created = create_campaign(&guided(&root)).unwrap();

        let index = build_index(&root).unwrap();
        assert!(index.issues.is_empty(), "{:?}", index.issues);
        let docs: Vec<_> = index
            .entries
            .iter()
            .filter(|e| e.entry_kind == "document")
            .collect();
        assert!(docs.len() >= 18, "only {} documents indexed", docs.len());
        let ids: HashSet<_> = docs.iter().map(|e| e.id.clone()).collect();
        assert_eq!(ids.len(), docs.len());
        let sections: BTreeSet<_> = docs.iter().map(|e| e.section.as_str()).collect();
        for section in [
            "Worldbuilding",
            "Characters",
            "Creatures & Monsters",
            "Adventures",
            "Maps",
            "Rules & Mechanics",
            "Sessions",
            "Game Master",
        ] {
            assert!(
                sections.contains(section),
                "wiki is missing section {section}"
            );
        }

        let path = Path::new("Characters/Non-Player Characters/Captain Mara.md");
        let mut doc = read_document(&root, path).unwrap();
        let id = doc.identity.id.clone();
        let hash = doc.source_hash.clone();
        doc.body = "Edited by the GM.\n".into();
        save_document(&root, &doc, Some(&hash)).unwrap();
        let saved = read_document(&root, path).unwrap();
        let moved = Path::new("Characters/Non-Player Characters/Allies/Captain Mara.md");
        crate::document::service::move_document(&root, path, moved, &saved.source_hash).unwrap();

        let reopened = project_from_root(&root).unwrap();
        assert_eq!(reopened.id.to_string(), created.manifest.project_id);
        assert_eq!(reopened.project_type, "campaign");
        assert_eq!(read_campaign_manifest(&root).unwrap(), created.campaign);
        let again = read_document(&root, moved).unwrap();
        assert_eq!(again.identity.id, id);
        assert_eq!(again.body, "Edited by the GM.\n");
        let rebuilt = build_index(&root).unwrap();
        assert!(rebuilt
            .entries
            .iter()
            .any(|e| e.id == id && e.relative_path == moved.to_string_lossy()));
    }

    #[test]
    fn blank_campaign_seeds_only_the_overview_and_defaults_to_system_neutral() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Empty");
        let input: NewCampaignInput = serde_json::from_value(serde_json::json!({
            "project_root": root.to_string_lossy(), "name": "Empty", "setup_mode": "blank",
        }))
        .unwrap();
        create_campaign(&input).unwrap();
        let campaign = read_campaign_manifest(&root).unwrap();
        assert_eq!(campaign.setup_mode, "blank");
        assert_eq!(campaign.rules.pack_id, "system-neutral");
        let docs: Vec<_> = build_index(&root).unwrap().entries.into_iter()
            .filter(|e| e.entry_kind == "document").map(|e| e.relative_path).collect();
        assert_eq!(docs, vec![OVERVIEW_PATH.to_string()]);
        for dir in CAMPAIGN_DIRECTORIES {
            assert!(root.join(dir).is_dir());
        }
    }

    #[test]
    fn campaign_builder_answers_reach_campaign_yaml_and_wiki_pages() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Builder");
        let input: NewCampaignInput = serde_json::from_value(serde_json::json!({
            "project_root": root.to_string_lossy(), "name": "Builder", "setup_mode": "guided",
            "rules": {
                "pack_id": "system-neutral", "edition": "Any / not specified", "progression_type": "tier",
                "starting_level": "Tier 2", "advancement": "Milestones", "house_rules_mode": "modified",
                "house_rules": "No fumbles.", "allowed_content": ["Core rules"], "creation_notes": "Humans only."
            },
            "world": { "mode": "new", "name": "Ember Marches", "origin": "original", "scale": "Region", "era": "Medieval", "notes": "Ash falls." },
            "worldbuilding": [
                { "id": "geography", "answers": [ { "question": "What is the general climate?", "answer": "Cold" } ] },
                { "id": "settlements", "entries": [ { "name": "Cinderford", "description": "**What purpose does it serve?** Trade" } ] }
            ],
            "party": { "expected_players": "4", "origin": "first-session", "players": [ { "player_name": "Sam", "character_name": "Kestrel", "concept": "A scout who lost her watch." } ] },
            "npcs": [ { "name": "Vey", "disposition": "rival", "role": "Rival" } ],
            "creatures": [ { "name": "Cinder Hound", "kind": "Beast", "detail": "Elite" } ],
            "factions": [ { "name": "Ash Council", "kind": "Rule", "detail": "Hostile" } ],
            "story": { "structure": "sandbox", "length": "long" },
            "session": { "play_mode": "hybrid", "dice": "physical", "schedule": "Weekly", "session_length": "3 hours", "session_zero": true }
        }))
        .unwrap();
        create_campaign(&input).unwrap();
        let campaign = read_campaign_manifest(&root).unwrap();
        assert_eq!(campaign.rules.progression_type.as_deref(), Some("tier"));
        assert_eq!(campaign.rules.starting_level.as_deref(), Some("Tier 2"));
        assert_eq!(campaign.rules.house_rules_mode.as_deref(), Some("modified"));
        assert_eq!(
            campaign.rules.character_creation,
            Some(CharacterCreationManifest { allowed_content: vec!["Core rules".into()], notes: Some("Humans only.".into()) })
        );
        assert_eq!(campaign.world.setting_origin.as_deref(), Some("original"));
        assert_eq!(campaign.world.scale.as_deref(), Some("Region"));

        let geo = read_document(&root, Path::new("Worldbuilding/Geography & Climate/Geography & Climate Overview.md")).unwrap();
        assert!(geo.body.contains("## What is the general climate?\n\nCold"));
        let town = read_document(&root, Path::new("Worldbuilding/Settlements & Locations/Cinderford.md")).unwrap();
        assert_eq!(meta(&town, "template_id").and_then(Value::as_str), Some("settlement-city"));
        assert!(town.body.starts_with("**What purpose does it serve?** Trade"));
        let world = read_document(&root, Path::new("Worldbuilding/Ember Marches.md")).unwrap();
        assert_eq!(meta(&world, "scale").and_then(Value::as_str), Some("Region"));
        assert_eq!(campaign.party.expected_players.as_deref(), Some("4"));
        assert_eq!(campaign.party.origin.as_deref(), Some("first-session"));
        assert_eq!(campaign.play.schedule.as_deref(), Some("Weekly"));
        assert_eq!(campaign.play.session_length.as_deref(), Some("3 hours"));
        let text = |path: &str, key: &str| {
            let doc = read_document(&root, Path::new(path)).unwrap();
            meta(&doc, key).and_then(Value::as_str).map(str::to_string)
        };
        assert_eq!(text("Characters/Non-Player Characters/Vey.md", "disposition").as_deref(), Some("rival"));
        assert_eq!(text("Creatures & Monsters/Custom Creatures/Cinder Hound.md", "threat").as_deref(), Some("Elite"));
        assert_eq!(text("Worldbuilding/Organizations & Factions/Ash Council.md", "attitude").as_deref(), Some("Hostile"));
        assert_eq!(text(OVERVIEW_PATH, "campaign_structure").as_deref(), Some("sandbox"));
        let pc = read_document(&root, Path::new("Characters/Player Characters/Kestrel.md")).unwrap();
        assert!(pc.body.starts_with("A scout who lost her watch."));
        let zero = read_document(&root, Path::new("Sessions/Session Plans/Session Zero.md")).unwrap();
        assert!(zero.body.contains("## Session Zero Agenda"));
        assert_eq!(campaign.starting_session_id.as_deref(), Some(zero.identity.id.as_str()));
    }

    #[test]
    fn unknown_builder_choices_are_rejected() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Bad");
        for patch in [
            serde_json::json!({ "setup_mode": "wizard" }),
            serde_json::json!({ "world": { "mode": "new", "origin": "stolen" } }),
            serde_json::json!({ "rules": { "pack_id": "system-neutral", "house_rules_mode": "chaos" } }),
            serde_json::json!({ "worldbuilding": [ { "id": "nations", "entries": [ { "name": " " } ] } ] }),
            serde_json::json!({ "party": { "origin": "teleported" } }),
            serde_json::json!({ "story": { "structure": "railroad" } }),
        ] {
            let mut value = serde_json::json!({ "project_root": root.to_string_lossy(), "name": "Bad", "setup_mode": "guided" });
            for (k, v) in patch.as_object().unwrap() {
                value[k] = v.clone();
            }
            let input: NewCampaignInput = serde_json::from_value(value).unwrap();
            assert!(create_campaign(&input).is_err(), "{patch}");
            assert!(!root.exists());
        }
    }

    #[test]
    fn existing_destinations_are_never_overwritten() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Taken");
        create_campaign(&quick_start(&root)).unwrap();
        let before = read_project_manifest(&root).unwrap();
        assert!(create_campaign(&quick_start(&root))
            .unwrap_err()
            .contains("empty"));
        assert_eq!(read_project_manifest(&root).unwrap(), before);

        let other = parent.path().join("Has files");
        fs::create_dir_all(&other).unwrap();
        fs::write(other.join("notes.txt"), "keep me").unwrap();
        assert!(create_campaign(&quick_start(&other)).is_err());
        assert_eq!(
            fs::read_to_string(other.join("notes.txt")).unwrap(),
            "keep me"
        );
        assert!(!other.join("project.yaml").exists());

        let empty = parent.path().join("Empty");
        fs::create_dir_all(&empty).unwrap();
        create_campaign(&quick_start(&empty)).unwrap();
        assert!(read_campaign_manifest(&empty).is_ok());
    }

    #[test]
    fn failed_publication_restores_destination_and_removes_staging() {
        let parent = tempfile::tempdir().unwrap();
        let staging = parent.path().join(".missing-staging");
        let root = parent.path().join("Target");
        fs::create_dir(&root).unwrap();
        assert!(publish(&staging, &root).is_err());
        assert!(root.is_dir(), "empty destination must be restored");
    }

    #[test]
    fn validation_rejects_bad_input_before_writing() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Bad");
        type Mutate = fn(&mut NewCampaignInput);
        let cases: Vec<(Mutate, &str)> = vec![
            (|i| i.name = " ".into(), "name"),
            (|i| i.rules.pack_id = "dnd".into(), "rules pack"),
            (|i| i.rules.pack_id = "custom-homebrew".into(), "homebrew"),
            (|i| i.world.mode = "shared".into(), "world"),
            (
                |i| {
                    i.worldbuilding.push(WorldCategoryInput {
                        id: "nope".into(),
                        ..Default::default()
                    })
                },
                "category",
            ),
            (|i| i.npcs.push(NpcInput::default()), "NPC"),
            (
                |i| i.locations.entries.push(NamedEntryInput::default()),
                "Location",
            ),
            (|i| i.session.play_mode = "vr".into(), "Tabletop"),
            (|i| i.session.dice = "cards".into(), "Dice"),
        ];
        for (mutate, needle) in cases {
            let mut input = quick_start(&root);
            mutate(&mut input);
            let error = create_campaign(&input).unwrap_err();
            assert!(error.contains(needle), "expected '{needle}' in '{error}'");
        }
        assert!(!root.exists());
        assert_eq!(fs::read_dir(parent.path()).unwrap().count(), 0);
        assert!(
            create_campaign(&quick_start(Path::new("relative/campaign")))
                .unwrap_err()
                .contains("absolute")
        );
        assert!(create_campaign(&quick_start(&parent.path().join("missing/parent/x"))).is_err());
    }

    #[test]
    fn campaign_manifest_must_match_project_manifest() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Mismatch");
        create_campaign(&quick_start(&root)).unwrap();
        let raw = fs::read_to_string(root.join("campaign.yaml")).unwrap();
        let project_id = read_project_manifest(&root).unwrap().project_id;
        fs::write(
            root.join("campaign.yaml"),
            raw.replace(&project_id, &Uuid::new_v4().to_string()),
        )
        .unwrap();
        assert!(read_campaign_manifest(&root)
            .unwrap_err()
            .contains("does not match"));
    }

    #[test]
    fn existing_book_projects_still_open_unchanged() {
        let parent = tempfile::tempdir().unwrap();
        let novel_root = parent.path().join("Novel");
        let manifest = ProjectManifest::new("Old Book", "novel");
        crate::project_root::initialize_project_root_from_manifest(&novel_root, &manifest).unwrap();
        create_campaign(&quick_start(&parent.path().join("Campaign"))).unwrap();

        let novel = project_from_root(&novel_root).unwrap();
        assert_eq!(novel.project_type, "novel");
        assert!(novel_root.join("Manuscript").is_dir());
        assert!(!novel_root.join("campaign.yaml").exists());
        assert!(!novel_root.join("Creatures & Monsters").exists());
        assert!(build_index(&novel_root).unwrap().issues.is_empty());
        assert!(read_campaign_manifest(&novel_root).is_err());
    }
}
