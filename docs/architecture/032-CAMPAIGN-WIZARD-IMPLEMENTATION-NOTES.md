# 032 — Campaign Wizard Implementation Notes

**Status:** Implemented (first milestone)
**Implements:** `020-CAMPAIGN-WIZARD-ARCHITECTURE.md`, which is the design authority. This note records what is built and where the build deviates from 020. It is numbered 032 because 020–031 are already taken.

## Built

- **Entry:** Home "New Campaign", File > New Project, and the Story Development rail button (with no project open) open `src/lib/components/CampaignWizardDialog.svelte`, which offers Quick Start or Guided Setup.
- **Quick Start:** name, destination and rules system (System-Neutral by default, or Custom/Homebrew with a system name), plus an optional premise. It seeds only the Campaign Overview.
- **Guided Setup:** the ten steps in 020 §3 order. Every content field is optional. Validation only requires a name and destination, a system name when Custom/Homebrew is chosen, and names on rows the GM added. Values survive back and forth navigation. Review shows the planned files.
- **Backend:** `src-tauri/src/campaign_wizard.rs` with the `create_new_campaign` command:
  - writes `project.yaml` (`project_type: campaign`, unchanged `lithomark-project` v1 marker)
  - writes `campaign.yaml` (schema v1, same `project_id`, rules pack and version, world mode, `Sessions/Campaign State/`, play preferences, party, `starting_session_id`). `read_campaign_manifest` validates that both files agree.
  - creates the full 020 §4 tree.
  - stages creation in a sibling `.<name>.catalyst-staging-<uuid>` folder, verifies folders, manifests, the wiki index and unique IDs, then publishes with a rename. A non-empty destination is refused, and on failure only the staging folder is removed.
- **Templates:** campaign entity templates are registered in the built-in registry (`entity_templates.rs`, `campaign_templates()`): Campaign Overview, NPC Profile, Player Character (Placeholder), Creature Profile, Quest / Adventure, Encounter, Session Plan, Session Log, House Rules, GM Notes and Map Reference. Each has an image field, overview fields and a Markdown scaffold, like the inherited templates. Every seeded record is created the way the wiki's New Document command creates one: `template_id`, subtype, every template field, the scaffold below any prose the GM typed, and `visibility: gm-only`. Seeded records therefore render in the wiki's Overview panel, and the same templates are offered in the wiki's New Document menu under Campaign, Characters, Creatures and Adventures.
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
| §9 GM Reference Library (guides, random tables) | Not built. The GM worksheets are registered templates instead. | Stage 5 in 020 §12, with its own licensing gate |
| Hierarchical place parents, privacy enforcement | Only `visibility` metadata is stored | Later World Builder and GM Console work |

## Not started (per 020 §7)

Character Builder and Sheet Designer. Player characters are placeholder records (`status: placeholder`, `character_record: Pending Character Studio`), and `Characters/Player Characters/README.md` says so.
