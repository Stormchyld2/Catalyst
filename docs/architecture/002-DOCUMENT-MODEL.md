# Catalyst — Document Model

**Document:** 002-DOCUMENT-MODEL.md  
**Status:** Locked Baseline  
**Project:** Catalyst  
**Parent Specification:** 000-CATALYST-ARCHITECTURE.md  
**Related Specification:** 001-PROJECT-FILE-FORMAT.md

---

# 1. Purpose

This specification defines how Catalyst represents, loads, edits, validates, saves, and exchanges textual documents.

The document model separates four concerns:

1. Authoritative project files
2. Catalyst domain documents
3. Editor representation
4. Rendered or exported output

The editor must not become the permanent definition of a Catalyst document.

---

# 2. Fundamental Architecture

The document pipeline is:

```text
Authoritative Project File
          │
          ▼
      File Parser
          │
          ▼
Catalyst Document Model
          │
          ▼
      Editor Adapter
          │
          ▼
        TipTap
          │
          ▼
   User Edits Document
          │
          ▼
      Editor Adapter
          │
          ▼
Catalyst Document Model
          │
          ▼
      Serializer
          │
          ▼
Authoritative Project File
```

TipTap is an implementation component.

The Catalyst Document Model is the application contract.

---

# 3. Authoritative Format

Markdown is the default authoritative textual format for Catalyst documents.

YAML front matter contains structured document metadata.

Example:

```markdown
---
lithomark:
  type: scene
  id: scn-7c42be89

title: Gallery Arrival
chapter: ch-01a82c74
pov: chr-a84f2931
status: draft
---

Leah stepped through the gallery doors.

The smell reached her first.
```

The file remains usable outside Catalyst.

---

# 4. Document Identity

Every structured Catalyst document must have a stable ID.

Example:

```yaml
lithomark:
  type: scene
  id: scn-7c42be89
```

The ID identifies the document.

The following do not:

- Filename
- File path
- Title
- Chapter number
- Display order

Renaming or moving a document must not change its stable ID.

---

# 5. Document Types

The document model must support multiple domain document types.

Initial types include:

```text
manuscript
part
chapter
scene
beat
character
location
organization
object
event
relationship
plot-thread
character-arc
reveal
clue
foreshadowing
payoff
research
note
template
```

Additional types may be introduced by future Catalyst project-format versions.

Plugins may register namespaced extension types through the Plugin API.

---

# 6. Domain Document

A Catalyst domain document conceptually contains:

```text
CatalystDocument
│
├── identity
├── type
├── metadata
├── content
├── relationships
├── revision state
└── source information
```

Conceptual representation:

```text
Document
  id
  type
  title
  metadata
  body
  references
  source_path
  source_revision
```

The exact Rust and TypeScript implementations may differ.

The semantic contract must remain consistent.

---

# 7. Metadata

Metadata is structured information describing a document.

Examples include:

- Stable ID
- Document type
- Title
- Status
- POV
- Chapter
- Timeline event
- Tags
- Relationships
- Custom fields

Metadata belongs in YAML front matter where it is authoritative project information.

---

# 8. Body Content

The Markdown body contains writer-facing textual content.

For manuscript documents, this is primarily prose.

For Story Bible documents, the body may contain descriptive notes.

For research documents, it may contain research notes and source commentary.

Example:

```markdown
# Leah Bennett

## Background

Leah joined NOPD after completing...
```

---

# 9. Structured Data Versus Prose

Information required for deterministic application behavior should be structured.

For example:

```yaml
birth_date: 1987-04-16
```

is preferable to relying exclusively upon:

```text
Leah was born on April 16, 1987.
```

The prose may still contain that information.

However, continuity systems operate on explicit structured facts rather than attempting to infer truth from prose.

---

# 10. Editor Independence

Catalyst must not permanently store manuscript content exclusively as TipTap JSON.

TipTap may use its own internal document representation while editing.

That representation is temporary application state.

The durable representation is the Catalyst project document.

This ensures that:

- TipTap may be upgraded.
- TipTap may be replaced.
- Documents remain accessible externally.
- Plugins do not depend directly on TipTap internals.
- Future mobile editors can use another implementation.

---

# 11. Editor Adapter

The Editor Adapter translates between:

