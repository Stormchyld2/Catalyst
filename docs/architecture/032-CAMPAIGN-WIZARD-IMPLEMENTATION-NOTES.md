# 032 — Campaign Wizard Implementation Notes

**Status:** Implemented (first milestone)
**Implements:** `020-CAMPAIGN-WIZARD-ARCHITECTURE.md`, which is the design authority. This note records what is built and where the build deviates from 020. It is numbered 032 because 020–031 are already taken.

## Built

- **Entry:** Home "New Campaign", File > New Project, and the Story Development rail button (with no project open) open `src/lib/components/CampaignWizardDialog.svelte`, which offers Quick Start or Guided Setup.
- **Quick Start:** name, destination and rules system (System-Neutral by default, or Custom/Homebrew with a system name), plus an optional premise. It seeds only the Campaign Overview.
- **Blank Campaign:** name and destination, optional rules system (System-Neutral by default). It creates the full tree, both manifests and the Campaign Overview anchor, with no GM library and no other content.
- **Guided Campaign Builder:** the ten steps in 020 §3 order, written as GM-facing questions. Step 1 asks for identity (genre and tone pick lists with custom text, premise, GM, a date picker, destination). Step 2 asks for rules (installed packs only, edition, progression terminology, advancement, house-rule mode and notes, and character-creation restrictions saved as `rules.character_creation` in campaign.yaml for future Character Studio validation). Step 3 asks for world type (Existing and Import are shown disabled, with the reason), name, origin (published settings are recorded by name only), scale, era and distinctive features. Step 4 lets the GM pick worldbuilding areas and shows each area's own questions and named entries (for example, a settlement's purpose and location). Answers become an "<Area> Overview" page, and each entry becomes its own templated page. Every step after Step 1 can be skipped, and answers survive moving between steps. The questions for Economy & Trade onward in Step 4 were drafted by Claude, because the spec message was cut off there. Steps 5 to 10 were also written by Claude in the same style: Step 5 asks for expected players, party start and players (a character concept appears once a character is named). Step 6 covers NPCs (disposition, role, want), creatures (type and threat level) and factions (goal and attitude). Step 7 asks for structure and length, which hide arcs and side quests for one-shots and episodic play, plus the conflict, first adventure and hook. Step 8 covers the starting location, location types and map files. Step 9 covers play mode, dice, a player display (hidden for online play), schedule, session length and Session Zero, which adds an agenda to the first session plan. Step 10 is a review whose cards jump back to each step. New answers are stored additively in campaign.yaml (party.expected_players/origin, play.schedule/session_length) and in record metadata (campaign_structure, campaign_length, threat, attitude).
- **Backend:** `src-tauri/src/campaign_wizard.rs` with the `create_new_campaign` command:
  - writes `project.yaml` (`project_type: campaign`, `catalyst-project` v1 marker)
  - writes `campaign.yaml` (schema v1, same `project_id`, rules pack and version, world mode, `Sessions/Campaign State/`, play preferences, party, `starting_session_id`). `read_campaign_manifest` validates that both files agree.
  - creates the full 020 §4 tree.
  - stages creation in a sibling `.<name>.catalyst-staging-<uuid>` folder, verifies folders, manifests, the wiki index and unique IDs, then publishes with a rename. A non-empty destination is refused, and on failure only the staging folder is removed.
- **Templates:** campaign entity templates are registered in the built-in registry (`entity_templates.rs`, `campaign_templates()`): Campaign Overview, NPC Profile, Player Character (Placeholder), Creature Profile, Quest / Adventure, Encounter, Session Plan, Session Log, House Rules, GM Notes and Map Reference. Each has an image field, overview fields and a Markdown scaffold, like the inherited templates. Every seeded record is created the way the wiki's New Document command creates one: `template_id`, subtype, every template field, the scaffold below any prose the GM typed, and `visibility: gm-only`. Seeded records therefore render in the wiki's Overview panel, and the same templates are offered in the wiki's New Document menu under Campaign, Characters, Creatures and Adventures.
- **GM Reference Library (020 §9):** every new campaign gets 7 original guides (starting a campaign and Session Zero, planning adventures, worldbuilding, NPCs, encounters, improvisation, dice and checklists) and 10 original random tables (NPC names, motivations, settlements, taverns, rumors, hooks, wilderness discoveries, weather, complications, treasure). They use the new `gm-guide` and `random-table` templates and record `source` and `license`.
- **Wiki adapter (020 §4):** the Story Development index, folder picker, New Document, New Folder and Move read `CAMPAIGN_ROOTS` alongside the book roots. Book projects are unchanged.

## Deviations from 020 (with reasons)

| 020 says | Built | Why |
|---|---|---|
| Open Campaign Dashboard | Opens Story Development (wiki) | No Campaign Dashboard exists yet |
| Installed verified rules packs | Only System-Neutral and Custom/Homebrew | No verified packs are installed |
| Linked shared or imported world | Rejected. Only New or Decide later. | Shared-world overrides are not built. Rejecting avoids a half-working option. |
| Campaign artwork in Basics | Set later via the Campaign Overview image field | Avoids copying user files during the wizard |
| Optional resume drafts | Not implemented | Optional in 020 |
| Exact §4 tree | Adds `Sessions/Roll Logs/` | The global dice roller (PR #2) stores roll history there |
| §9 GM Reference Library installed once and shared by all campaigns | Copied into each new campaign under `Game Master/Guides` and `Game Master/Random Tables` | 020 allows a per-campaign copy for customization. A shared install location is not built yet. All content is original (`src-tauri/src/gm_library.rs`), so it passes the licensing gate. |
| Hierarchical place parents, privacy enforcement | Only `visibility` metadata is stored | Later World Builder and GM Console work |

## Not started (per 020 §7)

Character Builder and Sheet Designer. Player characters are placeholder records (`status: placeholder`, `character_record: Pending Character Studio`), and `Characters/Player Characters/README.md` says so.

## Catalyst identifiers (LithoMark rename)

New files use Catalyst names. Files written by earlier builds still open, and Catalyst never rewrites them silently.

| What | Written now | Still read |
|---|---|---|
| Document identity key | `catalyst:` | `lithomark:` (kept on save for files that use it) |
| project.yaml format | `catalyst-project` | `lithomark-project` |
| App folder (trash) | `.catalyst/` | `.lithomark/trash` |
| Save temp / staging files | `.catalyst-save-`, `.catalyst-staging-` | `.lithomark-save-` (recovery) |
| User template key | `catalyst_template` | `lithomark_template` |
| Review formats and extensions | `catalyst-review`, `catalyst-feedback`, `catalyst-review-copy`, `catalyst-review-distribution-record` | the `lithomark-` forms, opened from the in-app picker only. The OS file associations are Catalyst-only, so Catalyst never claims LithoMark's files. |
| Plugin manifest | `minimumCatalystVersion` | `minimumLithoMarkVersion` |
| Projection source ids | `catalyst:file:` and similar | `lithomark:` prefixes |
| Window events | `catalyst:before-project-change` and the others | none |
| Local storage | `catalyst:theme`, `catalyst:accent`, `catalyst:showSampleProject`, `catalyst:unsaved-chapter:` | the `lithomark:` keys are migrated or read as a fallback |

`legacy_data.rs` (the one-time import from LithoMark's app data folder) was removed, because Catalyst must not import LithoMark data.
