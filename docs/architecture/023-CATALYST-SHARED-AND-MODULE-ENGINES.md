# 023 — Catalyst Engine Architecture and Reuse Contracts

**Status:** Approved planning baseline; inventory, not proof of implementation  
**Updated:** 2026-11-23  
**Scope:** Engines shared by Catalyst, Catalyst Character Studio, and future companion apps; module-owned engines; dependencies, API contracts, ownership, milestones and test gates.  
**Authority:** Apply alongside architecture 020 (Campaign Wizard), 021 (Character Studio), 022 (Sourcebook Library) and existing stable-ID/document service contracts. Do not overwrite active workstreams.

## 1. Intent and decision rules

Catalyst is an offline-first tabletop RPG operating environment. **Create reusable platform engines once, then let multiple workspaces consume them through stable contracts.** No duplicate rules evaluator inside Character Studio and Encounter Builder; no second dice parser inside Character Studio; no separate sourcebook indexing subsystem per workspace.

The standalone **Catalyst Character Studio** repository is a *temporary build/test host* for reusable engines and a future integrated workspace. It must not become a second owner of Catalyst's shared engine contracts. Establish versioned contract fixtures and an extraction/integration plan before copying implementations.

A shared engine is a service or library with at least two plausible consumers and no UI/workspace dependency. A module-specific engine implements domain behavior uniquely owned by one workspace. Promote module-specific functionality to shared only after a second actual consumer and stable abstraction are demonstrated.

Engine state must be clearly identified as:
- **Authoritative**: normal project files; user-authored records.
- **Derived/rebuildable**: indexes, render caches, calculation projections, thumbnails, graph layouts.
- **Ephemeral**: UI session state, drafts, dice animation state.
- **Event/history**: session rolls, state mutations and audit/undo history persisted as portable files.

Compatibility: Windows/Svelte 5/TypeScript/Tauri 2/Rust now; offline Android and LAN later. Avoid hard OS bindings in pure engine packages. Preserve existing working LithoMark-derived services and literal compatibility keys unless a tested migration is approved.

## 2. Shared engine registry (cross-workspace)

