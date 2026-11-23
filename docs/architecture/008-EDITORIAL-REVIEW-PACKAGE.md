# LithoMark Editorial Review Package

## Status

Accepted architecture.

The LithoMark Editorial Review Package is the standard interchange format for manuscript review, beta reading, editorial feedback, ARC distribution, and returned review material.

The format is designed so that a reviewer does not need LithoMark to participate in the review process.

## Core Principle

A LithoMark review package is a portable collection of ordinary Markdown and YAML files. Markdown compatibility is a primary requirement, not an export convenience.

A review package must remain readable and useful in ordinary Markdown-capable applications even when LithoMark is not installed. It must not require a proprietary container, proprietary document body format, TipTap JSON, SQLite database, or LithoMark-specific application state for a reviewer to read the manuscript and provide feedback.

Review-copy protection is based on identification, integrity, provenance, and traceability. It must not compromise the portability of the manuscript or require proprietary DRM.

## Package Structure

```text
<Project Title> - Review/
├── README.md
├── REVIEW-TERMS.md
├── review.yaml
├── Manuscript/
│   ├── Prologue.md
│   ├── Chapter 01/
│   │   ├── Scene 01.md
│   │   └── Scene 02.md
│   └── Chapter 02/
└── Feedback/
    ├── Overall Feedback.md
    └── Scenes/
        ├── Scene 01.md
        └── Scene 02.md
```

The directory may be transported as a ZIP archive, but the archive is only a transport mechanism. The extracted contents are the review package.

## review.yaml

`review.yaml` contains package-level machine-readable metadata identifying the review distribution, review round, recipient, originating project, source documents, and integrity information.

The schema must include sufficient information to validate a returned review without relying on filenames.

The native format identifier is:

```yaml
format: lithomark-review
format_version: 1
```

A native review manifest includes concepts such as:

```yaml
format: lithomark-review
format_version: 1

review_id: rev-550e8400-e29b-41d4-a716-446655440002
round_id: rnd-550e8400-e29b-41d4-a716-446655440003

project:
  id: prj-550e8400-e29b-41d4-a716-446655440004
  title: All That Remains
  author: Jason Williams

reviewer:
  id: rvr-550e8400-e29b-41d4-a716-446655440005
  name: Jane Smith
  email: jane@example.com

distribution:
  purpose: beta-review
  created_at: 2026-09-27T22:00:00Z

integrity:
  algorithm: sha256
  manuscript_sha256: "<sha256>"
  files:
    - source_id: scn-550e8400-e29b-41d4-a716-446655440000
      path: Manuscript/Chapter 01/Scene 01.md
      sha256: "<sha256>"
```

Reviewer email is optional. The writer must provide at least one useful human-readable recipient identifier, such as a reviewer name or other recipient label.

Stable IDs, rather than filenames, are authoritative for matching returned review material to the originating project.

## Review and Recipient Identity

LithoMark distinguishes the editorial round, the distributed review package, and the recipient.

`round_id` identifies an editorial, beta-reading, or ARC round that may contain one or more recipients.

`review_id` uniquely identifies a specific distributed review package.

`reviewer_id` identifies the recipient for whom that package was prepared.

If the same manuscript is sent to five reviewers, each recipient receives a separate `review_id` and `reviewer_id`. Those packages may share the same `round_id`.

This allows LithoMark to establish which exact manuscript copy was distributed, to whom it was distributed, and when it was created without relying on filenames.

## Manuscript Files

Manuscript review copies are ordinary Markdown documents.

```markdown
---
type: scene
id: scn-550e8400-e29b-41d4-a716-446655440000
chapter_id: ch-550e8400-e29b-41d4-a716-446655440001
title: Gallery
review_copy: true
review_id: rev-550e8400-e29b-41d4-a716-446655440002
reviewer_id: rvr-550e8400-e29b-41d4-a716-446655440005
reviewer: Jane Smith
distribution: beta-review
exported_at: 2026-09-27T22:00:00Z
---

> **Review Copy - Not for Distribution or Publication**
>
> Prepared for: Jane Smith
> Project: All That Remains
> Author: Jason Williams
> Review ID: rev-550e8400-e29b-41d4-a716-446655440002

# Gallery

Leah stepped through the gallery doors...
```

The Markdown body must use standard Markdown wherever practical. LithoMark must not require proprietary body syntax for the manuscript to remain readable.

