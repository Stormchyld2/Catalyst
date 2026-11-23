# LithoMark Writer OS — Kindling Migration Plan

**Document:** 007-KINDLING-MIGRATION.md  
**Status:** Locked Baseline  
**Project:** LithoMark Writer OS  
**Parent Specification:** 000-LITHOMARK-ARCHITECTURE.md  
**Related Specifications:** 001-PROJECT-FILE-FORMAT.md, 002-DOCUMENT-MODEL.md, 003-STABLE-ID-SYSTEM.md, 004-CANON-MODEL.md, 005-PLUGIN-ARCHITECTURE.md, 006-AGENT-API.md

---

# 1. Purpose

This specification defines the engineering strategy for using the open-source Kindling application as a foundation for LithoMark Writer OS.

The objective is not to rename Kindling and continue its existing architecture unchanged.

The objective is to reuse mature, compatible components while progressively replacing architectural assumptions that conflict with LithoMark.

The migration must preserve working functionality wherever practical.

---

# 2. Migration Principle

LithoMark should evolve from Kindling through controlled architectural replacement.

Preferred strategy:

```text
Working Kindling Foundation
          │
          ▼
Introduce LithoMark Boundaries
          │
          ▼
Replace Incompatible Services
          │
          ▼
Migrate Existing Features
          │
          ▼
Add LithoMark Domain Systems
          │
          ▼
Remove Legacy Dependencies
          │
          ▼
LithoMark Writer OS
```

Avoid:

```text
Delete most of Kindling
        │
        ▼
Rewrite everything
        │
        ▼
Hope functionality returns
```

---

# 3. Foundation Assessment

The Kindling foundation provides substantial reusable engineering.

The audited architecture includes:

- Tauri 2
- Rust backend
- Svelte 5
- TypeScript
- TipTap 3
- SQLite
- Chapter management
- Scene management
- Beat management
- Rich-text editing
- References
- Revisions
- Editorial comments
- Tracked changes
- Writing goals
- Statistics
- Import systems
- Export systems
- Screenplay support
- Snapshots
- Automated testing
- End-to-end testing
- Visual testing

These capabilities make Kindling valuable as an implementation foundation.

---

# 4. Primary Architectural Conflict

Kindling's primary persistence architecture is database-centered.

Conceptually:

```text
Kindling UI
    │
    ▼
Tauri Commands
    │
    ▼
Rust Models
    │
    ▼
SQLite
```

LithoMark requires:

```text
LithoMark UI
      │
      ▼
LithoMark Domain API
      │
      ▼
Domain Services
      │
      ▼
Authoritative Project Files
      │
      ├── Markdown
      ├── YAML
      ├── JSON
      └── Assets
      │
      ▼
Rebuildable Index
      │
      ▼
SQLite
```

This is the central migration problem.

---

# 5. Migration Objective

The migration is complete when:

1. Writer-owned project data lives in ordinary project files.
2. SQLite contains only rebuildable or disposable derived data.
3. UI components operate through LithoMark domain services.
4. Editor content is independent from TipTap's internal format.
5. Stable IDs identify all significant objects.
6. Kindling-specific product identity has been removed.
7. Restricted Kindling design assets are not used.
8. Existing compatible functionality operates against LithoMark services.
9. LithoMark-specific systems can be added without reintroducing database lock-in.

---

# 6. Preserve Working Software

The migration should keep the application buildable and runnable at frequent checkpoints.

Each migration phase should aim to end with:

```text
Build passes
Tests pass
Application launches
Basic editing works
Project remains recoverable
```

Large untestable migration branches should be avoided.

---

# 7. Source Repository Strategy

The original Kindling source should remain available as an untouched reference.

Recommended structure:

```text
Development/
├── Kindling-Source/
└── LithoMark/
```

`Kindling-Source` remains useful for:

- Comparing original behavior
- Recovering implementations
- Reviewing upstream fixes
- Auditing license notices
- Understanding migration regressions

Development occurs in the LithoMark repository.

---

# 8. Upstream Preservation

The original Kindling repository should not be treated as disposable after migration begins.

Where practical, maintain an upstream Git reference.

Conceptually:

```text
origin
    → LithoMark repository

kindling-upstream
    → Original Kindling repository
```

This allows selected upstream changes to be reviewed later.

---

# 9. Fork Versus Import

LithoMark may begin from a fork or clean imported copy of Kindling.

The important requirement is preserving:

- Git history where useful
- License attribution
- Original copyright notices
- Clear LithoMark ownership of new work

