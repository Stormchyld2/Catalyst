# 031 — Dice Implementation Roadmap

**Status:** Phases 1–3 complete, pending review
**Date:** 2026-11-23
**Parent:** 027-DICE-ROLLER-ARCHITECTURE.md

## Phase 1 — 2d20 proof of concept (done)

Done:

- Global floating, nonblocking tray: rail button, View menu item, command palette, `Mod+Shift+D`.
- Drag, resize, and remembered position, size and mode. Compact and expanded modes.
- Dice Box physics roll of `Nd20` (default `2d20`), with per-die values and total.
- Roll, Reroll, Clear, Close. Session history that survives closing the tray.
- Lazy renderer, one instance, released on close or compact. ResizeObserver resizing. Reduced-motion default. Low-performance mode. WebGL fallback to instant rolls.
- Bundled offline assets, CSP updated, no remote requests.
- Unit and component tests (71 new). Real-browser verification of physics rolls under the production CSP with all non-local requests blocked.

**Exit criteria:** a user can visibly roll two D20s offline, see both values, and see the correct total. Met in Chromium with SwiftShader WebGL. **Still to check by hand on Windows:** the Tauri/WebView2 window, a real GPU, high-DPI, and the network physically disabled.

## Phase 2 — Complete dice tray (done)

- All dice with physics: d4, d6, d8, d10, d12, d20, d100 (each verified in Chromium against the settled faces).
- Mixed expressions, modifiers, keep-highest/lowest (`4d6kh3`), typed expression entry; +/- for dice and the modifier.
- Advantage/disadvantage as distinct roll kinds; natural 20 and natural 1 shown as criticals.
- Compact result panel with an amber-gold total, readable at small sizes.
- Dice colour, size, mat surface, roll speed, animation, low-performance mode; synthesized sound with volume.
- Muted Catalyst wordmark printed into the mat.
- Custom parser kept instead of `@3d-dice/dice-roller-parser`: the supported notation is small and fully tested.

## Phase 3 — GM integration (done, API only)

- Roll request builders for initiative, attacks, damage, saves, skills, ability checks and encounter rolls (doc 030).
- Public, GM-private and player-private rolls; visibility rules for future senders.
- File-authoritative campaign session history (doc 029).
- The GM Console, characters and encounters modules themselves do not exist yet.

## Phase 4 — Player display and LAN readiness

- Host-authoritative resolution and the roll event protocol (doc 029).
- Player-safe visibility: private rolls are never transmitted to unauthorized clients.
- Public animation synchronization, which needs a renderer that can land on predetermined values.

Phases 1 to 3 do not wait for LAN multiplayer.

## Phase 5 — Enhancements

Critical-success effects, presets and favourites, dice sets, accessibility passes, touch gestures, Android performance.

## Known limitations

- Dice Box 1.1.4 has no `destroy()`; the adapter terminates its workers instead (doc 028).
- Dice Box cannot land dice on chosen values, so multiplayer clients cannot replay a host's roll as physics (doc 029).
- Reopening the expanded tray, or changing dice colour, size, surface or speed, restarts the 3D renderer (about a second).
- Resizing the tray by hand clears settled dice from the mat (Dice Box rebuilds its world); the result panel keeps the values.
- The number colour on the dice comes from the Dice Box theme and is not separately configurable.
- The d4 is small at the default dice size; the large size helps.
- Still to check by hand on Windows: the Tauri/WebView2 window, a real GPU, high-DPI, sound output, and the network physically disabled.
