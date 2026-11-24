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
    pub documents_created: usize,
    pub copied_assets: usize,
    pub destination: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
struct Candidate { path: PathBuf, relative: PathBuf, size: u64 }

fn acceptable_extension(format: &RpgFormat, extension: &str) -> bool {
    match format {
        RpgFormat::Markdown | RpgFormat::Obsidian => matches!(extension, "md" | "markdown"),
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
        RpgFormat::Markdown | RpgFormat::Obsidian => "Create editable Markdown records in Worldbuilding/Imported, retaining original Markdown. Obsidian wikilinks remain text until reference resolution is supported.",
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
        RpgFormat::Obsidian => vec!["Obsidian wikilinks, embeds and plugins are not automatically remapped.".into()],
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
    let stem = safe_name(title);
    let dest = unique_path(root, Path::new("Worldbuilding/Imported"), &stem, "md")?;
    let relative = dest.strip_prefix(root).map_err(|e| e.to_string())?;
    let mut document = create_document(root, relative, "note", Some(title), content)?;
    document.metadata.insert("visibility".into(), serde_yaml::Value::String("gm-only".into()));
    document.metadata.insert("subtype".into(), serde_yaml::Value::String(subtype.into()));
    save_document(root, &document, Some(&document.source_hash)).map(|_| ())
}

fn copy_original(root: &Path, candidate: &Candidate, category: &str) -> Result<String, String> {
    let stem = safe_name(candidate.relative.file_stem().and_then(|x| x.to_str()).unwrap_or("source"));
    let ext = extension(&candidate.path);
    let dest = unique_path(root, Path::new("Assets/Imports").join(category).as_path(), &stem, &ext)?;
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

fn copy_one(root: &Path, candidate: &Candidate, format: &RpgFormat, notes: &mut Vec<String>) -> Result<(usize, usize), String> {
    match format {
        RpgFormat::Markdown | RpgFormat::Obsidian => {
            let raw = text_from_source(&candidate.path)?;
            let stem = candidate.relative.file_stem().and_then(|s| s.to_str()).unwrap_or("Imported note");
            let title = safe_name(stem);
            // Preserve the Markdown prose and frontmatter as-is in the body.
            write_note(root, &title, &raw, "imported-markdown")?;
            Ok((1,0))
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
                let text = text_from_source(&candidate.path)?;
                if !text.contains('<') { return Err("XML export is invalid.".into()); }
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
    let mut documents = 0usize;
    let mut assets = 0usize;
    let mut notes = Vec::new();
    for c in &list {
        let (d,a) = copy_one(&root,c,&format,&mut notes)?;
        documents += d; assets += a;
    }
    notes.sort();
    notes.dedup();
    Ok(RpgImportResult {
        imported: list.len(), documents_created: documents, copied_assets: assets,
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