A destructive copy-and-paste process that loses licensing provenance should be avoided.

---

# 10. Licensing Requirement

Applicable Kindling MIT license notices must be retained as required.

LithoMark may change:

- Product name
- Architecture
- Features
- UI
- Branding

but must comply with retained open-source licensing obligations.

---

# 11. Press Design System Restriction

The Kindling repository states that its MIT license does not grant rights to the Press design system, brand artwork, application icons, or `DESIGN_GUIDE.md`.

Therefore these elements are classified:

```text
REMOVE / REPLACE
```

LithoMark must develop its own:

- Visual identity
- Application icon
- Logo
- Design tokens
- Component styling
- Product branding

No assumption should be made that Kindling's design assets are reusable merely because the source code is MIT licensed.

---

# 12. Component Classification

Every significant Kindling subsystem should be classified as:

```text
KEEP
KEEP + MODIFY
REPLACE
REMOVE
NEW
```

This classification guides migration.

---

# 13. Technology Classification

| Component | Decision |
|---|---|
| Tauri 2 | KEEP |
| Rust | KEEP |
| Svelte 5 | KEEP |
| TypeScript | KEEP |
| TipTap 3 | KEEP + MODIFY |
| SQLite | KEEP + REPURPOSE |
| Kindling SQLite source-of-truth architecture | REPLACE |
| Kindling branding | REMOVE |
| Press design assets | REMOVE / REPLACE |

---

# 14. Feature Classification

| Feature | Decision |
|---|---|
| Chapters | KEEP + MODIFY |
| Scenes | KEEP + MODIFY |
| Beats | KEEP + MODIFY |
| Rich-text editor | KEEP + MODIFY |
| Autosave | MODIFY |
| References | KEEP + EXPAND |
| Custom fields | KEEP + EXPAND |
| Tags | KEEP + EXPAND |
| Revisions | KEEP + MODIFY |
| Comments | KEEP + MODIFY |
| Tracked changes | KEEP + MODIFY |
| Find/replace | KEEP |
| Goals | KEEP |
| Statistics | KEEP |
| Importers | KEEP + MODIFY |
| Exporters | KEEP + EXPAND |
| Screenplay support | KEEP |
| Snapshots | KEEP + MODIFY |
| Templates | REBUILD / EXPAND |
| Kindling updater | REPLACE |

---

# 15. LithoMark New Systems

The following systems are new:

```text
Project File Service
LithoMark Document Model
Stable ID Service
Story Bible
Canon Model
Continuity Engine
Timeline Model
Relationship Model
Plot Thread Model
Character Arc Model
Reveal Model
Research System
Plugin API
Plugin Permission System
External Agent API
LithoMark Template System
Project Integrity System
```

---

# 16. Migration Phases

The migration should occur in explicit phases.

Recommended phases:

```text
Phase 0 — Baseline and Repository Setup
Phase 1 — De-Branding and Licensing
Phase 2 — Domain Boundary
Phase 3 — Stable Identity
Phase 4 — File-Based Project Format
Phase 5 — Document Model and Editor
Phase 6 — SQLite Repurposing
Phase 7 — Existing Feature Migration
Phase 8 — Story Bible
Phase 9 — Timeline and Canon
Phase 10 — Continuity
Phase 11 — Story Development Systems
Phase 12 — Revision and Recovery
Phase 13 — Plugin Foundation
Phase 14 — External Agent Foundation
Phase 15 — Import/Export Hardening
Phase 16 — Productization
```

---

# 17. Phase 0 — Baseline

Before changing Kindling:

1. Clone or import source.
2. Install dependencies.
3. Build application.
4. Run automated tests.
5. Launch application.
6. Create test project.
7. Verify editing.
8. Verify save/load.
9. Record known failures.
10. Commit baseline.

Tag recommended:

```text
kindling-baseline
```

No architecture migration should begin until the baseline builds successfully.

---

# 18. Baseline Documentation

Record:

```text
Rust version
Node version
Package manager version
Tauri version
Operating system
Build commands
Test commands
Known warnings
Known failures
```

This prevents migration bugs from being confused with preexisting issues.

---

# 19. Phase 1 — De-Branding and Licensing

Early migration should establish independent product identity.

Replace:

```text
Kindling
```

with:

```text
LithoMark
```

where the text represents product identity rather than required attribution.

---

# 20. Do Not Blindly Global Replace

A global text replacement is unsafe.

Occurrences of `Kindling` may represent:

- Product branding
- Package names
- Rust modules
- Database identifiers
- Migration identifiers
- Documentation
- License notices

Each category should be reviewed appropriately.

---

# 21. Attribution Preservation

Required upstream license notices must remain intact.

Product branding and legal attribution are separate concerns.

LithoMark must not remove legally required attribution during de-branding.

---

# 22. Remove Restricted Assets

Remove or replace:

- Kindling application icons
- Kindling brand artwork
- Press-specific visual assets
- Restricted design documentation
- Press-specific design dependencies where applicable

Temporary neutral styling is acceptable during migration.

Architecture matters more than polished branding at this stage.

---

# 23. LithoMark Application Identity

Update appropriate application metadata:

```text
Application name
Window title
Package metadata
Bundle identifiers
Executable metadata
About information
Repository references
Update endpoints
```

Final production identifiers may be selected later if domains or organization identifiers are not yet finalized.

---

# 24. Phase 2 — Introduce Domain Boundary

This is the most important structural refactor before changing storage.

Current pattern:

```text
UI
 ↓
Database Commands
 ↓
SQLite
```

Target intermediate pattern:

```text
UI
 ↓
LithoMark Domain Commands
 ↓
Domain Services
 ↓
Existing Kindling Database Repository
 ↓
SQLite
```

At this stage SQLite may still be authoritative.

The goal is first to isolate it.

---

# 25. Why Domain Boundary Comes First

Replacing persistence while UI components directly depend upon database commands creates unnecessary risk.

Instead:

```text
First:
Separate UI from persistence.

Then:
Replace persistence behind the domain boundary.
```

This is a classic strangler migration strategy.

---

# 26. Service Architecture

Initial service boundaries should include:

```text
ProjectService
DocumentService
ManuscriptService
ReferenceService
RevisionService
SearchService
ExportService
ImportService
```

Later:

```text
StoryBibleService
TimelineService
CanonService
ContinuityService
PluginService
AgentService
```

---

# 27. Repository Abstraction

Existing database operations should initially be wrapped behind repository interfaces.

Conceptually:

```text
DocumentService
      │
      ▼
DocumentRepository
      │
      ▼
KindlingSqliteRepository
```

Later:

```text
DocumentService
      │
      ▼
DocumentRepository
      │
      ▼
LithoMarkFileRepository
```

This allows storage migration without rewriting every UI component.

---

# 28. Phase 3 — Stable Identity

Introduce the Stable ID Service defined in:

`003-STABLE-ID-SYSTEM.md`

New and migrated significant objects receive stable IDs.

Initial priority:

```text
Project
Chapter
Scene
Beat
Reference
Comment
Revision
```

Later objects use the same service.

---

# 29. Existing Kindling IDs

Existing Kindling database IDs should not automatically become permanent LithoMark identity.

If existing IDs are implementation-specific database keys, generate LithoMark stable IDs.

Maintain migration mapping while needed.

Example:

```text
Kindling scene ID:
42

LithoMark:
scn-df925d68-d3ef-4d90-bfa4-6fa7834203c7
```

---

# 30. ID Migration Table

During migration, temporary mapping may be stored:

```text
kindling_type
kindling_id
lithomark_id
```

All relationships must be migrated through this map.

---

# 31. Phase 4 — File-Based Project Format

Implement:

`001-PROJECT-FILE-FORMAT.md`

Create:

```text
ProjectService
ProjectManifest
ProjectScanner
ProjectValidator
FileRepository
```

LithoMark should become capable of creating an empty file-based project before full editor migration.

---

# 32. Project Creation

New LithoMark project creation should produce:

```text
Project/
├── lithomark.yaml
├── Manuscript/
├── Story Bible/
├── Timeline/
├── Story/
├── Research/
├── Notes/
├── Templates/
├── Assets/
└── .lithomark/
```

Unused folders may be created lazily if desired.

---

# 33. Project Opening

Project opening should eventually follow:

```text
Select Project Folder
        │
        ▼
Read lithomark.yaml
        │
        ▼
Validate Format
        │
        ▼
Scan Project Files
        │
        ▼
Build Object Map
        │
        ▼
Validate References
        │
        ▼
Build/Rebuild Index
        │
        ▼
Open Workspace
```

---

# 34. Project Scanner

The Project Scanner must discover authoritative LithoMark objects from project files.

It should identify:

- File path
- Stable ID
- Type
- Metadata
- Relationships
- Content hash

The scanner feeds derived indexes.

---

# 35. Dual Persistence Transition

A temporary migration period may require both SQLite and files.

