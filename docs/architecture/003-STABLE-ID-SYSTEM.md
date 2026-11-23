# Catalyst — Stable ID System

**Document:** 003-STABLE-ID-SYSTEM.md  
**Status:** Locked Baseline  
**Project:** Catalyst  
**Parent Specification:** 000-CATALYST-ARCHITECTURE.md  
**Related Specifications:** 001-PROJECT-FILE-FORMAT.md, 002-DOCUMENT-MODEL.md

---

# 1. Purpose

This specification defines permanent identity for objects within Catalyst.

Stable identifiers allow objects to be renamed, moved, reorganized, reordered, or displayed differently without breaking relationships.

A Catalyst object's identity must never depend solely upon:

- Name
- Title
- Filename
- File path
- Chapter number
- Scene number
- Display order
- Current location in the project

---

# 2. Fundamental Rule

Every significant Catalyst domain object receives a stable ID.

Example:

```yaml
lithomark:
  type: character
  id: chr-550e8400-e29b-41d4-a716-446655440000

name: Leah Bennett
```

References use:

```yaml
character: chr-550e8400-e29b-41d4-a716-446655440000
```

They do not use:

```yaml
character: Leah Bennett
```

as permanent identity.

---

# 3. Identity Versus Display Name

Identity and display name are separate concepts.

```text
Stable Identity
      │
      └── chr-550e8400-e29b-41d4-a716-446655440000

Display Name
      │
      └── Leah Bennett
```

The display name may change.

The stable ID does not.

---

# 4. Rename Example

Initial state:

```yaml
id: chr-550e8400-e29b-41d4-a716-446655440000
name: Leah Bennett
```

Later:

```yaml
id: chr-550e8400-e29b-41d4-a716-446655440000
name: Leah Warren
```

Relationships remain valid because they reference:

```text
chr-550e8400-e29b-41d4-a716-446655440000
```

rather than the character's name.

---

# 5. Identifier Format

Catalyst stable IDs use:

```text
<prefix>-<uuid>
```

Example:

```text
chr-550e8400-e29b-41d4-a716-446655440000
```

UUIDs should use a standards-based UUID implementation.

The initial implementation should use UUID version 4 unless a later architectural decision adopts another UUID version for a specific reason.

---

# 6. Why UUIDs

UUIDs provide:

- Extremely low collision probability
- Offline generation
- No central ID server requirement
- Cross-device generation
- Plugin compatibility
- Import compatibility
- Merge compatibility
- Git-friendly project creation
- Future synchronization compatibility

Catalyst must not require an internet service to create object IDs.

---

# 7. Human-Readable Prefixes

The UUID is preceded by a type prefix.

Examples:

```text
prj-
prt-
ch-
scn-
beat-
chr-
loc-
org-
obj-
evt-
rel-
plt-
arc-
rev-
clu-
foresh-
pay-
res-
note-
tpl-
fact-
ann-
chg-
snap-
```

Prefixes improve:

- Debugging
- Log readability
- YAML readability
- Integrity reporting
- Developer experience

---

# 8. Core Prefix Registry

Initial core prefixes:

| Object | Prefix |
|---|---|
| Project | `prj-` |
| Part | `prt-` |
| Chapter | `ch-` |
| Scene | `scn-` |
| Beat | `beat-` |
| Character | `chr-` |
| Location | `loc-` |
| Organization | `org-` |
| Object/Item | `obj-` |
| Timeline Event | `evt-` |
| Relationship | `rel-` |
| Plot Thread | `plt-` |
| Character Arc | `arc-` |
| Reveal | `rev-` |
| Clue | `clu-` |
| Foreshadowing | `foresh-` |
| Payoff | `pay-` |
| Research Item | `res-` |
| Note | `note-` |
| Template | `tpl-` |
| Canon Fact | `fact-` |
| Annotation | `ann-` |
| Tracked Change | `chg-` |
| Snapshot | `snap-` |

Additional core prefixes may be added through project-format revisions.

---

# 9. Project Identity

Every project has a permanent ID.

