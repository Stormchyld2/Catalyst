# Catalyst New Campaign Wizard

**Status:** Implemented (first milestone)
**Builds on:** [016 New Book Foundation](016-NEW-BOOK-FOUNDATION.md). The same rules apply: setup state is temporary, Create Campaign is the persistence boundary, and the generated Markdown/YAML files with stable IDs are authoritative afterward. No database is involved.

## Entry points

Home "New Campaign", File > New Project, and the Story Development rail button (with no project open) open `CampaignWizardDialog.svelte`. The person picks one of two modes:

- **Quick Start**: name, game system, folder, optional pitch and a campaign starter. The backend fills in the defaults and installs every Game Master template.
- **Guided Setup** (10 steps): Campaign Basics, Players & Table, World Overview (at least one detail is required), Worldbuilding Categories, Locations, NPCs & Villains, Story & Hooks (plus the starter), First Session, Game Master Toolkit, Review & Create.

Both modes call `create_new_campaign` (`src-tauri/src/campaign_wizard.rs`), register the folder in the project registry, activate the project and open Story Development.

## Storage

`project.yaml` uses the existing `lithomark-project` v1 format with `project_type: campaign`, so existing projects and the reopen path are unchanged. All of the shared Catalyst folders are still created. These campaign folders are added under `Story Development/` so the existing wiki, file tree, search and Overview card work without changes:

```text
Story Development/
├── Campaign/                 Campaign Overview.md (metadata), Table & Safety.md, House Rules.md
├── Adventures/               Arcs/, Quests & Hooks/, Encounters/
├── Sessions/                 Session Prep/ (Session 0 and Session 1), Session Logs/
├── Characters/               NPCs/, Villains & Antagonists/, Player Characters/ (README placeholder)
├── Worldbuilding & Settings/ Overview.md plus category folders
└── Game Master/              Templates/, Random Tables/, Handouts/
Assets/Handouts, Assets/Tokens
```

Campaign metadata (system, edition, genre, tone, table settings, conflict, starter, setup mode, and `campaign_id` equal to the project id) lives in the YAML frontmatter of `Campaign Overview.md`. Every seeded document gets a core stable ID (`note-`, `chr-`, `loc-`, `org-`, `obj-`, `evt-`, `plt-`, `tpl-`).

The creator refuses a non-empty destination, including an existing Catalyst project, before writing anything.

## Deferred integrations

- **Character Studio / Character Sheet Designer**: the next milestone. Player Characters only gets a README placeholder that the index ignores.
- **Map Studio, encounter tools**: their folders (`Assets/Maps`, `Adventures/Encounters`) exist but nothing is seeded into them.
- GM templates are ordinary `template` documents in `Game Master/Templates`. They are not yet registered in the entity-template registry.
- The legacy New Book Foundation dialog and command are still in place but are no longer linked from the UI.