This must be tightly controlled.

Possible temporary architecture:

```text
Domain Service
      │
      ├── Legacy SQLite Repository
      └── LithoMark File Repository
```

This is migration infrastructure only.

It must not become permanent architecture.

---

# 36. Single Authority During Transition

At every migration phase, one system must be designated authoritative.

Avoid ambiguous bidirectional truth.

Example transitional rule:

```text
Legacy projects:
SQLite authoritative

New LithoMark test projects:
Files authoritative
```

Then migrate legacy projects explicitly.

---

# 37. Phase 5 — Document Model

Implement:

`002-DOCUMENT-MODEL.md`

Create a LithoMark document abstraction independent from TipTap.

Conceptually:

```text
Project File
    │
    ▼
Parser
    │
    ▼
LithoMarkDocument
    │
    ▼
Editor Adapter
    │
    ▼
TipTap
```

---

# 38. Markdown Parser

Implement parsing for:

- YAML front matter
- Markdown body
- Stable IDs
- Document type
- Metadata
- Relationships

Parsing must preserve unknown metadata where practical.

---

# 39. Serializer

Implement:

```text
LithoMarkDocument
      │
      ▼
Serializer
      │
      ▼
YAML + Markdown
```

Round-trip tests are mandatory.

---

# 40. Editor Adapter

Existing TipTap functionality should be retained behind an Editor Adapter.

The editor should no longer assume its internal representation is permanent storage.

---

# 41. TipTap JSON Migration

If Kindling currently stores TipTap JSON or related rich-text structures in SQLite, migration must convert them to LithoMark's authoritative document representation.

Conversion must be tested against:

- Paragraphs
- Headings
- Emphasis
- Strong emphasis
- Lists
- Scene breaks
- Links
- Other supported marks/nodes

Unsupported structures must not be silently discarded.

---

# 42. Autosave Migration

Current autosave behavior must evolve from:

```text
Editor
 ↓
Database
```

to:

```text
Editor
 ↓
Document Model
 ↓
Safe File Write
 ↓
Index Update
```

The safe-write requirements from `001-PROJECT-FILE-FORMAT.md` apply.

---

# 43. Phase 6 — Repurpose SQLite

Once authoritative files function correctly, SQLite becomes derived infrastructure.

SQLite may contain:

```text
Object index
Search index
Full-text search
Relationship index
Statistics
Word counts
Derived continuity data
UI acceleration data
```

SQLite must no longer contain irreplaceable writer-owned content.

---

# 44. Index Rebuild

Implement:

```text
Rebuild Project Index
```

This operation should:

1. Delete or replace index database.
2. Scan project files.
3. Parse objects.
4. Validate IDs.
5. Resolve references.
6. Build search index.
7. Build relationship index.
8. Build derived statistics.
9. Report errors.

---

# 45. Destructive Index Test

A critical architecture test:

```text
Close LithoMark
Delete .lithomark/index.db
Open LithoMark
```

Expected result:

```text
Project rebuilds successfully.
No writer content is lost.
```

This should become an automated integration test where practical.

---

# 46. Phase 7 — Existing Feature Migration

Once file persistence and document abstraction work, migrate Kindling features one subsystem at a time.

Recommended order:

```text
1. Chapters
2. Scenes
3. Beats
4. References
5. Tags
6. Custom fields
7. Search
8. Goals
9. Statistics
10. Comments
11. Tracked changes
12. Revisions
13. Snapshots
14. Screenplay support
15. Import
16. Export
```

---

# 47. Chapter Migration

Chapters become file-based structural objects.

Chapter ordering must use stable IDs.

Database row order must not remain authoritative.

---

# 48. Scene Migration

Scenes become Markdown documents.

Each scene receives:

```yaml
lithomark:
  type: scene
  id: scn-UUID
```

Scene metadata migrates into documented YAML fields.

---

# 49. Beat Migration

Beats may be represented as:

- Structured scene metadata
- Separate objects
- Dedicated planning files

The final model should preserve existing useful Kindling behavior while conforming to stable IDs and file ownership.

---

# 50. References Migration

Kindling references provide a useful starting point for Story Bible architecture.

Existing reference concepts should be generalized into domain entities.

Migration may transform:

```text
Kindling Reference
```

into:

```text
LithoMark Story Bible Object
```

where appropriate.

---

# 51. Custom Fields

Custom fields should become structured YAML metadata.

Unknown custom fields must survive parsing and serialization.

---

# 52. Tags

Tags should be portable project metadata rather than database-only values.

