# 020 — Catalyst Campaign Wizard & RPG Project Architecture

**Status:** Approved design baseline (2026-11-23)  
**Scope:** New Campaign Wizard; campaign folder and metadata schema; worldbuilding starter templates; offline Game Master reference library; compatibility and future integrations.  
**Authority:** Supersedes `016-NEW-BOOK-FOUNDATION.md` **for new Catalyst campaigns only**. Do not rewrite existing book project structures.  
**Related:** `001-PROJECT-FILE-FORMAT.md`, `002-DOCUMENT-MODEL.md`, `003-STABLE-ID-SYSTEM.md`, `004-CANON-MODEL.md`, `005-PLUGIN-ARCHITECTURE.md`.

## 1. Product and storage rules

Catalyst is an offline-first tabletop RPG campaign, worldbuilding, adventure and Game Master studio. New projects are campaigns, **not books**. Preserve the tested Svelte 5, Tauri 2, Rust, stable-ID, Markdown/YAML/JSON and template foundations while adapting their workflows to tabletop roleplaying.

The wizard is a **temporary seeder**. After Create, ordinary files are authoritative. No cloud, account, online AI, runtime CDN or new mandatory database is allowed. Existing SQLite code may remain for backward compatibility, but new RPG document authority must not reside in SQLite. No automatic import or modification of original LithoMark projects.

The current `project.yaml` manifest and project discovery services are still used by inherited code. Keep that bootstrap format compatible, add separate RPG domain metadata in `campaign.yaml`, and avoid two mutable copies of the same truth. The legacy `lithomark-project` format identifier is an **internal compatibility marker**, not user-facing branding. Change it only through a tested migration.

World Builder inherits the useful Story Development wiki/editor, templates, relationships, timeline and canon tools. Adventure Studio handles quests, branching narrative, scenes and encounters. The wizard must be able to create valid campaigns even when Character Studio, Map Studio and GM Console are unfinished.

## 2. Entry workflow

**New Campaign** presents two paths. Both generate the **same manifest schema and full directory tree**.

**Quick Start:** required nonblank campaign name, explicit local destination directory, rules system (default: **System-Neutral**); optional premise. Never require character stats, worldbuilding details, first encounter or maps. Create a blank usable campaign quickly.

**Guided Setup:** ten optional-content steps below. Preserve existing previous/next, step navigation, validation, folder picker, review and seeding logic wherever useful. All nonessential sections may be skipped. Do not auto-invent fictional facts to fill blanks.

Flow: Dashboard → New Campaign → Quick Start / Guided Setup → Review planned files → Validate → Create in safe staging area → Verify → Publish → Open Campaign Dashboard.

Before Create, wizard state is temporary. Optional resume drafts are allowed only in Catalyst-owned temporary app storage; never treat them as a competing campaign authority. Cancel must not change the destination.

## 3. Approved ten-step Guided Setup

1. **Campaign Basics:** name, subtitle, description/premise, genre, theme/tone, artwork, optional GM display name, starting date and local destination.
2. **Rules & Game System:** System-Neutral, installed verified rules packs (possibly licensed 5e-compatible), Custom/Homebrew; optional advancement, start level, house rules, dice preferences and difficulty. No D&D-only assumptions.
3. **World & Setting:** new, linked shared, imported or blank world; optional world name, era, calendar, broad geography and setting notes. Shared worlds must not become writable campaign directories.
4. **Worldbuilding Foundations:** optional government, nations, geography, settlements, lore, history, cultures/species, religion, magic, languages, trade, technology, factions, laws and natural world entries.
5. **Players & Characters:** optional players, party name, starting progression, imported characters or minimal placeholders. Full builder is deferred.
6. **NPCs, Creatures & Factions:** optional allies, villains, organizations, creature profiles, motivations and relationships; combat statistics are not required for every NPC.
7. **Adventure & Campaign Story:** central conflict, starting adventure, story arcs, main/side quests, objectives, branches, milestones, rewards and consequences. No mandatory manuscript chapter structure.
8. **Locations & Maps:** starting location, settlements, regions/dungeons, existing map references. Do not pretend Map Studio has generated files until implemented.
9. **Session Setup:** optional first session, GM notes, encounter references, tabletop style (physical/hybrid/digital), dice and player-display preferences. No dependency on GM Console completion.
10. **Review & Create:** comprehensive selections summary, files/folders preview, selected rules-pack/version, warnings, validation and deliberate Create action.

