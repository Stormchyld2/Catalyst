# LithoMark Writer OS — Canon Model

**Document:** 004-CANON-MODEL.md  
**Status:** Locked Baseline  
**Project:** LithoMark Writer OS  
**Parent Specification:** 000-LITHOMARK-ARCHITECTURE.md  
**Related Specifications:** 001-PROJECT-FILE-FORMAT.md, 002-DOCUMENT-MODEL.md, 003-STABLE-ID-SYSTEM.md

---

# 1. Purpose

This specification defines the LithoMark Canon Model.

The Canon Model records explicit facts established by the writer about the story world.

It provides structured information that can be used for:

- Continuity checking
- Timeline validation
- Character tracking
- Relationship tracking
- Knowledge tracking
- Location tracking
- Injury and status tracking
- Object possession
- Story Bible integration
- Project integrity checks

LithoMark does not determine canon through generative artificial intelligence.

The writer establishes canon.

---

# 2. Fundamental Definition

In LithoMark:

**Canon is an explicit recorded fact about the project world.**

Examples:

```text
Leah Bennett is a detective.
Ray Carter died on November 3, 2023.
The shooting occurred in the Irish Channel.
Harrison Grant is an alias used by Caleb Warren.
Leah was wounded during the shooting.
A character learned a particular fact in Chapter 12.
A weapon changed possession after an event.
```

Canon records what the writer has deliberately established.

---

# 3. Canon Is Not Prose Inference

LithoMark Core must not automatically decide that a sentence in manuscript prose establishes canon.

Example prose:

```text
Leah thought Harrison looked guilty.
```

LithoMark must not automatically create:

```text
Harrison Grant is guilty.
```

Likewise:

```text
Marcus claimed he was home all evening.
```

does not establish:

```text
Marcus was home all evening.
```

Statements made by characters may be:

- True
- False
- Mistaken
- Incomplete
- Manipulative
- Speculative

Canon therefore requires explicit structured recording.

---

# 4. Writer Authority

The writer is the ultimate authority over canon.

LithoMark may:

- Store facts
- Compare facts
- Detect contradictions
- Identify missing references
- Identify impossible chronology
- Warn about possible continuity problems

LithoMark must not silently rewrite canon to resolve a contradiction.

The writer decides what is correct.

---

# 5. Canon Fact Identity

Every canon fact has a stable ID.

Example:

```yaml
lithomark:
  type: canon-fact
  id: fact-b1ca8a4a-2812-45ac-a017-e605e774bec7
```

Canon fact IDs follow:

`003-STABLE-ID-SYSTEM.md`

---

# 6. Basic Canon Fact Structure

A canon fact conceptually contains:

```text
CanonFact
│
├── id
├── subject
├── predicate/type
├── value
├── temporal scope
├── certainty/status
├── source
├── notes
└── metadata
```

Example:

```yaml
lithomark:
  type: canon-fact
  id: fact-b1ca8a4a-2812-45ac-a017-e605e774bec7

subject: chr-ray-carter
fact_type: death

value:
  date: 2023-11-03
  location: loc-irish-channel

status: confirmed
```

---

# 7. Subject

A canon fact normally applies to a stable LithoMark object.

Examples:

```text
Character
Location
Organization
Object
Event
Relationship
```

Example:

```yaml
subject: chr-leah-bennett
```

The subject is identified by stable ID rather than display name.

---

# 8. Fact Types

Initial core fact categories should include:

```text
identity
alias
birth
death
age
occupation
role
location
residence
relationship
status
physical-attribute
injury
medical-status
possession
ownership
knowledge
event-participation
organization-membership
rank
title
ability
restriction
custom
```

The Canon Model must remain extensible.

---

# 9. Identity Facts

Identity facts describe what an entity fundamentally is.

Example:

```yaml
subject: chr-caleb-warren
fact_type: identity

value:
  entity_type: person
```

Identity facts may also participate in identity reveals or alias structures.

---

# 10. Aliases

Aliases must not automatically create separate characters.

Example:

```yaml
subject: chr-caleb-warren
fact_type: alias

value:
  name: Harrison Grant

valid_from: 2004
```

The alias refers to the same stable character object.

This allows LithoMark to understand that:

```text
Caleb Warren
Harrison Grant
```