Example:

```yaml
project:
  id: prj-550e8400-e29b-41d4-a716-446655440000
```

Copying a project as a backup preserves the ID.

Creating a new project from another project may require generating a new project ID depending upon the operation.

---

# 10. Duplicate Project Operation

Catalyst must distinguish between:

```text
Backup Project
```

and:

```text
Duplicate as New Project
```

A backup preserves all IDs.

A duplicated new project should receive a new project ID.

Internal object IDs may be regenerated when necessary to prevent future cross-project ambiguity.

The duplication operation must preserve all internal relationships.

---

# 11. Object Creation

When a new structured object is created, Catalyst generates its stable ID immediately.

Example:

```text
Create Character
      │
      ▼
Generate UUID
      │
      ▼
chr-f47ac10b-58cc-4372-a567-0e02b2c3d479
      │
      ▼
Create Character Document
```

An object should not exist in authoritative project storage without an ID when its type requires stable identity.

---

# 12. IDs Are Immutable

Once assigned, a stable ID must not normally change.

Changing:

```text
chr-AAA
```

to:

```text
chr-BBB
```

would invalidate existing references.

Therefore stable IDs are treated as immutable identity.

---

# 13. Manual Editing

Because Catalyst projects are human-readable, users may manually edit IDs.

Catalyst must tolerate this possibility but should warn when changing an ID breaks references.

If:

```yaml
id: chr-AAA
```

becomes:

```yaml
id: chr-BBB
```

Catalyst's integrity system should identify references still pointing to:

```text
chr-AAA
```

---

# 14. Duplicate IDs

No two authoritative objects within the same project may share the same stable ID.

Example conflict:

```text
Story Development/Characters/Main Cast/Leah Bennett.md
chr-550e8400-e29b-41d4-a716-446655440000

Story Development/Characters/Main Cast/Leah Copy.md
chr-550e8400-e29b-41d4-a716-446655440000
```

Catalyst must report this as an integrity error.

---

# 15. Duplicate Resolution

When duplicate IDs are detected, Catalyst should not silently choose one.

The user should be given repair options.

Possible resolution:

```text
Keep original ID for Leah Bennett.md

Generate new ID for Leah Copy.md

Update references if appropriate
```

Repair operations should create a backup or revision before changing authoritative data.

---

# 16. Missing IDs

If Catalyst discovers a structured object requiring an ID but none exists, the project integrity system may offer to generate one.

Example:

```yaml
lithomark:
  type: character

name: Leah Bennett
```

Catalyst may propose:

```yaml
id: chr-550e8400-e29b-41d4-a716-446655440000
```

Automatic repair should be logged and safely saved.

---

# 17. Type Validation

The ID prefix should correspond to the object's declared type.

Example:

```yaml
lithomark:
  type: character
  id: chr-550e8400-e29b-41d4-a716-446655440000
```

is valid.

This:

```yaml
lithomark:
  type: character
  id: loc-550e8400-e29b-41d4-a716-446655440000
```

should produce an integrity warning or error.

---

# 18. References

References between objects use stable IDs.

Example:

```yaml
pov: chr-550e8400-e29b-41d4-a716-446655440000
```

Multiple references:

```yaml
characters:
  - chr-550e8400-e29b-41d4-a716-446655440000
  - chr-f47ac10b-58cc-4372-a567-0e02b2c3d479
```

---

# 19. Relationship Example

A relationship object may contain:

```yaml
lithomark:
  type: relationship
  id: rel-1c6e7257-a12b-4f77-a491-7ef184cebdf0

source: chr-550e8400-e29b-41d4-a716-446655440000
target: chr-f47ac10b-58cc-4372-a567-0e02b2c3d479

relationship_type: partner
```

The relationship survives renaming either character.

---

# 20. Timeline Example

A timeline event may contain:

```yaml
lithomark:
  type: event
  id: evt-95a1bb7c-b28e-48cc-9555-c7c59bc8a235

participants:
  - chr-550e8400-e29b-41d4-a716-446655440000

locations:
  - loc-e78986d0-f559-4a31-9c89-64f579ccbd28
```

