//! Tauri IPC Command Handlers
//!
//! This module contains all the command handlers that the frontend can call
//! via Tauri's `invoke()` API. Commands are organized into submodules:
//!
//! - [`state`]: Application state management
//! - [`import`]: Import commands for Plottr, Markdown
//! - [`crud`]: CRUD operations for projects, chapters, scenes, beats
//! - [`sync`]: Sync/reimport functionality
//! - [`archive`]: Archive and restore commands
//! - [`lock`]: Lock/unlock commands
//! - [`export`]: Export commands for Markdown, DOCX
//! - [`snapshot`]: Snapshot/versioning commands
//! - [`settings`]: App-wide settings
//! - [`feedback`]: Feedback payload model, builder, and validation

mod archive;
mod blank_project;
mod crud;
#[cfg(debug_assertions)]
mod demo_fixture;
mod detect;
mod entity_templates;
mod export;
mod export_prototype;
pub mod feedback;
mod fields;
mod import;
mod lock;
mod link_resolver;
mod manuscript;
mod new_book_foundation;
mod campaign_wizard;
mod native_review;
mod novelwriter_sync;
mod project_root;
mod reference_copy;
mod sample_project;
mod screenplay_project;
mod search;
mod session;
mod settings;
mod snapshot;
pub(crate) mod staging;
mod soundboard;
mod story_development;
mod state;
mod sync;
mod system_tools;
mod tags;
mod templates;
mod workspace_export;

// Re-export everything for backwards compatibility with lib.rs
pub use archive::*;
pub use blank_project::*;
pub use crud::*;
#[cfg(debug_assertions)]
pub use demo_fixture::*;
pub use detect::*;
pub use entity_templates::*;
pub use export::*;
pub use export_prototype::*;
pub use feedback::*;
pub use fields::*;
pub use import::*;
pub use native_review::*;
pub use new_book_foundation::*;
pub use campaign_wizard::*;
pub use project_root::*;
pub use lock::*;
pub use link_resolver::*;
pub use reference_copy::*;
pub use sample_project::*;
pub use screenplay_project::*;
pub use search::*;
pub use session::*;
pub use settings::*;
pub use snapshot::*;
pub use soundboard::*;
pub use story_development::*;
pub use state::*;
pub use sync::*;
pub use system_tools::*;
pub use tags::*;
pub use templates::*;
pub use workspace_export::*;

mod writing;
pub use writing::*;

mod revisions;
pub use revisions::*;
pub mod editorial;
pub use editorial::*;
