# LithoMark New Book Foundation

**Status:** Locked architecture  
**Scope:** New-book creation, initial Story Development seeding, and handoff into the normal project workspace  
**Principle:** The setup experience seeds ordinary authoritative project files; it never becomes a competing project database.

## 1. Purpose

Creating a new book should not drop the writer into an empty file tree or require the writer to understand LithoMark's internal folder structure before beginning.

LithoMark therefore provides a guided **New Book Foundation** flow from Home. The flow gathers enough initial story-development information to seed the project, then creates normal Markdown/YAML/JSON files and opens the project in the **Story Development Wiki**.

The setup form is temporary. After project creation, the generated files become authoritative.

## 2. User flow

```text
LithoMark Home
    ↓
New Book
    ↓
New Book Foundation
    ↓
Project Basics
    ↓
Characters
    ↓
Locations
    ↓
Worldbuilding
    ↓
Style & Voice
    ↓
Themes & Core Ideas
    ↓
Brainstorming
    ↓
Synopsis
    ↓
Review & Create Book
    ↓
Generate authoritative project files
    ↓
Open Story Development Wiki
    ↓
Continue development
    ↓
Outline / Structure when the author is ready
```

The author may move backward and forward through setup. Individual fields and nonessential sections may be optional where their specification says so, but **Worldbuilding is a required foundation stage**. The writer may enter a minimal worldbuilding foundation and expand it later; the flow must not silently skip the stage entirely.

Story-development outline methodology is deliberately deferred until after the initial foundation. The Manuscript Structure step may seed the manuscript's chapter/scene scaffold, but it does not select or replace the writer's plotting methodology.

## 3. Initial sections

### Project Basics

Expected fields may include:

- title;
- project type;
- primary genre;
- subgenre(s);
- target audience;
- optional target length;
- optional series information.

### Characters

Characters are repeatable structured entries. Initial setup should collect only enough information to seed a useful character file, for example:

- name;
- role;
- aliases;
- short description.

The full Character template remains available after project creation for deeper development.

### Locations

Locations are repeatable structured entries, for example:

- name;
- type;
- aliases;
- short description.

The full Location template remains available after project creation.

### Worldbuilding

Worldbuilding is mandatory as a foundation category for a new book. Required completion means the writer has deliberately established at least a minimal world context; it does not require exhaustive lore or a specific genre-oriented checklist.

Worldbuilding may collect broad initial information such as:

- setting;
- time period;
- organizations;
- factions;
- systems/rules;
- other project-specific notes.

This section seeds the Wiki; it is not intended to replace later detailed worldbuilding files.

### Style & Voice

May include:

- POV;
- tense;
- tone;
- style notes;
- voice notes;
- optional comparison/influence notes.

### Themes & Core Ideas

May include:

- themes;
- central conflict;
- stakes;
- story intent;
- recurring motifs or ideas.

### Brainstorming

**Brainstorming** is the preferred LithoMark term. Do not use "Braindump" as the product label.

Brainstorming occurs after the structured entity/setup sections so the setup session already knows the names entered for characters, locations, organizations, and similar entities.

The first implementation may use free-form text. Visual card, canvas, or Kanban planning can be added later without changing the authoritative file model.

### Synopsis

Synopsis follows Brainstorming. The writer summarizes the story after establishing the major ingredients and exploring ideas.

## 4. Setup-session data

Before the writer presses Create, setup data is temporary application state.

It must not create permanent project files on every keystroke.

Example temporary entity:

```text
temp-character-1
name: Leah Warren
aliases:
  - Leah
role: protagonist
```

Temporary setup state may be discarded if the writer cancels project creation.

## 5. Creation boundary

Pressing **Create Book** is the persistence boundary.

At that point LithoMark:

1. validates the destination/project root;
2. creates the normal LithoMark project structure;
3. assigns stable LithoMark IDs;
4. creates the initial Markdown/YAML/JSON files;
5. populates those files from the setup data;
6. creates any required project metadata;
7. opens the new project;
8. navigates directly to Story Development / Wiki.

Once created, those files are authoritative.

The setup state must not remain a second hidden source of truth.

## 6. Initial generated files

