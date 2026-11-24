//! Offline, file-first tabletop content importer.
//! Original sources are preserved under Assets/Imports, and extracted notes
//! use CatalystDocument with stable IDs. No silent manuscript conversion.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use walkdir::WalkDir;

use crate::document::service::{create_document, save_document};
use crate::project_root::read_project_manifest;

const MAX_FILES: usize = 2000;
const MAX_TOTAL_BYTES: u64 = 512 * 1024 * 1024;
const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PREVIEW: usize = 120;
const MAX_TEXT: usize = 512 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RpgFormat {
    Markdown, Obsidian, Assets, Foundry, Roll20, FantasyGrounds, Pdf,
}

#[derive(Debug, Clone, Serialize)]
pub struct RpgImportPreview {
    pub format: RpgFormat,
    pub source: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub sample_files: Vec<String>,
    pub warnings: Vec<String>,
    pub behavior: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RpgImportResult {
    pub imported: usize,
    /// Files that could not be imported, with the reason. Other files continue.
    pub skipped: Vec<String>,
    pub documents_created: usize,
    pub copied_assets: usize,
    pub destination: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
struct Candidate { path: PathBuf, relative: PathBuf, size: u64 }

fn acceptable_extension(format: &RpgFormat, extension: &str) -> bool {
    match format {
        RpgFormat::Markdown => matches!(extension, "md" | "markdown"),
        // Vault attachments (images, PDFs) are copied alongside the notes.
        RpgFormat::Obsidian => matches!(extension, "md" | "markdown" | "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg" | "pdf"),
        RpgFormat::Assets => matches!(extension, "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg"),
        RpgFormat::Foundry | RpgFormat::Roll20 => matches!(extension, "json" | "zip"),
        RpgFormat::FantasyGrounds => matches!(extension, "mod" | "xml"),
        RpgFormat::Pdf => extension == "pdf",
    }
}

fn extension(path: &Path) -> String {
    path.extension().and_then(|x| x.to_str()).unwrap_or("").to_ascii_lowercase()
}

fn safe_name(segment: &str) -> String {
    let clean: String = segment.chars().map(|c| {
        if c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.') { c }
        else { '_' }
    }).collect();
    let clean = clean.trim().trim_matches('.').trim();
    let clean = if clean.is_empty() { "Imported item" } else { clean };
    clean.chars().take(96).collect()
}

fn candidates(source: &Path, format: &RpgFormat) -> Result<Vec<Candidate>, String> {
    let metadata = fs::symlink_metadata(source).map_err(|e| format!("Cannot inspect import source: {e}"))?;
    if metadata.file_type().is_symlink() { return Err("Linked source paths are not accepted.".into()); }
    if metadata.is_dir() && !matches!(format, RpgFormat::Markdown | RpgFormat::Obsidian | RpgFormat::Assets | RpgFormat::Foundry | RpgFormat::Roll20) {
        return Err("This format requires a single export file.".into());
    }
    if !metadata.is_dir() && !metadata.is_file() { return Err("Select a regular file or directory.".into()); }
    let mut result = Vec::new();
    let mut total = 0_u64;
    let mut inspect = |path: PathBuf, relative: PathBuf| -> Result<(), String> {
        let m = fs::symlink_metadata(&path).map_err(|e| format!("Cannot inspect source file: {e}"))?;
        if m.file_type().is_symlink() || !m.is_file() { return Ok(()); }
        if !acceptable_extension(format, &extension(&path)) { return Ok(()); }
        if m.len() > MAX_FILE_BYTES { return Err(format!("File '{}' exceeds 64 MiB.", path.display())); }
        total = total.checked_add(m.len()).ok_or("Source size overflow")?;
        if total > MAX_TOTAL_BYTES { return Err("Import exceeds 512 MiB; split into smaller batches.".into()); }
        if result.len() >= MAX_FILES { return Err("More than 2,000 files; split this import.".into()); }
        result.push(Candidate { path, relative, size: m.len() });
        Ok(())
    };
    if metadata.is_file() {
        inspect(source.to_path_buf(), PathBuf::from(source.file_name().ok_or("Invalid file name")?))?;
    } else {
        for entry in WalkDir::new(source).follow_links(false).max_depth(30) {
            let entry = entry.map_err(|e| format!("Cannot scan source folder: {e}"))?;
            if entry.file_type().is_symlink() { continue; }
            if entry.file_type().is_file() {
                let relative = entry.path().strip_prefix(source).map_err(|e| e.to_string())?.to_path_buf();
                if relative.components().any(|c| c.as_os_str().to_string_lossy().starts_with('.')) { continue; }
                inspect(entry.path().to_path_buf(), relative)?;
            }
        }
    }
    if result.is_empty() { return Err("No supported files found in the selected source.".into()); }
    result.sort_by(|a,b| a.relative.cmp(&b.relative));
    Ok(result)
}

fn inspect_source(source_path: &str, format: &RpgFormat) -> Result<(PathBuf, Vec<Candidate>), String> {
    let source = PathBuf::from(source_path);
    if !source.is_absolute() { return Err("Select an absolute local import path.".into()); }
    let list = candidates(&source, format)?;
    Ok((source, list))
}

fn behavior(format: &RpgFormat) -> &'static str {
    match format {
        RpgFormat::Markdown => "Create editable records in Worldbuilding/Imported, keeping the source folder structure. Source frontmatter is kept as metadata, and an untouched copy of each original file is saved in Assets/Imports/Markdown Sources.",
        RpgFormat::Obsidian => "Create editable records in Worldbuilding/Imported, keeping the vault's folders. [[Wikilinks]] work when a note with that title exists. Images and PDFs are copied to Assets/Imports/Obsidian Attachments; ![[embeds]] stay as text. Originals are kept in Assets/Imports/Markdown Sources.",
        RpgFormat::Assets => "Copy images, maps and tokens to Assets/Imports. Files are not converted to VTT scenes.",
        RpgFormat::Foundry => "Preserve Foundry JSON or ZIP exports in Assets/Imports, with a readable source index. Scene walls, actor mechanics and compendium relationships are not converted.",
        RpgFormat::Roll20 => "Preserve available Roll20 JSON or ZIP exports and create a source index. There is no universal Roll20 campaign export schema.",
        RpgFormat::FantasyGrounds => "Keep .mod or XML intact as reference material. Only a module metadata summary is created; no rule record conversion.",
        RpgFormat::Pdf => "Copy the original PDF as an offline reference asset and create an index note. PDF text extraction/OCR is not part of this importer.",
    }
}

#[tauri::command]
pub async fn preview_rpg_content_import(source_path: String, format: RpgFormat) -> Result<RpgImportPreview, String> {
    let (_source, list) = inspect_source(&source_path, &format)?;
    let warnings = match format {
        RpgFormat::Foundry => vec!["Foundry versions and game systems have different schemas. Data is preserved, not converted into executable mechanics.".into()],
        RpgFormat::Roll20 => vec!["Roll20 does not provide one universal campaign JSON export; third-party exports may be unsupported.".into()],
        RpgFormat::FantasyGrounds => vec!["Fantasy Grounds .mod is a ZIP/XML module. Proprietary module contents are kept intact; Catalyst does not bypass DRM.".into()],
        RpgFormat::Pdf => vec!["PDFs are copied as-is. Scanned pages and embedded text are not indexed yet.".into()],
        RpgFormat::Obsidian => vec!["Embeds, plugin data and aliases are not converted. Wikilinks resolve by note title.".into()],
        _ => vec![],
    };
    Ok(RpgImportPreview {
        format: format.clone(), source: source_path, file_count: list.len(),
        total_bytes: list.iter().map(|f| f.size).sum(),
        sample_files: list.iter().take(MAX_PREVIEW).map(|f| f.relative.to_string_lossy().into_owned()).collect(),
        warnings, behavior: behavior(&format).into(),
    })
}

fn project_root(root_path: &str) -> Result<PathBuf, String> {
    let root = PathBuf::from(root_path);
    if !root.is_absolute() || !root.is_dir() { return Err("Choose an existing Catalyst campaign folder.".into()); }
    let manifest = read_project_manifest(&root)?;
    if manifest.project_type != "campaign" { return Err("Import destination must be a Catalyst campaign, not a manuscript project.".into()); }
    crate::campaign_wizard::read_campaign_manifest(&root)?;
    Ok(root)
}

fn unique_path(root: &Path, folder: &Path, stem: &str, ext: &str) -> Result<PathBuf, String> {
    let path = root.join(folder);
    fs::create_dir_all(&path).map_err(|e| format!("Could not create import folder: {e}"))?;
    for index in 0..10000 {
        let basename = if index == 0 { stem.to_string() } else { format!("{stem}-{index}") };
        let dest = path.join(format!("{basename}.{ext}"));
        if !dest.exists() { return Ok(dest); }
    }
    Err("Too many import filename collisions.".into())
}

fn validate_image_file(path: &Path) -> Result<(), String> {
    let ext = extension(path);
    let bytes = fs::read(path).map_err(|e| format!("Could not inspect image: {e}"))?;
    let valid = match ext.as_str() {
        "png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "jpg" | "jpeg" => bytes.starts_with(b"\xff\xd8\xff"),
        "gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        "webp" => bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
        "svg" => {
            let markup = std::str::from_utf8(&bytes)
                .map_err(|_| "Invalid UTF-8 SVG.".to_string())?
                .to_ascii_lowercase();
            markup.contains("<svg")
                && !["<script", "<foreignobject", "javascript:", "onload=", "onerror=", "href=", "xlink:href", "<iframe", "<image", "<use"]
                    .iter().any(|tag| markup.contains(tag))
        }
        _ => false,
    };
    if valid { Ok(()) } else { Err(format!("Image '{}' has an unsupported or unsafe file signature.", path.display())) }
}

fn text_from_source(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("Cannot read import source: {e}"))?;
    if bytes.len() > MAX_TEXT { return Err("Markdown or JSON text exceeds 512 KiB; split into smaller records.".into()); }
    String::from_utf8(bytes).map_err(|_| "Source is not valid UTF-8.".into())
}