---

# 21. Canon Example

A canon fact may reference objects by stable ID.

```yaml
lithomark:
  type: canon-fact
  id: fact-b1ca8a4a-2812-45ac-a017-e605e774bec7

subject: chr-f47ac10b-58cc-4372-a567-0e02b2c3d479

fact_type: death

value:
  date: 2023-11-03
  location: loc-e78986d0-f559-4a31-9c89-64f579ccbd28
```

The fact remains valid even if the character or location is renamed.

---

# 22. Manuscript Structure

Chapters and scenes use stable IDs.

Example chapter:

```yaml
lithomark:
  type: chapter
  id: ch-256f9128-7745-40f9-a831-c33ef8bf3562

order:
  - scn-df925d68-d3ef-4d90-bfa4-6fa7834203c7
  - scn-81e08abc-f3fc-47af-b8f5-09c05610c847
```

Scene filenames do not determine structural identity.

---

# 23. Display Numbers

Chapter and scene numbers are presentation metadata.

Example:

```yaml
number: 5
```

Changing Chapter 5 to Chapter 6 does not change:

```text
ch-256f9128-7745-40f9-a831-c33ef8bf3562
```

---

# 24. Reordering

Moving a scene from Chapter 5 to Chapter 7 does not change its ID.

The parent relationship and ordering information change.

Identity remains constant.

---

# 25. File Rename

Renaming:

```text
Leah Bennett.md
```

to:

```text
Leah Warren.md
```

does not change:

```text
chr-550e8400-e29b-41d4-a716-446655440000
```

Catalyst should detect this as a rename or move where possible.

---

# 26. File Movement

Moving:

```text
Story Development/Characters/Main Cast/Leah Bennett.md
```

to another valid location does not inherently change the object's ID.

Path and identity remain separate.

---

# 27. Indexing

SQLite indexes may map stable IDs to current file locations.

Example derived mapping:

```text
chr-550e... → Story Development/Characters/Main Cast/Leah Bennett.md
```

This mapping is rebuildable.

The file and its embedded stable ID remain authoritative.

---

# 28. Missing References

If an object references an ID that cannot be found, Catalyst records a broken reference.

Example:

```text
Missing object:

chr-550e8400-e29b-41d4-a716-446655440000
```

The reference must not be silently removed.

---

# 29. Orphan Detection

Catalyst may identify objects that exist but are not referenced by expected structures.

Examples:

- Scene not assigned to a chapter
- Reveal not linked to a plot thread
- Relationship referencing missing objects
- Canon fact referencing deleted subject

Orphan status does not automatically mean the object should be deleted.

---

# 30. ID Lookup

Catalyst should provide efficient lookup:

```text
stable ID
   │
   ▼
index
   │
   ▼
current object path
   │
   ▼
document
```

This may use SQLite for performance.

If the index is unavailable, Catalyst reconstructs it from project files.

---

# 31. Cross-Project Identity

Stable IDs are globally collision-resistant.

However, Catalyst references should normally remain project-scoped unless a future specification explicitly supports cross-project references.

A project must not assume another project is available.

---

# 32. Series Support

Future Catalyst versions may support series-level workspaces.

A series may contain multiple projects.

Stable IDs make it possible to reference shared characters, locations, or events without depending upon display names.

Series-level identity must be defined separately before implementation.

---

# 33. Import

Imported objects receive Catalyst stable IDs.

Example:

```text
Scrivener Character
      │
      ▼
Catalyst Importer
      │
      ▼
Generate chr-UUID
      │
      ▼
Create Character Document
```

Where imported formats contain reliable permanent IDs, Catalyst may record them as external IDs.

---

# 34. External IDs

Catalyst may preserve identifiers from external systems.

Example:

```yaml
external_ids:
  scrivener: "ABC-123"
```

External IDs do not replace the Catalyst stable ID.

---

# 35. Export

Exporters may include or omit Catalyst IDs depending upon the target format.

A DOCX manuscript normally does not need Catalyst IDs visible to the reader.

