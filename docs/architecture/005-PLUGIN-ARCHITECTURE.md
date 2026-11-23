# Catalyst — Plugin Architecture

**Document:** 005-PLUGIN-ARCHITECTURE.md  
**Status:** Locked Baseline  
**Project:** Catalyst  
**Parent Specification:** 000-CATALYST-ARCHITECTURE.md  
**Related Specifications:** 001-PROJECT-FILE-FORMAT.md, 002-DOCUMENT-MODEL.md, 003-STABLE-ID-SYSTEM.md, 004-CANON-MODEL.md

---

# 1. Purpose

This specification defines the Catalyst plugin architecture.

Catalyst is designed as an extensible writing platform.

The plugin system allows first-party and community developers to extend Catalyst without requiring modifications to Catalyst Core.

The system must balance:

- Extensibility
- Security
- Stability
- Writer control
- Backward compatibility
- Project portability
- Community development

Plugins must extend Catalyst through documented APIs rather than depending upon internal implementation details.

---

# 2. Fundamental Architecture

The plugin architecture is:

```text
Community / First-Party Plugin
            │
            ▼
     Catalyst Plugin API
            │
            ▼
      Permission Layer
            │
            ▼
      Catalyst Domain API
            │
            ▼
        Catalyst Core
            │
            ▼
      Project Services
```

Plugins must not normally bypass this architecture.

---

# 3. Core Principle

Plugins extend Catalyst.

Plugins do not become Catalyst Core.

Core project integrity must not depend upon an optional plugin unless the writer explicitly chooses to use a plugin-defined project feature.

Catalyst must remain usable when optional plugins are disabled.

---

# 4. Plugin Tiers

Catalyst defines two primary plugin tiers:

```text
Tier 1
Catalyst Extension

Tier 2
Native Catalyst Plugin
```

These tiers have different security and capability requirements.

---

# 5. Tier 1 — Catalyst Extensions

Tier 1 extensions are the standard community plugin format.

They should be the preferred mechanism for most plugins.

Typical technologies may include:

- TypeScript
- JavaScript
- HTML
- CSS
- Catalyst Plugin API

Tier 1 extensions must operate within Catalyst-defined capabilities.

They should not receive unrestricted operating-system access.

---

# 6. Tier 1 Use Cases

Tier 1 extensions may provide:

- Sidebar panels
- Commands
- Story Bible tools
- Plotting tools
- Timeline visualizations
- Genre tools
- Templates
- Custom entity views
- Reports
- Statistics
- Editor tools
- Exporters
- Importers
- Canon visualizations
- Relationship graphs
- Specialized search
- Workflow tools

Examples:

```text
Fantasy Calendar
Family Tree
Crime Evidence Board
Romance Beat Tracker
Name Generator
Character Relationship Graph
Scene Progress Dashboard
```

---

# 7. Tier 2 — Native Catalyst Plugins

Tier 2 plugins may contain native code.

Potential technologies include:

- Rust
- Tauri plugins
- Platform-specific native components

Native plugins may be required for:

- Operating-system integrations
- Specialized file processing
- Hardware access
- High-performance computation
- Native third-party SDKs
- Advanced system integrations

---

# 8. Native Plugin Risk

Native plugins inherently carry greater risk.

A native plugin may potentially access:

- Filesystem
- Network
- Operating-system services
- Native libraries
- Process execution
- Sensitive project information

Therefore Tier 2 plugins require stronger review and authorization.

---

# 9. Default Community Model

The standard Catalyst Community Plugin ecosystem should prioritize Tier 1 extensions.

Native plugins should be exceptional.

The preferred model is:

```text
Most plugins
    ↓
Tier 1 Extension
    ↓
Controlled Catalyst APIs
```

rather than:

```text
Every plugin
    ↓
Native unrestricted code
```

---

# 10. Plugin Identity

Every plugin must have a globally unique identifier.

Recommended format:

```text
reverse-domain-style identifier
```

Example:

```text
com.example.fantasy-calendar
```

First-party plugins may use:

```text
app.catalyst.git
app.catalyst.github
```

The final Catalyst-owned namespace should correspond to the project's eventual official domain.

---

# 11. Plugin Manifest

Every plugin must contain a manifest.

Recommended filename:

```text
lithomark-plugin.json
```

Example:

```json
{
  "manifestVersion": 1,
  "id": "com.example.fantasy-calendar",
  "name": "Fantasy Calendar",
  "version": "1.4.2",
  "description": "Fantasy calendar and chronology tools for Catalyst.",
  "author": "Example Developer",
  "license": "MIT",
  "apiVersion": "1",
  "minimumLithoMarkVersion": "1.0.0",
  "type": "extension",
  "entry": "dist/index.js",
  "permissions": [
    "project.read",
    "timeline.read",
    "timeline.write",
    "ui.sidebar"
  ]
}
```

---

# 12. Manifest Requirements

The manifest should define at minimum:

- Manifest version
- Plugin ID
- Plugin name
- Plugin version
- Author
- Plugin type
- Entry point
- Required Catalyst API version
- Minimum supported Catalyst version
- Requested permissions

Optional fields may include:

- Description
- Homepage
- Repository
- License
- Funding
- Issue tracker
- Documentation
- Icon
- Screenshots
- Supported project types

---

# 13. Semantic Versioning

Plugins should use semantic versioning where practical.

Example:

```text
1.4.2
```

Catalyst Plugin API versions must be explicitly declared.

Compatibility must not be inferred solely from plugin version.

---

# 14. API Versioning

The Catalyst Plugin API is versioned independently from the Catalyst application.

Example:

```text
Catalyst application:
3.2.0

Plugin API:
2
```

A plugin declares:

```json
{
  "apiVersion": "2"
}
```

This allows application development without unnecessarily breaking plugins.

---

# 15. Compatibility

Before loading a plugin, Catalyst must verify:

- Manifest validity
- Plugin API compatibility
- Minimum application version
- Platform compatibility
- Required capabilities
- Plugin integrity where applicable

An incompatible plugin must not be loaded blindly.

---

# 16. Permission Model

Plugins operate under explicit permissions.

Permissions must follow least-privilege principles.

A plugin receives only the capabilities it declares and the writer authorizes.

---

# 17. Permission Categories

Initial permission categories may include:

```text
project
document
manuscript
story-bible
timeline
canon
continuity
research
templates
ui
editor
import
export
network
filesystem
clipboard
settings
notifications
```

---

# 18. Project Permissions

Examples:

```text
project.read
project.metadata.write
project.create-object
```

A plugin with `project.read` must not automatically receive permission to modify the project.

---

# 19. Manuscript Permissions

Examples:

```text
manuscript.read
manuscript.write
manuscript.create
manuscript.delete
```

Reading an unpublished manuscript is a meaningful permission.

Catalyst should clearly communicate this during installation.

---

# 20. Story Bible Permissions

Examples:

```text
story-bible.read
story-bible.write
story-bible.create
story-bible.delete
```

---

# 21. Timeline Permissions

Examples:

```text
timeline.read
timeline.write
timeline.create
timeline.delete
```

---

# 22. Canon Permissions

Examples:

```text
canon.read
canon.propose
canon.write
canon.delete
continuity.read
continuity.register-rules
```

Plugins must not receive canon modification permission merely because they can read canon.

---

# 23. Research Permissions

Examples:

```text
research.read
research.write
research.create
```

Binary research assets may require additional filesystem or asset permissions.

---

# 24. UI Permissions

Examples:

```text
ui.sidebar
ui.panel
ui.toolbar
ui.statusbar
ui.command
ui.context-menu
ui.settings
```

UI permissions allow plugins to contribute interface elements without unrestricted application control.

---

# 25. Editor Permissions

Examples:

```text
editor.read-selection
editor.modify-selection
editor.read-document
editor.modify-document
editor.annotation
editor.command
```

A plugin capable only of operating on selected text should not require access to the entire project.

---

# 26. Import Permissions

Plugins may register importers.

Example:

```text
import.register
```

Importers should convert external content through Catalyst domain services.

They must not bypass stable ID generation or project validation.

---

# 27. Export Permissions

Plugins may register exporters.

Example:

```text
export.register
```

Exporters may receive normalized project information according to granted permissions.

---

# 28. Network Permission

Network access must be explicit.

Example:

```text
network.access
```

A plugin without network permission must not be capable of transmitting manuscript content externally through supported APIs.

---

# 29. Network Domains

Catalyst may support domain-scoped network permissions.

Example:

```json
{
  "network": {
    "domains": [
      "api.example.com"
    ]
  }
}
```

This is preferable to unrestricted internet access where practical.

---

# 30. External Filesystem Permission

Access outside the current Catalyst project must require explicit permission.

Examples:

```text
filesystem.read-external
filesystem.write-external
```

A plugin should not automatically receive access to the user's entire computer.

---

# 31. Project Filesystem Access

Plugins should normally use Catalyst APIs rather than raw filesystem access even inside the current project.