fn write_note(root: &Path, title: &str, content: &str, subtype: &str) -> Result<(), String> {
    write_note_in(root, Path::new("Worldbuilding/Imported"), title, content, subtype, None, None)
}

fn write_note_in(
    root: &Path,
    folder: &Path,
    title: &str,
    content: &str,
    subtype: &str,
    source_frontmatter: Option<serde_yaml::Value>,
    original: Option<&str>,
) -> Result<(), String> {
    let stem = safe_name(title);
    let dest = unique_path(root, folder, &stem, "md")?;
    let relative = dest.strip_prefix(root).map_err(|e| e.to_string())?;
    let mut document = create_document(root, relative, "note", Some(title), content)?;
    document.metadata.insert("visibility".into(), serde_yaml::Value::String("gm-only".into()));
    document.metadata.insert("subtype".into(), serde_yaml::Value::String(subtype.into()));
    if let Some(frontmatter) = source_frontmatter {
        document.metadata.insert("source_frontmatter".into(), frontmatter);
    }
    if let Some(original) = original {
        document.metadata.insert("source_file".into(), serde_yaml::Value::String(original.into()));
    }
    save_document(root, &document, Some(&document.source_hash)).map(|_| ())
}

/// Split a leading YAML frontmatter block. Invalid YAML is left in the body
/// untouched rather than dropped.
fn split_source_frontmatter(raw: &str) -> (Option<serde_yaml::Value>, &str) {
    let text = raw.trim_start_matches('\u{feff}');
    let Some(rest) = text.strip_prefix("---\n").or_else(|| text.strip_prefix("---\r\n")) else {
        return (None, raw);
    };
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == "---" {
            let yaml = &rest[..offset];
            let body = &rest[offset + line.len()..];
            return match serde_yaml::from_str::<serde_yaml::Value>(yaml) {
                Ok(value @ serde_yaml::Value::Mapping(_)) => (Some(value), body),
                _ => (None, raw),
            };
        }
        offset += line.len();
    }
    (None, raw)
}

