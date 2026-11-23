# LithoMark — Multi-Window & Adaptive Display Architecture

**Status:** Approved product direction; implementation pending  
**Date:** 2026-10-06  
**Scope:** Core desktop application (Tauri + Svelte)  
**Applies to:** Write, Story Development, Planning & Outline, Review, and other eligible workspaces

## 1. Purpose

LithoMark supports authors using one or several monitors, including landscape, portrait, and mixed-orientation arrangements. The goal is a coordinated, flexible writing environment—not separate application instances or copies of the book.

This specification records agreed product behavior and proposed implementation boundaries. It does **not** imply these features are already implemented.

## 2. Locked product decisions

1. **One authoritative project folder per book.** All windows displaying the same book use its normal local files, stable document IDs, and existing file-authoritative services. No proprietary project container, mandatory content database, or cloud runtime dependency.
2. **Chapter-based writing.** Manuscript chapters remain separate Markdown files, opened as individual editor tabs. Do not introduce a combined full-manuscript editing surface.
3. **Continuous editor only.** Write scrolls continuously *within the current chapter*. Do not add simulated print pages, artificial page boundaries, or a book-page editing mode.
4. **Formatting belongs to Publish.** Page dimensions, margins, pagination, print layout, and export previews belong to Publishing & Export Studio, not Write. Editor appearance settings must not change stored manuscript formatting.
5. **Both automatic and manual multi-window workflows.** Authors may assign workspaces to displays through saved layouts, and may detach individual tabs, documents, panels, or whole workspaces into native windows.
6. **Orientation-aware per window.** Portrait, landscape, and mixed-orientation displays are supported. Windows reflow independently based on their usable dimensions; monitor rotation must not alter content.
7. **Single-monitor parity.** All writing functions work with one display. Multi-window features are optional and must degrade gracefully if a display disappears.
8. **Project isolation.** Multiple windows must never mix content, metadata, state, or save targets between separate novels.

## 3. User experience

### 3.1 Automatic layout assignment

The Display Manager discovers available displays and usable work areas. Authors can choose a layout preset or restore a previously saved arrangement, assigning Write, Story Development, Planning, Review, and other eligible workspaces to particular displays.

Examples:
- **Writing:** portrait display → Write; landscape display → Story Development and outline.
- **Revision:** one window → current chapter; another → preceding chapter or Review.
- **Single screen:** standard tabs and docked panels; no second monitor required.

Display detection should not silently relocate an author's windows on every launch. Provide a preference for restoring the last layout and a straightforward reset-to-primary-display action.

### 3.2 Manual window detachment

Allow the following actions as the feature matures:
- Open a workspace or document tab in a new native window.
- Move a window to a selected connected display.
- Drag or otherwise move a detached window between monitors using normal operating-system window behavior.
- Return a detached tab/panel to the main window.
- Display different chapters concurrently, with each chapter remaining a separate file.

The implementation may introduce a menu-based **Open in New Window** command before full drag-out docking. Do not assume OS-level window dragging automatically implements tab-to-window detachment.

### 3.3 Linked and independent reference behavior

Reference windows support:
- **Follow:** clicking a resolved wiki/entity link in Write updates a designated reference window without replacing the manuscript tab.
- **Independent:** navigation in one window does not change another.
- **Pinned:** a reference window stays on a chosen document until explicitly changed.

A link with ambiguous or unresolved targets continues to use existing resolver behavior; multi-window routing must not bypass resolution or invent matches. Follow behavior is opt-in and scoped to the correct project.

### 3.4 Portrait writing

When a monitor is rotated so that it is taller than it is wide, Write should adapt to the window's **usable width**, not rely only on a global monitor-orientation flag. Provide:
- Centered, adjustable manuscript column and editor font size.
- Collapsible navigation, toolbar, and reference panels.
- Optional focus presentation and typewriter scrolling.
- Continuous scrolling within one chapter.
- Preservation of unsaved edits, selection/cursor, and scroll position through resize, rotation, and window moves.

A physically clockwise-rotated display is supported regardless of which orientation label the operating system reports. Do not rotate manuscript text inside the app.

## 4. Technical architecture (proposed)

### Display Manager
Uses Tauri-supported native display/window APIs to enumerate displays, work areas, scale factors, and window geometry. Handle OS display changes, DPI changes, window moves, and disconnected monitors. Verify API support and event behavior against the installed Tauri version during implementation.