Tag indexing may remain in SQLite.

---

# 53. Comments

Comments must move from database/editor-specific storage into the LithoMark annotation model.

They should use resilient document anchors.

---

# 54. Tracked Changes

Tracked changes should become domain-level objects rather than TipTap-only state.

Accept/reject behavior should continue to operate through the Document Service.

---

# 55. Goals

Writing goals may remain project metadata or application state depending upon their meaning.

Writer-defined durable goals should be portable where practical.

Derived progress remains cacheable.

---

# 56. Statistics

Statistics are derived.

They may remain primarily SQLite-backed after file migration.

Deleting the statistics cache must not affect manuscript content.

---

# 57. Phase 8 — Story Bible

Expand migrated references into the LithoMark Story Bible.

Initial object types:

```text
Characters
Locations
Organizations
Objects
Events
Relationships
Procedures
Custom
```

Story Bible files become authoritative.

---

# 58. Story Bible UI

Existing Kindling reference UI may be adapted where useful.

The underlying data model must use LithoMark domain services rather than Kindling database records.

---

# 59. Phase 9 — Timeline

Implement the Timeline Model.

Timeline events receive stable IDs.

Support:

```text
Exact dates
Approximate dates
Ranges
Relative dates
Unknown dates
```

Timeline data must integrate with manuscript scenes and Story Bible objects.

---

# 60. Canon

Implement:

`004-CANON-MODEL.md`

Initial canon support should prioritize high-value deterministic facts:

```text
Birth
Death
Age
Alias
Occupation
Location
Relationship
Injury
Possession
Knowledge
Status
```

---

# 61. Character Knowledge

Knowledge tracking should be included early in Canon development because it provides major value for:

- Crime
- Mystery
- Thriller
- Suspense
- Reveal-heavy fiction

It also demonstrates the value of LithoMark's structured architecture.

---

# 62. Phase 10 — Continuity Engine

Build deterministic continuity rules over:

```text
Stable IDs
Timeline
Canon
Scene metadata
Relationships
Character knowledge
```

Initial rules may include:

```text
Character after death
Knowledge before discovery
Age mismatch
Location overlap
Possession conflict
Relationship-date conflict
Alias timing conflict
```

---

# 63. Continuity Is Not AI

The initial Continuity Engine should be fully deterministic.

This makes it:

- Predictable
- Testable
- Offline
- Explainable
- Privacy-preserving

---

# 64. Phase 11 — Story Development

Add first-class systems for:

```text
Plot Threads
Character Arcs
Reveals
Clues
Foreshadowing
Payoffs
Beat Sheets
Story Structures
```

These systems use stable IDs and project files.

---

# 65. Reveal System

Reveals should connect:

```text
Underlying Canon Fact
        │
        ▼
Reveal
        │
        ▼
Character Knowledge Change
        │
        ▼
Scene / Timeline Event
```

This provides powerful deterministic story-state tracking.

---

# 66. Phase 12 — Revision and Recovery

Migrate and strengthen Kindling revision/snapshot capabilities.

LithoMark Core must provide:

```text
Autosave
Revision history
Named snapshots
Recovery
Backup assistance
```

Git remains optional.

---

# 67. Recovery Testing

Test:

- Application crash
- Interrupted write
- Invalid Markdown
- Invalid YAML
- External modification
- Conflicted file
- Missing index
- Deleted cache
- Partial migration

Writer content must remain recoverable.

---

# 68. Phase 13 — Plugin Foundation

Only after the Domain API is sufficiently stable should public plugin infrastructure begin.

Implement according to:

`005-PLUGIN-ARCHITECTURE.md`

Initial steps:

```text
Event system
Permission registry
Plugin manifest
Plugin loader
Plugin API v1
Developer mode
Local plugin loading
```

---

# 69. Do Not Build Registry First

The Community Plugin Registry should not precede a stable local plugin system.

Correct order:

```text
Plugin API
   ↓
Local plugins
   ↓
Real plugin development
   ↓
API stabilization
   ↓
Packaging
   ↓
Registry
```

---

# 70. Phase 14 — External Agent Foundation

After domain services and permissions are stable, implement:

`006-AGENT-API.md`

Initial Agent API should focus on:

```text
Read project
Read manuscript
Read Story Bible
Read timeline
Read canon
Read continuity
Create notes
Create proposals
```

Direct write access may follow.

---

# 71. No AI Dependency

Agent API implementation must not introduce:

- Required AI SDK
- Required AI account
- Required cloud service
- Embedded generative model

