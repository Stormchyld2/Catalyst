//! File-authoritative Catalyst document persistence.
//!
//! Markdown/YAML files are project truth. This service deliberately has no
//! database dependency: SQLite may index documents later, but deleting an index
//! must never delete or make these documents unrecoverable.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

const FRONTMATTER_DELIMITER: &str = "---";
const TEMP_PREFIX: &str = ".catalyst-save-";
/// Recovery artifacts written before the Catalyst rename. Still recognised.
const LEGACY_TEMP_PREFIX: &str = ".lithomark-save-";

/// Frontmatter key holding a document's stable identity.
pub const IDENTITY_KEY: &str = "catalyst";
/// Identity key used by documents written before the Catalyst rename. It is
/// still read, and a legacy file keeps its key when saved, so existing
/// projects are never silently migrated.
pub const LEGACY_IDENTITY_KEY: &str = "lithomark";

/// Project-local hidden folder (trash and similar app state).
pub const APP_DIR: &str = ".catalyst";
/// Hidden folder used before the Catalyst rename; still read for trash entries.
pub const LEGACY_APP_DIR: &str = ".lithomark";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentIdentity {
    #[serde(rename = "type")]
    pub document_type: String,
    pub id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CatalystDocument {
    pub identity: DocumentIdentity,
    pub title: Option<String>,
    pub metadata: BTreeMap<String, Value>,
    pub body: String,
    pub relative_path: PathBuf,
    pub source_hash: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SaveResult {
    pub source_hash: String,
    pub bytes_written: usize,
}

pub fn read_document(project_root: &Path, relative_path: &Path) -> Result<CatalystDocument, String> {
    let path = resolve_project_path(project_root, relative_path)?;
    if !path.is_file() {
        return Err(format!("Document '{}' does not exist.", relative_path.display()));
    }

    let raw = fs::read_to_string(&path)
        .map_err(|e| format!("Could not read document '{}': {e}", path.display()))?;
    let mut document = parse_document(relative_path, &raw)?;
    // Conflict hashes represent the bytes currently on disk, not the
    // normalized in-memory representation used by the parser.
    document.source_hash = sha256_hex(raw.as_bytes());
    Ok(document)
}

pub fn create_document(
    project_root: &Path,
    relative_path: &Path,
    document_type: &str,
    title: Option<&str>,
    body: &str,
) -> Result<CatalystDocument, String> {
    validate_document_type(document_type)?;
    let path = resolve_project_path(project_root, relative_path)?;
    if path.exists() {
        return Err(format!("Document '{}' already exists.", relative_path.display()));
    }

    let document = CatalystDocument {
        identity: DocumentIdentity {
            document_type: document_type.trim().to_string(),
            id: generate_stable_id(document_type)?,
        },
        title: title.map(str::trim).filter(|v| !v.is_empty()).map(str::to_string),
        metadata: BTreeMap::new(),
        body: normalize_body(body),
        relative_path: relative_path.to_path_buf(),
        source_hash: String::new(),
    };

    save_document(project_root, &document, None)?;
    read_document(project_root, relative_path)
}


/// Move or rename an authoritative Markdown document without changing identity.
///
/// The caller must provide the hash observed when the document was loaded. This
/// prevents a UI action from moving a file that another editor changed after
/// the user opened it. The destination is never overwritten.
pub fn move_document(
    project_root: &Path,
    from_relative_path: &Path,
    to_relative_path: &Path,
    expected_source_hash: &str,
) -> Result<CatalystDocument, String> {
    if from_relative_path == to_relative_path {
        let current = read_document(project_root, from_relative_path)?;
        if current.source_hash != expected_source_hash {
            return Err(format!(
                "Document conflict: '{}' changed on disk after it was loaded.",
                from_relative_path.display()
            ));
        }
        return Ok(current);
    }

    let source = resolve_project_path(project_root, from_relative_path)?;
    let destination = resolve_project_path(project_root, to_relative_path)?;
    if !source.is_file() {
        return Err(format!("Document '{}' does not exist.", from_relative_path.display()));
    }
    if destination.exists() {
        return Err(format!("Document '{}' already exists.", to_relative_path.display()));
    }

    let current = read_document(project_root, from_relative_path)?;
    if current.source_hash != expected_source_hash {
        return Err(format!(
            "Document conflict: '{}' changed on disk after it was loaded.",
            from_relative_path.display()
        ));
    }

    let parent = destination
        .parent()
        .ok_or_else(|| "Document destination has no parent.".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("Could not create document directory '{}': {e}", parent.display()))?;

    fs::rename(&source, &destination).map_err(|e| {
        format!(
            "Could not move document '{}' to '{}': {e}",
            from_relative_path.display(),
            to_relative_path.display()
        )
    })?;

    let moved = read_document(project_root, to_relative_path)?;
    if moved.identity != current.identity {
        return Err("Document identity changed during move.".into());
    }
    Ok(moved)
}

/// Move a document into a project-local, recoverable trash entry.
///
/// The original path and stable identity are retained in the trash manifest.
/// No linked documents are altered; unresolved references remain visible.
pub fn delete_document(
    project_root: &Path,
    relative_path: &Path,
    expected_source_hash: &str,
) -> Result<DocumentIdentity, String> {
    let path = resolve_project_path(project_root, relative_path)?;
    let current = read_document(project_root, relative_path)?;
    if current.source_hash != expected_source_hash {
        return Err(format!(
            "Document conflict: '{}' changed on disk after it was loaded.",
            relative_path.display()
        ));
    }

    let trash_root = project_root.join(APP_DIR).join("trash");
    // Refuse to write into a redirected trash folder outside the project.
    for directory in [project_root.join(APP_DIR), trash_root.clone()] {
        if directory.is_symlink() {
            return Err(format!("Trash directory '{}' must not be a symbolic link.", directory.display()));
        }
    }
    fs::create_dir_all(&trash_root)
        .map_err(|e| format!("Could not create document trash: {e}"))?;
    let entry = trash_root.join(Uuid::new_v4().to_string());
    fs::create_dir(&entry)
        .map_err(|e| format!("Could not create document trash entry: {e}"))?;
    let manifest = entry.join("original-path.txt");
    if let Err(error) = fs::write(&manifest, relative_path.to_string_lossy().as_bytes()) {
        let _ = fs::remove_dir(&entry);
        return Err(format!("Could not record original document path: {error}"));
    }

    // Recheck immediately before relocation to catch external edits while the
    // trash manifest was being prepared. Never overwrite an existing entry.
    let actual = fs::read(&path)
        .map_err(|e| format!("Could not recheck document before deletion: {e}"))?;
    if sha256_hex(&actual) != expected_source_hash {
        let _ = fs::remove_dir_all(&entry);
        return Err(format!(
            "Document conflict: '{}' changed on disk during deletion.",
            relative_path.display()
        ));
    }
    if let Err(error) = fs::rename(&path, entry.join("document.md")) {
        let _ = fs::remove_dir_all(&entry);
        return Err(format!("Could not move document into recoverable trash: {error}"));
    }
    Ok(current.identity)
}

/// A project-local deleted document available for explicit restoration.
#[derive(Clone, Debug)]
pub struct TrashedDocument {
    pub entry_id: String,
    pub original_relative_path: PathBuf,
    pub identity: DocumentIdentity,
    pub source_hash: String,
}

/// List valid trash entries without modifying the project. Invalid entries
/// remain untouched; the caller can report a scan error to the author.
pub fn list_trashed_documents(project_root: &Path) -> Result<Vec<TrashedDocument>, String> {
    if !project_root.is_absolute() || !project_root.is_dir() {
        return Err("Trash listing requires an existing absolute project directory.".into());
    }
    let mut results = Vec::new();
    // New deletions go to .catalyst/trash; entries made before the rename in
    // .lithomark/trash stay listable and restorable.
    for app_dir in [APP_DIR, LEGACY_APP_DIR] {
        list_trash_root(project_root, &project_root.join(app_dir).join("trash"), &mut results)?;
    }
    results.sort_by(|a, b| a.original_relative_path.cmp(&b.original_relative_path));
    Ok(results)
}

fn trash_entry_dir(project_root: &Path, entry_id: &str) -> PathBuf {
    let current = project_root.join(APP_DIR).join("trash").join(entry_id);
    if current.exists() {
        current
    } else {
        project_root.join(LEGACY_APP_DIR).join("trash").join(entry_id)
    }
}

fn list_trash_root(project_root: &Path, trash: &Path, results: &mut Vec<TrashedDocument>) -> Result<(), String> {
    if !trash.exists() { return Ok(()); }
    let root = fs::canonicalize(project_root).map_err(|e| format!("Invalid project root: {e}"))?;
    let verified_trash = fs::canonicalize(trash).map_err(|e| format!("Invalid trash directory: {e}"))?;
    if !verified_trash.starts_with(&root) || fs::symlink_metadata(trash)
        .map_err(|e| format!("Cannot inspect trash: {e}"))?.file_type().is_symlink() {
        return Err("Trash directory must be inside the project and not a symlink.".into());
    }
    for entry in fs::read_dir(trash).map_err(|e| format!("Cannot list trash: {e}"))? {
        let entry = entry.map_err(|e| format!("Cannot inspect trash entry: {e}"))?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() || entry.file_type().map_err(|e| e.to_string())?.is_symlink() {
            continue;
        }
        let entry_id = entry.file_name().to_string_lossy().into_owned();
        if Uuid::parse_str(&entry_id).is_err() { continue; }
        let manifest = entry.path().join("original-path.txt");
        let document = entry.path().join("document.md");
        if !manifest.is_file() || !document.is_file() { continue; }
        if fs::symlink_metadata(&manifest).map_err(|e| e.to_string())?.file_type().is_symlink()
            || fs::symlink_metadata(&document).map_err(|e| e.to_string())?.file_type().is_symlink() {
            continue;
        }
        let original = PathBuf::from(fs::read_to_string(&manifest)
            .map_err(|e| format!("Cannot read trash manifest: {e}"))?);
        if resolve_project_path(project_root, &original).is_err() { continue; }
        let raw = fs::read(&document).map_err(|e| format!("Cannot read trashed document: {e}"))?;
        let Ok(text) = std::str::from_utf8(&raw) else { continue; };
        let Ok(parsed) = parse_document(&original, text) else { continue; };
        results.push(TrashedDocument {
            entry_id, original_relative_path: original,
            identity: parsed.identity, source_hash: sha256_hex(&raw),
        });
    }
    Ok(())
}

/// Restore only an inspected trash entry to its unoccupied original path.
/// No overwrite, path substitution, or identity reassignment is permitted.
pub fn restore_trashed_document(
    project_root: &Path,
    entry_id: &str,
    expected_source_hash: &str,
) -> Result<CatalystDocument, String> {
    let candidate = list_trashed_documents(project_root)?
        .into_iter().find(|item| item.entry_id == entry_id)
        .ok_or_else(|| "Trash entry is missing or invalid.".to_string())?;
    if candidate.source_hash != expected_source_hash {
        return Err("Trashed document changed since it was listed.".into());
    }
    let destination = resolve_project_path(project_root, &candidate.original_relative_path)?;
    if destination.exists() || fs::symlink_metadata(&destination).is_ok() {
        return Err("Restore destination already exists; no file was overwritten.".into());
    }
    let parent = destination.parent().ok_or("Restore destination has no parent.")?;
    fs::create_dir_all(parent).map_err(|e| format!("Cannot create restore folder: {e}"))?;
    let root = fs::canonicalize(project_root).map_err(|e| e.to_string())?;
    let verified_parent = fs::canonicalize(parent).map_err(|e| e.to_string())?;
    if !verified_parent.starts_with(&root) {
        return Err("Restore destination escapes the project.".into());
    }
    let entry_dir = trash_entry_dir(project_root, entry_id);
    let trash_file = entry_dir.join("document.md");
    let latest = fs::read(&trash_file).map_err(|e| format!("Cannot recheck trash: {e}"))?;
    if sha256_hex(&latest) != expected_source_hash {
        return Err("Trashed document changed during restoration.".into());
    }
    // Create the destination exclusively before copying, to prevent replacement
    // of a document created by another process after the initial existence check.
    use std::io::Write;
    let mut target = fs::OpenOptions::new().write(true).create_new(true).open(&destination)
        .map_err(|e| format!("Restore destination is unavailable (no overwrite): {e}"))?;
    if let Err(error) = target.write_all(&latest).and_then(|_| target.sync_all()) {
        drop(target);
        let _ = fs::remove_file(&destination);
        return Err(format!("Cannot write restored document: {error}"));
    }
    drop(target);
    let restored = read_document(project_root, &candidate.original_relative_path)?;
    if restored.identity != candidate.identity {
        return Err("Restored document identity mismatch; trash was retained.".into());
    }
    // Preserve the trash copy if cleanup fails; it remains available for review.
    fs::remove_dir_all(&entry_dir)
        .map_err(|e| format!("Document restored, but trash cleanup failed: {e}"))?;
    Ok(restored)
}

pub fn save_document(
    project_root: &Path,
    document: &CatalystDocument,
    expected_source_hash: Option<&str>,
) -> Result<SaveResult, String> {
    validate_identity(&document.identity)?;
    let path = resolve_project_path(project_root, &document.relative_path)?;

    if let Some(expected) = expected_source_hash {
        if path.exists() {
            let current = fs::read(&path)
                .map_err(|e| format!("Could not inspect document '{}': {e}", path.display()))?;
            let current_hash = sha256_hex(&current);
            if current_hash != expected {
                return Err(format!(
                    "Document conflict: '{}' changed on disk after it was loaded.",
                    document.relative_path.display()
                ));
            }
        } else if !expected.is_empty() {
            return Err(format!(
                "Document conflict: '{}' was removed after it was loaded.",
                document.relative_path.display()
            ));
        }
    }

    // A document written before the Catalyst rename keeps its legacy identity
    // key so saving never migrates an existing project behind the user's back.
    let identity_key = if path.exists() && file_uses_legacy_identity(&path) {
        LEGACY_IDENTITY_KEY
    } else {
        IDENTITY_KEY
    };
    let serialized = serialize_document_with_key(document, identity_key)?;
    let parent = path.parent().ok_or_else(|| "Document path has no parent.".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("Could not create document directory '{}': {e}", parent.display()))?;

    let temp = parent.join(recovery_file_name(&path, "tmp"));
    // Flush the staged file before touching the authoritative document.
    // A failed write or sync leaves the original file intact.
    let stage_result = (|| -> Result<(), String> {
        use std::io::Write;
        let mut staged_file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| format!("Could not create staged document '{}': {e}", temp.display()))?;
        staged_file
            .write_all(serialized.as_bytes())
            .map_err(|e| format!("Could not write staged document '{}': {e}", temp.display()))?;
        staged_file
            .sync_all()
            .map_err(|e| format!("Could not flush staged document '{}': {e}", temp.display()))
    })();
    if let Err(error) = stage_result {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }

    // Validate the staged bytes before replacing authoritative content.
    let staged = fs::read_to_string(&temp)
        .map_err(|e| format!("Could not verify staged document '{}': {e}", temp.display()))?;
    if let Err(error) = parse_document(&document.relative_path, &staged) {
        let _ = fs::remove_file(&temp);
        return Err(format!("Staged document failed validation: {error}"));
    }

    // Detect an external edit that arrived while we were staging the save.
    // The earlier check alone is insufficient if another editor writes meanwhile.
    if let Some(expected) = expected_source_hash {
        let actual = if path.exists() {
            Some(fs::read(&path).map_err(|e| format!("Could not recheck document before save: {e}"))?)
        } else {
            None
        };
        let conflict = match actual {
            Some(ref bytes) => sha256_hex(bytes) != expected,
            None => !expected.is_empty(),
        };
        if conflict {
            let _ = fs::remove_file(&temp);
            return Err(format!(
                "Document conflict: '{}' changed on disk during save.",
                document.relative_path.display()
            ));
        }
    }

    // On Windows, ReplaceFileW swaps the staged file over the destination in
    // one filesystem operation and can retain a recovery copy of the original.
    // On other platforms, same-directory rename replaces the destination.
    if let Err(error) = replace_staged_document(&temp, &path) {
        // Keep the verified staged bytes on failure for manual recovery.
        return Err(format!(
            "Could not finalize document save: {error}. Staged content retained at '{}'.",
            temp.display()
        ));
    }

    Ok(SaveResult {
        source_hash: sha256_hex(serialized.as_bytes()),
        bytes_written: serialized.len(),
    })
}

// Keep staged files in the same directory to avoid cross-volume replacement.
#[cfg(not(windows))]
fn replace_staged_document(staged: &Path, destination: &Path) -> Result<(), String> {
    fs::rename(staged, destination).map_err(|e| e.to_string())
}

#[cfg(windows)]
fn replace_staged_document(staged: &Path, destination: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn ReplaceFileW(
            replaced_file_name: *const u16,
            replacement_file_name: *const u16,
            backup_file_name: *const u16,
            replace_flags: u32,
            exclude: *const std::ffi::c_void,
            reserved: *const std::ffi::c_void,
        ) -> i32;
    }

    if !destination.exists() {
        return fs::rename(staged, destination).map_err(|e| e.to_string());
    }

    // Win32 ReplaceFileW is not automatically long-path aware even though
    // Rust's std::fs can create deeply nested campaign staging files. In
    // particular, two nested staging folders plus the recovery name can
    // exceed MAX_PATH. canonicalize returns the extended-length \\?\\ form
    // on Windows, so the three API arguments address the same files that
    // std::fs has just verified. Never canonicalize the backup itself: the
    // backup is a new file which does not exist yet.
    let canonical_destination = fs::canonicalize(destination)
        .map_err(|e| format!("Could not resolve destination for Windows replace: {e}"))?;
    let canonical_staged = fs::canonicalize(staged)
        .map_err(|e| format!("Could not resolve staged file for Windows replace: {e}"))?;
    let backup = canonical_destination
        .with_file_name(recovery_file_name(destination, "bak"));
    let wide = |path: &Path| -> Vec<u16> {
        path.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
    };
    let dest_wide = wide(&canonical_destination);
    let staged_wide = wide(&canonical_staged);
    let backup_wide = wide(&backup);
    // SAFETY: Each pointer references a null-terminated UTF-16 path held alive
    // for the duration of this synchronous Windows API call.
    let success = unsafe {
        ReplaceFileW(
            dest_wide.as_ptr(),
            staged_wide.as_ptr(),
            backup_wide.as_ptr(),
            0,
            std::ptr::null(),
            std::ptr::null(),
        )
    };
    if success == 0 {
        return Err(format!(
            "{}; any recovery backup is at '{}'",
            std::io::Error::last_os_error(),
            backup.display()
        ));
    }
    // A failed backup cleanup does not invalidate a successful document save.
    let _ = fs::remove_file(backup);
    Ok(())
}

