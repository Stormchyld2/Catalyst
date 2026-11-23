# 022 — Dice Result Protocol

**Status:** Record shape, roll kinds, criticals and campaign session files implemented; network transport planned
**Date:** 2026-11-23
**Parent:** 020-DICE-ROLLER-ARCHITECTURE.md

## Roll record

Defined in `src/lib/dice/types/dice.ts` as `RollRecord`. A record is created only after a roll resolves successfully.

```json
{
  "id": "roll-3b9b6d2e-6f1c-4a7e-9c55-0f3d1f2a7b10",
  "sessionId": null,
  "expression": "2d20kh1+5",
  "kind": "advantage",
  "dice": [
    { "type": "d20", "value": 20 },
    { "type": "d20", "value": 7, "dropped": true }
  ],
  "modifier": 5,
  "natural": 20,
  "total": 25,
  "critical": "success",
  "visibility": "public",
  "source": "character",
  "resolution": "physics",
  "timestamp": "2026-11-23T13:41:42.000Z",
  "characterId": "character-…",
  "purpose": "attack",
  "label": "Mira — Longsword attack"
}
```

| Field | Meaning |
| --- | --- |
| `id` | `roll-` plus a UUID v4. Roll IDs are event IDs, not project entity IDs, so they don't use the stable-ID prefixes from doc 003. |
| `sessionId` | The active campaign session, or `null` for a temporary roll |
| `expression` | Canonical notation, in written order (`2d20+1d6+3`, `4d6kh3`, `2d20kh1+5`) |
| `kind` | `sum`, `advantage` (`2d20kh1`) or `disadvantage` (`2d20kl1`) |
| `dice` | Each die in term order; every `value` is an integer from 1 to the die's sides. `dropped: true` marks dice that a keep rule discarded. d100 is one value from 1 to 100; a d10 reports 10 for its 0 face. |
| `modifier` | Flat modifier, kept separate from the dice |
| `natural` | Sum of the kept dice before the modifier |
| `total` | `natural + modifier` |
| `visibility` | `public`, `gm-private` or `player-private` |
| `source` | `dice-tray`, `gm-console`, `encounter`, `character`, `campaign` |
| `resolution` | `physics` (settled 3D dice) or `random` (crypto source, no animation) |
| `timestamp` | ISO-8601 time of resolution |
| `critical` | `success` (natural 20) or `failure` (natural 1), only when exactly one d20 is counted |
| `playerId`, `characterId`, `npcId`, `encounterId`, `purpose`, `label` | Optional context, e.g. `purpose: "initiative"` |

A plain `2d20` is a **sum**. Advantage and disadvantage are separate roll kinds with their own `kind` value and a dropped die; they are never confused with the sum. Dice Box reports settled dice grouped by type, so the engine assigns them to the written terms type by type before applying keep rules.

## Rules

1. **Record only after resolution.** Cleared, closed, superseded, timed-out and invalid rolls create no record.
2. **Display what was recorded.** The tray shows the record's values. It never animates one outcome and records another, and it never changes a settled value.
3. **Validate before recording.** `validateDice` checks the count of each die type and every value's range.
4. **Unbiased randomness** for non-physics rolls: `crypto.getRandomValues` with rejection sampling.

## Campaign session files

History is kept in memory (`RollHistory`, newest first, 500 entries) for the running app session. While a campaign session is active (`diceTray.startCampaignSession(projectRoot, title)` or `resumeCampaignSession`), `CampaignSessionSink` also appends each of that session's records to ordinary files in the project, through Rust commands in `src-tauri/src/dice_sessions.rs`:

```text
<project>/Notes/Campaign Sessions/<session-id>/session.json   { id, title, startedAt }
<project>/Notes/Campaign Sessions/<session-id>/rolls.jsonl    one RollRecord per line
```

- **File-authoritative, no database** (`AGENTS.md`): append-only JSON Lines, readable without Catalyst; reopening a session reads the file back into history.
- **Safe paths:** session IDs must be `session-` plus a UUID v4, so no path can escape the folder. `session.json` is written to a temporary file and renamed; each appended line is synced to disk.
- **Validated on write:** the Rust side rejects a record whose ID, session, die faces, totals, enum values or timestamp are invalid. Damaged lines are skipped on read and counted (`skippedLines`), never fatal.
- **Location needs sign-off:** `Notes/` is one of the canonical roots; a dedicated `Campaign Sessions/` root may suit the Campaign Studio better once it exists. Moving it is a constant in `dice_sessions.rs`.
- Rolls with no active session are not stored. A failed write leaves the roll in memory and shows "Roll not saved to the session file".

Tauri commands: `start_dice_session`, `append_dice_roll`, `get_dice_session_history`, `list_dice_sessions`.

`RollHistory.filter` supports session, die type, player, character, NPC and purpose. The tray's history popover filters by die type.

## Multiplayer authority (Phase 4)

Two clients running independent physics will not produce the same outcome, so in multiplayer:

1. A client sends a **roll request** (expression, visibility, context) to the host.
2. The **host** resolves the roll with the unbiased random source and records it. The host is authoritative.
3. The host sends the **record** only to clients allowed to see it (rules below).
4. Clients that receive a public record may animate it, **only if** the renderer can land on the recorded values. Dice Box 1.1.4 cannot force outcomes (doc 021), so until a presentation adapter that can is found (for example, investigating `@3d-dice/dice-box-threejs`), receiving clients show the result without a physics animation that might disagree with it.

Message shapes: `roll.request` → `{ requestId, request: RollRequest }`; `roll.result` → `{ requestId, record: RollRecord }`; `roll.rejected` → `{ requestId, reason }`.

## Visibility

| Visibility | Who receives the record |
| --- | --- |
| `public` | GM and approved player displays |
| `gm-private` | GM only |
| `player-private` | The rolling player and the GM, where the game rules allow |

Visibility is enforced by **not sending** the record (or any animation of it) to unauthorized clients. Hiding UI elements is not enough. A private roll never reaches the public display. `integration/visibility.ts` implements these rules (`canView(record, viewer)`, `visibleTo(records, viewer)`) for any future sender; `diceTray.onRecorded` is the hook a player display would subscribe to.
