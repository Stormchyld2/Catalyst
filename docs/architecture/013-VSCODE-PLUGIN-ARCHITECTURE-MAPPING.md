# LithoMark Writer OS — VS Code Plugin Architecture Mapping

**Document:** 013-VSCODE-PLUGIN-ARCHITECTURE-MAPPING.md
**Status:** Locked Implementation Direction
**Project:** LithoMark Writer OS
**Related:** 002-DOCUMENT-MODEL.md, 003-STABLE-ID-SYSTEM.md, 005-PLUGIN-ARCHITECTURE.md, 012-LITHOMARK-WRITER-OS-INTEGRATION-ARCHITECTURE.md

---

# 1. Decision

LithoMark uses the Visual Studio Code extension architecture as its primary conceptual model for plugin development, adapted for a local-first Writer OS.

LithoMark does not attempt binary or API compatibility with VS Code extensions.

The target developer experience is deliberately familiar:

```text
Manifest
  ↓
Activation
  ↓
Extension Context
  ↓
Contributions + Runtime API
  ↓
Disposables / Cleanup
```

Obsidian remains an important source of writing, wiki, backlink, graph, metadata, and knowledge-management plugin concepts.

# 2. What LithoMark Adopts from VS Code

LithoMark adopts these patterns:

- declarative plugin manifest;
- unique publisher/plugin identity;
- independently versioned Plugin API;
- explicit activation/deactivation lifecycle;
- lazy activation;
- static contribution declarations;
- runtime registration APIs;
- command registry;
- context-aware menus and UI contributions;
- settings/configuration contributions;
- extension context;
- subscription/disposable cleanup model;
- capability-oriented APIs rather than direct UI/internal access;
- restricted execution for untrusted or insufficiently authorized capabilities;
- first-party plugins using the same public contracts where practical.

# 3. What LithoMark Changes

LithoMark is a Writer OS, not a source-code editor.

Its plugin API therefore exposes domain concepts rather than VS Code implementation concepts:

```text
lithomark.project
lithomark.documents
lithomark.manuscript
lithomark.storyDevelopment
lithomark.editor
lithomark.workspace
lithomark.commands
lithomark.events
lithomark.ui
lithomark.settings
lithomark.import
lithomark.export
```

Stable project/document/entity IDs are preferred over raw paths.

Plugins must not treat Svelte, Tauri, TipTap, SQLite, or filesystem layout details as the public API.

# 4. Manifest Mapping

VS Code's package.json combines package metadata and extension declarations.

LithoMark uses a dedicated:

`lithomark-plugin.json`

The manifest remains declarative and contains identity, compatibility, entry point, permissions, activation rules, and contributions.

Target shape:

```json
{
  "manifestVersion": 1,
  "id": "com.example.scene-dashboard",
  "name": "Scene Dashboard",
  "version": "1.0.0",
  "author": "Example Developer",
  "apiVersion": "1",
  "minimumLithoMarkVersion": "0.1.0",
  "type": "extension",
  "entry": "dist/extension.js",
  "permissions": [
    "project.read",
    "document.read",
    "ui.panel",
    "ui.command"
  ],
  "activationEvents": [
    "onCommand:com.example.scene-dashboard.open",
    "onView:com.example.scene-dashboard.panel"
  ],
  "contributes": {
    "commands": [
      {
        "command": "com.example.scene-dashboard.open",
        "title": "Open Scene Dashboard"
      }
    ],
    "views": [
      {
        "id": "com.example.scene-dashboard.panel",
        "location": "story-development",
        "title": "Scene Dashboard"
      }
    ]
  }
}
```

# 5. Activation

Plugins should be lazy by default.

A plugin is not executed merely because it is installed.

Initial LithoMark activation-event vocabulary should support:

- `onCommand:<id>`
- `onView:<id>`
- `onProjectOpen`
- `onDocumentType:<type>`
- `onWorkspace:<workspace>`
- `onExport:<format>`
- `onImport:<format>`

Broad startup activation should be exceptional.

Static contributions may implicitly establish activation where unambiguous, as VS Code does for modern command contributions.

Activation never grants permissions. It only determines when already-authorized code may start.

# 6. Extension Entry Point

The preferred Tier 1 TypeScript contract is:

```ts
import * as lithomark from "@lithomark/api";

export function activate(context: lithomark.ExtensionContext) {
  const disposable = lithomark.commands.registerCommand(
    "com.example.scene-dashboard.open",
    () => {
      // Plugin behavior.
    }
  );

  context.subscriptions.push(disposable);
}

export function deactivate() {}
```

The exact package name is provisional until implementation.

# 7. Extension Context

Each activated plugin receives an ExtensionContext owned by the host.

Initial responsibilities:

- plugin identity;
- granted permissions;
- subscriptions/disposables;
- namespaced global settings/state;
- namespaced project/workspace state;
- plugin installation/package location where safe;
- controlled durable plugin-data access;
- host/API version information.

ExtensionContext must not expose unrestricted Tauri handles, raw application internals, database connections, or arbitrary project filesystem access.

# 8. Disposables

Runtime registrations return Disposable handles.

Examples:

- command registration;
- event subscription;
- view registration;
- status contribution;
- editor contribution.

