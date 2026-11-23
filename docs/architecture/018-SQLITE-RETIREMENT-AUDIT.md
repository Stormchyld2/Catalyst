# SQLite retirement: dependency and data-safety gate

Status: **audit started; removal blocked until migration is verified** (2026-10-07).

## Findings verified in source

- `src-tauri/Cargo.toml` still declares `rusqlite = { version = "0.38", features = ["bundled"] }`.
- `src-tauri/src/lib.rs` registers `pub mod db`, initializes `AppState::new(app_data_dir)` during Tauri setup, and manages that state. The module header still describes SQLite as the persistence layer; this is transitional, not the target architecture.
- `src-tauri/src/commands/state.rs` owns a mutex-protected SQLite connection and opens `kindling.db`. It also contains database-version backup handling.
- `src-tauri/src/db/` contains schema, queries, writing, revisions, tags, session, fields, reference-copy and detection modules.
- `src-tauri/src/commands/crud.rs` calls database functions for projects, chapters, scenes, goals and more.
- `src-tauri/src/commands/project_root.rs` still uses the database for project lookup/registration and root updates.

**Do not remove the dependency, delete the database, or bypass state initialization yet.** This would break active command handlers and could strand existing projects.

## Required migration gates

1. Inventory all `state.db`, `crate::db`, `db::` and `rusqlite` uses across Rust source; map each public Tauri command to its replacement and test.
2. Establish a file-backed project registry/discovery path that can reopen existing project folders without consulting SQLite. Keep each book independent. Any global recent-projects list must be a disposable index of paths, not the authority for book content.
3. Migrate CRUD, goals, writing statistics, references, tags, sessions and revisions to their file-backed authorities, with compatibility reads where necessary.
4. Provide a deliberate, verified import/recovery route for legacy SQLite-only data. Never silently discard it or overwrite manuscript files.
5. Test a fresh installation, an existing SQLite installation, offline reopen, project rename/move, multi-book isolation and backup recovery.
6. Only after all callers are migrated: remove `AppState`'s connection, `db` module registration, `rusqlite` and transitive lockfile dependencies. Keep user databases/backups untouched unless the user explicitly requests deletion.
7. Run `cargo test`, `cargo check`, `npm run check` and targeted integration tests before declaring retirement complete.

## Related priorities

- Manuscript cutover tests now include same-chapter reorder and cross-chapter movement after activation. These tests alone do not prove that all SQLite-backed commands have been replaced.
- Book title display in `src/App.svelte` is committed; verify in a running desktop build.
- Relationship Map backend should use project-local files, stable entity IDs and explicit relationship IDs, never a new required database.

## Completion criterion

SQLite is retired only when the application starts and its supported commands work without opening `kindling.db`, all legacy-only data has a safe migration path, and the Rust dependency is removed. Until then, report it as **in progress**.
