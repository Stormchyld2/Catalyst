# Catalyst Audiobook Studio — Architecture Roadmap

**Status:** Future subsystem  
**Priority:** Post-core Writer OS / post-review-package implementation

**Architectural principle:** Catalyst manages the manuscript, production workflow, metadata, audio state, and provenance. Speech engines remain replaceable providers.

---

# 1. Core Architectural Boundary

Audiobook Studio must never make a text-to-speech engine authoritative over Catalyst project data.

The architecture should be:

```text
Catalyst Project
        │
        ▼
Audiobook Studio
        │
        ▼
NarrationProvider
        │
        ├──────────────┬──────────────────┬──────────────────┐
        ▼              ▼                  ▼                  ▼
     Kokoro          Piper          Other Local         Cloud Provider
     Local           Local            Provider             Optional
     FREE            FREE             Optional
        │              │                  │                  │
        └──────────────┴──────────────────┴──────────────────┘
                               │
                               ▼
                       Generated Audio
                               │
                               ▼
                     Catalyst Audio Project
                               │
                    ┌──────────┼──────────┐
                    ▼          ▼          ▼
                   WAV        MP3        M4B
```

Kokoro is the initial reference narration provider. It is not part of Catalyst's core architecture.

This separation protects Audiobook Studio if Kokoro disappears, changes licensing, becomes unsuitable for the project, or is superseded by a substantially better local model.

Catalyst should be capable of replacing the narration engine without changing the audiobook project format or losing production data.

---

# Phase A — Narration Provider Framework

Before building the Audiobook Studio UI, establish a provider-independent Rust abstraction.

For example:

```rust
trait NarrationProvider {
    fn capabilities(&self) -> ProviderCapabilities;
    fn voices(&self) -> Result<Vec<Voice>>;
    fn synthesize(
        &self,
        request: NarrationRequest
    ) -> Result<NarrationResult>;
}
```

The provider layer must understand capabilities rather than assuming every narration engine operates the same way.

Potential capabilities include:

```text
voice_cloning
voice_design
multi_speaker
streaming
word_timestamps
emotion_control
speed_control
pitch_control
pronunciation_control
watermarking
```

A provider reports which capabilities it actually supports.

Catalyst must not assume that Kokoro—or any other provider—supports every capability.

The provider structure could eventually resemble:

```text
providers/
    mod.rs
    kokoro.rs          # primary/reference local provider
    piper.rs           # possible lightweight local fallback
    voicestudio.rs     # possible external local provider
    elevenlabs.rs      # possible optional cloud provider
    system_tts.rs      # possible accessibility/preview provider
```

Catalyst communicates with narration engines through provider adapters.

A provider may:

- execute through a local inference runtime;
- communicate with an external local service;
- communicate with an optional cloud API;
- or use another implementation strategy supported by the provider framework.

No narration provider is authoritative over Audiobook Studio project data.

The provider abstraction—not Kokoro's implementation details—defines how Audiobook Studio communicates with narration engines.

---

# Phase B — Audiobook Project Format

Audiobook configuration must follow Catalyst's existing local-first, file-authoritative architecture.

A project could contain:

```text
My Novel/
├── Manuscript/
├── Characters/
├── Locations/
├── Audiobook/
│   ├── audiobook.yaml
│   ├── pronunciation.yaml
│   ├── voices.yaml
│   ├── chapters/
│   ├── renders/
│   └── exports/
```

There should be no proprietary audiobook database required to open or reconstruct the project.

SQLite or another database may eventually be used as a disposable index or cache for performance, but deleting that database must never destroy the audiobook project.

The authoritative state remains in normal project files.

An initial `audiobook.yaml` might contain:

```yaml
format: lithomark-audiobook
format_version: 1

title: All That Remains
author: Jason Williams

narration:
  provider: kokoro
  narrator_voice: voice-narrator-001

output:
  default_format: m4b
  sample_rate: 44100
```

The configuration remains human-readable, portable, versionable, and recoverable without Catalyst.

---

# Phase C — Manuscript → Narration Pipeline

This is where Catalyst can provide considerably more value than a generic text-to-speech application.

Catalyst already understands the manuscript structurally.

Instead of sending a 90,000-word text file directly to a narration engine, Catalyst should process:

```text
Novel
└── Chapter
    └── Scene
        └── Narration Segment
```

Catalyst creates deterministic Narration Segments.

Each segment receives a stable Catalyst identifier.

For example:

```text
nar-550e8400...
```

The production relationship becomes:

```text
Source Markdown
      │
      ▼
Narration Segment
      │
      ▼
NarrationProvider Request
      │
      ▼
Generated Audio
```

Every generated audio segment records the cryptographic hash of the source text used to produce it.

If the author changes:

```text
Caleb walked through the doorway.
```

to:

```text
Caleb stepped cautiously through the doorway.
```

Catalyst can detect:

```text
SOURCE CHANGED
Audio regeneration required.
```

The author can then regenerate only the affected narration segment rather than regenerating an entire chapter or audiobook.

This selective regeneration system should be considered one of Audiobook Studio's core architectural features.

---

# Phase D — Voice Library

Audiobook Studio should maintain its own provider-independent Voice Library.

Each voice receives a stable Catalyst ID.

For example:

```yaml
voices:
  - id: voice-narrator-001
    name: Primary Narrator

    provider:
      type: kokoro
      voice_id: provider-specific-id

    role: narrator
```

Catalyst uses:

```text
voice-narrator-001
```

throughout the audiobook project.

The provider-specific identifier remains confined to the voice-provider mapping.

Kokoro's internal voice identifiers should therefore never become authoritative identifiers throughout the project.

If the author later changes narration engines, Catalyst can remap:

```text
voice-narrator-001
```

to another provider and voice without restructuring the audiobook project.

---

# Phase E — Character Voice Assignment

Because Catalyst already maintains Character Profiles, Audiobook Studio can eventually connect character metadata with voice assignments.

For example:

```text
Caleb Warren
Voice: Caleb Male 01

Leah
Voice: Leah Female 02

Narration
Voice: Main Narrator
```

Audiobook Studio might display:

```text
CHARACTER VOICES

Narrator         Main Narrator       ✓
Caleb Warren     Caleb Male 01       ✓
Leah             Leah Female 02      ✓
Marcus Haverly   Unassigned          !
```

Character dialogue detection should initially require author confirmation rather than silently assuming who is speaking.

The system may later provide assisted speaker detection, but Catalyst should not automatically make uncertain dialogue attribution authoritative.

The author remains responsible for approving speaker assignments.

---

# Phase F — Pronunciation Dictionary

A pronunciation dictionary is essential for serious audiobook production.

For example:

```yaml
entries:
  - text: Muriale's
    pronunciation: mur-ee-AL-eez

  - text: Tchoupitoulas
    pronunciation: chop-ih-TOO-lus

  - text: Caleb
    pronunciation: KAY-lub
```

The pronunciation dictionary belongs to Catalyst—not the narration provider.

This allows pronunciation rules to survive provider changes.

Audiobook Studio could eventually support several scopes:

```text
Project Dictionary
Author Dictionary
Language Dictionary
```

Provider adapters are responsible for translating Catalyst pronunciation instructions into whatever mechanism a particular provider supports.

If a provider lacks native pronunciation controls, the adapter may use safe text preprocessing or report that the capability is unavailable.

---

# Phase G — Audiobook Production UI

Eventually the Catalyst sidebar receives:

```text
AUDIOBOOK STUDIO

Overview
Narrator
Character Voices
Pronunciations
Chapters
Production
Mastering
Export
```

The Chapter view might display:

```text
CHAPTER 7 — BEFORE THE BODIES

✓ Scene 1       Rendered
✓ Scene 2       Rendered
! Scene 3       Manuscript Changed
○ Scene 4       Not Rendered

Duration: 31:42

[Render Missing]
[Render Chapter]
[Play Chapter]
```

This gives the author an actual audiobook production environment rather than a simple text-to-speech button.

The interface should make the state of every chapter, scene, and narration segment visible.

---

# Phase H — Audio Review Workflow

Audio review should be designed into the architecture from the beginning.

Every generated narration segment should have an independent production status.

Initial states:

```text
Unreviewed
Approved
Needs Regeneration
Rejected
```

The author listens to generated narration and explicitly approves it.

Therefore:

```text
Generated
```

does not automatically mean:

```text
Production Approved
```

Catalyst keeps generation state and human approval state separate.

A completed audiobook should be buildable from approved material without requiring the author to remember which passages still contain problems.

---

# Phase I — Production Ledger and Reproducibility

Audiobook Studio should borrow the provenance and integrity principles established by Catalyst's review-package architecture.

Every render should record sufficient information to identify:

