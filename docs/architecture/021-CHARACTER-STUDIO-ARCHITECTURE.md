# 021 — Catalyst Character Studio, Guided Builder & Sheet Designer

**Status:** Approved feature architecture and open-source evaluation baseline · 2026-11-23  
**Scope:** Character creation, system-independent character records, guided/manual builder, digital/print sheets, progression, rules packs, GM/player permissions and offline integration.  
**Implementation status:** Design approved; **do not infer feature implementation from this specification**.  
**Dependencies:** `020-CAMPAIGN-WIZARD-ARCHITECTURE.md`, `001-PROJECT-FILE-FORMAT.md`, `002-DOCUMENT-MODEL.md`, `003-STABLE-ID-SYSTEM.md`, `005-PLUGIN-ARCHITECTURE.md`.  
**Design benchmark:** D&D Beyond-style guided user experience, not copied proprietary design, code, content or branding.

## 1. Goal and principles

Character Studio must allow a GM or player to **(A) build a character through guided choices** or **(B) bypass the builder and fill a character sheet manually**. Both workflows create the **same canonical character record** and open the same digital character sheet. The selected rules pack controls which fields, choices, restrictions and derived statistics are meaningful. A fully system-neutral/manual mode is mandatory.

Catalyst remains offline-first, Windows/Tauri/Svelte 5/Rust initially, later Android and local LAN. No account, cloud API, external font/CDN, required internet, or new authoritative database. User files in normal folders remain authoritative. Preserve inherited stable-ID conventions and explicit project boundaries. Reuse existing Catalyst document service, entity-template registry, custom fields, link resolver, writer/wiki editor, recovery and export facilities.

A first-class character can be created without selecting an RPG system beyond System-Neutral. A 5e-compatible rules pack is the first detailed guided example, but **the core engine is not D&D-only**.

## 2. Character Studio navigation

`Character Studio` is a campaign workspace. Its library groups:
- **Player Characters** — full player character sheets and ownership metadata.
- **Non-Player Characters** — optional brief roleplay profile or full combat sheet.
- **Creatures & Monsters** — stat blocks and optional species/creature templates, stored under the campaign's existing creature folders.
- **Templates & Rules Packs** — installed system templates and GM-authored custom sheets.
- **Party Overview** — characters, links, resources, conditions and campaign roles.

Offer `New Character`, `Import`, `Duplicate`, `Print`, `Export`, `Archive`. Avoid presenting future modules as functional until implemented.

### Entry choice

**Guided Character Builder:** rules-driven sequence of selectable options, prerequisites, explanations and calculated outcomes.

**Manual Character Creation:** creates a blank sheet immediately, supports `Rules-Assisted` and `Fully Manual` modes. In fully manual mode, allow user-entered values even if inconsistent with a particular rules pack, and clearly distinguish derived versus overridden values.

A user can start manually and later invoke the builder or vice versa without overwriting authored fields or unknown extensions. When moving between modes, preview any required migration and conflicts.

## 3. Guided builder: initial ten-step 5e-compatible profile

The actual step schema comes from the installed rules pack; these are **default 5e-compatible steps**, not universal hardcoded steps. Skip/rename/reorder steps as necessary for another RPG.

1. **Identity** — name, player, character concept, pronouns, portrait, aliases.
2. **Species/Ancestry** — permitted options, variations, traits, languages, movement and prerequisite explanations.
3. **Class/Profession** — occupation/class, level, subclass when applicable, allowed feature choices.
4. **Background** — origins, proficiencies, benefits, roleplay hooks and optional narrative.
5. **Attributes** — standard array, point buy, roll or manual methods allowed by the rules pack.
6. **Skills & Proficiencies** — skills, saves, tools, weapons, armor, languages, permitted counts and prerequisites.
7. **Equipment** — packages, currency, weapons/armor, inventory, carried/equipped states.
8. **Spells & Features** — powers, spells, feats, resources and limited-use mechanics; skip when irrelevant.
9. **Personality & History** — appearance, ideals, motivations, relationships, goals, freeform backstory.
10. **Review & Create** — validate outstanding choices, show source-aware calculation breakdown, generate record and sheet.

