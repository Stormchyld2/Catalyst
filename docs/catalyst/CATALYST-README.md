# Catalyst
### Writer OS

**Your stories. Your workspace. Your files.**

Catalyst is a desktop writing application in development for authors who want to plan, draft, organize, revise, and prepare their books without juggling a collection of disconnected tools.

The goal is straightforward: bring the manuscript and the world around it into one focused workspace—while keeping the writing in ordinary files that belong to you.

> **Development status:** Catalyst is under active development. This repository is not a finished consumer release. Features described below include both work in progress and longer-term plans; availability may change as the application evolves.

## One place for the writing process

A novel is more than a manuscript. It has characters, locations, research, outlines, continuity notes, revisions, and feedback. Catalyst is being designed to keep those pieces connected without forcing authors into a rigid process.

### Write

A dedicated manuscript workspace is being developed for drafting and editing chapters, with chapter navigation, writing progress, and access to relevant story information alongside the text.

### Develop your story

Organize characters, locations, worldbuilding, organizations, items, and general reference material. The Story Development workspace is designed to connect planning information with the manuscript through links and familiar document-based navigation.

### Outline and plan

Plan chapters and scenes, track story beats, and refine structure. A more visual outlining workspace is planned as development continues.

### Review and revise

The roadmap includes tools for developmental review, revision, and sharing manuscripts with beta readers or editors—including a separate, free read-only reviewer experience so collaborators won't need the full writing application.

### Listen, produce, and publish

Planned capabilities include text-to-speech reading, an optional audiobook production studio, and publishing/export tools for preparing formatted manuscripts. These are not all available in the current development build.

## Built around your files

**Your writing should outlast the software used to create it.**

Catalyst follows a local-first approach:

- Projects live in regular folders on your computer.
- Manuscripts and planning material use portable formats such as Markdown, YAML, and JSON wherever practical.
- No proprietary project container is required for your writing.
- Local work does not depend on an internet connection.
- Git and GitHub integration is being developed as an optional way to manage versions and synchronize projects—not as a requirement for writing.

Each book can have its own project folder, keeping manuscripts and supporting material separate.

## A workspace that fits the author

Catalyst draws inspiration from the flexibility of Markdown knowledge bases, the organization of dedicated writing planners, and the practical tools of desktop development environments.

The design direction includes a dark, readable interface, customizable writing workflows, templates, linked reference material, and optional tools that authors can use without cluttering the drafting experience.

The long-term vision includes community-created themes and plugins. AI-generated writing is **not** a built-in requirement or the focus of the application.

## Where development stands

Catalyst is currently being built as a **Windows desktop application** using Tauri, Svelte, TypeScript, and Rust.

Current development is focused on the foundations of the writing experience: local projects, manuscript editing, Story Development, chapter and scene planning, and the connections between them. These areas are still being implemented and tested.

Future development is expected to expand the outlining, review, accessibility, audio, export, and customization experiences. The feature set and release schedule are subject to change.

**There is no public download or release date announced in this README.**

## Follow the project

Catalyst is being developed for writers who want a practical, connected workspace without surrendering control of their manuscripts.

Project updates, screenshots, and preview announcements will be shared as development progresses.

- [The Draft Society on Discord](https://discord.gg/ZAm38KDXdz)
- [Jason Williams — Author website](https://authorjasonwilliams.com)

## For developers and contributors

This repository contains the application source and development documentation. Architecture decisions and implementation details are maintained in the repository's `docs/` directory.

Catalyst is a work in progress; the presence of source code does not mean a feature is complete or ready for general use.

## License

A project license has not yet been announced. Do not assume the repository is licensed for redistribution or commercial reuse merely because its source is visible.

---

*Catalyst — Writer OS. Keep the story together. Keep the writing yours.*