Preferred:

```text
Plugin
  ↓
Document API
  ↓
Document Service
```

Avoid:

```text
Plugin
  ↓
Directly rewrite manuscript file
```

---

# 32. Permission Prompt

During installation, Catalyst should display requested permissions clearly.

Example:

```text
Fantasy Calendar wants permission to:

✓ Read timeline
✓ Modify timeline events
✓ Add a sidebar panel

It cannot:

✗ Read manuscript text
✗ Access the internet
✗ Access files outside this project
```

The writer must be able to make an informed choice.

---

# 33. Permission Changes

If an update requests additional permissions, Catalyst must not silently grant them.

Example:

```text
Version 1.4.2:
timeline.read

Version 2.0.0:
timeline.read
network.access
```

The update requires renewed authorization.

---

# 34. Permission Revocation

Writers must be able to revoke plugin permissions.

Depending upon the permission, revocation may:

- Disable functionality
- Disable the plugin
- Require restart
- Remove access immediately

Catalyst should clearly communicate the result.

---

# 35. Plugin Lifecycle

A plugin may pass through:

```text
Discovered
Installed
Disabled
Enabled
Loaded
Running
Stopped
Updated
Uninstalled
```

Catalyst Core controls the lifecycle.

---

# 36. Installation

Plugins may eventually be installed from:

- Catalyst Community Registry
- Local plugin package
- Developer directory
- First-party distribution

Installation must validate the plugin before activation.

---

# 37. Community Registry

Catalyst should support a public Community Plugin Registry.

The registry contains metadata about approved or listed community plugins.

The registry itself should not require Catalyst to host every plugin's source repository.

A plugin may remain independently developed.

---

# 38. Registry Architecture

Conceptually:

```text
Developer Repository
        │
        ▼
Plugin Release
        │
        ▼
Community Registry Metadata
        │
        ▼
Catalyst Plugin Browser
        │
        ▼
User Installation
```

---

# 39. Registry Metadata

Registry entries may include:

```text
Plugin ID
Name
Description
Developer
Current version
Repository
License
Supported platforms
Required permissions
API compatibility
Release information
Verification status
```

---

# 40. Community Plugin Browser

Catalyst may provide:

```text
Settings
└── Plugins
    ├── Installed
    ├── Community
    ├── Updates
    └── Developer
```

The exact UI may evolve.

---

# 41. Plugin Verification

Catalyst may support verification states such as:

```text
First-party
Verified community
Community
Local/developer
```

Verification must not be represented as an absolute guarantee of safety.

---

# 42. Signing

Plugin packages should eventually support cryptographic signing.

Signing may help verify:

- Publisher identity
- Package integrity
- Update authenticity

The exact signing architecture may be specified separately.

---

# 43. Checksums

Downloaded plugin packages should be integrity checked.

A registry may provide cryptographic hashes for released packages.

Catalyst must reject corrupted packages.

---

# 44. Automatic Updates

Plugins may support automatic updates.

Automatic updates must respect:

- Compatibility
- Permission changes
- Signing
- Writer settings

Updates requesting new privileges require writer approval.

---

# 45. Rollback

Where practical, Catalyst should preserve the previous plugin version long enough to allow rollback after a failed update.

Plugin updates must not be allowed to silently corrupt project data.

---

# 46. Plugin Failure Isolation

A failing plugin should not crash the entire Catalyst application where practical.

Catalyst should isolate plugin failures and report:

```text
Plugin:
Fantasy Calendar

Status:
Disabled after repeated errors
```

---

# 47. Startup Failure

If a plugin prevents normal startup, Catalyst should support Safe Mode.

Example:

```text
Catalyst Safe Mode

Community plugins disabled.
```

This allows the writer to access the project and troubleshoot.

---

# 48. Plugin Logging

Plugins may write diagnostic logs through a controlled logging API.

Logs should identify:

- Plugin ID
- Severity
- Timestamp
- Message

Plugins should not unnecessarily log manuscript content.

---

# 49. Plugin Settings

Plugins may register settings.

Settings should be namespaced by plugin ID.

Example:

```text
com.example.fantasy-calendar
```

Plugin settings must not collide with Catalyst Core settings.

---

# 50. Plugin Data

Plugin data falls into two categories:

```text
Disposable Plugin State

Durable Project Plugin Data
```

These must be treated differently.

---

# 51. Disposable Plugin State

Disposable plugin state may live under:

```text
.lithomark/plugins/<plugin-id>/
```

Examples:

- Cache
- UI state
- Derived indexes
- Temporary files

Deleting this information must not destroy writer-created project information.

---

# 52. Durable Plugin Data

If a plugin creates writer-owned durable project information, that data must use a documented portable storage mechanism.

It must not exist exclusively inside an opaque plugin cache.

Example:

```text
Plugin Data/
└── com.example.fantasy-calendar/
    └── calendar.yaml
```

The final standard location may be defined during implementation.

---

# 53. Plugin Data Ownership

Durable plugin-created content belongs to the writer.

Uninstalling a plugin must not silently delete durable writer-created data.

Catalyst should offer options such as:

```text
Uninstall plugin

Keep project data
Remove plugin cache
```

Destructive removal of durable data requires explicit confirmation.

---

# 54. Unknown Plugin Data

If a project contains durable data belonging to a plugin that is not installed, Catalyst should preserve the data.

It may display:

```text
This project contains data for:

Fantasy Calendar
com.example.fantasy-calendar

Plugin not installed.
```

The data must not be silently discarded.

---

# 55. Plugin Document Types

Plugins may register custom document or entity types.

These must be namespaced.

Example:

```text
com.example.familytree:family-record
```

This prevents collisions with Core types.

---

# 56. Plugin Stable IDs

Plugins requiring stable object identity should request IDs through Catalyst Core.

Plugins must not create IDs that conflict with Core prefixes.

The stable ID rules are defined in:

`003-STABLE-ID-SYSTEM.md`

---

# 57. Plugin Events

Plugins may subscribe to approved domain events.

Examples:

```text
project.opened
project.closed
document.created
document.saved
document.deleted
scene.changed
canon.changed
timeline.changed
selection.changed
```

Subscription does not grant access beyond the plugin's permissions.

---

# 58. Event Payload Security

Event payloads should contain only information permitted for the receiving plugin.

A plugin with no manuscript-read permission must not receive full manuscript text through an event payload.

---

# 59. Plugin Commands

Plugins may register commands.

Example:

```text
Generate Character Relationship Chart
Open Fantasy Calendar
Validate Scene Against Timeline
```

Commands may appear in:

- Command palette
- Menus
- Context menus
- Keyboard shortcuts

depending upon declared UI contributions.

---

# 60. Command Palette

Catalyst should provide a central command registry.

Core and plugin commands should use the same discoverable mechanism where practical.

Plugin commands remain namespaced internally.

---

# 61. Keyboard Shortcuts

Plugins may declare commands capable of receiving user-configurable shortcuts.

Plugins should not permanently seize critical Core shortcuts.

Conflicts must be detectable.

---

# 62. Sidebar Extensions

Plugins may contribute sidebar views.

Example:

```text
Left Sidebar

Manuscript
Story Bible
Timeline
Research
Fantasy Calendar
```

The plugin must not need access to unrelated project data merely to render a panel.

---

# 63. Panel Extensions

Plugins may create workspace panels.

Examples:

- Family tree
- Map
- Calendar
- Statistics dashboard
- Evidence board
- Relationship graph

Panels operate through supported APIs.

---

# 64. Editor Extensions

Editor extensions require special care.

Plugins must not depend directly upon undocumented TipTap internals.

Catalyst should expose stable editor-extension APIs.

Conceptually:

```text
Plugin
   │
   ▼
Catalyst Editor API
   │
   ▼
Editor Adapter
   │
   ▼
TipTap
```

---

# 65. TipTap Isolation

TipTap is not the Catalyst Plugin API.

A plugin must not assume TipTap will always be the editor.

If Catalyst changes editor implementation, compliant plugins should continue functioning wherever their declared API remains supported.

---

# 66. Selection API

A plugin may request access to selected text.

Example:

```text
editor.read-selection
```

This should return a normalized Catalyst representation.

It should not expose unnecessary editor internals.

---

# 67. Document Modification

Plugins with appropriate permission may request changes.

Preferred operations include:

```text
insertText()
replaceSelection()
replaceRange()
updateMetadata()
createAnnotation()
```

Changes pass through the Document Service.

---

# 68. Transaction Model

Plugin changes should be applied as transactions where practical.

Example:

```text
Begin plugin operation
        │
        ▼
Validate permissions
        │
        ▼
Apply changes
        │
        ▼
Validate document
        │
        ▼
Create revision
        │
        ▼
Commit
```

Failure should avoid partially corrupted project state.

---

# 69. Canon Extensions

Plugins may:

- Read canon with permission
- Display canon
- Register custom fact types
- Register deterministic continuity rules
- Propose facts
- Modify facts with stronger permission