```text
Catalyst Document Model
```

and:

```text
TipTap Document Model
```

The adapter is responsible for:

- Parsing supported Markdown structures
- Creating TipTap nodes and marks
- Converting editor changes back into Catalyst content
- Preserving supported metadata
- Preserving unknown metadata where practical
- Preventing editor-specific implementation details from leaking into project files

---

# 12. Markdown Compatibility

Catalyst should support common Markdown constructs where appropriate.

Initial support should include:

- Paragraphs
- Headings
- Emphasis
- Strong emphasis
- Lists
- Blockquotes
- Horizontal rules
- Links
- Images
- Code blocks
- Inline code

Manuscript writing may use only a subset during normal operation.

Research and notes may use a broader Markdown feature set.

---

# 13. Extended Markdown

Catalyst may define extensions to standard Markdown.

Extensions must be:

- Documented
- Parseable
- Human-readable where practical
- Gracefully degradable outside Catalyst

Catalyst should avoid inventing custom syntax when standard Markdown or YAML metadata can represent the same information adequately.

---

# 14. Rich Formatting

Some writing features may require formatting that does not map perfectly to basic Markdown.

Examples include:

- Comments
- Tracked changes
- Inline annotations
- Special manuscript marks
- Editorial suggestions

These features must not force the manuscript itself into an opaque proprietary binary format.

Where possible, such information should be stored separately from the prose.

---

# 15. Sidecar Metadata

Document-specific structured information that would unnecessarily pollute manuscript Markdown may be stored in documented sidecar files.

Example:

```text
Scene 01.md
Scene 01.lithomark.yaml
```

or through an ID-based metadata structure defined by a later specification.

Sidecars may be appropriate for:

- Editorial comments
- Tracked-change metadata
- Complex annotations
- Revision information

The exact implementation must preserve portability and recoverability.

---

# 16. Annotation Anchoring

Comments and annotations must not depend solely upon character offsets.

Character offsets become invalid whenever earlier text changes.

Catalyst should use resilient anchors capable of relocating annotations after reasonable edits.

An annotation anchor may include:

```text
document_id
selected_text
prefix_context
suffix_context
structural_location
```

Example concept:

```yaml
document: scn-7c42be89

anchor:
  selected_text: "The smell reached her first."
  prefix: "gallery doors."
  suffix: "She stopped."
```

The exact anchoring algorithm may evolve.

---

# 17. Editorial Comments

Comments are attached to documents through annotation anchors.

A comment conceptually contains:

```text
id
document_id
anchor
author
created
modified
status
body
```

Possible status values include:

```text
open
resolved
archived
```

Comments should remain available even if the editor implementation changes.

---

# 18. Tracked Changes

Tracked changes may include:

```text
insert
delete
replace
```

A tracked change must preserve enough information to:

- Display the proposed change
- Accept the change
- Reject the change
- Identify its document
- Maintain revision history

Tracked changes should be modeled at the Catalyst domain level rather than existing exclusively as TipTap state.

---

# 19. Autosave

Editor changes may initially exist in memory.

Catalyst must periodically serialize changed documents to authoritative project files.

Conceptually:

```text
User types
    │
    ▼
Editor state changes
    │
    ▼
Document Model updates
    │
    ▼
Document marked dirty
    │
    ▼
Autosave debounce
    │
    ▼
Serialize
    │
    ▼
Safe file write
    │
    ▼
Document marked saved
```

---

# 20. Dirty State

Catalyst must distinguish between:

```text
Clean
Dirty
Saving
Saved
Conflict
Error
```

The UI should provide appropriate feedback without distracting the writer.

---

# 21. Safe Saving

Saving an authoritative document should use the safe-write strategy established in `001-PROJECT-FILE-FORMAT.md`.

Conceptually:

```text
Serialize Document
       │
       ▼
Write Temporary File
       │
       ▼
Validate
       │
       ▼
Atomic Replace
       │
       ▼
Update File Revision
```

A failed write must not destroy the previously valid file.

---

# 22. Source Revision

Catalyst should track whether the underlying source file changed after loading.

A source revision may use:

- File modification time
- File size
- Content hash
- Combination of these methods

Before overwriting an externally modified file, Catalyst must detect the change.

---

# 23. External Modification

