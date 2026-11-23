# 023 — Dice GM Console Integration

**Status:** Planned (Phase 3). The shared entry point exists today.
**Date:** 2026-11-23
**Parent:** 020-DICE-ROLLER-ARCHITECTURE.md

## Entry point

Every RPG module uses the same engine and the same floating tray through `src/lib/dice/index.ts`:

```ts
import { diceTray } from "$lib/dice";

diceTray.open();                         // show the tray (keeps its last mode)
const record = await diceTray.roll({
  expression: "1d20+5",                  // modifiers computed by the rules engine
  source: "gm-console",
  visibility: "gm-private",
  sessionId,
  characterId,
  purpose: "attack",
});
// record is null if the roll was cancelled or failed. It is never a guess.
```

A module must not create its own renderer, canvas or random source.

## GM Console controls

| Control | Implementation |
| --- | --- |
| Quick roll | Preset buttons that call `diceTray.roll` |
| Custom expression | Text field parsed with `parseDiceNotation`; errors shown inline |
| Advantage / disadvantage | New roll kinds (`2d20kh1` / `2d20kl1`) once the parser supports keep-highest/lowest; recorded separately from a `2d20` sum |
| Public roll | `visibility: "public"` |
| Private GM roll | `visibility: "gm-private"`; never sent to player displays |
| Roll history | `diceTray.history`, filtered by session |
| Clear tray | `diceTray.clearTray()` |

## Encounter and character rolls

Encounters (initiative, attacks, damage, saves, skill checks, random tables) and characters (ability scores, skills, saves, weapons, spells, custom actions) build the expression and context from their own data, then call `diceTray.roll`. The user never has to retype notation. Character modifiers are applied by Catalyst's rules engine into the expression. The renderer never knows about them.

## Phase 1 limits

- The tray rolls `Nd20` (1 to 20 dice). Other dice and modifiers are parsed but rejected by `PHASE_1_CAPABILITIES` until Phase 2 validates them end to end (d10 and percentile face mapping in particular).
- No GM Console, encounter or character module exists in Catalyst yet. When one lands, it imports the entry point above.