/// Sanitized parent folders of a source-relative path (the file name excluded).
fn safe_dirs(relative: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    if let Some(parent) = relative.parent() {
        for part in parent.components() {
            if let std::path::Component::Normal(name) = part {
                out.push(safe_name(&name.to_string_lossy()));
            }
        }
    }
    out
}

struct ImportContext {
    /// Sanitized name of the selected folder or file, so separate imports
    /// do not mix in one folder.
    source_folder: PathBuf,
}

fn copy_original(root: &Path, candidate: &Candidate, category: &str) -> Result<String, String> {
    copy_original_to(root, candidate, &Path::new("Assets/Imports").join(category))
}

fn copy_original_to(root: &Path, candidate: &Candidate, folder: &Path) -> Result<String, String> {
    let stem = safe_name(candidate.relative.file_stem().and_then(|x| x.to_str()).unwrap_or("source"));
    let ext = extension(&candidate.path);
    let dest = unique_path(root, folder, &stem, &ext)?;
    // create_new avoids clobbering user assets even if another importer runs.
    let mut input = fs::File::open(&candidate.path).map_err(|e| e.to_string())?;
    let mut output = fs::OpenOptions::new().write(true).create_new(true).open(&dest).map_err(|e| e.to_string())?;
    std::io::copy(&mut input, &mut output).map_err(|e| format!("Failed to import asset: {e}"))?;
    Ok(dest.strip_prefix(root).map_err(|e| e.to_string())?.to_string_lossy().replace('\\', "/"))
}