Plugins must not silently redefine Core canon semantics.

---

# 70. Continuity Rules

Plugin continuity rules must be deterministic.

A rule receives structured data and returns structured findings.

Conceptually:

```text
Structured Project State
        │
        ▼
Plugin Rule
        │
        ▼
Finding
```

A plugin may use its own internal logic, but Core must clearly identify findings originating from plugins.

---

# 71. Plugin Findings

A continuity finding should include:

```text
rule_id
plugin_id
severity
affected_objects
message
```

This allows writers to identify the source.

---

# 72. Templates

Plugins may contribute templates.

Examples:

```text
Fantasy World Bible
Police Procedural Case File
Romance Beat Sheet
Screenwriting Structure
```

Templates must use standard Catalyst project mechanisms.

---

# 73. Importers

A plugin importer converts an external format into Catalyst domain objects.

Imported objects must:

- Receive stable IDs
- Pass validation
- Use authoritative project storage
- Preserve source information where useful

---

# 74. Exporters

A plugin exporter receives permitted normalized Catalyst data.

It may generate external files.

Export operations should use writer-selected destinations unless explicitly configured otherwise.

---

# 75. Network Plugins

Network-capable plugins require clear disclosure.

Example:

```text
This plugin can send data over the internet.
```

If manuscript access is also requested, Catalyst should make the combination especially clear.

---

# 76. Sensitive Permission Combination

Example:

```text
✓ Read entire manuscript
✓ Access internet
```

This combination creates a potential path for unpublished work to leave the computer.

Catalyst should prominently disclose this.

---

# 77. No Hidden Network Access

Tier 1 plugins without network permission must not receive network access through the supported plugin runtime.

---

# 78. Native Plugin Disclosure

Native plugins should display stronger warnings.

Example:

```text
Native Plugin

This plugin contains native code and may have capabilities beyond standard Catalyst extensions.
```

Installation requires explicit authorization.

---

# 79. Native Plugin Review

Plugins distributed as verified native plugins through an official Catalyst registry should undergo stronger review than standard extensions.

Review may include:

- Source inspection
- Build verification
- Permission review
- Malware scanning
- Dependency review
- Signature verification

The exact review program may evolve.

---

# 80. First-Party Plugins

Catalyst may distribute optional first-party plugins.

Initial candidates include:

```text
Git Integration
GitHub Integration
```

These remain plugins so writers who do not need them are not forced into those workflows.

---

# 81. Git Plugin

A first-party Git plugin may provide:

- Repository initialization
- Status
- Commit
- History
- Branch support
- Diff
- Restore
- Push/pull where configured

Basic Catalyst revision history remains Core.

---

# 82. GitHub Plugin

A GitHub plugin may provide:

- Authentication
- Repository creation
- Push/pull
- Remote status
- Conflict assistance

GitHub must not be required for normal Catalyst operation.

---

# 83. No Built-In AI Requirement

Catalyst's plugin architecture does not imply built-in AI.

Community developers may eventually create integrations with external services where permitted by project policy.

Such plugins must obey:

- Network permissions
- Manuscript permissions
- Writer authorization
- Plugin security requirements

Catalyst Core remains AI-free.

---

# 84. External Agent Separation

The Plugin API and Agent API are separate interfaces.

```text
Plugin
   ↓
Plugin API

External Application / Agent
   ↓
Agent API
```

A plugin is code loaded by Catalyst.

An external agent communicates with Catalyst from outside the application.

Their security models differ.

---

# 85. Developer Mode

Catalyst should provide Developer Mode.

Potential features:

```text
Load unpacked plugin
Reload plugin
Inspect plugin logs
View permissions
Inspect API calls
Open developer documentation
```

Developer Mode must clearly distinguish development plugins from normal installed plugins.

---

# 86. Local Development

Developers should be able to test plugins without publishing them to the Community Registry.

Example:

```text
Load Plugin From Folder
```

Development plugins should display an appropriate warning.

---

# 87. Plugin SDK

Catalyst should eventually provide an official Plugin SDK.

The SDK may include:

- TypeScript types
- API client
- Manifest schema
- Development tooling
- Example plugins
- Testing utilities
- Packaging tools

---

# 88. Documentation

The plugin ecosystem requires stable documentation.

Documentation should include:

- Getting started
- API reference
- Permission reference
- UI extension points
- Document API
- Canon API
- Timeline API
- Template API
- Import/export API
- Security requirements
- Publishing process

---

# 89. Example Plugin

