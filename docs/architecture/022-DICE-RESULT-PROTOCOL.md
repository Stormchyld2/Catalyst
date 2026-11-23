# 022 — Dice Result Protocol

**Status:** Record shape implemented (Phase 1); persistence and network transport planned
**Date:** 2026-11-23
**Parent:** 020-DICE-ROLLER-ARCHITECTURE.md

## Roll record

Defined in `src/lib/dice/types/dice.ts` as `RollRecord`. A record is created only after a roll resolves successfully.

```json
{
  "id": "roll-3b9b6d2e-6f1c-4a7e-9c55-0f3d1f2a7b10",
  "sessionId": null,
  "expression": "2d20",
  "dice": [
    { "type": "d20", "value": 14 },
    { "type": "d20", "value": 7 }
  ],
  "modifier": 0,
  "natural": 21,
  "total": 21,
  "visibility": "public",
  "source": "dice-tray",
  "resolution": "physics",
  "timestamp": "2026-11-23T13:41:42.000Z"
}
```

| Field | Meaning |
| --- | --- |
| `id` | `roll-` plus a UUID v4. Roll IDs are event IDs, not project entity IDs, so they don't use the stable-ID prefixes from doc 003. |
| `sessionId` | The active campaign session, or `null` for a temporary roll |
| `expression` | Canonical notation (`2d20`, `1d4+2d6+3`) |
| `dice` | Each die in throw order; every `value` is an integer from 1 to the die's sides |
| `modifier` | Flat modifier, kept separate from the dice |
| `natural` | Sum of the dice before the modifier |
| `total` | `natural + modifier` |
| `visibility` | `public`, `gm-private` or `player-private` |
| `source` | `dice-tray`, `gm-console`, `encounter`, `character`, `campaign` |
| `resolution` | `physics` (settled 3D dice) or `random` (crypto source, no animation) |
| `timestamp` | ISO-8601 time of resolution |
| `characterId`, `encounterId`, `purpose` | Optional context, e.g. `purpose: "initiative"` |

A plain `2d20` is a **sum**. Advantage and disadvantage are separate roll kinds (keep highest or lowest) and will carry their own field. They must never be confused with the sum.

## Rules

1. **Record only after resolution.** Cleared, closed, superseded, timed-out and invalid rolls create no record.
2. **Display what was recorded.** The tray shows the record's values. It never animates one outcome and records another, and it never changes a settled value.
3. **Validate before recording.** `validateDice` checks the count of each die type and every value's range.
4. **Unbiased randomness** for non-physics rolls: `crypto.getRandomValues` with rejection sampling.

## Session history persistence (Phase 3)

Phase 1 keeps history in memory (`RollHistory`, newest first, 500 entries) for the running app session. When campaign sessions exist, a `RollHistorySink` appends each record through the project services to an ordinary file inside the campaign project, for example:

```text
<campaign project>/sessions/<session-id>/rolls.jsonl
```

One JSON record per line, append-only, readable without Catalyst. This follows the storage rules in `AGENTS.md`: the files are authoritative, any index of them is rebuildable, and there is no new database. Rolls with no active session are not stored.

History filters (session, player, character, die type, purpose) work over the records. `RollHistory.filter` already supports session, die type, character and purpose; player arrives with player identity in Phase 4.

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

Visibility is enforced by **not sending** the record (or any animation of it) to unauthorized clients. Hiding UI elements is not enough. A private roll never reaches the public display.
