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
This **original Dungeons & Dragons 5th Edition (2014-style)** one-shot introduces World Builder, quests, encounters, sessions, clues and reusable templates. Allow roughly 2–3 hours. It is written for **four 1st-level D&D 5e characters** (adjustable for three to five PCs). Players supply their own fifth-edition rules; no proprietary rules pack is bundled. This is an independent original adventure, not an official D&D product.

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
Use D&D 5e ability checks, saving throws, initiative and the original monster statistics provided in this demo. Checks and attacks use a physical die or Catalyst Dice Roller; there is no automatic 5e combat adapter bundled. All outcomes are suggestions, not hard-coded mechanics. The supplied map references are textual location diagrams, **not** finished Map Studio battle maps. No working VTT player connection, soundboard configuration, token or automatic encounter system is implied.
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
Kindness, evidence, a promise of protection or a perceptive question will reveal the information. Suggested checks: Charisma (Persuasion) DC 10 to gain Jessa's confidence, Wisdom (Insight) DC 10 to notice her fear, or Intelligence (Investigation) DC 11 to examine tracks. An honest promise of protection succeeds without a roll.

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
A Hollow Echo manifests if the chimes are violently smashed; use the single low-powered 5e Hollow Echo; allow avoidance or retreat.
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
Use the original Thornbound Sentinel statistics in Creatures & Monsters/Custom Creatures/Thornbound Sentinel - 5e Stat Block.md. Its encounter is optional.

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
        visibility: "gm-only",
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
**Play time:** roughly 2–3 hours. **Rules:** D&D 5e, four 1st-level characters, 2014-style encounter assumptions.

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
        path: "Creatures & Monsters/Custom Creatures/Thornbound Sentinel - 5e Stat Block.md",
        template: "creature-profile",
        title: "Thornbound Sentinel - 5e Stat Block",
        visibility: "gm-only",
        body: r#"# Thornbound Sentinel
*Medium plant construct, unaligned* — **Armor Class 12** (woven bark), **Hit Points 11** (2d8+2), **Speed 25 ft.**

| STR | DEX | CON | INT | WIS | CHA |
| --- | --- | --- | --- | --- | --- |
| 14 (+2) | 10 (+0) | 12 (+1) | 3 (−4) | 11 (+0) | 5 (−3) |

**Condition Immunities:** charmed, frightened. **Senses:** passive Perception 10. **Languages:** understands the old watchword but cannot speak. **Challenge:** approximately CR 1/8 (25 XP), GM-adjustable; this is an original example creature, not a published D&D monster.

## Traits
**Warden's Purpose.** The sentinel attacks only creatures deliberately damaging the protective ward symbols. A creature can use an action and succeed on a DC 10 Intelligence (Arcana) or Wisdom (Religion) check to show the sentinel that it is repairing a ward. A player who correctly restores a fallen symbol succeeds automatically.

## Actions
**Thorn Strike.** Melee Weapon Attack: +3 to hit, reach 5 ft., one target. Hit: 3 (1d4+1) piercing damage.

