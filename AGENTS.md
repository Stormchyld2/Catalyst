# Catalyst: Agent Instructions

These instructions apply to the whole repository and to every coding agent (ChatGPT, Codex, Claude). Read this file and [`CODING-GUARDRAILS.md`](CODING-GUARDRAILS.md) before changing anything.

**Coding style is mandatory.** `CODING-GUARDRAILS.md` governs how much code to write: edit existing files before adding new ones, no abstraction until it is used three times, no unrequested features or speculative config, small diffs, and honest reports. The architecture rules below are required. Do not add layers beyond them.

## 1. What Catalyst is

- Catalyst is an **offline-first tabletop RPG campaign and Game Master studio** for Windows. It is built with Svelte 5, TypeScript, Vite and Tauri 2, with a Rust backend in `src-tauri/`.
- It was forked from a snapshot of LithoMark, a writing app (see `docs/catalyst/MIGRATION-FROM-LITHOMARK.md`). Much of the code still uses LithoMark and Kindling names, and some writer-era features (manuscript, scenes, editorial review) are still present. Treat those as legacy: keep them working where they still run, but don't extend them or use them as defaults for new campaigns.
- **Never touch LithoMark.** `Stormchyld2/LithoMark` is a separate product. Don't read from it, copy data from it, import its projects automatically or push to it.
- The Character Builder lives in its own repo, `Stormchyld2/Catalyst-Character-Builder`, and connects to Catalyst through typed ports. Don't build a second character system here.

## 2. Authorities

These decide disputes, in this order. Docs 020–026 live on the `docs/campaign-wizard-architecture` branch until it merges; read them there if they aren't on your branch.

1. `docs/architecture/020-CAMPAIGN-WIZARD-ARCHITECTURE.md`: new campaigns, the folder tree, `campaign.yaml`, templates and the GM starter library.
2. `docs/architecture/023-…` and `024-…`: shared and module engines and who owns them. Check the code before declaring an engine done or starting a second copy (dice parser, rules evaluator, file service, search, renderer).
3. `docs/architecture/025-CATALYST-DESIGN-SYSTEM.md`: UI colors and branding.
4. `002-DOCUMENT-MODEL.md`, `003-STABLE-ID-SYSTEM.md` and `004-CANON-MODEL.md`: the file, ID and canon rules that still apply.
5. `001-PROJECT-FILE-FORMAT.md` and `016-NEW-BOOK-FOUNDATION.md`: legacy book projects only.

If the code and the docs disagree, investigate and report it. Don't silently pick one.

## 3. Storage and architecture rules

