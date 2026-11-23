# Catalyst — Project File Format

**Document:** 001-PROJECT-FILE-FORMAT.md  
**Status:** Locked Baseline  
**Project:** Catalyst  
**Parent Specification:** 000-CATALYST-ARCHITECTURE.md

---

# 1. Purpose

This specification defines the permanent on-disk structure of a Catalyst project.

The Catalyst project format is designed around five primary requirements:

1. Writer ownership
2. Human-readable storage
3. Portability
4. Recoverability
5. Long-term compatibility

A Catalyst project is a standard filesystem directory.

The project must remain understandable and recoverable even without the Catalyst application.

SQLite databases, search indexes, caches, and other generated application data are not authoritative project storage.

---

# 2. Fundamental Rule

The authoritative project is the filesystem.

```text
Project Files
     │
     ├── Manuscript
     ├── Story Development
     ├── Notes
     ├── Templates
     └── Assets
```

Catalyst may generate indexes and caches from these files.

If all generated Catalyst application data is deleted, Catalyst must be capable of reconstructing its project state from the authoritative files.

---

# 3. Project Root

Every Catalyst project is contained within a single root directory.

Example:

```text
All That Remains/
```

The directory name is primarily for human organization.

The project identity is determined by the project metadata rather than solely by the directory name.

The directory may therefore be renamed without changing the project's internal identity.

---

# 4. Required Project Manifest

Every Catalyst project must contain:

```text
lithomark.yaml
```

at the project root.

This is the project's primary manifest.

Minimum example:

```yaml
lithomark:
  project_format: 1

project:
  id: prj-550e8400-e29b-41d4-a716-446655440000
  title: All That Remains
  type: novel
  created: 2026-09-26T15:00:00-04:00
  modified: 2026-09-26T15:00:00-04:00
```

The project manifest must remain human-readable.

---

# 5. Project Format Version

Every project must declare its project format version.

Example:

```yaml
lithomark:
  project_format: 1
```

This value describes the structure and schema of the Catalyst project.

It is independent of the installed Catalyst application version.

For example:

```text
Catalyst application: 3.4.1
Project format:        2
```

Multiple application versions may support the same project format.

---

# 6. Project Identifier

Every project receives a permanent stable identifier.

Example:

```yaml
project:
  id: prj-550e8400-e29b-41d4-a716-446655440000
```

The identifier must not depend upon:

- Project title
- Directory name
- Computer
- Username
- Filesystem path

Moving or renaming the project must not change the project identifier.

Stable identifiers are defined further in:

`003-STABLE-ID-SYSTEM.md`

---

# 7. Recommended Root Structure

A standard fiction project should normally contain:

```text
All That Remains/
│
├── lithomark.yaml
│
├── Manuscript/
├── Story Development/
├── Notes/
├── Templates/
├── Assets/
│
└── .lithomark/
```

Not every project must use every visible directory.

For example, a short story may contain only:

```text
Short Story/
├── lithomark.yaml
├── Manuscript/
├── Notes/
└── .lithomark/
```

Catalyst must tolerate optional unused directories.

---

# 8. Directory Naming

Default Catalyst directories use readable names.

Examples:

```text
Manuscript
Story Development
Notes
Templates
Assets
```

These names are intended to make a project understandable outside Catalyst.

Catalyst must not rely exclusively on folder display names to determine object identity.

Where necessary, stable IDs and metadata determine identity.

---

# 9. Manuscript Directory

The `Manuscript` directory contains authoritative manuscript content.

Example:

```text
Manuscript/
├── Prologue.md
├── Chapter 01/
│   ├── Scene 01.md
│   ├── Scene 02.md
│   └── Scene 03.md
├── Chapter 02/
│   ├── Scene 01.md
│   └── Scene 02.md
└── Epilogue.md
```

Catalyst must support multiple organizational models.

A writer may use:

- One file per manuscript
- One file per chapter
- One file per scene
- Chapters containing scene files
- Parts containing chapters containing scenes

The project's manifest or structural metadata records the logical order.

Filesystem alphabetical order must not be treated as the sole canonical manuscript order.

---

# 10. Manuscript Documents

Markdown is the default authoritative manuscript text format.

Example:

```markdown
---
lithomark:
  type: scene
  id: scn-7c42be89

title: Royal Street Gallery
chapter: ch-01a82c74
pov: chr-leah-bennett
status: draft
---

Leah stepped through the gallery doors.

The smell reached her first.
```

