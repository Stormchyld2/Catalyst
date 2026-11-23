# Catalyst Project Workspace Architecture

**Status:** Locked architecture  
**Scope:** Core Writer OS application shell and cross-module interaction model  
**Principle:** One project, one authoritative file tree, many coordinated workspaces.

## 1. Purpose

Catalyst is a Writer OS, not a collection of separate mini-applications. Writing, planning, project knowledge, reviews, text-to-speech, audiobook production, publishing, Git, files, and terminal tools must operate on the same project and the same authoritative files.

Major features must plug into a shared application shell instead of inventing independent navigation, duplicate project data, or isolated workflows.

## 2. Permanent workspace model

The desktop workspace uses four functional regions:

1. **Module Rail — far left**
   - Answers: **What part of Catalyst am I using?**
   - Switches major workspaces.

2. **Workspace Navigation — left panel**
   - Answers: **What item am I working with?**
   - Changes according to the active module.

3. **Main Work Area — center**
   - Answers: **What am I doing?**
   - Holds the manuscript editor, Wiki page, Review Studio, Audiobook Studio, etc.

4. **Context Panel — right**
   - Answers: **What do I need to know right now?**
   - Contextual, not a second project browser.

Conceptual shell:

```text
┌────────┬─────────────────┬────────────────────────────┬──────────────────┐
│ MODULE │ WORKSPACE       │                            │                  │
│ RAIL   │ NAVIGATION      │       MAIN WORK AREA       │     CONTEXT      │
│        │                 │                            │                  │
│ Home   │ Files /         │ Editor                     │ Scene / Chapter  │
│ Write  │ Wiki /          │ Wiki Page                  │ Context          │
│ Plan   │ Reviews /       │ Review Studio              │                  │
│ Review │ Audio / etc.    │ TTS / Audiobook Studio     │ Wikilink detail  │
│ Audio  │                 │ Git / Terminal             │ Review detail    │
└────────┴─────────────────┴────────────────────────────┴──────────────────┘
```

All side regions should be collapsible to support distraction-free writing.

## 3. Module Rail

The Module Rail is a narrow, persistent application-level switcher. Initial module families are expected to include:

- Home / Project Dashboard
- Write
- Story Development
- Review
- Listen / TTS
- Audiobook
- Publish / Export
- Git
- Terminal
- Settings

The exact iconography and final grouping may evolve, but the architectural rule does not: major subsystems enter through the shared shell rather than creating separate application chrome.

## 4. Home versus Project Workspace

New-book creation begins from Home using the guided **New Book Foundation** defined in `016-NEW-BOOK-FOUNDATION.md`. The setup is temporary until Create Book; creation seeds authoritative project files and opens the Story Development Wiki. The author chooses an outline/structure method later rather than during initial project creation.


### Catalyst Home

The application Home screen is project-oriented, not feature-oriented. It should emphasize:

- Continue Writing
- Recent Projects
- New Project
- Open Project Folder
- Import Project
- Clone from GitHub
- Recent activity

Audiobook Studio, Review Studio, Wiki, and similar project-bound tools should not dominate global Home as disconnected product buttons.

### Project Dashboard

Opening a project may present an overview of that book/project before entering a specific workspace. The dashboard can summarize:

- manuscript progress and last-edited location
- planning/knowledge counts and open story threads
- review round status
- TTS/audiobook status
- version-control state
- relevant project actions

The dashboard summarizes one project; it does not become another source of truth.

## 5. Write workspace

When **Write** is active:

- Workspace Navigation is primarily the actual project file/folder tree.
- Main Work Area is the manuscript editor.
- Context Panel is limited to the current scene or current chapter.

The file tree reflects real project files and folders. Catalyst may decorate entries with useful derived information such as word counts, status, POV, review indicators, or Git state, but those decorations must not replace the authoritative filesystem structure.

Example:

```text
▼ Manuscript
  ▼ Chapter 08
      Scene 01.md
      Scene 02.md
    ● Scene 03.md
      Scene 04.md
```

## 6. Context Panel: locked scope rule

The Context Panel is **scene-aware first and chapter-aware second**.

Default scope: **Scene**  
Optional scope toggle: **Chapter**

It must not become a whole-project Wiki browser.

Typical contextual categories include:

- characters
- locations
- timeline/date
- POV
- plot/story threads
- scene notes
- chapter notes
- review annotations/status
- other metadata relevant to the active scene/chapter

Conceptual control:

```text
CONTEXT

[ SCENE ] [ CHAPTER ]

Characters
Locations
Timeline
Story Threads
Notes
Review
```

Whole-project exploration belongs in the Story Development workspace.

## 7. Wikilinks and contextual drill-in

Catalyst should support Obsidian-style internal links such as:

```markdown
[[Leah]]
[[Harrison Grant]]
[[Royal Street]]
```

Typing `[[` in the manuscript should provide project-aware autocomplete. Completing or selecting a link resolves it against the project's knowledge files.

When a wikilink is activated in the editor, the linked entity opens **temporarily in the right Context Panel**. The manuscript remains in place.

Example:

```text
Context
‹ Back to Scene 3

LEAH
Character

Role: Protagonist
Occupation: Detective
...

[Open Full Wiki Page]
```

Returning closes the drill-in and restores scene/chapter context.

Aliases should be supported where practical, for example:

```markdown
[[Characters/Leah Warren|Leah]]
```

The link resolver may use a disposable/rebuildable index for speed, but the target Markdown/YAML files remain authoritative.

### Link-resolution contract

The shared resolver is a project service, not a Write-only or Story-Development-only store. Write, Story Development, search, Review, and the Context Panel consume the same resolution results.

- stable IDs are the durable identity of structured targets;
- wikilink text, titles, aliases, filenames, and paths are lookup/display inputs, never replacement identity;
- ambiguity is surfaced to the writer rather than resolved by guesswork;
- backlinks are derived projections and can be rebuilt from authoritative files;
- resolving or opening a link does not copy the target into the manuscript or Context Panel;
- ordinary project resolution is strictly scoped to the active project;
- legacy Kindling SQLite reference tables and reference-copy commands are migration/compatibility infrastructure and must not become the Catalyst link graph.

The Context Panel receives the active scene/chapter stable ID and resolved references from shared project services. It may keep transient navigation history and UI state, but it must not persist a competing copy of entity, canon, timeline, manuscript, or backlink data.

## 8. Scene/chapter context sources

Context can be assembled from both:

1. **Declared metadata** — explicit scene/chapter frontmatter or structured project files.
2. **Detected references** — wikilinks and other resolvable references found in the current scene/chapter.

Conceptual scene metadata:

```yaml
chapter: 8
scene: 3
pov: Leah
characters:
  - Leah
  - Harrison Grant
locations:
  - Royal Street
timeline:
  - 2024-06-10
threads:
  - Crane Investigation
  - Harrison Suspicion
```

Detected references supplement declared context; they do not create a second authoritative database.

## 9. Story Development workspace

The Story Development workspace is Catalyst's integrated story-development environment. It combines Plottr-style visual plotting and outlining with structured Story Development/Wiki knowledge, timeline, research, relationships, and linked Markdown. Plot/Outline is a first-class area inside Story Development, not a separate top-level module.

It provides whole-project development and knowledge navigation. Depending on the project template it may expose:

- Plot & Outline
- Visual story board / plotlines / acts / chapters / scenes / beats
- Timeline
- Story Development Wiki
- Characters
- Locations
- Organizations
- Relationships
- Plot Threads
- Research
- Worldbuilding
- Custom Collections

Templates can specialize these collections. A crime project may expose Cases, Evidence, Victims, Suspects, Law Enforcement, and Procedures; a fantasy project may expose Kingdoms, Magic, Religions, Creatures, and Artifacts.

