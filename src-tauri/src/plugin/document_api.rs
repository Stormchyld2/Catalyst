//! Permission-gated document API exposed to Catalyst plugins.
//!
//! Plugins receive this facade rather than direct filesystem or persistence
//! access. Authoritative writes still flow through the Core Document Service.

use std::path::{Path, PathBuf};

use crate::document::service::{
    create_document, read_document, save_document, CatalystDocument, SaveResult,
};

use super::{ExtensionContext, PluginPermission};

pub struct PluginDocumentApi<'a> {
    project_root: &'a Path,
    context: &'a ExtensionContext,
}

impl<'a> PluginDocumentApi<'a> {
    pub fn new(project_root: &'a Path, context: &'a ExtensionContext) -> Result<Self, String> {
        if !project_root.is_absolute() {
            return Err("Plugin Document API requires an absolute project root.".into());
        }
        Ok(Self {
            project_root,
            context,
        })
    }

    pub fn read(&self, relative_path: &Path) -> Result<CatalystDocument, String> {
        self.context.require(PluginPermission::DocumentRead)?;
        read_document(self.project_root, relative_path)
    }

    pub fn create(
        &self,
        relative_path: &Path,
        document_type: &str,
        title: Option<&str>,
        body: &str,
    ) -> Result<CatalystDocument, String> {
        self.context.require(PluginPermission::DocumentCreate)?;
        create_document(
            self.project_root,
            relative_path,
            document_type,
            title,
            body,
        )
    }

    pub fn save(
        &self,
        document: &CatalystDocument,
        expected_source_hash: Option<&str>,
    ) -> Result<SaveResult, String> {
        self.context.require(PluginPermission::DocumentWrite)?;
        save_document(self.project_root, document, expected_source_hash)
    }

    /// Returns the project-relative path a plugin can safely retain as a
    /// document locator. The absolute project root is deliberately not exposed.
    pub fn relative_path(document: &CatalystDocument) -> PathBuf {
        document.relative_path.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::plugin::{PluginGrant, PluginManifest, PluginPublisher};

    fn manifest(permissions: &[&str]) -> PluginManifest {
        PluginManifest::parse_json(
            &serde_json::json!({
                "manifestVersion": 1,
                "id": "com.example.documents",
                "name": "Documents",
                "version": "1.0.0",
                "author": "Example",
                "apiVersion": "1",
                "minimumLithoMarkVersion": "0.1.0",
                "type": "extension",
                "entry": "dist/index.js",
                "permissions": permissions
            })
            .to_string(),
        )
        .unwrap()
    }

    fn context(
        requested: &[&str],
        granted: BTreeSet<PluginPermission>,
    ) -> ExtensionContext {
        let manifest = manifest(requested);
        let grant =
            PluginGrant::new(&manifest, PluginPublisher::Community, granted).unwrap();
        ExtensionContext::new(grant)
    }

    #[test]
    fn read_requires_document_read_permission() {
        let temp = tempfile::tempdir().unwrap();
        let path = Path::new("Notes/Test.md");
        create_document(temp.path(), path, "note", Some("Test"), "Body").unwrap();

        let context = context(&[], BTreeSet::new());
        let api = PluginDocumentApi::new(temp.path(), &context).unwrap();
        let error = api.read(path).unwrap_err();
        assert!(error.contains("DocumentRead"));
    }

    #[test]
    fn create_requires_document_create_permission() {
        let temp = tempfile::tempdir().unwrap();
        let context = context(
            &["document.create"],
            BTreeSet::from([PluginPermission::DocumentCreate]),
        );
        let api = PluginDocumentApi::new(temp.path(), &context).unwrap();
        let created = api
            .create(
                Path::new("Notes/Created.md"),
                "note",
                Some("Created"),
                "Plugin body",
            )
            .unwrap();
        assert_eq!(created.title.as_deref(), Some("Created"));
        assert!(temp.path().join("Notes/Created.md").is_file());
    }

    #[test]
    fn create_does_not_imply_read_permission() {
        let temp = tempfile::tempdir().unwrap();
        let context = context(
            &["document.create"],
            BTreeSet::from([PluginPermission::DocumentCreate]),
        );
        let api = PluginDocumentApi::new(temp.path(), &context).unwrap();
        let path = Path::new("Notes/Created.md");
        api.create(path, "note", None, "Body").unwrap();
        assert!(api.read(path).is_err());
    }

    #[test]
    fn save_requires_document_write_and_preserves_conflict_detection() {
        let temp = tempfile::tempdir().unwrap();
        let path = Path::new("Notes/Test.md");
        let mut document =
            create_document(temp.path(), path, "note", Some("Test"), "Original").unwrap();

        let no_write = context(&["document.read"], BTreeSet::from([PluginPermission::DocumentRead]));
        let api = PluginDocumentApi::new(temp.path(), &no_write).unwrap();
        document.body = "Plugin edit\n".into();
        assert!(api.save(&document, Some(&document.source_hash)).is_err());

        let write = context(
            &["document.write"],
            BTreeSet::from([PluginPermission::DocumentWrite]),
        );
        let api = PluginDocumentApi::new(temp.path(), &write).unwrap();

        std::fs::write(
            temp.path().join(path),
            std::fs::read_to_string(temp.path().join(path))
                .unwrap()
                .replace("Original", "External"),
        )
        .unwrap();

        let error = api
            .save(&document, Some(&document.source_hash))
            .unwrap_err();
        assert!(error.contains("conflict"));
        assert!(std::fs::read_to_string(temp.path().join(path))
            .unwrap()
            .contains("External"));
    }

    #[test]
    fn document_service_path_safety_remains_enforced_for_plugins() {
        let temp = tempfile::tempdir().unwrap();
        let context = context(
            &["document.create"],
            BTreeSet::from([PluginPermission::DocumentCreate]),
        );
        let api = PluginDocumentApi::new(temp.path(), &context).unwrap();
        let error = api
            .create(Path::new("../outside.md"), "note", None, "Nope")
            .unwrap_err();
        assert!(error.contains("escape"));
    }

    #[test]
    fn plugin_api_rejects_relative_project_root() {
        let context = context(&[], BTreeSet::new());
        assert!(PluginDocumentApi::new(Path::new("relative-project"), &context).is_err());
    }
}