If another application changes an open Catalyst document, Catalyst must not silently overwrite it.

Possible responses include:

```text
Reload external version
Keep Catalyst version
Compare versions
Merge
Save both
```

Where automatic reconciliation is unsafe, the writer decides.

---

# 24. File Watcher

Catalyst should monitor authoritative project directories for relevant changes.

The file watcher may detect:

```text
create
modify
rename
move
delete
```

Changes should update the domain model and rebuild derived indexes as necessary.

---

# 25. Rename Handling

A renamed file retains the same document identity if its stable ID remains unchanged.

Example:

```text
Scene 01.md
```

becomes:

```text
Gallery Discovery.md
```

while:

```yaml
id: scn-7c42be89
```

remains unchanged.

Catalyst therefore treats the operation as a rename rather than deletion plus creation.

---

# 26. Document Movement

Documents may move within the project.

Example:

```text
Manuscript/Chapter 05/Scene 02.md
```

may move to:

```text
Manuscript/Chapter 06/Scene 01.md
```

Stable IDs allow references to survive the move.

Structural metadata must be updated appropriately.

---

# 27. Manuscript Order

Filesystem order is not authoritative manuscript order.

Catalyst maintains explicit logical order.

Example:

```yaml
order:
  - scn-a1214b70
  - scn-c7252d91
  - scn-19f26ae4
```

The UI may allow drag-and-drop reordering.

Reordering changes logical structure without requiring destructive document rewriting.

---

# 28. Chapter and Scene Relationship

Scenes reference their parent chapter by stable ID.

Example:

```yaml
chapter: ch-7fa51c20
```

The chapter maintains scene order.

This provides both:

- Parent reference
- Explicit ordering

Integrity checks can detect disagreement between the two.

---

# 29. POV Metadata

Scenes may identify one or more point-of-view characters.

Example:

```yaml
pov:
  - chr-a84f2931
```

For projects requiring a single POV:

```yaml
pov: chr-a84f2931
```

The project template may define which model is expected.

---

# 30. Scene Status

Catalyst should support configurable document workflow status.

Default examples may include:

```text
planned
outline
draft
revised
edited
final
```

Status is metadata rather than filename convention.

---

# 31. Word Counts

Word counts are derived information.

The authoritative manuscript contains words.

The word-count value itself may be cached.

Therefore:

```text
Manuscript text = authoritative
Word count = derived
```

Word counts do not need to be written into every manuscript file on each edit.

---

# 32. Statistics

Derived statistics may include:

- Word count
- Character count
- Reading time
- Scene length
- Chapter length
- Daily writing totals
- Revision totals

These may be stored in SQLite or other caches.

They must be reconstructable from authoritative project data where applicable.

---

# 33. Links Between Documents

Documents should link to other Catalyst objects using stable IDs internally.

Example metadata:

```yaml
characters:
  - chr-a84f2931

locations:
  - loc-5b12ac84

events:
  - evt-2023-11-03-shooting
```

Human-facing displays resolve IDs into current names.

---

# 34. Human-Readable Links and Wikilinks

Markdown body content may use ordinary Markdown links and Catalyst wikilinks.

The canonical writer-facing wikilink syntax is:

```text
[[Display Name]]
[[Display Name|Custom Label]]
```

A human-readable wikilink is presentation text, not durable object identity. When a wikilink resolves unambiguously to a structured Catalyst object, Catalyst records or derives the resolved target stable ID through the shared link-resolution layer. Structured metadata and machine-facing APIs use the stable ID.

Resolution is scoped to the active project. A normal book-level resolver must not search unrelated projects.

Resolution rules:

1. An explicit stable-ID reference is exact.
2. A human-readable wikilink may resolve by current title/name and documented aliases.
3. Multiple matching objects are ambiguous and require writer selection; Catalyst must not guess.
4. A missing target remains a visible unresolved link.
5. Renaming or moving a resolved target does not change its stable identity.
6. Backlinks are derived from authoritative project files and may be cached/indexed only in disposable, rebuildable infrastructure.

Catalyst may render resolved wikilinks using the target's current display name while retaining the writer's authored Markdown unless the writer explicitly chooses to rewrite link text.

Application UI may use an internal navigation URI or object descriptor, but such runtime navigation tokens are not authoritative project storage and must not replace stable IDs.