// Encode the original filename as UTF-8 bytes in hex. Unlike a delimiter-based
// name, this round-trips spaces, punctuation, and Unicode without ambiguity.
// Legacy UUID-only artifacts remain discoverable but have no known destination.
fn recovery_file_name(destination: &Path, extension: &str) -> String {
    let name = destination.file_name().expect("Document has a filename").to_string_lossy();
    let encoded: String = name.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect();
    format!("{TEMP_PREFIX}{encoded}-{}.{}", Uuid::new_v4(), extension)
}

fn decode_recovery_destination(artifact: &Path) -> Option<String> {
    let name = artifact.file_name()?.to_str()?;
    let unprefixed = name.strip_prefix(TEMP_PREFIX).or_else(|| name.strip_prefix(LEGACY_TEMP_PREFIX))?;
    let stem = unprefixed.strip_suffix(".tmp").or_else(|| unprefixed.strip_suffix(".bak"))?;
    // UUIDs contain hyphens: take the fixed 36-character UUID suffix rather
    // than splitting on the last hyphen (which loses most of the UUID).
    let uuid_start = stem.len().checked_sub(36)?;
    let uuid = stem.get(uuid_start..)?;
    Uuid::parse_str(uuid).ok()?;
    let encoded = stem.get(..uuid_start)?.strip_suffix('-')?;
    if encoded.is_empty() || encoded.len() % 2 != 0 {
        return None;
    }
    let bytes: Vec<u8> = (0..encoded.len()).step_by(2)
        .map(|i| u8::from_str_radix(&encoded[i..i + 2], 16).ok())
        .collect::<Option<_>>()?;
    let original = String::from_utf8(bytes).ok()?;
    if original == "." || original == ".." || original.contains('/') || original.contains('\\') {
        return None;
    }
    Some(original)
}