A Catalyst-compatible Markdown export may preserve them.

---

# 36. Plugin Objects

Plugins may create plugin-defined objects.

Plugin object types must be namespaced.

Example conceptual type:

```text
com.example.familytree.person-record
```

Plugin-created objects requiring durable identity must use stable IDs.

---

# 37. Plugin Prefixes

Plugins must not invent unregistered short prefixes that could collide with Catalyst Core.

Plugin APIs should provide an ID-generation service.

Example:

```text
plugin-com.example.familytree-550e8400-e29b-41d4-a716-446655440000
```

The exact plugin ID representation may be finalized in:

`005-PLUGIN-ARCHITECTURE.md`

---

# 38. Plugin ID Generation

Preferred plugin flow:

```text
Plugin
   │
   ▼
requestStableId(type)
   │
   ▼
Catalyst
   │
   ▼
Generated ID
```

This allows Catalyst to enforce consistency.

---

# 39. External Agents

External agents must use stable IDs when referencing Catalyst objects.

Example:

```json
{
  "operation": "read_character",
  "id": "chr-550e8400-e29b-41d4-a716-446655440000"
}
```

Agents should not depend upon ambiguous names.

---

# 40. Name Resolution

For human convenience, Catalyst may allow searching by name.

Example:

```text
Search:
Leah Bennett
```

may resolve to:

```text
chr-550e8400-e29b-41d4-a716-446655440000
```

If multiple matches exist, the user or calling system must disambiguate.

---

# 41. Aliases

Aliases are attributes of an object.

They are not separate identity unless intentionally modeled as separate objects.

Example:

```yaml
name: Caleb Warren

aliases:
  - Harrison Grant
```

Both names may refer to the same stable character ID when they represent the same canonical person.

---

# 42. Canonical Identity

When two apparent identities are revealed to be the same person, Catalyst should support merging or aliasing them without destroying historical project information.

This operation must be deliberate.

It must not occur automatically based on matching names.

---

# 43. Object Merge

A future object-merge operation should:

1. Select surviving stable ID.
2. Identify duplicate object.
3. Update references.
4. Preserve relevant metadata.
5. Preserve revision history.
6. Archive or remove duplicate.
7. Validate project integrity.

A backup or snapshot must precede destructive merge operations.

---

# 44. Object Split

The reverse problem may occur when one object was incorrectly used for two distinct entities.

A future split operation should support generating a new stable ID and selectively reassigning references.

---

# 45. Deletion

Deleting an object does not permit silent deletion of every reference to it.

Catalyst should identify dependencies before destructive deletion.

Example:

```text
Delete Leah Bennett?

Referenced by:

14 scenes
7 timeline events
12 canon facts
4 relationships
2 plot threads
1 character arc
```

The writer chooses how to proceed.

---

# 46. Soft Delete

Catalyst may support a recoverable trash or soft-delete mechanism.

A soft-deleted object retains its stable ID.

Restoring the object restores its identity.

---

# 47. Permanent Delete

Permanent deletion removes the authoritative object.

References may become broken unless deliberately reassigned or removed.

Catalyst must warn before destructive permanent deletion.

---

# 48. Revision History

Stable IDs allow revision systems to recognize the same object across:

- Renames
- Moves
- Title changes
- Reordering

Revision identity must therefore be based on stable ID rather than file path alone.

---

# 49. Git Operations

Git may interpret a rename differently depending on content changes.

Catalyst does not depend upon Git's rename detection.

The embedded stable ID establishes application identity independently.

---

# 50. Sync Conflicts

External synchronization may accidentally create duplicate files.

Example:

```text
Leah Bennett.md
Leah Bennett (conflicted copy).md
```

If both contain the same stable ID, Catalyst detects the duplicate-ID conflict.

It must not silently treat them as separate characters.

---

# 51. Copy and Paste of Files

Manually copying a structured Catalyst document may duplicate its stable ID.

Catalyst must detect this during indexing.

The user may choose:

```text
Treat as duplicate copy
    → Generate new ID

Treat as conflicting version
    → Compare/merge
```

---

# 52. Template IDs

Templates may contain placeholders rather than fixed object IDs.

A template must not create every new project with identical stable IDs unless those IDs are intentionally template-level identifiers.

Instantiation should generate new project object IDs.

---

# 53. Deterministic Template Relationships

When a template creates several linked objects, Catalyst should:

1. Generate new IDs.
2. Maintain a temporary old-to-new mapping.
3. Rewrite internal references.
4. Validate the generated project.

This preserves relationships without reusing IDs across unrelated projects.

---

# 54. ID Case

Stable IDs should use lowercase prefixes and lowercase UUID representation.

Preferred:

```text
chr-550e8400-e29b-41d4-a716-446655440000
```

Avoid:

```text
CHR-550E8400-E29B-41D4-A716-446655440000
```

Comparisons should follow the canonical representation.

---

# 55. Whitespace

Stable IDs must not contain spaces.

Valid:

```text
chr-550e8400-e29b-41d4-a716-446655440000
```

Invalid:

```text
chr Leah Bennett
```

---

# 56. User-Facing IDs

Normal users should rarely need to interact with full UUIDs.

The UI should display:

```text
Leah Bennett
```

rather than:

```text
chr-550e8400-e29b-41d4-a716-446655440000
```

Stable IDs remain available in:

- Developer tools
- Advanced metadata
- Integrity reports
- Logs
- Plugin APIs
- Agent APIs

---

# 57. Developer Visibility

Developer Mode may expose:

```text
Name:
Leah Bennett

Type:
Character

Stable ID:
chr-550e8400-e29b-41d4-a716-446655440000

File:
Story Development/Characters/Main Cast/Leah Bennett.md
```

This assists plugin developers and troubleshooting.

---

# 58. Logging

Logs should use stable IDs when identifying objects.

Example:

```text
Updated character chr-550e8400-e29b-41d4-a716-446655440000
```

Sensitive manuscript content should not be unnecessarily included in logs.

---

# 59. API Contract

Catalyst APIs should accept stable IDs for precise object operations.

Example:

```text
getObject(id)
updateObject(id)
deleteObject(id)
getReferences(id)
```

Name-based APIs may exist for search but should not replace ID-based operations.

---

# 60. Referential Integrity

Catalyst must maintain a referential-integrity layer.

The system should be capable of identifying:

- Missing references
- Duplicate IDs
- Invalid prefixes
- Invalid UUIDs
- Type mismatches
- Orphaned references
- Circular relationships where prohibited

---

# 61. Integrity Rebuild

Because project files are authoritative, referential integrity must be reconstructable by scanning project files.

Conceptually:

```text
Project Scan
    │
    ▼
Collect IDs
    │
    ▼
Validate uniqueness
    │
    ▼
Collect references
    │
    ▼
Resolve references
    │
    ▼
Report problems
    │
    ▼
Build index
```

---

# 62. Performance

Stable ID resolution may use SQLite indexes for speed.

Example derived table:

```text
stable_id
type
file_path
display_name
content_hash
```

This table is rebuildable.

It is not authoritative.

---

# 63. Future Database Changes

Changing from SQLite to another indexing technology must not change project object IDs.

Stable identity belongs to the project domain, not the database implementation.

---

# 64. Future Editor Changes

Changing from TipTap to another editor must not change object IDs.

Stable identity belongs to the project domain, not the editor.

---

# 65. Future UI Changes

Changing Svelte or redesigning the Catalyst UI must not change object IDs.

Stable identity is independent of presentation.

---

# 66. Future Mobile Applications

Desktop and mobile Catalyst implementations must recognize the same stable IDs.

A character created on one supported platform remains the same character on another.

---

# 67. Future Collaboration

If collaborative editing is introduced, stable IDs provide common object identity across participants and devices.

The UUID architecture allows IDs to be generated independently without a central sequential counter.

---

# 68. No Sequential Primary Identity

Catalyst must not use database-style sequential integers as permanent cross-file identity.

