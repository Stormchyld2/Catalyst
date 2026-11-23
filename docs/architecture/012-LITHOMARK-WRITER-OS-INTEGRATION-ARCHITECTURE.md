# LithoMark Writer OS — Integration and Manuscript Lifecycle Architecture

**Status:** Locked architecture  
**Scope:** Product workflow, cross-workspace integration, Story Development, Review lifecycle, and Core Preview boundary  
**Principle:** The project is the product. Workspaces are coordinated views and workflows over one authoritative project.

## 1. Purpose

LithoMark Writer OS follows the natural lifecycle of a manuscript rather than presenting a collection of disconnected tools.

The canonical lifecycle is:

```text
Story Development
      ↓
Outline
      ↓
Write
      ↓
Developmental Review
      ↓
Revision
      ↓
Beta Reading
      ↓
Revision
      ↓
ARC
      ↓
Publish
```

This is a guiding workflow, not a mandatory wizard. Writers may move backward and forward at any time. A developmental review may return the writer to Story Development; beta feedback may require plot restructuring; revision may alter canon, timeline, characters, or manuscript prose.

All stages operate on one LithoMark project and one authoritative set of project files.

## 2. Core product rule

**The project is the product.**

A writer opens a project once. The writer does not import the book separately into Story Development, Review, Audiobook, or Publishing.

Every workspace consumes the same project through LithoMark domain services and stable identities.

Modules must not create competing authoritative representations of manuscript, character, location, timeline, plot, or publication metadata.

Exported review, Beta, ARC, audiobook, and publishing artifacts are deliberate snapshots or generated deliverables. They do not replace the authoritative working project.

## 3. Major workspace model

The principal creative and production workspaces are:

- **Home / Project Dashboard**
- **Story Development**
- **Outline**
- **Write**
- **Review**
- **Listen / TTS**
- **Audiobook**
- **Publish / Export**

Project services include:

- Git
- Files
- Terminal
- Settings

The Module Rail should remain comprehensible. Subsystems that belong to a larger stage should normally appear inside that workspace instead of becoming unnecessary top-level modules.

## 4. Story Development workspace

**Story Development** is LithoMark's canonical story-bible, worldbuilding, canon/chronology, and linked-knowledge environment.

It is not merely a Wiki. It owns and presents the structured story information used across the project, including characters, locations, organizations, worldbuilding, events, relationships, canon facts, and chronology.

**Outline is a separate top-level workspace between Story Development and Write.** It consumes the same authoritative project records and stable identities for chapters, scenes, beats, plotlines, chronology, and linked Story Development entities.

### Outline integration

- visual story board / timeline
- acts and parts
- chapters
- scenes
- beats
- plotlines
- subplots
- story threads
- story structure templates
- reveals, clues, foreshadowing, and payoffs
- character arcs
- drag-and-drop structural planning where safe

Plot cards must reference the same chapter and scene identities used by Write. They must not become a second manuscript structure that requires synchronization.

### Story Bible / Wiki

- characters
- locations
- organizations
- objects/items
- events
- professions/procedures
- worldbuilding
- relationships
- custom entities and collections
- custom fields
- tags

### Timeline and continuity context

- chronology
- dates/date ranges
- relative ordering
- scene relationships
- character participation
- location relationships
- canon-relevant facts

### Development knowledge

- themes
- character development
- open questions
- research
- notes
- custom templates
- linked knowledge

Obsidian-style wikilinks and backlinks may connect manuscript and Story Development material. The resolver may use disposable indexes, but the project files remain authoritative.

The cross-module link graph is a shared project projection. Structured relationships use stable IDs; human-readable wikilinks resolve to those identities when possible. Write, Story Development, Review, search, and Context Panel navigation must not maintain separate link authorities. Ambiguous or broken links remain explicit integrity states rather than being silently rebound by filename, path, or nearest-name matching.

Kindling-era SQLite reference tables, scene-reference UUID records, and cross-project reference-copy commands are transitional compatibility code. They are not the target LithoMark wikilink/backlink model and must not be used as the persistence authority for new project linking.

## 5. Write workspace

Write owns the manuscript authoring experience, not a separate manuscript database.

It provides chapters, scenes, prose editing, manuscript navigation, word counts, search, revision operations, and related writing tools over authoritative project files.

Write and Story Development share stable chapter, scene, character, location, timeline, and story-thread identities.

A rename or structural move must preserve identity wherever practical so that Story Development, Review, audio, and publishing references remain attached.

## 6. Shared application context

The Writer OS shell maintains a shared active context so workspace changes remain coherent.

Conceptually:

```text
Active Project
├── Active Workspace
├── Active Chapter
├── Active Scene
├── Active Entity
├── Active Selection
└── Navigation History
```

Selection should travel between workspaces when meaningful.

Examples:

- Switch from a manuscript scene to Story Development and retain the current chapter/scene context.
- Activate a character wikilink in Write and inspect that character without losing the manuscript position.
- Open a review issue and navigate directly to the affected scene.
- Open a plot item and locate the manuscript scene that implements it.
- Listen to the currently active scene without creating a second text copy.

## 7. Context Panel as connective tissue

The right Context Panel remains scene-aware first and chapter-aware second in Write.

It may surface relevant information from multiple subsystems without turning into a whole-project browser:

- characters
- locations
- POV
- timeline/date
- plotlines/story threads
- scene purpose or notes
- developmental context
- review/revision items
- listening/audio state where useful

The Context Panel references shared entities. It must not create duplicate records.

## 8. Developmental Review

Developmental Review is the bridge between Story Development, Write, and Revision.

It evaluates the manuscript in structural context, including:

- structure
- chronology
- plot progression
- plot holes
- pacing
- scene function
- character arcs
- subplots/story threads
- reveals and payoffs
- continuity
- timeline
- other project-specific developmental concerns

Review may read Story Development information to understand the intended structure. It does not own or duplicate that information.

## 9. Revision is a cross-workspace workflow

Revision is a lifecycle stage, not necessarily a separate top-level application.

Revision work normally occurs in Write and Story Development and is driven by issues from Developmental Review, Beta Reading, the author, or other permitted sources.

LithoMark should support durable **Revision Items**.

A Revision Item may contain:

- stable revision ID
- issue/title
- source
- review round or reader where applicable
- category
- severity/priority if used
- related chapter/scene IDs
- related entity IDs
- related plotline/thread IDs
- status
- resolution notes
- timestamps

A single Revision Item may therefore be visible in Review, beside a scene in Write, and against related Story Development material without being duplicated.

## 10. Beta Reading

Beta Reading is part of the Review lifecycle.

Beta packages are recipient-specific manuscript snapshots intended to gather reader-response and developmental feedback while the manuscript may still change.

The Beta workflow should support:

- review rounds
- reader/recipient identity
- recipient-specific package creation
- visible review-copy identification
- package manifest and integrity
- durable author-side distribution ledger
- returned feedback
- feedback comparison/aggregation
- conversion of feedback into Revision Items
- package/import traceability through stable IDs

The native `lithomark-review` package infrastructure is the portability foundation for this workflow.

## 11. ARC

ARC is also part of Review but represents a later manuscript lifecycle stage.

An ARC package is an advance reading copy intended primarily for pre-publication readers/reviewers rather than ordinary developmental beta work.

ARC should share the secure/traceable package and distribution infrastructure with Beta Reading while maintaining distinct workflow metadata and purpose.

Expected ARC concerns include:

- ARC recipient/reviewer
- edition/version identity
- package creation
- distribution ledger
- visible copy identification
- integrity/provenance
- follow-up state
- returned notes where applicable

Beta and ARC must not be collapsed into one ambiguous status even when they share implementation infrastructure.

## 12. Review package boundary

Exported Developmental, Beta, or ARC review packages are deliberate snapshots.

They must not become the author's live authoritative manuscript.

The author project retains durable distribution evidence independently of exported copies.

Returned packages and feedback must never silently overwrite the authoritative manuscript. Applying a reviewer-authored prose change requires explicit author action.

The detailed interchange rules remain defined by **008-EDITORIAL-REVIEW-PACKAGE.md**.

## 13. Publish as lifecycle endpoint

Publish / Export consumes the authoritative project and produces deliberate publishing deliverables.

It should reuse project metadata and manuscript structure rather than maintaining another independent title/author/book record where the project already owns that information.

Publishing may include formatting, validation, metadata, and generation of formats such as EPUB, PDF, DOCX, print-oriented output, and other supported deliverables.

Publish is the natural endpoint of the manuscript lifecycle, but LithoMark must never prevent a writer from publishing earlier or returning to prior stages afterward.

## 14. Specialized audio workflows

Listen/TTS and Audiobook are specialized workflows connected to the same manuscript lifecycle.

Listen/TTS supports proofreading and listening during writing and revision.

Audiobook supports production of a narrated edition.

Neither owns manuscript prose. Shared pronunciation, voice, or narration infrastructure should be reused where appropriate without conflating proofreading with audiobook production.

## 15. Cross-workspace actions

Stable objects may expose contextual actions such as:

```text
Open in Write
Open in Story Development
View Planning Context
Review
Create Revision Item
Listen
Open in Audiobook
Export
Show in Files
View Git History
```