/// Recovery artifacts are *reported*, never automatically applied to manuscript files.
/// A staged file may contain newer edits; a backup may contain the last good copy.
/// The original destination cannot be inferred from legacy random artifact names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryArtifact {
    pub relative_path: PathBuf,
    pub kind: String,
    pub valid_document: bool,
    /// None for legacy random-name artifacts or names that cannot be decoded.
    pub original_relative_path: Option<PathBuf>,
}

/// Scan only the project tree, without following directory symlinks. Nothing is
/// deleted or restored: the author must explicitly choose a recovery action.
pub fn discover_recovery_artifacts(project_root: &Path) -> Result<Vec<RecoveryArtifact>, String> {
    if !project_root.is_absolute() || !project_root.is_dir() {
        return Err("Recovery scan requires an existing absolute project directory.".into());
    }
    let mut pending = vec![project_root.to_path_buf()];
    let mut found = Vec::new();
    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|e| format!("Could not scan recovery directory '{}': {e}", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Could not inspect recovery entry: {e}"))?;
            let file_type = entry.file_type()
                .map_err(|e| format!("Could not inspect recovery entry type: {e}"))?;
            let path = entry.path();
            if file_type.is_dir() {
                pending.push(path);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with(TEMP_PREFIX) && !name.starts_with(LEGACY_TEMP_PREFIX) {
                continue;
            }
            let kind = if name.ends_with(".tmp") {
                "staged"
            } else if name.ends_with(".bak") {
                "backup"
            } else {
                continue;
            };
            let relative_path = path.strip_prefix(project_root)
                .map_err(|e| format!("Could not locate recovery artifact: {e}"))?
                .to_path_buf();
            let valid_document = fs::read_to_string(&path)
                .ok()
                .and_then(|raw| parse_document(&relative_path, &raw).ok())
                .is_some();
            let original_relative_path = decode_recovery_destination(&path)
                .and_then(|file_name| {
                    let destination = path.with_file_name(file_name);
                    destination.strip_prefix(project_root).ok().map(Path::to_path_buf)
                });
            found.push(RecoveryArtifact {
                relative_path,
                kind: kind.into(),
                valid_document,
                original_relative_path,
            });
        }
    }
    found.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(found)
}