Wizard navigation should permit back/forward, optional step skip, draft checkpoint/recovery, and reviewing selected options without restarting. Warn before choosing selections that invalidate downstream dependencies, show an explicit impact preview and preserve unrelated notes.

The builder has no independent final data authority. Before final Create it holds a draft state; afterward the character file is authoritative. Interrupted drafts can be recovered from app-owned temporary storage; on commit validate and write through the existing project document service.

## 4. Manual creation and homebrew

- Start from a rules-defined sheet template **or** blank System-Neutral template.
- Add, rename, hide, reorder and configure fields/sections; choose field types: text, long text, number, boolean, select, multi-select, resource counter, tag, image, linked entity, inventory table, repeating records, progress track.
- Custom abilities, skills, defenses, resources, dice and advancement paths must not require new code.
- Rules-Assisted mode may validate and calculate; Fully Manual mode does not enforce selected-system prerequisites.
- Imported/unknown fields remain round-trippable; no silent deletion on template or rules-pack upgrades.
- Support an editable NPC quick-profile, a full NPC sheet and reusable creature stat blocks without requiring a full character wizard.

## 5. One authoritative character schema

Create one versioned, portable format for each character. Suggested location: `Characters/Player Characters/<name>.md` or `Characters/Non-Player Characters/<name>.md` using YAML frontmatter + Markdown notes, with ancillary structured JSON/YAML only when justified. Identity is a stable character ID in the existing documented format; filename/title is not identity.

Illustrative conceptual record (subject to implementation-level validation):

```yaml
---
catalyst:
  schema_version: 1
  type: character
  id: chr-550e8400-e29b-41d4-a716-446655440000
  campaign_id: 4ce8f61b-6f46-4db6-8766-d09839191864
  rules_pack:
    id: system-neutral
    version: '1'
  sheet_template:
    id: system-neutral-basic
    version: '1'
  creation_method: manual
  validation_mode: fully-manual
  visibility: gm-and-owner
name: Elara
player_id: null
identity: {}
attributes: {}
skills: {}
defenses: {}
resources: {}
features: []
inventory: []
spells_or_powers: []
conditions: []
relationships: []
advancement_history: []
custom_fields: {}
---
## Background
Write backstory here.
```

Do not place personal credentials or security secrets in character records. Use relative asset paths for portraits and token images, safe under campaign root. Character records should link to world locations, groups/factions, other characters, quests, inventory items and encounters by ID. Link changes must survive file moves and renames.

**Derived values** are computed from base choices plus named modifiers with provenance (source rules pack, item, condition, GM override, etc.). Never overwrite authored base values with cached derivations. On edit, recompute dependents. Preserve a human-readable breakdown of each calculated total.

A character sheet view, printer, builder, GM console and dice actions are all **projections** of the same authoritative record. No duplicated mutable character database.

## 6. Rules-pack contract

A versioned rules pack defines:
- ID, name, version, declared edition and required license/attribution.
- Builder step configuration and ordering.
- Identity labels and configurable sections.
- Species/ancestry, class/profession, background, equipment, skill and spell option catalogs as applicable.
- Numeric and categorical attribute definitions, units and boundaries.
- Eligibility conditions, prerequisites, pick counts, dependencies and consequences.
- Derived formula rules and modifier provenance.
- Health/resource systems, rest/recovery and conditions.
- Advancement model (level-based, XP/purchase, milestones, or custom).
- Dice notation/rules and linked actions.
- Digital and printable sheet templates.

**Core rules:** `system-neutral` is required and needs no content beyond blank field types. Initial detailed rules: a verified licensed 5e-compatible SRD pack with declared edition (do not mix 2014 and 2024 rules silently). All other systems are future compatible without distributing unlicensed rulebooks. Imported homebrew packs must validate schema, detect cyclic dependencies and isolate unsafe content.