The API remains provider-neutral.

---

# 72. Phase 15 — Import Hardening

Existing Kindling importers should be adapted to produce LithoMark domain objects and files.

Potential import sources already supported by Kindling should be preserved where practical.

Examples include:

```text
Scrivener
Plottr
yWriter
novelWriter
Obsidian
```

---

# 73. Import Pipeline

Target:

```text
External Project
      │
      ▼
Importer
      │
      ▼
LithoMark Domain Objects
      │
      ▼
Stable ID Assignment
      │
      ▼
Validation
      │
      ▼
Authoritative Project Files
      │
      ▼
Index Build
```

---

# 74. Import Safety

Import should normally create a new LithoMark project rather than destructively modifying the source project.

Original external project files remain untouched.

---

# 75. Export Hardening

Existing Kindling export functionality should be adapted to consume the LithoMark Document Model.

Exporters must not depend directly upon:

- SQLite schema
- TipTap JSON
- Svelte UI state

---

# 76. Phase 16 — Productization

Only after architecture migration is stable should development focus heavily on final product polish.

Productization includes:

```text
LithoMark visual identity
Application icons
Installer
Updater
First-run experience
Settings
Documentation
Crash reporting policy
Release channels
Plugin browser
Accessibility polish
Performance optimization
```

---

# 77. Updater Replacement

Kindling-specific update infrastructure must be replaced.

LithoMark requires its own:

- Update endpoints
- Signing
- Release metadata
- Release channels

No Kindling production update infrastructure should remain active.

---

# 78. Migration of Existing Kindling Projects

If LithoMark eventually supports importing Kindling projects, use an explicit migration wizard.

Conceptually:

```text
Select Kindling Project
        │
        ▼
Read Legacy Database
        │
        ▼
Create New LithoMark Project Folder
        │
        ▼
Generate Stable IDs
        │
        ▼
Convert Documents
        │
        ▼
Convert Metadata
        │
        ▼
Convert Relationships
        │
        ▼
Write Markdown/YAML
        │
        ▼
Validate
        │
        ▼
Build Index
        │
        ▼
Migration Report
```

---

# 79. Never Destroy Legacy Project During Migration

Migration must not modify the original Kindling project destructively.

Create a new LithoMark project.

The original remains available as fallback.

---

# 80. Migration Report

After migration, LithoMark should report:

```text
Projects migrated
Chapters migrated
Scenes migrated
Words migrated
References migrated
Comments migrated
Tracked changes migrated
Unsupported features
Warnings
Errors
```

This makes migration auditable.

---

# 81. Unsupported Data

Unsupported legacy data must not be silently discarded.

Options include:

```text
Preserve as raw migration data
Convert to note
Record in migration report
Require user review
```

---

# 82. Migration Fixtures

Create test fixtures representing Kindling projects with:

- Simple novel
- Large novel
- Comments
- Tracked changes
- References
- Screenplay
- Custom fields
- Tags
- Import-derived content

These fixtures should be used repeatedly during migration development.

---

# 83. Round-Trip Tests

For migrated documents:

```text
Kindling content
      │
      ▼
Migration
      │
      ▼
LithoMark file
      │
      ▼
LithoMark parser
      │
      ▼
Editor
```

The resulting content must preserve intended manuscript meaning and supported formatting.

---

# 84. Test Strategy

Testing should operate at multiple levels:

```text
Unit Tests
Integration Tests
Migration Tests
End-to-End Tests
Visual Tests
Recovery Tests
Performance Tests
```

Kindling's existing testing infrastructure should be preserved where useful.

---

# 85. Golden Files

Markdown serialization and migration should use golden-file tests.

Example:

```text
Input Kindling fixture
        │
        ▼
Migration
        │
        ▼
Expected Markdown/YAML
```

Unexpected differences fail the test.

---

# 86. Performance Baseline

Before major migration, record baseline performance where practical:

```text
Startup time
Project open time
Scene switch time
Search time
Autosave latency
Export time
Memory usage
```

LithoMark should avoid unnecessary regressions.

---

# 87. Large Project Testing

Test projects should include:

```text
100+ chapters
1,000+ scenes
Large Story Bible
Thousands of canon facts
Large timeline
Large research collection
```

File-based architecture must remain responsive.

---

# 88. Indexing Strategy

File scanning should not require reparsing the entire project after every small edit.

Derived indexes may use:

```text
Modification times
Content hashes
Known object IDs
Incremental indexing
```

Full rebuild remains available.

