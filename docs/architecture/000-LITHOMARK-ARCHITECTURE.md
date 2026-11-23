# LithoMark Writer OS — Architecture Specification

**Document:** 000-LITHOMARK-ARCHITECTURE.md  
**Status:** Locked Baseline  
**Project:** LithoMark Writer OS  
**Purpose:** Primary architectural authority for LithoMark Writer OS

---

# 1. Purpose

LithoMark Writer OS is a local-first writing and story-development environment designed to provide writers with a durable, portable, extensible workspace for developing, writing, revising, organizing, and maintaining complex writing projects.

LithoMark is not intended to be merely a word processor.

It is a Writer Operating System.

The system combines manuscript writing with structured story information, project organization, canon tracking, continuity checking, timelines, research, plotting, revision management, extensibility, and external automation capabilities.

This document establishes the architectural principles that all LithoMark development must follow.

When an implementation decision conflicts with this document, this document takes precedence unless the architecture is formally amended.

LithoMark's canonical manuscript lifecycle is:

> **Story Development → Outline → Write → Developmental Review → Revision → Beta Reading → Revision → ARC → Publish**

This lifecycle is a guiding workflow rather than a mandatory wizard. All stages operate on one authoritative project. The detailed cross-workspace contract is defined in **012-LITHOMARK-WRITER-OS-INTEGRATION-ARCHITECTURE.md**.

---

# 2. Product Identity

Product Name:

**LithoMark**

Product Subtitle:

**Writer OS**

LithoMark is an independent application.

The application may use, modify, and incorporate open-source software where licensing permits.

The initial application foundation is derived in part from the open-source Kindling writing application.

LithoMark must not depend upon Kindling branding, identity, infrastructure, or continued development.

Where Kindling code is retained, all applicable open-source license and copyright requirements must remain intact.

LithoMark branding, application identity, documentation, visual language, services, infrastructure, and product direction are independent.

---

# 3. Foundational Technology

The initial LithoMark desktop architecture uses:

- Tauri 2
- Rust
- Svelte
- TypeScript
- TipTap
- SQLite where appropriate for non-authoritative indexes and caches

These technologies may evolve over time.

The permanent LithoMark project format must not depend upon any particular frontend framework, editor, database engine, or application implementation.

A future LithoMark implementation must be capable of reading an existing LithoMark project without requiring the original UI or database implementation.

---

# 4. Core Architectural Principle

The fundamental LithoMark architecture is:

```text
LithoMark UI
      │
      ▼
LithoMark Domain API
      │
      ▼
LithoMark Domain Models
      │
      ▼
LithoMark Project Service
      │
      ▼
Authoritative Project Files
      │
      ├── Markdown
      ├── YAML
      ├── JSON
      └── Project Assets
      │
      ▼
Rebuildable Infrastructure
      │
      ├── SQLite indexes
      ├── Search indexes
      ├── Caches
      ├── Temporary files
      └── Generated metadata
```

Authoritative project information must never exist solely inside a disposable cache or internal database.

---

# 5. Local-First Principle

LithoMark is local-first.

A writer must be able to:

- Create a project without an account.
- Open a project without an internet connection.
- Write without an internet connection.
- Access project files directly.
- Back up a project independently of LithoMark.
- Move a project between computers.
- Archive a project independently.
- Recover the writer's work without access to LithoMark services.

Internet connectivity must not be required for normal writing and project-management operations.

Future cloud functionality may be offered as an optional service.

Cloud functionality must never become a prerequisite for ownership or access to a writer's work.

---

# 6. Data Ownership

The writer owns the project.

LithoMark must not create artificial dependency on LithoMark software for access to the writer's manuscript or core project information.

Whenever practical, durable project information must use documented, human-readable formats.

Primary formats include:

- Markdown
- YAML
- JSON
- Standard image formats
- Standard document formats
- Standard media formats

Proprietary binary formats should be avoided for authoritative project information.

---

# 7. Project Storage

A LithoMark project is fundamentally a normal filesystem directory.

Example:

```text
My Novel/
│
├── lithomark.yaml
│
├── Manuscript/
├── Story Bible/
├── Timeline/
├── Story/
├── Research/
├── Notes/
├── Templates/
├── Assets/
│
└── .lithomark/
```

The exact project format is defined by:

**001-PROJECT-FILE-FORMAT.md**

The visible project directories contain durable project information.

The `.lithomark` directory contains LithoMark-specific operational information.

---