Allow back-navigation, correction, skip, and abandonment. Fields must only be mandatory when needed for a valid project and selected rules pack. All completed choices must survive navigation within the wizard.

## 4. Required physical campaign layout

Create the following actual folders for both creation paths. Do not keep `Manuscript/` as a new-campaign root. Empty directories must physically exist after initialization, even though Git cannot track them.

```text
Campaign Name/
├── project.yaml
├── campaign.yaml
├── Worldbuilding/
│   ├── Geography & Climate/
│   ├── Governments & Politics/
│   ├── Kingdoms & Nations/
│   ├── Settlements & Locations/
│   ├── History & Timelines/
│   ├── Lore & Legends/
│   ├── Cultures & Species/
│   ├── Religions & Deities/
│   ├── Magic & Supernatural/
│   ├── Languages/
│   ├── Economy & Trade/
│   ├── Technology & Crafting/
│   ├── Organizations & Factions/
│   ├── Laws & Customs/
│   └── Natural World/
├── Characters/
│   ├── Player Characters/
│   ├── Non-Player Characters/
│   ├── Character Templates/
│   └── Portraits & Tokens/
├── Creatures & Monsters/
│   ├── Bestiary/
│   ├── Custom Creatures/
│   └── Encounter Groups/
├── Adventures/
│   ├── Main Campaign/
│   ├── Side Quests/
│   ├── Quest Chains/
│   ├── Scenes/
│   └── Puzzles & Traps/
├── Encounters/
│   ├── Combat/
│   ├── Social/
│   ├── Exploration/
│   └── Random Tables/
├── Maps/
│   ├── World/
│   ├── Regional/
│   ├── Settlements/
│   ├── Dungeons/
│   └── Battle Maps/
├── Items & Equipment/
│   ├── Weapons & Armor/
│   ├── Magic Items/
│   ├── Consumables/
│   ├── Treasure & Loot/
│   └── Crafting/
├── Rules & Mechanics/
│   ├── Rules Packs/
│   ├── House Rules/
│   ├── Conditions/
│   └── Spells & Abilities/
├── Sessions/
│   ├── Session Plans/
│   ├── Session Logs/
│   ├── Recaps/
│   └── Campaign State/
├── Game Master/
│   ├── Secrets & Clues/
│   ├── GM Notes/
│   ├── Random Tables/
│   ├── Handouts/
│   ├── Screen References/
│   └── Guides/
├── Assets/
│   ├── Images/
│   ├── Audio/
│   ├── Map Textures/
│   └── Tokens/
└── Exports & Print/
    ├── Character Sheets/
    ├── Maps/
    ├── Handouts/
    └── Campaign Reports/
```

If internal services cannot immediately index this tree, create a documented adapter; do not misrepresent the manuscript hierarchy as the target RPG format. Do not generate hundreds of blank worldbuilding documents. Seed only supplied entries or explicitly selected template examples.

## 5. Manifests and entity records

Use `project.yaml` for existing project discovery, project ID, title, project type, timestamps and compatibility-required properties. For new campaigns `project_type` must identify a campaign. `campaign.yaml` holds RPG-only details such as schema version, same project ID, rules-pack ID/version, optional world reference, campaign-local world-state path and default play preferences.

Conceptual example, adapt to real `ProjectManifest` validation:

```yaml
# project.yaml
format: lithomark-project # transitional format compatibility
format_version: 1
project_id: 550e8400-e29b-41d4-a716-446655440000
title: The Ashen Coast
project_type: campaign
created_at: '2026-11-23T00:00:00Z'
```

