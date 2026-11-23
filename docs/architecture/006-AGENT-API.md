# LithoMark Writer OS — External Agent API

**Document:** 006-AGENT-API.md  
**Status:** Locked Baseline  
**Project:** LithoMark Writer OS  
**Parent Specification:** 000-LITHOMARK-ARCHITECTURE.md  
**Related Specifications:** 001-PROJECT-FILE-FORMAT.md, 002-DOCUMENT-MODEL.md, 003-STABLE-ID-SYSTEM.md, 004-CANON-MODEL.md, 005-PLUGIN-ARCHITECTURE.md

---

# 1. Purpose

This specification defines the LithoMark External Agent API.

The Agent API allows explicitly authorized external applications, automation tools, scripts, developer tools, and agentic systems to interact with LithoMark through a controlled interface.

The Agent API does not make LithoMark an AI application.

LithoMark Core contains no requirement for generative artificial intelligence.

The API is a general-purpose external integration boundary.

---

# 2. Fundamental Principle

External software must not require unrestricted access to LithoMark's internal implementation.

Preferred architecture:

```text
External Application
        │
        ▼
LithoMark Agent API
        │
        ▼
Authentication
        │
        ▼
Permission Layer
        │
        ▼
LithoMark Domain API
        │
        ▼
Project / Document Services
        │
        ▼
Authoritative Project Files
```

External tools interact with LithoMark through documented operations.

---

# 3. No Built-In AI

LithoMark does not provide built-in generative AI.

The Agent API does not include:

- Embedded language models
- Required cloud AI services
- Automatic manuscript generation
- Automatic rewriting
- Mandatory AI accounts
- Hidden manuscript transmission

An external AI system may use the API only if the writer deliberately connects and authorizes it.

---

# 4. General-Purpose Integration

The Agent API may be used by non-AI software.

Examples include:

- Command-line tools
- Automation scripts
- Backup tools
- Publishing workflows
- Research utilities
- External editors
- Developer tools
- Accessibility tools
- Workflow automation
- External agents

Therefore the API must remain provider-neutral.

---

# 5. Disabled by Default

External Agent API access must be disabled by default.

A new LithoMark installation must not automatically expose project data to external applications.

The writer must explicitly enable external access.

---

# 6. Writer Authorization

External access requires writer authorization.

The writer should be able to determine:

```text
Which application is connected?
Which project can it access?
What information can it read?
What information can it modify?
Can it propose changes?
Can it delete anything?
When was it last active?
```

---

# 7. Least Privilege

External applications receive only the permissions necessary for their intended task.

For example, an application that only needs to inspect the Story Bible should not automatically receive:

```text
manuscript.write
canon.delete
project.delete
```

---

# 8. Connection Scope

Authorization may be scoped to:

```text
Application
Project
Permission Set
Session
Duration
```

A connection authorized for one project must not automatically gain access to every LithoMark project.

---

# 9. Connection Modes

LithoMark should eventually support two broad authorization modes:

```text
Session Authorization
Persistent Authorization
```

Session authorization expires when the session ends.

Persistent authorization remains until revoked or expired according to writer settings.

---

# 10. Local-First Transport

The initial Agent API should prioritize local communication.

Possible transport mechanisms include:

- Localhost HTTP
- Local WebSocket
- Local IPC
- Named pipes
- Unix domain sockets

The final transport should be selected during implementation.

Remote network exposure is not required for Agent API v1.

---

# 11. No Public Network Listener by Default

LithoMark must not expose the Agent API directly to the public internet by default.

The initial architecture should assume:

```text
External Tool
     │
     ▼
Local Machine
     │
     ▼
LithoMark
```

Remote access, if ever implemented, requires a separate security design.

---

# 12. Authentication

Every external connection must authenticate.

Possible mechanisms include:

- Short-lived access tokens
- Application tokens
- Challenge-response
- Operating-system mediated authentication

The exact implementation may evolve.

Unauthenticated project access is prohibited.

---

# 13. Pairing

LithoMark should support an explicit pairing workflow.

Conceptually:

```text
External Application
        │
        ▼
Requests Connection
        │
        ▼
LithoMark Displays Request
        │
        ▼
Writer Reviews Identity
        │
        ▼
Writer Reviews Permissions
        │
        ▼
Approve / Deny
```

---

# 14. Pairing Information

The authorization screen should display information such as:

```text
Application:
Example Editorial Tool

Requested Project:
All That Remains

Requested Permissions:

✓ Read manuscript
✓ Read Story Bible
✓ Read timeline
✓ Create notes
✓ Propose manuscript edits

✗ Direct manuscript modification
✗ Modify canon
✗ Delete files
```

---

# 15. Agent Identity

Each connected external application receives an identity.

Conceptually:

```text
agent_id
display_name
application_identifier
authorization_id
permissions
created
last_used
```

This identity supports permissions and auditing.

---

# 16. API Versioning

The Agent API is versioned independently from the LithoMark application.

Example:

```text
LithoMark:
3.2.0

Agent API:
1
```

External applications must negotiate or declare supported API versions.

---

# 17. Capability Discovery

External clients should be able to discover available capabilities after authentication.

Example conceptual response:

```json
{
  "apiVersion": 1,
  "capabilities": [
    "project.read",
    "document.read",
    "story-bible.read",
    "timeline.read",
    "proposal.create"
  ]
}
```

The response must reflect granted permissions rather than every possible LithoMark capability.

---

# 18. Permission Categories

Initial Agent API permission categories may include:

```text
project
document
manuscript
story-bible
timeline
canon
continuity
story
research
notes
templates
proposals
revisions
export
events
```

---

# 19. Project Permissions

Examples:

```text
project.read
project.structure.read
project.metadata.read
project.metadata.write
```

Reading project metadata does not automatically grant manuscript access.

---

# 20. Document Permissions

Examples:

```text
document.read
document.create
document.write
document.delete
```

These may be further restricted by document type.

---

# 21. Manuscript Permissions

Manuscript permissions must be granular.

Examples:

```text
manuscript.read
manuscript.create
manuscript.write
manuscript.delete
manuscript.propose
```

`manuscript.propose` is deliberately separate from `manuscript.write`.

---

# 22. Story Bible Permissions

Examples:

```text
story-bible.read
story-bible.create
story-bible.write
story-bible.delete
story-bible.propose
```

---

# 23. Timeline Permissions

Examples:

```text
timeline.read
timeline.create
timeline.write
timeline.delete
timeline.propose
```

---

# 24. Canon Permissions

Canon requires particularly explicit permissions.

Examples:

```text
canon.read
canon.propose
canon.create
canon.write
canon.retcon
canon.delete
```

Reading canon does not permit changing canon.

---

# 25. Continuity Permissions

Examples:

```text
continuity.read
continuity.run
```

An external tool may request LithoMark's deterministic continuity results without modifying the project.

---

# 26. Story Development Permissions

Examples:

```text
story.read
story.create
story.write
story.delete
story.propose
```

These may apply to:

- Plot threads
- Character arcs
- Reveals
- Clues
- Foreshadowing
- Payoffs
- Beat sheets

---

# 27. Research Permissions

Examples:

```text
research.read
research.create
research.write
research.delete
```

Binary asset access may require additional explicit capabilities.

---

# 28. Notes Permissions

Examples:

```text
notes.read
notes.create
notes.write
notes.delete
```

A useful low-risk integration may receive permission to create notes without modifying manuscript prose.

---

# 29. Revision Permissions

Examples:

```text
revisions.read
snapshots.create
```

External systems should not receive permission to destroy revision history.

---

# 30. Proposal Model

Proposals are a first-class Agent API concept.

A proposal allows an external system to suggest a change without immediately modifying authoritative project content.

Conceptually:

```text
External Tool
      │
      ▼
Create Proposal
      │
      ▼
LithoMark Stores Proposal
      │
      ▼
Writer Reviews
   │         │
Accept     Reject
   │
   ▼
Normal LithoMark Change Pipeline
```

---

# 31. Proposal Types

Initial proposal types may include:

```text
manuscript-edit
metadata-change
story-bible-change
timeline-change
canon-fact
canon-retcon
plot-change
note
relationship-change
```

---

# 32. Manuscript Edit Proposal

Example conceptual request:

```json
{
  "type": "manuscript-edit",
  "documentId": "scn-df925d68-d3ef-4d90-bfa4-6fa7834203c7",
  "operation": "replace",
  "anchor": {
    "selectedText": "The smell reached her first."
  },
  "replacement": "The smell hit her first."
}
```

LithoMark displays the proposed change to the writer.

---

# 33. Proposal Anchoring

Text proposals must use resilient anchors.

They must not depend exclusively upon absolute character offsets.

Anchoring should follow the principles defined in:

`002-DOCUMENT-MODEL.md`

---

# 34. Canon Proposal

An external application may propose:

```json
{
  "type": "canon-fact",
  "subject": "chr-leah-bennett",
  "factType": "injury",
  "value": {
    "type": "gunshot-wound",
    "bodyLocation": "shoulder"
  },
  "source": {
    "documentId": "scn-example"
  }
}
```

This does not become confirmed canon merely because an external application proposed it.

---

# 35. Proposal Status

Proposals may have status:

```text
pending
accepted
rejected
expired
superseded
```

---

# 36. Proposal Identity

Every proposal should have a stable identifier.

Example:

```text
proposal-UUID
```

Proposal ID format may be formally added to the Stable ID registry before implementation.

---

# 37. Direct Write Access

Some external applications may legitimately require direct write permission.

Example:

```text
document.write
```

Direct writes still pass through LithoMark's normal services.

They do not mean unrestricted filesystem access.

---

# 38. Direct Write Pipeline

Required architecture:

```text
External Application
        │
        ▼
Authenticated API Request
        │
        ▼
Permission Check
        │
        ▼
Domain Validation
        │
        ▼
Document Service
        │
        ▼
Revision Creation
        │
        ▼
Safe Save
        │
        ▼
Index Update
```

---

# 39. No Direct SQLite Access

External applications must never depend upon LithoMark's SQLite schema.

Incorrect:

```text
Agent
  ↓
index.db
```

Required:

```text
Agent
  ↓
Agent API
  ↓
Domain API
```

SQLite remains an internal rebuildable implementation detail.

---

# 40. No Direct Internal File Mutation

The Agent API must not expose unrestricted project-file mutation merely for convenience.

External applications should operate through domain operations.

This protects:

- Stable IDs
- Validation
- Revisions
- Conflict detection
- Canon integrity
- Event generation

---

# 41. Stable IDs

Agent API operations use stable IDs.

Example:

```json
{
  "characterId": "chr-550e8400-e29b-41d4-a716-446655440000"
}
```

Names may be used for search but not as authoritative identity.

---

# 42. Search

The Agent API may expose search operations.

Examples:

```text
searchDocuments()
searchCharacters()
searchLocations()
searchTimeline()
searchCanon()
```

Search results return stable IDs.

---

# 43. Object Resolution

An external tool may search:

```text
Leah Bennett
```

and receive:

```text
Name:
Leah Bennett

Type:
Character

ID:
chr-550e8400-e29b-41d4-a716-446655440000
```

If several results match, the external client must disambiguate.

---

# 44. Project Structure

With permission, an external application may inspect project structure.

Example conceptual response:

```text
Project
├── Manuscript
│   ├── Prologue
│   ├── Chapter 1
│   └── Chapter 2
├── Story Bible
├── Timeline
└── Story
```

The API should return structured objects rather than requiring filesystem path interpretation.

---

# 45. Manuscript Reading

With `manuscript.read`, an external application may retrieve manuscript content.

Requests should support limited scope.

Examples:

```text
Entire manuscript
Chapter
Scene
Selection
Range of scenes
```

Applications should not be forced to retrieve the entire manuscript when only one scene is needed.

---

# 46. Story Bible Reading

With permission, external applications may retrieve:

- Characters
- Locations
- Organizations
- Objects
- Relationships
- Procedures
- Custom Story Bible objects

---

# 47. Canon Reading

Canon queries should support:

```text
Facts for object
Facts by type
Facts active at date
Fact history
Character knowledge at date
Relationship state at date
Object possession at date
```

This allows external tools to work with structured story state.

---

# 48. Timeline Reading

Timeline queries may support:

```text
Events by date
Events by character
Events by location
Events within range
Event details
```

---

# 49. Scene Context

A particularly useful API operation may retrieve scene context.

Conceptually:

```text
getSceneContext(sceneId)
```

Possible response:

```text
Scene
Date
POV
Characters
Location
Applicable canon
Character knowledge
Active relationships
Plot threads
Reveals
Continuity findings
```

This is derived from structured LithoMark data.

---

# 50. Context Is Permission-Aware

Scene context must not bypass permissions.

An external application without canon permission must not receive hidden canon merely because it requested scene context.

---

# 51. Spoiler Control

External access may eventually support story-position or spoiler scopes.

Example:

```text
Only provide information known through Chapter 7.
```

This is useful when an external tool should not receive future project information.

It is not required for Agent API v1 but should remain architecturally possible.

---

# 52. Event Subscription

Authorized clients may subscribe to LithoMark domain events.

Examples:

```text
project.opened
document.saved
document.created
document.deleted
canon.changed
timeline.changed
proposal.accepted
proposal.rejected
```

---

# 53. Event Permissions

An application only receives event data it has permission to access.

A client without manuscript permission may receive:

```text
document.changed
```

without receiving manuscript text.

---

# 54. Event Transport

If the initial API uses HTTP, event delivery may use:

- WebSocket
- Server-sent events
- Local event channel

The implementation must remain local-first.

---

# 55. Audit Log

Agent API activity should be auditable.

Significant operations may record:

```text
Timestamp
Agent identity
Project
Operation
Affected object
Result
```

---

# 56. Content Logging

Audit logs should avoid unnecessarily duplicating manuscript content.

Example:

Preferred:

```text
Agent X proposed change to scn-123.
```

Avoid logging the entire scene unless required.

---

# 57. Change Attribution

Changes created through the Agent API should be attributable.

Example:

```text
Source:
External Agent

Agent:
Example Editorial Tool
```

This may appear in revision history.

---

# 58. Revocation

The writer must be able to revoke external authorization.

Revocation should invalidate future access.

Example:

```text
Settings
└── External Connections
    └── Example Editorial Tool
        └── Revoke Access
```

---

# 59. Emergency Disable

LithoMark should provide a simple control to disable all Agent API access.

Example:

```text
External Connections: OFF
```

This should stop new external requests.

---

# 60. Token Storage

Authentication credentials must be stored securely.

Plain-text permanent access tokens should not be unnecessarily exposed in project files.

Machine-specific authentication data belongs outside portable authoritative project content.

---

# 61. Project Files Must Not Contain Secrets

`lithomark.yaml` and other portable project files must not contain permanent Agent API credentials.

This prevents credentials from being accidentally:

- Committed to Git
- Shared in ZIP archives
- Synchronized
- Published

---

# 62. Rate Limiting

LithoMark may enforce reasonable request limits.

This protects the application from:

- Runaway scripts
- Accidental request loops
- Resource exhaustion

Local applications should still have sufficient throughput for legitimate workflows.

---

# 63. Request Cancellation

Long-running Agent API operations should be cancellable where practical.

Examples:

- Project-wide search
- Export
- Continuity analysis
- Bulk proposal creation

---

# 64. Transaction Support

Bulk modifications should support transaction-like behavior where practical.

Conceptually:

```text
Begin Transaction
      │
      ▼
Validate Operations
      │
      ▼
Apply Changes
      │
      ▼
Validate Project
      │
      ├── Failure → Roll Back
      │
      └── Success → Commit
```

---

# 65. Bulk Operations

Potential bulk operations include:

```text
Create multiple notes
Update metadata
Create timeline events
Create proposals
Import structured Story Bible information
```

Bulk access must still obey permissions.

---

# 66. Destructive Operations

Destructive operations require stronger permission.

Examples:

```text
document.delete
canon.delete
story-bible.delete
timeline.delete
```

These permissions should not be granted casually.

---

# 67. Protected Operations

Some operations may require interactive confirmation even when an external application has broad write access.

Examples:

- Delete large portions of manuscript
- Delete project
- Bulk canon retcon
- Destructive project migration
- Clear revision history

Agent API v1 should favor safety over maximum automation.

---

# 68. Project Deletion

The Agent API should not initially expose whole-project deletion.

Project deletion can remain a writer-controlled LithoMark UI operation.

---

# 69. Revision Integration

Agent-created changes should participate in normal revision history.

External writes must not bypass:

- Revision creation
- Snapshots
- Safe saves
- Conflict checks

---

# 70. Conflict Detection

If an external application attempts to update a document that changed since it was read, LithoMark should detect the conflict.

The API may use:

```text
revision ID
content hash
version token
```

Example:

```json
{
  "documentId": "scn-example",
  "expectedRevision": "rev-123"
}
```

---

# 71. Optimistic Concurrency

Agent API writes should generally use optimistic concurrency.

Conceptually:

```text
Read revision 12
      │
      ▼
Prepare change
      │
      ▼
Submit expecting revision 12
      │
      ▼
Current revision?
   │          │
  12         13
   │          │
Apply       Conflict
```

This prevents silent overwrites.

---

# 72. Conflict Response

A conflict response should provide enough information for the client to decide whether to:

- Re-read
- Recalculate
- Submit proposal
- Ask writer
- Abort

LithoMark must not silently overwrite newer work.

---

# 73. Error Model

The Agent API should use structured errors.

Examples:

```text
AUTHENTICATION_REQUIRED
PERMISSION_DENIED
PROJECT_NOT_FOUND
OBJECT_NOT_FOUND
REVISION_CONFLICT
VALIDATION_FAILED
UNSUPPORTED_API_VERSION
RATE_LIMITED
PROJECT_READ_ONLY
```

---

# 74. Read-Only Mode

LithoMark may expose a global or per-project Agent API read-only mode.

In read-only mode:

```text
Read operations → allowed according to permission
Write operations → denied
Proposals → configurable
```

---

# 75. Proposal-Only Mode

A useful security mode is:

```text
Read + Propose
No Direct Write
```

This is particularly appropriate for external editorial or agentic systems.

---

# 76. Recommended Default External Integration Profile

For external editorial tools, the recommended starting permission profile is:

```text
project.read
manuscript.read
story-bible.read
timeline.read
canon.read
continuity.read
notes.create
manuscript.propose
canon.propose
```

Direct modification remains disabled.

---

# 77. Trust Profiles

LithoMark may eventually provide understandable permission presets.

Examples:

```text
Read Only
Editorial Assistant
Project Automation
Full Project Access
Custom
```

The exact permission list must always remain inspectable.

---

# 78. No Provider Preference

The Agent API must not be designed specifically for:

- OpenAI
- Anthropic
- Google
- Microsoft
- Local LLM systems
- Any single automation provider

It is a LithoMark API.

Any compatible external system may use it if the writer authorizes access.

---

# 79. No Mandatory Account

Using the Agent API locally must not require a LithoMark cloud account.

LithoMark's local-first principle remains intact.

---

# 80. No Mandatory Internet

Local Agent API use must not require internet access.

A local automation or local agent should be capable of interacting with LithoMark entirely offline.

---

# 81. External Service Responsibility

If an authorized external application sends information to a cloud service, that behavior belongs to the external application's security and privacy model.

LithoMark should clearly distinguish:

```text
LithoMark granted this application access.
```

from:

```text
LithoMark itself transmitted the manuscript.
```

---

# 82. Connection Visibility

When external access is active, LithoMark should provide visible status.

Example:

```text
External Connections: 1 active
```

The indicator should not be unnecessarily intrusive.

---

# 83. Active Operation Visibility

For significant write operations, LithoMark may display:

```text
Example Editorial Tool is creating 12 proposals.
```

This improves transparency.

---

# 84. External Connection Manager

LithoMark should eventually provide:

```text
Settings
└── External Connections
    ├── Enabled / Disabled
    ├── Connected Applications
    ├── Permissions
    ├── Sessions
    ├── Activity
    └── Revoke
```

---

# 85. Developer Tools

Developer Mode may expose:

- API endpoint information
- API version
- Connection logs
- Permission inspection
- Request testing
- Schema documentation

Secrets must not be casually displayed.

---

# 86. API Documentation

The Agent API requires machine-readable and human-readable documentation.

Potential formats include:

- OpenAPI
- JSON Schema
- Generated reference documentation

The final format depends upon the selected transport.

---

# 87. SDKs

LithoMark may eventually provide lightweight SDKs.

Potential SDKs:

```text
TypeScript
Python
Rust
```

SDKs are conveniences.

The underlying protocol should remain independently implementable.

---

# 88. Command-Line Integration

A future LithoMark CLI may use the same domain contracts as the Agent API.

Example:

```text
lithomark project inspect
lithomark scene list
lithomark continuity check
```

The CLI must not require a different project data model.

---

# 89. External Editor Integration

The Agent API may eventually support editor integrations.

Example:

```text
External editor
      │
      ▼
Open scene
      │
      ▼
Edit
      │
      ▼
Save through LithoMark
```

This complements direct Markdown file editing rather than replacing it.

---

# 90. Direct File Editing Remains Valid

The existence of the Agent API does not prohibit writers from opening Markdown files directly.

LithoMark remains file-based.

The Agent API exists for structured external integration.

---

# 91. File Watcher Interaction

If an external tool modifies project files directly instead of using the Agent API, LithoMark's normal file-watcher and conflict-detection mechanisms still apply.

Such changes are not treated as authenticated Agent API operations.

---

# 92. Plugin Versus Agent

The distinction is:

```text
Plugin:
Runs as an extension of LithoMark.

Agent:
Runs outside LithoMark and communicates through an external API.
```

The two systems may expose similar domain concepts but have separate security boundaries.

---

# 93. Agent API Must Use Domain Layer