The exact template-dependent structure may evolve, but a book can conceptually begin with:

```text
Book/
├── project.yaml
├── Manuscript/
├── Story Development/
│   ├── Foundation.md
│   ├── Brainstorming.md
│   ├── Synopsis.md
│   ├── Style & Voice.md
│   ├── Themes.md
│   └── Worldbuilding/
├── Characters/
│   ├── Character One.md
│   └── Character Two.md
├── Locations/
│   └── Location One.md
└── Planning/
```

Existing LithoMark templates should be used to create entity files where practical. Setup values populate the appropriate initial template fields; the writer completes the richer files afterward in the Wiki.

## 7. Entity recognition during setup

The first implementation does **not** require automatic entity recognition in Brainstorming.

Characters, locations, and similar entities are collected through structured setup fields. This is sufficient to create their initial files reliably.

A later enhancement may maintain an in-memory entity list during setup so names already entered can be highlighted or resolved inside Brainstorming and Synopsis text.

That enhancement must not require AI.

## 8. Entity recognition after creation

After project creation, entity resolution operates against authoritative project files and a disposable/rebuildable index.

Explicit `[[wikilinks]]` remain supported, but ordinary writing should not require the author to manually insert wikilinks everywhere.

Later deterministic entity recognition may use:

- entity names;
- aliases;
- stable IDs;
- known entity types;
- explicit user disambiguation where names collide.

The manuscript text remains ordinary text unless the writer explicitly chooses to insert a link.

## 9. Outline boundary

Outline is not part of the mandatory initial foundation.

Story Development answers:

> What is this story?

Outline answers:

> How does this story unfold?

When the writer chooses **Start Outline**, LithoMark may ask the writer to choose a structure/template such as:

- Blank Outline;
- Three-Act;
- Save the Cat;
- Hero's Journey;
- Crime / Mystery;
- Thriller;
- Romance;
- custom user template.

Outline templates may provide structure-specific prompts and files. They must not rewrite or replace the Story Development foundation.

## 10. Canvas / Kanban planning

A planning canvas or Kanban board belongs inside Story Development and may be available immediately after project creation.

The first version does not need bidirectional semantic analysis. A board may simply store explicit cards/columns in a normal project file.

Preferred storage must remain portable and understandable, using Markdown/YAML/JSON rather than an opaque proprietary database.

## 11. Relationship to templates

Templates have two roles:

1. **Entity/file templates** seed generated Character, Location, Worldbuilding, and other files.
2. **Outline/structure templates** are selected later when the writer begins outlining.

The New Book Foundation must not require the writer to select a plotting methodology before developing the story. Manuscript Structure is a separate concern: it may establish the initial chapter/scene scaffold while plot/outline methodology remains optional and editable later.

## 12. AI policy

The New Book Foundation is fully functional without AI.

Core behavior consists of:

- structured forms;
- deterministic validation;
- template/file generation;
- stable IDs;
- navigation;
- optional deterministic entity matching.

A future AI plugin may assist with suggestions, expansion, or synthesis through the public Plugin API, but Core must not depend on AI for project creation or Story Development.

## 13. Governing rule

The New Book Foundation is a **project seeder**, not a permanent parallel model.

Before Create:
> setup state is temporary.

After Create:
> project files and stable IDs are authoritative.


## 14. Current implementation checkpoint

The first backend seeding path is now implemented as a local-file-only service.

Current behavior:

- creates a validated `project.yaml` without requiring SQLite project state;
- creates the canonical local project directories, including Story Development and Planning;
- seeds Foundation, Brainstorming, Synopsis, Style & Voice, Themes, and Worldbuilding files;
- creates Character and Location files with stable LithoMark IDs;
- stores structured setup values in YAML frontmatter and prose notes in Markdown bodies;
- sanitizes unsafe filename characters while preserving the human-facing document title;
- handles duplicate entity names by creating distinct normal files;
- returns the manifest and resolved local paths to the caller.

The guided frontend is now implemented. It holds setup state only in the New Book Foundation UI and passes that temporary state to the file-authoritative command when the writer chooses Create Book. Successful creation activates the new local project and opens Story Development directly.