## Encounter Scaling
For a party of three level-1 characters, use 8 hit points; for five level-1 characters use 14. The sentinel can be bypassed without fighting. Do not treat the rough challenge estimate as an officially calculated CR.
"#,
    },
    DemoEntry {
        path: "Creatures & Monsters/Custom Creatures/Hollow Echo - 5e Stat Block.md",
        template: "creature-profile",
        title: "Hollow Echo - 5e Stat Block",
        visibility: "gm-only",
        body: r#"# Hollow Echo
*Small undead manifestation, neutral* — **Armor Class 12**, **Hit Points 7** (2d6), **Speed 0 ft., fly 30 ft. (hover).**

| STR | DEX | CON | INT | WIS | CHA |
| --- | --- | --- | --- | --- | --- |
| 4 (−3) | 14 (+2) | 10 (+0) | 8 (−1) | 12 (+1) | 11 (+0) |

**Condition Immunities:** prone. **Senses:** darkvision 60 ft., passive Perception 11. **Languages:** understands Common; communicates with fragments of remembered words. **Challenge:** approximately CR 1/8 (25 XP), GM-adjustable; original example, not published material.

## Actions
**Echo Touch.** Melee Spell Attack: +3 to hit, reach 5 ft., one target. Hit: 3 (1d4+1) psychic damage.

## Fractured Memory (Recharge 5–6)
One creature within 30 feet must succeed on a DC 10 Wisdom saving throw or have disadvantage on its next ability check before the end of its next turn. This does not remove memories or control player characters.

## Encounter Scaling
Use at most one echo at a time for a 1st-level party; avoid running it immediately after the sentinel. The echo disperses automatically when the seal is restored.
"#,
    },
    DemoEntry {
        path: "Game Master/Screen References/D&D 5e Beginner Rules for The Hollow Crown.md",
        template: "gm-screen-reference",
        title: "D&D 5e Beginner Rules for The Hollow Crown",
        visibility: "gm-only",
        body: r#"# Adventure-Specific D&D 5e Checks
Designed for four **1st-level characters**, based on 2014-style fifth edition. The players should have access to their own D&D rules and character sheets.

| Situation | Suggested check | DC | Success |
| --- | --- | --- | --- |
| Inspect bell frame | Intelligence (Investigation) | 10 | Notice the missing weight |
| Follow sleepwalkers' tracks | Wisdom (Survival) | 10 | Find path to signal shed |
| Calm Jessa | Charisma (Persuasion) | 10 | She reveals a witness account |
| Recognize the old ward | Intelligence (Arcana) or Wisdom (Religion) | 11 | Learn the ward is protective |
| Find hidden stair | Wisdom (Perception) | 11 | Discover mechanism |
| Stabilize bell chimes | Dexterity using suitable tools | 10 | Avoid an echo encounter |
| Align Crown Seal | Intelligence (Arcana) | 11 | Restore the ward |

## Fail-Forward
A failed check adds time, noise, a complication or a new bargain; it must never permanently block the main story. Strong in-fiction choices may succeed automatically.

## Final Chamber
The seal can be restored by solving the three-marks puzzle, repaired through an appropriate creative method, destroyed, or negotiated over. There is no compulsory boss fight.

## Level-One Survival Guidance
The guardian and echo are alternatives, not a gauntlet. Telegraph danger, allow retreat, and let clear nonviolent solutions work without rolling. Avoid unexpected lethal damage to new players.

## Combat
Use the original Thornbound Sentinel and Hollow Echo stat-block documents. All other NPCs are narrative unless the GM chooses to supply suitable legal 5e statistics. Do not assume automatic initiative, HP tracking or character integration in Catalyst.

## Party Size
For three PCs, use 8 hit points for the sentinel; for five PCs use 14. Use only one hostile creature at a time. Never force combat, and avoid an echo battle immediately after the sentinel.
"#,
    },
    DemoEntry {
        path: "Items & Equipment/Treasure & Loot/5e Adventure Rewards.md",
        template: "loot-treasure",
        title: "5e Adventure Rewards",
        visibility: "gm-only",
        body: r#"# Optional D&D 5e Rewards
Intended for four level-1 adventurers:
- 20 gp from Mira Fen's village fund, split as the party chooses.
- One potion of healing (standard fifth-edition common item) found in the tower's signal shed, subject to the GM's available rules source.
- The Crown Shard, a **nonmagical** narrative artifact with a future plot hook.
- Optional milestone advancement after a satisfactory resolution, at the GM's discretion.

No reward is granted automatically by the Catalyst inventory system. Add items and currency in Character Studio when integrated, or track them on paper.
"#,
    },
    DemoEntry {
        path: "Sessions/Session Plans/D&D 5e Encounter and Session Timing.md",
        template: "session-plan",
        title: "D&D 5e Encounter and Session Timing",
        visibility: "gm-only",
        body: r#"# GM Session Clock (2–3 hours)
- **0–25 minutes:** intro, hook and village witnesses.
- **25–60 minutes:** road and tower exploration, find tuning-weight clue.
- **60–100 minutes:** guardian negotiation or optional combat; enter vault.
- **100–145 minutes:** puzzle, rival-faction scene and decisive bargain.
- **145–180 minutes:** consequences, rewards and session log.

## Expected Challenges
One or two low-stakes social DC 10–12 checks, one short exploration sequence, one puzzle with multiple solutions, and at most one short optional combat. Tune difficulty after seeing the characters' actual capabilities.

## Character Options
For a demonstration group, encourage diverse roles (frontline, investigator, healer/support and negotiator). Catalyst does not bundle pre-generated D&D character sheets.

## Session Zero
Explain sleepwalking, memory themes and player safety boundaries. Players may request alternatives without affecting success or rewards.
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


/// Expansion v2 contains complete running notes, premade options and keyed maps.
/// Existing campaign users receive missing records without overwriting edits.
const EXPANDED_ENTRIES: &[DemoEntry] = &[
    DemoEntry {
        path: "Adventures/Main Campaign/COMPLETE ADVENTURE - Run from Start to Finish.md",
        template: "gm-guide",
        title: "COMPLETE ADVENTURE - Run from Start to Finish",
        visibility: "gm-only",
        body: r###"# THE HOLLOW CROWN
**Complete first-level D&D 5e starter adventure • 4 characters (3–5 adjustable) • 3–4 hours • original adventure**

## What the GM prepares
Read this walkthrough, the three numbered scene dossiers, and the three ending cards. Have the three maps open. Give players the four premade character cards if needed. A physical d20 and ordinary 5e character sheets also work. *Catalyst does not contain an official D&D 5e rules pack or automate character stats.*

## The truth (GM only)
Briarwatch was built atop a vault holding a fragment of an ancient king's memories. The Crown Seal and village bell form a protective resonance. During a sleepwalking episode, one villager carried the bell's **bronze tuning weight** to the old signal shed. Without its counterweight the seal weakens and draws three villagers to the vault. The Ash Wardens seek restoration; the Gloam Seekers seek the archive beyond the ward. Scholar Kael Voss does **not** control the sleepers; the Hollow King's voice is cunning but bound by the ring. All three sleepers are alive and recover regardless of the final choice.

## Act 1: Briar Glen (45–60 minutes)
Read the village opening aloud from *Scene One - GM Dossier*. Mira hires the party for **20 gp total** plus lodging on safe return. The bell frame has a broken ring and matching crown carving. Orin can give the old tower key. Jessa saw a sleepwalker with the heavy bronze weight heading to the signal shed. Present these through roleplay; failed checks add complications, never conceal required clues. Hand out *Fragment from the Bell* once inspected. Allow a quick visit to the mill or old shrine if the players seek more information.

## Act 2: Road, tower and shed (60–90 minutes)
Present *Scene Two - GM Dossier* and the Watchtower map. Three routes: overgrown road, rear drainage channel, or steep ridge. The **signal shed** holds the missing weight and a potion of healing. The Thornbound Sentinel guards the inner landing and need not be fought. The chimes point to the spiral stair. Give characters warnings before any harm. If combat occurs use only **one** sentinel and avoid a second forced encounter.

## Act 3: Crown Vault (45–60 minutes)
Use *Scene Three - GM Dossier*, Vault Map, and the puzzle. Three motionless sleepers stand *outside* the ring. Kael may arrive to plead for archive access. Elowen counsels repair. The Hollow King bargains through the seal. Players may restore the seal, break it, bargain with the voice, or devise an equivalent fourth option. Every ending frees the villagers, and none requires fighting a boss.

## Ending & rewards (15–20 minutes)
Read the corresponding ending card. Award **20 gp total**, optional potion if not consumed, and a nonmagical Crown Shard if earned. Advancement after the one-shot is a GM choice; no automatic character progression occurs. Record actual decisions in Session Log. **Do not seed a fictional completed log.**

## Standard 5e checks (suggested only)
- Investigation DC 10: notice the missing weight; automatically reveal with close inspection.
- Persuasion DC 10: reassure Jessa; honesty and care succeed without rolling.
- Survival DC 10: follow wet prints to the signal shed; on failure takes longer.
- Perception DC 11: find the stair; on failure the chimes give an obvious clue.
- Arcana or Religion DC 11: understand the protective ward.
- Arcana DC 11 or puzzle solution: align the crown marks; creative tool use also works.

## Difficulty protection at level 1
Never place more than one starter creature in front of the party at once. Use an optional sentinel **or** an optional echo; after any fight, skip additional hostile creatures. Enemies hesitate, telegraph attacks, and can be pacified or fled. Adapt if the party is weaker, wounded or new to the rules.

## Where to go next
All supporting material is stored in normal Catalyst campaign folders, with reusable RPG templates and editable Markdown. Maps are simple **labeled schematic diagrams**, not VTT-calibrated tactical artwork. No internet, D&D proprietary content, automatic encounter tracking or player sync is required.
"###,
    },
    DemoEntry {
        path: "Adventures/Scenes/Scene One - GM Dossier.md",
        template: "adventure-scene",
        title: "Scene One - GM Dossier",
        visibility: "gm-only",
        body: r###"# Scene One: The Silent Bell
**Place:** Briar Glen • **Time:** dusk • **Goal:** players understand the problem and choose their approach

## Read aloud
> A pale evening mist rolls between crooked wooden rooftops. An empty bell frame hangs over the square. The hammer is still, though the wind pushes against it. At the well, a lantern-bearing woman calls, “You there—please. Three of ours walked into the fog, and we cannot get the bell to ring.”

## Running the village
Mira Fen speaks plainly, offers 20 gp total and a warm meal, and cares more about the missing villagers than the ruined bell. Orin Hale, guard captain, initially insists nobody climb the tower after dark but can be persuaded by practical preparations. Jessa, an older witness at the well, fears being called a liar. Let every player ask one useful question.

### Mira Fen
- **Says:** “Find them alive. Money I can spare, but not another family.”
- **Knows:** the bell quieted at midday, then three villagers walked toward the watch road.
- **Wants:** safe rescue; she offers the key to the village storehouse and lodging.
- **If pressed:** she knows Elowen understands the old ward signs.

### Orin Hale
- **Says:** “The old stair is rotten. You won't help anyone by falling.”
- **Knows:** the bell's internal bronze weight was removed, not broken.
- **Wants:** competent investigation; provides lanterns and the tower key if respected.
- **If alienated:** trails the party at a distance, but does not stop them.

### Jessa at the well
- **Says:** “I heard the singing while they slept. The youngest carried something heavy.”
- **Knows:** a sleepwalker carried a bronze weight toward the signal shed.
- **Wants:** reassurance she won't be blamed; honest kindness succeeds automatically.
- **If ignored:** footprints point to the same shed.

### Elowen Thatch
- **Says:** “The seal holds a name too old to speak. Repair the bell before you break the ring.”
- **Knows:** the seal is protective, the three marks are seed, flame, crown.
- **Wants:** the sleepers back and the ward intact.
- **If invited:** offers to accompany to tower entrance but will not fight.

## Player-led investigation
**Bell frame:** DC 10 Investigation or simply thorough inspection reveals an empty mount and scratched bronze dust. **Old road:** DC 10 Survival tracks three bare-footed sleepers; failure costs 10 minutes before finding the obvious wagon ruts. **Well:** DC 10 Persuasion or ordinary sincere assurance earns Jessa's story. **Shrine:** faded imagery shows seed, fire, crown in that order. **Mill:** a worker heard Kael discussing a hidden archive, but no villain is proven.

## Essential clue redundancy
Regardless of rolls, give at least two of these: missing weight, signal shed trail, bell fragment, tower crown symbol. If players skip the square, Orin encounters them at the road and explains the problem.

## Escalation without a dead end
Delay: distant chimes grow louder and one abandoned shoe is found on the road. Social mistake: villagers become guarded but still give directions. Failed skill check: spend more time or face a harmless gust of echoing voices—not a random deadly battle.

## Transition
Ask: “Do you take the road, follow the stream, or try the ridge?” Open Scene Two and the tower map.
"###,
    },
    DemoEntry {
        path: "Adventures/Scenes/Scene Two - GM Dossier.md",
        template: "adventure-scene",
        title: "Scene Two - GM Dossier",
        visibility: "gm-only",
        body: r###"# Scene Two: The Briarwatch Ascent
**Goal:** recover the missing bronze weight and reach the vault; exploration is the default, combat optional.

## Read aloud
> The watchtower emerges above the thorn trees like a broken finger. Bronze wind chimes tap against empty stone windows. Beyond the entrance, a wicker figure stands as still as the roots twisted around its feet.

## Three approaches
**Old watch road:** easiest; the party sees footprints heading toward a small signal shed. **Stream drainage:** an ankle-deep passage leads behind the shed; careful movement avoids the watch landing. **Ridge path:** steep but accessible, reveals a side window and broken carvings. No route requires a successful roll to continue.

## Keyed location 1 — Tower road
Wet prints and a torn blue ribbon catch the light. DC 10 Survival shortens the search; failure costs 10 minutes. A player can recover the ribbon as evidence of a villager's passage.

## Keyed location 2 — Signal shed
A warped timber door opens onto old signal equipment and **the missing bronze tuning weight**. A small locked cupboard (DC 10 thieves' tools or loose hinge/prying) holds **one potion of healing**. The weight is heavy but portable. If the party skips the shed, the tower chimes guide them back to it.

## Keyed location 3 — Watch landing
The Thornbound Sentinel stands beside a fallen ward symbol. It **does not attack on sight**. Characters who replace the symbol, speak “Hold the line,” or move without damaging runes pass peacefully. If a character deliberately destroys the ward after a warning, the sentinel attacks using its custom 5e document. It disengages if the party retreats or protects the markings.

## Keyed location 4 — Upper bell landing
Hollow chimes ring when wind passes through them. DC 11 Perception identifies a repeating low tone near the hidden stair. Without a successful check, a chime knocks against a wall and reveals its seam. Smashing all chimes manifests an optional Hollow Echo; if the sentinel was already fought, use eerie voices instead of a second fight.

## Keyed location 5 — Spiral stair
A broad stone stair descends to the vault. The crown marks are visible in the order **seed → flame → crown** on a faded relief. Let players copy or sketch them. Avoid a fall trap; beginner groups should focus on discovery.

## Play example
A rogue notices a broken rune; a cleric recognizes it as protective (Religion DC 11); the fighter resets the stone; the wizard hears the low chime. Each contribution is meaningful, and none requires a perfect skill distribution.

## Exits and retreat
Players can exit toward the village at any time, rest as the GM permits, or ask Elowen to help. There is no hard timer that makes the adventure unwinnable.

## Transition
Use the Crown Vault schematic and Scene Three. Have the missing tuning weight and puzzle clue available.
"###,
    },
    DemoEntry {
        path: "Adventures/Scenes/Scene Three - GM Dossier.md",
        template: "adventure-scene",
        title: "Scene Three - GM Dossier",
        visibility: "gm-only",
        body: r###"# Scene Three: The Crown Vault
**Goal:** free three sleepwalking villagers and resolve the damaged ward without forcing a boss fight.

## Read aloud
> Under the tower, pale light hangs in a ring carved into the floor. Three villagers stand motionless beyond the ring, breathing softly. Within it floats a cracked bronze crown. A voice rises from the stones: “I have kept their songs. What will you give to hear them again?”

## Room features
**North:** a dry stair and entry arch. **Center:** suspended Crown Seal over the etched ring. **East:** a narrow, sealed archive door; Kael longs to open it. **West:** a cracked alcove with three rotating mark-discs. **South:** three sleepers at the outer wall. The party can reach the sleepers; they become confused and return to trance if guided out before the ward is resolved. They are never targeted by combat.

## Faction choices
**Elowen (Ash Wardens):** “The bell protects the valley. Restore its rhythm.” She knows the mark order but initially fears speaking it; a gentle request earns it. **Kael (Gloam Seekers):** “The history behind that door belongs to everyone.” He is not a kidnapper and may compromise. **Hollow King:** speaks only from within the ring; offers secrets and a bargain but cannot reach out or force a deal.

## The Three Crown Marks
An inscription: “Life before fire; duty beyond both.” Three discs: **seed, flame, crown**. Correct order stabilizes the ring. DC 11 Arcana can assist; DC 11 Religion recognizes a protective rite; direct experimentation and clue gathering are equally valid. Every failed attempt creates sound and urgency but no instant character death. Creative mechanical repair is acceptable.

## Restoring the seal
Place the bronze weight into the bell mechanism at the edge of the chamber and align seed–flame–crown. A character can do either step without a roll if they have discovered how. Elowen joins hands with the group to hold the rhythm. The sleepers awaken safely. Read *Ending A - The Bell Rings Again*.

## Breaking the seal
Deliberately destroy the ring (tools or magic as appropriate) after describing that it binds something. A wave of cold wind passes through the party; the sleepers awaken. No unavoidable damage. Read *Ending B - The Open Vault*. The archive door can open, but the region loses the ward.

## Bargaining
The voice offers the sleepers' freedom in exchange for one memory, promise, service or confession **voluntarily chosen by a player**, never a forced removal of character identity or agency. Players may counteroffer, or reject the bargain. Read *Ending C - The Price of a Name* when agreement is made.

## Fourth solution
If players create a coherent plan to repair the ward while letting scholars document the archive, combine restorative and archival outcomes. Reward creative compromise. Do not require the written three ending labels.

## If they attack the ring or voice
The ring is an object, not a combatant. The voice cannot be slain with ordinary weapon attacks. Explain visible cracks, allow time to change course, and resolve their choice narratively. Avoid a lethal surprise fight.

## Resolution checklist
- All three sleepers alive and awake.
- Party understands the effect of its decision.
- 20 gp total from Mira if villagers return, nonmagical Crown Shard if appropriate.
- Update Session Log, quest status and campaign-state record after actual play only.
"###,
    },
    DemoEntry {
        path: "Game Master/GM Notes/NPC Dialogue and Motivation Guide.md",
        template: "gm-notes",
        title: "NPC Dialogue and Motivation Guide",
        visibility: "gm-only",
        body: r###"# NPC Guide for Beginning Game Masters
**Mira Fen:** direct, calm but frightened. Opens: “I need help, not heroes with grand speeches.” Secret: she suspects the village bell was never a normal bell. Wants family members safe. Changes her mind when the party produces a concrete plan.

**Orin Hale:** skeptical protector. Opens: “Take the lanterns. Leave the bragging.” Secret: he saw a sleepwalker entering the old road after midnight. Wants to prevent a panic. Can lend the key, rope and two lanterns.

**Jessa:** older witness, speaks softly. Opens: “They were walking with their eyes closed.” Secret: she saw the bronze weight carried toward the signal shed. Wants safety and respect; cannot be coerced into revealing anything more than she knows.

**Elowen Thatch:** quiet Ash Warden keeper. Opens: “The marks are not ornaments. They are a promise.” Secret: the king's voice cannot leave the ring. Wants the ward restored, but may accept supervised archive access.

**Kael Voss:** polite, impatient researcher. Opens: “Please, don't mistake curiosity for cruelty.” Secret: the Gloam Seekers hired him to map the royal archive. Wants written history freed from concealment. He does not intend harm to the sleepers; he will bargain honestly if convinced it saves them.

**The Hollow King:** many overlapping whispers, never a physical attacking body. Opens: “A name is a door. Give me yours and I will show you mine.” It cannot leave the circle while intact. It can offer a true historical clue and a misleading claim in the same speech.

## Three quick improv rules
1. Let a player question change an NPC's response when it is plausible.
2. If a conversation stalls, reveal a tangible clue instead of demanding another roll.
3. Keep NPCs as witnesses or negotiators, never the main characters making all choices.
"###,
    },
    DemoEntry {
        path: "Game Master/GM Notes/GM Decision Tree and Failure Recovery.md",
        template: "gm-notes",
        title: "GM Decision Tree and Failure Recovery",
        visibility: "gm-only",
        body: r###"# GM Decision Tree
**Players ask who is missing:** three sleepwalkers; villagers saw them take the tower road.
**Players ask where to go:** Jessa → signal shed, Orin → watch road, Elowen → old ward carvings.
**Players refuse Mira's quest:** have a sleeper pass nearby or a relative ask for a safe escort. If they still refuse, respect their choice and describe the ongoing consequence; offer a different hook to the same mystery.
**Players blame Kael:** his notes prove interest in the archive; he is not responsible for sleepwalking.
**Players attack a friendly NPC:** pause and establish table expectations; NPCs seek cover. No punitive boss battle.
**Players bypass the tower:** stream passage leads to the vault stair with an obvious crown mark.
**Players never find the weight:** a chime guide, footprint trail and Elowen independently point to the signal shed.
**Players fail the disc puzzle:** give stronger mural clues, allow alternative repair, or offer the bargain.
**Players rest or leave:** villagers remain asleep, not dead; the mystery stays available.
**Players destroy seal before understanding it:** let consequences become apparent and offer a final chance to revise before the irreversible commitment.
**Players want to speak to the sleepers:** they breathe, mumble and can be moved without harm; true consciousness returns after a resolution.
**Players attempt unexpected magic:** resolve within normal 5e rules and reward intention without requiring a specific spell.

## GM Clue Funnel
Bell frame → missing weight; Jessa → signal shed; ridge relief → seed/flame/crown; sentinel → protects runes; Kael → archive; Hollow King → ring boundary. No single clue is necessary.

## Level-One Safety
Enemies do not pursue retreating PCs beyond their assigned ward. If party resources are depleted, replace combat with negotiations. A failed check never equals unavoidable instant defeat.

## Adventure pacing
Village can compress to 20 minutes; tower to 40; vault to 30. For a longer evening, allow exploration of old records and one extra harmless echo event. Do not pad with deadly encounters.
"###,
    },
    DemoEntry {
        path: "Game Master/GM Notes/Session Zero and Beginner Rules Guide.md",
        template: "session-zero",
        title: "Session Zero and Beginner Rules Guide",
        visibility: "gm-only",
        body: r###"# Session Zero for a New D&D Group (10–15 minutes)
Ask players whether memory loss, sleepwalking, peril to innocents or supernatural voices should be excluded or faded out. Agree how anyone can pause or change a scene. Choose a shared tone (gentle mystery, adventurous, eerie). Establish that player choices shape the ending.

## D&D 5e basics
Describe an action → GM decides if a roll is needed → roll a d20 + the relevant ability modifier + proficiency bonus if applicable → equal or exceed the DC. At **level 1 proficiency bonus is +2**. Advantage: roll two d20s and take the higher; disadvantage: take the lower. In combat use initiative (d20 + Dexterity modifier), actions, movement and optionally bonus actions/reactions from character rules. A character at zero hit points is unconscious and uses death saving throw rules from their chosen fifth-edition source; don't improvise punitive instant deaths.

## Dice practice
Every player may make a harmless practice roll before the game starts. Demonstrate DC 10 Investigation on the silent bell as an example. Skill checks are not required when the described approach is clearly sufficient.

## Table expectations
Let players ask rules questions without shame. Keep descriptions brief; ask each character what they do. Players own their characters and may decline a proposed memory bargain.
"###,
    },
    DemoEntry {
        path: "Game Master/GM Notes/Adventure Completion Checklist.md",
        template: "gm-notes",
        title: "Adventure Completion Checklist",
        visibility: "gm-only",
        body: r###"# Before players arrive
- [ ] Read Complete Adventure and three GM scene dossiers
- [ ] Open village, tower and vault schematic maps
- [ ] Print or share premade level-one characters if needed
- [ ] Open NPC guide, clue/GM decision tree, and optional encounters
- [ ] Review ending cards and rewards
- [ ] Ask about content boundaries

# During play
- [ ] Present village mystery and bell fragment
- [ ] Give more than one route to the weight
- [ ] Allow peaceful bypass of sentinel
- [ ] Offer at least three endings
- [ ] Avoid back-to-back fights at level one
- [ ] Record which NPCs the players trust

# After play
- [ ] Confirm sleepers saved and outcome
- [ ] Pay 20 gp total and other rewards if earned
- [ ] Write Session Log and update Quest Chain/Campaign State
- [ ] Preserve player edits; demo opening state is not overwritten
- [ ] Decide whether to continue with the lost archive as a second adventure
"###,
    },
    DemoEntry {
        path: "Adventures/Main Campaign/Ending A - The Bell Rings Again.md",
        template: "quest-adventure",
        title: "Ending A - The Bell Rings Again",
        visibility: "gm-only",
        body: r###"# Restoration ending — read aloud
> The bronze weight catches in its cradle. A deep note shudders through the stone, then rises into the night. The crown seals together in amber light. One by one, the sleepers open their eyes.

## Immediate result
The villagers are safe. The ward again holds the Hollow King's voice inside the circle. Mira pays **20 gp total** to the party; Elowen offers them the Crown Shard as a reminder that protection must be maintained.

## NPC outcomes
Elowen respects the party and asks one PC to learn the watchword. Kael is frustrated but may accept supervised access to non-sensitive records. Orin celebrates quietly; Jessa is vindicated.

## If continuing
A new chime from an older tower signals that other seals may need repair. No sequel is required for a satisfying ending.
"###,
    },
    DemoEntry {
        path: "Adventures/Main Campaign/Ending B - The Open Vault.md",
        template: "quest-adventure",
        title: "Ending B - The Open Vault",
        visibility: "gm-only",
        body: r###"# Destruction ending — read aloud
> The ring splits with a sound like breaking ice. Air rushes outward. The three sleepers gasp and stumble into waiting arms. Beyond the broken crown, a stone door opens onto shelves long hidden from the world.

## Immediate result
The sleepers are free and Mira pays **20 gp total**. The ward is gone; echoes may now be heard in the distant hills. The party has access to the historical archive but not a guarantee of what it contains.

## NPC outcomes
Kael is jubilant yet shaken by the costs. Elowen warns that the village needs a replacement protection. Orin asks the party to help plan watch patrols.

## If continuing
The first page of the archive mentions a second crown sealed across the valley. The adventure still has a complete resolution: all villagers survive.
"###,
    },
    DemoEntry {
        path: "Adventures/Main Campaign/Ending C - The Price of a Name.md",
        template: "quest-adventure",
        title: "Ending C - The Price of a Name",
        visibility: "gm-only",
        body: r###"# Bargain ending — read aloud
> The crown dims. A last note passes between the walls, gentle as a breath. The sleepers wake. In the ring, a voice whispers a single promise: “A debt remembered is a road you will one day travel.”

## Bargain boundaries
The **player** chooses what their character offers: a future task, a willingly volunteered secret, a nonessential keepsake, or a promise. The GM and player agree on the terms before the pact. Never unilaterally erase a player's background or remove character control.

## Immediate result
All sleepers are saved; Mira pays **20 gp total**. The ring remains weakened but stable for now. The Hollow King cannot leave it yet. The debt becomes an optional future quest hook.

## If continuing
The voice requests a harmless retrieval mission in the lost archive. The adventure has ended even if the next quest is declined.
"###,
    },
    DemoEntry {
        path: "Worldbuilding/Settlements & Locations/Briar Glen - Keyed Village Guide.md",
        template: "location-profile",
        title: "Briar Glen - Keyed Village Guide",
        visibility: "gm-only",
        body: r###"# Briar Glen — Village Keys
**V1 Bell Square:** empty frame, bronze scrape, Mira waiting.
**V2 Well:** Jessa knows the tuning weight went to the signal shed.
**V3 Mill:** worker heard Kael discussing a royal archive.
**V4 Old Shrine:** mural shows seed → flame → crown.
**V5 North Road Gate:** Orin lends the tower key and two lanterns.
**V6 Guest Cottage:** safe place to discuss, take breaks or rest if the GM allows.

## Route map
V1 connects to V2 and V5. V2 connects to V3. V3 leads to V4. V5 leads to the old tower road. This is a location guide, not a dimensioned tactical map.

## Village atmosphere
Small community, low threat, damp fall air, lanterns and the smell of wet timber. NPCs prioritize rescue. The bell is silent; there is no countdown to inevitable death.
"###,
    },
    DemoEntry {
        path: "Maps/Settlements/Briar Glen - Village Schematic.md",
        template: "map-reference",
        title: "Briar Glen - Village Schematic",
        visibility: "gm-only",
        body: r###"# Briar Glen — keyed GM schematic (not to scale)
```text
                        NORTH / TOWER ROAD
                              [V5 Gate]
                                  |
 [V3 Mill] -- [V2 Well] -- [V1 Bell Square] -- [V6 Guest Cottage]
      |
 [V4 Old Shrine]      STREAM TO THE SOUTH
```
**V1** bell frame and Mira; **V2** Jessa witness; **V3** Kael rumor; **V4** mark order; **V5** Orin's supplies and key; **V6** respite.

Use as a verbal navigation aid, not a VTT-accurate battle grid.
"###,
    },
    DemoEntry {
        path: "Maps/Dungeons/Briarwatch Tower - Keyed Map.md",
        template: "map-reference",
        title: "Briarwatch Tower - Keyed Map",
        visibility: "gm-only",
        body: r###"# Briarwatch Tower — keyed schematic
```text
             [T4 Upper Bell Landing]
                       |
[T1 Old Road] -- [T3 Watch Landing] -- [T2 Signal Shed]
                       |
                 [T5 Spiral Stair]
                       |
                  [C1 Crown Vault]
```
**T1:** visible tracks. **T2:** missing bronze tuning weight, healing potion. **T3:** peaceful sentinel. **T4:** resonating chimes, stair clue. **T5:** seed/flame/crown relief. **C1:** branching resolution.

Each route can be traversed without locked progression or forced combat.
"###,
    },
    DemoEntry {
        path: "Maps/Dungeons/Crown Vault - Keyed Map.md",
        template: "map-reference",
        title: "Crown Vault - Keyed Map",
        visibility: "gm-only",
        body: r###"# Crown Vault — keyed schematic
```text
                 [C1 ENTRY / STAIR]
                        |
 [C4 MARK DISCS] -- [C2 CROWN RING] -- [C3 ARCHIVE DOOR]
                        |
                [C5 THREE SLEEPERS]
```
**C1:** entrance. **C2:** seal ring; the voice is confined inside. **C3:** sealed records beyond. **C4:** seed/flame/crown discs, DC 11 Arcana optional. **C5:** living and unharmed sleepers.

**Scale:** symbolic rather than gridded; physical combat is not required. Mark the crown ring as a boundary the Hollow King cannot cross.
"###,
    },
];

/// Editable player-reference cards for the beginner one-shot. These do not
/// claim to be live Character Studio character sheets.
const PREMADE_ENTRIES: &[DemoEntry] = &[
    DemoEntry {
        path: "Characters/Player Characters/Rowan Ash - Level 1 Premade.md",
        template: "player-character-placeholder",
        title: "Rowan Ash - Level 1 Premade",
        visibility: "gm-only",
        body: r###"# Rowan Ash — Level 1 Premade D&D 5e Character
**Human Fighter • Level 1 • Background: custom narrative adventurer • Proficiency +2**

## Fast stats
Armor Class **16** (chain mail), Hit Points **12**, Speed **30 ft.**, Initiative **+1**, Proficiency **+2**.

## Ability scores
STR 16 (+3), DEX 12 (+1), CON 14 (+2), INT 8 (−1), WIS 10 (+0), CHA 13 (+1)

## Saving throws
Strength +5, Constitution +4

## Skills
Athletics +5, Perception +2, Survival +2, Persuasion +3

## Attacks and spells
Longsword: **+5 to hit**, 1d8+3 slashing (one-handed); light crossbow: **+3**, 1d8+1 piercing, ammunition/range per your 5e source.

## Equipment
Chain mail, shield, longsword, light crossbow and 20 bolts, backpack, rope, torch and rations. Shield is included in AC 18 **if equipped**; listed AC 16 assumes no shield equipped.

## Features and resources
**Fighting Style—Defense:** +1 AC while wearing armor (thus AC 17 with chain mail, AC 19 with shield). **Second Wind:** once per short or long rest, bonus action regain 1d10+1 HP. These are brief prompts; refer to your 5e rules for exact details.

## Personality and adventure hook
You learned that your family once maintained the road to Briarwatch. You want to protect the villagers without destroying the old tower.

## Your first scene
Ask Orin for an old key. You're the party member most likely to recognize the sentinel's stance as defensive.

## How to play
Describe what you want to do; when the GM calls for an ability check, roll d20 plus the listed relevant modifier. Your hit points are a small resource at level one—retreat or negotiate when sensible. Track ammunition, spells and other resources manually or on a compatible 5e sheet. These are **editable beginner reference cards**, not fully automated Character Studio sheets. Consult your own D&D 5e (2014-style) source for exact class features, spell rules, actions, conditions and races.
"###,
    },
    DemoEntry {
        path: "Characters/Player Characters/Nyra Vale - Level 1 Premade.md",
        template: "player-character-placeholder",
        title: "Nyra Vale - Level 1 Premade",
        visibility: "gm-only",
        body: r###"# Nyra Vale — Level 1 Premade D&D 5e Character
**Halfling Rogue • Level 1 • Background: custom narrative adventurer • Proficiency +2**

## Fast stats
Armor Class **14** (leather armor), Hit Points **9**, Speed **25 ft.**, Initiative **+3**, Proficiency **+2**.

## Ability scores
STR 8 (−1), DEX 16 (+3), CON 12 (+1), INT 13 (+1), WIS 14 (+2), CHA 12 (+1)

## Saving throws
Dexterity +5, Intelligence +3

## Skills
Acrobatics +5, Sleight of Hand +5, Stealth +5, Investigation +3, Perception +4, Insight +4

## Attacks and spells
Shortbow: **+5 to hit**, 1d6+3 piercing; dagger: **+5 to hit**, 1d4+3 piercing. **Sneak Attack:** +1d6 damage once per turn when the standard 5e conditions apply.

## Equipment
Leather armor, shortbow and 20 arrows, two daggers, thieves' tools, rope, hooded lantern, rations.

## Features and resources
**Expertise:** Stealth and Perception (already reflected above). **Thieves' Cant** and halfling luck/bravery are narrative prompts; use your rules source for exact mechanics.

## Personality and adventure hook
You have heard rumors of an archive under the tower. Your curiosity is stronger than your fear.

## Your first scene
Examine bell scrapes (Investigation), reassure Jessa, and open the signal shed cupboard using thieves' tools.

## How to play
Describe what you want to do; when the GM calls for an ability check, roll d20 plus the listed relevant modifier. Your hit points are a small resource at level one—retreat or negotiate when sensible. Track ammunition, spells and other resources manually or on a compatible 5e sheet. These are **editable beginner reference cards**, not fully automated Character Studio sheets. Consult your own D&D 5e (2014-style) source for exact class features, spell rules, actions, conditions and races.
"###,
    },
    DemoEntry {
        path: "Characters/Player Characters/Sera Dawn - Level 1 Premade.md",
        template: "player-character-placeholder",
        title: "Sera Dawn - Level 1 Premade",
        visibility: "gm-only",
        body: r###"# Sera Dawn — Level 1 Premade D&D 5e Character
**Dwarf Cleric • Level 1 • Background: custom narrative adventurer • Proficiency +2**

## Fast stats
Armor Class **16** (scale mail and shield), Hit Points **10**, Speed **25 ft.**, Initiative **+0**, Proficiency **+2**.

## Ability scores
STR 14 (+2), DEX 10 (+0), CON 14 (+2), INT 10 (+0), WIS 16 (+3), CHA 10 (+0)

## Saving throws
Wisdom +5, Charisma +2

## Skills
Insight +5, Medicine +5, Religion +2, History +2

## Attacks and spells
Mace: **+4 to hit**, 1d6+2 bludgeoning. Spell attack **+5**, spell save **DC 13**.

## Equipment
Scale mail, shield (included in AC 16), mace, holy symbol, healer's supplies, explorer's kit.

## Features and resources
**Channel features:** at level 1 use your selected domain's rules; no Channel Divinity until later levels in traditional 2014-style cleric. **Spell slots:** two 1st-level slots. Prepare four cleric spells (Wisdom modifier 3 + level 1) from your available rules, plus any appropriate domain spells. Suggested: *cure wounds*, *bless*, *guiding bolt*, *detect magic*. Suggested cantrips: *sacred flame*, *guidance*, *spare the dying*. Spell effects require a licensed/open reference; no proprietary spell text is bundled.

## Personality and adventure hook
You are drawn to forgotten ways of keeping people safe. The seed, flame and crown signs feel like a broken promise.

## Your first scene
Speak with Elowen, recognize a protective ward (Religion DC 11), and use healing cautiously at level one.

## How to play
Describe what you want to do; when the GM calls for an ability check, roll d20 plus the listed relevant modifier. Your hit points are a small resource at level one—retreat or negotiate when sensible. Track ammunition, spells and other resources manually or on a compatible 5e sheet. These are **editable beginner reference cards**, not fully automated Character Studio sheets. Consult your own D&D 5e (2014-style) source for exact class features, spell rules, actions, conditions and races.
"###,
    },
    DemoEntry {
        path: "Characters/Player Characters/Calder Reed - Level 1 Premade.md",
        template: "player-character-placeholder",
        title: "Calder Reed - Level 1 Premade",
        visibility: "gm-only",
        body: r###"# Calder Reed — Level 1 Premade D&D 5e Character
**Human Wizard • Level 1 • Background: custom narrative adventurer • Proficiency +2**

## Fast stats
Armor Class **12** (no armor), Hit Points **7**, Speed **30 ft.**, Initiative **+2**, Proficiency **+2**.

## Ability scores
STR 8 (−1), DEX 14 (+2), CON 12 (+1), INT 16 (+3), WIS 13 (+1), CHA 10 (+0)

## Saving throws
Intelligence +5, Wisdom +3

## Skills
Arcana +5, Investigation +5, History +5, Insight +3

## Attacks and spells
Quarterstaff: **+1 to hit**, 1d6−1 bludgeoning. Spell attack **+5**, spell save **DC 13**.

## Equipment
Quarterstaff, spellbook, component pouch, scholar's pack, lantern and rations.

## Features and resources
**Spell slots:** two 1st-level slots. **Arcane Recovery:** once per day after a short rest recover one 1st-level slot under traditional 2014-style rules. Suggested cantrips: *mage hand*, *light*, *ray of frost*. Spellbook suggestions (6): *mage armor*, *magic missile*, *shield*, *detect magic*, *sleep*, *comprehend languages*. Prepare four of those; use your rules source for full effects.

## Personality and adventure hook
You are an apprentice scholar who distrusts secret archives and wants a chance to document them responsibly.

## Your first scene
Talk with Kael, interpret the crown markings (Arcana DC 11), and offer a creative repair of the bell.

## How to play
Describe what you want to do; when the GM calls for an ability check, roll d20 plus the listed relevant modifier. Your hit points are a small resource at level one—retreat or negotiate when sensible. Track ammunition, spells and other resources manually or on a compatible 5e sheet. These are **editable beginner reference cards**, not fully automated Character Studio sheets. Consult your own D&D 5e (2014-style) source for exact class features, spell rules, actions, conditions and races.
"###,
    },
    DemoEntry {
        path: "Game Master/Handouts/Player Quick Start - The Hollow Crown.md",
        template: "handout",
        title: "Player Quick Start - The Hollow Crown",
        visibility: "gm-only",
        body: r###"# Welcome to the adventure
You are a new adventuring party arriving in **Briar Glen**, a small village with a silent bell. Three villagers have been seen wandering into the mist. You may explore, talk, fight, investigate or change your mind. The story reacts to your choices.

## Starting character choices
- Rowan Ash: hardy fighter, protects others.
- Nyra Vale: quick rogue, notices clues.
- Sera Dawn: cleric, healer and ward-lore guide.
- Calder Reed: wizard, investigator of magic.

## Your turn
Say what you try. The GM may ask for a **d20 + ability modifier + proficiency bonus** check. You can suggest clever solutions instead of attacking. No single die roll should stop the adventure.

## Character choices matter
Try to save the villagers, but decide what happens to the ancient Crown Seal yourselves. Every ending is valid. Ask for breaks or changes to uncomfortable scenes.

*This is an independent original D&D 5e-compatible demo, not an official Wizards of the Coast adventure.*"###,
    },
    DemoEntry {
        path: "Game Master/Handouts/Player Rumor Cards.md",
        template: "handout",
        title: "Player Rumor Cards",
        visibility: "gm-only",
        body: r###"# Player Rumor Cards — cut into separate handouts
1. **Village Bell:** “It always rings at dusk. Today it didn't.”
2. **The Old Road:** “Footprints appear in the wet earth, but the people who made them don't remember walking.”
3. **The Tower:** “The stone crown over its door is cracked.”
4. **The Scholar:** “An outsider asked about the lost archive beneath the tower.”
5. **The Shrine:** “Seed, flame, crown — some say they were the symbols of an old oath.”

Give one or two cards when a player asks around. No card is required to advance; all clues have alternative routes. These cards do not reveal the hidden truth outright."###,
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
            "subtitle": "An Original D&D 5e Adventure for Four Level-1 Characters",
            "premise": "The bell of Briarwatch Tower has fallen silent, and villagers are walking in their sleep toward an ancient crown-shaped seal.",
            "genre": "Fantasy Mystery",
            "tone": "Mysterious, heroic, choice-driven",
            "gm_name": "Demo Game Master"
        },
        "rules": { "pack_id": "custom-homebrew", "custom_system_name": "Dungeons & Dragons 5e (2014-style)", "edition": "5e (2014)", "progression_type": "level", "starting_level": "1", "advancement": "Milestone" },
        "world": {
            "mode": "new", "name": "The Briar Marches",
            "era": "Frontier Age", "origin": "original",
            "notes": "A misty borderland where forgotten royal wards protect small villages from echoes."
        },
        "party": { "name": "The Bellward Company", "expected_players": "4" },
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
            "play_mode":"physical", "dice":"digital",
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
        seed_entry(&staging, &DemoEntry {
            path: "Maps/Dungeons/Briarwatch Tower - Text Map.md",
            template: "map-reference",
            title: "Briarwatch Tower - Text Map",
            body: LOCATION_MAP,
            visibility: "gm-only",
        })?;
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
        genre: Some("D&D 5e Fantasy Mystery".into()),
        description: Some("An original D&D 5e fantasy adventure for four level-1 characters, with NPCs, encounters, quests, clues, monsters and GM notes.".into()),
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
