# Catalyst Architecture Documentation — Conversion Register

**Status:** Documentation triage register · 2026-11-23  
**Purpose:** Separate current Catalyst RPG requirements from donor-era LithoMark/Kindling engineering records without breaking compatibility.

## Editorial policy

1. **Never bulk-replace every occurrence** of `LithoMark` or `Kindling`. Preserve literal file extensions, internal keys, historical project format identifiers, migration names, tests, stable-ID prefixes, and references to the original donor repository.
2. An inherited document that still describes a working implementation is a **legacy technical reference**, not automatically an approved Catalyst feature specification.
3. Prefer writing a **new Catalyst RPG architecture document** that links to reusable donor mechanisms rather than overwriting the source of truth for existing book files.
4. Distinguish **Approved target / Implemented / Legacy supported / Planned / Deprecated** explicitly. Never imply that a proposal has been coded or tested.
5. Current code wins when identifying implementation status; architecture wins when identifying intended product behavior. A deviation must be documented and resolved, not hidden.
6. Documentation changes should use independent branches and PRs. Do not rewrite work on Claude's concurrent dice-roller, wizard, or test-stabilization branches.

## Active authority and sequence

- **New RPG campaigns:** `020-CAMPAIGN-WIZARD-ARCHITECTURE.md` — approved design, not yet proof of implementation.
- **Character Studio:** `021-CHARACTER-STUDIO-ARCHITECTURE.md` — approved guided/manual builder, rules packs, sheets and donor evaluation; not yet proof of implementation.
- **Core storage/stable-ID contracts:** `001-PROJECT-FILE-FORMAT.md`, `002-DOCUMENT-MODEL.md`, `003-STABLE-ID-SYSTEM.md`; read with the RPG exceptions in 020.
- **Canon/continuity:** `004-CANON-MODEL.md`; useful core mechanism, later adapt domain language to campaigns and worlds.
- **Plugins / agents:** `005-PLUGIN-ARCHITECTURE.md` and `006-AGENT-API.md`; inspect implementations before designing RPG extensions.

## Legacy-document disposition

| File | Disposition | Catalyst action |
|---|---|---|
| `000-LITHOMARK-ARCHITECTURE.md` | Legacy donor baseline | Keep intact for provenance; write a new Catalyst RPG root architecture, do not rename old program semantics as already implemented |
| `001-PROJECT-FILE-FORMAT.md` | Active compatibility contract | Preserve original schema/literals; add a documented campaign schema and compatibility adapter |
| `002-DOCUMENT-MODEL.md` | Active shared infrastructure | Reuse file-authoritative documents and stable IDs; add RPG document types via reviewed contract |
| `003-STABLE-ID-SYSTEM.md` | Active shared infrastructure | Preserve IDs on rename/move; extend only through approved type registry |
| `004-CANON-MODEL.md` | Reusable design | Adapt from story canon to GM-approved world/campaign canon and state |
| `005-PLUGIN-ARCHITECTURE.md` | Reusable design | Define versioned rules-pack/generator extensions without weakening permissions |
| `006-AGENT-API.md` | Reusable design | Preserve access boundaries; no unapproved cloud requirement |
| `007-KINDLING-MIGRATION.md` | Historical | Keep donor-specific migration details |
| `008-EDITORIAL-REVIEW-PACKAGE.md` | Legacy writing feature | Retain for compatibility; not a V1 RPG prerequisite |
| `009-LITHOMARK-AUDIOBOOK-STUDIO.md` | Legacy optional feature | Evaluate for later offline atmosphere/TTS; don't confuse audiobook with RPG soundboard |
| `010-LITHOMARK-PROJECT-WORKSPACE-ARCHITECTURE.md` | Reusable workspace mechanics | Reuse tabs/tree/editor/navigation in World Builder |
| `011-LITHOMARK-PUBLISHING-&-EXPORT-STUDIO.md` | Legacy + reusable export | Plan new Print Studio for character sheets, maps, handouts and reports |
| `012-LITHOMARK-WRITER-OS-INTEGRATION-ARCHITECTURE.md` | Historical integration | Keep as donor compatibility reference |
| `013-VSCODE-PLUGIN-ARCHITECTURE-MAPPING.md` | Developer reference | Retain as optional; not product roadmap |
| `014-ECOSYSTEM-REGISTRY-WEBSITE.md` | Deferred | Not a V1 tabletop requirement; no cloud dependency |
| `015-IMPLEMENTATION-STATUS-ROADMAP.md` | Historical checkpoint | Do not treat status of writer features as status of RPG features; establish separate Catalyst tracker |
| `016-NEW-BOOK-FOUNDATION.md` | Legacy book creation | **Superseded by 020 for all new Catalyst RPG campaigns**; keep for existing book projects |
| `017-OUTLINE-MODULE-AUDIT.md` | Reusable design | Adapt graph/outline infrastructure to Adventure Studio |
| `018-SQLITE-RETIREMENT-AUDIT.md` | Active risk register | Continue transitional SQLite retirement without rewriting working code prematurely |
| `018-STORY-DEVELOPMENT-COMPLETION-AUDIT.md` | Reusable implementation audit | Reassess capabilities for World Builder |
| `019-STORY-DEVELOPMENT-PLANNING-CANVAS.md` | Reusable visual planning | Adapt to quests, locations, NPC relationships and planning boards |
| `multi-window-adaptive-display.md` | Reusable design | Apply to GM/private and player-facing displays after security review |
| `020-CAMPAIGN-WIZARD-ARCHITECTURE.md` | **Approved new RPG target** | Implement and validate through tests before declaring complete |
| `021-CHARACTER-STUDIO-ARCHITECTURE.md` | **Approved new RPG target** | Reuse selected licensed donors after audit; implement guided/manual builder and sheet engine |

## Missing dedicated Catalyst architecture documents

Create these as separate, reviewable design milestones:
- Catalyst root product architecture and feature matrix.
- World Builder RPG taxonomy and entity templates beyond initial wizard seeding.
- Adventure Studio / quest and encounter documents and transitions.
- Map Studio manual editor, generators, map schema, asset licenses and VTT interfaces.
- 3D Dice Roller global tray, physics, roll history and GM Console integration.
- Game Master Console, campaign state, private/public visibility and LAN authority.
- Print Studio: player/GM handouts, maps, ink-friendly character sheets.
- Android companion and offline LAN sync, staged after Windows foundation.

Do not number new architecture docs by collision with currently existing document filenames; choose next available number after inspecting the repository.

## Delivery workflow

1. Inventory code and ongoing PRs; never document speculative features as finished.
2. Add target RPG documentation and link it from `AGENTS.md`.
3. Implement in isolated feature branches.
4. Validate behavior and compatibility; record exact tests.
5. Retire legacy *presentation* only after replacement is operating; keep old file format readers as needed.