The YAML front matter contains structured metadata.

The Markdown body contains manuscript content.

Catalyst's editor may visually hide the YAML front matter during normal writing.

---

# 11. Front Matter

Catalyst documents may contain YAML front matter.

Front matter begins and ends with:

```text
---
```

Example:

```yaml
---
lithomark:
  type: scene
  id: scn-7c42be89

title: Gallery Arrival
status: draft
pov: chr-leah-bennett
---
```

The `lithomark` namespace is reserved for Catalyst-defined metadata.

Plugins must not place arbitrary values inside the reserved Catalyst namespace.

---

# 12. Custom Metadata

Custom metadata may be supported.

Application-defined custom fields should use documented structures.

Plugin-specific metadata must be namespaced.

Example:

```yaml
plugins:
  com.example.fantasy-calendar:
    calendar_date: "Fourth Moon, Day 17"
```

This prevents plugins from creating conflicting metadata fields.

---

# 13. Chapter Representation

A chapter is a stable Catalyst object.

A chapter may be represented by a directory:

```text
Chapter 05/
```

with optional metadata:

```text
Chapter 05/
├── chapter.yaml
├── Scene 01.md
├── Scene 02.md
└── Scene 03.md
```

Example `chapter.yaml`:

```yaml
lithomark:
  type: chapter
  id: ch-7fa51c20

title: The Gallery
number: 5
status: draft

order:
  - scn-a1214b70
  - scn-c7252d91
  - scn-19f26ae4
```

This explicit order prevents file renaming from accidentally changing manuscript structure.

---

# 14. Scene Representation

A scene should normally be represented by a Markdown document.

Example:

```text
Scene 01.md
```

The scene's identity comes from metadata rather than its filename.

Example:

```yaml
lithomark:
  type: scene
  id: scn-a1214b70
```

The writer may rename:

```text
Scene 01.md
```

to:

```text
Leah Arrives at Gallery.md
```

without changing the scene's identity.

---

# 15. Story Development

The `Story Development` directory is the canonical home for structured planning, Story Bible/Wiki material, timeline material, research, and linked story-development knowledge.

Recommended structure:

```text
Story Development/
├── Foundation/
├── Planning & Outline/
│   ├── Brainstorming/
│   ├── Master Outline/
│   └── Beat Sheets/
├── Characters/
│   ├── Main Cast/
│   ├── Relationships/
│   └── Character Arcs/
├── Worldbuilding & Settings/
│   ├── Locations/
│   ├── Organizations/
│   └── Governments & Factions/
├── Timeline/
│   └── Events/
└── Research/
    └── Sources/
```

The hierarchy is writer-facing organization, not object identity. Stable IDs and document metadata remain authoritative when files are renamed or moved.

Story Development is one domain/workspace. Plot/Outline, Story Bible/Wiki, Timeline, Research, relationships, arcs, reveals, clues, foreshadowing, and payoffs are areas within that domain rather than competing top-level persistence systems.

Legacy projects may contain older top-level `Story Bible/`, `Story/`, `Timeline/`, or `Research/` directories. Catalyst must tolerate and read supported legacy locations during migration. New projects and newly generated content use the canonical `Story Development/` hierarchy. Migration must be explicit and non-destructive; Catalyst must not silently move or delete writer files.

---

# 16. Story Development Entity Documents

Each significant entity should normally have its own human-readable Markdown/YAML document with a stable Catalyst ID.

Example:

```text
Story Development/
└── Characters/
    └── Main Cast/
        └── Leah Bennett.md
```

Example metadata:

```yaml
lithomark:
  type: character
  id: chr-a84f2931

name: Leah Bennett
status: active
aliases: []

story:
  role: protagonist
```

Structured facts required for deterministic behavior belong in machine-readable metadata. Long-form development notes belong in the Markdown body.

Locations, organizations, governments/factions, objects/items, events, research records, relationships, and custom entities follow the same stable-ID principle. References between structured objects use stable IDs rather than names or filenames.

---

# 17. Planning, Plot, and Character Arcs

Planning structures live under `Story Development/Planning & Outline/` or another documented Story Development subdirectory.

Plot threads, character arcs, reveals, clues, foreshadowing, payoffs, beats, and story structures may be first-class documents. Their identity comes from stable IDs, not their physical location.

Example:

```yaml
lithomark:
  type: plot-thread
  id: plt-550e8400-e29b-41d4-a716-446655440000

title: Harrison Grant Identity
status: active
```

