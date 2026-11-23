# 024 — Catalyst Complete Engine Build Register and Ownership Matrix

> **Count update:** initial S01–S29 + M01–M18 + C01–C12 = 59; architecture 026 adds M19–M21, so current planned total = **62**. Existing main module table remains the original 18, with the three additions in §4A.

**Status:** Approved scope and assignment matrix; implementation statuses remain **unverified** pending code-level audit  
**Updated:** 2026-11-23  
**Repositories:** [Catalyst](https://github.com/Stormchyld2/Catalyst) and [Catalyst Character Studio](https://github.com/Stormchyld2/Catalyst-Character-Builder)  
**Sources of detail:** [023 — Shared and Module Engines](023-CATALYST-SHARED-AND-MODULE-ENGINES.md) and Character Studio [010 — Engine Build Plan](https://github.com/Stormchyld2/Catalyst-Character-Builder/blob/docs/architecture-foundation/docs/architecture/010-CHARACTER-ENGINE-BUILD-PLAN.md).

This is the **complete ownership and build-action register**, not a claim that every engine still needs to be created from scratch. Actual status must be determined by reading current code, PRs, tests and build logs before selecting work. Keep this document in sync with 023 and the Character Studio 010 specification when assigning or moving ownership.

## 1. Implementation ownership rules

- **Catalyst Character Studio team (user + ChatGPT, separate repository)** owns **M01–M04 and C01–C12**, plus incubation of **pure reusable S04–S06 rules mechanics**. These shared mechanics remain portable, framework-independent, and must use reviewed interfaces that the main Catalyst app can consume.
- **Catalyst main app/Claude** owns S01–S03 shared infrastructure adapters and S07–S29 other platform engines. **S07/S08 are already an active dice workstream**, so finish/review that implementation before changing contracts.
- **Catalyst main app/Claude** owns M05–M18. M05 is the Campaign Wizard workstream, which should not silently be replaced by a second implementation.
- **Cross-repository contracts:** final Catalyst integration uses its authoritative files, ID/link services, dice engine, PDF/sourcebooks, permission services and print/export primitives. Standalone Character Studio may have test adapters, **not competing global engines**.
- No code moves or dependency vendoring without explicit interface tests, pinned donor revisions, license notices and actual tests. No GPL donor source imported by default.
- Pure shared engine interfaces must not import Svelte/Tauri/browser UI or be coupled to one desktop app.
- **Definitions of status:** `Unverified` = not yet inspected in source. `Existing`, `Partial`, `Planned`, `Blocked`, `Tested` require commit/path/test evidence. All rows below start Unverified intentionally; the **owner** and **work target** are decisions, not observed build status.

## 2. Shared engine register — all 29 engines

| ID | Engine | Implementation owner | Required build action | Priority | Verified status |
|---|---|---|---|---|---|
| S01 | **Project/File Authority & Transaction Engine** (existing project/document services are source) | Catalyst main repository | Audit and adapt inherited services; do not replace | P0 — extend, not replace | Unverified |
| S02 | **Stable Identity, References & Link Graph** (existing ID/link service) | Catalyst main repository | Audit and adapt inherited services; do not replace | P0 — reuse | Unverified |
| S03 | **Schema, Template & Custom Field Engine** (existing template registry adapter) | Catalyst main repository | Audit and adapt inherited services; do not replace | P0 | Unverified |
| S04 | **Rules Pack & Content Registry** | Character Studio develops pure core; Catalyst owns integration/API | Build reusable core in Character Studio; publish contract/fixtures; integrate in Catalyst | P0 | Unverified |
| S05 | **Declarative Formula & Modifier Engine** | Character Studio develops pure core; Catalyst owns integration/API | Build reusable core in Character Studio; publish contract/fixtures; integrate in Catalyst | P0 | Unverified |
| S06 | **Choice, Prerequisite & Progression Engine** | Character Studio develops pure core; Catalyst owns integration/API | Build reusable core in Character Studio; publish contract/fixtures; integrate in Catalyst | P0 | Unverified |
| S07 | **Dice Notation & Outcome Engine** | Catalyst dice workstream (Claude) | Finish and test existing dice work; no parallel rewrite | P0 | Unverified |
| S08 | **3D Dice Presentation/Physics Adapter** | Catalyst dice workstream (Claude) | Finish and test existing dice work; no parallel rewrite | P1 — active Claude work | Unverified |
| S09 | **State, Event Log & Undo Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P1 | Unverified |
| S10 | **Permission, Visibility & Redaction Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P1 | Unverified |
| S11 | **Search, Index & Cross-Reference Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P1 | Unverified |
| S12 | **Document Render, PDF & Extraction Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P1 | Unverified |
| S13 | **Sourcebook & License Provenance Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P1 | Unverified |
| S14 | **Print/Layout/Export Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P1 | Unverified |
| S15 | **Asset Library & Media Transform Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P1 | Unverified |
| S16 | **Table, Random Generator & Seed Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P1 | Unverified |
| S17 | **Spatial, Grid & Geometry Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P2 | Unverified |
| S18 | **Map Document, Layer & Visibility Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P2 | Unverified |
| S19 | **Timeline, Calendar & World State Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P2 | Unverified |
| S20 | **Graph, Relationship & Workflow Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P2 | Unverified |
| S21 | **Turn, Initiative & Combat Resolution Core** | Catalyst main repository | Audit existing code, then create or extend as needed | P2 | Unverified |
| S22 | **Loot, Inventory, Equipment & Economy Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P2 | Unverified |
| S23 | **Scene Presentation & Session Orchestration Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P2 | Unverified |
| S24 | **Audio Playback/Cue Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P3 | Unverified |
| S25 | **LAN Sync, Presence & Transport Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P3 | Unverified |
| S26 | **Import/Export & Compatibility Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P2 | Unverified |
| S27 | **Package/Extension Runtime** | Catalyst main repository | Audit existing code, then create or extend as needed | P2 | Unverified |
| S28 | **Local Backup, Recovery & Diagnostics Engine** | Catalyst main repository | Audit existing code, then create or extend as needed | P1 | Unverified |
| S29 | **Localization, Units & Accessibility Core** | Catalyst main repository | Audit existing code, then create or extend as needed | P2 | Unverified |

**Ownership precision:** S04/S05/S06 **development is in Character Studio**; parent Catalyst maintains the shared interfaces and integrates those engines. S07 remains the single authoritative dice evaluator. S08 is presentation and physics only. S12 PDF mechanics and S13 source ownership/citations are separate; both are global Catalyst services.

## 3. Module-owned engine register — all 18 engines

| ID | Engine and purpose | Implementation owner | Required build action | Dependencies | Verified status |
|---|---|---|---|---|---|
| M01 | **Guided Character Builder workflow**: edition-driven steps, UI cards, drafts, selection review, incomplete-choice dashboard | Catalyst Character Studio | Build in separate repo; integrate with parent later | S01–S07, S15 | Unverified |
| M02 | **Interactive Character Sheet renderer/designer**: widgets, editable fields, tabs, skins and layouts | Catalyst Character Studio | Build in separate repo; integrate with parent later | S03, S05, S09, S14, S15 | Unverified |
| M03 | **Character advancement UX**: level-up walkthrough, before/after diff and approval | Catalyst Character Studio | Build in separate repo; integrate with parent later | S04–S06, S09, S10 | Unverified |
| M04 | **NPC/Creature stat block composer** | Catalyst Character Studio | Build in separate repo; integrate with parent later | S03–S06, S14, S22 | Unverified |
| M05 | **New Campaign seeding orchestrator**: Quick Start, 10-step Guided Setup, folder plan and atomic creation | Catalyst Campaign Wizard workstream (Claude) | Continue existing wizard branch; verify current progress | S01–S04, S15, S26 | Unverified |
| M06 | **Worldbuilding profile/editor**: governments, nations, locations, cultures, religions, flora/fauna | Catalyst main repository | Audit then implement in dedicated feature milestone | S02, S03, S15, S19, S20 | Unverified |
| M07 | **Quest/scene branching editor and puzzle/trap composer** | Catalyst main repository | Audit then implement in dedicated feature milestone | S02, S03, S09, S20, S23 | Unverified |
| M08 | **Encounter composition and difficulty estimator**: rules-dependent budgets, groups, environments | Catalyst main repository | Audit then implement in dedicated feature milestone | S04–S07, S16, S21, S22 | Unverified |
| M09 | **Manual map editor**: brushes, drawing, object tools, undo, floors and export | Catalyst main repository | Audit then implement in dedicated feature milestone | S09, S15, S17, S18 | Unverified |
| M10 | **Procedural map generator adapters**: world/regional/settlement/dungeon/battle maps | Catalyst main repository | Audit then implement in dedicated feature milestone | S15–S18, S16 | Unverified |
| M11 | **GM command dashboard**: party, encounters, timeline, initiative, secrets, roll console | Catalyst main repository | Audit then implement in dedicated feature milestone | S02, S07–S10, S19–S23 | Unverified |
| M12 | **Session prep, chronology, recaps and continuity reports** | Catalyst main repository | Audit then implement in dedicated feature milestone | S02, S09, S11, S19, S23 | Unverified |
| M13 | **Reader/UI and extraction-review workflow** | Catalyst main repository | Audit then implement in dedicated feature milestone | S03, S11–S13, S15 | Unverified |
| M14 | **RPG-specific print profiles and page-layout editor** | Catalyst main repository | Audit then implement in dedicated feature milestone | S03, S14, S15 | Unverified |
| M15 | **Player-safe tactical presentation, fog/tokens UI** | Catalyst main repository | Audit then implement in dedicated feature milestone | S08, S10, S17, S18, S23, S25 | Unverified |
| M16 | **Hex crawl/travel route rules adapter** | Catalyst main repository | Audit then implement in dedicated feature milestone | S04, S16–S19, S22 | Unverified |
| M17 | **Crafting recipes, downtime and domain rules adapter** | Catalyst main repository | Audit then implement in dedicated feature milestone | S04–S06, S19, S22 | Unverified |
| M18 | **Clue graph/handout workflow** | Catalyst main repository | Audit then implement in dedicated feature milestone | S02, S09–S11, S20, S23 | Unverified |

## 4. Character-specific engine register — all 12 engines

The following rows mirror Character Studio's approved implementation plan, not an independent parent-app task list. **Do not assign these to Claude working in the main Catalyst repository.**

| ID | Engine | What must be built | Implementation owner | Verified status |
|---|---|---|---|---|
| C01 | Character Record, Draft & Migration | Typed canonical records, stable IDs, schema versions, safe saves, wizard drafts, unknown-field roundtrips, migration previews | Catalyst Character Studio repo (user + ChatGPT) | Unverified |
| C02 | Character Option Catalog & Source Filter | Rules-pack, edition and licensed-source-aware species/class/subclass/background/feat/spell/equipment catalogs; browse/search/filter | Catalyst Character Studio repo (user + ChatGPT) | Unverified |
| C03 | Build Legality & Completeness Validator | Prerequisite checks, selection counts, mutually exclusive choices, pending decisions, clear error explanations | Catalyst Character Studio repo (user + ChatGPT) | Unverified |
| C04 | Multiclass, Feature & Advancement Planner | Level and subclass progression, feature grants, feats/ability improvements, multiclass requirements and before/after diffs | Catalyst Character Studio repo (user + ChatGPT) | Unverified |
| C05 | Spellbook & Power Management | Known/prepared spells, available spells, spell lists, slots, casting resources, conditions like concentration when pack defines them | Catalyst Character Studio repo (user + ChatGPT) | Unverified |
| C06 | Equipment Effects & Loadout | Inventory, worn/equipped/attuned state, weapons/armor/defenses, carrying limits, equip-dependent modifiers | Catalyst Character Studio repo (user + ChatGPT) | Unverified |
| C07 | Character Resources, Recovery & Conditions | HP/temporary HP, rests, exhaustion/other conditions, death saves and non-D&D resource models driven by pack | Catalyst Character Studio repo (user + ChatGPT) | Unverified |
| C08 | Action & Roll Binding | Build roll actions/expressions for checks, saves, attacks, damage or abilities using the **parent shared dice engine contract**; do not implement separate dice evaluation | Catalyst Character Studio repo (user + ChatGPT) | Unverified |
| C09 | Character Sheet Binding & Theme | Render fields and derived stats into customizable widgets, tabs, responsive design, decorative/clean/ink-friendly skins | Catalyst Character Studio repo (user + ChatGPT) | Unverified |
| C10 | Character Sourcebook Linker | Edition, sourcebook and page references for character options using the **parent shared Sourcebook Library PDF/citation API** | Catalyst Character Studio repo (user + ChatGPT) | Unverified |
| C11 | Character Import/Export & Print Profiles | Native Markdown/YAML/JSON roundtrips, character import mapping, blank/completed/decorative PDF sheet profiles using shared print interfaces | Catalyst Character Studio repo (user + ChatGPT) | Unverified |
| C12 | Character History, Diff & Reconciliation | Auditable changes, advancement history, undo/recovery where valid, rule-pack upgrades, conflicting manual overrides and template migrations | Catalyst Character Studio repo (user + ChatGPT) | Unverified |

## 4A. Remote and Android module additions (approved after initial 59-engine inventory)

| ID | Engine / module | Implementation owner | Required work | Status |
|---|---|---|---|---|
| M19 | Catalyst Remote — GM mobile browser workspace | Parent Catalyst / Claude | QR-paired local browser controller; soundboard first, then dice, encounters, maps, references and party actions; uses S25/S10/S24 | Planned; not yet audited |
| M20 | Player Sharing Service adapter | Parent Catalyst / Claude | Character-bound authenticated player endpoints and projected character/inventory records, GM approvals, safe sync commands and audit; uses S25/S10/S09 | Planned; not yet audited |
| M21 | Android Character Companion | Standalone Character Studio team | Android mobile character sheets/inventory/resources, offline characters, sync client using same character domain; after Windows completion | Planned; not yet audited |

**Parallel-code exclusion:** User reports Claude currently working on soundboard (S24), left panel, Frost Blue theme migration (025), and Campaign Wizard (M05). Treat them as **In progress (user-reported, unverified)**; check active branches and avoid touching their implementation paths. For the remote, implement stable S25/S10/S09 contracts and M19/M20 afterward, without replacing soundboard work.

## 5. Shared engine handoff checklist

Every shared engine must expose a documented versioned typed contract with clear authority:

- `S01–S03`: file/document/ID/schema implementations in parent; Character Studio supplies persistence/template/ID adapters, never its own competing authoritative parent project database.
- `S04–S06`: Character Studio incubates rules-pack registry, deterministic declarative calculations, choice and progression mechanics. Parent approves framework-independent schema, imports tested package source, and applies same version/fixtures to Encounter and GM.
- `S07`: parent owns notation, results, roll history and global floating UI; Character Studio C08 sends typed action intents via DiceEvaluator port. Avoid double random rolls.
- `S09/S10`: parent owns change events and GM/player redaction; Character Studio C12 handles character-specific reconciliation using shared contracts.
- `S12/S13`: parent owns PDF reading/search/citation and legality metadata; Character Studio C10 provides sourcebook/page links only.
- `S14`: parent owns general printing/PDF layout primitives; Character Studio C11 owns character-specific layouts.
- `S15/S22/S26`: parent owns generic asset, inventory/economy and compatibility ports; Character Studio C06/C11 own character use cases and adapters.
- `S17/S18`: parent Map Studio source; never implement a separate spatial/map data authority inside character sheets.
- `S23/S25`: parent GM/LAN source; Character Studio supports host-authoritative, player-safe integration later, but doesn't independently implement global multiplayer transport.

### How to avoid duplicate work

Before Claude starts any engine, compare engine ID, intended owner, actual repository paths, open PRs, source schemas, and tests. An existing engine with passing tests should be reused with adapters. When implementation ownership changes, update both this register and the Character Studio 010 plan before coding.

## 6. Build order and milestones

1. **Audit & baseline:** read actual code in both repos, identify existing/partial/planned with commit/path/test evidence. Report conflicts rather than altering parallel feature branches. Create the shared DTO/schema/contract fixtures.
2. **Character Studio owned milestone:** C01 first; then C02/C03 with S04–S06 pure cores; C09/M02 manual sheet MVP; M01 guided builder; C04–C08/M03 advancement and play; C10–C12/M04 sourcebooks, printing and NPCs.
3. **Parent Catalyst owned milestone:** finish S07/S08 dice work and M05 wizard; audit existing S01/S02/S03 service infrastructure. Establish S04–S06 **adapters only**, not duplicate implementations.
4. **Parent shared foundation:** S09–S16, S28; reuse real file/document/search and PDF services where inherited code already works.
5. **World/map/GM:** S17–S23, S26–S27 and M06–M18 based on product milestones.
6. **Later features:** S24/S25/S29, local LAN/Android/player display and advanced audio/accessibility work as dependencies permit.

**Do not implement 62 engines simultaneously.** This list contains **29 shared (S01–S29), 21 module-specific (M01–M21), and 12 character-specific (C01–C12)**—some intentionally wrap or consume shared services and are not separate executable processes. A build addendum must pick a small, testable slice.

## 7. Per-engine execution record (required during implementation)

For each ID, maintain a small record within the PR/implementation tracker:

```yaml
engine_id: S04
implementation_owner: Catalyst Character Studio
status: Unverified
source_repo: Stormchyld2/Catalyst-Character-Builder
files: []
interfaces: []
schema_versions: []
tests_run: []
test_results: []
source_licenses: []
integration_consumers: []
dependencies: [S01, S03]
blockers: []
pull_request: null
```

Do not claim `Tested` without exact commands/results; do not claim `Existing` solely from inherited architecture or a mockup. Code outside this documentation PR is unchanged.

## 8. Acceptance gates

- Working offline from actual local files (Markdown/YAML/JSON); no cloud/account or new authoritative SQLite requirement.
- Stable IDs, schema migration and conflict-free save/reopen, plus deterministic source-aware rule calculations.
- Isolated Svelte/TypeScript/Tauri/Rust boundaries and reusable pure core modules.
- Dice evaluation centralized, no duplicated parser or contradictory roll outcomes.
- Sourcebook PDFs stored locally with rights-aware links; OCR/extraction never automatically activates rules.
- GM-only data filtered at service boundary before any player display/LAN transport.
- For integrated engines: cross-repo contract fixtures pass and imports preserve unknown fields.
- Licensing, attribution, source commit and tests recorded in the implementation PR.

**Definition of done for this register:** the main Catalyst repo contains all shared, module and character-specific engine IDs with required work and named implementation ownership. **Definition of done for the engines themselves is separate and requires audited code and tests.**
