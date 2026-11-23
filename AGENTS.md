# Catalyst — Active Agent Instructions

> **Catalyst is an offline-first tabletop RPG Campaign & Adventure Studio.** The following new-project instructions supersede inherited LithoMark writer/book instructions in this file. Inherited writer architecture remains historical/compatibility reference only; do not silently rebrand or mutate existing book projects.

## Active RPG architecture hierarchy

1. **`docs/architecture/020-CAMPAIGN-WIZARD-ARCHITECTURE.md` is authoritative for all NEW Catalyst tabletop campaigns.** It governs Quick Start, the ten-step Guided Setup, approved tabletop folder hierarchy, `campaign.yaml`, worldbuilding templates, GM starter library, creation safety and tests.
2. **`docs/architecture/021-CHARACTER-STUDIO-ARCHITECTURE.md` is authoritative for the approved Guided/Manual Character Builder, system-independent sheets, advancement, rules packs, open-source donor audit and printing.** Do not implement a D&D-only character record or copy GPL-licensed donor code without explicit licensing approval.
3. **`docs/architecture/022-RPG-SOURCEBOOK-LIBRARY-ARCHITECTURE.md` governs the shared offline PDF sourcebook library, readable purchased/homebrew books, annotations, search, reviewed rules extraction and cross-module services.** Importing a PDF must not automatically activate its content as rules or authorize redistribution.
4. **`docs/architecture/023-CATALYST-SHARED-AND-MODULE-ENGINES.md` governs reusable cross-workspace engines, module-specific engines, ownership, contracts, dependency order and integration with standalone Catalyst Character Studio.** Audit existing implementations before starting an engine build; do not duplicate the dice parser, rules evaluator, file service, search or renderer.
5. **`docs/architecture/024-ENGINE-IMPLEMENTATION-OWNERSHIP-MATRIX.md` is the complete 59-engine assignment register** (S01–S29, M01–M18, C01–C12). **Character Studio owns M01–M04, C01–C12 and incubates pure S04–S06; main Catalyst owns the remaining implementation, contract adapters, global dice, PDF and sourcebook services.** Audit source code before declaring an engine implemented or asking Claude to build it.
6. **`docs/architecture/025-CATALYST-DESIGN-SYSTEM.md` is authoritative for all Catalyst UI branding and colors.** Global accent is **Frost Blue `#6EA9CF`**, not antique gold; use versioned semantic tokens shared with Character Studio and preserve legacy styling only during staged migration.
7. Existing `016-NEW-BOOK-FOUNDATION.md` and `001-PROJECT-FILE-FORMAT.md` continue to govern **legacy book compatibility** and inherited project service constraints until explicitly migrated. Do not use their novel steps or manuscript folders as defaults for new campaigns.
8. Preserve stable ID/document service/file-authority principles from `002-DOCUMENT-MODEL.md`, `003-STABLE-ID-SYSTEM.md`, and `004-CANON-MODEL.md`. No new mandatory SQLite authority or cloud dependency.
9. The approved roadmap separates **Campaign Wizard**, **World Builder**, **Adventure Studio**, **Character Studio**, **Map Studio**, **GM Console**, and **global Dice Roller**. Unfinished modules are not allowed to block valid campaign creation.
10. For git, use Catalyst's active feature branch and dedicated PR; do not assume the inherited LithoMark `migration/kindling-foundation` branch exists in Catalyst. Do not merge to `main` without user approval. Coordinate with ongoing Claude feature branches and do not edit their code as part of documentation cleanup.

**When inherited statements below conflict with the above for new RPG campaigns, the above controls.** Historical references to LithoMark/Kindling are intentionally retained where they describe disk formats, migration, provenance, or still-working writer-era services.

---

# LithoMark Writer OS — Agent Instructions

These instructions apply to the entire repository. Read them before modifying code.

## Mission

LithoMark is a local-first Writer OS. It combines manuscript writing, structured story development, linked knowledge, review workflows, publishing/export, audio/TTS capabilities, Git integration, templates, and an extensible plugin architecture without making the application itself the only way a writer can access their work.

Preserving user ownership, portability, durable identity, and architectural boundaries is more important than implementing a feature quickly.

## Non-Negotiable Storage Rules

1. Ordinary files and folders are authoritative for user content.
2. Do not introduce a proprietary project container.
3. Do not make a database authoritative for user content.
4. Prefer Markdown, YAML, JSON, and normal local folders wherever practical.
5. Any cache, index, search database, or projection must be disposable and fully rebuildable from authoritative project files.
6. SQLite may be used only for rebuildable infrastructure unless an architecture document explicitly states otherwise.
7. A LithoMark project must remain understandable and recoverable without LithoMark.
8. Projects/books are isolated. Never resolve ordinary project data across project boundaries.
9. Runtime features must use actual project files. Do not add mock, demo, placeholder, or hard-coded project data as a production fallback.
10. Synthetic fixtures are allowed in tests, QA, and explicit demo harnesses only.