These objects may reference manuscript scenes, events, characters, and other Story Development objects using stable IDs.

---

# 18. Timeline Within Story Development

Chronological project information lives under `Story Development/Timeline/`.

Example:

```text
Story Development/
└── Timeline/
    └── Events/
        └── Irish Channel Shooting.md
```

A timeline event is a normal authoritative Catalyst document:

```yaml
lithomark:
  type: event
  id: evt-550e8400-e29b-41d4-a716-446655440000

title: Irish Channel Shooting

date:
  type: exact
  value: 2023-11-03

participants:
  - chr-a84f2931

locations:
  - loc-5b12ac84
```

Timeline dates may support exact dates, approximate dates, date ranges, relative dates, and unknown dates. Timeline views and indexes are projections over these authoritative files.

---

# 19. Research Within Story Development

Story-specific research normally lives under `Story Development/Research/` and may contain Markdown notes, sources, images, attachments, links, and source metadata.

Catalyst should avoid modifying external research files unnecessarily. Project-managed research references should use portable project-relative paths where practical.

---

# 28. Notes

The `Notes` directory provides flexible writer-controlled notes.

Example:

```text
Notes/
├── Ideas.md
├── Questions.md
├── Revision Notes.md
└── Parking Lot.md
```

Notes do not need to participate in structured story systems unless explicitly linked.

---

# 29. Templates

The `Templates` directory contains project-specific templates.

Example:

```text
Templates/
├── Scene.md
├── Character.md
├── Location.md
└── Research Note.md
```

Application-wide templates may exist outside individual projects.

Project templates override or supplement global templates according to Catalyst template rules.

---

# 30. Assets

The `Assets` directory contains project-managed media.

Example:

```text
Assets/
├── Images/
├── Maps/
├── Audio/
└── Documents/
```

Project files should reference assets using relative paths whenever practical.

Example:

```markdown
![Downtown New Orleans](../Assets/Maps/downtown-new-orleans.png)
```

Absolute machine-specific paths should not be used for portable project assets.

---

# 31. The `.lithomark` Directory

The project root may contain:

```text
.lithomark/
```

This directory contains application-generated operational information.

Example:

```text
.lithomark/
├── index.db
├── cache/
├── history/
├── recovery/
├── logs/
└── state.json
```

Information stored here must be considered replaceable unless another Catalyst specification explicitly states otherwise.

---

# 32. `.lithomark/index.db`

`index.db` may contain SQLite indexes used for:

- Full-text search
- Relationship traversal
- Fast entity lookup
- Word counts
- Statistics
- Derived continuity information
- UI acceleration

`index.db` must be rebuildable.

Deleting it must not destroy authoritative project information.

Catalyst should automatically rebuild the database when necessary.

---

# 33. Cache

Example:

```text
.lithomark/cache/
```

Cache files may include:

- Rendered previews
- Parsed document representations
- Thumbnails
- Temporary conversions
- Derived metadata

Caches may be deleted safely.

---

# 34. Recovery Data

Example:

```text
.lithomark/recovery/
```

Recovery information may contain temporary emergency copies created during editing.

Recovery files protect against crashes and interrupted writes.

Recovery information must not replace proper durable project storage.

---

# 35. Revision History

Catalyst may maintain local revision information under:

```text
.lithomark/history/
```

However, revision architecture must account for backup and portability.

Important user-created named snapshots may be treated differently from disposable application history.

The exact revision storage strategy should be defined separately before implementation.

---

# 36. Safe File Writes

Catalyst must avoid directly overwriting authoritative project files in a manner that risks corruption.

Preferred write process:

```text
Existing File
     │
     ▼
Write Temporary File
     │
     ▼
Flush / Validate
     │
     ▼
Atomic Replace
     │
     ▼
Updated File
```

Where supported by the operating system, authoritative document writes should use atomic replacement.

---

# 37. Autosave

Autosave must save changes into authoritative project files.

Autosave must not mean:

> The change exists only in SQLite.

The editor may temporarily buffer changes for performance.

However, the durable autosave destination is the project filesystem.

---

# 38. File Change Detection

Catalyst must assume that project files may be changed outside the application.

Examples include:

- VS Code
- Obsidian
- Git
- GitHub clients
- Synchronization software
- Backup restoration
- Manual file editing

Catalyst should monitor project files for external changes.

External changes must trigger appropriate reload, reconciliation, or conflict handling.

Catalyst must not silently overwrite externally modified authoritative files.