Do not execute arbitrary JS, Rust, shell or eval strings supplied by rules packs. Use a sandboxed, restricted declarative expression interpreter with safe arithmetic, allowed functions, explicit references, bounded evaluation, cycle detection, error reporting and regression tests. Fail gracefully on missing pack/version or unsupported formula. Never assign an unrecognized pack a default 5e rules meaning.

## 7. Digital character sheet and sheet designer

**Visual direction:** Catalyst Obsidian + Frost Blue app chrome with charcoal/gunmetal and silver neutrals, per [025](025-CATALYST-DESIGN-SYSTEM.md); amber-gold and ornate fantasy styling are optional sheet *skins* only, with a clean modern default and accessibility/contrast options. Keep decorative art away from text, clickable controls and small fields.

Digital sheet tabs/sections configured by the rules pack: Overview, Attributes, Skills, Combat, Inventory, Spells/Powers, Features, Background, Notes, Conditions, Relationships, History. Tab labels and counts must adapt to non-D&D systems.

Key UX: clear editable/derived state; direct dice-roll actions; tracking current/max resources; condition badges; equipment/spell usage; editable freeform notes; portrait, tokens and files; autosave/recovery; responsive narrow layout; keyboard accessibility. Never invent live gameplay mechanics for a system lacking them.

**Sheet Designer:** Grid or section-based configurable layout editor supporting:
- Drag/reorder components, resize, sections, tabs and print page breaks.
- Field widgets, repeating rows and inventory/feature tables.
- Formula bindings and source-aware tooltips.
- Custom headers/branding, imagery and themes.
- Preview with sample data; validation errors in the editor.
- Template versioning, import/export and duplicate.
- Safe propagation rules: existing characters retain data when a template changes; preview migrations and permit undo.

The template determines presentation; the character record owns values. Reusing one sheet template for multiple characters must not share their data.

## 8. Printing and export

Generate:
1. **Decorative Fantasy** printed/PDF sheet (based on approved Catalyst character-sheet aesthetic).
2. **Ink-Friendly** black/white sheet.
3. **Blank** paper sheet for handwriting.
4. **Completed** populated sheet.
5. Later: GM/NPC quick stat blocks and party summary pages.

Respect letter/A4 paper selection, print margins, page breaks, wrapping, tabular readability and offline font embedding. Export normal portable character record files and documented JSON/YAML exchange where supported. Printed and digital layouts must derive from the same record without data drift. Do not include third-party proprietary sheet artwork or trademarks without rights.

## 9. Advancement and lifecycle

The same builder engine also supports **Level Up / Advance**. Rules packs define progression choices and calculations; the engine shows before/after previews, prerequisites, unresolved choices and source breakdowns. Level-less systems may progress via skill points or other rules, and System-Neutral allows manual change history.

Keep an append-only/auditable progression/event record or versions through existing file-authoritative history. Retaining history must not force game outcome rollback. GM approvals where required are explicit; do not silently alter a character owned by another player.

Support character duplication, archive/restore, import, template switching with conflict preview and safe character sheet upgrades. Stable identity persists across renames and folder moves. Duplicating creates a distinct ID.

## 10. Game Master, campaign, map and dice integrations

- Character Studio's campaign root is created by `020`; wizard may create blank/minimal characters but does **not** depend on this builder being finished.
- World Builder links birthplace, hometown, factions, history, religion, NPC relationships and places.
- Adventure Studio links quests, scenes, milestones, rewards and consequences.
- Encounter Builder and GM Console present rules-appropriate status, initiative, attack, damage, saving throw and resource actions.
- All dice actions call the **shared globally accessible 3D Dice Roller** with a validated expression, roll purpose, character ID and visibility; no second dice engine.
- Only the GM/authorized owner may change protected character state; public display and LAN clients receive prefiltered player-safe data, not hidden JSON with CSS concealment.
- Shared-world references are read-only until a campaign-local state override is explicitly approved. Character files stay within campaign boundaries.
- Future Android touch interface uses same records and rules pack semantics; avoid Windows-only assumptions in the domain model.