The engine remains generic. Templates define project-specific structure.

## 10. Shared project model

Subsystems must consume the same project entities rather than maintaining duplicate copies.

For example, a single character file such as `Characters/Leah.md` can participate in:

- Wiki/Story Development Wiki
- manuscript wikilinks
- scene/chapter context
- timeline relationships
- review context
- TTS pronunciation/voice workflows
- Audiobook Studio character voice assignments

There must not be separate independent "Leah" records for each subsystem.

## 11. TTS Reader versus Audiobook Studio

These are related but distinct workflows built on shared narration infrastructure.

### TTS Reader

Purpose: **Read what I am working on to me.**

It supports proofreading/listening to current manuscript material and can use the active scene/chapter context.

### Audiobook Studio

Purpose: **Produce an audiobook from this manuscript.**

It owns production workflow such as:

- narrator/character voice assignments
- pronunciation rules
- segmentation
- generation/regeneration
- approvals
- production status
- mastering
- chapter audio
- provenance
- audiobook export

Both consume the same manuscript; neither becomes authoritative over manuscript text.

## 12. Review integration

Review Studio consumes the authoritative manuscript and project context but does not own the manuscript.

Review annotations may appear in the Context Panel when relevant to the active scene/chapter. Full reviewer/round navigation belongs in the Review workspace.

The native `lithomark-review` package architecture remains the review portability and distribution boundary.

## 13. File-authoritative project architecture

This workspace architecture reinforces Catalyst's existing storage rule:

- files and folders are authoritative
- no proprietary project container is required
- no database is required as the sole source of truth for user content
- Markdown/YAML/JSON and ordinary local folders are preferred
- caches and indexes must be disposable and rebuildable

Conceptual project structure:

```text
Project/
├── lithomark.yaml
├── Manuscript/
├── Characters/
├── Locations/
├── Timeline/
├── Research/
├── Notes/
├── Reviews/
│   ├── Rounds/
│   └── Distribution/
├── Audiobook/
│   ├── audiobook.yaml
│   ├── voices.yaml
│   ├── pronunciation.yaml
│   ├── chapters/
│   ├── renders/
│   └── exports/
├── Assets/
└── .git/
```

A user must be able to retain understandable project content even without Catalyst installed.

## 14. Module integration contract

Every new major Catalyst subsystem should answer these questions before implementation:

1. Which authoritative project files does it read?
2. Which authoritative project files does it write?
3. What disposable indexes/caches, if any, does it maintain?
4. What appears in Workspace Navigation?
5. What appears in the Main Work Area?
6. What information is appropriate for the Context Panel?
7. How does it reference stable project entities?
8. Does it duplicate data already owned elsewhere? If yes, redesign it.
9. Can its generated data survive and remain understandable outside Catalyst?
10. Does it preserve offline/local-first operation?


## 15. Git workspace operational contract

Git is a project-support tool, not the authority for Catalyst project state. The ordinary project files remain authoritative whether or not a project is under version control.

The Git workspace should provide writer-friendly operations for status, commit, pull, push, branch awareness, and conflict handling without requiring routine command-line use.

Catalyst must never silently discard or overwrite local work to complete a Git operation. Before an operation that could replace working-tree content, Catalyst must detect relevant local changes and surface the condition to the user.

The workspace must distinguish at minimum:

- clean working tree;
- uncommitted local changes;
- local commits not yet pushed;
- remote commits not yet pulled;
- diverged local and remote histories;
- merge/rebase conflicts;
- detached HEAD or otherwise unsupported branch state;
- missing or invalid remote configuration;
- authentication/authorization failure;
- repository unavailable/offline.

Pull and Push remain separate explicit actions. A combined sync action may be offered later, but it must not obscure which operations are being performed.

Merge conflicts must be treated as an explicit user-resolved state. Catalyst may assist with conflict presentation, but it must not automatically choose manuscript content during a conflict unless the user explicitly selects a resolution.