/// Create a durable, uniquely named safety copy before a recovery operation.
/// The destination is not modified. Callers must explicitly authorize any
/// subsequent replacement, and must not discard this copy on a failed restore.
pub fn preserve_document_before_recovery(
    project_root: &Path,
    relative_path: &Path,
) -> Result<PathBuf, String> {
    let destination = resolve_project_path(project_root, relative_path)?;
    if !destination.is_file() {
        return Err("Original document is missing; cannot create a safety copy.".into());
    }
    let parent = destination.parent()
        .ok_or_else(|| "Original document has no parent directory.".to_string())?;
    let safety_path = parent.join(recovery_file_name(&destination, "bak"));
    // create_new prevents overwriting another recovery copy. fs::copy creates
    // the destination but does not guarantee that it is durable on power loss.
    let mut source = fs::File::open(&destination)
        .map_err(|e| format!("Could not read original before recovery: {e}"))?;
    let mut safety = fs::OpenOptions::new().write(true).create_new(true)
        .open(&safety_path)
        .map_err(|e| format!("Could not create recovery safety copy: {e}"))?;
    if let Err(error) = std::io::copy(&mut source, &mut safety)
        .and_then(|_| safety.sync_all())
    {
        drop(safety);
        let _ = fs::remove_file(&safety_path);
        return Err(format!("Could not preserve original before recovery: {error}"));
    }
    Ok(safety_path.strip_prefix(project_root)
        .map_err(|e| format!("Could not resolve recovery safety copy: {e}"))?
        .to_path_buf())
}

/// Restore a verified recovery artifact only after the caller explicitly
/// confirms both hashes. Preserve the current document before replacement.
/// The recovery artifact is retained for inspection and retry.
pub fn restore_recovery_artifact(
    project_root: &Path,
    artifact_relative_path: &Path,
    expected_artifact_hash: &str,
    expected_destination_hash: &str,
) -> Result<PathBuf, String> {
    let candidate = discover_recovery_artifacts(project_root)?
        .into_iter()
        .find(|item| item.relative_path == artifact_relative_path)
        .ok_or_else(|| "Recovery candidate no longer exists.".to_string())?;
    if !candidate.valid_document {
        return Err("Cannot restore an invalid recovery document.".into());
    }
    let destination_relative = candidate.original_relative_path
        .ok_or_else(|| "Cannot restore an artifact without a known original document.".to_string())?;
    let destination = resolve_project_path(project_root, &destination_relative)?;
    let artifact = project_root.join(artifact_relative_path);
    // Never follow links, including parent-directory links, out of the project.
    let canonical_root = fs::canonicalize(project_root)
        .map_err(|e| format!("Could not verify project root: {e}"))?;
    let canonical_parent = fs::canonicalize(destination.parent().ok_or("Missing destination parent.")?)
        .map_err(|e| format!("Could not verify destination parent: {e}"))?;
    if !canonical_parent.starts_with(&canonical_root) {
        return Err("Recovery destination escapes the project.".into());
    }
    let canonical_artifact = fs::canonicalize(&artifact)
        .map_err(|e| format!("Could not verify recovery file: {e}"))?;
    if !canonical_artifact.starts_with(&canonical_root) {
        return Err("Recovery artifact escapes the project.".into());
    }
    if !destination.is_file() || fs::symlink_metadata(&destination)
        .map_err(|e| format!("Could not inspect destination: {e}"))?
        .file_type().is_symlink() {
        return Err("Restore requires an existing regular original document.".into());
    }
    let recovered = fs::read(&artifact)
        .map_err(|e| format!("Could not read recovery file: {e}"))?;
    if sha256_hex(&recovered) != expected_artifact_hash {
        return Err("Recovery file changed since preview. Rescan before restoring.".into());
    }
    let recovered_text = std::str::from_utf8(&recovered)
        .map_err(|e| format!("Recovery file is not UTF-8: {e}"))?;
    let parsed = parse_document(&destination_relative, recovered_text)?;
    let current = fs::read(&destination)
        .map_err(|e| format!("Could not inspect current document: {e}"))?;
    if sha256_hex(&current) != expected_destination_hash {
        return Err("Current document changed since preview. Rescan before restoring.".into());
    }
    let current_doc = parse_document(&destination_relative, std::str::from_utf8(&current)
        .map_err(|e| format!("Current document is not UTF-8: {e}"))?)?;
    if parsed.identity != current_doc.identity {
        return Err("Recovery document identity differs from the original.".into());
    }
    // Preserve the current manuscript before any modification.
    let safety_copy = preserve_document_before_recovery(project_root, &destination_relative)?;
    let staged = destination.parent().ok_or("Missing destination parent.")?
        .join(recovery_file_name(&destination, "tmp"));
    let result = (|| -> Result<(), String> {
        use std::io::Write;
        let mut file = fs::OpenOptions::new().write(true).create_new(true)
            .open(&staged).map_err(|e| format!("Could not stage restoration: {e}"))?;
        file.write_all(&recovered)
            .and_then(|_| file.sync_all())
            .map_err(|e| format!("Could not flush restoration: {e}"))?;
        // Windows ReplaceFileW cannot replace a staged file while this handle
        // is still open without delete sharing. Close it before replacement.
        drop(file);
        // Detect concurrent changes after staging, too.
        let latest = fs::read(&destination)
            .map_err(|e| format!("Could not recheck destination: {e}"))?;
        if sha256_hex(&latest) != expected_destination_hash {
            return Err("Current document changed during restoration.".into());
        }
        replace_staged_document(&staged, &destination)
            .map_err(|e| format!("Could not complete restoration: {e}"))
    })();
    if result.is_err() {
        // The original safety copy and the source recovery file remain intact.
        let _ = fs::remove_file(&staged);
    }
    result.map(|_| safety_copy)
}

/// Whether an existing file's frontmatter uses the pre-rename identity key.
fn file_uses_legacy_identity(path: &Path) -> bool {
    let Ok(raw) = fs::read_to_string(path) else { return false };
    let normalized = raw.replace("\r\n", "\n");
    let Ok((yaml, _)) = split_frontmatter(&normalized) else { return false };
    serde_yaml::from_str::<BTreeMap<String, Value>>(yaml)
        .map(|root| root.contains_key(LEGACY_IDENTITY_KEY) && !root.contains_key(IDENTITY_KEY))
        .unwrap_or(false)
}