fn json_record(path: &Path) -> Result<(String, String), String> {
    let meta = fs::metadata(path).map_err(|e| format!("Cannot read JSON source: {e}"))?;
    let fallback = safe_name(path.file_stem().and_then(|s| s.to_str()).unwrap_or("VTT export"));
    if meta.len() > MAX_TEXT as u64 {
        return Ok((fallback, "## Large VTT export\n\nOriginal JSON source is preserved intact, but its contents exceed the current inline preview limit. No rule mechanics are converted.".into()));
    }
    let raw = text_from_source(path)?;
    let value: Json = serde_json::from_str(&raw).map_err(|e| format!("Invalid JSON export: {e}"))?;
    let (title, kind) = if let Some(obj) = value.as_object() {
        let title = obj.get("name").or_else(|| obj.get("title"))
            .and_then(Json::as_str).map(safe_name).unwrap_or_else(|| fallback.clone());
        let kind = obj.get("type").or_else(|| obj.get("_type"))
            .and_then(Json::as_str).unwrap_or("record").to_string();
        (title, kind)
    } else if let Some(items) = value.as_array() {
        (fallback, format!("array with {} records", items.len()))
    } else {
        return Err("JSON export must contain a record or array of records.".into());
    };
    let summary = format!("## Source record\n\n**Name:** {title}\n\n**Export type:** {kind}\n\nOriginal JSON is preserved under Assets/Imports. Mechanics, linked IDs and rule dependencies are not converted.");
    Ok((title, summary))
}