may represent the same canonical person.

---

# 11. Hidden Identity

The story may conceal an identity from characters or readers while LithoMark still records the objective canon.

Example:

```yaml
subject: chr-caleb-warren
fact_type: alias

value:
  name: Harrison Grant

status: confirmed
```

Character knowledge about this fact is tracked separately.

Therefore:

```text
Objective Canon
≠
Character Knowledge
≠
Reader Knowledge
```

---

# 12. Temporal Facts

Many facts are true only during part of the story.

Example:

```text
Leah works as a detective.
```

may be true during one period but not another.

A temporal fact may therefore contain:

```yaml
valid_from: 2023-01-01
valid_to: 2025-06-30
```

Either boundary may be omitted.

---

# 13. Permanent Facts

Some facts normally do not change.

Example:

```yaml
subject: chr-leah-bennett
fact_type: birth

value:
  date: 1987-04-16
```

Such facts may omit temporal boundaries.

---

# 14. State Facts

Some facts represent changing state.

Examples:

```text
Current location
Employment
Relationship status
Injury
Possession
Organization membership
Rank
Knowledge
Alive/deceased status
```

These facts must support temporal transitions.

---

# 15. State Transition Example

Example:

```text
Before November 3:
Ray Carter → alive

November 3:
Ray Carter → killed

After November 3:
Ray Carter → deceased
```

LithoMark should be capable of representing this without deleting historical state.

---

# 16. Death

A death fact should record structured information where known.

Example:

```yaml
lithomark:
  type: canon-fact
  id: fact-ray-death

subject: chr-ray-carter
fact_type: death

value:
  date: 2023-11-03
  location: loc-irish-channel
  cause: gunshot

status: confirmed
```

Once established, this may create derived continuity constraints.

---

# 17. Derived Constraints

A confirmed death may imply:

```text
Character cannot normally be physically alive after death date.
```

However, LithoMark should flag rather than automatically reject violations.

Valid story reasons might include:

- Flashback
- Dream
- Hallucination
- Recording
- Photograph
- Memory
- Mistaken identity
- Supernatural story rules

The writer decides.

---

# 18. Birth and Age

Age should preferably be derived from birth date where possible.

Example:

```yaml
subject: chr-leah-bennett
fact_type: birth

value:
  date: 1987-04-16
```

Given a scene date, LithoMark may calculate age.

This is preferable to repeatedly storing:

```text
Age: 36
```

when the story spans multiple years.

---

# 19. Approximate Birth Information

Not every story requires exact dates.

LithoMark should support:

```text
Exact date
Year only
Approximate year
Range
Unknown
```

Example:

```yaml
value:
  date_type: approximate
  year: 1987
```

---

# 20. Physical Attributes

Physical attributes may include:

```text
height
eye color
hair color
handedness
tattoos
scars
distinguishing features
```

Example:

```yaml
subject: chr-example
fact_type: physical-attribute

value:
  attribute: eye-color
  value: green
```

---

# 21. Changing Physical Attributes

Physical attributes may change.

Examples:

- Hair color
- Weight
- Facial hair
- Scars
- Injuries
- Clothing
- Prosthetics

Temporal scope may therefore apply.

---

# 22. Injuries

Injuries should support structured tracking.

Example:

```yaml
subject: chr-leah-bennett
fact_type: injury

value:
  type: gunshot-wound
  body_location: shoulder
  severity: serious
  event: evt-irish-channel-shooting

valid_from: 2023-11-03
status: confirmed
```

Recovery may be represented by subsequent facts or status transitions.

---

# 23. Occupation

Occupation may be temporal.

Example:

```yaml
subject: chr-leah-bennett
fact_type: occupation

value:
  occupation: detective
  organization: org-nopd

valid_from: 2018
```

---

# 24. Organization Membership

Example:

```yaml
subject: chr-leah-bennett
fact_type: organization-membership

value:
  organization: org-nopd
  role: detective

status: confirmed
```

---

# 25. Location

Characters and objects may have location facts.

Example:

```yaml
subject: chr-leah-bennett
fact_type: location

value:
  location: loc-royal-street-gallery

valid_from: 2024-06-03T09:12:00
valid_to: 2024-06-03T11:08:00
```