# 8. SQLite Policy

SQLite is permitted within LithoMark.

SQLite must not be the sole authoritative repository for manuscript or critical project information.

SQLite may be used for:

- Search indexes
- Cached metadata
- Relationship indexes
- Performance acceleration
- Derived statistics
- Temporary processing state
- Plugin indexes
- Rebuildable application state

A corrupted or deleted LithoMark index database must be reconstructable from authoritative project files.

Deleting the LithoMark cache must never destroy the manuscript or core story information.

---

# 9. LithoMark Domain Layer

LithoMark must maintain a domain layer between the user interface and physical project storage.

The UI must not directly manipulate project files.

Plugins must not directly manipulate LithoMark's internal indexes.

External agents must not directly manipulate internal indexes.

Instead:

```text
UI
Plugin
External Agent
     │
     ▼
LithoMark Domain API
     │
     ▼
Project Services
     │
     ▼
Project Files
```

This separation protects project integrity and allows LithoMark's implementation to evolve without breaking projects or extensions.

---

# 10. Document Model

The editor implementation must not define the permanent LithoMark document format.

LithoMark maintains its own document abstraction.

The relationship is:

```text
Project File
     │
     ▼
LithoMark Document Model
     │
     ▼
Editor Adapter
     │
     ▼
TipTap
```

TipTap is the initial editor implementation.

TipTap may be replaced in the future without changing the permanent LithoMark project format.

The document architecture is defined in:

**002-DOCUMENT-MODEL.md**

---

# 11. Stable Object Identity

Every significant LithoMark domain object must have a stable internal identifier.

Examples include:

- Projects
- Chapters
- Scenes
- Characters
- Locations
- Organizations
- Objects
- Events
- Relationships
- Plot threads
- Character arcs
- Reveals
- Clues
- Research records
- Notes

Names must not function as permanent identifiers.

Example:

```yaml
id: chr-a84f2931
name: Leah Bennett
```

References use the stable ID:

```yaml
character: chr-a84f2931
```

Changing a character's name therefore does not destroy existing relationships.

The identifier architecture is defined in:

**003-STABLE-ID-SYSTEM.md**

---

# 12. Core Domain Systems

LithoMark Core is responsible for fundamental writing and project functionality.

Core systems include:

## Manuscript

- Projects
- Parts
- Chapters
- Scenes
- Beats
- Manuscript editing
- Word counts
- Status tracking
- Comments
- Revisions
- Search
- Find and replace

## Story Bible

- Characters
- Locations
- Organizations
- Objects
- Events
- Relationships
- Professions and procedures
- Custom entities
- Custom fields
- Tags

## Story Development

Story Development is the canonical story-bible, worldbuilding, timeline/canon, and linked-knowledge environment. It owns the structured story information that other workspaces project and consume.

**Outline is a separate top-level workspace in the Writer OS rail, positioned between Story Development and Write.** Outline operates on the same stable chapter, scene, beat, entity, chronology, and relationship identities; it is a coordinated projection, not a second planning database.

Story Development supports planning knowledge such as:

- Plot threads
- Subplots
- Character arcs
- Beats
- Reveals
- Clues
- Foreshadowing
- Payoffs
- Story structures

## Timeline

- Events
- Dates
- Date ranges
- Approximate dates
- Relative chronology
- Character participation
- Scene relationships
- Location relationships

## Canon

- Facts
- Dates
- Ages
- Locations
- Relationships
- Occupations
- Physical attributes
- Possessions
- Injuries
- Deaths
- Knowledge
- Aliases
- Events
- Status

## Continuity

LithoMark may deterministically compare recorded project information to identify possible conflicts.

Continuity checking must be based on explicit project information rather than generative inference.

## Research

- Research documents
- Sources
- Notes
- Images
- Attachments
- Project references

## Revision and Recovery

- Autosave
- Revision history
- Snapshots
- Backups
- Recovery
- Project integrity checks

## Review Lifecycle

- Developmental Review
- Revision Items
- Beta Reading packages and rounds
- ARC packages and recipient distribution
- Durable review/distribution ledger
- Returned feedback and revision routing

Review packages are deliberate snapshots and must not replace the authoritative working manuscript. Beta and ARC may share native `lithomark-review` infrastructure while remaining distinct workflow types.

## Import and Export

LithoMark should preserve and expand the mature import/export functionality inherited from Kindling where appropriate.

---

# 13. Canon Philosophy

Canon is explicit recorded fact.