fn copy_one(root: &Path, candidate: &Candidate, format: &RpgFormat, context: &ImportContext, notes: &mut Vec<String>) -> Result<(usize, usize), String> {
    match format {
        RpgFormat::Markdown | RpgFormat::Obsidian => {
            let ext = extension(&candidate.path);
            if ext != "md" && ext != "markdown" {
                // Obsidian attachment: copy it, keeping the vault's folders.
                if ext != "pdf" { validate_image_file(&candidate.path)?; }
                let folder = Path::new("Assets/Imports/Obsidian Attachments").join(&context.source_folder).join(safe_dirs(&candidate.relative));
                copy_original_to(root, candidate, &folder)?;
                return Ok((0, 1));
            }
            let raw = text_from_source(&candidate.path)?;
            let stem = candidate.relative.file_stem().and_then(|s| s.to_str()).unwrap_or("Imported note");
            // The title keeps the source name so [[wikilinks]] still resolve.
            let title = stem.trim().to_string();
            let (frontmatter, body) = split_source_frontmatter(&raw);
            let folder = Path::new("Worldbuilding/Imported").join(&context.source_folder).join(safe_dirs(&candidate.relative));
            let original = copy_original_to(root, candidate,
                &Path::new("Assets/Imports/Markdown Sources").join(&context.source_folder).join(safe_dirs(&candidate.relative)))?;
            let subtype = if matches!(format, RpgFormat::Obsidian) { "imported-obsidian" } else { "imported-markdown" };
            write_note_in(root, &folder, if title.is_empty() { "Imported note" } else { &title }, body, subtype, frontmatter, Some(&original))?;
            Ok((1,1))
        }
        RpgFormat::Assets => {
            validate_image_file(&candidate.path)?;
            copy_original(root, candidate, "Images Maps Tokens")?;
            Ok((0,1))
        }
        RpgFormat::Pdf => {
            let mut head = [0u8; 5];
            fs::File::open(&candidate.path).map_err(|e| e.to_string())?
                .read_exact(&mut head).map_err(|e| e.to_string())?;
            if &head != b"%PDF-" { return Err("Selected PDF has no valid PDF header.".into()); }
            let stored = copy_original(root, candidate, "PDF References")?;
            write_note(root, &safe_name(candidate.relative.file_stem().and_then(|s| s.to_str()).unwrap_or("PDF")), &format!("## PDF reference\n\nSource file: `{}`\n\nOriginal PDF preserved. No OCR or automatic content extraction performed.",stored), "imported-pdf")?;
            Ok((1,1))
        }
        RpgFormat::Foundry | RpgFormat::Roll20 => {
            let (title, summary) = if extension(&candidate.path) == "zip" {
                let input = fs::File::open(&candidate.path).map_err(|e| e.to_string())?;
                let archive = zip::ZipArchive::new(input).map_err(|_| "Not a valid ZIP export.".to_string())?;
                if archive.len() > MAX_FILES { return Err("VTT ZIP contains too many entries.".into()); }
                (
                    safe_name(candidate.relative.file_stem().and_then(|s| s.to_str()).unwrap_or("VTT archive")),
                    format!("## VTT ZIP archive\n\nArchive contains {} entries. Original file preserved intact; linked data, macros and executable mechanics are not converted.", archive.len()),
                )
            } else {
                json_record(&candidate.path)?
            };
            let stored = copy_original(root, candidate, "VTT Exports")?;
            write_note(root, &safe_name(&title), &format!("{summary}\n\n**Saved source:** `{stored}`"), "imported-vtt")?;
            notes.push("VTT mechanics, online assets and linked object references have not been converted.".into());
            Ok((1,1))
        }
        RpgFormat::FantasyGrounds => {
            if extension(&candidate.path) == "mod" {
                let file = fs::File::open(&candidate.path).map_err(|e| e.to_string())?;
                let archive = zip::ZipArchive::new(file).map_err(|_| "Not a valid .mod ZIP archive.".to_string())?;
                if archive.len() > MAX_FILES { return Err("Fantasy Grounds module has too many entries.".into()); }
            } else {
                // db.xml exports are often many megabytes; check the header only.
                let mut head = [0u8; 512];
                let read = fs::File::open(&candidate.path).map_err(|e| e.to_string())?
                    .read(&mut head).map_err(|e| e.to_string())?;
                let text = String::from_utf8_lossy(&head[..read]);
                if !text.trim_start_matches('\u{feff}').trim_start().starts_with('<') {
                    return Err("XML export is invalid.".into());
                }
            }
            let stored = copy_original(root, candidate, "Fantasy Grounds")?;
            let title = safe_name(candidate.relative.file_stem().and_then(|s| s.to_str()).unwrap_or("FG Module"));
            write_note(root, &title, &format!("## Fantasy Grounds module\n\nSource file: `{stored}`\n\nThe original .mod/XML is preserved. Its ruleset-dependent records are not automatically converted."), "imported-fantasy-grounds")?;
            Ok((1,1))
        }
    }
}

