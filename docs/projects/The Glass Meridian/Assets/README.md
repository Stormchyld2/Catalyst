# Demo Assets

This folder belongs to the normal LithoMark project tree.

The demo uses filename-only image references so LithoMark can exercise its image lookup behavior while remaining compatible with explicit project-root-relative paths.

Expected demo image filenames:

- `mara-vale.png`
- `tomas-vale.png`
- `nia-corren.png`
- `elias-venn.png`
- `old-quay.png`
- `east-reach.png`
- `meridian-operations-center.png`
- `Venn Compass.png`

LithoMark image reference rules:

1. A reference containing a path, such as `Assets/Locations/old-quay.png`, is resolved explicitly from the project root.
2. A filename-only reference, such as `old-quay.png`, searches the indexed project images.
3. Exactly one filename match resolves automatically.
4. Multiple matching filenames are treated as ambiguous and must be corrected to an explicit project-root-relative path.
5. Missing images never prevent the project itself from loading.