YAML frontmatter may contain LithoMark metadata, but another Markdown editor must be able to ignore that metadata without making the manuscript unusable.

Human-readable filenames are preferred. Stable UUID-based IDs belong in metadata and are not required in filenames.

Recipient-specific review metadata and the visible review-copy notice exist only in the exported review copy. They are not part of the authoritative manuscript.

## Feedback Files

Feedback is also stored as ordinary Markdown. Reviewers may provide overall manuscript feedback and scene-specific feedback without using LithoMark.

```markdown
---
type: review-feedback
review_id: rev-550e8400-e29b-41d4-a716-446655440002
reviewer_id: rvr-550e8400-e29b-41d4-a716-446655440005
scene_id: scn-550e8400-e29b-41d4-a716-446655440000
reviewer: Jane Smith
---

# Scene Feedback

The opening works well, but the character motivation becomes less clear after the conversation.

## Notes

- The transition into the gallery feels abrupt.
- The timeline could be clearer here.
```

The Markdown body is reviewer-authored content. LithoMark may use the YAML metadata to associate feedback with the appropriate project, chapter, scene, review package, recipient, or review round.

## Review-Copy Identification

Every exported manuscript Markdown file must visibly identify itself as a review copy.

The notice must appear after YAML frontmatter and before manuscript content. It must use ordinary Markdown so it remains visible in generic Markdown applications.

At minimum, the notice identifies:

- the file as a review copy;
- the intended recipient;
- the project title;
- the author or pen name when available;
- the unique review ID;
- that the copy is not intended for unauthorized distribution or publication.

The notice is a provenance and traceability mechanism. It is not DRM and must not make the Markdown file proprietary or unreadable outside LithoMark.

## Integrity and Provenance

LithoMark review packages use SHA-256 integrity hashes.

The integrity manifest in `review.yaml` records a SHA-256 hash for each exported manuscript file.

Each per-file hash is calculated from the exact bytes of the completed exported Markdown file after its YAML frontmatter, recipient-specific review metadata, visible review-copy notice, and manuscript Markdown have been assembled.

LithoMark also calculates a deterministic whole-manuscript SHA-256 value from the completed source inventory.

The whole-manuscript hash must not depend on filesystem enumeration order, ZIP metadata, ZIP compression, operating-system behavior, or the order in which files happen to be discovered.

The canonical whole-manuscript input is constructed from the manuscript source inventory sorted deterministically by normalized relative path and stable source ID. Each canonical inventory record contains:

```text
normalized relative path
stable source ID
SHA-256 of exact exported file bytes
```

LithoMark hashes the deterministic canonical representation of those records to produce `manuscript_sha256`.

The ZIP archive itself is not the canonical object for manuscript integrity. Recompressing an unchanged review directory must not cause the manuscript integrity value to change.

The integrity manifest allows LithoMark to determine whether a returned review-copy manuscript file differs from the exact file originally exported.

A cryptographic hash establishes integrity and identifies the exported bytes. It does not, by itself, establish legal authorship or copyright ownership.

## Distribution Ledger

LithoMark must retain an author-side record of every recipient-specific review, beta-reading, editorial, or ARC package it exports.

The distribution record belongs to the authoritative project and must not depend on the reviewer returning the original package unchanged.

At minimum, a distribution record retains:

- `review_id`;
- `round_id`;
- `reviewer_id`;
- recipient name or identifier;
- recipient email when supplied;
- project ID;
- project title;
- author or pen name when available;
- UTC export timestamp;
- distribution purpose;
- whole-manuscript SHA-256 hash;
- the source inventory;
- per-file SHA-256 hashes.

The author-side record establishes what LithoMark exported, which source documents were included, which exact exported bytes were distributed, who the package was prepared for, and when it was created.

A returned review package must never be allowed to rewrite, replace, or become authoritative over the original author-side distribution record.

The durable representation of the distribution ledger must follow LithoMark's file-authoritative architecture. It may be indexed by SQLite for convenience or performance, but SQLite must not be its sole authoritative storage.

The distribution ledger must survive deletion and rebuilding of LithoMark's disposable index/cache.

## REVIEW-TERMS.md

Every native review package contains a human-readable `REVIEW-TERMS.md`.

The file explains in plain language that:

- the manuscript is a private review or ARC copy;
- copyright and authorship remain with the author or applicable rights holder;
- the recipient is not authorized by the review package to redistribute the manuscript;
- the recipient is not authorized by the review package to publish the manuscript;
- the recipient is not authorized by the review package to commercially exploit the manuscript;
- the recipient is not authorized by the review package to claim authorship of the manuscript.

`REVIEW-TERMS.md` is ordinary Markdown and must remain readable without LithoMark.

The terms file is informational and evidentiary. LithoMark must not represent it as DRM, guaranteed prevention of copying, guaranteed legal protection, or a substitute for legal advice or a separately executed agreement when one is required.

## Manuscript Safety

Importing a review package must never silently overwrite the author's manuscript. The manuscript contained in a review package is a review copy.

A returned review package is never authoritative over the writer's project files.

If a reviewer edits a manuscript copy directly, LithoMark may detect those differences and present them as proposed changes. Applying a reviewer-authored prose change to the authoritative manuscript requires an explicit author decision.

Recipient-specific review metadata and visible review-copy notices must never be imported as authoritative manuscript metadata or manuscript prose.

LithoMark must distinguish among:

1. reviewer-authored feedback;
2. changes made to the review-copy manuscript;
3. review-package provenance metadata;
4. the authoritative writer manuscript.

Those categories must not be silently merged.

## Structured Editorial Changes

LithoMark may support richer editorial functionality including anchored comments, suggested replacements, discussions, reviewer messages, accept/reject decisions, writer responses, editorial rounds, and revision tracking.

These capabilities must be layered on top of the portable review format. They must not make ordinary Markdown manuscript and feedback files unreadable without LithoMark.

Internal editor representations such as TipTap JSON or HTML are not the canonical interchange representation.

## Conversion Boundary

During the Kindling migration, manuscript prose may still originate as HTML stored in SQLite.

```text
Current editor/storage representation
        ↓
LithoMark document conversion
        ↓
Standard Markdown
        ↓
Portable review package
```

A plain HTML tag stripper is not considered sufficient for canonical LithoMark conversion because it may discard meaningful formatting.

The review system should ultimately use the same Markdown/document conversion layer used by LithoMark project persistence.

The review exporter must create its manuscript Markdown through that shared conversion boundary rather than through a separate lossy conversion implementation.

## Import Review

LithoMark provides an Import Review workflow.

Import must:

1. validate the package format and supported format version;
2. validate project, review, round, and recipient identity;
3. associate returned material using stable IDs rather than filenames alone;
4. preserve reviewer-authored feedback;
5. verify the original export against the retained author-side distribution record;
6. compare returned manuscript files with their original SHA-256 integrity information;
7. detect changed manuscript review copies;
8. distinguish review-copy metadata and notices from manuscript prose;
9. prevent review-copy metadata or notices from contaminating authoritative manuscript files;
10. prevent silent replacement of authoritative manuscript files;
11. require an explicit author decision before applying reviewer-authored prose changes;
12. preserve existing writer decisions and newer review state when reconciling returned material.

A modified manuscript review copy is evidence of a proposed change, not authorization to replace the writer's source document.

## Export for Review

Export for Review is initiated from an open LithoMark project.

The writer may select the manuscript scope to include and identifies the intended recipient.

LithoMark then:

1. creates the review and recipient identities;
2. records the review round;
3. converts the selected manuscript through the shared Markdown conversion boundary;
4. adds recipient-specific review metadata and visible review-copy notices;
5. creates ordinary Markdown feedback templates;
6. creates `README.md`;
7. creates `REVIEW-TERMS.md`;
8. calculates SHA-256 hashes;
9. writes `review.yaml`;
10. records the author-side distribution ledger entry;
11. publishes the completed review directory or transport ZIP only after package creation succeeds.

The resulting package must be usable by a reviewer who does not own or install LithoMark.

A partially written package must not be published as a completed review package. Package creation should occur in a temporary sibling location and be published only after all required files and integrity information have been successfully written.

## Protection Boundary

LithoMark review protection is based on identification, integrity, provenance, and traceability rather than DRM.

A reviewer who can read a manuscript can technically copy its text. LithoMark must not claim otherwise.

The protection model is:

```text
Portable Markdown
        +
Recipient identification
        +
Review, round, and reviewer IDs
        +
Visible review-copy notice
        +
SHA-256 integrity manifest
        +
Author-side distribution ledger
        =
Traceable portable review copy
```