LithoMark does not determine what is canon through artificial intelligence.

The writer establishes canon.

Example:

```yaml
subject: chr-ray-carter
fact_type: death
date: 2023-11-03
location: loc-irish-channel
status: confirmed
```

LithoMark may compare other structured project information against that fact.

For example, if a scene dated after November 3, 2023 records the same character as physically present and alive, LithoMark may identify a potential continuity conflict.

The writer decides whether the conflict is intentional.

The canon architecture is defined in:

**004-CANON-MODEL.md**

---

# 14. No Built-In Artificial Intelligence

LithoMark Writer OS does not contain a built-in generative AI writing system.

Core writing functionality must not require:

- OpenAI
- Anthropic
- Google Gemini
- Local language models
- Cloud AI services
- Any other generative AI provider

LithoMark remains fully functional without generative AI.

This does not prohibit externally authorized software from interacting with LithoMark through documented interfaces.

---

# 15. External Agent Architecture

LithoMark may expose a controlled interface for external agents and automation systems.

External access must be:

- Disabled by default.
- Explicitly authorized by the user.
- Permission controlled.
- Revocable.
- Auditable where practical.

Potential permissions include:

- Read project structure
- Read manuscript
- Read selected documents
- Read Story Bible
- Read timeline
- Read canon
- Create notes
- Create proposed changes
- Modify permitted documents
- Create project entities

Destructive permissions require stronger authorization.

External agents must not receive unrestricted filesystem or database access merely because they are connected to LithoMark.

The agent architecture is defined in:

**006-AGENT-API.md**

---

# 16. Plugin Architecture

LithoMark supports extensibility through a formal plugin architecture.

Community plugins must interact with LithoMark through documented APIs.

Plugins must not depend upon undocumented internal implementation details.

The plugin architecture must provide:

- Plugin manifests
- Versioning
- Compatibility declarations
- Permission declarations
- Installation
- Uninstallation
- Updating
- Enable/disable controls
- Error isolation where practical
- Community distribution capability

Possible plugin permissions include:

- Read manuscript
- Modify manuscript
- Read Story Bible
- Modify Story Bible
- Read timeline
- Add UI panels
- Add commands
- Add exporters
- Add importers
- Network access
- External filesystem access

Plugins must not receive unnecessary privileges.

The plugin architecture is defined in:

**005-PLUGIN-ARCHITECTURE.md**

---

# 17. Core Versus Plugins

LithoMark uses a **Core + First-Party Plugins + Community Plugins** model.

Core contains the minimum capabilities required to keep a project trustworthy, portable, openable, editable, and recoverable. This includes the project/file format services, authoritative Document Service, stable IDs, safe saving and recovery primitives, shared domain/application context, plugin host/API/permissions, and the minimum Write and Story Development model required to understand the project.

Optional product capabilities should be first-party plugins where practical. Official plugins may be bundled and enabled by default, but bundling does not make their data authoritative Core state.

First-party plugins must use the documented, permission-controlled Plugin API wherever reasonably possible. LithoMark must not depend on secret first-party persistence shortcuts that make the community API a second-class architecture.

Disabling an optional first-party or community plugin must not make the core manuscript inaccessible or delete durable writer-owned plugin data.

Specialized functionality may be implemented as plugins.

Examples:

- Fantasy calendars
- Specialized genealogy systems
- Interactive maps
- Genre-specific analytical tools
- External publishing services
- Third-party integrations
- Specialized exporters
- Advanced Git services

---

# 18. Version Control

LithoMark Core provides writer-friendly revision history, snapshots, backup, and recovery.

Users must not be required to understand Git.

Advanced Git functionality may be implemented as a first-party LithoMark extension.

GitHub integration may be implemented separately.

The standard filesystem project architecture should remain compatible with external version-control systems.

---

# 19. Template Architecture

Templates are first-class LithoMark resources.

Templates may define:

- Project structures
- Story structures
- Story Bible entities
- Custom fields
- Tags
- Plot frameworks
- Beat sheets
- Research structures
- Genre-specific structures

LithoMark must not hard-code one genre or writing methodology into the core application.

Potential templates include:

- Crime
- Mystery
- Thriller
- Romance
- Fantasy
- Science Fiction
- Horror
- Historical Fiction
- Literary Fiction
- Nonfiction
- Memoir
- Screenplay

Community-created templates may be supported.

---

# 20. Portability

A LithoMark project must remain portable.

A user should be able to copy:

```text
My Novel/
```

to another supported computer and open the project.

Project portability must not depend upon registry entries, hidden system databases, cloud accounts, or machine-specific absolute paths.

Where machine-specific information is unavoidable, it must remain separate from authoritative project information.

---

# 21. Backup and Recovery

Loss prevention is a core architectural requirement.

LithoMark must be designed under the assumption that:

- Applications crash.
- Computers lose power.
- Storage devices fail.
- Users accidentally delete information.
- Files become corrupted.
- Plugins fail.
- Synchronization tools create conflicts.

LithoMark must therefore support layered protection including:

- Safe writes
- Autosave
- Revision history
- Snapshots
- Backups
- Integrity checking
- Recovery mechanisms

These mechanisms have distinct responsibilities and must not be presented as interchangeable:

- **Autosave** persists the writer's current edits safely and promptly. It is not a historical backup.
- **Revision history** preserves recoverable prior document states at a useful editing granularity. It is not a substitute for an independent backup.
- **Snapshots** are deliberate point-in-time project recovery states managed by LithoMark. They must identify what project state they represent.
- **Backups** are independent recoverable copies intended to survive damage, deletion, or corruption of the working project. A backup should not rely solely on the same disposable database/cache as the working project.
- **Git/version control** records repository history when configured. It is useful protection and collaboration infrastructure, but LithoMark must not describe Git as a complete backup strategy.
- **Integrity checks** detect missing, malformed, inconsistent, or corrupted project state; detection does not itself constitute recovery.

Recovery operations must favor preservation over destructive replacement. When practical, LithoMark should preserve the damaged/current state before restoring an older state so recovery itself does not erase potentially recoverable work.

Backups, snapshots, and migrations should record sufficient project-format/version information to support safe restoration. Machine-specific caches and indexes should be rebuildable after recovery rather than treated as essential backup payloads.

A single corrupted database must never make the manuscript unrecoverable.

---

# 22. Import Philosophy

LithoMark should make adoption easy.

Where practical, LithoMark should support migration from established writing systems.

Kindling-derived import capabilities should be retained or adapted where licensing and architecture permit.

Potential import sources include:

- Scrivener
- novelWriter
- Plottr
- yWriter
- Obsidian
- Longform
- Markdown
- Standard text/document formats

Imported projects must be converted into the native LithoMark project architecture.

---

# 23. Export Philosophy

LithoMark must never intentionally trap a writer's work.

Writers must be able to export manuscripts and project information into useful external formats.

Potential export formats include:

- Markdown
- DOCX
- EPUB
- Plain text
- Other standard publishing formats

Export capabilities may expand through plugins.

---

# 24. User Interface Philosophy

LithoMark should behave as a Writer OS rather than a conventional word processor overloaded with controls.

The interface should emphasize:

- Writing
- Project navigation
- Story understanding
- Context
- Relationships
- Organization
- Focus

Complex systems should remain available without constantly occupying the writing workspace.

The application should progressively reveal complexity as needed.

---

# 25. Kindling-Derived Architecture

Kindling provides the initial implementation foundation for several LithoMark systems.

Components considered candidates for retention or adaptation include:

- Tauri application structure
- Rust backend architecture
- Svelte frontend architecture
- TipTap editor integration
- Chapters
- Scenes
- Beats
- References
- Custom fields
- Tags
- Comments
- Revisions
- Writing statistics
- Search
- Importers
- Exporters
- Screenplay functionality
- Testing infrastructure

Kindling's SQLite-as-authoritative-project-storage architecture must not become LithoMark's permanent storage model.

Kindling-specific branding must be removed.

Third-party branding, artwork, icons, or design assets not licensed for LithoMark use must not be incorporated.

The migration strategy is defined in:

**007-KINDLING-MIGRATION.md**

---

# 26. Architectural Boundaries

LithoMark development should maintain clear boundaries between:

```text
Presentation
    │
Domain
    │
Services
    │
Persistence
```

Presentation must not define domain truth.

Persistence must not define domain behavior.

Plugins must interact through supported APIs.

External agents must interact through supported APIs.

Project files must remain independent of the current UI implementation.

---

# 27. Forward Compatibility

LithoMark project formats and APIs must be versioned.

Project metadata must identify the applicable format version.

Example:

```yaml
lithomark:
  project_format: 1
```

Future LithoMark versions must provide migrations when project structures change.

Migrations must preserve user data.

Whenever practical, migrations should be reversible or preceded by an automatic backup.