---

# 35. Broken References

A document may reference an ID that no longer exists.

Catalyst must not silently discard the reference.

Instead, the project integrity system should report:

```text
Broken reference

Document:
scn-7c42be89

Missing object:
chr-a84f2931
```

The writer may then repair or intentionally remove the reference.

---

# 36. Unknown Metadata Preservation

Catalyst must preserve unknown YAML metadata whenever practical.

Example:

```yaml
future_feature:
  value: example
```

An older Catalyst version should not automatically erase the field merely because it does not understand it.

This supports:

- Forward compatibility
- Plugins
- Manual extensions

---

# 37. Plugin Document Access

Plugins must interact with documents through the Catalyst Plugin API.

Plugins should receive a normalized document representation rather than direct TipTap internals.

Conceptually:

```text
Plugin
   │
   ▼
Plugin API
   │
   ▼
Catalyst Document Model
```

Not:

```text
Plugin
   │
   ▼
TipTap internals
```

This protects plugin compatibility if the editor changes.

---

# 38. Plugin Editing

A plugin with appropriate permissions may request document changes.

The Plugin API should support operations such as:

```text
readDocument
updateMetadata
insertText
replaceRange
createAnnotation
createDocument
```

Permissions are defined in:

`005-PLUGIN-ARCHITECTURE.md`

Plugins must not bypass normal save, validation, revision, and conflict mechanisms.

---

# 39. External Agent Access

External agents interact with documents through the Agent API.

They must not directly modify files unless Catalyst explicitly exposes such an operation.

Preferred flow:

```text
External Agent
      │
      ▼
Agent API
      │
      ▼
Permission Check
      │
      ▼
Document Operation
      │
      ▼
Validation
      │
      ▼
Revision / Audit
      │
      ▼
Safe Save
```

The Agent API is defined in:

`006-AGENT-API.md`

---

# 40. Agent Suggestions

Catalyst may allow an external agent to propose changes without immediately applying them.

Example:

```text
External Agent
      │
      ▼
Proposed Change
      │
      ▼
Catalyst
      │
      ▼
Writer Reviews
   │       │
 Accept   Reject
```

This is distinct from Catalyst containing built-in AI.

---

# 41. Undo and Redo

Editor undo/redo may use TipTap's editing history while a document is open.

Undo/redo is separate from durable revision history.

Conceptually:

```text
Editor Undo
    ≠
Project Revision History
```

Closing the application must not make revision protection dependent solely on the editor's undo stack.

---

# 42. Revision History

Durable revisions should operate on Catalyst documents rather than TipTap state.

A revision may include:

```text
revision_id
document_id
timestamp
source
previous_hash
new_hash
change metadata
```

The exact revision storage architecture will be defined separately.

---

# 43. Named Snapshots

A writer may create named project or document snapshots.

Example:

```text
Before Structural Rewrite
Sent to Developmental Editor
First Draft Complete
Agent Suggestions Before Review
```

Snapshots must not rely on TipTap's internal history.

---

# 44. Document Hashing

Catalyst may calculate content hashes for:

- Change detection
- Revision identification
- External modification detection
- Integrity verification
- Cache invalidation

The hash algorithm is an implementation detail unless stored as part of a permanent project format.

---

# 45. Parsing Errors

Invalid YAML or malformed Markdown must not cause silent data loss.

If a document cannot be parsed:

1. Preserve the original file.
2. Report the problem.
3. Avoid destructive rewriting.
4. Offer repair where possible.

The raw document should remain accessible.

---

# 46. YAML Safety

Catalyst should use safe YAML parsing.

Project YAML must be treated as data.

It must not permit arbitrary code execution.

---

# 47. Character Encoding

All authoritative textual documents use UTF-8.

Catalyst must preserve Unicode content including:

- Accented characters
- Non-Latin scripts
- Typographic punctuation
- Symbols

---

# 48. Typographic Characters

Catalyst may support writer-friendly typography such as:

- Curly quotation marks
- Em dashes
- Ellipses
- Nonbreaking spaces

These characters may be stored directly as UTF-8.

The application should not require proprietary encoding for typography.

---

# 49. Search

Full-text search operates over the Catalyst Document Model and/or derived indexes.