- the source text;
- the narration provider;
- the model;
- the voice;
- the synthesis settings;
- the generated audio;
- and the human review state.

A render record could resemble:

```yaml
segment_id: nar-82731
source_sha256: ...

provider:
  id: kokoro
  version: ...

model:
  id: kokoro-82m
  version: ...

voice:
  lithomark_id: voice-narrator-001
  provider_voice_id: ...

synthesis:
  speed: 1.0
  language: en-us

generated_at: 2026-11-12T21:32:11Z

audio:
  sha256: ...
  duration_ms: ...

review:
  status: approved
```

The exact schema can evolve during implementation, but the architectural requirement is important:

**Catalyst must know what source text, provider, model, voice, and settings produced each audio segment.**

This provides durable provenance and makes selective regeneration safer.

If the author returns months later and regenerates one paragraph, Catalyst can identify the production configuration used by the surrounding approved narration.

The Production Ledger should remain human-readable wherever practical.

---

# Phase J — Audiobook-Oriented Mastering

Catalyst should not attempt to become a full digital audio workstation.

Audiobook Studio only needs audiobook-oriented production tools.

Potential functions include:

```text
Trim silence
Normalize levels
Chapter spacing
Fade in/out
Noise processing
Peak checking
Loudness checking
```

Catalyst may also validate audio against common audiobook distribution requirements.

Those requirements must remain configurable and version-aware because distributors may change their technical specifications.

Mastering should operate downstream from narration generation.

The narration provider creates speech.

Catalyst manages audiobook production.

---

# Phase K — Export

Initial export formats should include:

```text
WAV
MP3
M4B
```

WAV provides a high-quality production/master format.

MP3 provides broad compatibility and chapter-oriented distribution files.

M4B is particularly valuable because Catalyst already understands the manuscript's chapter structure.

Catalyst could automatically generate or embed:

```text
Title
Author
Narrator
Cover
Chapter markers
Chapter titles
Duration
Metadata
```

Eventually a command such as:

```text
Build Audiobook
```

could generate the complete deliverable from approved chapter audio.

Unapproved or stale narration should be detected before final production.

---

# Phase L — Provenance and Distribution Readiness

Audiobook Studio should support production provenance even if that functionality is not prominent in the initial user interface.

Catalyst could generate an:

```text
Audiobook Production Record
```

containing information such as:

```text
Author
Work
Narration method
Provider
Provider version
Models
Model versions
Voice sources
Voice rights
Generation dates
Human review status
Audio hashes
Mastering information
Export information
```

This record does not guarantee acceptance by Audible, Apple Books, Spotify, or another distributor.

Its purpose is to give the author a durable and verifiable record of how the audiobook was produced.

---

# Phase M — Voice Rights

Voice rights should be a first-class Audiobook Studio concept.

A voice entry should eventually support metadata such as:

```yaml
rights:
  source: author
  consent_recorded: true
  commercial_use: true
```

For cloned, imported, purchased, licensed, or otherwise externally sourced voices, Catalyst should not merely store an audio sample or provider identifier.

The project should be capable of recording the provenance and usage rights associated with that voice.

Potential voice sources could include:

```text
Bundled
Author-owned
Licensed
Provider-supplied
Cloned with consent
Custom/imported
```

This becomes particularly important if Catalyst is distributed publicly or supports providers capable of voice cloning.

Voice capability and voice rights are separate concepts.

A provider being technically capable of using a voice does not establish that the user has the right to use that voice.

---

# What Audiobook Studio Should NOT Become

Several boundaries should be established from the beginning.

Catalyst should not become:

- a general-purpose voice cloning application;
- a full digital audio workstation;
- a podcast production suite;
- a music-production environment;
- a proprietary audiobook format;
- or an application dependent upon one specific AI model.

Audiobook Studio should remain:

**A manuscript-aware audiobook production system.**

Catalyst owns the production workflow.

Narration providers generate speech.

Standard audio formats remain the final deliverables.

---

# Development Order

When development reaches Audiobook Studio, implementation should proceed incrementally.

## Foundation

```text
NarrationProvider interface
        ↓
Kokoro provider adapter
        ↓
Audiobook YAML schema
        ↓
Stable narration segments
        ↓
Source hashing
        ↓
Render provenance
```

## Production

```text
Narrator selection
        ↓
Chapter/scene rendering
        ↓
Render status
        ↓
Selective regeneration
        ↓
Pronunciation dictionary
```

## Advanced Production

