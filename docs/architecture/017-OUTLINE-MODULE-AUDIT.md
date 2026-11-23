# Outline Module — Architecture Audit (2026-10-07)

## Decision
Place **Outline** between **Story Dev** and **Write** in the workspace rail. Use a Plottr-inspired timeline as the primary eventual view: chapters/acts horizontally, plotlines vertically, with selectable scene cards. Additional Board and hierarchical Outline views share the same records. Do not require a particular plot structure.

## Existing implementation
- `src/lib/workspaces.ts` and `ModuleRail.svelte`: canonical workspace registration and navigation; Outline is now registered and positioned between Story Dev and Write.
- `src-tauri/src/story_development.rs`: rebuildable, file-authoritative index of project-local Markdown documents and images.
- `StoryDevelopmentWorkspace.svelte`: document tabs, wiki tree, Markdown/metadata editing, and project-local session state.
- `src-tauri/src/write_projection.rs`: derives chapter/scene/beat UI structures from `Manuscript`, `Story Development/Planning & Outline/Master Outline`, `Story Development/Planning & Outline/Beat Sheets`, and legacy `Planning`. Reads `Manuscript/Write Metadata.yaml` for chapter order and scene status overrides. Projection is not a content authority.
- Write sidebar and prose-save guard already protect tested chapter and scene switches.

## Risks / required contracts before visual editing
1. **Stable identity:** Current projection also matches planned and manuscript chapters by normalized title, and generates some scene IDs using chapter/title/position. A drag/drop timeline must not change an entity's identity. Audit and migrate those relationships to persistent IDs before implementing reorder.
2. **Single source of truth:** Outline and Write must reference the same project-local chapter/scene/beat identities; never maintain a second independent outline database or duplicate prose.
3. **Planning vs prose:** Planning metadata, plotline membership, and ordering should be editable without overwriting manuscript bodies. Define which Markdown/YAML documents own each field and preserve legacy reads.
4. **Relationships:** Store references to Story Development entities by stable document ID, with resolvable titles as display labels. Missing/ambiguous references need explicit UI states.
5. **Save safety:** Timeline drag/drop, workspace switching, and scene selection must honor pending prose writes and conflict checks. Never silently discard unsaved text.
6. **Rebuildability:** Any derived timeline index is disposable and can be reconstructed from ordinary project files. Keep each novel isolated within its own folder.

## Implementation phases
1. Define file-backed outline model, stable IDs, migration/backward compatibility, and Rust read/write/reorder commands; add round-trip and reorder tests.
2. Build Timeline workspace: horizontal chapters, vertical plotlines, cards and detail panel; add accessibility and keyboard reordering.
3. Add Board and tree views over identical data; integrate linked Story Development entries.
4. End-to-end test: New Book → Story Dev → Outline → Write → close/reopen → inspect project files.

## Current scope
The navigation entry is registered, but the Outline editor/timeline has **not** been implemented. The shell's existing future-workspace fallback is shown when Outline is selected. Do not describe the module as functional until its file-backed model and UI are implemented and tested.

## Locked Timeline Product Direction

The Outline workspace should match the functional depth of dedicated plotting software while presenting a distinctly Catalyst interface rather than copying another product's visual design.

The primary Outline experience is the **Timeline**:
- chapters / acts across the horizontal axis;
- plotlines across the vertical axis;
- draggable scene cards at their intersections;
- selectable cards opening a detail panel;
- shared chapter, scene, and beat identities with Write;
- linked Story Development entities by stable ID.

Two additional views operate over exactly the same file-backed records:
- **Outline:** hierarchical chapters → scenes → beats.
- **Board:** Kanban-style organization by status, act, plotline, or other supported grouping.

These are views, not separate data stores.

Catalyst should distinguish:
- **Narrative order:** the order in which the reader encounters chapters/scenes.
- **Story chronology:** when events occur in-world.

Story chronology may diverge from narrative order for flashbacks, mysteries, thrillers, parallel timelines, and nonlinear stories.

Reordering any chapter or scene changes position only. Position and display numbering must never participate in permanent identity.