Search indexes may be stored in SQLite.

The search index is not authoritative.

If deleted, it must be rebuildable from project documents.

---

# 50. Find and Replace

Find and replace must operate through document services.

Multi-document replacement should:

1. Identify matches.
2. Allow review where appropriate.
3. Apply changes through the document model.
4. Create revision information.
5. Save safely.
6. Update indexes.

Bulk replacement must not directly perform uncontrolled filesystem text replacement.

---

# 51. Export

Exporters consume Catalyst domain documents.

Conceptually:

```text
Project Files
     │
     ▼
Document Model
     │
     ▼
Exporter
     │
     ├── DOCX
     ├── EPUB
     ├── Markdown
     ├── Plain Text
     └── Other Formats
```

Exporters should not depend directly upon TipTap.

---

# 52. Import

Importers convert external content into Catalyst domain documents.

Conceptually:

```text
Scrivener
novelWriter
Plottr
yWriter
Markdown
Other
    │
    ▼
Importer
    │
    ▼
Catalyst Document Model
    │
    ▼
Validation
    │
    ▼
Project Files
```

Imported data must receive stable Catalyst IDs.

---

# 53. Copy and Paste

Normal clipboard operations should support:

- Plain text
- Common rich text
- Markdown-compatible formatting

Pasted content should be normalized into the Catalyst-supported document model.

Unsupported hidden formatting should not unnecessarily contaminate project files.

---

# 54. Images

Documents may reference project assets.

Preferred representation:

```markdown
![Gallery floor plan](../../Assets/Images/gallery-floor-plan.png)
```

Paths should remain relative to the project where practical.

The editor may render the image visually.

---

# 55. Attachments

Non-image files should normally exist in `Assets` or `Research`.

Documents may reference them through relative links or structured metadata.

Catalyst should not embed large binary attachments directly inside Markdown.

---

# 56. Scene Breaks

Catalyst should support manuscript scene breaks using portable textual representation.

A default may use:

```markdown
***
```

The editor may render this as a visually styled scene break.

Export profiles may convert it into publisher-appropriate formatting.

---

# 57. Formatting Versus Semantics

Catalyst should distinguish semantic structure from visual appearance.

For example:

```text
Chapter Heading
```

is a semantic role.

Its font, size, spacing, and alignment are presentation concerns.

This allows the same manuscript to be rendered differently for:

- Writing
- Printing
- Submission
- EPUB
- DOCX
- Screen display

---

# 58. Export Profiles

Export formatting should not require destructive changes to manuscript documents.

An export profile may specify:

```text
Font
Margins
Paragraph indentation
Line spacing
Chapter heading style
Scene break style
Title-page layout
Page numbering
```

The manuscript remains content-focused.

---

# 59. Screenplay Documents

Kindling-derived screenplay capabilities may use specialized document semantics.

Screenplay elements may include:

```text
scene-heading
action
character
dialogue
parenthetical
transition
```

These should be represented at the Catalyst domain level rather than permanently depending upon TipTap node types.

---

# 60. Nonfiction Documents

The document model must not assume every project is fiction.

Nonfiction may require:

- Sections
- Citations
- Footnotes
- Sources
- Figures
- Tables
- Appendices

The core document abstraction should remain extensible enough to support these without replacing the entire architecture.

---

# 61. Document Validation

Before durable save, Catalyst may validate:

- Stable ID
- Document type
- YAML syntax
- Required metadata
- Parent references
- Path validity
- Duplicate IDs

Validation errors should prevent unsafe transformations but should not unnecessarily prevent writers from recovering or editing their prose.

---

# 62. Domain Events

Document operations should emit domain events.

Examples:

```text
document.created
document.opened
document.changed
document.saved
document.renamed
document.moved
document.deleted
metadata.changed
annotation.created
revision.created
```

These events may support:

- UI updates
- Index rebuilding
- Plugins
- External integrations
- Revision systems

Event APIs must respect permissions.

---

# 63. Document Service

All core document operations should pass through a Document Service.

Conceptually:

```text
DocumentService

open()
create()
read()
update()
save()
rename()
move()
delete()
restore()
validate()
```

The Document Service enforces project rules.

---

# 64. No Direct UI Persistence

Svelte UI components must not become responsible for writing authoritative project files directly.