ID | Engine / authoritative ownership | Consumers | MVP contract and outputs | Priority
---|---|---|---|---
S01 | **Project/File Authority & Transaction Engine** (existing project/document services are source) | All modules | safely read/write/version portable Markdown/YAML/JSON, atomic transactions, schema migrations, recovery, path guarding, project registry | P0 — extend, not replace
S02 | **Stable Identity, References & Link Graph** (existing ID/link service) | Character, World, Adventure, Map, GM, Sourcebooks | assign typed IDs, resolve/reverse links, detect ambiguous/broken refs, preserve rename/move | P0 — reuse
S03 | **Schema, Template & Custom Field Engine** (existing template registry adapter) | Wizard, Character, World, NPC, Items, Sourcebooks, Sheets | versioned schemas, typed/repeating fields, validation, profile templates, safe defaults, migrations | P0
S04 | **Rules Pack & Content Registry** | Character, Encounter, GM, Dice, Items, Spells, Monsters | install/version/validate licensed pack data, expose typed options, dependencies and edition/source provenance | P0
S05 | **Declarative Formula & Modifier Engine** | Character, Combat, Encounters, Items, Economy, Travel | safe expression AST/DSL, dependency graph, cycle detection, source-aware calculation traces, deterministic values | P0
S06 | **Choice, Prerequisite & Progression Engine** | Character builder, Level Up, NPC builder, Crafting/Rewards | selection limits, eligibility, outstanding choices, impact preview, advancement transactions and validation | P0
S07 | **Dice Notation & Outcome Engine** | 3D roller, Character, Encounters, GM, Random Tables | parse dice expressions, RNG with provenance, modifiers/advantage when pack allows, structured results and roll history interface | P0
S08 | **3D Dice Presentation/Physics Adapter** | floating global dice tray, GM, player-facing view | consume verified rolls, render physics/animations/themes/audio, support offline fallback; keep visual physics distinct from authoritative outcome | P1 — active Claude work
S09 | **State, Event Log & Undo Engine** | Sessions, GM, Character, Combat, World, Quests | durable event IDs, actor/action/time, reversible commands where valid, snapshots/replay with schema migration | P1
S10 | **Permission, Visibility & Redaction Engine** | GM, Player Display, LAN, Character, Notes, Maps, Sourcebooks | enforce GM/private/public/undiscovered/owner access at service serialization boundary; audit changes | P1
S11 | **Search, Index & Cross-Reference Engine** | Sourcebooks, Character, World, Journal, Maps, Rules | document/rule/PDF text indexing with scoped queries, filters, backlinks, citations; indexes disposable | P1
S12 | **Document Render, PDF & Extraction Engine** | Sourcebooks, Print Studio, Character, GM, Handouts | local PDF reading/search, text layer, optional OCR adapter, page anchors, thumbnails, safe annotations | P1
S13 | **Sourcebook & License Provenance Engine** | Sourcebook, Character, Rules, World, Export | local owned-book registry, edition/license/source citations, reference page links, user-approved content extraction | P1
S14 | **Print/Layout/Export Engine** | Character, GM, World, Maps, Campaign, Handouts | print page model, PDF generation, US Letter/A4, styles, section flow, editable templates, offline assets | P1
S15 | **Asset Library & Media Transform Engine** | Character portraits/tokens, Maps, World, Audio, Handouts | safe import/dedup/hash, path/rights metadata, resize/thumbnail/crop, portable references | P1
S16 | **Table, Random Generator & Seed Engine** | GM, World, NPC, Encounters, Loot, Travel | declarative weighted tables, deterministic seeded output, provenance and optional commit-to-canon | P1
S17 | **Spatial, Grid & Geometry Engine** | Map Studio, Battle VTT, Encounter, Travel | coordinate projections, square/hex grids, measuring, snapping, walls, hit testing, geometry serialization | P2
S18 | **Map Document, Layer & Visibility Engine** | Map Studio, GM, Player Display, Encounters | structured editable map records, layers, viewport, tokens, fog state, GM/public projection | P2
S19 | **Timeline, Calendar & World State Engine** | World, Adventure, Sessions, GM, Character | arbitrary calendar, events, dated changes, causal links, campaign-local world overlays | P2
S20 | **Graph, Relationship & Workflow Engine** | World, Adventure, Character, NPC Factions, Quest Chains | typed relation graph, validation, derived diagram/layout, quest/scene transitions without database authority | P2
S21 | **Turn, Initiative & Combat Resolution Core** | Encounter, GM Console, Character, VTT | rule-pack-bound initiative/turn/actions/conditions/recovery, portable event results; never hard-code 5e | P2
S22 | **Loot, Inventory, Equipment & Economy Engine** | Character, Encounter, GM, World, Crafting | item catalogs, stacks, quantities, equip/attune state, currency conversions, safe rules-driven effects | P2
S23 | **Scene Presentation & Session Orchestration Engine** | GM, VTT, Player Display, Journal, Maps, Audio | scene state transitions, handouts, visibility, queued cues, session log events | P2
S24 | **Audio Playback/Cue Engine** | GM ambiance, Maps, Sessions, Character effects | local media queue, scene cues, fades, channel volume, saved playlists, no cloud | P3
S25 | **LAN Sync, Presence & Transport Engine** | GM host, Player Display, Character clients, VTT, Android | local discovery/pairing, host-authoritative commands, offline reconnect, version negotiation, redacted state sync | P3
S26 | **Import/Export & Compatibility Engine** | all modules | adapters for native formats and legally supported third-party formats, dry-run mapping, backup, migration and conflict reporting | P2
S27 | **Package/Extension Runtime** | rules packs, generators, map assets, plugins | signed/hashed manifest validation, permission caps, dependency graph, safe sandbox, offline installation | P2
S28 | **Local Backup, Recovery & Diagnostics Engine** | all modules | backup bundles, integrity checks, safe restore, error reports free of sensitive/book content | P1
S29 | **Localization, Units & Accessibility Core** | all workspaces | locale/unit/date formatting, screen-reader labels, keyboard actions, configurable contrast; don't embed English strings in engines | P2

**Important:** S04–S07 are pure domain engines and should be extracted as framework-independent packages from Character Studio if developed there. S08 is renderer/physics only; no separate authoritative dice calculator. S12 and S13 are distinct: document/PDF mechanics versus ownership, citations and licensing.

## 3. Module-specific engine registry (not shared by default)

