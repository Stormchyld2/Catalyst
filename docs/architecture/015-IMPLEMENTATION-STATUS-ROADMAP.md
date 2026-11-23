# LithoMark Writer OS — Implementation Status and Roadmap


## SQLite Removal Direction

SQLite is transitional legacy state from Kindling and is not part of the target LithoMark project architecture.

- Authoritative project identity and metadata live in `project.yaml` inside the local project folder.
- Runtime subsystems should accept or resolve the active local project root directly and operate on files beneath that root.
- GitHub is optional synchronization/version control only; normal project reads and writes must not depend on network access.
- New subsystems must not add new SQLite dependencies.
- Existing SQLite-backed commands should be migrated incrementally to file-authoritative equivalents, then removed when no longer referenced.
- Recent-project/history convenience state, if retained, must be non-authoritative and rebuildable without affecting project contents.

## Locked navigation model

LithoMark uses two intentionally distinct home destinations:

- **Global Home** is the permanent Home workspace on the Writer OS rail. It is the project launcher/library for creating, importing, opening, and switching books. Visiting Global Home does not implicitly close the active project.
- **Project Dashboard** is the active book's command center. It is entered from the project/editor sidebar and never substitutes for Global Home.
- Project workspaces (Story Development, Outline, Write, Review, Listen, Publish, Git, Terminal, and enabled plugin workspaces) remain scoped to the active project.
- Navigation must remain circular: Global Home → project → project workspaces → Project Dashboard → project workspaces → Global Home, without forcing a project close or creating a dead end.
- Optional plugin workspaces such as Audiobook Studio appear in navigation only when their plugin contribution is installed and enabled.

This navigation contract is locked unless a later architecture change explicitly supersedes it.

## Runtime Data Rule

LithoMark must not ship or depend on mock, demo, placeholder, or hard-coded project data in normal application flows.

- Live workspaces must read from the actual project folder and authoritative Markdown/YAML/JSON files from the beginning of implementation.
- Disposable indexes and caches are allowed only when they are derived from and rebuildable from those files.
- Temporary synthetic files and fixture data are allowed inside automated tests, temporary test directories, QA/demo harnesses, or isolated development fixtures.
- Test doubles may simulate IPC or external boundaries in unit tests, but they must not replace the real production storage path or become runtime fallbacks.
- New features should be wired to their intended real file/folder source as they are introduced so no later "mock data cleanup" migration is required.


**Status:** Active implementation checkpoint  
**Branch:** `migration/kindling-foundation`  
**Purpose:** Record what is architecturally locked, what is implemented and tested, what remains transitional, and the next implementation sequence.

## Story Development canonical storage

New projects use one canonical `Story Development/` hierarchy for Story Bible/Wiki entities, planning and outline material, timeline information, and story-specific research. These are areas of one Story Development domain, not independent top-level persistence authorities.

Older projects using top-level `Story Bible/`, `Story/`, `Timeline/`, or `Research/` locations remain readable migration inputs. New writes target the canonical Story Development hierarchy, and any migration must be explicit and non-destructive.

## Locked foundation

LithoMark is a local-first Writer OS. Normal project files and folders are authoritative; Markdown/YAML/JSON are preferred. SQLite may remain only as disposable/rebuildable cache or transitional compatibility state.

A project is the product. Write, Story Development, Review, TTS, Audiobook, Publish, Git, and other capabilities operate on the same project and stable identities rather than importing or duplicating manuscript data.

The manuscript lifecycle is:

Story Development → Outline → Write → Developmental Review → Revision → Beta Reading → Revision → ARC → Publish

Core + first-party plugins + community plugins is the extension model. First-party plugins use the documented public Plugin API wherever reasonably possible.

TTS Reader is Core. Audiobook Studio is a bundled optional first-party plugin consuming supported Core TTS/audio APIs. Audiobook Studio must not duplicate manuscript authority or create a competing TTS architecture.

Interactive mapping is deferred to a later optional first-party plugin. Core Story Development retains authoritative location/worldbuilding data and exposes supported APIs; map implementation must not block Core development.

## Implemented and green

### Project root foundation
- Explicit project root distinct from source/import path.
- Portable file-oriented project foundation.
- Focused project-root tests green: 6 passed, 0 failed.

