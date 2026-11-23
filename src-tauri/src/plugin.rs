//! Stable plugin contracts for Catalyst.
//!
//! This module defines host-facing plugin metadata and capability validation.
//! It does not execute third-party code yet. Keeping these contracts in Core
//! lets first-party and community plugins target the same permission boundary.

use std::{collections::{BTreeMap, BTreeSet}, sync::{Arc, Mutex}};

use serde::{Deserialize, Serialize};

pub mod document_api;

pub const PLUGIN_MANIFEST_VERSION: u32 = 1;
pub const PLUGIN_API_VERSION: &str = "1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginKind {
    Extension,
    Native,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginPublisher {
    FirstParty,
    Community,
    LocalDeveloper,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginManifest {
    #[serde(rename = "manifestVersion")]
    pub manifest_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    #[serde(rename = "minimumCatalystVersion", alias = "minimumLithoMarkVersion")]
    pub minimum_catalyst_version: String,
    #[serde(rename = "type")]
    pub kind: PluginKind,
    pub entry: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub permissions: BTreeSet<PluginPermission>,
    #[serde(rename = "activationEvents", default)]
    pub activation_events: BTreeSet<ActivationEvent>,
    #[serde(default)]
    pub contributes: PluginContributions,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginContributions {
    #[serde(default)]
    pub commands: Vec<CommandContribution>,
    #[serde(default)]
    pub views: Vec<ViewContribution>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandContribution {
    pub command: String,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewContribution {
    pub id: String,
    pub location: String,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActivationEvent {
    Command(String),
    View(String),
    ProjectOpen,
    DocumentType(String),
    Workspace(String),
    Export(String),
    Import(String),
}

impl Serialize for ActivationEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ActivationEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for ActivationEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command(id) => write!(f, "onCommand:{id}"),
            Self::View(id) => write!(f, "onView:{id}"),
            Self::ProjectOpen => write!(f, "onProjectOpen"),
            Self::DocumentType(kind) => write!(f, "onDocumentType:{kind}"),
            Self::Workspace(workspace) => write!(f, "onWorkspace:{workspace}"),
            Self::Export(format) => write!(f, "onExport:{format}"),
            Self::Import(format) => write!(f, "onImport:{format}"),
        }
    }
}

impl ActivationEvent {
    pub fn parse(raw: &str) -> Result<Self, String> {
        if raw == "onProjectOpen" {
            return Ok(Self::ProjectOpen);
        }
        for (prefix, constructor) in [
            ("onCommand:", Self::Command as fn(String) -> Self),
            ("onView:", Self::View),
            ("onDocumentType:", Self::DocumentType),
            ("onWorkspace:", Self::Workspace),
            ("onExport:", Self::Export),
            ("onImport:", Self::Import),
        ] {
            if let Some(value) = raw.strip_prefix(prefix) {
                if value.trim().is_empty() {
                    return Err(format!("Activation event '{raw}' requires a value."));
                }
                return Ok(constructor(value.to_string()));
            }
        }
        Err(format!("Unsupported activation event '{raw}'."))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PluginPermission {
    #[serde(rename = "project.read")]
    ProjectRead,
    #[serde(rename = "project.metadata.write")]
    ProjectMetadataWrite,
    #[serde(rename = "document.read")]
    DocumentRead,
    #[serde(rename = "document.create")]
    DocumentCreate,
    #[serde(rename = "document.write")]
    DocumentWrite,
    #[serde(rename = "document.delete")]
    DocumentDelete,
    #[serde(rename = "story-development.read")]
    StoryDevelopmentRead,
    #[serde(rename = "story-development.write")]
    StoryDevelopmentWrite,
    #[serde(rename = "editor.read-selection")]
    EditorReadSelection,
    #[serde(rename = "editor.modify-selection")]
    EditorModifySelection,
    #[serde(rename = "ui.command")]
    UiCommand,
    #[serde(rename = "ui.panel")]
    UiPanel,
    #[serde(rename = "ui.sidebar")]
    UiSidebar,
    #[serde(rename = "import.register")]
    ImportRegister,
    #[serde(rename = "export.register")]
    ExportRegister,
    #[serde(rename = "network.access")]
    NetworkAccess,
    #[serde(rename = "filesystem.read-external")]
    FilesystemReadExternal,
    #[serde(rename = "filesystem.write-external")]
    FilesystemWriteExternal,
    #[serde(rename = "process.execute")]
    ProcessExecute,
}

pub type CommandHandler = Arc<dyn Fn() -> Result<(), String> + Send + Sync>;

#[derive(Clone)]
pub struct CommandRegistry {
    handlers: Arc<Mutex<BTreeMap<String, RegisteredCommand>>>,
}

#[derive(Clone)]
struct RegisteredCommand {
    plugin_id: String,
    handler: CommandHandler,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}

impl CommandRegistry {
    pub fn register(
        &self,
        manifest: &PluginManifest,
        context: &ExtensionContext,
        command_id: &str,
        handler: CommandHandler,
    ) -> Result<DisposableFn, String> {
        context.require(PluginPermission::UiCommand)?;
        if manifest.id != context.plugin_id {
            return Err("Plugin manifest and extension context identities do not match.".into());
        }
        if !manifest
            .contributes
            .commands
            .iter()
            .any(|command| command.command == command_id)
        {
            return Err(format!(
                "Command '{command_id}' is not declared by plugin '{}'.",
                manifest.id
            ));
        }

        let mut handlers = self.handlers.lock().map_err(|_| "Command registry lock poisoned.")?;
        if handlers.contains_key(command_id) {
            return Err(format!("Command '{command_id}' is already registered."));
        }
        handlers.insert(
            command_id.to_string(),
            RegisteredCommand {
                plugin_id: manifest.id.clone(),
                handler,
            },
        );
        drop(handlers);

        let registry = self.clone();
        let command_id = command_id.to_string();
        let plugin_id = manifest.id.clone();
        Ok(DisposableFn::new(move || {
            registry.unregister_owned(&plugin_id, &command_id);
        }))
    }

    pub fn execute(&self, command_id: &str) -> Result<(), String> {
        let handler = {
            let handlers = self.handlers.lock().map_err(|_| "Command registry lock poisoned.")?;
            handlers
                .get(command_id)
                .map(|registered| Arc::clone(&registered.handler))
                .ok_or_else(|| format!("Command '{command_id}' is not registered."))?
        };
        handler()
    }

    pub fn contains(&self, command_id: &str) -> bool {
        self.handlers
            .lock()
            .map(|handlers| handlers.contains_key(command_id))
            .unwrap_or(false)
    }

    fn unregister_owned(&self, plugin_id: &str, command_id: &str) {
        if let Ok(mut handlers) = self.handlers.lock() {
            let owned = handlers
                .get(command_id)
                .map(|registered| registered.plugin_id == plugin_id)
                .unwrap_or(false);
            if owned {
                handlers.remove(command_id);
            }
        }
    }
}

pub trait Disposable: Send {
    fn dispose(&mut self);
}

pub struct DisposableFn {
    callback: Option<Box<dyn FnOnce() + Send>>,
}

impl DisposableFn {
    pub fn new(callback: impl FnOnce() + Send + 'static) -> Self {
        Self { callback: Some(Box::new(callback)) }
    }
}

impl Disposable for DisposableFn {
    fn dispose(&mut self) {
        if let Some(callback) = self.callback.take() {
            callback();
        }
    }
}

#[derive(Default)]
pub struct SubscriptionStore {
    subscriptions: Vec<Box<dyn Disposable>>,
}

impl SubscriptionStore {
    pub fn push(&mut self, disposable: impl Disposable + 'static) {
        self.subscriptions.push(Box::new(disposable));
    }

    pub fn len(&self) -> usize {
        self.subscriptions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.subscriptions.is_empty()
    }

    pub fn dispose_all(&mut self) {
        for subscription in self.subscriptions.iter_mut().rev() {
            subscription.dispose();
        }
        self.subscriptions.clear();
    }
}

impl Drop for SubscriptionStore {
    fn drop(&mut self) {
        self.dispose_all();
    }
}

pub struct ExtensionContext {
    pub plugin_id: String,
    pub publisher: PluginPublisher,
    pub api_version: String,
    grant: PluginGrant,
    subscriptions: SubscriptionStore,
}

impl ExtensionContext {
    pub fn new(grant: PluginGrant) -> Self {
        Self {
            plugin_id: grant.plugin_id.clone(),
            publisher: grant.publisher.clone(),
            api_version: PLUGIN_API_VERSION.to_string(),
            grant,
            subscriptions: SubscriptionStore::default(),
        }
    }

    pub fn require(&self, permission: PluginPermission) -> Result<(), String> {
        self.grant.require(permission)
    }

    pub fn granted(&self, permission: &PluginPermission) -> bool {
        self.grant.permissions.contains(permission)
    }

    pub fn subscriptions(&self) -> &SubscriptionStore {
        &self.subscriptions
    }

    pub fn subscriptions_mut(&mut self) -> &mut SubscriptionStore {
        &mut self.subscriptions
    }

    pub fn deactivate(&mut self) {
        self.subscriptions.dispose_all();
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginGrant {
    pub plugin_id: String,
    pub publisher: PluginPublisher,
    pub permissions: BTreeSet<PluginPermission>,
}

impl PluginManifest {
    pub fn parse_json(raw: &str) -> Result<Self, String> {
        let manifest: Self =
            serde_json::from_str(raw).map_err(|e| format!("Invalid plugin manifest JSON: {e}"))?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.manifest_version != PLUGIN_MANIFEST_VERSION {
            return Err(format!(
                "Unsupported plugin manifest version '{}'. Expected '{}'.",
                self.manifest_version, PLUGIN_MANIFEST_VERSION
            ));
        }
        if self.api_version != PLUGIN_API_VERSION {
            return Err(format!(
                "Unsupported Catalyst Plugin API version '{}'. Expected '{}'.",
                self.api_version, PLUGIN_API_VERSION
            ));
        }
        validate_plugin_id(&self.id)?;
        require_text("plugin name", &self.name)?;
        require_text("plugin version", &self.version)?;
        require_text("plugin author", &self.author)?;
        require_text("minimum Catalyst version", &self.minimum_catalyst_version)?;
        validate_entry(&self.entry)?;
        self.validate_contributions()?;
        Ok(())
    }

    fn validate_contributions(&self) -> Result<(), String> {
        let prefix = format!("{}.", self.id);
        let mut ids = BTreeSet::new();
        for command in &self.contributes.commands {
            require_text("command title", &command.title)?;
            if !command.command.starts_with(&prefix) {
                return Err(format!("Command '{}' must be namespaced by plugin id '{}'.", command.command, self.id));
            }
            if !ids.insert(command.command.as_str()) {
                return Err(format!("Duplicate command contribution '{}'.", command.command));
            }
        }
        for view in &self.contributes.views {
            require_text("view title", &view.title)?;
            require_text("view location", &view.location)?;
            if !view.id.starts_with(&prefix) {
                return Err(format!("View '{}' must be namespaced by plugin id '{}'.", view.id, self.id));
            }
            if !ids.insert(view.id.as_str()) {
                return Err(format!("Duplicate contribution id '{}'.", view.id));
            }
        }
        Ok(())
    }

    pub fn requested(&self, permission: &PluginPermission) -> bool {
        self.permissions.contains(permission)
    }
}

impl PluginGrant {
    pub fn new(
        manifest: &PluginManifest,
        publisher: PluginPublisher,
        permissions: BTreeSet<PluginPermission>,
    ) -> Result<Self, String> {
        if let Some(extra) = permissions
            .iter()
            .find(|permission| !manifest.permissions.contains(*permission))
        {
            return Err(format!(
                "Cannot grant permission '{extra:?}' because plugin '{}' did not request it.",
                manifest.id
            ));
        }
        Ok(Self {
            plugin_id: manifest.id.clone(),
            publisher,
            permissions,
        })
    }

    pub fn require(&self, permission: PluginPermission) -> Result<(), String> {
        if self.permissions.contains(&permission) {
            Ok(())
        } else {
            Err(format!(
                "Plugin '{}' is not authorized for permission '{permission:?}'.",
                self.plugin_id
            ))
        }
    }
}

fn validate_plugin_id(id: &str) -> Result<(), String> {
    let trimmed = id.trim();
    if trimmed != id || trimmed.is_empty() || trimmed.len() > 200 {
        return Err("Plugin id must be non-empty, trimmed, and at most 200 characters.".into());
    }
    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.len() < 3
        || parts.iter().any(|part| {
            part.is_empty()
                || !part
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
    {
        return Err(
            "Plugin id must use a reverse-domain-style identifier such as 'app.catalyst.git'."
                .into(),
        );
    }
    Ok(())
}

fn validate_entry(entry: &str) -> Result<(), String> {
    let entry = entry.trim();
    if entry.is_empty()
        || entry.starts_with('/')
        || entry.starts_with('\\')
        || entry.contains("..")
        || entry.contains(':')
    {
        return Err("Plugin entry must be a safe package-relative path.".into());
    }
    Ok(())
}

fn require_text(label: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{label} must not be empty."))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest_json(permissions: &[&str]) -> String {
        serde_json::json!({
            "manifestVersion": 1,
            "id": "com.example.timeline",
            "name": "Timeline",
            "version": "1.0.0",
            "author": "Example",
            "apiVersion": "1",
            "minimumCatalystVersion": "0.1.0",
            "type": "extension",
            "entry": "dist/index.js",
            "permissions": permissions,
        })
        .to_string()
    }

    #[test]
    fn parses_valid_manifest_and_permissions() {
        let manifest =
            PluginManifest::parse_json(&manifest_json(&["document.read", "ui.panel"])).unwrap();
        assert!(manifest.requested(&PluginPermission::DocumentRead));
        assert!(manifest.requested(&PluginPermission::UiPanel));
        assert!(!manifest.requested(&PluginPermission::NetworkAccess));
    }

    #[test]
    fn rejects_unknown_permission_instead_of_silently_granting_it() {
        let error =
            PluginManifest::parse_json(&manifest_json(&["manuscript.upload-everywhere"])).unwrap_err();
        assert!(error.contains("unknown variant"));
    }

    #[test]
    fn rejects_incompatible_api_version() {
        let raw = manifest_json(&[]).replace("\"apiVersion\":\"1\"", "\"apiVersion\":\"99\"");
        let error = PluginManifest::parse_json(&raw).unwrap_err();
        assert!(error.contains("Plugin API version"));
    }

    #[test]
    fn rejects_unsafe_entry_path() {
        let raw = manifest_json(&[]).replace("dist/index.js", "../outside.js");
        let error = PluginManifest::parse_json(&raw).unwrap_err();
        assert!(error.contains("package-relative"));
    }

    #[test]
    fn grant_cannot_exceed_requested_permissions() {
        let manifest = PluginManifest::parse_json(&manifest_json(&["document.read"])).unwrap();
        let permissions = BTreeSet::from([
            PluginPermission::DocumentRead,
            PluginPermission::NetworkAccess,
        ]);
        assert!(PluginGrant::new(&manifest, PluginPublisher::Community, permissions).is_err());
    }

    #[test]
    fn require_enforces_granted_permission_not_manifest_request() {
        let manifest =
            PluginManifest::parse_json(&manifest_json(&["document.read", "document.write"])).unwrap();
        let grant = PluginGrant::new(
            &manifest,
            PluginPublisher::Community,
            BTreeSet::from([PluginPermission::DocumentRead]),
        )
        .unwrap();

        assert!(grant.require(PluginPermission::DocumentRead).is_ok());
        assert!(grant.require(PluginPermission::DocumentWrite).is_err());
    }

    #[test]
    fn first_party_status_does_not_bypass_permissions() {
        let manifest = PluginManifest::parse_json(&manifest_json(&["ui.panel"])).unwrap();
        let grant = PluginGrant::new(
            &manifest,
            PluginPublisher::FirstParty,
            BTreeSet::from([PluginPermission::UiPanel]),
        )
        .unwrap();

        assert!(grant.require(PluginPermission::UiPanel).is_ok());
        assert!(grant.require(PluginPermission::DocumentRead).is_err());
    }

    #[test]
    fn parses_activation_events_and_contributions() {
        let mut value: serde_json::Value = serde_json::from_str(&manifest_json(&["ui.command", "ui.panel"])).unwrap();
        value["activationEvents"] = serde_json::json!([
            "onCommand:com.example.timeline.open",
            "onView:com.example.timeline.panel",
            "onProjectOpen"
        ]);
        value["contributes"] = serde_json::json!({
            "commands": [{"command":"com.example.timeline.open","title":"Open Timeline"}],
            "views": [{"id":"com.example.timeline.panel","location":"story-development","title":"Timeline"}]
        });
        let manifest = PluginManifest::parse_json(&value.to_string()).unwrap();
        assert!(manifest.activation_events.contains(&ActivationEvent::ProjectOpen));
        assert_eq!(manifest.contributes.commands.len(), 1);
        assert_eq!(manifest.contributes.views.len(), 1);
    }

    #[test]
    fn rejects_unknown_activation_event() {
        let mut value: serde_json::Value = serde_json::from_str(&manifest_json(&[])).unwrap();
        value["activationEvents"] = serde_json::json!(["whenever:I-feel-like-it"]);
        assert!(PluginManifest::parse_json(&value.to_string()).is_err());
    }

    #[test]
    fn rejects_empty_activation_event_value() {
        let mut value: serde_json::Value = serde_json::from_str(&manifest_json(&[])).unwrap();
        value["activationEvents"] = serde_json::json!(["onCommand:"]);
        assert!(PluginManifest::parse_json(&value.to_string()).is_err());
    }

    #[test]
    fn contributions_must_be_plugin_namespaced() {
        let mut value: serde_json::Value = serde_json::from_str(&manifest_json(&["ui.command"])).unwrap();
        value["contributes"] = serde_json::json!({
            "commands": [{"command":"other.plugin.open","title":"Open"}]
        });
        let error = PluginManifest::parse_json(&value.to_string()).unwrap_err();
        assert!(error.contains("namespaced"));
    }


    #[test]
    fn extension_context_exposes_grant_without_bypassing_permissions() {
        let manifest = PluginManifest::parse_json(&manifest_json(&["document.read", "document.write"])).unwrap();
        let grant = PluginGrant::new(
            &manifest,
            PluginPublisher::Community,
            BTreeSet::from([PluginPermission::DocumentRead]),
        ).unwrap();
        let context = ExtensionContext::new(grant);
        assert_eq!(context.plugin_id, "com.example.timeline");
        assert!(context.granted(&PluginPermission::DocumentRead));
        assert!(context.require(PluginPermission::DocumentWrite).is_err());
    }

    #[test]
    fn deactivate_disposes_subscriptions_in_reverse_order_once() {
        let manifest = PluginManifest::parse_json(&manifest_json(&[])).unwrap();
        let grant = PluginGrant::new(&manifest, PluginPublisher::Community, BTreeSet::new()).unwrap();
        let mut context = ExtensionContext::new(grant);
        let calls = Arc::new(Mutex::new(Vec::new()));
        for value in [1, 2, 3] {
            let calls = Arc::clone(&calls);
            context.subscriptions_mut().push(DisposableFn::new(move || {
                calls.lock().unwrap().push(value);
            }));
        }
        assert_eq!(context.subscriptions().len(), 3);
        context.deactivate();
        context.deactivate();
        assert_eq!(*calls.lock().unwrap(), vec![3, 2, 1]);
        assert!(context.subscriptions().is_empty());
    }

    #[test]
    fn subscription_store_disposes_on_drop() {
        let calls = Arc::new(Mutex::new(0));
        {
            let mut store = SubscriptionStore::default();
            let calls = Arc::clone(&calls);
            store.push(DisposableFn::new(move || {
                *calls.lock().unwrap() += 1;
            }));
        }
        assert_eq!(*calls.lock().unwrap(), 1);
    }


    fn command_manifest() -> PluginManifest {
        let mut value: serde_json::Value =
            serde_json::from_str(&manifest_json(&["ui.command"])).unwrap();
        value["contributes"] = serde_json::json!({
            "commands": [{"command":"com.example.timeline.open","title":"Open Timeline"}]
        });
        PluginManifest::parse_json(&value.to_string()).unwrap()
    }

    #[test]
    fn command_registry_registers_declared_owned_command() {
        let manifest = command_manifest();
        let grant = PluginGrant::new(
            &manifest,
            PluginPublisher::Community,
            BTreeSet::from([PluginPermission::UiCommand]),
        ).unwrap();
        let mut context = ExtensionContext::new(grant);
        let registry = CommandRegistry::default();
        let called = Arc::new(Mutex::new(0));
        let called_handler = Arc::clone(&called);
        let disposable = registry.register(
            &manifest,
            &context,
            "com.example.timeline.open",
            Arc::new(move || {
                *called_handler.lock().unwrap() += 1;
                Ok(())
            }),
        ).unwrap();
        context.subscriptions_mut().push(disposable);
        assert!(registry.contains("com.example.timeline.open"));
        registry.execute("com.example.timeline.open").unwrap();
        assert_eq!(*called.lock().unwrap(), 1);
        context.deactivate();
        assert!(!registry.contains("com.example.timeline.open"));
    }

    #[test]
    fn command_registry_rejects_undeclared_command() {
        let manifest = command_manifest();
        let grant = PluginGrant::new(
            &manifest,
            PluginPublisher::Community,
            BTreeSet::from([PluginPermission::UiCommand]),
        ).unwrap();
        let context = ExtensionContext::new(grant);
        let registry = CommandRegistry::default();
        let result = registry.register(
            &manifest,
            &context,
            "com.example.timeline.secret",
            Arc::new(|| Ok(())),
        );
        assert!(result.is_err());
    }

    #[test]
    fn command_registry_requires_ui_command_permission() {
        let manifest = command_manifest();
        let grant = PluginGrant::new(&manifest, PluginPublisher::Community, BTreeSet::new()).unwrap();
        let context = ExtensionContext::new(grant);
        let registry = CommandRegistry::default();
        let result = registry.register(
            &manifest,
            &context,
            "com.example.timeline.open",
            Arc::new(|| Ok(())),
        );
        assert!(result.is_err());
    }

    #[test]
    fn command_registry_rejects_duplicate_registration() {
        let manifest = command_manifest();
        let grant = PluginGrant::new(
            &manifest,
            PluginPublisher::Community,
            BTreeSet::from([PluginPermission::UiCommand]),
        ).unwrap();
        let context = ExtensionContext::new(grant);
        let registry = CommandRegistry::default();
        let first = registry.register(
            &manifest,
            &context,
            "com.example.timeline.open",
            Arc::new(|| Ok(())),
        ).unwrap();
        let second = registry.register(
            &manifest,
            &context,
            "com.example.timeline.open",
            Arc::new(|| Ok(())),
        );
        assert!(second.is_err());
        drop(first);
    }

    #[test]
    fn command_handler_errors_propagate_to_host() {
        let manifest = command_manifest();
        let grant = PluginGrant::new(
            &manifest,
            PluginPublisher::Community,
            BTreeSet::from([PluginPermission::UiCommand]),
        ).unwrap();
        let context = ExtensionContext::new(grant);
        let registry = CommandRegistry::default();
        let _registration = registry.register(
            &manifest,
            &context,
            "com.example.timeline.open",
            Arc::new(|| Err("plugin command failed".into())),
        ).unwrap();
        let error = registry.execute("com.example.timeline.open").unwrap_err();
        assert_eq!(error, "plugin command failed");
    }

}