---

# 28. Platform Strategy

Primary platform:

**Desktop**

Initial targets:

- Windows
- macOS
- Linux

Future platforms may include:

- Android
- iOS

Mobile implementations may initially provide a subset of desktop functionality.

The project format and domain architecture must not unnecessarily prevent future mobile support.

---

# 29. Security Principle

LithoMark should follow least-privilege principles.

Core functionality, plugins, and external integrations should receive only the capabilities necessary to perform their tasks.

Particular care must be taken with:

- Manuscript access
- Filesystem access
- Network access
- Plugin execution
- External agents
- Credentials
- Publishing integrations
- Destructive operations

Security restrictions must not compromise the writer's ownership of their own project files.

---

# 30. Architectural Decision Rule

When choosing between two implementations, LithoMark development should prefer the solution that better preserves:

1. Writer ownership
2. Data durability
3. Portability
4. Local-first operation
5. Open project formats
6. Recoverability
7. Extensibility
8. Security
9. Maintainability
10. Long-term compatibility

Convenience for the current implementation must not override long-term project integrity.

---

# 31. Governing Principle

LithoMark exists to help writers create and maintain their work.

The software is replaceable.

The writer's work is not.

Therefore:

**The manuscript and project belong to the writer, remain accessible outside LithoMark, and must never depend upon LithoMark's continued existence for their survival.**

---

# 32. Project Isolation and Optional Series Scope

Every LithoMark writing project is an independent project root with its own `lithomark.yaml`, authoritative files, project identity, indexes/caches, and optional Git repository.

Opening one project must not implicitly expose manuscript content, entities, canon, timeline data, review state, publishing configuration, or audiobook production state from another project.

Cross-project sharing is opt-in rather than ambient.

A future **Series Workspace** may coordinate multiple independent projects for legitimate series-level concerns such as:

- series title and book order;
- series chronology;
- recurring-character references;
- recurring-location references;
- series-level canon and continuity;
- shared series research or reference material;
- series publishing metadata.

A Series Workspace must reference member projects rather than merge them into one proprietary container. Each member book must remain independently openable, portable, archivable, publishable, and recoverable without the Series Workspace.

Series-level information must be clearly distinguishable from book-level information. LithoMark must not silently copy a book's authoritative entities into another book or treat similarly named entities in different projects as identical.

Where a series-level record references a project entity, it should use explicit project identity plus stable entity identity rather than name matching alone.

The Series Workspace is an optional coordination layer, not a replacement for the one-folder-per-book project model.

---

# 33. Core TTS and Optional Audiobook Studio

LithoMark's basic **TTS Reader is a Core capability**. It is part of the everyday writing and accessibility experience and must remain available without Audiobook Studio.

Core TTS responsibilities include:

- reading the current selection;
- reading a scene or chapter;
- play, pause, resume, and stop controls;
- voice and playback-speed selection;
- the provider/engine abstraction required to support local, system, or permitted external TTS engines;
- stable manuscript/document access required for playback.

The **Audiobook Studio is a bundled, optional first-party plugin**. It is specialized production functionality rather than a prerequisite for writing, project access, or basic text-to-speech.

Audiobook Studio may provide:

- narrator and voice assignments;
- pronunciation management;
- chapter production workflows;
- takes and regeneration;
- audio sequencing;
- production status;
- mastering;
- audiobook export and production metadata.

Audiobook Studio must consume supported Core APIs, including the public audio/TTS capability contract, rather than reaching into private TTS implementation details. It must follow the same public Plugin API and permission model used by other first-party plugins wherever reasonably possible.

Conceptually:

```text
LithoMark Core
├── Write
├── Story Development
├── Document / Manuscript Services
├── Plugin Host / API
└── TTS Reader + Audio Service Contract
             │
             ▼
      Audiobook Studio
      (optional bundled
       first-party plugin)
```

There must be only one underlying TTS/audio service architecture. Audiobook Studio must not create a competing TTS implementation or duplicate authoritative manuscript content.

Disabling or uninstalling Audiobook Studio must not:

- remove the Core TTS Reader;
- make manuscript content inaccessible;
- alter or duplicate authoritative manuscript text;
- prevent Write or Story Development from functioning.

Audiobook-specific durable writer-owned metadata must remain safely preserved according to the plugin data-preservation rules even when the plugin is disabled.

This distinction is locked:

> **TTS Reader is Core. Audiobook Studio is a bundled, optional first-party plugin that consumes Core TTS/audio services through supported APIs.**

