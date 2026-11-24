# 034 — GM Screen

**Status:** Proposed. Nothing here is built yet. Code starts only after Jason approves this doc.
**Date:** 2026-11-24
**Depends on:** 020 (campaign tree), 025 (design system), 027/030 (dice), 033 (soundboard).

## 1. What it is

One screen the GM keeps open during play. It gathers the at-the-table tools into fixed panels so
nothing needs a workspace switch mid-session. It is the "GM Console" that doc 030 already expects:
it calls the dice and soundboard APIs and owns no engine of its own.

- New workspace in the rail's **At the table** group, id `gm-screen`, label "GM Screen", next to
  Soundboard.
- Offline only. No network, no remote assets, no new dependencies.
- Obsidian + Frost Blue, using the existing `--cat-*` tokens. No new palette.

## 2. Layout and panel model

A CSS grid of panels. Every panel has the same shape:

```text
┌ panel ───────────────────────┐
│ header (title, actions)      │  fixed
│ frozen strip (optional)      │  fixed: never scrolls
├──────────────────────────────┤
│ body                         │  scrolls with the mouse wheel,
│ …                            │  no visible scrollbar
└──────────────────────────────┘
```

Default layout (16:9 screen, three columns):

```text
┌ Soundboard ─────┬ Dice ───────────┬ Reference cards ─────────┐
│ [Master volume] │ quick-roll pads │ [card tabs + add]        │
│ pad pad pad     │ [Open tray]     │ stat block / encounter / │
│ pad pad pad     ├ Rolls ──────────┤ session / note …         │
│ …               │ [Current roll]  │                          │
│                 │ history …       │                          │
├ Initiative ─────┴─────────────────┼ Notes ───────────────────┤
│ turn order, HP, conditions        │ session scratchpad       │
└───────────────────────────────────┴──────────────────────────┘
```

Phase 1 ships this one layout with show/hide per panel. Moving and resizing panels, and saved
layout presets, come in phase 3 (section 8), so the first version stays small.

### Scrolling (all panels)

One shared CSS class, added once to the existing Catalyst stylesheet:

```css
.cat-quiet-scroll { overflow-y: auto; scrollbar-width: none; overscroll-behavior: contain; }
.cat-quiet-scroll::-webkit-scrollbar { display: none; }
```

- The wheel and trackpad scroll normally; the scrollbar is never drawn (WebView2 honours both rules).
- Frozen strips are plain elements above the scroll body, not `position: sticky`, so they cannot drift.
- Accessibility (WCAG 2.2 AA): with no scrollbar, a soft fade at the top or bottom edge shows that
  more content exists. Each body is keyboard-focusable (`tabindex="0"`) so arrow keys, Page Up/Down
  and Home/End scroll it, with a visible focus ring. Fades are static under reduced motion.

## 3. Soundboard panel

Reuses doc 033. No second audio store and no copy of `soundboard.json`.

- **Frozen strip:** master fader, the three channel faders (Music, Ambience, Effects) and **Stop all**.
  These write the same `master_volume` and `channels` values the Soundboard workspace writes.
- **Body:** a compact grid of the pads in `soundboard.json`, same play/stop behaviour and channel
  rules. Missing-file pads show as missing. Editing pads (import, rename, channel) stays in the
  Soundboard workspace; the panel has a link to it.
- **Required change:** `SoundboardWorkspace.svelte` creates its own `SoundboardEngine` and disposes
  it on unmount, so music stops when you leave the workspace. The engine moves to one
  campaign-scoped instance in `src/lib/soundboard/` that both views use, so audio keeps playing
  across the GM Screen and the Soundboard. It is disposed when the campaign closes.

## 4. Dice panel and Rolls box

Reuses doc 027/030 through `import { diceTray } from "$lib/dice"`. The 3D table has one renderer and
Dice Box rebuilds its world on resize, so the GM Screen does **not** embed a second tray.

- **Dice panel:** quick-roll buttons (d4 to d100, ADV/DIS, public/GM-private) that call
  `diceTray.roll(...)`, plus an **Open tray** button that opens the existing floating tray centred
  over the screen. That is the "modal from a button": it is the approved tray, not a new component,
  and it stays nonblocking.
- **Rolls box:** reads `diceTray.history`.
  - Frozen strip: the **current roll** (label, dice, total, crit markers). It stays put until the
    next roll replaces it.
  - Body: earlier rolls, newest first, same quiet scrolling.
  - Rolls still save to the active campaign session file (doc 029). The box stores nothing itself.

## 5. Reference cards

A tabbed deck of cards the GM can add, remove, reorder and swap. A card is a **reference by stable
ID** to an existing campaign record; the content stays in its own file and is never copied.