Catalyst should maintain at least one minimal reference plugin.

Example:

```text
Word Count Panel
```

This demonstrates:

- Manifest
- Permissions
- UI registration
- Project API
- Packaging
- Testing

---

# 90. Plugin Testing

The SDK should support automated plugin testing.

Tests may verify:

- Manifest validity
- API compatibility
- Permission behavior
- UI registration
- Document operations
- Error handling

---

# 91. Core API Stability

Public Plugin APIs should be treated as compatibility contracts.

Internal Rust or Svelte implementation may change freely.

Published Plugin API behavior should change deliberately and through versioning.

---

# 92. Deprecated APIs

When possible, APIs should be deprecated before removal.

Example lifecycle:

```text
API v1
    ↓
Deprecated
    ↓
Migration documentation
    ↓
API v2
    ↓
v1 removed in later major compatibility window
```

---

# 93. Internal APIs

Catalyst may contain internal APIs that are not available to plugins.

Community developers must not rely on undocumented internal interfaces.

Internal APIs may change without plugin compatibility guarantees.

---

# 94. Resource Limits

Catalyst may enforce reasonable limits on plugin resource consumption.

Examples:

- CPU
- Memory
- Event frequency
- Storage
- Background execution

A runaway plugin must not make manuscript editing unusable.

---

# 95. Background Tasks

Plugins may request controlled background operations.

Background execution should be observable and cancellable where appropriate.

Plugins should not silently run indefinite tasks without justification.

---

# 96. Plugin Disable

Disabling a plugin should stop its runtime functionality while preserving durable project data.

Example:

```text
Plugin disabled.

Calendar data preserved.
```

---

# 97. Plugin Uninstall

Uninstallation should normally:

1. Disable plugin.
2. Remove executable/plugin package.
3. Remove disposable cache.
4. Preserve durable writer-created project data.
5. Offer optional cleanup.

---

# 98. Plugin Reinstallation

If a plugin is later reinstalled, it should be capable of rediscovering compatible durable plugin data.

---

# 99. Project Portability

A Catalyst project must remain portable even when it uses plugins.

If required plugins are missing, Catalyst should identify them.

Example:

```text
This project uses:

Fantasy Calendar
com.example.fantasy-calendar

Status:
Not installed
```

Core manuscript access must remain available.

---

# 100. Required Plugin Metadata

A project may record which plugins provide project-critical extensions.

Example:

```yaml
plugins:
  required:
    - id: com.example.fantasy-calendar
      minimum_version: 1.4.0
```

A missing required plugin should produce a warning, not make the entire project inaccessible.

---

# 101. Plugin Lock-In Prevention

Catalyst should discourage plugin designs that trap writer-owned content in inaccessible proprietary storage.

Community guidelines should encourage:

- Documented formats
- Export capability
- Human-readable data where practical
- Safe uninstall behavior

---

# 102. Security Incident Response

Catalyst should eventually support disabling or warning about known malicious plugin versions through registry metadata.

This mechanism must be transparent.

It should not silently delete writer data.

---

# 103. Registry Removal

Removing a plugin from the Community Registry must not automatically uninstall it from existing users.

Catalyst may warn users when a plugin has been removed for security reasons.

---

# 104. Offline Operation

Installed plugins that do not require network access should continue functioning offline.

The Community Registry is not required for normal use of already installed offline plugins.

---

# 105. Backup

Durable plugin project data must be included in normal project backup operations.

Disposable plugin caches do not need backup.

---

# 106. Integrity Checking

Project integrity checks should identify:

- Missing required plugins
- Unknown plugin data
- Invalid plugin metadata
- Plugin-defined broken references where supported

Core integrity checking must continue even when plugins are missing.

---

# 107. Accessibility

Plugin UI should follow Catalyst accessibility guidelines.

Community guidelines should cover:

- Keyboard navigation
- Screen readers
- Contrast
- Focus handling
- Scalable text

---

# 108. Theme Compatibility

Plugins should use Catalyst theme tokens and UI APIs rather than hard-coding assumptions about appearance.

This allows compatibility with:

- Light mode
- Dark mode
- Future themes
- Accessibility modes

---

# 109. Design Independence

Plugins must not depend upon Kindling or Press design-system internals.

Catalyst's plugin UI contract is independent.

---

# 110. Migration From Kindling

Kindling does not define Catalyst's community plugin architecture.

During migration:

```text
Kindling/Tauri Foundation
        │
        ▼
Catalyst Domain Layer
        │
        ▼
Catalyst Plugin API
```

