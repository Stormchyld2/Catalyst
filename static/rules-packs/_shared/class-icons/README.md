# Class icons

One folder per game system, one file per class: `<system-id>/<class-id>.svg` (or `.png`, 512×512, transparent background).

D&D 5e icons go in `dnd-5e/` and are shared by `dnd-5e-2014` and `dnd-5e-2024`:

```text
artificer  barbarian  bard  cleric  druid  fighter  monk
paladin  ranger  rogue  sorcerer  warlock  wizard
```

The file name is the class id, the same `id` the class has in `content/classes.json`. Reference an icon from that file by its path relative to `static/rules-packs/`:

```json
{ "id": "wizard", "icon": "_shared/class-icons/dnd-5e/wizard.svg" }
```

Only add artwork you made or are licensed to redistribute: this repository is public.
