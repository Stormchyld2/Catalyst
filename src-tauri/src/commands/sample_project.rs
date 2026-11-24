//! Self-contained, offline, file-authoritative tabletop RPG demonstration.
//! This replaces the inherited novel sample. Existing projects are never reset.
//! The sample uses the same campaign creation pipeline and template registry as
//! a GM-created campaign, and stays editable under the Catalyst app data folder.

use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::Value;
use tauri::State;
use uuid::Uuid;

use super::AppState;
use crate::campaign_wizard::{self, NewCampaignInput};
use crate::document::service::{create_document, save_document};
use crate::entity_templates::built_in_template;
use crate::models::{Project, SourceType};
use crate::project_registry;
use crate::project_root::read_project_manifest;

const DEMO_TITLE: &str = "The Hollow Crown";
const DEMO_SUBDIR: &str = "Demo Campaigns";

/// Entry paths are relative to the ordinary campaign folder tree, not a new
/// proprietary demo schema. The same stable-ID Document Service creates them.
struct DemoEntry {
    path: &'static str,
    template: &'static str,
    title: &'static str,
    body: &'static str,
    visibility: &'static str,
}

const ENTRIES: &[DemoEntry] = &[
    DemoEntry {
        path: "Adventures/Main Campaign/START HERE - GM Guide.md",
        template: "gm-guide",
        title: "START HERE - GM Guide",
        visibility: "gm-only",
        body: r#"# Welcome to The Hollow Crown
This **original, system-neutral** one-shot introduces World Builder, quests, encounters, sessions, clues and reusable templates. Allow roughly 2–3 hours. It does **not** assume D&D or require an installed rules pack.

## Your opening
The village of Briar Glen has lost its bell and three residents have begun sleepwalking toward Briarwatch Tower. The players are asked to investigate before sunset.

## Run the adventure
1. Read *Session 1 - The Silent Bell* in Sessions/Session Plans.
2. Start at Briar Glen, roleplay with Mira Fen and Orin Hale.
3. Give the players the bell fragment handout and let them investigate the tower.
4. At the vault, present three endings. Do not force a fight.
5. Record discoveries in Session Logs and update the campaign state.

## Three possible resolutions
- **Restore the seal:** the sleepers wake; the Ash Wardens demand ongoing protection of the tower.
- **Break the seal:** the sleepers return, but strange echoes begin to emerge from the valley.
- **Bargain with the imprisoned voice:** the sleepers wake at a cost chosen by the GM and players.

## GM secrets
The villagers' disappearances are an effect of the weakening ward, not intentional kidnappings. Kael Voss wants the Crown Seal because it will reveal a lost royal archive. The imprisoned Hollow King can lie but cannot cross the etched stone ring while the seal remains whole.

## Demo safety and mechanics
Use descriptive obstacles, negotiations and simple opposed checks appropriate for the players' system. No mandatory combat statistics or rulebooks. All outcomes are suggestions, not hard-coded mechanics. The supplied map references are textual location diagrams, **not** finished Map Studio battle maps. No working VTT player connection, soundboard configuration, token or automatic encounter system is implied.
"#,
    },
    DemoEntry {
        path: "Adventures/Main Campaign/The Hollow Crown - Adventure Overview.md",
        template: "adventure-overview",
        title: "The Hollow Crown - Adventure Overview",
        visibility: "gm-only",
        body: r#"# Adventure Summary
In the shadow of Briarwatch Tower, the old bell has gone silent and three villagers wander in their sleep. A broken crown-shaped seal in a hidden vault holds back an ancient echo called the Hollow King.

## Player Hook
Village steward Mira Fen offers safe lodging and a modest reward to anyone who investigates the missing bell and returns the missing villagers before nightfall.

## Objectives
- Speak to the witnesses and learn why the bell matters.
- Reach Briarwatch Tower and find the mechanism beneath it.
- Decide the fate of the Crown Seal and the imprisoned voice.

## Scene Sequence
1. The Silent Bell (social/investigation)
2. The Briarwatch Ascent (exploration)
3. The Crown Seal (negotiation, optional confrontation)

## Branches and Consequences
The party may ally with the Ash Wardens, trust Kael Voss, or reject both factions. The finale must allow restoration, destruction or bargain rather than a predetermined boss fight.

## Rewards
Gratitude of Briar Glen, a ceremonial Crown Shard, and potentially access to the lost archive.

## GM-Only Truth
The seal is failing because someone removed a bronze tuning weight, not because the sleepers were abducted.
"#,
    },
    DemoEntry {
        path: "Adventures/Scenes/Scene 1 - The Silent Bell.md",
        template: "adventure-scene",
        title: "Scene 1 - The Silent Bell",
        visibility: "gm-only",
        body: r#"## Opening description
Mist gathers around the rooflines of Briar Glen. The tower bell should announce sunset, but the village is silent. A woman in a wool cloak stands beside an empty bell frame.

## Player choices
Speak with Mira Fen, question the exhausted guard Orin Hale, inspect the bell frame, or follow the trail of bare footprints.

## Clues
- The bell's bronze tuning weight is missing.
- The sleepers all heard singing near the old road.
- The carved crown mark on the broken bell matches a mark on the watchtower door.

## Consequences
Friendly questioning earns a key to the upper tower. Threatening villagers makes Orin suspicious but does not halt the adventure.

## Next
Scene 2 - The Briarwatch Ascent.
"#,
    },
    DemoEntry {
        path: "Adventures/Scenes/Scene 2 - The Briarwatch Ascent.md",
        template: "adventure-scene",
        title: "Scene 2 - The Briarwatch Ascent",
        visibility: "gm-only",
        body: r#"## Setup
The abandoned tower is accessible by a narrow overgrown road. A loose stone stair, wind-driven chimes and a thornbound guardian obstruct the way.

## Exploration
The characters can find old maintenance markings, a hidden lever and a sealed staircase. Reward clever investigation rather than requiring a particular die result.

## Optional confrontation
The Thornbound Sentinel attacks only those who damage the ward carvings. It can be distracted, bypassed, pacified or fought using the selected RPG system.

## Discovery
The lower vault is not a prison for villagers. It is a prison for an echo preserved in the Crown Seal.

## Next
Scene 3 - The Crown Seal.
"#,
    },
    DemoEntry {
        path: "Adventures/Scenes/Scene 3 - The Crown Seal.md",
        template: "adventure-scene",
        title: "Scene 3 - The Crown Seal",
        visibility: "gm-only",
        body: r#"## Strong opening
A circular chamber holds a fractured bronze crown suspended in a ring of pale light. Three sleepwalkers stand motionless outside the ring. A voice speaks from every direction: "Give me my name, and I will return theirs."

## Decision
The Ash Wardens demand repair; Kael Voss argues for opening the archive; the Hollow King offers the party secret knowledge in exchange for loosening the seal.

## Outcomes
**Restore:** recover the missing weight and align three engraved marks. The sleepers wake; the ward remains.
**Break:** shatter the ring. The sleepers wake; the valley becomes vulnerable to echoes.
**Bargain:** exchange a remembered secret, promise or future duty. The sleepers wake; the GM records the cost as an open thread.

## Aftermath
Write down the chosen resolution in the Session Log and Campaign State Event template. Do not automatically commit any outcome during demo installation.
"#,
    },
    DemoEntry {
        path: "Adventures/Quest Chains/The Silent Bell - Quest Chain.md",
        template: "quest-chain",
        title: "The Silent Bell - Quest Chain",
        visibility: "gm-only",
        body: r#"## Quest Sequence
1. Learn what happened to the bell in Briar Glen.
2. Locate Briarwatch Tower and enter the lower vault.
3. Choose what happens to the Crown Seal.

## Optional branches
Find the missing tuning weight, reconcile the two factions, or search for the lost royal archive.

## Consequences
Record the party's choice in Sessions/Campaign State; none of the three possible endings is automatically canon.
"#,
    },
    DemoEntry {
        path: "Adventures/Puzzles & Traps/The Three Crown Marks.md",
        template: "puzzle-profile",
        title: "The Three Crown Marks",
        visibility: "gm-only",
        body: r#"## Setup
Three stone discs bear symbols of a seed, a flame and a crown. A worn inscription reads: "Life before fire, duty beyond both."

## Solution
Turn the discs into seed → flame → crown order to align the ward and restore the seal.

## Hints
A mural shows a tree growing, a forge burning, then a ruler kneeling. The players may also seek the Ash Warden's oral explanation.

## Alternative solutions
Mechanically stabilize the bell, redirect the sound through the chamber or negotiate with the imprisoned voice. Do not block progress behind this single puzzle.

## Consequences
A wrong sequence awakens a Hollow Echo but never permanently locks the chamber.
"#,
    },
    DemoEntry {
        path: "Encounters/Social/The Witness at the Well.md",
        template: "encounter-card",
        title: "The Witness at the Well",
        visibility: "gm-only",
        body: r#"## Setup
Old Jessa saw one of the sleepers carrying a bronze weight toward the tower. She fears being accused of theft.

## Participants
Jessa (witness), Mira Fen (ally), player characters.

## Approaches
Kindness, evidence, a promise of protection or a perceptive question will reveal the information. No specific game-system skill check is required.

## Outcome
The party learns the weight is hidden in the collapsed signal shed.
"#,
    },
    DemoEntry {
        path: "Encounters/Exploration/The Wind Chimes.md",
        template: "encounter-card",
        title: "The Wind Chimes",
        visibility: "gm-only",
        body: r#"## Setup
Bronze chimes ring out of sequence along the tower stairs, interfering with concentration and masking a door.

## Interactions
Listen for repeated notes, block the wind, inspect the etched markings or climb to the upper landing.

## Success
Find the hidden stair down to the Crown Seal.

## Complication
A Hollow Echo manifests if the chimes are violently smashed; adapt the severity to the selected ruleset.
"#,
    },
    DemoEntry {
        path: "Encounters/Combat/The Thornbound Guardian.md",
        template: "encounter-card",
        title: "The Thornbound Guardian",
        visibility: "gm-only",
        body: r#"## Setup
A wicker-and-root sentinel watches the vault entrance. It guards carved ward symbols rather than treasure.

## Combat is optional
The party may bypass it by restoring a fallen symbol, speaking the old watchword "Hold the line", or taking a quieter route.

## If fighting
Use creatures or stat blocks from the group's chosen ruleset. The demo provides no unlicensed combat statistics.

## Outcome
The sentinel retreats once the ward is secure or its protected threshold is passed.
"#,
    },
    DemoEntry {
        path: "Game Master/Secrets & Clues/The Missing Tuning Weight.md",
        template: "secret-clue",
        title: "The Missing Tuning Weight",
        visibility: "gm-only",
        body: r#"## GM-Only Truth
The bronze weight was carried into the signal shed by a sleepwalker. It is neither stolen by Kael nor destroyed.

## Discovery conditions
The players may find footprints, hear Jessa's testimony or inspect the bell frame.

## Player-facing information
The weight is needed to align the Crown Seal. The clue supports multiple routes into the finale.
"#,
    },
    DemoEntry {
        path: "Game Master/Secrets & Clues/Kael's Real Purpose.md",
        template: "secret-clue",
        title: "Kael's Real Purpose",
        visibility: "gm-only",
        body: r#"## GM-Only Truth
Kael Voss is a scholar and Gloam Seeker agent. He wants the lost royal archive, not the sleepers' lives. He is willing to bargain.

## Player-facing clues
His notes mention archive shelves beneath the vault rather than a throne. A challenge to his motives may turn him into an ally.
"#,
    },
    DemoEntry {
        path: "Game Master/Secrets & Clues/The Hollow King's Limit.md",
        template: "secret-clue",
        title: "The Hollow King's Limit",
        visibility: "gm-only",
        body: r#"## GM-Only Truth
The imprisoned voice can tempt and mislead, but cannot move beyond the ring while the ward is intact.

## Discovery
Ancient inscriptions, the Ash Warden's testimony or observation of the voice's behavior reveal the rule.

## Payoff
The party can bargain from a position of safety if they keep the ring intact.
"#,
    },
    DemoEntry {
        path: "Game Master/Handouts/Fragment from the Bell.md",
        template: "handout",
        title: "Fragment from the Bell",
        visibility: "public",
        body: r#"# A fragment of old bronze
> When the crown falls silent, three shall walk.
> Let seed wake flame; let flame remember duty.
> No king is greater than the bell.

Hand this text to the players after they inspect the bell frame. Keep the solution to the disc puzzle in GM-only notes.
"#,
    },
    DemoEntry {
        path: "Items & Equipment/Treasure & Loot/The Crown Shard.md",
        template: "loot-treasure",
        title: "The Crown Shard",
        visibility: "gm-only",
        body: r#"## Contents
A palm-sized bronze fragment with a faint, repeating chime.

## Discovery
Found beside the ring after a peaceful resolution.

## Effect
Narrative keepsake and possible future quest hook. It has **no automatic mechanical bonuses** until the GM and selected ruleset define them.

## Ownership
Awarded to the party if appropriate.
"#,
    },
    DemoEntry {
        path: "Worldbuilding/Settlements & Locations/Briarwatch Tower Site.md",
        template: "dungeon-site",
        title: "Briarwatch Tower Site",
        visibility: "gm-only",
        body: r#"## Rooms and Areas
1. Overgrown Road — footprints and torn blue ribbon.
2. Watch Landing — broken chimes and sentinel.
3. Signal Shed — missing bronze weight.
4. Spiral Stair — ward markings.
5. Crown Vault — sleepers, seal and final choice.

## Map
See Maps/Dungeons/Briarwatch Tower - Text Map.md. It is an original schematic, not a battle-map image.

## Encounter options
Roleplay, exploration, or combat; scale to the table's chosen system.

## Treasure
The Crown Shard, a harmless old bell-clapper, and a torn ledger page.
"#,
    },
    DemoEntry {
        path: "Game Master/Random Tables/Briarwatch Rumors.md",
        template: "random-table",
        title: "Briarwatch Rumors",
        visibility: "gm-only",
        body: r#"# Roll 1d6 — Rumors
| d6 | Rumor | GM truth |
| --- | --- | --- |
| 1 | The tower steals names. | Partly true: the echo blurs memory. |
| 2 | The steward hid the bell. | False. |
| 3 | A sleeper took a bronze weight. | True. |
| 4 | The Crown Seal is a royal treasure. | Misleading: it is a ward. |
| 5 | The guardian fears fire. | False: it protects ward carvings. |
| 6 | The old archive lies under the tower. | True. |

Use Catalyst dice when available, or physical dice. The Markdown table is not wired to automatic roll resolution.
"#,
    },
    DemoEntry {
        path: "Sessions/Session Plans/Session 1 - The Silent Bell.md",
        template: "session-plan",
        title: "Session 1 - The Silent Bell",
        visibility: "gm-only",
        body: r#"# Opening Session
**Play time:** roughly 2–3 hours. **Rules:** system-neutral.

## Strong Start
The village bell is silent at sunset. Mira Fen asks the party to find the missing villagers.

## Possible Scenes
- Briar Glen interviews — social play and first clue.
- The tower ascent — exploration and optional guardian.
- Crown Vault — final conversation and decisive choice.

## Key Clues
Missing tuning weight; Kael's archive motive; the seal's protective purpose.

## Session Goal
Reach the vault, identify the cause of sleepwalking, and choose an outcome. Stop before the ending only if the group needs more time.

## Session End
Use the Session Log template after play. The demo intentionally contains no fabricated past session events.
"#,
    },
    DemoEntry {
        path: "Game Master/GM Notes/GM Safety and Continuity.md",
        template: "gm-notes",
        title: "GM Safety and Continuity",
        visibility: "gm-only",
        body: r#"# GM Guidelines
Ask players which themes they want to avoid or fade to black. Use a brief Session Zero if this is a new table. The mystery should never depend on forcing characters to lose memories.

## Continuity
The opening state is unresolved. Nothing in the sample campaign records player choices or outcomes until the GM edits the documents.

## Optional challenge
Let the party use the game's normal checks if a system is installed. There are always noncombat routes.
"#,
    },
];