The Plugin API should be introduced only after core domain boundaries are sufficiently stable.

---

# 111. Implementation Order

Recommended implementation sequence:

```text
1. Catalyst Domain Layer
2. Document Service
3. Project Service
4. Stable ID Service
5. Event System
6. Permission Model
7. Plugin Runtime
8. Plugin API v1
9. Developer Mode
10. Local Plugin Loading
11. Plugin Packaging
12. Community Registry
13. Signing/Verification
```

The Community Registry should not be built before the local plugin API is stable enough to support real extensions.

---

# 112. Plugin API v1 Scope

The first public Plugin API should be intentionally smaller than the eventual platform.

Recommended initial scope:

```text
Project metadata
Read documents
Controlled document edits
Commands
Sidebar panels
Settings
Events
Story Bible read access
Timeline read access
Basic templates
```

Additional capabilities should be added after the API proves stable.

---

# 113. Security Rule

No plugin should receive a capability merely because granting it is convenient for development.

Every capability must have a clear reason.

---

# 114. Writer Control

The writer must be able to answer:

```text
What plugins are installed?

What can each plugin access?

Which plugins can modify my manuscript?

Which plugins can use the internet?

Which plugins contain native code?

Which plugins are currently enabled?
```

Catalyst should make these answers easy to obtain.

---

# 115. Governing Principle

Catalyst Core provides the stable writing platform.

Plugins expand what the platform can do.

The plugin ecosystem should make Catalyst more capable without making the writer surrender control of:

- Their manuscript
- Their computer
- Their project data
- Their privacy
- Their ability to open the project without a plugin

Extensibility must serve writer ownership rather than undermine it.

---

# 116. Interactive Map Plugin Boundary

Interactive cartography is intentionally deferred from Catalyst Core.

A future official map capability should be implemented as an optional first-party plugin through documented Plugin APIs. It may provide:

- imported or writer-created map canvases;
- location pins linked by stable entity ID;
- regions, routes, labels, and travel paths;
- event/location overlays;
- navigation from map features to Story Development entities;
- optional map-specific project files and assets.

The plugin must consume Core location, entity, relationship, timeline, asset, and project APIs rather than introducing a private authoritative story database.

Disabling or uninstalling the map plugin must not remove the writer's underlying Story Development locations/worldbuilding, manuscript access, or other Core data. Plugin-specific durable data remains writer-owned and portable; rendering caches and spatial indexes must be rebuildable.

---

# 117. Related Specifications

Parent specifications:

- `000-CATALYST-ARCHITECTURE.md`
- `001-PROJECT-FILE-FORMAT.md`
- `002-DOCUMENT-MODEL.md`
- `003-STABLE-ID-SYSTEM.md`
- `004-CANON-MODEL.md`

Related specifications:

- `006-AGENT-API.md`
- `007-KINDLING-MIGRATION.md`

Future supporting specifications should include:

- Plugin API Reference
- Plugin Manifest Schema
- Plugin Permission Registry
- Plugin Packaging Format
- Community Registry Specification
- Plugin Security Model
- Plugin Signing Specification
- Plugin Developer Guide

## First-Party Plugin Model

Catalyst uses a **Core + First-Party Plugins + Community Plugins** architecture.

Catalyst Core contains the capabilities required for project integrity and for a project to remain openable and recoverable:

- project-root and project-format services
- authoritative document model and persistence
- stable identity
- safe save, conflict detection, recovery, and integrity primitives
- shared application context and domain events
- plugin host, manifest validation, permissions, lifecycle, and API contracts
- minimum Write and Story Development capabilities required to understand and edit a project

Optional product capabilities should be implemented as first-party plugins where doing so does not compromise those guarantees.

Candidate first-party plugins include:

- Developmental Review
- Beta Reader / ARC workflows
- Listen / TTS
- Audiobook Studio
- Publishing Studio
- Git / GitHub integration
- Graph View
- advanced or specialized Story Development views
- optional importers/exporters and integrations

A first-party plugin is an official Catalyst component, not privileged project truth. It may be bundled with Catalyst and enabled by default when appropriate, but optional first-party plugins should be individually enableable/disableable.

### Public API parity rule

First-party plugins must use the documented Catalyst Plugin API wherever reasonably possible.

First-party code must not depend on secret database access, undocumented TipTap internals, or private persistence shortcuts merely to obtain capabilities denied to community plugins.

A capability that an official plugin legitimately needs should normally become a documented, permission-controlled Plugin API capability. Exceptions are limited to security-sensitive bootstrap/host functions or platform operations that cannot safely be exposed.

