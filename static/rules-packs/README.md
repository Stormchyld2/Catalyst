# Bundled rules packs

Rules packs that ship inside the Catalyst app. The engines read these JSON files; people never edit them in the app.

## Layout

```text
static/rules-packs/
  <pack-id>/
    manifest.json          # required: pack identity, license, dependencies
    content/classes.json   # optional, one file per category (classes, species,
    content/spells.json    #   backgrounds, feats, spells, equipment, monsters ...)
    definitions/*.json     # optional: fields, choices, progressions, modifiers ...
    sheets/default-sheet.json
  _shared/                 # assets used by more than one pack (class icons)
```

A pack only includes the categories it needs. The layout follows Character Builder doc 004 §4.

## Pack ids

Lowercase, digits and `-` only, system then edition, for example:

| Ruleset | Pack id |
| --- | --- |
| Pathfinder 2e | `pathfinder-2e` |
| Call of Cthulhu 7e | `call-of-cthulhu-7e` |
| Warhammer Fantasy Roleplay 4e | `warhammer-fantasy-4e` |
| D&D 5e (2014) | `dnd-5e-2014` |
| D&D 5e (2024) | `dnd-5e-2024` |

The folder name must equal `packId` in `manifest.json`. Ids never change once released, even if the title does.

## manifest.json

The schema is `PackManifest` in Catalyst-Character-Builder `packages/rules-core/src/index.ts`. Required fields:

```json
{
  "schemaVersion": 1,
  "coreSchemaVersion": 1,
  "packId": "dnd-5e-2024",
  "version": "1.0.0",
  "systemId": "dnd-5e",
  "editionIds": ["2024"],
  "kind": "base-rules",
  "title": "D&D 5e (2024) SRD",
  "publisher": "",
  "license": "CC-BY-4.0",
  "attribution": "",
  "distribution": "redistributable",
  "dependencies": [],
  "conflicts": [],
  "capabilities": []
}
```

Bump `version` whenever any file in the pack changes.

## Licensing

Both repositories are public, so everything here is published. Only add content you have the right to redistribute (for example SRD material under CC-BY or ORC, or your own text). Never paste rulebook text from a purchased PDF. Content that is only for your own use goes in a campaign's `Rules & Mechanics/Rules Packs/` folder instead, with `"distribution": "local-only"`.

## Character Builder copy

This folder is the source of truth. Catalyst-Character-Builder keeps an identical copy in its root `rules-packs/` folder. Change files here first, then copy the whole folder across unchanged so pack ids, versions and icon paths match.

Rulebook PDFs do not belong in this folder or in either repository. See doc 022 (Sourcebook Library).