#[tauri::command]
pub async fn apply_rpg_content_import(source_path: String, destination_root: String, format: RpgFormat) -> Result<RpgImportResult, String> {
    let root = project_root(&destination_root)?;
    let (source, list) = inspect_source(&source_path, &format)?;
    let canonical_root = fs::canonicalize(&root).map_err(|e| e.to_string())?;
    let canonical_source = fs::canonicalize(&source).map_err(|e| e.to_string())?;
    if canonical_source.starts_with(&canonical_root) {
        return Err("Cannot import a campaign into itself. Select a source outside the destination.".into());
    }
    if canonical_root.starts_with(&canonical_source) {
        return Err("The selected folder contains this campaign. Choose a folder that does not include the destination campaign.".into());
    }
    let source_name = if source.is_dir() {
        source.file_name()
    } else {
        source.file_stem()
    }
    .map(|name| safe_name(&name.to_string_lossy()))
    .unwrap_or_else(|| "Import".into());
    let context = ImportContext { source_folder: PathBuf::from(source_name) };
    let mut documents = 0usize;
    let mut assets = 0usize;
    let mut imported = 0usize;
    let mut notes = Vec::new();
    let mut skipped = Vec::new();
    // One bad file is reported and skipped; it never aborts the whole import
    // or leaves the user guessing which files arrived.
    for c in &list {
        match copy_one(&root, c, &format, &context, &mut notes) {
            Ok((d, a)) => { documents += d; assets += a; imported += 1; }
            Err(error) => skipped.push(format!("{}: {error}", c.relative.display())),
        }
    }
    notes.sort();
    notes.dedup();
    if imported == 0 {
        return Err(format!("Nothing was imported. {}", skipped.join("; ")));
    }
    Ok(RpgImportResult {
        imported, skipped, documents_created: documents, copied_assets: assets,
        destination: root.to_string_lossy().into_owned(), notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_traversal_by_only_using_sanitized_leaf_names() {
        assert_eq!(safe_name("../character"), "_character");
        assert_eq!(safe_name("entry: suspicious"), "entry_ suspicious");
    }
    #[test]
    fn format_extensions_are_explicit() {
        assert!(acceptable_extension(&RpgFormat::Obsidian, "md"));
        assert!(!acceptable_extension(&RpgFormat::Assets, "exe"));
        assert!(acceptable_extension(&RpgFormat::FantasyGrounds, "mod"));
        assert!(!acceptable_extension(&RpgFormat::Pdf, "png"));
    }
    fn campaign() -> (tempfile::TempDir, PathBuf) {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("Campaign");
        let input: crate::campaign_wizard::NewCampaignInput = serde_json::from_value(serde_json::json!({
            "project_root": root.to_string_lossy(), "name": "Campaign", "setup_mode": "quick-start",
            "basics": { "premise": "Test." },
        })).unwrap();
        crate::campaign_wizard::create_campaign(&input).unwrap();
        (parent, root)
    }

    #[tokio::test]
    async fn obsidian_import_keeps_folders_frontmatter_originals_and_attachments() {
        let (_parent, root) = campaign();
        let vault_parent = tempfile::tempdir().unwrap();
        let vault = vault_parent.path().join("My Vault");
        fs::create_dir_all(vault.join("NPCs")).unwrap();
        fs::write(vault.join("NPCs/Mira Fen.md"), "---\ntags: [npc]\nrole: steward\n---\nMira knows [[Briar Glen]].\n").unwrap();
        fs::write(vault.join("Briar Glen.md"), "A village.").unwrap();
        fs::write(vault.join("NPCs/map.png"), b"\x89PNG\r\n\x1a\nrest").unwrap();
        fs::write(vault.join("NPCs/broken.png"), b"not a png").unwrap();

        let result = apply_rpg_content_import(
            vault.to_string_lossy().into_owned(), root.to_string_lossy().into_owned(), RpgFormat::Obsidian,
        ).await.unwrap();
        assert_eq!(result.documents_created, 2);
        assert_eq!(result.skipped.len(), 1, "{:?}", result.skipped);
        assert!(result.skipped[0].contains("broken.png"));

        let mira = crate::document::service::read_document(&root, Path::new("Worldbuilding/Imported/My Vault/NPCs/Mira Fen.md")).unwrap();
        assert_eq!(mira.title.as_deref(), Some("Mira Fen"));
        assert!(mira.body.starts_with("Mira knows [[Briar Glen]]."), "{}", mira.body);
        assert!(!mira.body.contains("role: steward"));
        let fm = mira.metadata.get("source_frontmatter").unwrap();
        assert_eq!(fm.get("role").and_then(|v| v.as_str()), Some("steward"));
        let original = root.join("Assets/Imports/Markdown Sources/My Vault/NPCs/Mira Fen.md");
        assert_eq!(fs::read_to_string(original).unwrap(), fs::read_to_string(vault.join("NPCs/Mira Fen.md")).unwrap());
        assert!(root.join("Assets/Imports/Obsidian Attachments/My Vault/NPCs/map.png").is_file());
        // Source files are never modified.
        assert!(vault.join("NPCs/broken.png").is_file());

        // Importing again never overwrites; it adds numbered copies.
        apply_rpg_content_import(vault.to_string_lossy().into_owned(), root.to_string_lossy().into_owned(), RpgFormat::Obsidian).await.unwrap();
        assert!(root.join("Worldbuilding/Imported/My Vault/NPCs/Mira Fen-1.md").is_file());
        assert_eq!(crate::document::service::read_document(&root, Path::new("Worldbuilding/Imported/My Vault/NPCs/Mira Fen.md")).unwrap().identity, mira.identity);
    }

    #[tokio::test]
    async fn refuses_a_source_folder_that_contains_the_campaign() {
        let (parent, root) = campaign();
        fs::write(parent.path().join("note.md"), "x").unwrap();
        let error = apply_rpg_content_import(
            parent.path().to_string_lossy().into_owned(), root.to_string_lossy().into_owned(), RpgFormat::Markdown,
        ).await.unwrap_err();
        assert!(error.contains("contains this campaign"), "{error}");
    }

    #[tokio::test]
    async fn large_fantasy_grounds_xml_is_preserved() {
        let (_parent, root) = campaign();
        let source = tempfile::tempdir().unwrap();
        let xml = source.path().join("db.xml");
        let mut content = String::from("<?xml version=\"1.0\"?><root>");
        content.push_str(&"<entry>monster</entry>".repeat(40_000));
        content.push_str("</root>");
        fs::write(&xml, &content).unwrap();
        let result = apply_rpg_content_import(
            xml.to_string_lossy().into_owned(), root.to_string_lossy().into_owned(), RpgFormat::FantasyGrounds,
        ).await.unwrap();
        assert_eq!(result.copied_assets, 1);
        assert!(root.join("Assets/Imports/Fantasy Grounds/db.xml").is_file());
    }

    #[test]
    fn frontmatter_split_keeps_invalid_yaml_in_body() {
        let (fm, body) = split_source_frontmatter("---\na: [\n---\nText");
        assert!(fm.is_none());
        assert!(body.starts_with("---"));
        let (fm, body) = split_source_frontmatter("No frontmatter");
        assert!(fm.is_none());
        assert_eq!(body, "No frontmatter");
    }

    #[test]
    fn discovers_markdown_without_following_links() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".obsidian")).unwrap();
        fs::write(dir.path().join(".obsidian/config.md"), "hidden").unwrap();
        fs::write(dir.path().join("A.md"), "# A").unwrap();
        let files = candidates(dir.path(), &RpgFormat::Obsidian).unwrap();
        assert_eq!(files.len(), 1);
    }
}