Incorrect architecture:

```text
Svelte Component
      │
      ▼
fs.writeFile()
```

Required architecture:

```text
Svelte Component
      │
      ▼
Catalyst Command/API
      │
      ▼
Document Service
      │
      ▼
Persistence Layer
```

This separation is required for integrity, testing, plugins, agents, and future UI implementations.

---

# 65. No Direct Plugin Persistence

Plugins must not bypass the Document Service for core Catalyst documents.

This protects:

- Validation
- Revision history
- Conflict detection
- Stable IDs
- Backups
- Event generation

---

# 66. No Direct Agent Persistence

External agents must not bypass the Document Service.

Agent-originated changes should be distinguishable from writer-originated changes where audit history is enabled.

---

# 67. Kindling Migration

Kindling editor functionality may be retained where compatible.

However:

```text
Kindling Editor → SQLite
```

must evolve toward:

```text
TipTap
   │
Editor Adapter
   │
Catalyst Document Model
   │
Document Service
   │
Markdown/YAML
```

Kindling's existing database-oriented assumptions must not leak through the new domain boundary.

---

# 68. Testing Requirement

The Document Model requires automated testing.

Tests should cover:

- Markdown parsing
- YAML parsing
- Serialization
- Round-trip preservation
- Unicode
- Metadata preservation
- Unknown metadata
- Stable IDs
- File renaming
- File movement
- External modification
- Conflict detection
- Safe saving
- Annotation anchoring
- Plugin operations
- Agent operations
- Import conversion
- Export conversion

A document that is loaded and saved without intentional edits should not experience unnecessary content changes.

---

# 69. Round-Trip Integrity

A fundamental test is:

```text
File
 ↓
Parse
 ↓
Document Model
 ↓
Serialize
 ↓
File
```

The resulting document should preserve all supported information and, where practical, unknown information.

Catalyst must avoid gratuitous reformatting of externally edited Markdown.

---

# 70. Governing Principle

The editor is a tool used to modify the writer's document.

The editor is not the document.

TipTap is replaceable.

Svelte is replaceable.

SQLite indexes are replaceable.

Catalyst itself is replaceable.

The writer's document is permanent.

---

# 71. Related Specifications

Parent specifications:

- `000-CATALYST-ARCHITECTURE.md`
- `001-PROJECT-FILE-FORMAT.md`

Related specifications:

- `003-STABLE-ID-SYSTEM.md`
- `004-CANON-MODEL.md`
- `005-PLUGIN-ARCHITECTURE.md`
- `006-AGENT-API.md`
- `007-KINDLING-MIGRATION.md`

---



# 35. Template-Backed Manuscript Documents

Manuscript creation uses scoped templates while preserving a prose-first writing experience.

A newly created chapter, scene, prologue, epilogue, interlude, part/act divider, front-matter item, or back-matter item may initialize YAML metadata from the applicable manuscript template. The prose body may remain empty so the writer receives a clean writing surface.

Template initialization does not make the template authoritative after creation. The resulting Markdown/YAML document is authoritative.

Writers must be able to edit the complete YAML front matter and Markdown source. Direct YAML edits must be parsed and validated before authoritative replacement. Invalid YAML must not overwrite the last valid source. Unknown/custom metadata must be preserved where practical.

Outline templates and story-structure frameworks remain separate from manuscript templates. Three-Act Structure, Save the Cat, beat sheets, and similar frameworks may organize or reference chapters/scenes, but they must not replace manuscript identity or prose authority.

---

# 36. Derived Connections and Context Projections

Catalyst distinguishes declared facts from derived connections.

Authoritative inputs may include stable-ID metadata references, explicit relationship documents or relationship fields, resolved wikilinks, scene participants and locations, timeline-event references, canon facts, and ordinary Markdown/research files.

Derived outputs may include backlinks, mention counts, related-entity lists, relationship graphs, organization charts, scene context panels, entity timelines, and continuity indexes.

Derived outputs may be cached for performance but must be reconstructable from authoritative project files. Catalyst must not require writers to duplicate backlink counts, graph edges discovered from links, or other computed values into YAML.

A right-side context panel or graph is therefore a presentation/index projection, not a separate source of truth.

---

**END OF DOCUMENT MODEL SPECIFICATION**