The protection layer must not encrypt the manuscript into a proprietary format, require LithoMark to read it, or prevent the package from being opened in ordinary Markdown software.

This design intentionally favors durable evidence and traceability over restrictions that would undermine LithoMark's portability requirements.

## LithoMark Reviewer Companion

LithoMark must provide a free reviewer-facing companion application, provisionally named **LithoMark Reviewer**, for beta readers, critique partners, ARC readers, and professional editors who do not own LithoMark Writer OS.

The companion is a review client, not a restricted trial of Writer OS. A reviewer must not be required to purchase, license, or install LithoMark Writer OS in order to complete a review.

The first implementation should be a lightweight desktop application that consumes and returns the same native `lithomark-review` package defined by this specification. It must not introduce a second review-package format.

### Reviewer experience

A normal reviewer workflow is:

```text
Receive review package
        ↓
Open with LithoMark Reviewer
        ↓
Read manuscript
        ↓
Highlight / comment / categorize / suggest changes
        ↓
Answer author questions and add chapter/overall feedback
        ↓
Finish Review
        ↓
Export completed review package
        ↓
Return package to author
        ↓
Author imports into LithoMark Review Studio
```

Opening a package should take the reviewer directly to the review assignment rather than require creation of a LithoMark project, repository, vault, or workspace.

The companion should expose only capabilities necessary to perform and return a review. It does not need manuscript-authoring, project-planning, Git, terminal, audiobook-production, publishing, plugin-management, or other Writer OS workspaces.

### Review package remains authoritative for interchange

LithoMark Reviewer is a convenience client over the portable review-package contract. It must not become necessary to interpret the manuscript or basic feedback.

A valid review package must remain readable and reviewable using ordinary Markdown-capable tools as required elsewhere in this specification. This preserves the no-lock-in guarantee even if the companion application is unavailable.

The companion may maintain disposable local application state for convenience, but reviewer-authored durable feedback required for return to the author must be written into the portable review package or into a completed package derived from it.

### Manuscript safety

The review copy is read-only by default from the reviewer's perspective. Comments, findings, questions, reactions, and suggested changes are separate review data.

Where an editorial assignment permits a suggested replacement, the companion records the proposed change without silently rewriting the review manuscript as though it were authoritative source text.

The author remains the only party who can apply returned suggestions to the authoritative LithoMark project.

### Assignment-driven interface

The interface may adapt to the review type declared by the package.

A beta-reading assignment may emphasize simple reactions and categories such as pacing, character, confusion, continuity, positive reaction, and general comment.

An editorial assignment may expose richer categories and structured suggested changes such as developmental, line, copy, proofreading, POV, dialogue, style, continuity, query, and replacement suggestion.

These are interface affordances over the same review interchange model, not separate incompatible package types.

### Author questions and completion

A review package may include author-supplied questions at manuscript, chapter, scene, or other supported scope. LithoMark Reviewer should present those questions in context and preserve the reviewer's answers as structured returned feedback.

Before completion, the companion should summarize review progress, including relevant measures such as chapters visited, unanswered required questions, findings/comments created, and overall feedback status. Progress indicators assist the reviewer but must not prevent return of a partial review unless the package explicitly marks a response as required.

### Identity and privacy

Reviewer identity comes from the recipient-specific package and may be confirmed or supplemented by the reviewer. A LithoMark cloud account must not be required for the baseline desktop review workflow.

The companion should operate locally and offline for ordinary review work. Returning the completed package may use any transport chosen by the participants, such as email, cloud storage, file transfer, or Discord; LithoMark does not need to operate a manuscript-hosting service for this workflow.

### Integrity behavior

LithoMark Reviewer must preserve the package's review, round, project, reviewer, stable source IDs, and original integrity/provenance information.

It may add returned-review metadata and reviewer-authored feedback, but it must not rewrite the original provenance to imply that reviewer changes were part of the author's original export.

The author's retained distribution ledger remains authoritative for what was originally distributed.

### Shared implementation boundary

Where practical, Writer OS and LithoMark Reviewer should share the native review domain model, package parser/writer, integrity validation, anchoring logic, and compatible rendering components rather than implementing two divergent review engines.

The companion should remain independently packageable so that it can be distributed free of charge without exposing unrelated Writer OS functionality.

### Future web reviewer

A browser-based reviewer may be considered later, but it is not required for the baseline architecture. A hosted reviewer would introduce authentication, hosting, privacy, synchronization, storage, and operating-cost concerns that are intentionally avoided by the initial local companion design.