---

# 89. File Watcher

External file changes must update the domain layer and indexes.

Test with:

- VS Code
- Obsidian
- Git operations
- File Explorer rename
- Cloud-sync changes

LithoMark must not assume exclusive ownership of project files.

---

# 90. Git Compatibility

File output should minimize unnecessary churn.

Avoid rewriting YAML or Markdown files when nothing meaningful changed.

Stable serialization improves Git diffs.

---

# 91. Formatting Stability

The serializer should preserve formatting where practical.

Opening and closing a project should not rewrite hundreds of files.

This is important for:

- Git
- Cloud sync
- Backups
- Writer trust

---

# 92. Architecture Enforcement

Code review should reject new features that bypass established boundaries.

Examples of prohibited patterns:

```text
Svelte → SQLite directly

Plugin → SQLite directly

Agent → SQLite directly

UI → filesystem directly for domain documents

TipTap JSON → permanent manuscript source

Canon → cache only
```

---

# 93. Required Direction

New development should follow:

```text
Presentation
     │
     ▼
Domain API
     │
     ▼
Services
     │
     ▼
Persistence
```

---

# 94. Migration Technical Debt

Temporary migration code must be clearly marked.

Examples:

```text
legacy
migration
compat
temporary
```

Every temporary compatibility layer should have an identified removal condition.

---

# 95. Legacy Removal

Do not remove a legacy subsystem merely because the replacement exists.

Remove it when:

```text
Replacement implemented
Replacement tested
Data migration tested
Dependent features migrated
Rollback path understood
```

Then remove the old path.

---

# 96. Feature Flags

Feature flags may be useful during migration.

Example:

```text
file_project_format
new_document_model
new_story_bible
new_canon_engine
```

Flags should be removed when migration stabilizes.

---

# 97. Development Branch Strategy

Migration should use focused branches.

Examples:

```text
arch/domain-layer
arch/stable-ids
arch/file-projects
arch/document-model
feature/story-bible
feature/canon
feature/plugins
```

Avoid one enormous migration branch containing months of unrelated work.

---

# 98. Commit Strategy

Commits should be small enough to explain architectural movement.

Good:

```text
Add ProjectService abstraction

Route scene reads through DocumentService

Add stable IDs to scene model

Implement Markdown scene serializer
```

Avoid commits equivalent to:

```text
Rewrite app
```

---

# 99. Architectural Decision Records

Important deviations from documents `000` through `007` should be recorded before implementation where practical.

Recommended directory:

```text
docs/
└── architecture/
    └── decisions/
```

Example:

```text
ADR-001-markdown-parser.md
ADR-002-agent-api-transport.md
```

---

# 100. Documentation Must Track Implementation

Architecture documents are not decorative.

When implementation intentionally changes a locked architectural decision:

1. Document the reason.
2. Review consequences.
3. Update affected specification.
4. Implement change.
5. Update tests.

---

# 101. First Development Milestone

The first major LithoMark architecture milestone should be:

```text
LithoMark can create a project folder,
create a chapter,
create a scene,
edit the scene in TipTap,
save the scene as Markdown + YAML,
close the application,
delete index.db,
reopen the project,
rebuild the index,
and recover the scene perfectly.
```

This proves the central architecture.

---

# 102. Second Development Milestone

After the first milestone:

```text
Rename scene externally.
Move scene externally.
Edit scene in external editor.
Reopen/rescan in LithoMark.
Preserve stable identity.
Detect external changes.
```

This proves file ownership and portability.

---

# 103. Third Development Milestone

Next:

```text
Create Story Bible character.
Assign stable ID.
Reference character from scene.
Rename character.
Verify scene relationship remains intact.
```

This proves the Stable ID architecture.

---

# 104. Fourth Development Milestone

Next:

```text
Create timeline event.
Create canon fact.
Attach character.
Attach scene.
Run deterministic continuity rule.
Produce explainable warning.
```

This proves LithoMark's differentiated story-intelligence architecture.

---

# 105. Fifth Development Milestone

Next:

```text
Load local test plugin.
Grant limited permissions.
Read project metadata.
Add sidebar panel.
Verify manuscript access is denied.
```

This proves the Plugin API security boundary.

---

# 106. Sixth Development Milestone

Next:

```text
Enable local Agent API.
Authorize test client.
Grant read-only Story Bible access.
Verify manuscript access is denied.
Revoke client.
Verify further access fails.
```

This proves the external integration boundary.

---

# 107. Minimum Viable LithoMark