LithoMark may use these facts to identify impossible travel or simultaneous-location conflicts.

---

# 26. Residence

Residence differs from immediate physical location.

Example:

```yaml
subject: chr-leah-bennett
fact_type: residence

value:
  location: loc-leah-apartment
```

LithoMark must distinguish:

```text
Lives at
```

from:

```text
Currently located at
```

---

# 27. Relationships

Relationships should be first-class objects and may also participate in canon.

Example:

```yaml
lithomark:
  type: relationship
  id: rel-leah-ray

source: chr-leah-bennett
target: chr-ray-carter
relationship_type: police-partner

valid_to: 2023-11-03
```

Canon may establish the relationship as confirmed.

---

# 28. Relationship Changes

Relationships can change over time.

Examples:

```text
strangers
acquaintances
friends
partners
romantic partners
estranged
enemies
former partners
```

LithoMark should preserve relationship history rather than replacing previous state.

---

# 29. Possession

Possession tracks who physically controls an object.

Example:

```yaml
subject: obj-forged-journal
fact_type: possession

value:
  possessor: chr-example

valid_from: 2024-06-03T10:00:00
```

---

# 30. Ownership

Ownership is distinct from possession.

Example:

```text
Owner:
Marcus

Current possessor:
Leah
```

LithoMark must permit these to differ.

---

# 31. Character Knowledge

Knowledge is a critical continuity system.

A character must not normally act upon information they have not yet learned.

Knowledge facts should record:

```text
Who knows
What they know
When they learned it
How they learned it
Certainty
```

---

# 32. Knowledge Fact Example

```yaml
subject: chr-leah-bennett
fact_type: knowledge

value:
  knows_fact: fact-harrison-real-identity

learned_at:
  event: evt-identity-reveal

status: confirmed
```

Before that event, Leah should not normally behave as though she knows the fact.

---

# 33. Knowledge About Facts

Knowledge should reference another canon fact where practical.

Example:

```text
Objective fact:
fact-harrison-real-identity

Known by:
Natalie
Harrison

Not yet known by:
Leah
```

This prevents duplicating the underlying truth.

---

# 34. Belief Versus Knowledge

A character may believe something that is false.

LithoMark should distinguish:

```text
Objective Canon
Knowledge
Belief
Suspicion
Claim
```

These must not be treated as equivalent.

---

# 35. Belief

Example:

```yaml
subject: chr-leah-bennett
fact_type: belief

value:
  about: chr-suspect
  proposition: responsible-for-gallery-murder

status: suspected
```

A belief does not modify objective canon.

---

# 36. Suspicion

Suspicion may be useful for mystery and crime projects.

Example:

```yaml
subject: chr-leah-bennett
fact_type: suspicion

value:
  target: chr-suspect
  subject: gallery-murder

valid_from:
  scene: scn-example
```

This may help prevent premature investigative knowledge.

---

# 37. Claims

A character's statement can be recorded as a claim without being accepted as truth.

Example:

```yaml
subject: chr-marcus
fact_type: claim

value:
  proposition: home-all-evening
  truth_status: unverified
```

---

# 38. Reader Knowledge

Future versions may optionally track what the reader has been shown.

Reader knowledge is distinct from character knowledge.

This can support:

- Mystery reveals
- Dramatic irony
- Foreshadowing
- Reveal timing

Reader-knowledge tracking must not be required for basic canon functionality.

---

# 39. Sources

Canon facts should be capable of recording their source.

Potential sources include:

```text
Writer entry
Story Bible
Timeline event
Scene
Chapter
Imported project data
Template
External agent proposal
Plugin
```

Example:

```yaml
source:
  type: scene
  id: scn-example
```

---

# 40. Source Does Not Determine Truth

A source records where the fact was established or documented.

It does not mean every statement in that source is objectively true.

Canon status remains explicit.

---

# 41. Fact Status

Canon facts should support status.

Initial values:

```text
planned
provisional
confirmed
retconned
deprecated
disputed
```

---

# 42. Planned Facts

A writer may know something that has not yet appeared in the manuscript.

Example:

```text
Harrison is actually Caleb Warren.
```

This can exist as:

```yaml
status: planned
```

The fact is part of the writer's intended story truth but may not yet be revealed.

---