### File-authoritative Document Service
- Markdown/YAML documents are authoritative.
- Safe project-relative path validation.
- Safe staged saves with conflict detection.
- Unknown YAML metadata preserved semantically.
- Windows CRLF source hashes use actual on-disk bytes and do not false-conflict.
- Markdown-significant trailing spaces/tabs are preserved.
- Core stable IDs use canonical type-prefixed UUID v4 identifiers.
- Prefix/type mismatches are rejected.
- Document Service tests green: 10 passed, 0 failed.

### Review foundation
- Native portable review manifest/integrity foundation.
- Distribution ledger and review-package compatibility work.
- Review tests green: 45 passed, 0 failed.

### Plugin foundation
- Manifest and permission contracts.
- Activation events and declarative contributions.
- ExtensionContext and disposable subscription lifecycle.
- Permission-gated command registry.
- Permission-gated Plugin Document API routed through the Core Document Service.
- First-party status does not bypass permissions.
- Plugin tests green: 25 passed, 0 failed.

### Manuscript Structure Service
- File-authoritative chapter/scene structure using stable `ch-UUID` and `scn-UUID` identities.
- Explicit chapter scene ordering and explicit scene parent relationships.
- Move/reorder operations preserve stable identity.
- Structure integrity reporting detects missing, duplicate, orphaned, and mismatched references without silently repairing them.
- Structure mutations route through the Document Service conflict checks.
- Manuscript-focused tests green.

### New Book Foundation architecture
- Guided new-book setup is now locked as a temporary project-seeding workflow.
- Sequence: Project Basics → Characters → Locations → Worldbuilding → Style & Voice → Themes/Core Ideas → Brainstorming → Synopsis → Manuscript Structure → Review/Create.
- Worldbuilding is a required foundation stage. The writer may keep it minimal, but project creation does not treat the Worldbuilding stage itself as optional.
- Create Book generates ordinary authoritative files and enters Story Development / Wiki.
- Plot/outline methodology is deliberately selected later; Manuscript Structure may seed the initial chapter/scene scaffold without becoming a plotting methodology.
- Core workflow requires no AI.
- See `016-NEW-BOOK-FOUNDATION.md`.

### Frontend transition
- Native review adapter and transition bridge have passed Svelte checks.
- Existing Kindling-derived frontend/backend remains transitional while the new Writer OS shell is built.

## Transitional / deliberately not yet final

- SQLite-backed Kindling models and commands remain in the repository for migration/compatibility and must not regain authoritative status.
- The current UI is not yet the final Writer OS shell.
- No full file watcher/index rebuild layer yet.
- No final Git workflow UI yet.
- No public extension registry/browser yet.
- No centralized plugin Stable ID API for namespaced plugin object types yet.
- Some Kindling naming/comments remain and will be removed during migration cleanup.

## Next implementation sequence

1. New application shell/navigation and Home/project loading, including the New Book Foundation entry flow.
2. Write workspace.
3. Story Development workspace and generated-Wiki handoff.
4. Shared cross-module context, wikilinks, backlinks/resolution, and right Context Panel.
5. Git workflow.
6. Recovery/polish/Windows installer.
7. Core Preview: Write + Story Development.
8. Review expansion.
9. Core TTS Reader integration.
10. Optional first-party Audiobook Studio plugin.
11. Publishing Studio and later ecosystem/registry work.

## Manuscript Structure Service checkpoint

The next service must establish chapters/scenes as shared domain objects without creating a competing persistence layer.

Required baseline:
- Chapter and scene identity uses `ch-UUID` and `scn-UUID`.
- Explicit scene ordering is stored in authoritative chapter metadata; filenames and directory order are not structural truth.
- A scene's parent chapter relationship is explicit and stable-ID based.
- Reordering does not change IDs.
- Moving a scene between chapters does not change its scene ID.
- Renaming/moving files does not change document identity.
- Structure mutations route through the Document Service and retain optimistic conflict protection.
- Missing, duplicate, or mismatched structure references are reported rather than silently repaired.
- The service remains rebuildable from authoritative project files without SQLite.

## Preview boundary

The first public/core preview is Write + Story Development on the new file-authoritative architecture. Review, Audiobook Studio, Publishing Studio, ecosystem website/registry, and other optional systems do not block that preview.

## Governing implementation rule

Do not add a shortcut that makes UI state, plugin state, SQLite, or a specialized workspace more authoritative than the project files and stable domain identities.


### New Book Foundation backend seeder

Status: **implemented and locally verified**

- file-authoritative `project.yaml` creation without SQLite project lookup;
- canonical Story Development and Planning directories;
- local Markdown/YAML foundation seeding;
- Character and Location stable-ID document creation;
- filename sanitization and duplicate-name handling;
- Tauri command: `create_new_book_foundation`;
- guided frontend workflow is implemented and hands the created project directly into Story Development.