### Window Manager
Owns unique native window IDs, lifecycle, placement, restoration, and close behavior. Maintains the association between each window, active project, workspace, and open document IDs. Window identifiers are UI instances, not document identities.

### Workspace/Tab Manager
Separates view state (active tab, panel layout, scroll position, focus mode) from durable content. Supports moving a tab or workspace to a different window without creating a new manuscript file. Prevent duplicate registrations and stale event subscriptions.

### Document Coordinator
Coordinates reads, edits, saves, and file-change notifications across windows. Reuse existing file-authoritative document services, canonical IDs, safe paths, staged/atomic writes, and conflict checks. An in-memory edit buffer is **not** a second authoritative copy of a book.

**Critical rule:** Two windows editing the same file must never silently overwrite each other. Before enabling simultaneous editable views, define either a single editable owner with mirrored read-only views, or a tested shared edit session with revision/version checks. On external changes or conflicting revisions, surface a recoverable conflict instead of last-writer-wins. Do not rely solely on filesystem watchers to prevent lost updates.

### Layout Persistence
Persist window-layout preferences as ordinary JSON, separate from manuscript content. Support global presets and optional project-specific layouts. Persist logical display associations, window geometry, view IDs, and per-window view preferences; do not persist manuscript text in layout files. Restore only after validating available monitors, bounds, scale factors, and project access. Missing displays must relocate windows into a visible work area.

## 5. Project boundaries and lifecycle

- Every window must carry an explicit project identity/root context; commands may not infer a project from whichever window happened to be active.
- Switching or closing a project must not redirect another project's windows or writes.
- Closing a secondary window must not automatically close the book or discard unsaved changes.
- Closing the last project window follows normal save/unsaved-change safeguards.
- Reopening a layout must not create duplicate files or mutate the folder skeleton.
- No native AI integration is required by this system.

## 6. Phased implementation

**Phase A — Foundation:** document/window IDs, explicit project context, window registry, one-secondary-window prototype, safe close behavior, tests.

**Phase B — Detachable workspaces:** open/move/restore workspace windows; project-scoped routing; monitor loss fallback.

**Phase C — Detachable document tabs:** chapter and wiki tabs, independent navigation, safe edit ownership, coordinated change notifications.

**Phase D — Adaptive displays:** portrait-responsive editor, collapsible panels, resize/rotation handling, independent view preferences.

**Phase E — Layout presets and linked navigation:** saved display assignments, follow/independent/pinned reference modes, robust restoration.

The order may be adjusted for dependencies, but multi-window editing must not ship before safe save coordination is tested.

## 7. Verification and acceptance criteria

- One project can be viewed on multiple monitors without duplicated manuscript data.
- Two separate chapter files can be edited in different windows without cross-document writes.
- Two views of the *same* chapter cannot silently lose edits, including after an external file modification.
- Switching between portrait and landscape preserves active chapter, unsaved text, cursor, and scroll position.
- Disconnecting a secondary display leaves every window reachable.
- Restoring layouts on different DPI/scaling arrangements keeps windows visible.
- Linked entity navigation targets the correct project's reference window and honors pinned/independent mode.
- Layout changes do not modify Markdown, YAML frontmatter, chapter ordering, or publishing settings.
- All core writing workflows remain usable on a single monitor.
- Windows-specific tests are required initially; design must not unnecessarily preclude future desktop platforms.

## 8. Explicitly out of scope

- Full-manuscript continuous editing across chapter files.
- Simulated printed-page editing or pagination in Write.
- Automatic conversion of chapters into one file.
- Independent database-backed copies of project content per window.
- Requiring multiple monitors to use LithoMark.
- Automatic simultaneous opening of multiple different novels as a requirement for the first release. If added later, strict project isolation is mandatory.

## 9. Dependencies and open implementation questions

Before coding, inspect the current Tauri window setup, Svelte state/store boundaries, editor component lifecycle, file watcher coverage, and existing conflict detection. Choose the initial same-document editing policy (single editor owner versus synchronized edit session) through a separate implementation decision. Confirm the location and schema of layout JSON without adding a required content database.

**Implementation status:** Architectural design documented; no runtime multi-window functionality is claimed by this document.