const LOCATION_MAP: &str = r#"# Briarwatch Tower — schematic only
This is a text diagram for the GM, **not** a generated Map Studio asset or a grid-accurate battle map.

                        [Upper Bell Landing]
                                 |
[Road from Briar Glen] -- [Watch Landing] -- [Signal Shed]
                                 |
                            [Spiral Stair]
                                 |
                             [Crown Vault]

## Connections
The signal shed contains the missing tuning weight. The spiral stair leads to the vault. The guardian watches the stair but can be bypassed.
"#;

fn input_for(root: &Path) -> Result<NewCampaignInput, String> {
    let payload = serde_json::json!({
        "project_root": root.to_string_lossy().to_string(),
        "name": DEMO_TITLE,
        "setup_mode": "guided",
        "basics": {
            "subtitle": "An Introductory Catalyst Adventure",
            "premise": "The bell of Briarwatch Tower has fallen silent, and villagers are walking in their sleep toward an ancient crown-shaped seal.",
            "genre": "Fantasy Mystery",
            "tone": "Mysterious, heroic, choice-driven",
            "gm_name": "Demo Game Master"
        },
        "rules": { "pack_id": "system-neutral", "advancement": "GM-directed" },
        "world": {
            "mode": "new", "name": "The Briar Marches",
            "era": "Frontier Age", "origin": "original",
            "notes": "A misty borderland where forgotten royal wards protect small villages from echoes."
        },
        "party": { "name": "The Bellward Company", "expected_players": "3–5" },
        "npcs": [
            {"name":"Mira Fen","disposition":"ally","role":"Village steward","motivation":"Protect Briar Glen and the missing villagers","description":"Practical, plain-spoken steward who offers shelter and a reward. She knows the bell was used to keep night terrors away."},
            {"name":"Orin Hale","disposition":"neutral","role":"Village guard captain","motivation":"Keep the road safe and avoid panic","description":"Sleep-deprived captain who knows the bell's tuning weight disappeared."},
            {"name":"Elowen Thatch","disposition":"ally","role":"Keeper of the Ash Warden traditions","motivation":"Repair the ancient seal without sacrificing villagers","description":"An elderly ward-keeper who understands three carved crown marks."},
            {"name":"Kael Voss","disposition":"neutral","role":"Scholar and Gloam Seeker agent","motivation":"Enter the lost royal archive beneath the tower","description":"A persuasive scholar who knows more about the vault than he admits."}
        ],
        "creatures": [
            {"name":"Thornbound Sentinel","kind":"Guardian construct","detail":"Variable, ruleset-defined","description":"A wicker-and-root guardian that protects the stairwell. It responds to ward signs and can be bypassed."},
            {"name":"Hollow Echo","kind":"Supernatural echo","detail":"Variable, ruleset-defined","description":"A brief manifestation of lost memories. It is a hazard or optional creature, not a mandatory fight."}
        ],
        "factions": [
            {"name":"The Ash Wardens","kind":"Maintain the ancient seal","detail":"Potential allies","description":"A scattered group that protects frontier communities from echoes; Elowen preserves their traditions."},
            {"name":"The Gloam Seekers","kind":"Recover royal history at any cost","detail":"Ambiguous rivals","description":"Archivists and seekers led locally by Kael Voss, willing to risk opening the vault."}
        ],
        "story": {
            "structure":"mixed", "length":"one-shot",
            "central_conflict":"A failing protective seal threatens a village, while rival factions dispute whether it should be restored or opened.",
            "starting_adventure":"The Silent Bell",
            "starting_adventure_summary":"Investigate the quiet bell, explore Briarwatch Tower and decide the fate of the Crown Seal.",
            "side_quests":["The Missing Weight: trace the bronze tuning weight to a ruined signal shed."]
        },
        "locations": {
            "starting_location":"Briar Glen",
            "entries":[
                {"name":"Briar Glen","kind":"Frontier village","description":"A hamlet of timber homes beside the old watch road. The villagers fear the silent bell."},
                {"name":"Briarwatch Tower","kind":"Abandoned watchtower","description":"A stone tower with broken bells, wind chimes, a guardian and the vault below."},
                {"name":"Crown Vault","kind":"Ancient sealed chamber","description":"A chamber of stone rings and a hovering broken crown-shaped ward."}
            ]
        },
        "session": {
            "play_mode":"physical", "dice":"physical",
            "first_session_title":"Session One - GM Preparation",
            "first_session_notes":"Use Sessions/Session Plans/Session 1 - The Silent Bell for the complete runnable adventure plan.",
            "gm_notes":"All secrets are GM-only by default. This is an untouched opening state; the demo does not pretend any players have already completed a session."
        }
    });
    serde_json::from_value(payload).map_err(|e| format!("Invalid bundled adventure input: {e}"))
}