# 43. Confirmed Facts

A confirmed fact is accepted by the writer as current project canon.

Example:

```yaml
status: confirmed
```

Continuity checking may treat confirmed facts as authoritative.

---

# 44. Provisional Facts

A provisional fact is under consideration.

LithoMark should not treat provisional facts as hard continuity requirements by default.

---

# 45. Retcon

Writers change their minds.

LithoMark must support retconning.

A retconned fact should not simply disappear.

Example:

```yaml
status: retconned

superseded_by: fact-new-value
```

This preserves project history.

---

# 46. Fact Supersession

When one canon fact replaces another, LithoMark may record:

```yaml
supersedes:
  - fact-old-value
```

The old fact may record:

```yaml
superseded_by: fact-new-value
```

This provides traceable canon evolution.

---

# 47. Contradictory Facts

LithoMark must detect potentially contradictory confirmed facts.

Example:

```text
Fact A:
Ray died November 3, 2023.

Fact B:
Ray died December 12, 2023.
```

If both are confirmed and refer to the same canonical death, LithoMark should flag the conflict.

---

# 48. Temporal Non-Contradictions

Different values are not automatically contradictory when time explains the difference.

Example:

```text
2018:
Leah lives at Address A.

2024:
Leah lives at Address B.
```

These facts may both be correct.

Continuity checking must consider temporal scope.

---

# 49. Fact Scope

A fact may apply:

```text
Always
From a date
Until a date
Between dates
During an event
During a scene
During a chapter
At an exact moment
```

This allows LithoMark to model story state accurately.

---

# 50. Exact Dates

Example:

```yaml
valid_from: 2024-06-03T09:00:00
```

Time zone may be included where meaningful.

Projects that do not require exact time-zone handling need not specify it for every event.

---

# 51. Approximate Dates

LithoMark must support uncertain chronology.

Example:

```yaml
valid_from:
  type: approximate
  value: 2004
```

Approximate information must not be treated as falsely precise.

---

# 52. Date Ranges

Example:

```yaml
valid_from:
  type: range
  earliest: 2004-01-01
  latest: 2004-12-31
```

This can represent information known only to a range.

---

# 53. Relative Time

Facts may reference events.

Example:

```yaml
valid_from:
  after_event: evt-irish-channel-shooting
```

Relative chronology is useful when exact dates are not yet established.

---

# 54. Timeline Integration

Canon and timeline are separate but connected.

Timeline answers:

```text
What happened when?
```

Canon answers:

```text
What is true?
```

An event may create or change canon facts.

---

# 55. Event Effects

A timeline event may produce state changes.

Example:

```text
Irish Channel Shooting
        │
        ├── Ray status → deceased
        ├── Leah injury → gunshot wound
        ├── Leah status → hospitalized
        └── Leah knowledge → partner killed
```

These effects may be represented as linked canon facts.

---

# 56. Scene Integration

A scene may reference:

```yaml
timeline_event: evt-example

characters:
  - chr-example

locations:
  - loc-example
```

LithoMark may evaluate applicable canon facts for the scene's story date.

---

# 57. Continuity Engine

The Continuity Engine evaluates explicit structured information.

Conceptually:

```text
Scene
  │
  ├── Date
  ├── Characters
  ├── Location
  └── Structured metadata
       │
       ▼
Applicable Canon Facts
       │
       ▼
Deterministic Rules
       │
       ▼
Potential Conflict
```

---

# 58. Continuity Warnings

Warnings may include:

```text
Character appears after confirmed death.

Character knows information before recorded discovery.

Character appears in two distant locations simultaneously.

Object is possessed by two incompatible characters.

Character's age conflicts with birth date.

Relationship status conflicts with scene date.

Injury state conflicts with chronology.

Alias is used before its recorded adoption.

Organization membership conflicts with date.
```

---

# 59. Warning, Not Verdict

A continuity warning means:

```text
LithoMark found structured information that may conflict.
```

It does not mean:

```text
The writer made an error.
```

The writer may mark a warning:

```text
Resolved
Intentional
False positive
Deferred
```

---

# 60. Continuity Rule IDs

Continuity rules should have stable machine-readable identifiers.

Examples:

```text
continuity.character.after-death
continuity.knowledge.before-learned
continuity.location.overlap
continuity.age.mismatch
continuity.possession.conflict
```

This supports:

- Settings
- Plugins
- Suppression
- Testing
- Reports

---

# 61. Severity

Warnings may have severity levels such as:

```text
info
warning
error
```

Severity reflects structural confidence, not literary judgment.

Example:

```text
Confirmed death followed by ordinary physical appearance
```

may be high confidence.

A possible travel-time issue may be lower confidence.

---

# 62. Suppression

Writers must be able to intentionally suppress a warning.

Suppression should record:

```text
Rule
Affected objects
Reason
Scope
```

Suppression must not modify the underlying canon fact.

---

# 63. Continuity Exceptions

Some stories intentionally violate ordinary assumptions.

Examples:

- Time travel
- Clones
- Resurrection
- Alternate realities
- Supernatural beings
- Unreliable chronology

Project templates or settings should eventually allow relevant continuity rules to be configured.

---

# 64. Genre Independence

The Canon Model must work for:

- Crime
- Mystery
- Thriller
- Romance
- Fantasy
- Science fiction
- Historical fiction
- Horror
- Literary fiction
- Screenplay
- Nonfiction where appropriate

Core canon architecture must not assume a single genre.

---

# 65. Custom Facts

Writers and templates may define custom fact types.

Example:

```yaml
fact_type: custom

custom_type: magical-affinity

value:
  school: fire
```

Custom facts must remain structured and namespaced where necessary.

---

# 66. Plugin Fact Types

Plugins may register additional fact types through the Plugin API.

Example:

```text
com.example.fantasy.magic-school
```

Plugins must not overwrite core fact definitions.

---

# 67. Plugin Continuity Rules

Plugins may register deterministic continuity rules.

Example:

```text
Fantasy Calendar Plugin

Rule:
Character cannot attend events occurring on incompatible calendar dates.
```

Plugin rules operate through documented Canon and Timeline APIs.

---

# 68. No Direct Database Dependency

Canon must not exist only inside SQLite.

Canonical facts must exist in authoritative project files.

SQLite may index facts for performance.

Deleting the SQLite index must not delete canon.

---

# 69. Canon Storage

Canon is a Story Development domain system. Dedicated canon-fact files, when used, belong beneath the canonical `Story Development/Canon/` hierarchy rather than creating a separate top-level project authority.

Legacy canon locations may be read during migration, but new Core writes must use the canonical Story Development hierarchy. Entity metadata may still hold simple stable attributes as described below.


Canon facts may be stored in dedicated human-readable files.

Recommended project structure:

```text
Story Bible/
└── Story Development/
└── Canon/
    ├── Characters/
    ├── Locations/
    ├── Objects/
    ├── Relationships/
    └── Other/
```

Alternatively, simple structured facts may live in their subject entity's front matter where appropriate.

The implementation may use both approaches.

---

# 70. Embedded Facts

Simple stable attributes may be embedded directly in an entity document.

Example:

```yaml
birth:
  date: 1987-04-16
```

This is appropriate when the fact is simple, singular, and naturally belongs to the entity.

---

# 71. Dedicated Fact Objects

Complex, temporal, sourced, disputed, or independently referenced facts should use dedicated canon-fact objects.

Example:

```text
Ray's death
Leah's injury
Harrison's alias adoption
Character knowledge
Object possession
```

This hybrid approach prevents unnecessary file explosion while preserving structured complexity.

---

# 72. Canon Index

LithoMark may create a derived Canon Index in SQLite.

It may contain:

```text
fact_id
subject_id
fact_type
temporal_start
temporal_end
status
source
```

The index is rebuildable from authoritative files.

---

# 73. Fact Query API

LithoMark Core should support queries such as:

```text
getFacts(subject)
getFactsAt(subject, date)
getFactHistory(subject, type)
getKnowledge(character, date)
getLocation(subject, date)
getRelationships(subject, date)
getPossession(object, date)
```

These queries support UI, plugins, and authorized external agents.

---

# 74. Character State

LithoMark should be capable of constructing character state at a story moment.

Example:

```text
Leah Bennett
June 3, 2024

Status:
Alive

Occupation:
NOPD Detective

Current injuries:
Recovering gunshot wound

Known information:
A
B
C

Unknown information:
Harrison's real identity

Relationships:
...

Location:
Royal Street Gallery
```