```yaml
# campaign.yaml
schema_version: 1
project_id: 550e8400-e29b-41d4-a716-446655440000
rules:
  pack_id: system-neutral
  pack_version: '1'
world:
  reference_id: null
  local_state_path: Sessions/Campaign State/
play:
  mode: physical
  dice: physical
  player_display: false
starting_session_id: null
```

One canonical project ID; validate both manifests agree. Do not duplicate mutable project title or lore in two authoritative files. Version all new schemas, validate on read, and specify migration before changing a released schema.

Narrative entities: Markdown + YAML frontmatter; structured tables/settings: YAML/JSON; assets: local files referenced by relative safe paths. Stable type-compatible IDs from the existing ID service—not filenames, titles or paths. Moves/renames preserve references. Resolve links with existing shared linker. Report duplicate/missing/ambiguous IDs rather than silently repairing.

## 6. Worldbuilding template families

Each record supports optional template-defined fields, title, aliases, tags, images, Markdown body, stable ID, link IDs to related entities, timestamps/source metadata, and private/public/undiscovered visibility metadata. The GM must control what becomes established canon.

| Category | Illustrative fields |
|---|---|
| Government & Politics | type, leaders, succession, laws, alliances |
| Kingdoms & Nations | ruler, capital, borders, population, trade, conflicts |
| Settlements & Locations | parent location, type, inhabitants, sites, map link |
| Geography & Climate | biome, terrain, seasons, weather, hazards |
| History & Timelines | date, era, participants, cause, consequence |
| Lore & Legends | legend, versions, evidence, truth, known-by |
| Cultures & Species | customs, physiology, languages, relationships |
| Religions & Deities | beliefs, pantheon, temples, clergy, holidays |
| Magic & Supernatural | sources, schools, costs, limits, institutions |
| Languages | script, speakers, names, vocabulary |
| Economy & Trade | currency, commodities, industries, trade routes |
| Technology & Crafting | level, methods, inventions, materials |
| Organizations & Factions | goals, leadership, memberships, reputation |
| Laws & Customs | jurisdiction, norms, prohibitions, enforcement |
| Natural World | flora, fauna, ecology, habitats, dangers |

Support hierarchical places (world → region → nation → settlement → building → room) through stable-ID parent links with cycle protection. A location can also link to NPCs, factions, quests and maps. Inferred or generator-proposed facts never silently become campaign canon.

Reuse inherited document types/template registry where safe, but allow new RPG-specific subtypes and plugins to register fields without writing a competing database.

## 7. Character Studio handoff (not implemented in this milestone)

The campaign wizard creates directories and optionally minimally populated player/NPC records; it must not require the Character Builder.

Approved later design:
- **Guided Character Builder**: rules-driven steps, choices, validation, derived stats and printable/digital character sheet.
- **Manual Creation**: immediate editable blank sheet, either rules-assisted or fully freeform.
- **Single system-independent authoritative character record** for both methods, with dynamic rules-pack fields.
- Advancements, custom sheet designer, decorative/ink-friendly print, and shared 3D dice roller integration.
- System-Neutral works with no D&D ability/class assumptions. A 5e-compatible pack may ship only under verified content rights.

Record pending character integrations explicitly. Do not label unfinished placeholders as functioning character sheets.

## 8. Adventures, sessions, worlds and GM privacy

Adventures contain quests, branches, objectives, consequences, scenes and puzzles. Encounters include combat/social/exploration and link by stable IDs to characters, creatures, maps and rewards. Sessions contain preparations, chronological logs, player discoveries, recaps and events affecting campaign state. Maps and encounters may be referenced now without being generated.

One shared world can back several campaigns, but **campaign-local state changes must not mutate the shared world or another campaign**. Explicit world reference boundaries are an approved exception to normal project isolation; implement read-only base plus campaign-specific overrides/events.

Metadata should support GM-only, public, undiscovered and later player-specific access. In future LAN/player display, enforce privacy before serializing/transmitting data; CSS hiding is not security.

## 9. Offline Game Master starter references

Provide a reusable read-only **local GM Reference Library** accessible from all campaigns, preferably installed once and optionally copied into a specific campaign for customization.