## 11. Open-source engineering evaluation (not a blind copy mandate)

Evaluate *actual repository code and license at selected commit*, assets, third-party dependencies, content rights, offline behavior, platform compatibility and maintainability. Source links:

| Candidate | Role | Reported license | Integration assessment |
|---|---|---|---|
| Borealis — https://github.com/TeamBorealis/borealis | Guided 5e builder, advancement, offline file workflows, PDF | MIT OR Apache-2.0 | **High priority donor evaluation**; React/Tauri/Rust, so favor isolated portable calculations/data rather than porting UI wholesale |
| Sheetsmith — https://github.com/Richie735/obsidian-sheetsmith | System-neutral sheet templates, formulas, editable Markdown character records, layout designer | MIT | **High priority sheet designer evaluation**; Obsidian plugin APIs are not directly portable—extract pure engine ideas/code after audit |
| RPGForge — https://github.com/maximehp/RPGForge | System-independent rules-pack and deterministic overlay mechanisms | MIT | Evaluate pack validator/derived stats; current guided creator UX is reported incomplete |
| OpenPentacle — https://github.com/arrowedisgaming/OpenPentacle | 5e guided wizard, JSON content packs, leveling and PDF | GPL-3.0 | **Architecture/reference only by default**; copyleft obligations require explicit licensing decision before copying any code; server/SQLite/auth stack not suitable to import wholesale |
| Fablesheet — https://github.com/fablesheet/fablesheet | Offline Tauri character manager, generic sheet builder, combat and dice | GPL-3.0-or-later | **Architecture/reference only by default**; React+SQLite and GPL obligations make direct import a separate decision |

The permissive licenses of Borealis, Sheetsmith and RPGForge permit reuse **subject to their license notices and dependencies**, but their code is not automatically drop-in compatible with Catalyst's Svelte/file-authoritative services. MIT/Apache package components may carry separately licensed assets/content.

Borealis offers the most complete practical 5e guided-flow reference; Sheetsmith the closest manual/any-system template model; RPGForge the rules-pack boundary. **Preferred plan:** use Catalyst's inherited entity/document services as authority; evaluate pure reusable code from the three permissively licensed projects; adapt Svelte UI natively. Do not import any donor runtime database or account infrastructure.

**Legal gate:** Verify specific rules/data packs separately. SRD 5.1 and SRD 5.2.1 content may be used according to their applicable Creative Commons licenses with attribution; don't import proprietary book-only character choices or assume open-source code means all embedded game content is redistributable.

## 12. Engineering module boundaries (conceptual)

```text
src/lib/character-studio/
  domain/           # character types, IDs, drafts, changes, calculations
  rules/            # versioned packs, validators, constrained formulas
  builder/          # steps, dependencies, choices, progression
  sheets/           # renderer and sheet designer
  print/            # PDF/page layouts
  integrations/     # dice, maps, GM, campaign linking
  components/       # Svelte UI
  tests/
src-tauri/src/character_studio/
  storage/          # file-authoritative commands/adapters
  import_export/
```

Fit this into existing repository naming and architecture rather than creating a parallel file service.

## 13. Phases and acceptance gates

**Phase 0 — Donor/code audit and architecture proof**
- Inspect Catalyst's current Character/Template infrastructure; compare Borealis, Sheetsmith and RPGForge source and licenses; document exact reusable functions and dependencies.
- Prototype character schema, rules-pack validation and character save/reload on real campaign files.

**Phase 1 — Manual character MVP**
- Create System-Neutral character directly; editable sheet; save, reopen, duplicate and print basic ink-friendly sheet.
- Support custom attributes/sections and character linking.

**Phase 2 — Guided 5e builder**
- Implement licensed SRD pack, all required guided choices, prerequisites, calculations and `Review & Create`.
- Validate results against known rule examples; no unauthorized content.