## 15. Entity template registry

Entity/file templates are distinct from outline/structure templates.

LithoMark maintains one logical entity-template registry assembled from:

- built-in entity templates shipped with LithoMark;
- user-authored Markdown template files discovered recursively from the local `Templates` folder.

A user template is a normal Markdown file with YAML frontmatter under a `lithomark_template` root key. The template metadata declares its stable template id, display name, category, target document type, optional subtype, version, description, and structured fields. The Markdown body supplies the initial document body/section scaffold.

User template discovery rules:

- subfolders are allowed and exist only for user organization;
- YAML metadata determines the template's actual category/type;
- template IDs must be unique;
- a user template may not silently replace a built-in template with the same ID;
- invalid templates are reported as registry issues without hiding valid templates;
- adding a valid Markdown template to the local `Templates` folder makes it available after registry refresh/reload;
- no database import step is required.

LithoMark provides a **Manage Templates** UI for creating, editing, renaming, and deleting user templates. It writes the same ordinary Markdown/YAML template files that advanced users can create or edit manually. Built-in templates remain read-only.

### Foundation assignment behavior

During Review & Create, entities receive a compatible default template automatically based on their resolved type/subtype. Examples include:

- Character → Character Profile;
- Location → Location Profile;
- Government → Government Profile (`organization` document type, `government` subtype);
- Corporation/faction/institution → Organization Profile;
- Object/artifact → Object / Item Profile;
- Event → Event Profile;
- Research topic → Research Profile;
- general setting material → Worldbuilding Profile.

The Review & Create screen must let the writer correct the entity type and override the selected compatible template before files are generated.

Automatic assignment is a convenience, not an irreversible classification.

## 16. Demo project

LithoMark will include an explicit sample/demo project containing synthetic material created specifically for the application. It should demonstrate the normal project model with, at minimum:

- a small fictional world;
- multiple characters;
- multiple locations;
- one or more organizations/governments;
- general worldbuilding material;
- Story Development files;
- a small outline/planning example;
- approximately two short manuscript chapters with scenes.

The demo is not fallback runtime data. Opening the demo copies or creates a normal local LithoMark project folder, after which it behaves exactly like a user-created project and all content remains file-authoritative.


## 17. Story Development workspace presentation

Story Development uses a persistent three-region workspace with editor tabs:

- left: hierarchical local project file tree;
- top: open document/image tabs;
- center: Markdown Read/Edit/Split modes or a local image viewer;
- right: template-driven Overview card for Markdown entity files.

Tabs are UI/session state only. They point to authoritative local files and never become a content store.

Supported Story Development images are ordinary local project assets (PNG, JPG/JPEG, WEBP, GIF, SVG). Image files may appear in the navigation tree and open directly in the center viewer. Entity files may reference a project-relative image path through their template-defined image field (default: `image`), allowing the Overview card to display a portrait or visual reference.

Entity templates declare:

- structured fields;
- which fields appear in the compact Overview card;
- the image field used by the Overview card;
- a richer Markdown body scaffold for deep story-development notes.

The built-in templates intentionally support deep character, location, organization/government, worldbuilding, object/item, event, and research development while remaining ordinary Markdown/YAML files.


## 18. Guided UI checkpoint

The New Book Foundation UI is implemented with the locked ten-step sequence:

Project Basics → Characters → Locations → Worldbuilding → Style & Voice → Themes/Core Ideas → Brainstorming → Synopsis → Manuscript Structure → Review & Create.

Current behavior:

- optional development steps may be skipped;
- Character and Location rows are validated when present;
- the writer explicitly chooses the local destination folder;
- setup values remain temporary until Create Book;
- Create Book invokes the local-file seeder rather than creating database-authoritative project content;
- the resulting project is activated with its project root and opened directly in Story Development;
- Story Development retains its completed tabs, image handling, Overview/context, Contents navigation, Markdown editor, Split mode, search, and tab state behaviors.

Entity Review / template assignment remains a distinct integration concern. The guided UI does not force a plotting methodology during project creation. The Manuscript Structure step seeds only the initial manuscript structure selected by the writer and remains subordinate to the file-authoritative Manuscript Structure Service.
