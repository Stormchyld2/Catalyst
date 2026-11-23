# 018 — Story Development Completion Audit and Execution Plan

**Status:** Active implementation audit (not a declaration of feature completion)  
**Branch:** `migration/kindling-foundation`  
**Date:** 2026-10-10  
**Scope:** Complete Story Development foundations before finishing Outline and Write; preserve current tested work.

## 1. Reconciled decisions

- Canonical user files remain Markdown/YAML/JSON in ordinary project folders; no required database or mock runtime data.
- Stable prefixed UUID identities are authoritative across Story Development, Outline, Write, timeline, canon, and review. Rename/move/reorder must not change identity.
- Story Development owns story bible/worldbuilding, entities, links, chronology, canon and structured relationships. Outline is a separate top-level workspace between Story Development and Write, consuming the same IDs.
- Worldbuilding is a mandatory New Book Foundation stage. A minimal foundation is sufficient; comprehensive lore is not mandatory before writing.
- The author can enter Write immediately in an existing project; this does not remove the guided new-book setup or its Worldbuilding requirement.
- Interactive maps are deferred to a future optional first-party plugin using public Plugin APIs. No Core cartography implementation in this milestone.
- Character and organization states, relationships, allegiances, membership, rank, location, knowledge, etc. can vary across story chronology. Chronology and manuscript order must remain distinct.
- Core continuity warnings use explicit structured facts, never inferred truth or silent rewriting.
- Keep template families and recently added connected-story architecture; do not introduce a second entity, event, or scene authority.

## 2. Repository evidence reviewed

| Area | Evidence | Audit disposition |
| --- | --- | --- |
| Story Development workspace | `src/lib/components/StoryDevelopmentWorkspace.svelte` | Substantial UI foundation; verify complete behavior end-to-end |
| Frontend regression tests | `src/lib/components/StoryDevelopmentWorkspace.test.ts` | Tests for file-backed Wiki entries, links, images, tabs, editor, preview, metadata, saves, and dirty navigation |
| Rust story service | `src-tauri/src/story_development.rs` | Index, metadata/body updates, asset handling and focused tests exist |
| Tauri commands | `src-tauri/src/commands/story_development.rs` | Document, metadata, asset and recovery command layer exists |
| Template registry | `src-tauri/src/entity_templates.rs` | Built-in/custom entity template manifest and persistence support exist |
| Link resolver | `src-tauri/src/link_resolver.rs` | Project link index, resolution and backlink service exist |
| Markdown renderer | `src/lib/utils/markdownPreview.ts` | Markdown preview and wiki-link rendering exist |
| Outline backend | `src-tauri/src/outline.rs` | Preserve canonical scene identity and existing tests |
| Write projection | `src-tauri/src/write_projection.rs` | Preserve canonical scene identity and prose round trips |
| Architecture | `000`, `004`, `005`, `012`, `015`, `016`, `017` | Reconcile, do not replace existing contracts |

**Evidence boundary:** Presence of implementation/tests is not proof of all acceptance criteria or a fresh green test run. This audit inspected repository source and existing tests; no local build or test suite was executed during this audit.

## 3. Completion gates

### Gate A — Finish and harden Story Development essentials (first)

- Verify New Book creates required Worldbuilding and correct canonical Story Development files, then opens the Wiki; no duplicate project or manuscript authority.
- Verify real project create/read/update/rename/move/delete flows through authoritative Document Service with stale-save and invalid-file safeguards.
- Confirm template defaults and per-document assignment are persisted and retain unknown metadata. Test built-in and custom templates without mock runtime data.
- Verify navigation tree, top tabs, split/preview, right context/overview, local images, and resolved/ambiguous/unresolved wikilinks against actual project files.
- Verify backlinks and links survive rename/move by stable ID; distinguish discovered mentions from asserted canon.
- Preserve unsaved changes and recovery behavior through navigation and project switching.
- Close UI and backend test gaps for the above before proceeding.

### Gate B — Shared entity and canon contracts (second)

