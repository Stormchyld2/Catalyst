# 019 — Story Development Planning Canvas

**Status:** Locked product/design specification; implementation deferred
**Date:** 2026-10-11
**Parent:** 000-LITHOMARK-ARCHITECTURE.md
**Related:** 002 Document Model; 003 Stable IDs; 018 Story Development Completion Audit

## Purpose and boundary

Planning Canvas is an optional, freeform visual brainstorming and memory aid within the **Story Development** workspace, not a separate top-level workspace and not an authority for canon, timelines, relationships, or manuscript prose. Its cards, labels, arrows, groups, and notes are **not canon**, even when they depict existing characters or locations. A visual connection never automatically writes a canonical relationship or event. Authors must explicitly edit/confirm an authoritative Story Development document or canon record to promote an idea.

Implementation follows Gate A lifecycle/recovery completion and the relevant Gate B shared entity/relationship contracts. This specification does not change the gate ordering or authorize a premature graph editor or interactive cartography.

## Entry point and navigation

Add a **New Canvas** button/icon to the **bottom creation toolbar of the Story Development left navigation**, directly alongside the existing **New File** and **New Folder** buttons (and existing Templates control). It is not a new top-level rail workspace. The action prompts for a canvas name and destination folder, then creates a named project-local canvas. On successful creation, the canvas **immediately appears in the selected folder of the left Story Development navigation tree** and **automatically opens as the active new tab in the existing top tab bar**, showing its blank planning surface. A canvas stays in the tree after its tab is closed; clicking it in the tree reopens or activates its tab without duplicates. Existing open tabs remain intact. Creation and navigation honor save-before-navigation safeguards: an unsaved document must not be silently discarded. Canvas tabs use the existing close/reopen and session-restoration conventions, while preserving canvas IDs across rename/move. The new button is implemented when the canvas feature is built; do not expose a nonfunctional button before then.

Support multiple canvases per novel, with names such as Character Ideas, Plot Brainstorming, and Worldbuilding. The canvas opens as a zoomable/pannable freeform surface with draggable cards, connectors, grouping, fit-to-view, and local persistence.

## Cards and presentation

Use the reference of an **expanded, miniature Wiki page**, not a literal reproduction of Chronicler's Saltstead card. Design in LithoMark's dark theme with gold/blue accents, a readable title and optional subtitle, prominent configurable artwork, a compact type/category treatment, selected structured fields, and clickable linked metadata. Avoid unnecessary oversized empty image space.

Provide three per-card presentation modes:

- **Compact:** thumbnail/icon plus title for dense boards.
- **Standard (default):** title, image, and a few selected fields.
- **Expanded:** miniature Wiki page with richer metadata, images, and links; can also be opened in a larger popover/side inspector without permanently resizing the canvas.

Double-click or expand to inspect details. Open a linked Wiki document from its card. The author can configure displayed fields per card instance; two cards referencing the same document may show different fields or sizes.

Initial built-in card templates: Character, Location, Organization/Faction, Event, Scene/Plot Beat, Map Image, Image Reference, Freeform Note, Question, and Plot Thread. Reuse existing Story Development entity/template definitions for linked document cards rather than introducing another entity schema or template authority. Map Image is a resizable ordinary image card, **not** the deferred interactive maps plugin.

## Linked versus independent cards

- **Linked cards** reference an existing document by its canonical stable ID. Displayed title, selected metadata, and asset references are read from the authoritative document; moves/renames do not change identity. Missing or ambiguous references are surfaced, not silently reassigned. The canvas does not duplicate authoritative document content.
- **Independent cards** contain canvas-only brainstorming notes, provisional concepts, questions, labels, and visual references. They do not automatically create Wiki files or canon records. An explicit **Create Story Development Document** action may convert an idea into a real file using an existing template, with preview and user confirmation; the resulting linked card uses the newly assigned stable ID.
- **Connectors** are canvas-only visual annotations, optionally labeled and directed. They do not create canonical relationship records. Explicit canonical relationships, when implemented, use the shared Gate B services independently of these arrows.

## File authority and safety

Each canvas is a normal, portable project-owned **JSON file** in the Story Development hierarchy, with a stable canvas ID, schema version, cards, geometry, groups, connectors, display settings, and canvas-only annotations. References to linked entities use canonical stable IDs rather than paths. Actual Wiki content remains in its Markdown/YAML files; assets remain ordinary project files. Canvas persistence follows the same project-scoped service boundaries, optimistic conflict protection, and recovery safeguards as other author-owned content. No database or external service is required.

Do not infer canon from proximity, arrows, labels, brainstorming notes, or card layouts. Do not rewrite underlying entities when editing the canvas. Broken references remain visible. Deleting a canvas must not delete linked documents. Deleting a linked Wiki document must not silently delete canvas notes.

## Delivery and tests

Build after current Story Development Gate A safeguards and shared contracts, without blocking Outline or Write. Validate JSON round trips, ID preservation across document rename/move, missing references, non-canon connector behavior, independent-card conversion confirmation, local asset rendering, multiple canvases, and safe save/recovery. Keep UI tests synthetic and do not ship mock story content.