Plugins place these in `context.subscriptions`.

On disable, unload, failure isolation, or application shutdown, the host disposes registered resources.

This prevents stale commands/listeners/views after plugin deactivation.

# 9. Contributions

The initial declarative contribution model should include:

- `commands`
- `views`
- `menus`
- `configuration`
- `keybindings`
- `documentTypes`
- `entityTypes`
- `templates`
- `importers`
- `exporters`

Later contribution points may include themes, context-panel cards, editor decorations, graph providers, Story Development visualizations, and publishing providers.

A declaration makes a capability discoverable to the host; it does not itself grant access to project data.

# 10. Commands

Commands are a central interoperability primitive.

Command IDs must be namespaced by plugin ID.

Commands may be invoked from:

- command palette;
- menus;
- keyboard shortcuts;
- toolbar actions;
- context menus;
- other authorized plugins/API callers.

The host activates the owning plugin when required and checks relevant permissions before privileged domain operations occur.

# 11. Context Conditions

LithoMark should support a constrained equivalent of VS Code's context/when clauses.

Useful context keys include:

- `projectOpen`
- `workspace == write`
- `workspace == story-development`
- `workspace == review`
- `activeDocumentType == scene`
- `activeDocumentType == chapter`
- `hasSelection`
- `projectTrusted`
- `pluginPermission:<permission>`

The expression language must remain host-controlled and must not evaluate arbitrary JavaScript.

# 12. Workspace and Domain API

Unlike a generic filesystem workspace, LithoMark exposes writer-domain operations.

Examples:

```ts
lithomark.documents.get(id)
lithomark.documents.update(id, change)
lithomark.manuscript.getScenes(chapterId)
lithomark.storyDevelopment.getEntity(id)
lithomark.workspace.getActiveContext()
```

Actual signatures will be defined as implementation proceeds.

All authoritative document writes route through the Document Service.

# 13. Permission Boundary

VS Code-inspired ergonomics do not weaken LithoMark's security model.

The host checks the granted permission at the API boundary.

Examples:

```text
documents.get()
  → requires document.read

documents.update()
  → requires document.write

ui.registerPanel()
  → requires ui.panel

network request
  → requires network.access
```

First-party publisher status does not bypass this boundary.

# 14. Project Trust

LithoMark should distinguish plugin authorization from project trust.

Opening a project does not imply that project-provided executable plugin code is trusted.

Projects remain readable in a restricted/safe mode when optional executable plugins cannot run.

This is particularly important for projects cloned from Git or downloaded from another person.

# 15. UI Isolation

Plugins must not manipulate LithoMark's application DOM directly.

UI contributions operate through supported components, views, commands, menus, and controlled plugin surfaces.

This preserves the ability to replace Svelte components or redesign the shell without breaking compliant plugins.

Rich plugin views may eventually use a sandboxed webview-like surface with explicit messaging and permissions.

# 16. First-Party Plugin Rule

Official LithoMark plugins are reference consumers of the public API.

Candidate first-party plugins include:

- Review;
- Beta / ARC;
- Listen / TTS;
- Audiobook Studio;
- Publishing Studio;
- Git / GitHub;
- Graph View;
- advanced Story Development visualizations.

If one of these requires a legitimate capability absent from the public API, the preferred response is to design a safe public capability rather than add a private persistence shortcut.

# 17. Plugin SDK

The future SDK should provide:

```text
@lithomark/api
@lithomark/plugin-test
LithoMark type definitions
manifest JSON schema
sample plugin
packaging command
development host / reload workflow
documentation
```

The sample plugin should be intentionally small and demonstrate:

- manifest;
- activation;
- command;
- one view;
- one read permission;
- one optional write permission;
- settings;
- disposal;
- tests.

# 18. Community Development Goal

A TypeScript developer familiar with VS Code extensions should recognize LithoMark's lifecycle and contribution model immediately.

They should only need to learn LithoMark's writer-domain API and permission model.

# 19. Explicit Non-Goals

LithoMark will not:

- execute VS Code extensions unchanged;
- expose the VS Code API as its own;
- make Node.js unrestricted access a requirement for Tier 1 plugins;
- expose raw TipTap internals as the editor API;
- allow plugins to bypass stable IDs or the Document Service;
- require a plugin for a writer to recover or read the core manuscript;
- treat community plugins as implicitly trusted because they are installed.

# 20. Implementation Order

The plugin foundation should be implemented in this order:

1. Manifest + identity + permission contracts. **Started and tested.**
2. Activation-event and contribution manifest models.
3. ExtensionContext + Disposable abstractions.
4. Command registry.
5. Permission-gated Document API backed by Document Service.
6. Domain event bus with permission-filtered payloads.
7. Settings/state API.
8. View/UI contribution host.
9. TypeScript SDK and type definitions.
10. First LithoMark sample plugin.
11. First optional first-party plugin migrated onto the same API.
12. Packaging/signing/community registry later.

This order keeps the API grounded in real Core services while avoiding premature marketplace/runtime complexity.

---

**Governing implementation rule:** The plugin API is a stable, permission-controlled Writer OS contract. Internal implementation remains replaceable.
