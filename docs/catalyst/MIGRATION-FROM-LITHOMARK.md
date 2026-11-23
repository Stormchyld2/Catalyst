# Catalyst foundation: migration from LithoMark

Catalyst was established from a snapshot of the LithoMark codebase. Catalyst is an
independent repository: no LithoMark Git history was imported.

| | |
|---|---|
| Source repository | `Stormchyld2/LithoMark` |
| Source branch | `migration/kindling-foundation` |
| Source commit | `96e640cf7eb88cc5c0b586d5b946d09337ee0a99` ("Allow preserved legacy chapter backup after scene authority cutover", 2026-10-13) |
| Migration date | 2026-11-23 |
| Files in source | 532 tracked files |
| Files transferred | 531 (530 at their original paths, plus the LithoMark README relocated, see below) |

This is a foundation migration only. Application code, internal names (`lithomark`,
`com.lithomark.writer`, `kindling` identifiers), UI and module structure are unchanged
and will be adapted in later stages.

## What was preserved from Catalyst

- `README.md` (the original Catalyst README) is kept at the repository root.
- Catalyst contained no branding assets (no `images/` directory, logo or icon) at the
  time of migration, so nothing needed protecting there. The LithoMark icons and logos in
  `src-tauri/icons/`, `static/` and `src/assets/` came across unchanged as the app's
  current build assets; approved Catalyst artwork should be added to `images/` when ready.

## Files not transferred as-is

| File | Handling | Reason |
|---|---|---|
| `README.md` (LithoMark) | Moved to `docs/lithomark/LITHOMARK-README.md` | Catalyst's own README is kept at the root. |
| `.draft-script/analytics-cache.json` | Excluded | 9.5 MB generated editor analytics cache (file list, mtimes, word counts of docs). Not referenced by the application, build or tests. |
| LithoMark `.git` history | Excluded | Catalyst must remain independent. |

Untracked/ignored artifacts (`node_modules`, `dist`, `src-tauri/target`) were never part
of the copy. No secrets, keys or credential files were found in the source.

## Licensing note

`package.json` declares `"license": "MIT"`, but the source has no `LICENSE` file. The
bundled fonts carry their own OFL licenses in `static/fonts/licenses/`. Add a `LICENSE`
file to Catalyst when the licensing decision is made.

## Transitional SQLite dependencies (for a later controlled migration)

The file-authoritative architecture (Markdown/YAML/JSON project folders, stable IDs,
Document Service) is the target. Older Kindling-derived code still uses SQLite. These
were intentionally **not** removed. See `docs/architecture/018-SQLITE-RETIREMENT-AUDIT.md`
for the required migration gates.

- Crate dependency: `rusqlite = { version = "0.38", features = ["bundled"] }` in `src-tauri/Cargo.toml`.
- Database: `kindling.db` in the app data directory, opened by `AppState` in
  `src-tauri/src/commands/state.rs` (which also keeps pre-upgrade database backups);
  initialized during Tauri setup in `src-tauri/src/lib.rs` (`pub mod db`).
- `src-tauri/src/db/`: `schema.rs`, `queries.rs`, `writing.rs`, `revisions.rs`, `tags.rs`,
  `session.rs`, `fields.rs`, `reference_copy.rs`, `detect.rs`.
- Command modules using `state.db` / `crate::db` / `rusqlite`
  (`src-tauri/src/commands/`): `archive`, `blank_project`, `crud`, `demo_fixture`, `detect`,
  `editorial`, `export`, `export_prototype`, `fields`, `import`, `lock`, `novelwriter_sync`,
  `reference_copy`, `revisions`, `sample_revisions`, `screenplay_project`, `search`,
  `session`, `snapshot`, `state`, `sync`, `tags`, `templates`, `writing`.
- Other Rust modules: `detect.rs`, `models/snapshot.rs`, `write_projection.rs`,
  `project_discovery.rs`, `project_registry.rs`, `parsers/novelwriter/writer.rs` (+ tests).
- Frontend coupling: `src/lib/stores/synopsisSaves.svelte.ts` matches on rusqlite's
  error text for the synopsis lock lookup.

## Validation at migration time (Linux, Node 22, Rust 1.97)

Every frontend failure below was reproduced on an untouched checkout of the source
commit with identical results, so all are inherited, not caused by the migration.

| Step | Command | Result |
|---|---|---|
| Install | `npm ci` | Pass |
| Svelte/TS check | `npm run check` | Pass, 0 errors, 0 warnings |
| Frontend tests | `npm test` | **Fail**: 970 passed, 101 failed, 4 skipped (17 of 100 files). Same 101 failures in the source. |
| Rust tests | `npm run test:rust` | Pass, 857 passed |
| ESLint | `npm run lint` | **Fail**: no `eslint.config.js` exists in the source |
| Clippy | `npm run lint:rust` | **Fail**: 18 lint errors with `-D warnings` (unneeded clones, too many arguments, etc.) |
| Rust format | `npm run format:rust:check` | **Fail**: source is not `cargo fmt`-clean |
| Prettier | `npm run format:check` | **Fail**: 70 files not formatted |
| Production build | `npm run build` | **Fail**: `src/main.ts` imports `./app.css` but the file is `src/App.css`. Works on case-insensitive macOS/Windows file systems, fails on Linux. With that one case mismatch worked around locally (not committed), `vite build` succeeds. |

Other inherited notes:

- `package.json` scripts reference `qa/` and `e2e/` directories that are not present in
  the source branch (`test:qa-harness`, `qa:visual`, `test:e2e`, `tauri:qa`, `seed:demo`, ...).
  `src/test/dragWithMouseEvents.test.js` fails for the same reason.
- `package.jason` is an empty file in the source (likely a typo); kept as-is.
- A full `tauri build` bundle was not attempted because its `beforeBuildCommand`
  (`npm run build`) fails on Linux as above.
