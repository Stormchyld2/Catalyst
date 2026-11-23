# 030 — Dice GM Console Integration

**Status:** Integration API implemented. The GM Console, Character Studio and Adventure Studio (which owns encounters) do not exist in Catalyst yet; they call this API when they land.
**Date:** 2026-11-23
**Parent:** 027-DICE-ROLLER-ARCHITECTURE.md

## Entry point

Every RPG module uses the same engine and the same floating tray through `src/lib/dice/index.ts`:

```ts
import { diceTray, attackRoll, damageRoll } from "$lib/dice";

const goblin = { kind: "npc", id: "npc-…", name: "Goblin" } as const;
const hit = await diceTray.roll(attackRoll(goblin, { bonus: 4, mode: "advantage" }), { show: true });
if (hit?.critical === "success") {
  await diceTray.roll(damageRoll(goblin, "1d6+2", { critical: true }), { show: true }); // rolls 2d6+2
}
// A result is null if the roll was cancelled or failed. It is never a guess.
```

`{ show: true }` opens the tray (expanded, if animation is on) and waits up to 8 seconds for the 3D table before rolling; if 3D is unavailable the roll resolves instantly from the unbiased random source. Rolls made while a campaign session is active are saved to that session's file automatically (doc 029).

A module must not create its own renderer, canvas or random source.

## Roll request builders (`integration/rollRequests.ts`)

| Builder | Expression | Purpose |
| --- | --- | --- |
| `initiativeRoll(actor, { bonus })` | `1d20+bonus` | `initiative` |
| `attackRoll(actor, { bonus, weapon })` | `1d20+bonus` | `attack` |
| `savingThrow(actor, ability, { bonus })` | `1d20+bonus` | `saving-throw` |
| `skillCheck(actor, skill, { bonus })` | `1d20+bonus` | `skill-check` |
| `abilityCheck(actor, ability, { bonus })` | `1d20+bonus` | `ability-check` |
| `damageRoll(actor, "1d8+3", { critical })` | dice doubled on a critical, modifier not | `damage` |
| `encounterRoll(encounterId, "1d100", purpose)` | any expression | caller's purpose |

Every d20 builder takes `mode: "advantage" | "disadvantage"`, which becomes `2d20kh1` / `2d20kl1` and is recorded as its own `kind`. Actors are `{ kind: "character" | "npc", id, name?, playerId? }`. Labels read "Name — Purpose" in history.

**Visibility defaults:** character rolls are public, NPC rolls and encounter rolls are GM-private. Pass `visibility` to override (including `player-private` with the character's `playerId`).

## GM Console controls

| Control | Implementation |
| --- | --- |
| Quick roll | Preset buttons that call `diceTray.roll` with a builder |
| Custom expression | The tray's expression field, or `diceTray.engine.parse` for inline validation |
| Advantage / disadvantage | `mode` on any d20 builder; ADV/DIS buttons in the tray |
| Public / private GM roll | `visibility`; the eye button in the tray toggles public and GM-private |
| Roll history | `diceTray.history`, filterable by session, player, character, NPC, die type and purpose |
| Campaign session | `diceTray.startCampaignSession(projectRoot, title)`, `resumeCampaignSession`, `endCampaignSession` |
| Player display feed | `diceTray.onRecorded(listener)` plus `canView` / `visibleTo` before sending anything |
| Clear tray | `diceTray.clearTray()` |

## Waiting on other modules

- **GM Console:** no UI yet. The controls above are the API it will use.
- **Characters and NPCs:** no character or NPC model yet, so modifiers are passed in as `bonus`. Catalyst's rules engine will compute them.
- **Encounters:** no encounter model yet; `encounterId` is carried through records and filters.
- **Player display / LAN:** visibility rules and the record feed exist; transport is Phase 4.