fn seed_entry(root: &Path, entry: &DemoEntry) -> Result<(), String> {
    let template = built_in_template(entry.template)
        .ok_or_else(|| format!("Missing RPG template '{}'.", entry.template))?;
    let relative = Path::new(entry.path);
    let mut document = create_document(
        root, relative, &template.manifest.document_type,
        Some(entry.title), entry.body
    )?;
    document.metadata.insert("template_id".into(), Value::String(entry.template.into()));
    if let Some(subtype) = template.manifest.subtype {
        document.metadata.insert("subtype".into(), Value::String(subtype));
    }
    document.metadata.insert("visibility".into(), Value::String(entry.visibility.into()));
    save_document(root, &document, Some(&document.source_hash))
}

/// A successful re-open never overwrites user edits, rolls or local session data.
/// First creation is staged in a sibling directory before being published.
fn ensure_demo(root: &Path) -> Result<(), String> {
    if root.exists() {
        campaign_wizard::read_campaign_manifest(root)?;
        return Ok(());
    }
    let parent = root.parent().ok_or("Missing demo parent folder")?;
    fs::create_dir_all(parent).map_err(|e| format!("Could not make demo folder: {e}"))?;
    let staging = parent.join(format!(".hollow-crown-demo-{}", Uuid::new_v4().simple()));
    let result = (|| {
        let input = input_for(&staging)?;
        campaign_wizard::create_campaign(&input)?;
        for entry in ENTRIES {
            seed_entry(&staging, entry)?;
        }
        let map = staging.join("Maps/Dungeons/Briarwatch Tower - Text Map.md");
        fs::write(&map, LOCATION_MAP).map_err(|e| format!("Could not write sample map: {e}"))?;
        let _ = campaign_wizard::read_campaign_manifest(&staging)?;
        if root.exists() {
            return Err("Demo folder was created by another process. Nothing was overwritten.".into());
        }
        fs::rename(&staging, root).map_err(|e| format!("Could not publish demo: {e}"))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn load_demo_project(root: &Path) -> Result<Project, String> {
    let manifest = read_project_manifest(root)?;
    if manifest.project_type != "campaign" {
        return Err("Adventure demo is not a campaign project.".into());
    }
    let _ = campaign_wizard::read_campaign_manifest(root)?;
    let id = Uuid::parse_str(&manifest.project_id)
        .map_err(|e| format!("Invalid adventure demo ID: {e}"))?;
    Ok(Project {
        id,
        name: manifest.title,
        source_type: SourceType::Blank,
        source_path: None,
        project_root: Some(root.to_string_lossy().into_owned()),
        created_at: manifest.created_at.clone(),
        modified_at: manifest.modified_at.unwrap_or(manifest.created_at),
        author_pen_name: None,
        genre: Some("System-Neutral Fantasy Mystery".into()),
        description: Some("A playable, choice-driven fantasy adventure demonstrating Catalyst's campaign and GM templates.".into()),
        word_target: None,
        reference_types: Project::default_reference_types(),
        project_type: manifest.project_type,
        target_page_count: None,
    })
}

#[tauri::command]
pub async fn create_sample_project(state: State<'_, AppState>) -> Result<Project, String> {
    let root = state.app_data_dir.join(DEMO_SUBDIR).join(DEMO_TITLE);
    ensure_demo(&root)?;
    let project = load_demo_project(&root)?;
    project_registry::remember(&state.app_data_dir, project.id, &PathBuf::from(&root))?;
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adventure_demo_is_playable_and_preserves_user_edits() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join(DEMO_SUBDIR).join(DEMO_TITLE);
        ensure_demo(&root).unwrap();
        let project = load_demo_project(&root).unwrap();
        assert_eq!(project.project_type, "campaign");
        assert_eq!(project.name, DEMO_TITLE);
        assert!(root.join("campaign.yaml").exists());
        assert!(root.join("Adventures/Scenes/Scene 3 - The Crown Seal.md").exists());
        assert!(root.join("Encounters/Social/The Witness at the Well.md").exists());
        assert!(root.join("Game Master/Handouts/Fragment from the Bell.md").exists());
        assert!(root.join("Sessions/Session Plans/Session 1 - The Silent Bell.md").exists());
        assert!(!root.join("Manuscript").exists());
        let notes = root.join("Sessions/Session Logs/Actual play.md");
        fs::write(&notes, "The party chose to restore the seal.").unwrap();
        ensure_demo(&root).unwrap();
        assert_eq!(fs::read_to_string(&notes).unwrap(), "The party chose to restore the seal.");
        assert_eq!(load_demo_project(&root).unwrap().id, project.id);
    }

    #[test]
    fn invalid_existing_folder_is_never_destroyed() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join(DEMO_SUBDIR).join(DEMO_TITLE);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("my-notes.md"), "Important").unwrap();
        assert!(ensure_demo(&root).is_err());
        assert_eq!(fs::read_to_string(root.join("my-notes.md")).unwrap(), "Important");
    }

    #[test]
    fn all_demo_entries_use_registered_templates_and_unique_paths() {
        let mut paths = std::collections::BTreeSet::new();
        for entry in ENTRIES {
            assert!(paths.insert(entry.path));
            let template = built_in_template(entry.template).expect("registered template");
            assert!(!template.manifest.document_type.is_empty());
        }
    }
}