**Phase 3 — Advancement and play**
- Level/advance, equipment/resource/condition tracking, shared 3D dice actions, GM links and roll event association.

**Phase 4 — Template designer and print themes**
- Editable sheet layout, formula engine, portable templates; decorative/ink-friendly/blank print modes.

**Phase 5 — Wider systems and future clients**
- Additional rules packs, homebrew, robust import/export, player access and Android/LAN readiness.

### Required tests
- Both guided and manual creation persist **identical canonical schema**.
- System-Neutral uses no class/species/D&D-only assumptions.
- Choice selection/prerequisite changes invalidate only affected options.
- Formula source breakdown is correct; cycles/malicious expressions rejected.
- Rules pack absent/incompatible version gives recoverable errors, not corrupted characters.
- Multi-character templates preserve isolated values; template migration preserves unknown fields.
- Portrait/asset paths safe, ID links stable across file rename/move and campaign isolation maintained.
- Print layout correct in decorative, blank and ink-friendly formats.
- Character actions invoke the shared dice engine; session/public/private permissions enforced at boundaries.
- Fully offline Windows runtime; no new frontend/Rust regression failures.

**Definition of done (V1 Character Studio):** On Windows, user can create a System-Neutral or appropriately licensed 5e-compatible character through either Guided or Manual mode, save/reopen it as normal portable files, use a digital sheet, print an accurate sheet and invoke the existing global dice engine. Advanced sheet designer and additional rules packs may ship as separate milestones but must be documented, not mislabeled complete.

**Do not start broad character implementation until the Campaign Wizard's file/folder format is fixed and verified.** This document is intended to allow early donor evaluation while Claude's other branches remain independent.


## 14. D&D Beyond functional-parity benchmark (2026 review)

**Product direction:** As close as practicable to the **functional experience** of D&D Beyond's character creator and digital sheets, without copying proprietary branding, original illustrations, protected layout artwork, copyrighted sourcebook options, accounts or backend behavior. The approved Catalyst charcoal/silver/amber visual design remains distinct.

Official benchmark references:
- D&D Beyond Character Builder: https://www.dndbeyond.com/characters/builder
- Builder Sections support: https://dndbeyond-support.wizards.com/hc/en-us/articles/7747202748436-Builder-Sections
- Current user experience/changelog: https://www.dndbeyond.com/changelog/

### Parity requirements

| D&D Beyond-like behavior | Catalyst implementation requirement | Priority |
|---|---|---|
| Step-by-step creation and contextual help | Guided creation with searchable choice cards, summaries, prerequisites, explanations and progress indicator | V1 |
| Character choices reflected in live stats | Real-time, provenance-aware derived values recalculated from rules and equipment | V1 |
| Class, species, background and level choices | Rules-pack-driven optional steps and verified 5e SRD options; separate edition and source filtering | V1 |
| Ability score methods | Standard array, point buy, roll and manual where pack permits; dice from shared roller | V1 |
| Completeness feedback | Pending/illegal choices clearly flagged, no silently invented defaults | V1 |
| Final interactive sheet | One editable character record with tabs/sections for combat, actions, spells, abilities, skills, inventory and notes | V1 |
| Equipment affects stats | Equipped/unequipped, attuned where applicable, condition modifiers, item effects with breakdown | V1 |
| In-play counters | HP, rests, conditions, slots/resources, death saves and other system-relevant trackers | V1 |
| Level up | Re-enter relevant builder choices with before/after summary and retained history | V1 |
| Personalization | Portrait/token, character backstory, optional themed sheet skins and local template library | V1 |
| Printing/export | Completed, blank, decorative and ink-friendly sheets, JSON/YAML/Markdown portability | V1 baseline / expanded later |
| Spell selection and management | Search/filter permitted spells, known/prepared rules, slot tracking; packs supply data | V1 for SRD-only content |
| Companion/campaign sharing | GM/owner permission model and local campaign links, future player-safe LAN sync | Later |
| Cross-device sync | Offline first; local backup/import and future encrypted LAN/Android sync, never mandatory cloud | Later |

