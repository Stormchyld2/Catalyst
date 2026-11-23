//! File-derived, project-scoped link resolution for LithoMark.
//!
//! Authoritative Markdown/YAML files remain project truth. This module builds an
//! in-memory projection on demand; callers may cache it, but the cache is always
//! disposable and rebuildable.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_yaml::Value;

use crate::document::service::{read_document, LithoMarkDocument};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LinkTarget {
    pub id: String,
    pub document_type: String,
    pub title: String,
    pub aliases: Vec<String>,
    pub relative_path: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LinkResolution {
    Resolved { target: LinkTarget },
    Ambiguous { query: String, candidates: Vec<LinkTarget> },
    Unresolved { query: String },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Backlink {
    pub source_id: String,
    pub source_title: String,
    pub source_path: String,
    pub authored_link: String,
}

#[derive(Clone, Debug, Default)]
pub struct ProjectLinkIndex {
    by_id: BTreeMap<String, LinkTarget>,
    by_name: BTreeMap<String, Vec<String>>,
    documents: Vec<IndexedDocument>,
}

#[derive(Clone, Debug)]
struct IndexedDocument {
    target: LinkTarget,
    body: String,
}

pub fn build_project_link_index(project_root: &Path) -> Result<ProjectLinkIndex, String> {
    if !project_root.is_absolute() {
        return Err("Project root must be absolute.".into());
    }

    let mut paths = Vec::new();
    collect_markdown_paths(project_root, project_root, &mut paths)?;
    paths.sort();

    let mut index = ProjectLinkIndex::default();
    for relative_path in paths {
        // Unknown/plain Markdown is allowed in a project. Only structured
        // LithoMark documents participate as stable link targets.
        let document = match read_document(project_root, &relative_path) {
            Ok(document) => document,
            Err(_) => continue,
        };
        index.insert(document)?;
    }
    Ok(index)
}

impl ProjectLinkIndex {
    fn insert(&mut self, document: LithoMarkDocument) -> Result<(), String> {
        let target = LinkTarget {
            id: document.identity.id.clone(),
            document_type: document.identity.document_type.clone(),
            title: display_title(&document),
            aliases: aliases(&document.metadata),
            relative_path: portable(&document.relative_path),
        };

        if self.by_id.contains_key(&target.id) {
            return Err(format!("Duplicate stable ID '{}' in project link index.", target.id));
        }

        let mut names = BTreeSet::new();
        names.insert(normalize(&target.title));
        for alias in &target.aliases {
            names.insert(normalize(alias));
        }
        for name in names.into_iter().filter(|name| !name.is_empty()) {
            self.by_name.entry(name).or_default().push(target.id.clone());
        }

        self.by_id.insert(target.id.clone(), target.clone());
        self.documents.push(IndexedDocument { target, body: document.body });
        Ok(())
    }

    pub fn resolve(&self, query: &str) -> LinkResolution {
        let query = wikilink_target(query).trim();
        if let Some(target) = self.by_id.get(query) {
            return LinkResolution::Resolved { target: target.clone() };
        }

        let key = normalize(query);
        let Some(ids) = self.by_name.get(&key) else {
            return LinkResolution::Unresolved { query: query.to_string() };
        };

        let candidates: Vec<_> = ids.iter().filter_map(|id| self.by_id.get(id).cloned()).collect();
        if candidates.len() == 1 {
            LinkResolution::Resolved { target: candidates[0].clone() }
        } else {
            LinkResolution::Ambiguous { query: query.to_string(), candidates }
        }
    }

    pub fn backlinks(&self, target_id: &str) -> Vec<Backlink> {
        let mut backlinks = Vec::new();
        for source in &self.documents {
            for authored in extract_wikilinks(&source.body) {
                if let LinkResolution::Resolved { target } = self.resolve(&authored) {
                    if target.id == target_id {
                        backlinks.push(Backlink {
                            source_id: source.target.id.clone(),
                            source_title: source.target.title.clone(),
                            source_path: source.target.relative_path.clone(),
                            authored_link: authored,
                        });
                    }
                }
            }
        }
        backlinks
    }
}

fn collect_markdown_paths(project_root: &Path, directory: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|e| format!("Could not read '{}': {e}", directory.display()))? {
        let entry = entry.map_err(|e| format!("Could not read project entry: {e}"))?;
        let path = entry.path();
        if path.file_name().and_then(|v| v.to_str()) == Some(".lithomark") {
            continue;
        }
        if path.is_dir() {
            collect_markdown_paths(project_root, &path, output)?;
        } else if path.extension().and_then(|v| v.to_str()).is_some_and(|v| v.eq_ignore_ascii_case("md")) {
            output.push(path.strip_prefix(project_root).map_err(|_| "Project document escaped the project root.".to_string())?.to_path_buf());
        }
    }
    Ok(())
}

fn aliases(metadata: &BTreeMap<String, Value>) -> Vec<String> {
    match metadata.get("aliases") {
        Some(Value::Sequence(values)) => values.iter().filter_map(Value::as_str).map(str::trim).filter(|v| !v.is_empty()).map(str::to_string).collect(),
        Some(Value::String(value)) if !value.trim().is_empty() => vec![value.trim().to_string()],
        _ => Vec::new(),
    }
}

fn display_title(document: &LithoMarkDocument) -> String {
    document.title.clone().unwrap_or_else(|| {
        document.relative_path.file_stem().and_then(|v| v.to_str()).unwrap_or(&document.identity.id).to_string()
    })
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn portable(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn wikilink_target(value: &str) -> &str {
    let trimmed = value.trim();
    let inner = trimmed.strip_prefix("[[").and_then(|v| v.strip_suffix("]]")).unwrap_or(trimmed);
    inner.split_once('|').map(|(target, _)| target).unwrap_or(inner)
}

fn extract_wikilinks(body: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("[[") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("]]") else { break };
        let raw = after[..end].trim();
        if !raw.is_empty() {
            links.push(raw.to_string());
        }
        rest = &after[end + 2..];
    }
    links
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::service::{create_document, save_document};

    fn add(root: &Path, path: &str, kind: &str, title: &str, body: &str, aliases: &[&str]) -> LithoMarkDocument {
        let mut doc = create_document(root, Path::new(path), kind, Some(title), body).unwrap();
        if !aliases.is_empty() {
            doc.metadata.insert("aliases".into(), Value::Sequence(aliases.iter().map(|v| Value::String((*v).into())).collect()));
            save_document(root, &doc, Some(&doc.source_hash)).unwrap();
            doc = read_document(root, Path::new(path)).unwrap();
        }
        doc
    }

    #[test]
    fn resolves_stable_id_title_and_alias() {
        let temp = tempfile::tempdir().unwrap();
        let character = add(temp.path(), "Story Development/Characters/Main Cast/Mara Vale.md", "character", "Mara Vale", "", &["Captain Vale"]);
        let index = build_project_link_index(temp.path()).unwrap();
        for query in [&character.identity.id, "Mara Vale", "[[mara vale]]", "[[Captain Vale|Mara]]"] {
            assert!(matches!(index.resolve(query), LinkResolution::Resolved { ref target } if target.id == character.identity.id));
        }
    }

    #[test]
    fn reports_ambiguous_names_instead_of_guessing() {
        let temp = tempfile::tempdir().unwrap();
        add(temp.path(), "Story Development/Locations/Old Quay.md", "location", "Old Quay", "", &[]);
        add(temp.path(), "Story Development/Organizations/Old Quay.md", "organization", "Old Quay", "", &[]);
        let index = build_project_link_index(temp.path()).unwrap();
        assert!(matches!(index.resolve("[[Old Quay]]"), LinkResolution::Ambiguous { candidates, .. } if candidates.len() == 2));
    }

    #[test]
    fn keeps_missing_links_unresolved() {
        let temp = tempfile::tempdir().unwrap();
        let index = build_project_link_index(temp.path()).unwrap();
        assert_eq!(index.resolve("[[Missing Person]]"), LinkResolution::Unresolved { query: "Missing Person".into() });
    }

    #[test]
    fn derives_backlinks_from_authoritative_markdown() {
        let temp = tempfile::tempdir().unwrap();
        let target = add(temp.path(), "Story Development/Characters/Main Cast/Mara Vale.md", "character", "Mara Vale", "", &[]);
        let scene = add(temp.path(), "Manuscript/Scene 01.md", "scene", "Scene 01", "Mara meets [[Mara Vale]] at the quay.", &[]);
        let index = build_project_link_index(temp.path()).unwrap();
        let backlinks = index.backlinks(&target.identity.id);
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].source_id, scene.identity.id);
        assert_eq!(backlinks[0].authored_link, "Mara Vale");
    }

    #[test]
    fn moved_and_renamed_target_keeps_stable_id_links_and_derived_backlinks() {
        use crate::document::service::move_document;

        let temp = tempfile::tempdir().unwrap();
        let old_path = Path::new("Story Development/Characters/Old Name.md");
        let new_path = Path::new("Story Development/Characters/Main Cast/New Name.md");
        let target = add(temp.path(), old_path.to_str().unwrap(), "character", "Old Name", "", &[]);
        let reference = format!("Reference by identity: [[{}|The captain]].", target.identity.id);
        let source = add(temp.path(), "Story Development/Worldbuilding/Notes.md", "note", "Notes", &reference, &[]);

        let moved = move_document(temp.path(), old_path, new_path, &target.source_hash).unwrap();
        assert_eq!(moved.identity.id, target.identity.id);
        let mut renamed = moved;
        renamed.title = Some("New Name".into());
        save_document(temp.path(), &renamed, Some(&renamed.source_hash)).unwrap();

        let index = build_project_link_index(temp.path()).unwrap();
        for query in [target.identity.id.as_str(), "New Name"] {
            assert!(matches!(index.resolve(query), LinkResolution::Resolved { ref target }
                if target.id == renamed.identity.id && target.relative_path == new_path.to_string_lossy()));
        }
        let backlinks = index.backlinks(&target.identity.id);
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].source_id, source.identity.id);
        assert_eq!(backlinks[0].authored_link, format!("{}|The captain", target.identity.id));
        assert_eq!(backlinks[0].source_path, "Story Development/Worldbuilding/Notes.md");
        assert!(matches!(index.resolve("Old Name"), LinkResolution::Unresolved { .. }));
    }

    #[test]
    fn does_not_index_another_project() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        add(first.path(), "Story Development/Characters/Main Cast/First.md", "character", "First Character", "", &[]);
        add(second.path(), "Story Development/Characters/Main Cast/Second.md", "character", "Second Character", "", &[]);
        let index = build_project_link_index(first.path()).unwrap();
        assert!(matches!(index.resolve("First Character"), LinkResolution::Resolved { .. }));
        assert!(matches!(index.resolve("Second Character"), LinkResolution::Unresolved { .. }));
    }
}