Create original, legally redistributable guides on:
- Starting a tabletop RPG, choosing a game system, Session Zero and table expectations.
- Planning campaigns, quests, branches, character goals and consequences.
- Building worlds: geography, governments, religions, magic, cultures, economy.
- NPCs, factions, relationships and dialogue.
- Combat, social, exploration and environmental encounters.
- Dungeon, traps, puzzles, wilderness and travel.
- Improvisation, player agency, secrets, continuity and session recaps.
- Dice notation, probabilities, private/public rolls, GM checklists.

Add original editable random tables: NPC names, motivations, settlements, taverns, rumors, adventure hooks, wilderness discoveries, weather, complications and treasure ideas.

**Licensing gate:** Ship only original material, verified public domain, compatible Creative Commons or explicitly permitted SRD content. Record author/source/edition/license/attribution for every external item and verify art/fonts separately. Free-to-read is not license to redistribute. No remote dependencies. Until a source is cleared, omit it instead of treating an unlicensed web guide as bundled.

## 10. Safe creation pipeline

1. Validate chosen absolute destination, permissions, name, supported rules pack, path traversal/symlink safety and collision status.
2. Prepare user-visible creation plan (directories, selected documents, rules pack, starting state).
3. Stage files in a unique temporary sibling directory **on the same filesystem**; don't touch an unrelated existing destination.
4. Use existing manifest/stable-ID services to generate required metadata, full folder tree, selected entity files, optional starter notes and relationships.
5. Validate all manifests, file contents, duplicate IDs, required links and expected directories before publication.
6. Publish by safe rename/commit; avoid partially initialized visible campaigns.
7. Register/open the new campaign via existing project root service and navigate to Campaign Dashboard.
8. If anything fails, retain/report recoverable staging state or clean up only files owned by the attempted creation; never delete an existing user's project.

No file creation on every input change. No silent merge/overwrite. Do not expose mock/default novel data as a real campaign. Atomic writes and recovery follow the inherited Document Service contract.

## 11. Testing and release acceptance

Automated Rust/TS/UI tests plus manual Windows smoke checks must cover:

- Quick Start creates valid metadata and the **complete real folder hierarchy** without requiring worldbuilding.
- Guided Setup completes all 10 steps, supports skip/back/navigation and preserves entered values.
- Generated explicit NPC/location/quest items have stable IDs and correct folder destinations; no unwanted blank files.
- Rules-pack references validate; System-Neutral creates a playable blank campaign.
- Campaign opens, closes and reopens with all data intact and links resolving; IDs survive rename/move.
- Two campaigns cannot mutate each other's files or a shared-world base.
- Existing book projects still open without silent mutation/migration.
- Invalid paths, existing destinations, permission failures, corrupt manifests and interrupted staging report cleanly without data loss.
- GM privacy metadata is stored correctly; no claim of secure LAN filtering until implemented.
- Reference library and all dependency assets function without internet.
- Production build passes; no new frontend/Rust failures. Report inherited test failures separately.

## 12. Implementation stages and handoff

1. Audit actual `NewBookFoundationDialog.svelte`, `new_book_foundation.rs`, `project_root.rs`, document service, entity templates and navigation.
2. Implement **Quick Start and file-authoritative campaign seeder**, with correct real directories, manifests and tests.
3. Replace the inherited novel wizard with the ten-step **Guided Setup** for campaigns.
4. Add reusable worldbuilding entity templates, selected record creation and safe links.
5. Add original offline GM starter references, attribution manifests and access links.
6. Connect finished RPG workspaces later; Character Builder is a separate milestone.

Keep wizard work on its own feature branch and PR, distinct from dice roller and frontend stabilization. Document deviations from this baseline with reasons and update tests. Do not claim completion based on architecture docs alone.

**Definition of done:** A user can create a real tabletop campaign through Quick Start or Guided Setup, inspect the approved folders on disk, reopen and edit its authoritative files offline, and access working starter/reference resources without novel-specific requirements.