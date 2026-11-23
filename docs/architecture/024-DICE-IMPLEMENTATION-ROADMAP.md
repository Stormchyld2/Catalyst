# 024 — Dice Implementation Roadmap

**Status:** Phase 1 complete, pending review
**Date:** 2026-11-23
**Parent:** 020-DICE-ROLLER-ARCHITECTURE.md

## Phase 1 — 2d20 proof of concept (this change)

Done:

- Global floating, nonblocking tray: rail button, View menu item, command palette, `Mod+Shift+D`.
- Drag, resize, and remembered position, size and mode. Compact and expanded modes.
- Dice Box physics roll of `Nd20` (default `2d20`), with per-die values and total.
- Roll, Reroll, Clear, Close. Session history that survives closing the tray.
- Lazy renderer, one instance, released on close or compact. ResizeObserver resizing. Reduced-motion default. Low-performance mode. WebGL fallback to instant rolls.
- Bundled offline assets, CSP updated, no remote requests.
- Unit and component tests (71 new). Real-browser verification of physics rolls under the production CSP with all non-local requests blocked.

**Exit criteria:** a user can visibly roll two D20s offline, see both values, and see the correct total. Met in Chromium with SwiftShader WebGL. **Still to check by hand on Windows:** the Tauri/WebView2 window, a real GPU, high-DPI, and the network physically disabled.

## Phase 2 — Complete dice tray

- Enable d4, d6, d8, d12 after checking the settled-face mapping; then d10 and d100 (Dice Box renders d100 as a d100+d10 pair).
- Mixed expressions and modifiers in the tray. An expression field.
- Evaluate `@3d-dice/dice-roller-parser` for advanced notation (keep/drop, exploding).
- Advantage/disadvantage as distinct roll kinds.
- Additional themes (dark leather, walnut, stone, neutral tabletop); dice body and number colour; dice size.
- Bundled, redistributable rolling sounds with on/off and volume.

## Phase 3 — GM Console integration

- GM quick rolls, encounter rolls, character action rolls, initiative (doc 023).
- Public and private modes.
- File-authoritative session history via `RollHistorySink` (doc 022).

## Phase 4 — Player display and LAN readiness

- Host-authoritative resolution and the roll event protocol (doc 022).
- Player-safe visibility: private rolls are never transmitted to unauthorized clients.
- Public animation synchronization, which needs a renderer that can land on predetermined values.

Phases 1 to 3 do not wait for LAN multiplayer.

## Phase 5 — Enhancements

Critical-success effects, presets and favourites, dice sets, accessibility passes, touch gestures, Android performance.

## Known limitations

- Dice Box 1.1.4 has no `destroy()`; the adapter terminates its workers instead (doc 021).
- Reopening the expanded tray re-initializes the 3D renderer (about a second), in exchange for releasing GPU memory whenever the tray is closed.