1. **Files are the source of truth.** User content lives in ordinary Markdown, YAML and JSON files in normal folders. No proprietary container, no authoritative database and no required cloud service, account or telemetry. A campaign must stay readable without Catalyst.
2. **Caches are disposable.** Search indexes, backlinks and projections must be rebuildable from the files. Older Kindling code still uses SQLite (`kindling.db`). Don't add new SQLite dependencies; see `018-SQLITE-RETIREMENT-AUDIT.md`.
3. **Projects are isolated.** Never resolve data across project boundaries.
4. **No fake data in production.** No mock, demo or hard-coded project data as a runtime fallback. Fixtures belong in tests and explicit demo harnesses only.
5. **Respect the layering:** UI → domain/project services → files on disk (through Rust commands). The UI must not become a second persistence layer or write project files directly. Don't create a competing store for anything that already has one.
6. **IDs are identity.** Stable IDs (type prefix plus UUID v4) identify records. Paths, filenames and titles do not. Moves and renames keep IDs and references. Report duplicate IDs; never silently regenerate them.
7. **Links:** `[[Name]]` and `[[Name|Label]]` resolve through the shared link resolver only, within the active project. Ambiguous matches go to the user; missing targets stay visibly unresolved. Don't rewrite authored Markdown just because a link resolved.
8. **Preserve user data.** Keep unknown YAML and frontmatter fields, Markdown whitespace and existing file meaning. Never silently move, delete or rewrite user files. Migrations are explicit and non-destructive.
9. **Safe paths.** Guard against path traversal. Keep generated file and folder names short (see the guardrails' pitfalls on Windows path limits).
10. **Canon is what the GM wrote.** Never turn inference into canon, and never change canon to fix a continuity warning.

## 4. Campaigns and templates

- New campaigns follow 020: a `project.yaml` with `project_type: campaign`, a `campaign.yaml`, and the approved campaign folder tree. No book or manuscript folders in new campaigns.
- Every template follows the wiki template contract: an entity template registered in `src-tauri/src/entity_templates.rs` with an image field, `overview_fields` and a scaffold, so records render in the wiki's right-hand Overview panel.
- Each campaign setting lives in exactly one file. Don't write the same value to both `project.yaml` and a campaign document.

## 5. Design system

- Theme: **Obsidian + Frost Blue.** Accent `#6EA9CF`, hover `#8CC0E1`, background `#111214`, panels `#202328` / `#2C3036`, text `#F0F1F3` / `#A3A9B2`, borders `#41464E`.
- Use the existing design tokens under `src/styles/` (`src/styles/catalyst/catalyst-design-tokens.css` once the theme branch lands). Don't hard-code colors or invent a new palette.
- Amber/gold is retired from app chrome. It may appear only as an optional dice or character-sheet skin.
- Don't change the logo artwork.
- Target WCAG 2.2 AA: keyboard access, visible focus, non-color state cues and reduced motion.

## 6. Commands

Run commands from the repo root unless noted. Start with the narrowest command and broaden only as needed.

| Purpose | Command |
|---|---|
| One front-end test file | `npx vitest run src/path/file.test.ts` |
| All front-end tests | `npm test` |
| Svelte/TypeScript check | `npm run check` |
| One Rust test or module | `cd src-tauri && cargo test <name>` |
| All Rust tests | `npm run test:rust` |
| Front-end build | `npm run build` |
| Lint and format checks | `npm run check:all` |
| Run the app | `npm run tauri dev` |

Windows is the primary platform. Code behind `#[cfg(windows)]` counts as tested only when its tests actually run on Windows.

## 7. Git

- Work on a feature branch with one task per branch and PR. Never commit to `main` directly or merge into it without the owner's approval. The LithoMark branch `migration/kindling-foundation` does not exist here.
- Never force-push, rewrite published history or delete branches or tags unless asked.
- Check `git status` before editing. Preserve unrelated changes, and don't edit files that another active branch owns.
- Don't commit build output, temp files, secrets or local environment state.

## 8. Testing and reporting

- Every behavior change needs a test. Bugs get a regression test.
- A failing test is a bug report: diagnose it and fix the cause. When a feature changes, rewrite its tests; never delete or weaken them to get green.
- Never claim tests passed unless they ran and passed.
- At the end of a task, report:
  - the files changed and the behavior changed;
  - the commands run, with their pass/fail results;
  - what was **not** verified;
  - remaining issues;
  - the branch and commit, or whether the changes are uncommitted.
- Don't assume the next agent remembers this conversation. Record lasting decisions in code, tests or docs.

## 9. When the rules conflict with the task

Stop before building something that breaks these rules or an architecture doc. Describe the conflict, propose the smallest change that resolves it, and wait for approval.

**Governing principle:** user ownership, portability, stable identity and simple, working code beat implementation convenience.

## Owner-approved personal-use content policy (2026-11-24)

Catalyst is for the owner's private use only; no redistribution or commercial release is planned. Comprehensive lawful local rules content is the target for D&D 2014/2024, Pathfinder 2e Remastered, Call of Cthulhu 7e and WFRP 4e, plus System-Neutral. Commercial redistribution approval is not a personal-installation gate; applicable access, copying and software-license terms still apply. Preserve source provenance and per-record/asset redistribution classifications: redistributable, personal-only, permission-required, unknown. Keep restricted payloads outside tracked Git and shared artifacts. Any future distribution requires explicit owner approval, verified rights, allowlist filtering, dependency checks and attribution. Do not build a speculative distribution pipeline or duplicate Character Studio engines. Follow the corresponding updates to 020–024 and 026 on the campaign architecture branch. GM/player access controls remain separate from content rights.

## Local content storage safeguard

Owner-approved public-repository safeguard: copyrighted sourcebook PDFs, restricted extracted rules data and restricted artwork must remain outside both Git working trees, in an owner-selected local content root (recommended Windows Desktop/Catalyst Content, organized by system/edition). Resolve the actual folder through user selection, never hardcode a username or Desktop path. Store local paths in untracked local settings; do not upload private content to GitHub, CI, logs or shared artifacts. A future private repository does not automatically authorize upload; require explicit owner approval and applicable rights. Keep local content separate from distributable code.