pub fn parse_document(relative_path: &Path, raw: &str) -> Result<CatalystDocument, String> {
    let normalized = raw.replace("\r\n", "\n").replace('\r', "\n");
    let (yaml, body) = split_frontmatter(&normalized)?;
    let mut root: BTreeMap<String, Value> =
        serde_yaml::from_str(yaml).map_err(|e| format!("Invalid document YAML: {e}"))?;

    let identity_value = match (root.remove(IDENTITY_KEY), root.remove(LEGACY_IDENTITY_KEY)) {
        (Some(_), Some(_)) => {
            return Err("Document frontmatter has both 'catalyst' and legacy 'lithomark' identities.".into())
        }
        (Some(value), None) | (None, Some(value)) => value,
        (None, None) => return Err("Document frontmatter is missing 'catalyst' identity.".into()),
    };
    let identity: DocumentIdentity = serde_yaml::from_value(identity_value)
        .map_err(|e| format!("Invalid Catalyst document identity: {e}"))?;
    validate_identity(&identity)?;

    let title = match root.remove("title") {
        Some(Value::String(value)) => Some(value),
        Some(_) => return Err("Document title must be a string.".into()),
        None => None,
    };

    Ok(CatalystDocument {
        identity,
        title,
        metadata: root,
        body: normalize_body(body),
        relative_path: relative_path.to_path_buf(),
        source_hash: sha256_hex(normalized.as_bytes()),
    })
}

pub fn serialize_document(document: &CatalystDocument) -> Result<String, String> {
    serialize_document_with_key(document, IDENTITY_KEY)
}

fn serialize_document_with_key(document: &CatalystDocument, identity_key: &str) -> Result<String, String> {
    validate_identity(&document.identity)?;

    let mut root = document.metadata.clone();
    if root.contains_key(IDENTITY_KEY) || root.contains_key(LEGACY_IDENTITY_KEY) {
        return Err("Document metadata must not override the reserved 'catalyst' identity.".into());
    }
    if root.contains_key("title") {
        return Err("Document metadata must not override reserved 'title' field.".into());
    }

    root.insert(
        identity_key.into(),
        serde_yaml::to_value(&document.identity)
            .map_err(|e| format!("Could not serialize document identity: {e}"))?,
    );
    if let Some(title) = document.title.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        root.insert("title".into(), Value::String(title.to_string()));
    }

    let mut yaml = serde_yaml::to_string(&root)
        .map_err(|e| format!("Could not serialize document metadata: {e}"))?;
    yaml = yaml.replace("\r\n", "\n").trim().to_string();

    let body = normalize_body(&document.body);
    if body.is_empty() {
        Ok(format!("{FRONTMATTER_DELIMITER}\n{yaml}\n{FRONTMATTER_DELIMITER}\n"))
    } else {
        Ok(format!(
            "{FRONTMATTER_DELIMITER}\n{yaml}\n{FRONTMATTER_DELIMITER}\n\n{body}"
        ))
    }
}

fn split_frontmatter(raw: &str) -> Result<(&str, &str), String> {
    let mut lines = raw.split_inclusive('\n');
    let first = lines.next().unwrap_or_default();
    if first.trim_end_matches('\n').trim() != FRONTMATTER_DELIMITER {
        return Err("Document must begin with YAML frontmatter ('---').".into());
    }

    let after_first = first.len();
    let remainder = &raw[after_first..];
    let mut offset = 0usize;
    for line in remainder.split_inclusive('\n') {
        let line_without_newline = line.trim_end_matches('\n');
        if line_without_newline.trim() == FRONTMATTER_DELIMITER {
            let yaml = &remainder[..offset];
            let body_start = after_first + offset + line.len();
            let body = raw[body_start..].strip_prefix('\n').unwrap_or(&raw[body_start..]);
            return Ok((yaml, body));
        }
        offset += line.len();
    }
    Err("Document YAML frontmatter is not closed with '---'.".into())
}

fn validate_identity(identity: &DocumentIdentity) -> Result<(), String> {
    validate_document_type(&identity.document_type)?;
    let prefix = stable_id_prefix(&identity.document_type)?;
    let uuid = identity.id.strip_prefix(prefix).ok_or_else(|| {
        format!(
            "Catalyst document id '{}' must use prefix '{}' for type '{}'.",
            identity.id, prefix, identity.document_type
        )
    })?;
    let parsed = Uuid::parse_str(uuid)
        .map_err(|_| "Catalyst document id must contain a valid UUID.".to_string())?;
    let canonical = format!("{prefix}{}", parsed.hyphenated());
    if identity.id != canonical {
        return Err("Catalyst document id must use canonical lowercase prefix-UUID form.".into());
    }
    Ok(())
}

pub(crate) fn generate_stable_id(document_type: &str) -> Result<String, String> {
    let prefix = stable_id_prefix(document_type)?;
    Ok(format!("{prefix}{}", Uuid::new_v4().hyphenated()))
}

fn stable_id_prefix(document_type: &str) -> Result<&'static str, String> {
    match document_type.trim() {
        "part" => Ok("prt-"),
        "chapter" => Ok("ch-"),
        "scene" => Ok("scn-"),
        "beat" => Ok("beat-"),
        "outline" => Ok("out-"),
        "character" => Ok("chr-"),
        "location" => Ok("loc-"),
        "organization" => Ok("org-"),
        "object" | "item" => Ok("obj-"),
        "event" | "timeline-event" => Ok("evt-"),
        "relationship" => Ok("rel-"),
        "plot-thread" => Ok("plt-"),
        "character-arc" => Ok("arc-"),
        "reveal" => Ok("rev-"),
        "clue" => Ok("clu-"),
        "foreshadowing" => Ok("foresh-"),
        "payoff" => Ok("pay-"),
        "research" | "research-item" => Ok("res-"),
        "note" => Ok("note-"),
        "template" => Ok("tpl-"),
        "canon-fact" => Ok("fact-"),
        "annotation" => Ok("ann-"),
        "tracked-change" => Ok("chg-"),
        "snapshot" => Ok("snap-"),
        other if other.contains('.') => Err(format!(
            "Plugin-defined document type '{other}' must obtain identity through the Stable ID service; plugin IDs are not assigned by the core Document Service yet."
        )),
        other => Err(format!("Unsupported core Catalyst document type '{other}'.")),
    }
}

fn validate_document_type(document_type: &str) -> Result<(), String> {
    if document_type.trim().is_empty() {
        return Err("Catalyst document type must not be empty.".into());
    }
    Ok(())
}

fn normalize_body(body: &str) -> String {
    let normalized = body.replace("\r\n", "\n").replace('\r', "\n");
    if normalized.is_empty() {
        return String::new();
    }

    // Preserve Markdown-significant spaces and tabs at line endings. Only
    // canonicalize the number of terminal newline characters.
    let without_terminal_newlines = normalized.trim_end_matches('\n');
    if without_terminal_newlines.is_empty() {
        String::new()
    } else {
        format!("{without_terminal_newlines}\n")
    }
}