Files changed outside Catalyst are legitimate project changes. Catalyst must detect/reload or safely reconcile them rather than assuming all edits originate inside the application.

Git credentials and provider authentication are machine/user configuration and must not be stored as authoritative project content.

A project that has no Git repository, remote, or network connection must remain fully usable.

## 16. Optional runtime and component policy

Large or specialized runtimes, models, converters, validators, and provider-specific components should not automatically become mandatory dependencies of the Catalyst core installation.

Where practical, Catalyst should use a small core application plus optional locally managed components. Examples include narration runtimes/models, specialized publishing converters, distribution validators, or future computational tools.

Optional components must:

- be explicitly installable/enabled by the user;
- declare their version and compatibility requirements;
- expose installation/availability state to the relevant subsystem;
- fail gracefully when absent;
- remain replaceable where the subsystem architecture supports providers;
- avoid making project files inaccessible when removed;
- keep downloaded/runtime data outside authoritative project content unless the user intentionally stores a portable asset in the project.

A subsystem may recommend an optional component, but ordinary writing and access to the project must not depend on it.

Provider-specific model files are implementation resources, not project truth. Project configuration should reference stable Catalyst/provider identities and required capabilities rather than rely on machine-specific absolute model paths.

## 17. Project isolation and optional Series Workspace

The active Project Workspace has exactly one authoritative project context. Project search, wikilink autocomplete, entity resolution, canon/continuity checks, Review Studio, TTS project data, Audiobook Studio, Publishing Studio, and Git state are scoped to that active project unless the user explicitly enters a future cross-project/Series Workspace.

A future Series Workspace may sit above multiple independent books:

```text
Series Workspace
    ├── Book 1 — independent Catalyst project
    ├── Book 2 — independent Catalyst project
    └── Book 3 — independent Catalyst project
```

Its purpose is coordination, not consolidation. It may provide series chronology, recurring-entity references, series canon/continuity, book order, and series metadata while preserving each project's independent file tree and lifecycle.

Opening a book from a Series Workspace switches into that book's normal Project Workspace. The Series Workspace must not cause unrelated project entities to appear in ordinary book-level autocomplete, context, search, or editing flows.

Cross-project actions must be explicit and visually distinguishable from normal project-scoped actions.

## 18. UI implementation rule

Do not perform a large frontend redesign piecemeal while individual backend subsystems are still being proven.

Near-term frontend work should provide enough UI to validate native workflows such as Review. The broader frontend should then converge on this shared Project Workspace shell so future modules plug into a stable layout instead of forcing repeated redesigns.

## 19. Manuscript lifecycle integration

The shared shell must support the canonical Catalyst manuscript lifecycle defined in **012-CATALYST-WRITER-OS-INTEGRATION-ARCHITECTURE.md**:

> **Story Development → Write → Developmental Review → Revision → Beta Reading → Revision → ARC → Publish**

This is a navigational and workflow model, not a mandatory wizard. Workspaces share stable project identities and context so the writer can move backward and forward without importing, duplicating, or resynchronizing the project.

Review contains Developmental Review, Beta Reading, and ARC workflows. Revision is cross-workspace work performed primarily in Write and Story Development from durable review/revision items.

The first public Core Preview centers on **Write + Story Development**, including the Plot/Outline and Story Development/Wiki capabilities necessary to exercise the file-authoritative architecture with a real manuscript.

## 20. Locked interaction summary

The core UI grammar is:

> **Far left:** What part of Catalyst am I using?  
> **Left:** What item am I working with?  
> **Center:** What am I doing?  
> **Right:** What do I need to know right now?

For manuscript writing specifically:

> **Left:** real file/folder tree  
> **Center:** manuscript editor  
> **Right:** current scene context by default, current chapter context on demand

Wikilinks provide temporary contextual drill-in on the right without removing the writer from the manuscript.

This architecture is the baseline for future Catalyst UI and subsystem integration.