## Core Architecture

Target dependency direction:

```text
LithoMark UI
    -> LithoMark Domain API
        -> Domain Models / Domain Services
            -> Project Services
                -> Authoritative Project Files
                    -> Rebuildable Infrastructure
```

Respect this direction.

- UI code must not become a second persistence layer.
- UI components should call the appropriate command/domain/project service rather than manipulating authoritative files independently.
- Plugins and external agents must use public, permission-controlled APIs.
- Search indexes, backlinks, caches, projections, and derived views never become content authority.
- Do not create competing stores for manuscript, entity, canon, timeline, relationship, review, or link data.

## Stable Identity

Stable IDs are durable identity.

- Paths are not identity.
- Filenames are not identity.
- Titles/display names are not identity.
- Moves and renames must preserve stable IDs and references.
- Structured references and machine-facing APIs should use stable IDs.
- Detect duplicate IDs; never silently accept or regenerate around them.
- Core IDs use the documented type prefix plus canonical UUID v4.
- Do not invent a second ID scheme without an approved architecture change.

Read `docs/architecture/003-STABLE-ID-SYSTEM.md` before changing identity behavior.

## Documents and Links

Markdown/YAML documents remain authoritative.

Wikilink contract:

- Human-authored forms include `[[Display Name]]` and `[[Display Name|Custom Label]]`.
- Human-readable wikilink text is presentation/lookup text, not durable identity.
- Exact stable ID resolution takes precedence when applicable.
- Human-readable resolution uses documented title/name/alias rules within the active project.
- Multiple matches are ambiguous. Surface candidates and require writer selection; never guess.
- Missing targets remain visibly unresolved.
- Rename/move does not change target identity.
- Backlinks are derived from authoritative files and are rebuildable.
- Do not silently rewrite authored Markdown merely because a link was resolved.
- The shared project link resolver is the authority for link resolution. Do not implement separate resolver logic in Write, Story Development, Review, Search, or Context Panel.

Read `docs/architecture/002-DOCUMENT-MODEL.md` and `docs/architecture/010-LITHOMARK-PROJECT-WORKSPACE-ARCHITECTURE.md` before changing link behavior.

## Canon, Timeline, and Continuity

Canon, timeline, and continuity are connected but semantically distinct.

- Canon represents explicit writer-recorded facts. Do not make generative inference authoritative canon.
- Timeline represents temporal/event structure.
- Continuity evaluates explicit structured/project data.
- Storing these under the unified Story Development hierarchy does not erase their semantic boundaries.
- Never silently alter writer canon to repair a continuity conflict.

Read `docs/architecture/004-CANON-MODEL.md` before changing canon behavior.

## Canonical Project Organization

New content uses the canonical project hierarchy documented in `docs/architecture/001-PROJECT-FILE-FORMAT.md`.

Top-level conceptual roots include:

- `Manuscript/`
- `Story Development/`
- `Notes/`
- `Templates/`
- `Assets/`

Story Development contains the unified planning/wiki domain, including Foundation, Planning & Outline, Characters, Worldbuilding & Settings, Timeline, Research, and Canon as documented.

Legacy Kindling and early-LithoMark roots may remain readable for migration compatibility. Do not silently move, delete, or rewrite writer files. Migration must be explicit and non-destructive.

## Manuscript Structure

Manuscript Structure is a dedicated service/domain concern.

- Chapters and scenes use stable IDs.
- Chapter scene order is explicit.
- Scene parent relationships are explicit.
- Reorder/move operations preserve IDs.
- Integrity problems such as missing, duplicate, orphaned, or mismatched records must be reported rather than silently repaired.
- Mutations must flow through the authoritative document/project service.
- Manuscript Structure must not be conflated with a plotting methodology.

## New Book Foundation

The guided sequence is:

1. Project Basics
2. Characters
3. Locations
4. Worldbuilding
5. Style & Voice
6. Themes / Core Ideas
7. Brainstorming
8. Synopsis
9. Manuscript Structure
10. Review & Create

The Foundation workflow is a temporary project seeder. After creation, the ordinary files it creates are authoritative.

Do not force a plotting methodology during Manuscript Structure. Beat sheets, master outlines, and plotting methodologies remain optional Story Development workflows.

Read `docs/architecture/016-NEW-BOOK-FOUNDATION.md` before changing this workflow.

## Story Development

Story Development is one domain with multiple views/services, not multiple competing persistence systems.

- Story Bible/wiki, planning/outline, timeline, research, worldbuilding, characters, and canon can have specialized UX while sharing the same file-authoritative foundation.
- Story Development projections/indexes are rebuildable.
- Invalid project files should be surfaced as issues without hiding valid files.
- Shared assets may be read where documented, but assets do not automatically become independent domain authorities.

## Legacy Kindling Boundary

Legacy Kindling code is migration/compatibility input, not automatically target LithoMark architecture.

In particular:

- Legacy SQLite reference tables are not the LithoMark link graph authority.
- Legacy scene-reference raw UUID records are not the new stable-ID relationship model.
- Legacy cross-project reference-copy commands are not the target persistence model.
- Do not revive or extend these systems as new LithoMark infrastructure merely because code already exists.

When porting Kindling behavior, preserve useful behavior only after translating it through current LithoMark domain boundaries.

Read `docs/architecture/012-LITHOMARK-WRITER-OS-INTEGRATION-ARCHITECTURE.md` before significant Kindling migrations.

## Plugins

LithoMark supports Core, first-party plugins, and future community plugins.

- First-party status does not bypass permissions.
- Plugins use the public Plugin API and declared permissions.
- Plugin code must not bypass Domain Services to create an alternate persistence authority.
- Plugin-defined document types must integrate with the documented stable identity system.
- TTS Reader is Core; Audiobook Studio is an optional bundled first-party plugin consuming Core TTS/audio APIs.

## Review and Reviewer Workflows

Review, beta reading, and ARC workflows must preserve manuscript/project authority and integrity.

A future free reviewer/viewer is expected so reviewers do not need the full LithoMark application. Do not architect review data so that only the full desktop application can interpret or recover it.

## Context Panel

The Context Panel is scene-aware first and chapter-aware second.

- It consumes shared project services.
- It may keep transient UI/navigation state.
- It must not persist competing entity, canon, timeline, manuscript, backlink, or link-resolution data.
- Whole-project exploration belongs primarily in Story Development.
- Wikilink drill-in may present linked context without changing durable identity or creating duplicate content.

## Git and Repository Safety

The normal development branch is `migration/kindling-foundation` unless the user explicitly specifies another branch.

- Never merge to `main` without explicit user approval.
- Never force-push.
- Never rewrite published history.
- Never delete branches or tags unless explicitly requested.
- Do not discard unrelated working-tree changes.
- Inspect `git status` before broad edits.
- Keep commits focused and descriptive.
- Do not commit generated build output, temporary files, secrets, credentials, or local environment state.
- Before a risky or broad refactor, inspect the relevant architecture and current implementation first.

## Implementation Discipline

Before coding:

1. Inspect the relevant existing implementation.
2. Read the architecture documents governing the subsystem.
3. Identify the current domain/service boundary.
4. Preserve established behavior unless the task explicitly changes it.
5. Prefer the smallest coherent change that satisfies the requirement.

While coding:

- Do not duplicate an existing service just to make a UI feature easier.
- Do not bypass validation or integrity checks.
- Do not add silent fallback behavior that masks corrupt/invalid project data.
- Do not weaken type safety without a concrete reason.
- Preserve unknown YAML/frontmatter fields unless the documented operation explicitly removes them.
- Preserve Markdown-significant whitespace and existing file semantics.
- Protect against path traversal and unsafe project-relative paths.
- Treat Windows behavior as first-class; LithoMark is actively developed and tested on Windows 11.

## Testing

Every behavioral change should have appropriate tests.

For Rust changes, run the narrowest relevant tests first, then broader checks when warranted, such as:

```powershell
cargo test <relevant-module-or-test>
cargo check
```

Run commands from the correct crate directory when required.

For Svelte/TypeScript changes, run the focused Vitest file first, for example:

```powershell
npx vitest run <test-file>
```

Then broaden the test scope when the change warrants it.

Rules:

- Diagnose a failing test; do not simply weaken or delete it.
- Do not change production behavior solely to satisfy a broken test assertion.
- Distinguish infrastructure/worker startup failures from actual test failures.
- Add regression tests for bugs.
- Test ambiguous, unresolved, invalid, duplicate, and project-isolation cases where relevant.
- Do not claim tests passed unless they were actually executed and passed.
- Report commands run and their results.

## Architectural Changes

Do not make an architectural change casually.

If a task appears to require violating these instructions or an architecture document:

1. Stop before implementing the conflicting design.
2. Identify the conflict clearly.
3. Propose the smallest architecture change necessary.
4. Wait for explicit approval before changing the governing architecture.

Architecture documents are design authority, but implementation reality matters. If documentation and code disagree, investigate the discrepancy rather than silently choosing one.

## Agent Handoff

Work may alternate between Codex, ChatGPT/GitHub-assisted development, and the user.

At the end of a meaningful task:

- Summarize files changed.
- Summarize behavior changed.
- Report tests/checks actually run.
- Report remaining failures or uncertainties.
- State whether changes are committed or only present in the working tree.
- Keep the repository in a state another developer/agent can inspect and continue.

Do not assume another agent remembers prior conversation context. Leave important durable decisions in code, tests, or architecture documentation.

## Current Product Boundary

The initial public/core preview centers on:

- Write
- Story Development

Other subsystems may have foundations in place, but do not expand scope merely because they are planned.

## Governing Principle

When implementation convenience conflicts with writer ownership, portability, stable identity, explicit integrity, or the documented domain boundary, preserve the architecture and surface the conflict.