ID | Owner/workspace | Specialized engine | Depends on
---|---|---|---
M01 | Character Studio | **Guided Character Builder workflow**: edition-driven steps, UI cards, drafts, selection review, incomplete-choice dashboard | S01–S07, S15
M02 | Character Studio | **Interactive Character Sheet renderer/designer**: widgets, editable fields, tabs, skins and layouts | S03, S05, S09, S14, S15
M03 | Character Studio | **Character advancement UX**: level-up walkthrough, before/after diff and approval | S04–S06, S09, S10
M04 | Character Studio | **NPC/Creature stat block composer** | S03–S06, S14, S22
M05 | Campaign Wizard | **New Campaign seeding orchestrator**: Quick Start, 10-step Guided Setup, folder plan and atomic creation | S01–S04, S15, S26
M06 | World Builder | **Worldbuilding profile/editor**: governments, nations, locations, cultures, religions, flora/fauna | S02, S03, S15, S19, S20
M07 | Adventure Studio | **Quest/scene branching editor and puzzle/trap composer** | S02, S03, S09, S20, S23
M08 | Encounter Builder | **Encounter composition and difficulty estimator**: rules-dependent budgets, groups, environments | S04–S07, S16, S21, S22
M09 | Map Studio | **Manual map editor**: brushes, drawing, object tools, undo, floors and export | S09, S15, S17, S18
M10 | Map Studio | **Procedural map generator adapters**: world/regional/settlement/dungeon/battle maps | S15–S18, S16
M11 | GM Console | **GM command dashboard**: party, encounters, timeline, initiative, secrets, roll console | S02, S07–S10, S19–S23
M12 | Session Journal | **Session prep, chronology, recaps and continuity reports** | S02, S09, S11, S19, S23
M13 | Sourcebook Library | **Reader/UI and extraction-review workflow** | S03, S11–S13, S15
M14 | Print Studio | **RPG-specific print profiles and page-layout editor** | S03, S14, S15
M15 | Player Display / VTT | **Player-safe tactical presentation, fog/tokens UI** | S08, S10, S17, S18, S23, S25
M16 | Travel & Exploration | **Hex crawl/travel route rules adapter** | S04, S16–S19, S22
M17 | Crafting/Strongholds | **Crafting recipes, downtime and domain rules adapter** | S04–S06, S19, S22
M18 | Investigation/Clue Board | **Clue graph/handout workflow** | S02, S09–S11, S20, S23

A module-specific engine may be a plugin/module with a small internal service, but do not globally share UI state, character widgets or renderer objects as if they were domain engines.

## 4. Package and contract strategy

Build a **logical** Catalyst Engine SDK, even if monorepo packaging comes later:

- **Core:** identity, file authority, schema registry, typed errors, event envelopes.
- **RPG Mechanics:** rules packs, formulas/modifiers, choice solver, dice notation/evaluation, turn/combat primitives.
- **Content & Knowledge:** sourcebooks, PDF/extraction, searchable indexes, citation provenance, templates, graphs.
- **World & Spatial:** maps/geometry, calendars, event state, random generators.
- **Presentation adapters:** 3D dice, printable page renderers, local audio, Svelte character/map/GM components.
- **Transport adapters:** Tauri/Rust IPC, local file bridge, later LAN/Android; never import these into pure domain packages.

Suggested neutral interfaces:
- ProjectRepository, CharacterRepository, RulesPackRegistry, RulesEvaluator, ChoiceResolver, DiceEvaluator, RollHistoryRepository.
- DocumentReader, SourcebookRepository, SearchIndex, CitationResolver, TemplateRegistry, AssetRepository, PrintRenderer.
- EventStore, PermissionEvaluator, WorldStateRepository, MapRepository, EncounterResolver, CalendarService, SyncTransport.

Use typed input/output DTOs, semantic versions, capability flags, deterministic error classes, schema validation and consumer-driven contract tests. No module reads another module's private store or directly writes another's file area.

All pure engines should be usable without Tauri, Svelte, browser DOM, cloud, network or database. Integration adapters convert to actual Catalyst project services.

## 5. Rules, dice and results invariants

**Single source for RPG mechanics:** RulesPackRegistry selects the active pack/version; every rule-driven module explicitly receives that context. Calculation services must distinguish source base score, modifiers, prerequisites and manual override. Keep 2014 and 2024 SRD mechanics separate and attribute licensed source packs correctly.

**Dice:** one notation parser and outcome evaluator. It can expose deterministic seeded RNG for tests and secure local randomness for normal rolls. 3D physics must not quietly reroll or change the result without explicit configured behavior. If physics determines the outcome, the result engine must receive and verify it through the same event contract; no double-roll.

**Character:** creation methods (guided/manual) yield the same canonical versioned record. Level-up, combat, inventory and printed sheet are projections/commands on that record.

