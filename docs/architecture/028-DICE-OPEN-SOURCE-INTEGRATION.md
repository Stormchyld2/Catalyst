# 028 — Dice Open-Source Integration

**Status:** Assessment complete; Dice Box 1.1.4 integrated
**Date:** 2026-11-23
**Parent:** 027-DICE-ROLLER-ARCHITECTURE.md

## Decision

Use **`@3d-dice/dice-box` 1.1.4** (MIT), pinned to an exact version, as the only new runtime dependency. Do not adopt Dice UI or the parser packages yet.

## What was inspected

The installed package source (`node_modules/@3d-dice/dice-box/dist`), its `package.json`, licence and bundled assets, and npm registry metadata on 2026-11-23.

| Package | Version | Licence | Last publish | Decision |
| --- | --- | --- | --- | --- |
| `@3d-dice/dice-box` | 1.1.4 (gitHead `79e926eb6e66dd6bc9faafcce420992d7756e148`) | MIT | 2024-08-24 | **Adopted** |
| `@3d-dice/dice-ui` | 0.5.2 | MIT | 2024-07-08 | Not adopted. Pulls in `dat.gui`; Catalyst builds its own tray to match the approved mockup and design system |
| `@3d-dice/dice-roller-parser` | 0.2.6 | MIT | 2022-05-26 | Deferred to Phase 2. Catalyst's own parser covers `NdX±N`; revisit for advanced notation (keep/drop, exploding) |
| `@3d-dice/dice-parser-interface` | 0.2.1 | MIT | 2023-02-08 | Deferred with the parser |
| `@3d-dice/dice-box-threejs` | 0.0.12 | MIT | 2022-10-26 | Not adopted. A candidate to investigate for predetermined (host-authoritative) results in Phase 4 |

**Maintenance:** Dice Box has had no release since August 2024. It works with the current toolchain (Vite 7, Svelte 5, WebView2/Chromium), and Catalyst depends on a small surface of it (below), behind one adapter. Replacing it would touch only `engine/diceRenderer.ts`.

## Dice Box 1.1.4 runtime facts

- **Bundled engines:** Babylon.js 5.57.1 (Apache-2.0) and Ammo.js (a Bullet physics port, zlib licence) are compiled into Dice Box's `dist/*.js` files. The `@babylonjs/*` npm packages it lists as dependencies are installed but not imported at runtime.
- **Workers:** the physics worker and the OffscreenCanvas render worker are inlined as base64 and started from `blob:` URLs (with a `data:` fallback).
- **Assets fetched at runtime:** `{origin}{assetPath}ammo/ammo.wasm.wasm` and `{origin}{assetPath}themes/default/{theme.config.json, default.json, *.png, *.jpg}`.
- **Results:** `roll()` resolves once every die is asleep; each value is read from the simulated die's upward face (`Dice.getRollResult`).
- **WebGL missing:** Dice Box falls back to `world.none.js`, which uses `Math.random`. Catalyst checks WebGL first and uses its own crypto-based resolver instead, so this path is never used.
- **No predetermined results:** the public API is `init`, `roll`, `add`, `reroll`, `remove`, `clear`, `hide`, `show`, `updateConfig`, `getRollResults`. None of them can force an outcome.
- **No `destroy()`:** see below.

### API surface Catalyst uses

`new DiceBox(config)`, `init()`, `roll(string[])`, `clear()`, the `canvas` property, and the prototype method `resizeWorld()`. Config: `container`, `id`, `assetPath`, `origin`, `theme`, `themeColor`, `scale`, `enableShadows`, `shadowTransparency`, `lightIntensity`, `offscreen`, `delay`.

## Modifications and workarounds

Dice Box itself is unmodified. The adapter (`DiceBoxRenderer`) works around two gaps:

1. **No destroy.** During `init()` the adapter wraps the global `Worker` constructor to record the two workers Dice Box creates, and restores it afterwards. `dispose()` terminates them (which frees the WebGL context held by the render worker), asks an onscreen context to lose itself if one exists, and removes the canvas. Catalyst creates no other workers, so nothing else is captured.
2. **Untracked resize listener.** `resizeWorld()` adds a `window` resize listener that is never removed. The adapter overrides it on the instance, captures the handler synchronously instead of registering it, and calls it from the tray's `ResizeObserver`.

If a later Dice Box adds `destroy()`, use it and delete both workarounds.

## Offline assets

| Published path | Source | SHA-256 |
| --- | --- | --- |
| `assets/dice-box/ammo/ammo.wasm.wasm` | dice-box 1.1.4 `dist/assets` | `007a4e93dce707e222e5f895a792a9e089f9dce65b36c057a32416644aa6be5e` |
| `assets/dice-box/themes/default/default.json` (dice meshes) | same | `26bf6fb1edea72296208f75dce59af3e64d0ad035a2876c0b49b7f1d1de01b50` |
| `assets/dice-box/themes/default/theme.config.json` | same | `4f55c758bca4c10405652c50037705b15bb633f998ca153d84f4c216ef67716d` |
| `assets/dice-box/themes/default/diffuse-light.png` | same | `de15a8eed40be8123d301f8d0dff19be4d6577cc9ffb11086b81d7e38afd62a0` |
| `assets/dice-box/themes/default/diffuse-dark.png` | same | `acf62205d3af969375998cf00af402959da870e846712b45ddbd8df5ddf5d21f` |
| `assets/dice-box/themes/default/normal.png` | same | `77b7b5102d4190391d1189c8ef9eedf6d4d483afa18ce50e57f3335f13b7f5f9` |
| `assets/dice-box/themes/default/specular.jpg` | same | `2af3b8a5da79f9b7953c550c62647e50ac56fdd7cf1c3397d6c176c1eb6fa88c` |

The files live in `static/assets/dice-box/` (Vite's `publicDir`) with Dice Box's `LICENSE` and `THIRD-PARTY-LICENSES.txt` (Babylon.js and Ammo.js/Bullet licence texts). The theme was authored by Frank Ali and ships in the MIT-licensed package. `src/lib/dice/tests/diceAssets.test.ts` fails if the copies drift from the installed package, so upgrading means: bump the exact version, copy `dist/assets` again, update this table.

Dice Box's `postinstall` also copies its assets to `public/assets/`; that folder is not served by Catalyst and is git-ignored.

**No CDN, remote script, remote font or remote texture is used.** The 2026-11-23 verification blocked every non-localhost request and observed none.

## Sound

Dice Box 1.1.4 ships no sounds. Catalyst synthesizes its rolling sounds with the Web Audio API at runtime (`src/lib/dice/engine/diceSound.ts`), so no audio files are bundled and no third-party sound licence applies.

## Third-party notices

See `THIRD-PARTY-NOTICES.md` at the repository root.