| Card | Source | Shows |
|---|---|---|
| Creature / stat block | `creature-profile` record (Bestiary, Custom Creatures) | Overview fields, then the "Statistics Reference" section |
| NPC | `npc-profile` record | Disposition, motivation, secrets, statistics reference |
| Encounter | `encounter-card` record | Type, difficulty, participants (each opens its own card), terrain, tactics; **Roll initiative** sends participants to the tracker |
| Current session | Active session plan (`session-plan`) + dice session | Session number/date, strong start, scenes, secrets, NPCs, encounters |
| Note | Any Markdown record, e.g. `Game Master/Screen References/` | Rendered Markdown |

- Cards render with the shared Markdown renderer and link resolver, read-only, with an "Open in
  wiki" action for editing. Records marked GM-only stay GM-only (no player view yet).
- A missing or deleted record shows as an unresolved card; it is never silently dropped.
- There is no structured stat-block model yet (rules statistics are optional text on creature
  records). Cards show what the record holds. A structured block waits for the rules engine.

## 6. Proposed additions ("what else?")

Ranked by how much they help at the table. Each is a panel or a card type on the same model.

1. **Initiative / combat tracker.** Turn order, round counter, current-turn highlight, HP with
   quick +/- damage and heal, conditions with durations that count down each round. Rolls initiative
   through `initiativeRoll`. Combatants reference character, NPC or creature records by ID.
2. **Session notes / scratchpad.** Fast notes that save into the session's log
   (`Sessions/Session Logs/`), so nothing lives only in the screen.
3. **Random tables.** Roll on the GM library tables (`Game Master/Random Tables`, plus any campaign
   `Encounters/Random Tables`) with the dice engine; the result lands in the Rolls box.
4. **Conditions and rules quick reference.** Cards from `Rules & Mechanics/Conditions` and
   `House Rules`.
5. **Party roster.** One line per PC: name, player, AC/HP/passive scores where the record has them,
   linked to Character Builder records when that lands.
6. **Clocks and timers.** Progress clocks (4/6/8 segments), a real-time timer and torch/light
   durations in rounds or minutes.
7. **Handout / map display.** Show an image from `Game Master/Handouts` or `Maps`. A second-window
   player display can come later with doc 026 (LAN).
8. **Pinned wiki entries.** Same as Note cards, with one-click pinning from the wiki.
9. **Layout presets per campaign.** "Combat", "Social", "Exploration" layouts, switchable in one click.

## 7. Storage (file-authoritative)

One small file per campaign holds only the screen's own state:

```text
<Campaign>/Game Master/gm-screen.json
```

```json
{
  "format": "catalyst-gm-screen",
  "version": 1,
  "active_layout": "default",
  "layouts": {
    "default": {
      "panels": [
        { "id": "soundboard", "visible": true },
        { "id": "dice", "visible": true },
        { "id": "rolls", "visible": true },
        { "id": "cards", "visible": true, "cards": ["crt-<uuid>", "enc-<uuid>", "session"] },
        { "id": "initiative", "visible": true },
        { "id": "notes", "visible": true }
      ]
    }
  }
}
```

- Cards are stored as record IDs, never paths or titles. `session` means "the active session".
- Not stored here: pad levels (`soundboard.json`), rolls (dice session files), notes (session log),
  card content (the records). One value, one file.
- Combat state (phase 2) goes in `Sessions/Campaign State/combat.json` so it belongs to the
  campaign's play state and survives a restart mid-fight.
- Written atomically by a Rust command (`load_gm_screen` / `save_gm_screen`), path-guarded to the
  campaign root. Unknown fields are kept; a newer version is refused rather than overwritten.
  Deleting the file only resets the layout.
- Book-layout (legacy) projects don't get the GM Screen.

## 8. Phased delivery

Each phase is one PR, with tests, after the previous one merges.

| Phase | Scope |
|---|---|
| 1 | Workspace and grid, quiet-scroll class, Soundboard panel (shared engine), Dice panel + Rolls box, `gm-screen.json` with show/hide |
| 2 | Reference cards (creature, NPC, encounter, session, note), initiative/combat tracker with HP and conditions |
| 3 | Move/resize panels, layout presets, scratchpad, random tables, conditions cards |
| 4 | Party roster, clocks/timers, handout display; player display with doc 026 |

Phase 1 depends on the soundboard (PR #9) and dice (PR #2) being on the target branch
(`test/combined-build-v3` today) and on the rail rework for the At the table group.

## 9. Open questions for Jason

1. **Scrollbars:** hide them only inside the GM Screen, or across the whole app?
2. **Pads:** show every pad, or let you pick a subset of pads for the screen?
3. **Dice:** is "Open tray" (the existing floating tray, centred) enough, or do you want the tray
   docked inside the screen? Docking moves the one 3D table into the panel; it cannot be two copies.
4. **Additions:** which of section 6 do you want, and in what order? The plan assumes the tracker and
   cards first.
5. **Combat tracker HP:** track HP for PCs too, or only monsters and NPCs?
6. **Layout freedom:** is a fixed layout with show/hide fine for phase 1?