This state is derived from explicit project information.

---

# 75. Object State

Likewise:

```text
Forged Journal
June 4, 2024

Owner:
Unknown

Possessor:
Evidence Unit

Location:
NOPD Evidence Storage

Status:
Logged as evidence
```

---

# 76. Story State

LithoMark may construct a broader story-state snapshot for a given date or scene.

This can support:

- Continuity review
- Revision
- Scene planning
- Timeline inspection

No generative AI is required.

---

# 77. Scene Context

When a writer opens a scene, LithoMark may display relevant structured context.

Example:

```text
Scene date:
June 3, 2024

POV:
Leah Bennett

Leah knows:
...

Leah does not yet know:
...

Current injuries:
...

Active relationships:
...

Current plot threads:
...
```

This is a major intended benefit of the Canon Model.

---

# 78. Spoiler Awareness

Writer-facing canon may contain information not yet revealed in the manuscript.

LithoMark may eventually provide display controls to hide future canon while working in earlier scenes.

This is a UI concern.

The underlying canon remains intact.

---

# 79. Retcon Workflow

A retcon should conceptually follow:

```text
Select Fact
    │
    ▼
Create Replacement
    │
    ▼
Mark Previous Fact Retconned
    │
    ▼
Rebuild Canon Index
    │
    ▼
Run Continuity Check
    │
    ▼
Show Affected Scenes/Facts
```

This lets writers understand the consequences of changing canon.

---

# 80. Impact Analysis

Changing a canon fact may affect:

```text
Scenes
Timeline events
Relationships
Character knowledge
Plot threads
Reveals
Clues
Other facts
```

LithoMark should eventually provide impact analysis before major canon changes.

---

# 81. Deletion

Deleting a canon fact should identify dependent references.

LithoMark must not silently destroy those relationships.

A fact may instead be:

```text
deprecated
retconned
```

when preserving history is preferable.

---

# 82. Import

Imported structured facts should be converted into LithoMark canon where reliable.

Imported prose must not automatically become canon simply because it contains factual-looking sentences.

---

# 83. External Agent Proposals

An authorized external agent may propose canon facts.

Example:

```text
Proposed:

Leah Bennett
Injury:
Gunshot wound to shoulder
Source:
Chapter 3
```

Unless explicitly authorized otherwise, proposals should require writer confirmation before becoming confirmed canon.

---

# 84. Agent Permissions

Possible agent permissions include:

```text
canon.read
canon.propose
canon.create
canon.modify
canon.delete
```

Read access does not imply write access.

---

# 85. Plugin Permissions

Plugins may similarly receive:

```text
canon.read
canon.write
continuity.read
continuity.register-rules
```

Permissions are governed by:

`005-PLUGIN-ARCHITECTURE.md`

---

# 86. Auditability

Changes to significant canon facts should be revision-aware.

Where practical, LithoMark should record:

```text
What changed
When
Source of change
Previous value
New value
```

This supports safe revision and retcon workflows.

---

# 87. Validation

Canon validation should detect:

- Missing subjects
- Invalid stable IDs
- Unknown fact types
- Invalid dates
- Invalid temporal ranges
- Broken references
- Duplicate incompatible facts
- Impossible temporal ordering
- Invalid supersession chains

---

# 88. Circular Supersession

A fact must not indirectly supersede itself.

Invalid:

```text
Fact A supersedes Fact B
Fact B supersedes Fact A
```

The integrity checker must detect such cycles.

---

# 89. Performance

Canon queries may use SQLite-derived indexes for performance.

However:

```text
Canon files = authoritative
Canon index = derived
```

The entire index must be rebuildable.

---

# 90. Testing

Automated tests should include:

- Fact creation
- Fact modification
- Fact deletion
- Temporal facts
- Approximate dates
- Retcons
- Supersession
- Character knowledge
- Aliases
- Death constraints
- Age calculation
- Location overlap
- Possession changes
- Relationship history
- Broken references
- Plugin fact types
- Continuity rules
- Index reconstruction

---

# 91. No Literary Judgment

The Canon Model does not judge:

