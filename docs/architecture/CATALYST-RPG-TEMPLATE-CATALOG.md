# Catalyst RPG Template Catalog — Implementation and Migration

**Date:** 2026-11-23
**Scope:** World Builder, Campaign Wizard and Adventure/GM templates.
**Status:** Source-level changes committed; full Windows build and runtime validation pending.

## Approved catalog policy

- 34 original LithoMark worldbuilding templates remain usable, retaining IDs and existing document field keys.
- 37 novel-only project, manuscript, outline, publishing and editorial review *entity templates* are not offered for creation of new Catalyst documents.
- Old IDs are still resolved through `built_in_template(id)` for backward compatibility. Existing document bodies and user-owned content are never deleted or rewritten. IDs remain reserved against user-created template collisions.
- 13 core Catalyst RPG templates remain, with additional optional linking and asset reference fields.
- 23 more reusable templates are added for adventure scenes/overviews, quests, puzzles, hazards, secrets/clues, handouts, treasure, events, Session Zero, nations, deities, economy, calendars, flora, taverns/shops, dungeons, equipment, consumables, spell lore, crafting, GM screen references and RPG print profiles.
- Five bundled novel outline JSON presets are retired; the old `get_bundled_templates` API returns an empty list, preserving interface compatibility with older clients. User-created templates are retained.
- Template manager now supports catalog search/filter, exposes and saves scope, and can duplicate built-ins to user-owned editable copies.

## Architecture compatibility

The authoritative source remains ordinary Markdown documents and versioned template manifests. The `story-development` scope stays valid for old user templates. New optional scope values include `worldbuilding`, `campaign`, `adventure`, `encounter`, `session` and `gm`. Editor must round-trip selected scope.

All new reference fields in this iteration are descriptive IDs stored as text or ID lists. **They are not yet validated typed cross-document links**; until a shared reference-picker and link-validation port is integrated, they must not be advertised as enforcing referential integrity or GM permissions. Do not invent rules stats or put character mechanics into wiki documents. Document metadata `visibility` is descriptive only unless enforced by the host's permission service.

The original worldbuilding templates' display labels and example Markdown bodies were updated for campaign terminology but field IDs were preserved for legacy data. No automatic migration of existing document bodies occurs.

## Integration and validation checklist

1. Run `cargo fmt --check`, Rust unit tests, frontend unit tests, `svelte-check`, frontend build and Windows/Tauri smoke test.
2. Confirm legacy templates are absent from new-document menus but old linked documents remain viewable.
3. Confirm all RPG templates validate, have unique IDs, and can be used to create/save/reopen local Markdown records.
4. Confirm custom templates with `catalyst_template` and legacy `lithomark_template` frontmatter load; scope round-trips; duplication creates a new ID and does not overwrite a built-in.
5. Confirm Campaign Wizard uses the matching templates after integration and that its file preview reflects actual seeded output.
6. Confirm permission filtering and real reference selection before enabling player-facing sharing or dynamic links. A plain `visibility` field must not be treated as access control.
7. Replace the Glass Meridian novel demo with a file-authoritative RPG adventure demo in a separately validated change.

## Deferred engine work

A typed stable-ID reference picker and resolver, edition-aware ruleset selectors, playable random-table integration through shared dice S07, and true print-profile renderer integration are **not** implied by template definitions. Their adapters must be implemented and tested in the proper engine workstreams.

No branch change can by itself update a previously built Windows executable; rebuild after integration.
