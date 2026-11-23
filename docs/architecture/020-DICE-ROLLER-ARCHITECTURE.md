# 020 — Dice Roller Architecture

**Status:** Phase 1 implemented (global floating tray, physics `Nd20`); later phases planned
**Date:** 2026-11-23
**Parent:** 000-CATALYST-ARCHITECTURE.md
**Related:** 021 Dice Open-Source Integration; 022 Dice Result Protocol; 023 Dice GM Console Integration; 024 Dice Implementation Roadmap

## Purpose

Catalyst has one dice roller, shared by every surface that needs dice: the global tray today, and the Campaign Studio, Character Studio, Encounter Manager and Game Master Console as they land. Players and GMs watch physical-looking dice tumble and settle, and the settled faces are the result.

It is a global utility, not a workspace. It is not a separate application.

## User-facing shape

The tray is a **floating, nonblocking utility panel** that looks like a modal but does not behave like one (approved interface addendum, 2026-11-23).

- **Open it from anywhere:** the Dice button at the bottom of the module rail, the View → Dice Roller menu item, the command palette, or `Mod+Shift+D` (rebindable in Keyboard settings, like every Catalyst command).
- **Nonblocking:** no scrim, no focus trap, no `aria-modal`. It sits at `--z-overlay`, above workspaces and below real modal dialogs. Pointer events outside its bounds reach the workspace underneath, so the user can keep typing in an editor while it is open.
- **Draggable and resizable:** drag by the top grip (or move it with arrow keys when the grip is focused), resize from the bottom-right corner. Position, size and mode are remembered in `localStorage` (`catalyst:diceTray`) and re-fitted to the window when it changes size.
- **Compact and expanded modes:** compact is a small strip for quick rolls with instant results; expanded is the mockup layout with the 3D dice surface. Switching modes keeps history.
- **Layout (expanded):** die types and +/- on the left; settings, history, Roll, Reroll, Clear, Compact and Close on the right; the total and per-die values at the top; the dice-count strip along the bottom. Colours are the Catalyst charcoal, metallic silver and amber-gold identity.
- **History survives closing:** history lives in the global store, not the component.

## Layers

```text
Catalyst UI (DiceTray, ModuleRail button, command/shortcut)
    -> diceTray store            (global state, history, geometry, settings)
        -> DiceEngine            (parse, validate, resolve, build RollRecord)
            -> DiceRenderer      (3D presentation: Dice Box adapter)  [optional]
            -> RandomSource      (crypto, unbiased)                   [no renderer]
        -> RollHistory           (session history; file sink in Phase 3)
```

| Layer | Owns | Must not |
| --- | --- | --- |
| Engine (`engine/diceEngine.ts`, `diceNotation.ts`, `diceResults.ts`, `diceRandom.ts`) | Notation, validation, roll IDs, totals, modifiers, which dice this build may roll | Touch the DOM or WebGL |
| Renderer (`engine/diceRenderer.ts`) | Canvas, physics, camera, lighting, theme, lifecycle | Decide modifiers, visibility or history |
| Store (`stores/diceTray.svelte.ts`) | Open/mode/geometry, settings, current roll, cancellation | Persist content to project files directly |
| Integration (`integration/sessionHistory.svelte.ts`) | Session history and the future project-file sink | Become a database |

Character modifiers come from Catalyst's rules engine and arrive as part of the expression. The renderer never sees them.

## Source layout

```text
src/lib/dice/
├── index.ts                     public API for other modules
├── components/
│   ├── DiceTray.svelte          floating panel, drag/resize, compact/expanded
│   ├── DiceStage.svelte         owns one renderer while mounted
│   ├── DiceResult.svelte        total and per-die values
│   ├── DiceHistory.svelte       session roll history
│   ├── DiceSettings.svelte      animation and low-performance toggles
│   └── DieGlyph.svelte          die-shaped button icons
├── engine/                      engine and renderer adapter
├── integration/                 session history
├── stores/                      global tray store
├── types/                       shared types, Dice Box declaration
└── tests/                       unit and component tests
```

The spec's suggested `gmConsole.ts`, `encounters.ts` and `characters.ts` integration files are not created yet, because those modules do not exist in Catalyst. They call `diceTray.roll()` when they land (doc 023).

## Resolution

- **Expanded mode with animation on and WebGL available:** Dice Box simulates the throw; each die's settled upward face is its value. The engine validates the count and range of every value before building a record.
- **Otherwise** (compact mode, animation off, reduced motion by default, WebGL or Dice Box unavailable): the engine draws each die from `crypto.getRandomValues` with rejection sampling (no modulo bias), and the tray shows the values without animation.

Each record states which path produced it (`resolution: "physics" | "random"`). The tray never animates one value and records another.

## Renderer lifecycle and performance

- **Lazy:** Dice Box (about 2.5 MB of JavaScript including Babylon.js) is a dynamic import and its own chunk. It loads the first time the expanded tray shows the dice surface.
- **One instance:** `DiceStage` creates exactly one renderer while mounted. Repeated `init()` calls are no-ops, so no duplicate canvases appear.
- **Released on close:** closing the tray, switching to compact mode, or turning animation off unmounts `DiceStage`, which calls `dispose()`. That terminates the physics and render workers (freeing the WebGL context), drops the resize hook and removes the canvas. Reopening starts a fresh renderer (around a second).
- **Idle:** Dice Box's physics worker sleeps once dice settle. Rendering runs in an OffscreenCanvas worker, off the UI thread.
- **Resize:** a `ResizeObserver` on the dice surface drives Dice Box's resize handler directly. Dice Box's own `window` resize listener is never registered.
- **Reduced motion:** `prefers-reduced-motion: reduce` turns animation off by default; users can override it in tray settings.
- **Low-performance mode:** disables shadows.
- **WebGL check:** done before loading Dice Box. On failure the tray says so and keeps rolling with instant results.

## Cancellation and failure

A roll in progress is abandoned, and nothing is recorded, when the user presses Clear, closes the tray, switches mode, or starts a new roll. A roll that has not settled after 20 seconds fails with a visible error and records nothing. Invalid or not-yet-supported notation is rejected before any dice are thrown.

## Offline operation and CSP

All runtime assets are bundled (doc 021). The production CSP in `src-tauri/tauri.conf.json` gained only what Dice Box needs locally:

| Directive | Added | Why |
| --- | --- | --- |
| `script-src` | `'wasm-unsafe-eval'` | Compile the Ammo.js physics WebAssembly |
| `worker-src` | `'self' blob:` | Dice Box inlines its physics and render workers as blob URLs |
| `img-src` | `blob:` | Babylon.js texture decoding |
| `connect-src` | `'self'` | Load the bundled theme and WASM from the app origin |

No remote origin was added.

## Storage

Tray preferences use `localStorage` (per-user conveniences). Roll history is in memory for the running session in Phase 1; Catalyst has no campaign session model yet, and the spec does not require storing temporary rolls. Phase 3 adds a file-authoritative sink (doc 022). No database is introduced.