Do not confuse D&D Beyond's dedicated **Quickbuilder** with Catalyst **Quick Start Campaign**: the latter creates a campaign, not a level-one character. A suggested-character quickbuilder could be added later, subject to rules pack legality.

### Interaction quality

- Step cards show official source/edition and brief explanations; filter/search large option lists.
- Sticky per-step progress and **Needs Attention** summary, before Create or Level Up.
- Preview immediate character impact when selecting class, background, feat, weapon, armor, spell or stat increase.
- "Why is this number X?" inspector on any derived stat with modifiers and inapplicable contributions distinguished.
- Never discard notes or equipment when class/rules option changes; show invalidation/conflicts explicitly.
- The sheet is primarily an **in-play console**, not just a printable PDF on screen.
- No arbitrary free-account caps or required accounts; Catalyst owns files locally.
- Manual mode remains an equally visible choice even where guided choices are offered.

## 15. Better stack-aligned open-source candidates (verified repository README/license)

The first donor list was broad. Prioritize **compatible technology and a healthy legal boundary** over feature count. Evaluate code at pinned revision before reuse; validate noncode game content and art separately.

| Repository | Technology and fit | License | Recommendation |
|---|---|---|---|
| **rpgman** — https://github.com/caioldcarvalho/dnd-character-tracker | **Rust pure rules engine + Tauri 2 + SvelteKit/Svelte UI**, data-driven JSON content, guided Build/Sheet/Gear/Spells, live resources, stat provenance, 2024 SRD direction | MIT code | **First source audit**. Closest stack fit; engine concepts and selective functions are promising. Project README admits early status, incomplete spell database and partial species/background content. |
| **Character Forge** — https://github.com/papsop/dnd-character | Guided 2024 rules flow, live legality/derived stats, offline browser, print-ready PDF and JSON export, SRD 5.2.1 content | MIT code/art; CC BY 4.0 SRD data; fonts OFL | **Second audit** for wizard validation, step design and sheet printing. Check exact UI and PDF dependencies. |
| **Sheetsmith** — https://github.com/Richie735/obsidian-sheetsmith | Framework-independent TypeScript concepts, configurable field/formula engine and Markdown sheet layouts; tightly coupled to Obsidian shell | MIT code | **Third audit** for manual sheet designer and generic systems. Separate pure functions from Obsidian APIs. |
| **Borealis** — https://github.com/TeamBorealis/borealis | More complete guided character flow but React frontend; Rust/Tauri elements may be portable | MIT/Apache-2.0 reported; verify | Secondary audit if guided 5e features lacking elsewhere. |
| **OpenPentacle** — https://github.com/arrowedisgaming/OpenPentacle | Svelte 5 guided UI and pure TypeScript rules, but server/SQLite/auth and GPL-3.0 | GPL-3.0 | **Reference-only** absent explicit GPL licensing decision. Same framework does NOT override license incompatibility. |
| **D&D 5e Character Creator** — https://github.com/TaylorMouse/dnd_character_creator | Vanilla JS offline guided builder and live sheet; depends on **separately supplied 5etools data** and official sheet PDF for export | MIT code only | Reference for UX/tests, **do not ship generator data or official PDF** unless rights individually cleared. |

**Selected engineering strategy:** inspect `rpgman` Rust calculation engine first (including all dependency/licenses), `Character Forge` for validation and print path, and `Sheetsmith` for schema-driven manual sheet layouts. Port only clearly separable code into existing Catalyst services with notices and regression tests; do not import another project's app shell, database or content pipeline. Do not imply any reuse has occurred until an implementation PR proves it.

An application built in SvelteKit may have components usable in a plain Svelte/Tauri app, but **its routing/server APIs are not automatically portable**. The Rust engine must be audited for version/edition coupling and not promoted to a universal tabletop system engine without proof.