---

# 39. Conflict Detection

Before saving an open document, Catalyst should determine whether the underlying file has changed externally since it was loaded or last saved.

If a conflict exists, Catalyst should preserve both versions until the writer resolves the conflict.

The writer's work must not be silently discarded.

---

# 40. File Encoding

Text-based Catalyst project files must use:

**UTF-8**

UTF-8 should be used consistently for:

- Markdown
- YAML
- JSON
- Plain text

This ensures broad language compatibility and cross-platform portability.

---

# 41. Line Endings

Catalyst should tolerate:

- LF
- CRLF

Catalyst should avoid unnecessary line-ending churn.

This is particularly important for Git compatibility.

---

# 42. Relative Paths

Internal project references to files and assets should use paths relative to the project root whenever practical.

Avoid:

```text
C:\Users\Jason\Documents\All That Remains\Assets\map.png
```

Prefer:

```text
Assets/Maps/map.png
```

This allows projects to move between machines.

---

# 43. Filename Independence

Object identity must not depend on filenames.

A character file may change from:

```text
Leah Bennett.md
```

to:

```text
Leah Warren.md
```

while retaining:

```yaml
id: chr-a84f2931
```

All structured relationships therefore remain valid.

---

# 44. Deleted Objects

Deleting a file representing a structured object must be treated as deletion of that object.

Catalyst should detect references to missing IDs.

Example:

```text
Missing reference:
chr-a84f2931
```

Catalyst should provide repair options rather than silently deleting dependent relationships.

---

# 45. Duplicate IDs

Two authoritative objects must not share the same stable ID.

During project indexing, Catalyst must detect duplicate IDs.

Example:

```text
Duplicate stable ID detected:

chr-a84f2931

Files:
Story Development/Characters/Main Cast/Leah Bennett.md
Story Development/Characters/Main Cast/Leah Copy.md
```

The application must require or assist with resolution.

---

# 46. Unknown Files

Catalyst must tolerate files it does not recognize.

A writer may place additional files inside a project.

Catalyst must not automatically delete unknown files.

This is essential for filesystem ownership.

---

# 47. Unknown Metadata

Catalyst should preserve unknown metadata whenever possible.

This allows:

- Forward compatibility
- Plugin metadata
- Manual customization
- Newer project versions

Reading and rewriting a document should not unnecessarily destroy fields Catalyst does not understand.

---

# 48. Plugin Storage

Plugins must not freely modify internal Catalyst data.

Plugin-specific durable data should use documented plugin storage locations.

Example:

```text
.lithomark/
└── plugins/
    └── com.example.plugin/
```

If plugin data is critical to the writer's project, the plugin should use a documented project-data mechanism rather than hiding important information exclusively inside caches.

Plugin storage rules are defined further in:

`005-PLUGIN-ARCHITECTURE.md`

---

# 49. Git Compatibility

Catalyst projects are intentionally compatible with Git.

Human-readable files allow meaningful version history for:

- Manuscripts
- Story Bible changes
- Timeline changes
- Plot development
- Templates
- Project configuration

Generated caches should normally be excluded from Git.

A recommended `.gitignore` may include:

```gitignore
.lithomark/cache/
.lithomark/index.db
.lithomark/logs/
.lithomark/recovery/
```

Revision-history rules will determine whether `.lithomark/history/` should be tracked.

---

# 50. Project Reconstruction

Catalyst must support reconstruction of generated state.

Conceptually:

```text
Open Project
     │
     ▼
Read lithomark.yaml
     │
     ▼
Scan Project Files
     │
     ▼
Read Stable IDs
     │
     ▼
Parse Metadata
     │
     ▼
Build Entity Graph
     │
     ▼
Build Timeline Index
     │
     ▼
Build Search Index
     │
     ▼
Build Continuity Index
     │
     ▼
Create/Rebuild SQLite Cache
     │
     ▼
Project Ready
```

This operation may occur automatically when:

- `index.db` is missing
- Index schema changes
- Index corruption is detected
- User requests rebuild
- Catalyst detects major filesystem changes

---

# 51. Project Integrity Check

Catalyst should provide a project integrity operation.

Potential checks include:

- Valid project manifest
- Supported project format
- Duplicate IDs
- Missing IDs
- Broken references
- Invalid YAML
- Missing files
- Invalid relative paths
- Missing assets
- Orphaned objects
- Timeline reference errors
- Canon reference errors

Integrity checking must not modify project content without user authorization.

---