Actions operate on stable project identities rather than copying content between modules.

## 16. Stable identity requirement

Cross-workspace cohesion depends on stable IDs.

Significant objects such as chapters, scenes, characters, locations, plotlines, timeline events, review rounds, revision items, and recipients must not depend on display names or paths as permanent identity.

Renaming or moving a file must not unnecessarily sever relationships.

The detailed identity rules remain defined by **003-STABLE-ID-SYSTEM.md**.

## 17. File ownership rule

Every subsystem must identify which durable files it owns and which shared files it merely consumes.

A subsystem must not create a second authoritative copy because its UI needs a different presentation.

Examples:

- Story Development and Write share chapter/scene identity.
- Review consumes manuscript and Story Development context.
- Listen consumes manuscript.
- Audiobook consumes manuscript and stores audiobook-production state separately.
- Publish consumes manuscript and project metadata and stores generated/export state separately.
- Git observes/version-controls project files but is not project truth.

SQLite, search indexes, caches, relationship indexes, and derived views remain disposable/rebuildable.

## 18. Core Preview 1 boundary

The first public preview is the **LithoMark Writer OS Core Preview**.

Its principal authoring scope is:

### Write

A trustworthy file-authoritative manuscript editor suitable for real writing.

### Story Development

A meaningful implementation of:

- Plot & Outline visual planning
- Story Bible / Wiki
- characters
- locations
- organizations
- timeline
- relationships
- arcs
- themes
- subplots/story threads
- research
- notes
- templates
- wikilinks
- contextual navigation

The purpose of Preview 1 is to prove the Writer OS foundation: a writer can develop and write a real project in one cohesive local-first environment.

Review, Beta/ARC, Listen/TTS, Audiobook, and Publish remain part of the locked lifecycle architecture but do not have to block Core Preview 1 unless a specific capability is ready and stable enough to expose safely.

## 19. Core Preview trust test

Preview 1 should not be considered ready merely because screens render.

At minimum, a writer should be able to:

1. Create or open a normal filesystem project.
2. Develop the story using Story Development.
3. Create and organize chapters/scenes.
4. Write and edit manuscript prose.
5. Link manuscript and story knowledge.
6. Rename/move supported project objects without silently breaking identity.
7. Save safely.
8. Close LithoMark.
9. Reopen the project with content and relationships intact.
10. Recover safely from supported failure conditions.
11. Use Git/version control when configured without Git becoming mandatory.
12. Inspect understandable authoritative project files outside LithoMark.

The practical question is:

> Can a writer trust LithoMark with a real novel as the primary development and writing environment?

## 20. Lifecycle-aware Project Dashboard

The Project Dashboard may summarize the manuscript's current lifecycle state, for example:

```text
Story Development     active
Writing               in progress
Developmental Review  not started
Beta Reading          not started
ARC                    not started
Publish                not started
```

Lifecycle state is informational and workflow-supporting. It must not lock the writer out of workspaces or force a linear process.

## 21. Integration contract

Before implementing or expanding any major subsystem, answer:

1. Where does it sit in the manuscript lifecycle?
2. Which authoritative project objects does it consume?
3. Which authoritative project objects does it own?
4. Does it duplicate information already owned elsewhere?
5. How does it reference shared stable IDs?
6. What shared application context should follow into/out of it?
7. What belongs in Workspace Navigation?
8. What belongs in the Main Work Area?
9. What belongs in the Context Panel?
10. What cross-workspace actions should it expose?
11. What generated artifacts are snapshots/deliverables rather than project truth?
12. Can all durable state survive deletion/rebuild of disposable databases and indexes?
13. Does it preserve offline/local-first operation?
14. Does it preserve project isolation?

If a subsystem cannot answer these questions cleanly, its integration design is not complete.

## 22. Locked lifecycle summary

The canonical LithoMark manuscript journey is:

> **Story Development → Write → Developmental Review → Revision → Beta Reading → Revision → ARC → Publish**

The architectural interpretation is:

- **Story Development** creates and maintains structural/story knowledge.
- **Write** creates and revises the manuscript.
- **Developmental Review** evaluates manuscript and structure together.
- **Revision** routes actionable issues back into Write and Story Development.
- **Beta Reading** collects reader-response feedback through traceable packages/rounds.
- **Revision** incorporates accepted beta findings.
- **ARC** manages advance reading copies and recipient distribution.
- **Publish** creates final publishing deliverables.

All stages belong to one Writer OS and one project.

LithoMark's product promise is therefore:

> **LithoMark Writer OS supports the manuscript from first idea through publication in one local-first project.**