A meaningful initial LithoMark release should include:

```text
File-based projects
Markdown manuscript storage
Chapter/scene management
Rich editor
Stable IDs
Story Bible
Timeline
Basic canon
Basic continuity
Search
Revision history
Snapshots
Import
Export
Backup/recovery
```

Plugin and Agent systems may reach stable public availability after the core architecture proves reliable.

---

# 108. What Not to Build First

Do not prioritize these before the storage/domain architecture works:

```text
Plugin marketplace
Cloud synchronization
Mobile application
Complex collaboration
AI integration
Elaborate visual redesign
Advanced agent automation
```

The foundation must come first.

---

# 109. Highest-Risk Migration Areas

The highest technical risks are:

```text
SQLite → authoritative files
TipTap → Markdown round-trip fidelity
Comments/tracked-change anchoring
External file conflict handling
Stable ID migration
Legacy project conversion
Large-project indexing performance
```

These areas require disproportionate testing.

---

# 110. Lowest-Risk Reuse Areas

Likely lower-risk reuse areas include:

```text
Tauri shell
Rust infrastructure
Svelte application framework
TipTap editing interface
General UI interaction patterns
Import parser logic
Export transformation logic
Testing infrastructure
```

These should be reused where licensing and architecture permit.

---

# 111. Stop Conditions

A migration phase should not advance if it introduces known risks of manuscript loss.

Examples:

```text
Serializer drops formatting
Autosave can overwrite external changes
Duplicate IDs are ignored
Migration silently discards records
Index cannot rebuild
Failed write destroys previous file
```

These are blocking defects.

---

# 112. Writer Data Priority

During every engineering decision:

```text
Protect writer data
      >
Preserve architectural cleanliness
      >
Preserve legacy implementation convenience
```

Architecture exists to protect and empower the writer.

---

# 113. Success Criteria

The Kindling migration succeeds when Kindling is no longer architecturally necessary to understand LithoMark.

At that point:

- Kindling supplied useful starting code.
- LithoMark owns its domain model.
- LithoMark owns its project format.
- LithoMark owns its UI identity.
- LithoMark owns its extension architecture.
- LithoMark owns its external integration boundary.
- Writer files remain independent from both applications.

---

# 114. Final Target Architecture

The final target is:

```text
                    LithoMark Writer OS

                    ┌───────────────┐
                    │ LithoMark UI  │
                    └───────┬───────┘
                            │
                    ┌───────▼───────┐
                    │  Domain API   │
                    └───────┬───────┘
                            │
          ┌─────────────────┼─────────────────┐
          │                 │                 │
    ┌─────▼─────┐     ┌─────▼─────┐     ┌────▼─────┐
    │ Manuscript│     │Story/Canon│     │ Research │
    │ Services  │     │ Services  │     │ Services │
    └─────┬─────┘     └─────┬─────┘     └────┬─────┘
          │                 │                 │
          └─────────────────┼─────────────────┘
                            │
                    ┌───────▼────────┐
                    │ Project Service│
                    └───────┬────────┘
                            │
                    ┌───────▼────────┐
                    │ Project Files  │
                    │ MD/YAML/JSON   │
                    └───────┬────────┘
                            │
                    ┌───────▼────────┐
                    │Derived Indexes │
                    │    SQLite      │
                    └────────────────┘

      ┌──────────────────┐       ┌──────────────────┐
      │    Plugin API    │       │    Agent API     │
      └────────┬─────────┘       └────────┬─────────┘
               │                          │
               └──────────┬───────────────┘
                          │
                  Permission Layer
```

---

# 115. Governing Migration Principle

Kindling is the foundation.

It is not the architecture LithoMark must remain bound to.

Reuse what is mature.

Replace what conflicts with writer ownership.

Preserve working functionality.

Introduce boundaries before replacing implementations.

Never sacrifice manuscript safety for migration speed.

The end state must be recognizably and architecturally LithoMark.

---

# 116. Architecture Set

This document completes the initial LithoMark architecture set:

```text
000-LITHOMARK-ARCHITECTURE.md
001-PROJECT-FILE-FORMAT.md
002-DOCUMENT-MODEL.md
003-STABLE-ID-SYSTEM.md
004-CANON-MODEL.md
005-PLUGIN-ARCHITECTURE.md
006-AGENT-API.md
007-KINDLING-MIGRATION.md
```

These documents collectively define the baseline architecture for implementation.

Future specifications should extend this architecture rather than bypass it.

---

**END OF KINDLING MIGRATION PLAN**