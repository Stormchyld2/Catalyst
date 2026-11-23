# 011 — Catalyst Publishing & Export Studio — Architecture Roadmap

**Status:** Future subsystem  
**Priority:** Post-core Writer OS  
**Subsystem:** Publishing & Export Studio

**Architectural principle:** Catalyst manuscript files remain authoritative. Publication formatting is a separate, portable presentation layer applied during preview and export.

Publishing & Export Studio must never require proprietary formatting inside manuscript Markdown.

---

# 1. Core Architectural Boundary

Catalyst should distinguish between:

**Content** — what the author wrote.

and:

**Presentation** — how that content appears in a particular published or submitted format.

The basic architecture should be:

```text
Catalyst Project
        │
        ├── Manuscript Markdown
        ├── Chapter / Scene Structure
        ├── Project Metadata
        └── Publishing Configuration
                    │
                    ▼
          Publishing & Export Studio
                    │
                    ▼
             Document Model
                    │
             ┌──────┼──────┐
             ▼      ▼      ▼
           Styles  Layout  Metadata
             │      │      │
             └──────┼──────┘
                    ▼
               Export Engine
                    │
        ┌───────────┼────────────┐
        ▼           ▼            ▼
      DOCX         PDF          EPUB
        │
        ├───────────┬────────────┐
        ▼           ▼            ▼
      HTML       Markdown     Future Formats
```

The manuscript remains ordinary Markdown.

Formatting rules belong to Publishing & Export Studio.

Generated documents are outputs, not authoritative manuscript files.

---

# Phase A — Publishing Project Format

Publishing configuration should follow Catalyst's existing local-first and file-authoritative architecture.

A project could contain:

```text
My Novel/
├── Manuscript/
├── Characters/
├── Locations/
├── Publishing/
│   ├── publishing.yaml
│   ├── styles.yaml
│   ├── front-matter/
│   ├── back-matter/
│   ├── templates/
│   └── exports/
```

No proprietary publishing database should be required.

A disposable cache or index may eventually be used for performance, but deleting it must never destroy the publishing configuration.

An initial `publishing.yaml` could resemble:

```yaml
format: lithomark-publishing
format_version: 1

book:
  title: All That Remains
  author: Jason Williams
  language: en-US

manuscript:
  source: ../Manuscript/

defaults:
  profile: trade-fiction

exports:
  directory: exports/
```

The configuration remains human-readable, portable, Git-friendly, and recoverable outside Catalyst.

---

# Phase B — Intermediate Document Model

Catalyst should not convert Markdown directly and independently into every output format.

Instead:

```text
Markdown
    ↓
Catalyst Document Model
    ↓
Exporters
```

The internal document model should represent semantic structures such as:

```text
Book
Front Matter
Part
Chapter
Scene
Paragraph
Heading
Block Quote
Epigraph
Image
Scene Break
Page Break
Footnote
Endnote
Back Matter
```

This creates one consistent interpretation of the manuscript.

DOCX, PDF, EPUB, HTML, and future exporters consume the same document model.

The document model is an in-memory/intermediate representation.

It must not become the authoritative manuscript format.

---

# Phase C — Semantic Formatting

Formatting should be based on what content **is**, not on manually embedded appearance instructions.

For example:

```text
Chapter Title
Scene Break
Body Paragraph
First Paragraph
Block Quote
Epigraph
Front Matter Title
Copyright Text
Dedication
Acknowledgments
```

The style system then determines how each semantic element appears.

For example:

```yaml
chapter_title:
  alignment: center
  font_size: 18
  font_weight: bold
  space_before: 48
  space_after: 24

body:
  font_size: 11
  line_spacing: 1.15
  first_line_indent: 0.25in

first_paragraph:
  first_line_indent: 0
```

Exact schema details should be determined during implementation.

The important architectural rule is:

**Meaning belongs to the manuscript/document model. Appearance belongs to styles.**

---

# Phase D — Publishing Profiles

Catalyst should support reusable publishing profiles.

Initial profiles could include:

```text
Standard Manuscript
Trade Fiction
Trade Nonfiction
Paperback
Hardcover
Ebook
Custom
```

A profile provides sensible defaults rather than locking the author into a template.

For example:

```text
STANDARD MANUSCRIPT

12 pt readable font
Double spaced
1-inch margins
First-line indents
Page numbers
Author/title header
Chapter page breaks
```

while:

```text
TRADE FICTION

Book-oriented trim size
Publication typography
Chapter opening treatment
Scene-break styling
Running headers
Book page numbering
```

Users should be able to duplicate a built-in profile and customize it.

Built-in defaults should remain recoverable.

---