# 52. Project Migration

When a newer Catalyst version requires a project-format migration:

1. Validate the existing project.
2. Create a backup or snapshot.
3. Perform migration.
4. Validate the migrated project.
5. Update `project_format`.
6. Report migration results.

A failed migration must not destroy the original project.

---

# 53. Portability Requirement

A complete Catalyst project should normally be movable by copying its root directory.

Example:

```text
All That Remains/
```

The copied project should remain recognizable on another supported Catalyst installation.

No registry entry, machine-specific database, cloud account, or hidden external metadata may be required to identify the project.

---

# 54. Archive Requirement

A writer should be able to archive a Catalyst project using ordinary filesystem tools.

For example:

```text
All That Remains.zip
```

The archive should contain sufficient authoritative information to restore the project.

Generated indexes need not be preserved.

---

# 55. Sync Compatibility

Catalyst projects may be stored in directories managed by external synchronization systems.

Examples may include:

- OneDrive
- Dropbox
- Google Drive
- Syncthing
- Git
- Network storage

Catalyst must not assume that it has exclusive permanent control over project files.

Conflict detection and safe writes are therefore architectural requirements.

Catalyst should not claim that all third-party synchronization systems are inherently safe for simultaneous multi-device editing.

---

# 56. Project Locking

Catalyst may use temporary project locks to reduce accidental simultaneous editing.

Locks must not become permanent barriers to accessing a writer's files.

Stale locks must be detectable and recoverable.

A lock must never make the underlying manuscript inaccessible outside Catalyst.

---

# 57. Human Readability Requirement

A technically competent user should be able to inspect a Catalyst project without Catalyst and understand its basic organization.

For example:

```text
Story Development/Characters/Main Cast/Leah Bennett.md
```

is preferred over:

```text
objects/7f/a8/2931.dat
```

Stable internal IDs provide machine identity.

Human-readable names provide writer usability.

Catalyst uses both.

---

# 58. Separation of Content and Application State

Authoritative project content includes things such as:

- Manuscript text
- Character information
- Timeline events
- Canon facts
- Plot information
- Research
- Notes
- User templates
- Project settings required to understand the project

Application state includes things such as:

- Which panel was open
- Window dimensions
- Last cursor position
- Temporary search results
- Cached word counts
- Rendered previews

Application state should not contaminate authoritative content unnecessarily.

---

# 59. Machine-Specific State

Machine-specific state should be stored separately from portable project content.

Examples include:

- Window position
- Monitor information
- Local editor preferences
- Temporary paths
- Local plugin runtime information

Portable project files must not require this information.

---

# 60. Core Design Rule

When deciding where information belongs, ask:

> If Catalyst disappeared tomorrow, would the writer need this information to understand, recover, continue, or publish the project?

If yes, it should normally exist in authoritative project storage.

If no, and Catalyst can regenerate it, it may belong in `.lithomark`.

---

# 61. Governing Storage Principle

Catalyst's internal databases serve the project.

The project does not serve Catalyst's databases.

The authoritative work remains:

**files the writer owns, in formats the writer can access, inside a directory the writer controls.**

---

# 62. Project Isolation and Series References

A project root is an ownership and namespace boundary.

Stable IDs are stable within their documented scope, but cross-project references must include enough information to identify both the target project and target entity. Catalyst must not resolve cross-project identity from a display name alone.

Example conceptual reference:

```yaml
project_id: prj-550e8400-e29b-41d4-a716-446655440000
entity_id: chr-a84f2931
```

A future Series Workspace may maintain its own ordinary, human-readable manifest and series-level files outside or above member project roots. Member projects remain normal Catalyst projects and must not require the series manifest to open.

Series membership must be explicit. Merely storing two projects beside each other on disk does not make them related and must not cause their indexes, autocomplete, canon, search results, or other project data to overlap.

Shared author-level resources such as application preferences, installed plugins, reusable templates, or optional narration runtimes are not project content and must not be mistaken for cross-project manuscript state.

---

# 62. Related Specifications

This specification operates under:

- `000-CATALYST-ARCHITECTURE.md`

Related specifications:

- `002-DOCUMENT-MODEL.md`
- `003-STABLE-ID-SYSTEM.md`
- `004-CANON-MODEL.md`
- `005-PLUGIN-ARCHITECTURE.md`
- `006-AGENT-API.md`
- `007-KINDLING-MIGRATION.md`

---

**END OF PROJECT FILE FORMAT SPECIFICATION**