Required:

```text
Agent API
    │
    ▼
Domain Services
```

Prohibited architecture:

```text
Agent API
    │
    ├── Direct SQLite mutation
    ├── Direct TipTap mutation
    └── Unvalidated file overwrite
```

---

# 94. TipTap Independence

External integrations must not depend upon TipTap JSON.

The API operates on LithoMark document concepts.

This allows the editor implementation to change without breaking external integrations.

---

# 95. SQLite Independence

External integrations must not depend upon the SQLite schema.

This allows indexing technology to change without breaking the Agent API.

---

# 96. Svelte Independence

External integrations must not depend upon Svelte components or internal UI state.

The Agent API belongs to the LithoMark domain layer.

---

# 97. Future Mobile Support

A future mobile LithoMark implementation may expose a subset of the same Agent API concepts.

The domain contract should therefore avoid unnecessary desktop-only assumptions.

---

# 98. Security Testing

The Agent API requires automated security testing.

Tests should include:

- Unauthorized access
- Invalid tokens
- Expired tokens
- Permission escalation attempts
- Cross-project access attempts
- Invalid object IDs
- Direct-write restrictions
- Revision conflicts
- Destructive operations
- Rate limiting
- Revocation
- Malformed requests

---

# 99. Functional Testing

Tests should also include:

- Project queries
- Manuscript reads
- Story Bible reads
- Timeline queries
- Canon queries
- Proposal creation
- Proposal acceptance
- Direct writes
- Event subscription
- Stable ID resolution
- Conflict detection

---

# 100. Audit Testing

Tests must confirm that externally originated changes are correctly attributed and recorded.

---

# 101. Privacy Principle

Access to unpublished writing is sensitive.

The Agent API must therefore assume manuscript content is private unless the writer explicitly grants access.

Convenience does not override this principle.

---

# 102. Failure Behavior

If the Agent API subsystem fails, normal LithoMark writing must continue.

The writer must still be able to:

- Open project
- Edit manuscript
- Save
- Use Story Bible
- Use timeline
- Use canon
- Export

External integration is optional infrastructure.

---

# 103. Backward Compatibility

Agent API changes must be versioned deliberately.

An application update must not unnecessarily break compatible external tools.

---

# 104. Deprecation

Agent API capabilities should be deprecated before removal where practical.

Migration documentation should accompany breaking changes.

---

# 105. Initial Agent API Scope

Agent API v1 should remain deliberately focused.

Recommended initial capabilities:

```text
Authentication
Project listing
Project metadata
Project structure
Document reading
Story Bible reading
Timeline reading
Canon reading
Continuity results
Note creation
Proposal creation
Revision-aware writes for explicitly authorized clients
```

More powerful capabilities can be added later.

---

# 106. Implementation Order

Recommended implementation sequence:

```text
1. LithoMark Domain Layer
2. Project Service
3. Document Service
4. Stable ID Service
5. Canon Service
6. Timeline Service
7. Revision Service
8. Permission Framework
9. Local Authentication
10. Agent API Read Operations
11. Proposal System
12. Revision-Aware Writes
13. Event Subscriptions
14. SDK / Documentation
```

The Agent API should not be implemented before the underlying domain services are stable.

---

# 107. Security Priority

When usability and external automation conflict with project safety, the default behavior should favor preserving the writer's work.

A writer may deliberately grant broader permissions.

LithoMark should not grant them implicitly.

---

# 108. Governing Principle

LithoMark remains a local-first, AI-free Writer OS.

External tools may assist the writer when the writer chooses.

The writer decides:

- Whether external access exists
- Which application receives access
- Which project it can access
- What it can read
- What it can propose
- What it can change
- When that access ends

The external tool serves the writer.

The writer remains in control of the project.

---

# 109. Related Specifications

Parent specifications:

- `000-LITHOMARK-ARCHITECTURE.md`
- `001-PROJECT-FILE-FORMAT.md`
- `002-DOCUMENT-MODEL.md`
- `003-STABLE-ID-SYSTEM.md`
- `004-CANON-MODEL.md`
- `005-PLUGIN-ARCHITECTURE.md`

Related specification:

- `007-KINDLING-MIGRATION.md`

Future supporting specifications should include:

- Agent API Protocol
- Agent API Authentication
- Agent API Permission Registry
- Proposal Model
- Revision and Snapshot Model
- External Connection Security Model
- Agent API OpenAPI/Schema Definition

---

**END OF EXTERNAL AGENT API SPECIFICATION**