Avoid:

```text
character_id: 17
```

as the permanent identity.

Sequential integers may exist internally in disposable indexes for performance.

They must not replace stable project IDs.

---

# 69. No Name-Based Identity

Catalyst must never assume:

```text
"Leah Bennett" == permanent identity
```

Names may:

- Change
- Duplicate
- Contain aliases
- Use different spellings
- Represent titles
- Be intentionally unknown

Stable IDs solve this ambiguity.

---

# 70. No Path-Based Identity

Catalyst must never assume:

```text
Story Development/Characters/Main Cast/Leah Bennett.md
```

is permanent identity.

Paths change.

The embedded stable ID persists.

---

# 71. No Order-Based Identity

Catalyst must never assume:

```text
Chapter 5
Scene 2
```

is permanent identity.

Chapters and scenes are routinely reordered during writing.

---

# 72. Migration From Kindling

Kindling records migrated into Catalyst must receive stable Catalyst IDs if equivalent durable IDs are unavailable.

Migration must create a mapping:

```text
Kindling record
      │
      ▼
Catalyst stable ID
```

All migrated relationships must use the new stable IDs.

The migration process must validate that no relationships were lost.

---

# 73. Migration Mapping

During conversion, temporary mapping may resemble:

```text
kindling_scene_42
    →
scn-df925d68-d3ef-4d90-bfa4-6fa7834203c7
```

The temporary mapping may be discarded after successful migration unless needed for diagnostics.

---

# 74. Stable ID Service

Catalyst Core should expose a centralized Stable ID Service.

Conceptually:

```text
StableIdService

generate(type)
validate(id)
getType(id)
isCoreType(id)
```

All core systems should use the service rather than independently constructing IDs.

---

# 75. Validation Rules

A valid stable ID must:

1. Use a recognized prefix or valid plugin namespace.
2. Contain a valid UUID.
3. Use canonical lowercase representation.
4. Contain no whitespace.
5. Match the object's declared type where applicable.

---

# 76. Example Valid IDs

```text
prj-550e8400-e29b-41d4-a716-446655440000
ch-256f9128-7745-40f9-a831-c33ef8bf3562
scn-df925d68-d3ef-4d90-bfa4-6fa7834203c7
chr-f47ac10b-58cc-4372-a567-0e02b2c3d479
evt-95a1bb7c-b28e-48cc-9555-c7c59bc8a235
fact-b1ca8a4a-2812-45ac-a017-e605e774bec7
```

---

# 77. Example Invalid IDs

```text
character-1
Leah-Bennett
17
scene-five
CHR 123
chr-not-a-uuid
```

These may be external labels but are not valid Catalyst stable IDs.

---

# 78. Human Labels

Objects may additionally have human-friendly labels.

Example:

```yaml
id: chr-f47ac10b-58cc-4372-a567-0e02b2c3d479
name: Leah Bennett
short_name: Leah
```

Human labels may change freely without affecting identity.

---

# 79. Slugs

Catalyst may support optional slugs.

Example:

```yaml
slug: leah-bennett
```

Slugs may be useful for:

- URLs
- Human-readable links
- Export
- Navigation

Slugs must not replace stable IDs.

---

# 80. Canon Identity

Canon facts must reference stable object identities.

This ensures facts remain attached to the correct entity even after names or aliases change.

Canon identity rules are expanded in:

`004-CANON-MODEL.md`

---

# 81. Governing Principle

Names describe objects.

Paths locate objects.

Order arranges objects.

Stable IDs identify objects.

Only the stable ID is permanent identity.

---

# 82. Related Specifications

Parent specifications:

- `000-CATALYST-ARCHITECTURE.md`
- `001-PROJECT-FILE-FORMAT.md`
- `002-DOCUMENT-MODEL.md`

Related specifications:

- `004-CANON-MODEL.md`
- `005-PLUGIN-ARCHITECTURE.md`
- `006-AGENT-API.md`
- `007-KINDLING-MIGRATION.md`

---

**END OF STABLE ID SYSTEM SPECIFICATION**