---

# 34. Related Architecture Documents

This specification is supplemented by:

- `001-PROJECT-FILE-FORMAT.md`
- `002-DOCUMENT-MODEL.md`
- `003-STABLE-ID-SYSTEM.md`
- `004-CANON-MODEL.md`
- `005-PLUGIN-ARCHITECTURE.md`
- `006-AGENT-API.md`
- `007-KINDLING-MIGRATION.md`

These documents may provide implementation detail but must remain consistent with the principles established here.

---



# 35. Connected Story Intelligence and Workspace Projections

LithoMark's planning, manuscript, Story Development, research, timeline, canon, relationship, and review capabilities must operate on shared authoritative project documents and stable identities rather than becoming isolated feature silos.

The same story object may be presented through multiple workspace projections. For example, a scene may appear in Outline for planning, Write for prose, Timeline for chronology, a relationship or entity view for context, and Review for feedback. Those projections must not create competing scene identities or duplicate authoritative prose.

## 35.1 Planning-to-Prose Continuity

Outline and Write are views over compatible chapter and scene identity. Moving from planned scene to drafted scene must preserve stable IDs, ordering relationships, and relevant structured metadata. Story-structure frameworks such as Three-Act Structure, Save the Cat, beat sheets, and future outline templates organize the story but do not become a competing manuscript authority.

## 35.2 Context While Writing

Write should progressively expose relevant context without forcing the author to leave the manuscript. Context may include POV, location, characters present, scene purpose, conflict, chronology, related entities, notes, and linked Story Development documents. Context panels are projections of authoritative metadata and links, not a second storage system.

## 35.3 Research Workspace

Research remains ordinary project-owned files in a dedicated top-level Research/ hierarchy. LithoMark may provide richer list, grid, folder, favorites, recent, search, tagging, and linked-entity presentations over those files. Presentation state and indexes are derived; the research files remain authoritative and portable.

## 35.4 Flexible Entity and Codex Model

The template system must support both built-in and writer-defined entity/reference types. Custom categories, fields, and templates may describe concepts not anticipated by Core without requiring a proprietary database schema. Unknown metadata must be preserved according to the Document Model.

## 35.5 Shared Relationship Graph

Character maps, organization hierarchies, faction/alliance views, location connections, and story-wide graphs should be projections of one relationship/link infrastructure. Inputs may include explicit stable-ID relationship metadata, resolved wikilinks, backlinks, scene participation, organization membership, location references, and timeline events.

Explicit relationships carry semantics such as sibling, enemy, member-of, reports-to, or located-in. Derived backlinks and mentions must not be written into YAML merely to make a visualization work; rebuildable indexes may cache them.

## 35.6 Timeline, Event Effects, and Entity State

Timeline events may reference affected entities and explicit state changes. This supports event chronology, event-change views, entity timelines, and later deterministic continuity analysis. The architecture must be capable of answering questions such as what an entity knew, where it was, what status it had, or which relationships were active at a given story point when the writer has supplied the necessary structured facts.

Entity-state history must be derived from authoritative events/canon facts and stable IDs rather than maintained as an unrelated UI-only timeline.

## 35.7 Maps and Spatial Views

Interactive mapping is **not a Core deliverable**. It is reserved for a later optional **first-party plugin** built through the same documented Plugin API available to community extensions.

The map plugin may project existing stable-ID location, route, region, event, relationship, and asset data and may contribute map-specific writer-owned files when explicitly used. Core must expose enough supported location/entity/timeline APIs for the plugin to work without private persistence shortcuts.

Story Development still owns the underlying location/worldbuilding information. The absence or disabling of the map plugin must never make a project incomplete, prevent writing, or make location data inaccessible. Map assets and durable location metadata remain normal project-owned files; derived tiles, indexes, and rendering caches are disposable.

## 35.8 Priority Boundary

Near-term development prioritizes manuscript/scene integration, Story Development, full-project navigation, context panels, links, and relationship metadata. Relationship graphs, richer Research, timeline/event projections, backlinks, entity-state timelines, continuity intelligence, and organization hierarchy build on that foundation. Interactive cartography is explicitly deferred to the optional map plugin and must not block Core development.

This direction is locked: connected-story features share the same stable-ID, Markdown/YAML, file-authoritative foundation instead of creating parallel databases or duplicate story objects.

---

**END OF ARCHITECTURE SPECIFICATION**