# Phase E — Style System

`styles.yaml` should contain project-specific presentation rules.

Potential style categories include:

```text
Body
First Paragraph
Chapter Number
Chapter Title
Part Title
Scene Break
Block Quote
Epigraph
Letter
Journal Entry
Poetry
Footnote
Endnote
Caption
Front Matter
Back Matter
```

This gives authors control without forcing formatting markup into their prose.

Style inheritance may eventually allow:

```text
Base Book Style
      ↓
Fiction Style
      ↓
Project Overrides
```

Project-specific overrides should remain minimal and explicit.

---

# Phase F — Front Matter

Publishing & Export Studio should provide structured front-matter management.

Potential elements include:

```text
Half Title
Title Page
Copyright
Dedication
Epigraph
Table of Contents
Foreword
Preface
Introduction
Acknowledgments
```

The author should determine:

- which elements exist;
- their order;
- whether they appear in specific export formats;
- and how they are styled.

Front matter should be stored as ordinary files wherever practical.

For example:

```text
Publishing/
└── front-matter/
    ├── title.md
    ├── copyright.md
    ├── dedication.md
    └── epigraph.md
```

---

# Phase G — Back Matter

Back matter should use the same architecture.

Potential elements include:

```text
Acknowledgments
About the Author
Also by the Author
Author Note
Appendix
Glossary
Bibliography
Notes
Reader Resources
Newsletter / Website Information
Preview of Another Book
```

Example:

```text
Publishing/
└── back-matter/
    ├── acknowledgments.md
    ├── about-author.md
    └── also-by.md
```

Individual elements may be enabled or disabled for different export profiles.

---

# Phase H — Book Layout

For print-oriented output, Publishing & Export Studio should understand book layout concepts.

Potential settings include:

```text
Trim Size
Margins
Gutter
Mirror Margins
Bleed
Page Size
Paragraph Indentation
Line Spacing
Chapter Opening Position
Widow/Orphan Control
Page Break Rules
Blank Page Rules
```

The system should distinguish between:

```text
Digital Layout
```

and:

```text
Paged/Print Layout
```

because EPUB and print PDF have fundamentally different layout requirements.

---

# Phase I — Typography

Typography controls may include:

```text
Body Font
Heading Font
Font Size
Line Height
Paragraph Spacing
First-Line Indent
Letter Spacing
Text Alignment
Small Caps
Drop Caps
```

Catalyst should not assume that every font available on the author's computer can legally be embedded in every exported format.

Font embedding should therefore eventually track:

```text
Font
Source
Embedding Permission
Export Compatibility
```

The software should warn rather than silently embed a font when rights or technical compatibility are uncertain.

---

# Phase J — Chapter Opening Styles

Chapter openings deserve dedicated handling.

Potential layouts include:

```text
CHAPTER SEVEN

BEFORE THE BODIES
```

or:

```text
7
Before the Bodies
```

or:

```text
SEVEN

Before the Bodies
```

or title-only chapter openings.

Options could include:

```text
Chapter Number
Chapter Title
Subtitle
Ornament
Opening Drop
First Paragraph Treatment
Drop Cap
Small Caps
```

These should derive from manuscript structure rather than requiring the author to manually format every chapter.

---

# Phase K — Scene Breaks

Scene breaks should be semantic elements.

The manuscript may contain a standardized scene-break marker.

Publishing & Export Studio can then render it differently depending on profile.

For example:

```text
* * *
```

or:

```text
◆
```

or additional vertical spacing.

An ebook profile may use one representation while a print profile uses another.

The manuscript source remains unchanged.

---

# Phase L — Headers, Footers, and Page Numbers

Print-oriented exports should support:

```text
Running Author Name
Running Book Title
Running Chapter Title
Page Numbers
Different Odd/Even Headers
Different First Page
Suppress Header on Chapter Openings
Roman-Numeral Front Matter
Arabic-Numeral Main Matter
```

These rules belong to the publishing profile rather than the manuscript.

---

# Phase M — Live Publication Preview

Publishing & Export Studio should eventually provide a preview representing the selected output profile.

The interface might resemble:

```text
PUBLISHING STUDIO

Profile: Trade Fiction ▼
Format:  Print PDF ▼
Trim:    6 × 9 in

────────────────────────────────

            CHAPTER SEVEN

         BEFORE THE BODIES


Caleb stepped cautiously through the
doorway. The hallway beyond was dark...

────────────────────────────────

Pages: 327
Words: 91,482

[Previous Page]          [Next Page]

[Export]
```

The preview should be generated from the same document model and style configuration used by the exporter.

The goal is to minimize differences between preview and final output.

---