Any future web client must consume the same review contract and must not invalidate the offline desktop or ordinary-Markdown review paths.

### Governing reviewer-access rule

**Review participation must never require purchase or ownership of LithoMark Writer OS. LithoMark Reviewer provides the preferred free review experience, while the portable Markdown package remains the durable fallback.**

## Legacy Kindling Compatibility

The existing `kindling-editorial` version 1 package format is a legacy compatibility format. LithoMark may retain a reader/import path for existing legacy packages during migration.

New native LithoMark review packages must not use `kindling-editorial` as their format identifier. Legacy compatibility must not define the architecture of the native LithoMark review system.

Legacy compatibility is a migration concern and must not become the storage or interchange model for new LithoMark reviews.

## Relationship to Project Persistence

The review-package architecture intentionally follows the same principles as LithoMark project persistence:

```text
LithoMark Project                 Review Package
----------------                 --------------
Markdown documents        ->      Markdown documents
YAML metadata             ->      YAML metadata
Stable IDs                ->      Stable IDs
Normal folders            ->      Normal folders
Disposable indexes        ->      No required database
```

This allows review interoperability and project portability to share the same document model rather than creating two unrelated storage systems.

Review data that must outlive the application, including the author-side distribution ledger, must have a durable file-backed representation. A SQLite database may index that information but must not become its sole source of truth.

## Portability Requirement

A valid LithoMark review package must pass the following test:

1. Export a recipient-specific review package.
2. Verify that the author-side distribution record exists independently of the exported copy.
3. Copy the package to a computer without LithoMark.
4. Open the manuscript in an ordinary Markdown editor.
5. Read the complete review manuscript and its visible review-copy identification.
6. Read `README.md` and `REVIEW-TERMS.md` without LithoMark.
7. Write feedback using ordinary Markdown.
8. Optionally edit a manuscript review copy to propose a prose change.
9. Return the package.
10. Import the returned package into LithoMark.
11. Associate feedback with the correct project, review round, recipient, and source documents using stable IDs.
12. Verify unchanged files against their SHA-256 hashes.
13. Detect changed manuscript review copies.
14. Preserve reviewer feedback.
15. Do not import review-copy notices or provenance metadata as manuscript prose.
16. Do not silently overwrite the author's manuscript.
17. Require explicit author action before applying a reviewer-authored manuscript change.
18. Delete and rebuild any disposable SQLite review index without losing the authoritative distribution record.

If those requirements cannot be satisfied, the implementation does not meet the LithoMark review-package architecture.

## Browser-Based Reader Package

The primary recipient experience for Beta and ARC distribution is a self-contained browser-based reader package, not an installed desktop application.

A generated package should contain the manuscript, review metadata, reader UI assets, and feedback schema together in one transportable bundle. The expected delivery form is a ZIP archive that the recipient extracts and opens locally in a normal web browser.

The reader must not require:
- LithoMark installation;
- a reader account;
- an internet connection for normal reading or feedback;
- a local server;
- proprietary manuscript storage.

The browser reader must be designed around browser security constraints. Manuscript data needed by the reader should be bundled into the package rather than depending on unrestricted filesystem access. Returned feedback must be explicitly exported by the reader into a portable feedback file/package that LithoMark can import.

Beta and ARC packages use the same reader technology but expose different workflows:

- **Beta Reader mode:** passage highlighting, inline comments, chapter-level notes, structured questionnaires, and overall feedback.
- **ARC mode:** reading-first presentation with optional notes, completion/review prompts, and lighter-weight feedback controls.

Reviewer annotations never mutate the authoritative manuscript. Imported feedback is associated with the exact distributed manuscript revision using stable IDs, review IDs, hashes, and the author-side distribution ledger.

A future hosted-link reader may be added as an optional delivery method, but the offline browser package is the baseline architecture.

## Feedback and Review Separation

Inline comments, highlights, questionnaires, and reviewer notes belong to the review package / Review workflow, not to the core Write editor.

The author's manuscript remains unchanged while feedback is collected. LithoMark's Review workspace is responsible for importing, grouping, displaying, resolving, accepting, dismissing, or otherwise acting on returned feedback.

The system should preserve a reviewer-independent path: recipients must still be able to read and return useful feedback without purchasing or installing LithoMark.