This makes first-party plugins the reference implementation and continuous test of the community API.

### Plugin disablement guarantee

Disabling or uninstalling an optional plugin must not make the core manuscript inaccessible.

Durable writer-owned plugin data is preserved unless the writer explicitly requests deletion. Unsupported plugin data remains intact and identifiable.

A project may report that an optional capability is unavailable, but the absence of that plugin must not corrupt or erase authoritative core documents.

### Plugin boundary and Document Service

Plugins never write authoritative manuscript or Story Development files directly.

Authorized document changes follow:

```text
Plugin
  ↓
Permission Check
  ↓
Plugin / Domain API
  ↓
Document Service
  ↓
Validation + Conflict Detection + Safe Save
  ↓
Authoritative Markdown/YAML
```

The Document Service therefore forms part of the stable domain boundary now being built. Plugin APIs should expose document operations in terms of stable Catalyst documents and IDs rather than filesystem implementation details.

### Capability granularity

Plugin permissions should be narrow enough that a plugin can perform its purpose without receiving unrelated manuscript access. Reading an unpublished manuscript, modifying prose, accessing the network, writing outside the project, executing processes, and handling credentials are distinct capabilities and must not be implied by UI access.

The initial API design should favor capability-oriented operations such as:

- project/document discovery
- document read
- document create/update through Document Service
- metadata read/update
- Story Development entity operations
- selection/editor operations
- domain-event subscriptions
- commands and UI contributions
- import/export registration
- network access where explicitly authorized

### First-party packaging

Bundling a first-party plugin with the installer does not make it Core. Bundled plugins retain manifests, versions, permissions/capabilities, lifecycle state, and clear ownership.

Catalyst may grant bundled first-party plugins a trusted publisher status, but trust must not bypass project-integrity rules.

### Community compatibility objective

The plugin API must be versioned independently of the application and designed so a compliant community plugin does not need to know whether Catalyst currently uses Svelte, TipTap, SQLite indexes, or another internal implementation.

The public contract is the Catalyst domain/plugin API. Internal implementation remains replaceable.

---

---

**END OF PLUGIN ARCHITECTURE SPECIFICATION**

---

## Theme Extension Architecture

Catalyst themes are a formal extension class, but they are not executable plugins.

A theme changes presentation only. Installing a theme must never grant manuscript, project, filesystem, network, process, editor-write, or Story Development permissions.

### Theme Package

```text
my-lithomark-theme/
├── lithomark-theme.json
├── theme.css
├── README.md
└── LICENSE
```

The manifest identifies the theme, author, version, compatibility, supported appearance mode, and stylesheet entry. Theme packages must use safe package-relative resources.

### Stable Design Tokens

Catalyst's application shell and first-party workspaces must consume public design tokens rather than hard-coded component-specific colors wherever practical. Public tokens use a stable `--lm-*` namespace.

Initial token families cover backgrounds, editor/manuscript surfaces, text, accents/focus, borders, links/wikilinks, selection, controls, typography, Story Development cards/timeline/graph semantics, Review annotations, Git/status semantics, and audio/publishing status surfaces.

Themes target documented tokens and supported theme surfaces, not arbitrary internal Svelte classes or DOM structure.

### Built-in Theme Rule

Catalyst Dark and Catalyst Light use the same public token/theme mechanism expected of community themes wherever practical. Built-in themes are reference implementations of the public contract.

### Safety Boundary

Theme CSS is presentation data, not executable plugin code. Themes cannot call the Plugin API, read/write project documents, execute commands/processes, access the network, receive manuscript content through a theme API, or bypass the host theme/token boundary. Executable behavior belongs in a plugin.

### Appearance Settings

Appearance should expose active theme, installed/community themes, accent color where supported, editor font, UI font, font size, and interface scale. User overrides layer predictably over theme defaults without modifying installed packages.

### Theme Creator

A future built-in Theme Creator should allow non-programmers to start from a theme, edit supported tokens visually, preview representative Catalyst surfaces live, name/describe the theme, receive contrast/accessibility guidance, and export a valid Catalyst theme package. Advanced authors may edit the stylesheet directly.

### Community Themes

Community themes may use the same discovery/update infrastructure as community plugins while remaining a distinct, lower-risk package type. The registry and UI must clearly distinguish executable plugins from non-executable themes.

### Compatibility

Theme compatibility is independent of manuscript/project format compatibility. Internal Svelte component or DOM structure is not a supported theme API; documented design tokens are the compatibility boundary.
