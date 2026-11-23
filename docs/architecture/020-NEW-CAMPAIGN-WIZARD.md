# Catalyst New Campaign Wizard

**Status:** Implemented (first milestone)
**Builds on:** [016 New Book Foundation](016-NEW-BOOK-FOUNDATION.md). The same rules apply: setup state is temporary, Create Campaign is the persistence boundary, and the generated Markdown/YAML files with stable IDs are authoritative afterward. No database is involved.

## Entry points

Home "New Campaign", File > New Project, and the Story Development rail button (with no project open) open `CampaignWizardDialog.svelte`. The person picks one of two modes:

- **Quick Start**: name, game system, folder, optional pitch and a campaign starter. The backend fills in the defaults and installs every Game Master template.
- **Guided Setup** (10 steps): Campaign Basics, Players & Table, World Overview (at least one detail is required), Worldbuilding Categories, Locations, NPCs & Villains, Story & Hooks (plus the starter), First Session, Game Master Toolkit, Review & Create.

Both modes call `create_new_campaign` (`src-tauri/src/campaign_wizard.rs`), register the folder in the project registry, activate the project and open Story Development.

## Storage

`project.yaml` keeps the existing `lithomark-project` v1 format with `project_type: campaign`, so the reopen path is unchanged. A campaign root uses its own tabletop layout and none of the book folders (Manuscript, Audiobook, Reviews, Notes, Project Meta and Story Development are not created):

```text
<Campaign>/
├── project.yaml
├── Campaign/      Campaign Overview.md (metadata), Table & Safety.md, House Rules.md
├── World/         World Overview.md, Locations/, Factions & Governments/, Organizations & Guilds/,
│                  Cultures & Peoples/, Religions & Pantheons/, History & Lore/, Magic & Technology/,
│                  Bestiary & Creatures/, Items & Artifacts/, Languages/
├── Characters/    NPCs/, Villains & Antagonists/, Player Characters/ (README placeholder)
├── Adventures/    Arcs/, Quests & Hooks/, Encounters/
├── Sessions/      Session Prep/ (Session 0 and Session 1), Session Logs/, Roll Logs/ (dice history)
├── Game Master/   Templates/, Random Tables/, Handouts/
├── Maps/
└── Assets/        Art/, Handouts/, Tokens/
```

The Story Development wiki (index, folder picker, new document, move and new folder) reads these roots in addition to the book roots, using `CAMPAIGN_ROOTS` in `campaign_wizard.rs`. Existing book projects keep their layout and open unchanged.

Campaign metadata (system, edition, genre, tone, table settings, conflict, starter, setup mode, and `campaign_id` equal to the project id) lives in the YAML frontmatter of `Campaign Overview.md`. Every seeded document gets a core stable ID (`note-`, `chr-`, `loc-`, `org-`, `obj-`, `evt-`, `plt-`, `tpl-`).

The creator refuses a non-empty destination, including an existing Catalyst project, before writing anything.

## Deferred integrations

- **Character Studio / Character Sheet Designer**: the next milestone. Player Characters only gets a README placeholder that the index ignores.
- **Map Studio, encounter tools**: their folders (`Assets/Maps`, `Adventures/Encounters`) exist but nothing is seeded into them.
- GM templates are ordinary `template` documents in `Game Master/Templates`. They are not yet registered in the entity-template registry.
- The legacy New Book Foundation dialog and command are still in place but are no longer linked from the UI.