- Confirm typed entity records for characters, locations, organizations, objects, events, and writer-defined codex categories, reusing existing templates.
- Define explicit relationship identity, directionality, type, source/target, and optional validity intervals; separate explicit relationships from derived wikilinks/backlinks.
- Record event participation, event effects, and canonical temporal facts without creating another graph database.
- Define time-aware character and organization state: membership, rank, allegiance, status, knowledge, location and relationship changes.
- Provide deterministic point-in-time reconstruction from explicit facts, with unknown/undetermined states when data is missing.
- Keep story chronology separate from narrative/chapter order and from file revision history.
- Detect contradictory intervals, invalid references, and incompatible event effects as **warnings**; never silently rewrite writer content.
- Add Rust round-trip, conflict, invalid-ID, chronology, rename/move, and transition tests.

### Gate C — Workspace integration (third)

- Expose shared entity/relationship/timeline reads through documented services usable by Story Development, Outline and Write.
- Outline scenes and beats reference the existing canonical chapter/scene IDs and Story Development entities.
- Write context panels consume the same metadata and links; no duplicate prose, state or chapter trees.
- Confirm rename/reorder of Outline scenes does not overwrite manuscript prose or change scene IDs.
- Test representative flow: New Book → Worldbuilding/characters/locations → Story Development wiki → Outline scene planning → Write prose → return to story context.
- Confirm disabling optional plugins, including future maps, never blocks any step above.

### Gate D — Defer without blocking

- Visual relationship/organization/family-tree projections can follow stable relationship data and continuity APIs.
- Advanced Continuity Inspector UI can follow deterministic canon-state services.
- Interactive mapping belongs to a future optional plugin and is **out of scope**.
- Full Research enhancements, multi-monitor, reviewer dashboard, and other later work should not block this milestone.

## 4. Recommended implementation sequence

1. Inspect New Book/Story Development real-file integration and add missing regression tests.
2. Complete the existing Wiki/template/link/asset/dirty-save UX to Gate A acceptance.
3. Extend the existing Canon/Timeline model for relationship/event/state history with test-first services.
4. Add compact state-at-time and organization-membership views, using shared APIs.
5. Verify Outline/Write cross-workspace integration; then resume Outline completion followed by Write completion.

**Stop rule:** Do not expand into map editing, full graph rendering, unrelated publishing features, or a new persistence architecture while closing these gates.

## 5. Current outcome

- **Confirmed present:** substantial Story Development UI/backend, template registry, link resolver, preview, and focused tests.
- **Needs verification/completion:** end-to-end project setup, lifecycle operations, richer relationship/event-state data, deterministic point-in-time reconstruction, and Outline/Write integration.
- **Not claimed:** tests newly passed, advanced state engine implemented, or Story Development fully complete.

This audit is the active checkpoint for the next Story Development implementation pass.

## 6. Gate A verification checkpoint — 2026-10-11

**Disposition: Gate A focused implementation and regression requirements verified; proceed to Gate B.** This is not a claim of full graphical end-to-end coverage or complete Story Development functionality.

Verified on the active branch using user-reported local test runs:

- New Book Foundation → canonical Wiki index, required Worldbuilding, distinct stable IDs, and disk reopen after saving prose: `cargo test new_book_foundation::tests -j 1` — **9 passed**.
- File-authoritative Document Service, stale saves, rename/move/delete/trash, and guarded recovery integrity: `cargo test document::service::tests -j 1` — **23 passed**.
- Story Development workspace file-backed Wiki, navigation, tabs, links, previews, images, and unsaved-change behavior: focused Vitest run — **32 passed**; `npm run check` — **0 errors, 0 warnings**.
- Recovery Manager frontend confirmation, guarded hash handoff, and failed-save protection: focused Vitest run — **3 passed**; `npm run check` — **0 errors, 0 warnings**.
- Built-in/custom template assignment and metadata preservation: Story Development Rust suite — **16 passed**.
- Stable-ID link resolution and backlinks after rename/move: link resolver Rust suite — **6 passed**.
- New Book frontend activation and failure handling: focused Vitest run — **8 passed**.