# Phase N — DOCX Export

DOCX should be a first-class export format.

Primary uses include:

```text
Editor handoff
Agent submission
Publisher submission
Beta/reviewer copies
Traditional manuscript format
Editable formatted manuscripts
```

Where practical, Catalyst semantic styles should map to native Word styles rather than hard-coded formatting.

For example:

```text
Catalyst Body
        ↓
Word Normal / LM Body

Catalyst Chapter Title
        ↓
Word Heading / LM Chapter Title
```

This makes exported documents easier to edit professionally.

---

# Phase O — PDF Export

PDF should support at least two primary use cases:

```text
Proof / Review PDF
Print-Oriented PDF
```

Long term, print output may support:

```text
Trim dimensions
Embedded fonts
Bleed where appropriate
Image resolution checks
Margin validation
Page count
Blank-page handling
```

Catalyst should distinguish between an ordinary PDF and a validated print-oriented PDF.

---

# Phase P — EPUB Export

EPUB should be treated as a semantic digital publication rather than a fixed-layout print document.

EPUB generation should support:

```text
Chapter Navigation
Table of Contents
Metadata
Cover
Semantic Headings
Images
Footnotes
Endnotes
Accessibility Metadata
Language Metadata
```

Formatting should remain responsive to the reader's device and preferences.

Catalyst should avoid forcing print typography into EPUB.

Generated EPUB files should eventually be passed through standards validation before Catalyst reports them as validated.

---

# Phase Q — HTML and Markdown Export

HTML provides a useful portable publishing target for:

```text
Web publication
Author websites
Previews
Documentation
Interchange
```

Markdown export remains important because Catalyst's core philosophy requires portable text.

Markdown export could support:

```text
Whole Manuscript
Individual Chapters
Combined Manuscript
Clean Manuscript
Manuscript with Metadata
```

Catalyst should never make it difficult for an author to leave the application with their work intact.

---

# Phase R — Export Profiles

Authors should be able to save reusable export configurations.

For example:

```text
My Agent Submission
My 6×9 Paperback
My Ebook
ARC PDF
Editor DOCX
```

A profile might contain:

```yaml
id: export-paperback
name: My 6x9 Paperback

format: pdf
publishing_profile: trade-fiction

page:
  width: 6in
  height: 9in

front_matter: true
back_matter: true

output:
  filename: all-that-remains-paperback.pdf
```

These remain normal project configuration files.

---

# Phase S — Export Validation

Catalyst should validate an export before declaring it production-ready.

Validation may include:

```text
Missing Chapters
Empty Chapters
Broken Images
Missing Fonts
Unsupported Fonts
Missing Metadata
Invalid Internal Links
Unresolved Footnotes
Missing Cover
Layout Overflow
EPUB Validation Errors
Print Dimension Problems
```

The result should distinguish between:

```text
ERROR
WARNING
INFORMATION
```

An error may prevent a validated production build.

A warning informs the author without necessarily blocking export.

---

# Phase T — Export Manifest and Provenance

Like Audiobook Studio and the review-package architecture, publishing should maintain export provenance.

Each significant export can record:

```text
Source Manuscript Hash
Publishing Configuration Hash
Style Configuration Hash
Export Profile
Exporter Version
Generated At
Output SHA-256
Validation Result
```

For example:

```yaml
export_id: exp-82731
format: pdf

source:
  manuscript_sha256: ...

configuration:
  publishing_sha256: ...
  styles_sha256: ...

profile:
  id: export-paperback

generated_at: 2027-01-12T21:32:11Z

output:
  filename: all-that-remains-paperback.pdf
  sha256: ...

validation:
  status: passed
```

This allows Catalyst to determine exactly which manuscript and formatting configuration produced a particular file.

---

# Phase U — Stale Export Detection

Catalyst should know when an existing export no longer represents the current manuscript.

For example:

```text
ALL THAT REMAINS — Paperback PDF

Generated: January 12
Manuscript: CHANGED

⚠ Export is out of date.

[Rebuild]
```

Changes to any authoritative dependency can mark the export stale:

```text
Manuscript changed
Front matter changed
Back matter changed
Styles changed
Publishing settings changed
Cover changed
Export profile changed
```

This follows the same principle used by Audiobook Studio's source hashing.

---

# Phase V — Publishing Studio UI

Eventually the Catalyst sidebar could include:

```text
PUBLISHING

Overview
Book Setup
Front Matter
Back Matter
Styles
Page Layout
Preview
Export
```

The Overview might show:

```text
ALL THAT REMAINS

Manuscript
✓ 32 Chapters
✓ 91,482 Words

Publication
✓ Metadata
✓ Front Matter
✓ Back Matter
✓ Cover
✓ Styles

EXPORTS

Paperback PDF       ! Out of Date
Ebook EPUB           ✓ Current
Editor DOCX          ○ Not Generated

[Build Export]
```

This should feel like preparing a book for publication rather than operating a generic file converter.

---

# Phase W — Relationship to Audiobook Studio

Publishing & Export Studio and Audiobook Studio should remain separate subsystems built on common project structure.

They share:

```text
Manuscript
Chapter Structure
Book Metadata
Author Metadata
Cover Artwork
Provenance Principles
Source Hashing
```

But their production pipelines differ:

```text
                 Catalyst Manuscript
                         │
             ┌───────────┴───────────┐
             ▼                       ▼
       Publishing Studio       Audiobook Studio
             │                       │
       Document Model          Narration Segments
             │                       │
             ▼                       ▼
      DOCX/PDF/EPUB            WAV/MP3/M4B
```

Neither subsystem should become dependent upon the other.

---

# Phase X — Template System Integration

Publishing profiles should eventually integrate with Catalyst's broader template system.

A template could define:

```text
Book Metadata Defaults
Publishing Profile
Front Matter Structure
Back Matter Structure
Styles
Export Profiles
```

For example:

```text
Crime Novel
Romance Novel
Fantasy Novel
Memoir
Nonfiction
Short Story Collection
```

Templates provide starting points.

They do not create proprietary manuscript formats.

---

# What Publishing & Export Studio Should NOT Become

Catalyst should not attempt to become:

- Microsoft Word;
- Adobe InDesign;
- Affinity Publisher;
- a general desktop-publishing application;
- a graphic-design suite;
- or a proprietary publishing ecosystem.

Publishing & Export Studio should remain:

**A manuscript-aware book formatting and export system.**

Its purpose is to take the structured manuscript Catalyst already understands and reliably transform it into professional publishing and submission formats.

---

# Development Order

Implementation should proceed incrementally.

## Foundation

```text
Publishing YAML Schema
        ↓
Intermediate Document Model
        ↓
Semantic Style System
        ↓
Export Profiles
```

## Manuscript Export

```text
Markdown Parsing
        ↓
Chapter/Scene Structure
        ↓
DOCX Export
        ↓
Basic PDF Export
```

## Book Production

```text
Front Matter
        ↓
Back Matter
        ↓
Page Layout
        ↓
Headers/Footers
        ↓
Chapter Styling
        ↓
Live Preview
```

## Digital Publishing

```text
EPUB Export
        ↓
Navigation
        ↓
Metadata
        ↓
Cover Integration
        ↓
Validation
```

## Production Readiness

```text
Print-Oriented PDF
        ↓
Export Validation
        ↓
Export Manifest
        ↓
Source Hashing
        ↓
Stale Export Detection
```

---

# Initial Minimum Viable Publishing Studio

The first useful version does not need every feature described in this roadmap.

An initial release could provide:

```text
Manuscript
    ↓
Publishing Profile
    ↓
Basic Style Configuration
    ↓
Front/Back Matter
    ↓
DOCX / PDF / EPUB
```

That alone would allow an author to move from writing to a formatted deliverable without leaving Catalyst for routine export work.

More advanced typography, print validation, reusable custom profiles, and specialized publishing features can follow later.

---

# Architectural Rule

The permanent rule for Publishing & Export Studio is:

**The manuscript describes the book. The publishing configuration describes its presentation. Exporters create disposable deliverables from those authoritative sources.**

The manuscript must never depend on a particular export format.

A DOCX file is not the manuscript.

A PDF file is not the manuscript.

An EPUB file is not the manuscript.

They are generated representations of the manuscript.

If every generated export is deleted, Catalyst must be capable of rebuilding them from the authoritative project files.

If Publishing & Export Studio itself someday disappears, the author's manuscript, front matter, back matter, metadata, styles, and configuration should remain accessible as ordinary files.

That preserves Catalyst's local-first, file-authoritative, application-independent philosophy.

## Locked Preview Boundary

Live book-formatting and typesetting preview belongs exclusively to Publishing & Export Studio.

The Write workspace must remain focused on manuscript creation and revision. It may expose structural or semantic information needed for writing, but it must not become a second page-layout or book-formatting system.

Publishing preview is responsible for rendered output concerns including:
- trim size and page geometry;
- typography and paragraph styles;
- chapter-opening treatments;
- front and back matter;
- headers, footers, folios, and page breaks;
- print/PDF layout;
- EPUB presentation;
- export-profile-specific appearance.

This boundary prevents manuscript editing and publication formatting from becoming competing sources of truth.