```text
Character voices
        ↓
Dialogue assignment
        ↓
Audio review/approval
        ↓
Production ledger
        ↓
Audiobook mastering
```

## Publishing

```text
WAV/MP3 export
        ↓
M4B builder
        ↓
Metadata
        ↓
Provenance report
        ↓
Distribution validation
```

This allows Catalyst to deliver a useful Audiobook Studio well before every advanced feature is complete.

---

# Reference Provider Strategy

Kokoro should serve as Catalyst's initial reference local narration provider.

The intended baseline experience is:

```text
Catalyst
    +
Local Narration Provider
    +
Audiobook Studio
    =
Local Audiobook Production
```

The architecture must nevertheless remain provider-independent.

A future user could potentially choose:

```text
Kokoro
Piper
Another local provider
Optional commercial local provider
Optional cloud provider
```

without changing the underlying Catalyst Audiobook Project.

The free, local path should remain a first-class workflow rather than a reduced demonstration mode.

Optional commercial or cloud providers may offer capabilities unavailable from the baseline local provider, but Audiobook Studio itself must not require them.

---

---

# Shared Narration Infrastructure — TTS Reader and Audiobook Studio

Catalyst's ordinary TTS Reader and Audiobook Studio are separate user workflows built on shared narration infrastructure.

The TTS Reader is transient and manuscript-oriented: read the current selection, sentence, scene, chapter, or other supported scope for proofreading and listening. It may support sentence-level start position, voice preview, playback speed, and pause/pronunciation controls. TTS playback state does not become authoritative manuscript content.

Audiobook Studio is durable and production-oriented: stable narration segments, voice assignments, render state, approvals, source hashes, production provenance, mastering, and export.

Both workflows share the NarrationProvider registry, runtime/model availability, voice discovery and preview, pronunciation infrastructure, speed/pause controls where supported, and playback/render job infrastructure. They must not maintain incompatible duplicate provider systems.

Audiobook-specific production records remain under the Audiobook project structure. Lightweight TTS Reader preferences may be user/application settings unless the author explicitly promotes a setting into project-level narration configuration.

---

# Runtime, Model, and Optional Component Policy

Kokoro is the reference local provider, but its runtime and model assets are optional locally managed components rather than mandatory authoritative project data.

The preferred distribution model is a reasonably small Catalyst core installer. When a user enables Local Narration, Catalyst may install or download a compatible narration runtime and selected model assets.

This preserves a completely local narration path without forcing large narration dependencies onto writers who never use TTS or audiobook production.

Narration components must expose version and compatibility information. Catalyst should report states such as installed, not installed, incompatible, unavailable, or update available without making the manuscript inaccessible.

Model/runtime installation locations are machine-specific implementation details and must not become authoritative absolute paths in portable project files. Project configuration should identify provider, model/voice identity, capabilities, and relevant versions in portable terms.

Removing Kokoro or another narration runtime must not remove manuscript text, voice mappings, pronunciation dictionaries, render ledgers, approved audio, or exports.

Background narration and audiobook rendering must not block manuscript editing. Long-running generation should use a cancellable job/queue model with visible progress and recoverable production state where practical.

Voice preview should be a first-class shared capability when supported so users can evaluate a voice without rendering a chapter.

---

# UI Reference Principles

External reader/TTS applications may be used as design references, but their architecture must not define Catalyst's project model.

Useful interaction patterns include a consolidated narrator/voice settings panel, dynamic voice discovery grouped by useful language/accent metadata rather than raw provider IDs, immediate voice previews, sentence-level playback start for proofreading, configurable speed and punctuation/paragraph/scene pauses where supported, provider-independent pronunciation rules, and background chapter/full-book rendering with visible progress.

Catalyst should go beyond generic document readers by using its structural knowledge of chapters, scenes, stable entities, manuscript changes, and production approval state.

# Architectural Rule

The long-term rule for Audiobook Studio is:

**Manuscript data belongs to the author. Audiobook project data belongs to the author. Generated audio belongs in standard files. Narration engines are replaceable tools.**

No narration provider should become the authoritative store for the manuscript, voice assignments, pronunciation dictionary, narration segments, review state, production history, or finished audiobook.

If a narration provider is removed entirely, the Catalyst project should remain intact, readable, and recoverable.

If Catalyst itself someday disappears, the author's manuscript, configuration files, production records, and exported audio should still exist as ordinary files.

That preserves the same local-first, file-authoritative philosophy used throughout Catalyst.