- Whether a plot is good
- Whether a character is believable
- Whether prose is effective
- Whether a reveal is satisfying
- Whether a story should be changed

Its role is structural factual consistency.

---

# 92. No Generative Requirement

All core canon functionality must operate without generative AI.

The system is based upon:

```text
Structured Facts
+
Timeline
+
Stable IDs
+
Deterministic Rules
=
Continuity Assistance
```

---

# 93. Canon and Story Bible

The Story Bible describes the story world.

Canon records structured truth within that world.

They overlap but are not identical.

Example:

```text
Character page:
Rich descriptive information

Canon:
Structured facts used by the system
```

Both may be presented together in the UI.

---

# 94. Canon and Timeline

Timeline:

```text
When did something happen?
```

Canon:

```text
What became true, stopped being true, or remained true?
```

Events may alter canon.

Canon gives events consequences.

---

# 95. Canon and Plot

Plot describes narrative construction.

Canon describes story-world truth.

A reveal may expose an existing canon fact without creating that fact.

Example:

```text
Canon:
Harrison is Caleb.

Reveal:
Leah discovers Harrison is Caleb in Chapter 22.
```

These are separate objects.

---

# 96. Canon and Character Knowledge

This distinction is mandatory:

```text
Truth
    │
    ├── Known by Character A
    ├── Unknown to Character B
    └── Incorrectly believed by Character C
```

Mystery, thriller, suspense, and dramatic irony depend upon this separation.

---

# 97. Canon and Reader Knowledge

Future reader-knowledge systems should follow the same principle:

```text
Objective Truth
Character Knowledge
Reader Knowledge
```

Each may change at different story moments.

---

# 98. Governing Principle

LithoMark does not decide the truth of the writer's story.

The writer records the truth.

LithoMark remembers it.

LithoMark tracks when it is true.

LithoMark tracks who knows it.

LithoMark identifies when structured information appears inconsistent with it.

The writer decides what to do.

---

# 99. Related Specifications

Parent specifications:

- `000-LITHOMARK-ARCHITECTURE.md`
- `001-PROJECT-FILE-FORMAT.md`
- `002-DOCUMENT-MODEL.md`
- `003-STABLE-ID-SYSTEM.md`

Related specifications:

- `005-PLUGIN-ARCHITECTURE.md`
- `006-AGENT-API.md`
- `007-KINDLING-MIGRATION.md`

Future supporting specifications should include:

- Timeline Model
- Relationship Model
- Story Bible Model
- Continuity Rule Engine
- Plot and Reveal Model
- Revision and Snapshot Model

---



# 100. Connected Timeline and Entity-State Projections

The Canon Model, Timeline, relationships, and Story Development entities form a connected deterministic story-state system.

A timeline event may identify participating entities and explicit effects. Those effects may begin, end, or alter canon facts such as location, employment, possession, injury, relationship status, organization membership, aliases, or character knowledge.

From the same authoritative event/fact history, LithoMark may provide multiple projections:

- Master event chronology
- Event-change view showing affected entities
- Per-entity timeline
- Relationship history
- Organization membership/history
- Scene-time context
- Continuity warnings

These projections must not maintain independent copies of story truth.

When sufficient structured information exists, the system should eventually support deterministic questions such as:

- What did this character know at this story point?
- Where was this character at this time?
- Was this relationship active during this scene?
- Was this character a member of this organization yet?
- Which event changed this entity's state?

Absence of structured data is not permission to invent an answer. Prose may inform the writer, but Core continuity logic operates on explicit structured facts and relationships.

Backlinks and resolved wikilinks may help discover connections, but discovery is distinct from canon. A mention of an entity does not automatically assert a new canon fact or relationship.

Relationship and organization graphs are visual projections over shared stable-ID relationship data and derived links. They must not require a separate authoritative graph database.

A point-in-time entity view should be able to reconstruct, where explicitly recorded, the state of a character, organization, relationship, location, or other entity at a selected story moment. This includes organization membership and hierarchy changes, rank/title changes, alliances, ownership/possession, location, injuries/status, and character knowledge.

Narrative order and story chronology remain distinct. A scene may appear later in the manuscript while depicting an earlier story time; state reconstruction must use story chronology rather than assuming chapter position equals time.

---

**END OF CANON MODEL SPECIFICATION**