fn resolve_project_path(project_root: &Path, relative_path: &Path) -> Result<PathBuf, String> {
    if !project_root.is_absolute() {
        return Err("Project root must be absolute.".into());
    }
    if relative_path.as_os_str().is_empty() || relative_path.is_absolute() {
        return Err("Document path must be a non-empty project-relative path.".into());
    }
    if relative_path.extension().and_then(|v| v.to_str()) != Some("md") {
        return Err("Authoritative textual documents must use the .md extension.".into());
    }
    for component in relative_path.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err("Document path must not escape the project root.".into());
        }
    }
    Ok(project_root.join(relative_path))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn creates_reads_and_round_trips_file_authoritative_document() {
        let temp = root();
        let path = Path::new("Manuscript/Chapter 01/Scene 01.md");
        let created = create_document(
            temp.path(),
            path,
            "scene",
            Some("Opening"),
            "Synthetic scene body.\n",
        )
        .unwrap();

        assert_eq!(created.identity.document_type, "scene");
        assert_eq!(created.title.as_deref(), Some("Opening"));
        assert_eq!(created.body, "Synthetic scene body.\n");
        assert!(temp.path().join(path).is_file());

        let raw = fs::read_to_string(temp.path().join(path)).unwrap();
        assert!(raw.contains("type: scene"));
        assert!(raw.contains("title: Opening"));
    }

    #[cfg(windows)]
    #[test]
    fn long_demo_staging_paths_can_update_documents_with_recovery_backup() {
        // Reproduce the two nested staging folders used by the demo:
        // a long parent plus an auto-generated .catalyst-save-...bak name.
        let temp = root();
        let deep = temp.path()
            .join("Demo Campaigns")
            .join(format!(".hollow-crown-demo-{}", "a".repeat(36)))
            .join(format!(".catalyst-staging-{}", "b".repeat(36)))
            .join("Adventures/Main Campaign");
        fs::create_dir_all(&deep).unwrap();
        let project = deep.parent().unwrap().parent().unwrap();
        let relative = Path::new("Adventures/Main Campaign/Campaign Overview.md");
        let mut doc = create_document(
            project, relative, "note", Some("Campaign Overview"), "First draft."
        ).unwrap();
        doc.body = "Edited campaign overview.\n".into();
        let result = save_document(project, &doc, Some(&doc.source_hash));
        assert!(result.is_ok(), "long-path Windows replace failed: {result:?}");
        let reopened = read_document(project, relative).unwrap();
        assert!(reopened.body.contains("Edited campaign overview."));
    }

    #[test]
    fn preserves_unknown_metadata_on_save() {
        let temp = root();
        let path = Path::new("Characters/Test Character.md");
        let mut doc = create_document(temp.path(), path, "character", Some("Test Character"), "Notes.").unwrap();
        doc.metadata.insert(
            "future_feature".into(),
            serde_yaml::from_str("nested:\n  enabled: true\n").unwrap(),
        );
        save_document(temp.path(), &doc, Some(&doc.source_hash)).unwrap();

        let loaded = read_document(temp.path(), path).unwrap();
        assert!(loaded.metadata.contains_key("future_feature"));
    }

    #[test]
    fn rejects_stale_save_after_external_change() {
        let temp = root();
        let path = Path::new("Manuscript/Scene.md");
        let mut doc = create_document(temp.path(), path, "scene", Some("Scene"), "Original.").unwrap();

        fs::write(
            temp.path().join(path),
            fs::read_to_string(temp.path().join(path)).unwrap().replace("Original.", "External."),
        )
        .unwrap();

        doc.body = "Catalyst edit.\n".into();
        let error = save_document(temp.path(), &doc, Some(&doc.source_hash)).unwrap_err();
        assert!(error.contains("conflict"));
        assert!(fs::read_to_string(temp.path().join(path)).unwrap().contains("External."));
    }

    #[test]
    fn recovery_scan_reports_valid_and_invalid_artifacts_without_modifying_them() {
        let temp = root();
        let path = Path::new("Characters/Example.md");
        create_document(temp.path(), path, "character", Some("Example"), "Safe.").unwrap();
        let original = fs::read(temp.path().join(path)).unwrap();
        let directory = temp.path().join("Characters");
        let staged = directory.join(".lithomark-save-interrupted.tmp");
        let backup = directory.join(".lithomark-save-interrupted.bak");
        fs::write(&staged, &original).unwrap();
        fs::write(&backup, "incomplete").unwrap();

        let artifacts = discover_recovery_artifacts(temp.path()).unwrap();
        assert_eq!(artifacts.len(), 2);
        assert_eq!(artifacts[0].kind, "backup");
        assert!(!artifacts[0].valid_document);
        assert_eq!(artifacts[1].kind, "staged");
        assert!(artifacts[1].valid_document);
        assert_eq!(fs::read(temp.path().join(path)).unwrap(), original);
        assert!(staged.exists() && backup.exists());
    }

    #[test]
    fn recovery_filename_round_trips_unicode_and_rejects_legacy_uuid_names() {
        let name = Path::new("Characters/Éloïse — 海.md");
        let staged = recovery_file_name(name, "tmp");
        assert_eq!(
            decode_recovery_destination(Path::new(&staged)).as_deref(),
            Some("Éloïse — 海.md")
        );
        assert_eq!(
            decode_recovery_destination(Path::new(".lithomark-save-12345678-1234-1234-1234-123456789abc.tmp")),
            None
        );
    }

    #[test]
    fn recovery_artifacts_identify_original_document_and_keep_legacy_unmatched() {
        let temp = root();
        let path = Path::new("Characters/Mara Voss.md");
        create_document(temp.path(), path, "character", Some("Mara"), "Body.").unwrap();
        let original = fs::read(temp.path().join(path)).unwrap();
        let named = temp.path().join("Characters").join(recovery_file_name(path, "tmp"));
        let legacy = temp.path().join("Characters/.lithomark-save-interrupted.bak");
        fs::write(&named, &original).unwrap();
        fs::write(&legacy, &original).unwrap();

        let artifacts = discover_recovery_artifacts(temp.path()).unwrap();
        assert_eq!(artifacts.len(), 2);
        assert_eq!(
            artifacts.iter().find(|item| item.kind == "staged").unwrap().original_relative_path,
            Some(path.to_path_buf())
        );
        assert_eq!(
            artifacts.iter().find(|item| item.kind == "backup").unwrap().original_relative_path,
            None
        );
        assert_eq!(fs::read(temp.path().join(path)).unwrap(), original);
    }

    #[test]
    fn guarded_restore_preserves_current_and_rejects_stale_hashes() {
        let temp = root();
        let path = Path::new("Characters/Mara.md");
        let original = create_document(temp.path(), path, "character", Some("Mara"), "First.").unwrap();
        let artifact = Path::new("Characters").join(recovery_file_name(path, "tmp"));
        let recovery_bytes = fs::read(temp.path().join(path)).unwrap();
        fs::write(temp.path().join(&artifact), &recovery_bytes).unwrap();
        let mut edited = original.clone();
        edited.body = "New edits.".into();
        save_document(temp.path(), &edited, Some(&original.source_hash)).unwrap();
        let current_bytes = fs::read(temp.path().join(path)).unwrap();
        let stale = restore_recovery_artifact(
            temp.path(), &artifact, &sha256_hex(&recovery_bytes), &original.source_hash
        ).unwrap_err();
        assert!(stale.contains("changed"));
        assert_eq!(fs::read(temp.path().join(path)).unwrap(), current_bytes);
        let backup = restore_recovery_artifact(
            temp.path(), &artifact, &sha256_hex(&recovery_bytes), &sha256_hex(&current_bytes)
        ).unwrap();
        assert_eq!(fs::read(temp.path().join(path)).unwrap(), recovery_bytes);
        assert_eq!(fs::read(temp.path().join(backup)).unwrap(), current_bytes);
        assert!(temp.path().join(artifact).exists());
    }

    #[test]
    fn guarded_restore_rejects_artifact_modified_after_preview() {
        let temp = root();
        let path = Path::new("Characters/Mara.md");
        create_document(temp.path(), path, "character", Some("Mara"), "Original.").unwrap();
        let current = fs::read(temp.path().join(path)).unwrap();
        let artifact = Path::new("Characters").join(recovery_file_name(path, "tmp"));
        fs::write(temp.path().join(&artifact), &current).unwrap();
        let preview_hash = sha256_hex(&current);

        let changed = String::from_utf8(current.clone()).unwrap().replace("Original.", "Changed artifact.");
        fs::write(temp.path().join(&artifact), changed.as_bytes()).unwrap();
        let error = restore_recovery_artifact(
            temp.path(), &artifact, &preview_hash, &sha256_hex(&current)
        ).unwrap_err();

        assert!(error.contains("Recovery file changed"));
        assert_eq!(fs::read(temp.path().join(path)).unwrap(), current);
        assert_eq!(fs::read(temp.path().join(&artifact)).unwrap(), changed.as_bytes());
    }

    #[test]
    fn guarded_restore_refuses_unknown_destination_and_wrong_identity() {
        let temp = root();
        let path = Path::new("Characters/Mara.md");
        let original = create_document(temp.path(), path, "character", Some("Mara"), "First.").unwrap();
        let other = create_document(temp.path(), Path::new("Characters/Other.md"), "character", Some("Other"), "Other.").unwrap();
        let bytes = fs::read(temp.path().join("Characters/Other.md")).unwrap();
        let artifact = Path::new("Characters").join(recovery_file_name(path, "tmp"));
        fs::write(temp.path().join(&artifact), &bytes).unwrap();
        let error = restore_recovery_artifact(temp.path(), &artifact, &sha256_hex(&bytes), &original.source_hash).unwrap_err();
        assert!(error.contains("identity"));
        assert_eq!(read_document(temp.path(), path).unwrap().identity, original.identity);
        assert_ne!(other.identity, original.identity);
        let legacy = Path::new("Characters/.lithomark-save-legacy.tmp");
        fs::write(temp.path().join(legacy), fs::read(temp.path().join(path)).unwrap()).unwrap();
        let error = restore_recovery_artifact(temp.path(), legacy, &original.source_hash, &original.source_hash).unwrap_err();
        assert!(error.contains("known original"));
    }

    #[test]
    fn successful_save_leaves_no_staging_files() {
        let temp = root();
        let path = Path::new("Characters/Example.md");
        let mut doc = create_document(temp.path(), path, "character", Some("Example"), "First.").unwrap();
        doc.body = "Second.".into();
        save_document(temp.path(), &doc, Some(&doc.source_hash)).unwrap();
        let parent = temp.path().join("Characters");
        assert!(fs::read_to_string(parent.join("Example.md")).unwrap().contains("Second."));
        assert!(fs::read_dir(parent).unwrap().all(|entry| {
            !entry.unwrap().file_name().to_string_lossy().starts_with(TEMP_PREFIX)
        }));
    }

    #[test]
    fn refuses_path_traversal() {
        let temp = root();
        let error = create_document(
            temp.path(),
            Path::new("../outside.md"),
            "note",
            None,
            "Nope",
        )
        .unwrap_err();
        assert!(error.contains("escape"));
        assert!(!temp.path().parent().unwrap().join("outside.md").exists());
    }

    #[test]
    fn legacy_identity_key_is_read_and_kept_on_save() {
        let temp = root();
        let path = Path::new("Notes/old.md");
        fs::create_dir_all(temp.path().join("Notes")).unwrap();
        fs::write(
            temp.path().join(path),
            "---\nlithomark:\n  id: note-11111111-1111-4111-8111-111111111111\n  type: note\n  schema_version: 1\ntitle: Old\n---\nBody\n",
        ).unwrap();
        let mut document = read_document(temp.path(), path).unwrap();
        assert_eq!(document.identity.id, "note-11111111-1111-4111-8111-111111111111");
        document.body = "Edited".into();
        save_document(temp.path(), &document, Some(&document.source_hash)).unwrap();
        let raw = fs::read_to_string(temp.path().join(path)).unwrap();
        assert!(raw.contains("lithomark:") && !raw.contains("catalyst:"));
        assert_eq!(read_document(temp.path(), path).unwrap().body.trim(), "Edited");
    }

    #[test]
    fn new_documents_use_catalyst_identity_key() {
        let temp = root();
        create_document(temp.path(), Path::new("Notes/new.md"), "note", Some("New"), "Body").unwrap();
        let raw = fs::read_to_string(temp.path().join("Notes/new.md")).unwrap();
        assert!(raw.contains("catalyst:") && !raw.contains("lithomark"));
    }

    #[test]
    fn invalid_frontmatter_is_not_rewritten() {
        let temp = root();
        let path = Path::new("Notes/broken.md");
        fs::create_dir_all(temp.path().join("Notes")).unwrap();
        fs::write(temp.path().join(path), "---\nlithomark: [broken\n---\nBody\n").unwrap();

        assert!(read_document(temp.path(), path).is_err());
        assert!(fs::read_to_string(temp.path().join(path)).unwrap().contains("[broken"));
    }

    #[test]
    fn move_preserves_identity_and_rejects_overwrite() {
        let temp = tempfile::tempdir().unwrap();
        let original = Path::new("Story Development/Characters/Test Character.md");
        let destination = Path::new("Story Development/Characters/Renamed Character.md");
        let existing = Path::new("Story Development/Characters/Existing.md");

        let created = create_document(temp.path(), original, "character", Some("Test Character"), "Body.").unwrap();
        create_document(temp.path(), existing, "character", Some("Existing"), "Other.").unwrap();

        let moved = move_document(temp.path(), original, destination, &created.source_hash).unwrap();
        assert_eq!(moved.identity, created.identity);
        assert_eq!(moved.relative_path, destination);
        assert!(!temp.path().join(original).exists());
        assert!(temp.path().join(destination).is_file());

        let error = move_document(temp.path(), destination, existing, &moved.source_hash).unwrap_err();
        assert!(error.contains("already exists"));
        assert!(temp.path().join(destination).is_file());
        assert!(temp.path().join(existing).is_file());
    }

    #[test]
    fn deletion_retains_document_and_original_path_in_project_trash() {
        let temp = tempfile::tempdir().unwrap();
        let path = Path::new("Story Development/Characters/Recoverable.md");
        let created = create_document(temp.path(), path, "character", Some("Recoverable"), "Keep this.").unwrap();
        let deleted = delete_document(temp.path(), path, &created.source_hash).unwrap();
        assert_eq!(deleted, created.identity);
        assert!(!temp.path().join(path).exists());

        let trash = temp.path().join(".catalyst/trash");
        let entries: Vec<_> = fs::read_dir(trash).unwrap().map(Result::unwrap).collect();
        assert_eq!(entries.len(), 1);
        let entry = entries[0].path();
        assert_eq!(fs::read_to_string(entry.join("original-path.txt")).unwrap(), path.to_string_lossy());
        let saved = fs::read_to_string(entry.join("document.md")).unwrap();
        assert!(saved.contains(&created.identity.id));
        assert!(saved.contains("Keep this."));
    }

    #[test]
    fn trashed_document_restores_with_original_identity_and_bytes() {
        let temp = tempfile::tempdir().unwrap();
        let path = Path::new("Story Development/Characters/Restorable.md");
        let original = create_document(temp.path(), path, "character", Some("Restorable"), "Original words.").unwrap();
        let bytes = fs::read(temp.path().join(path)).unwrap();
        delete_document(temp.path(), path, &original.source_hash).unwrap();
        let items = list_trashed_documents(temp.path()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].identity, original.identity);
        let restored = restore_trashed_document(temp.path(), &items[0].entry_id, &items[0].source_hash).unwrap();
        assert_eq!(restored.identity, original.identity);
        assert_eq!(fs::read(temp.path().join(path)).unwrap(), bytes);
        assert!(list_trashed_documents(temp.path()).unwrap().is_empty());
    }

    #[test]
    fn trash_restore_rejects_occupied_path_and_changed_trash() {
        let temp = tempfile::tempdir().unwrap();
        let path = Path::new("Story Development/Locations/Blocked.md");
        let original = create_document(temp.path(), path, "location", Some("Blocked"), "First.").unwrap();
        delete_document(temp.path(), path, &original.source_hash).unwrap();
        let entry = list_trashed_documents(temp.path()).unwrap().remove(0);
        fs::write(temp.path().join(path), "Do not overwrite.").unwrap();
        assert!(restore_trashed_document(temp.path(), &entry.entry_id, &entry.source_hash).unwrap_err().contains("already exists"));
        assert_eq!(fs::read_to_string(temp.path().join(path)).unwrap(), "Do not overwrite.");
        fs::remove_file(temp.path().join(path)).unwrap();
        let trash_file = temp.path().join(".catalyst/trash").join(&entry.entry_id).join("document.md");
        fs::write(&trash_file, fs::read_to_string(&trash_file).unwrap() + "\nEdited externally.").unwrap();
        assert!(restore_trashed_document(temp.path(), &entry.entry_id, &entry.source_hash).unwrap_err().contains("changed"));
        assert!(!temp.path().join(path).exists());
    }

    #[test]
    fn same_path_move_requires_fresh_hash() {
        let temp = tempfile::tempdir().unwrap();
        let path = Path::new("Story Development/Characters/Example.md");
        let created = create_document(temp.path(), path, "character", Some("Example"), "Original.").unwrap();
        let unchanged = move_document(temp.path(), path, path, &created.source_hash).unwrap();
        assert_eq!(unchanged.identity, created.identity);

        fs::write(
            temp.path().join(path),
            fs::read_to_string(temp.path().join(path)).unwrap() + "\nExternal change.\n",
        ).unwrap();
        let error = move_document(temp.path(), path, path, &created.source_hash).unwrap_err();
        assert!(error.contains("conflict"));
        assert!(temp.path().join(path).is_file());
    }

    #[test]
    fn move_and_delete_reject_stale_hashes() {
        let temp = tempfile::tempdir().unwrap();
        let original = Path::new("Story Development/Locations/Test Location.md");
        let destination = Path::new("Story Development/Locations/Moved Location.md");
        let created = create_document(temp.path(), original, "location", Some("Test Location"), "Body.").unwrap();

        fs::write(
            temp.path().join(original),
            fs::read_to_string(temp.path().join(original)).unwrap() + "\nExternal edit.\n",
        ).unwrap();

        let move_error = move_document(temp.path(), original, destination, &created.source_hash).unwrap_err();
        assert!(move_error.contains("conflict"));
        assert!(temp.path().join(original).is_file());

        let delete_error = delete_document(temp.path(), original, &created.source_hash).unwrap_err();
        assert!(delete_error.contains("conflict"));
        assert!(temp.path().join(original).is_file());

        let reloaded = read_document(temp.path(), original).unwrap();
        let deleted = delete_document(temp.path(), original, &reloaded.source_hash).unwrap();
        assert_eq!(deleted, reloaded.identity);
        assert!(!temp.path().join(original).exists());
    }

    #[test]
    fn stable_id_survives_body_and_title_changes() {
        let temp = root();
        let path = Path::new("Manuscript/Scene.md");
        let mut doc = create_document(temp.path(), path, "scene", Some("Old"), "One.").unwrap();
        let id = doc.identity.id.clone();
        doc.title = Some("New".into());
        doc.body = "Two.\n".into();
        save_document(temp.path(), &doc, Some(&doc.source_hash)).unwrap();

        let loaded = read_document(temp.path(), path).unwrap();
        assert_eq!(loaded.identity.id, id);
        assert_eq!(loaded.title.as_deref(), Some("New"));
        assert_eq!(loaded.body, "Two.\n");
    }

    #[test]
    fn crlf_document_can_be_saved_without_false_conflict() {
        let temp = root();
        let path = Path::new("Notes/Windows.md");
        let created = create_document(temp.path(), path, "note", Some("Windows"), "Body").unwrap();
        let raw = fs::read_to_string(temp.path().join(path)).unwrap();
        fs::write(temp.path().join(path), raw.replace("\n", "\r\n")).unwrap();

        let mut loaded = read_document(temp.path(), path).unwrap();
        loaded.body = "Edited\n".into();
        save_document(temp.path(), &loaded, Some(&loaded.source_hash)).unwrap();

        assert!(fs::read_to_string(temp.path().join(path)).unwrap().contains("Edited"));
        assert_ne!(created.source_hash, loaded.source_hash);
    }

    #[test]
    fn preserves_markdown_trailing_spaces() {
        let temp = root();
        let path = Path::new("Notes/Markdown.md");
        let created = create_document(
            temp.path(),
            path,
            "note",
            Some("Markdown"),
            "Hard break here.  \nNext line.\n",
        )
        .unwrap();
        assert_eq!(created.body, "Hard break here.  \nNext line.\n");

        let loaded = read_document(temp.path(), path).unwrap();
        assert_eq!(loaded.body, "Hard break here.  \nNext line.\n");
    }

    #[test]
    fn generated_ids_use_type_prefix_and_uuid() {
        let temp = root();
        let scene = create_document(
            temp.path(),
            Path::new("Manuscript/Scene.md"),
            "scene",
            None,
            "Body",
        )
        .unwrap();
        assert!(scene.identity.id.starts_with("scn-"));
        assert!(Uuid::parse_str(scene.identity.id.strip_prefix("scn-").unwrap()).is_ok());

        let character = create_document(
            temp.path(),
            Path::new("Characters/Test Character.md"),
            "character",
            Some("Test Character"),
            "Notes",
        )
        .unwrap();
        assert!(character.identity.id.starts_with("chr-"));
    }

    #[test]
    fn rejects_id_prefix_that_does_not_match_document_type() {
        let identity = DocumentIdentity {
            document_type: "character".into(),
            id: format!("loc-{}", Uuid::new_v4()),
        };
        let error = validate_identity(&identity).unwrap_err();
        assert!(error.contains("prefix"));
    }

}