**GM/player visibility:** deny-by-default for secret content in outbound serialization; player-facing apps never receive GM-only plaintext and rely on CSS to hide it. A standalone client does not bypass permissions.

**Books:** PDF viewing/search is separate from executable imported rule content. Extracted rules enter a review queue and are not activated until validated and explicitly approved. Do not distribute user-imported copyrighted files.

## 6. Engine handoff between two repositories

Repositories:
- Parent: Stormchyld2/Catalyst
- Temporary incubator: Stormchyld2/Catalyst-Character-Builder (app display name: Catalyst Character Studio).

Contract ownership belongs in this document in parent Catalyst. Character Studio maintains an implementation/adapter document pointing here; before integration compare engine schema versions and run tests.

Never copy a full second UI app into Catalyst as the integration method. Extract or vendor **only license-cleared pure engine modules and focused Svelte components** that meet parent service contracts; adapt persistence to the real Catalyst document service. Keep a manifest listing donor source URL, exact commit SHA, license, copied paths, license notices, assets excluded and subsequent modifications. GPL projects may guide UX design but their code is not copied without a conscious GPL licensing decision.

**Integration gates:**
1. Lock interface schemas, portable fixtures and source licenses.
2. Verify pure modules independently in both repositories.
3. Implement adapters to Catalyst's file/document/ID services (not a parallel authoritative store).
4. Compare fixture outputs against the standalone tool, including rules-pack version.
5. Verify import/export roundtrip, rolled results, printing and GM visibility.
6. Land via isolated PR with migration notes and no silent change to old records.

Avoid circular dependencies: Character Studio may consume shared rules/dice/sourcebook contracts; parent Catalyst must never depend on the Character Studio **application** for game-wide rules or file reading.

## 7. Engine dependency order and separate Claude build session

**Stage A — audit and contracts (no duplicate implementation).** Inspect actual source/services and branches; label each Sxx as Existing-working, Partially-implemented, Proposed or Blocked. Record owner, module paths, public functions, license and test evidence. Create contract fixtures before editing code.

**Stage B — P0 core:** S01/S02 existing adapters and integrity tests; S03 schema/template; S04 pack registry; S05 expression/modifier evaluator; S06 choice/progression resolver; S07 shared dice evaluator. Focus on pure logic and regression tests; don't rewrite active 3D dice UI.

**Stage C — usable foundation:** S09 event history, S10 privacy, S11 search, S12/S13 PDF/sourcebook, S14 export, S15 asset service, S16 random table, S28 backup. These support character sheets, the Campaign Wizard, GM references and basic encounters.

**Stage D — maps/world/GM:** S17–S23, S26–S27; enable manual/creative maps, combat, world state, quests and GM console.

**Stage E — multiplayer/polish:** S24, S25, S29 and Android adapters. Do not start LAN transport before redaction/authority and state contracts are tested.

**Small first addendum milestone:** deliver S04–S06 as a standalone, dependency-light rules core, with example System-Neutral pack, verified SRD pack manifest/schema only, and contract tests. Treat actual SRD content import as a separate licensed task. If S07 already exists in Claude's dice branch, adopt/test the existing parser rather than implement another.

Before any build, identify active PRs and paths touched; avoid conflicts with wizard/dice branches. Keep docs/contracts-only changes in their own PR, then one feature branch per coherent engine slice.

## 8. Universal engine definition of done

- Clear API and owner; input/output types and schema versions.
- Pure unit tests and consumer contract tests; no arbitrary executable pack expressions.
- Deterministic fixtures, malformed/corrupt input tests, collision/path traversal and version compatibility.
- Offline verification; no CDN, network, remote licensing call or external account.
- Portable persistence; derived data rebuildable and no second authoritative store.
- Correct attribution/license manifest for third-party code, packs, assets and fonts.
- Robust error codes and logs free from secrets, purchased book contents or private GM notes.
- Validated Windows build and no new unexplained frontend/Rust regressions.
- Document what is genuinely implemented versus planned; successful tests and commit/PR evidence.
- Safe reuse by a second consumer without modifying core business rules or exposing UI-specific state.

## 9. Approved priorities and scope caveat

This registry is a target map of engine boundaries **not an instruction to build all 29 shared engines at once**. For the next engineer/Claude session, prioritize an audit and the smallest reusable core. Many listed engines are future milestones, and some functionality may already exist under inherited names. Identify and adapt those rather than rewriting them.

Architecture is not implementation. Keep current Campaign Wizard and Dice Roller status as independently verified, and keep Character Studio as a separate addendum workstream until contracts are ready.