### Entity template registry

Status: **full template system implemented, pending local test checkpoint**

- rich built-in entity/file templates for characters, locations, organizations/governments, worldbuilding, objects/items, events, and research;
- recursive local `Templates` folder discovery;
- ordinary Markdown + YAML user-template format;
- strict manifest validation for IDs, fields, field types, Overview mappings, and custom image-field mappings;
- user/built-in template ID collision protection;
- invalid-template issue reporting without hiding valid templates;
- file-backed create, edit/rename, and delete operations for user templates;
- built-in templates remain read-only;
- Story Development manager UI for browsing built-ins and managing user templates;
- Story Development Overview resolves explicitly assigned templates and honors template-declared Overview fields and image field;
- no SQLite dependency or proprietary template store;
- New Book Foundation Review & Create template assignment remains the next integration layer.

### Demo project

Status: **architecture locked, implementation pending**

- explicit synthetic sample project only;
- normal local LithoMark project folder;
- representative Story Development, entities, worldbuilding, planning, and two short manuscript chapters;
- never used as silent runtime fallback data.


### Story Development tabs and images

Status: **implemented, pending local test checkpoint**

- persistent top tabs for opened Story Development documents and images;
- local project image discovery under Story Development roots including Assets/Images;
- safe project-relative image byte reads with traversal rejection;
- center-pane image viewer;
- Overview-card image support through an `image` metadata path;
- deeper built-in entity templates with template-declared Overview fields.


### Story Development UX completion

Status: **implemented and locally verified**

The Story Development workspace now includes:

- file search across title, path, and document type;
- Contents navigation generated from Markdown headings with stable rendered anchors;
- Markdown formatting controls for bold, italic, strikethrough, and H1-H3;
- live Split mode with explicit Editor and Preview panes;
- active-tab dirty indication and guarded close/navigation for unsaved work;
- close-tab behavior with a bounded recently-closed stack and Reopen Closed Tab action.

These behaviors remain UI/session state over authoritative local project files and do not introduce a secondary content store.


### New Book Foundation guided UI

Status: **implemented and locally verified**

- ten-step guided flow: Project Basics → Characters → Locations → Worldbuilding → Style & Voice → Themes/Core Ideas → Brainstorming → Synopsis → Manuscript Structure → Review & Create;
- setup state remains temporary until Create Book;
- optional development steps may be skipped;
- local destination folder is selected explicitly;
- character and location rows validate before creation;
- Create Book calls the file-authoritative `create_new_book_foundation` command;
- successful creation activates the new local project and opens Story Development directly;
- the handoff preserves the completed Story Development tabs, images, Overview/context, Contents, editor, Split mode, and search UX;
- plotting methodology is not forced during book creation; the Manuscript Structure step seeds only the writer-selected initial chapter/scene scaffold;
- entity type/template correction and assignment on Review & Create remains the next integration block.

## Locked Write Experience Additions

### Writing history
LithoMark will provide lightweight local writing history independently of Git.

- Keep a small rotating set of automatic snapshots per chapter, targeting **five snapshots** rather than unlimited history.
- Snapshot retention should be spread across meaningful recent points in time over several days, not created continuously on every keystroke.
- A snapshot is created only when content has changed.
- Restoring an older version must preserve the current version first so recovery is non-destructive.
- Authors may later create named/manual checkpoints that are retained separately from the rotating automatic set.
- Snapshot storage remains project-local and must not become a second manuscript authority.
- Git remains repository-level version control/synchronization; Writing History is author-facing local recovery.

### Expanded writing goals
Extend the existing writing-progress foundation rather than replacing it.

Planned additions include:
- daily and weekly targets;
- project deadline / target-date tracking;
- writing-session statistics;
- progress trends;
- separate drafting and revision activity where practical.

Goals and statistics should remain optional and non-intrusive. They must not interrupt or gamify the writing flow by default.

### Distraction-free writing
No separate full-screen subsystem is required. Hiding/collapsing the existing sidebars already produces the intended editor-first writing surface. Preserve and polish that behavior rather than creating a duplicate editor mode.

### Review and Publish boundaries
- Inline reader comments, highlights, questionnaires, and returned feedback belong to the browser-based Beta/ARC reader package and Review workspace.
- Book-formatting preview belongs to Publishing & Export Studio.
- Write remains focused on writing, revision, manuscript structure, history, and writing progress.