**Evidence boundary:** These are separately executed focused tests, not a single full-suite run. The New Book disk-reopen regression proves authoritative file persistence and Wiki reindexing; it does not simulate a complete graphical project-switching session. Manual GUI end-to-end project switching and recovery remains a separate validation item. Do not treat it as passed or silently waive it.

**Gate B entry:** Audit existing canon, entity, and chronology services against architecture documents before changing code. Reuse canonical prefixed UUIDs, explicit writer-authored facts, and ordinary project files. Do not create a second entity store or infer canon from prose. Gate D visual mapping and optional maps remain deferred.


## 7. Gate B implementation audit — 2026-10-13

**Disposition: Gate B remains OPEN.** Focused user-run tests for four newly added backend checkpoints passed (14/14 total); these do not establish completion of the full Gate B contract.

### Verified focused checkpoints

1. `canon_contract::tests`: 3/3 passed — explicit fact and relationship shape, prefixed UUID v4 IDs, and ISO date interval validation.
2. `canon_store::tests`: 4/4 passed — Markdown/YAML Document Service persistence, reopen, optimistic hash protection, duplicate-ID rejection within each canonical folder, and invalid-file preservation.
3. `canon_chronology::tests`: 3/3 passed — deterministic point-in-time projection of confirmed facts and dated relationships; planned facts excluded and overlapping facts retained.
4. `canon_continuity::tests`: 4/4 passed — overlapping conflicting confirmed values for selected exclusive fact types, optional entity-index missing-reference warnings, and avoidance of a knowledge false positive.

These tests were reported by the user on the active branch. No fresh full-suite, GUI end-to-end, or cross-module integration test is claimed here.

### Remaining Gate B requirements and constraints

- **Canonical typed entity integration:** Existing entity templates and Story Development file-backed entity records are present, but the new Canon services do not yet resolve their IDs against the actual shared entity index. Missing-reference detection currently requires a caller-supplied index and does not automatically validate relationship subjects.
- **Events:** No dedicated explicit event participation/effects contract, persistence, historical transition application, or incompatible-effect diagnostics has been established by the four checkpoints. Reuse existing event document identity; do not create a competing event authority.
- **Temporal scope:** Current validity intervals support strict ISO calendar dates only. The Canon architecture also contemplates relative/approximate, event/scene-linked, and more precise temporal references. Decide compatible evolution before treating the date-only model as final; never infer story dates from manuscript position.
- **Historical state:** Existing projections show dated confirmed facts and relationships, but do not yet establish event-driven transitions, complete organization/character state semantics, or unknown-vs-inapplicable distinctions for every field.
- **Continuity:** Overlap checks are intentionally narrow. Event-effect conflicts, missing canonical references from actual files, and scene-time checks remain unimplemented. Diagnostics must stay advisory, with stable rule IDs and no silent rewriting.
- **Storage integrity:** The Canon store uses Document Service and checks duplicates within its designated fact/relationship directories. Cross-project identity indexing, unknown metadata preservation on record updates, and cross-module rename/move tests remain to be verified.
- **Acceptance tests:** Add event round-trip, transition, invalid reference, rename/move, chronological reconstruction, and cross-module integration tests before declaring Gate B complete.

### Recommended bounded closure sequence

1. Integrate read-only Canon reference checks with the existing Story Development entity index, without new entity storage.
2. Add minimal explicit event participation/effects records linked to existing event IDs, with guarded persistence and focused transition tests.
3. Expand continuity diagnostics only for deterministic contradictions supported by those explicit event/state records.
4. Verify rename/move identity, metadata preservation, and temporal semantics with focused tests. Record remaining advanced temporal features explicitly if deferred.
5. Close Gate B only after the relevant tests pass; then proceed to Gate C Story Development → Outline → Write integration.

**Stop rule remains active:** no relationship graph UI, interactive maps, new event database, or advanced inspector before shared core workflow integration.
