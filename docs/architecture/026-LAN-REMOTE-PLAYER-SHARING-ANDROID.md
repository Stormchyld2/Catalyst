# 026 — Catalyst LAN Remote, Player Sharing & Android Companion Architecture

**Status:** Approved product direction; architecture target, NOT implemented or tested  
**Date:** 2026-11-23  
**Applies to:** Catalyst GM Desktop; Catalyst Remote (GM mobile browser); Catalyst Character Studio Windows; future Catalyst Character Companion Android; future player-facing web fallback.  
**Related documents:** [021 Character Studio](021-CHARACTER-STUDIO-ARCHITECTURE.md), [023 Shared Engines](023-CATALYST-SHARED-AND-MODULE-ENGINES.md), [024 Engine Ownership](024-ENGINE-IMPLEMENTATION-OWNERSHIP-MATRIX.md), [025 Design System](025-CATALYST-DESIGN-SYSTEM.md), [022 Sourcebooks](022-RPG-SOURCEBOOK-LIBRARY-ARCHITECTURE.md).

## 1. Product decisions

1. **Windows Catalyst Character Studio is completed first.** It owns the character record, guided/manual builder, in-play sheet and character-specific support engines in the separate `Stormchyld2/Catalyst-Character-Builder` repository.
2. **An Android Character Companion is the preferred long-term player experience.** It reuses portable character domain/rules engines, while implementing an Android-optimized interface and local storage adapter; it is not merely a remote desktop screen.
3. **Players have access only to explicitly assigned character data and their permitted inventory, resources, spells, advancement, notes and roll actions.** GM secrets and unrelated player records never leave the GM authority boundary.
4. **Catalyst Remote** is a distinct, responsive mobile browser interface aimed initially at GMs. It starts with the Campaign Soundboard and later gains encounter controls, dice, references, maps/scenes and party tools.
5. **Player Sharing Service** and **GM Control Service** are *logically separate access domains* on the same reusable LAN transport/server engine. Do not assume two independently administered servers, two permanent listeners or two engines are necessary. Separate endpoints, principals, scopes, serializers and audit trails; a second process/port is an implementation option only after security testing demonstrates a need.
6. **Fully local/offline-first:** Wi-Fi or a PC-hosted hotspot is sufficient; the internet, public cloud, account registration and external discovery are not prerequisites. Android may edit an authorized local character while disconnected, with changes reconciled on later connection.
7. GM desktop is the **authority for the hosted campaign**. The Android app's local character copy is a permitted offline working replica, not a second uncoordinated campaign source of truth. A player may also keep entirely independent personal characters that are not campaign-bound.
8. All interfaces follow the [Catalyst Obsidian & Frost Blue visual system](025-CATALYST-DESIGN-SYSTEM.md); local character records remain portable user-owned files.

## 2. System topology

```text
                            Local Wi-Fi / PC hotspot, no internet
  ┌─────────────────────────────────────────────────────────────────┐
  │ Catalyst GM Desktop (Windows)                                    │
  │                                                                 │
  │  Authoritative campaign file/document service + stable IDs      │
  │  Rules, dice, session/event, permissions, sourcebook, audio      │
  │                       |                                         │
  │           S25: LAN Transport + Presence + Session Auth          │
  │               /                              \                  │
  │     GM Control Service                 Player Sharing Service   │
  │   GM scoped routes/events             Player scoped routes       │
  │     soundboard first                  character + inventory     │
  └─────────────┬────────────────────────────────┬───────────────────┘
                |                                |
       Paired GM mobile browser           Paired Android app
         Catalyst Remote               Character Companion
       controls; GM-safe data            offline local replica
                                                |
                                          later sync/reconcile
```

The GM browser remote and player Android app are clients of the host's **same shared permission/transport infrastructure**, not independent LAN servers in the character builder or audio module.

## 3. GM mobile browser remote (Catalyst Remote)

**MVP scope — Campaign Soundboard:** authenticate/pair via desktop-displayed time-limited QR code or short code, view music/ambience/effect channels, trigger local audio, select saved scenes, crossfade, adjust per-channel/master volume, pause/mute, and receive live playback state. Actual audio playback stays on the GM PC. Never upload source audio or expose unrestricted filesystem paths to the browser.

**Expandable remote capabilities (after soundboard):**
- encounter/initiative view and GM-authorized next-turn, conditions and event commands;
- shared dice requests and roll history through the single global dice outcome engine S07;
- scoped GM quick reference and sourcebook page citations/search (do not expose book file endpoints beyond authenticated authorized requests);
- map and scene controls, including player-display reveals;
- party overview, HP/resources and authorized updates;
- quick GM notes, scene cues, timers/handouts when underlying engines exist.

The remote is **not** another copy of the campaign application. All remote actions are typed commands accepted or rejected by GM-host services; no arbitrary file mutations, shell calls or automatic access to entire campaign storage. Start as responsive web UI; consider installable PWA after local HTTPS/session and cache lifecycle are tested.

## 4. Android Player Companion

**Goal:** full player-facing interactive character functionality equivalent to relevant Windows Character Studio features, optimized for phone screens, not a GM dashboard.

**Player can view or modify when permitted:**
- current owned character's identity, attributes, proficiency/skills, actions, spells/features, equipment, inventory, currency and notes;
- HP, conditions, consumables, ammunition, resources, spell slots, recovery/rests, prepared spells, carried/equipped items, and in-play action availability;
- locally initiated dice rolls bound to compatible campaign dice event contracts;
- character updates and advancement **subject to pack rules and GM approval policy**;
- read-only approved campaign references, handouts and optionally selected sourcebook citations (not the whole paid PDF library by default).

**Explicitly exclude:** GM-only notes, hidden maps, other players' private character fields, campaign-wide modification rights, unrestricted server file browsing, remote plugin execution and automated sharing of purchased books.

A personal standalone character has its own local record and can work with no host. When a character is linked to a GM campaign, use a stable campaign/character ID pairing and well-defined host permissions. A player can continue permitted edits offline; an offline copy is not guaranteed to include every host-only resource.

**Platform strategy:** keep pure TypeScript/Rust character rules/data logic separate from Svelte UI and Tauri-specific Windows shell. Target Android-compatible shared domain logic and an Android-specific storage/transport integration layer. Whether mobile UI uses Tauri Android or another wrapper is **deferred to a measured compatibility/build audit** after the Windows character app is validated. Never promise direct recompilation of Windows-only plugins into Android without checking dependencies.

## 5. Player Sharing Service and privacy model

The host exposes **logical** service boundaries:

- `/gm/*`: GM-only, paired GM-controller principals; controls approved host commands, including soundboard.
- `/player/*`: authenticated player principals, bound to assigned character IDs and requested capability scopes.
- `/session/*`: optional common endpoints for health, pairing and version negotiation; no sensitive plaintext until authorized.

Paths are illustrative, not frozen URL contracts. **All access must be verified on the host**, not by hiding forbidden information in client UI. Querying a different character ID, traversing a path, forging an event or submitting a modified payload cannot bypass ownership checks.

**Scoped permission model:**
- Identity: host GM, GM remote, player device, guest display; each device/session has independent revocable credentials.
- Resource: campaign ID, character ID, item ID, rules-pack/sourcebook ID, session ID.
- Actions: read sheet, edit notes, use consumable, change HP, modify inventory, request/add item, level up, roll, view reference, control audio, etc.
- Policy: per campaign and optionally per character. Default-deny GM secrets and other-player resources; optional GM approvals for inventory grants, magic items, advancement and other sensitive actions.
- Data shapes: scoped and explicitly projected response DTOs, not full record with a client-side hidden flag.
- Multi-character ownership: a player's account may be assigned several characters, but permissions evaluated independently on every requested character.
- Revocation: GM can unlink a device, revoke pairing or role, remove ownership and terminate sessions. Stolen token replay and reconnect must be constrained.

**Pairing and transport security:** prefer host-generated short-lived, single-use QR/pairing challenges; authenticate device identity after pairing, require authorization to assign a player to a specific character, support session expiration and revocation, rate-limit pairing attempts. Use encrypted local transport when feasible, pin/trust the host identity and document local HTTPS certificate/onboarding behavior. **Do not imply Wi-Fi alone provides confidentiality.** LAN exposure and firewall policy require explicit opt-in and device/interface scoping.

## 6. Authoritative changes, offline sync and conflicts

Personal standalone characters: Android local files are authoritative until deliberately imported/shared. Campaign-linked character: GM desktop remains the authoritative campaign ledger.

**Sync protocol minimum:**
1. On pairing, host sends a permitted character projection plus `characterId`, `campaignId`, schema version, rules-pack version, server revision and capability grants.
2. Player edits create local queued **commands** with globally unique operation IDs, known base revision, device identity, timestamps and field-specific payload (not arbitrary whole-file overwrites).
3. On reconnect, host validates credentials, ownership, allowed action, prerequisites and base revision; commits approved changes atomically and replies with accepted/rejected/pending-approval results and new revision.
4. Rejected commands stay visible to the player with an explanation; never silently discard or silently overwrite host state. Host remains the final arbiter for managed campaigns.
5. Idempotency keys prevent duplicate item consumption, level gains, resource changes or purchase transactions on reconnect.
6. Simple non-overlapping note edits may auto-merge when safely provable; **inventory quantities, currency, HP, spell slots, level-ups and campaign state require typed commands and conflict-aware reconciliation**. Do not use blind last-writer-wins for economic or combat state.
7. Revoked devices cannot sync; offline local caches should be protected by device-level storage controls and clearly labeled as last-synced, not actively authorized. Describe revocation limits for previously downloaded plaintext.
8. Unknown/new schema fields are preserved on safe round trips, not erased by an older Android build.

On every shared character change, the host records who changed what, previous/replacement values where appropriate, approval identity, source rule-pack version, timestamp and operation ID. Audit logs are portable project files, with safeguards against unbounded history growth.

## 7. Reusable engine responsibility and ownership

This architecture **extends** existing 023/024 IDs, rather than defining duplicate engines:

| Existing engine | Required extension | Owner |
|---|---|---|
| S25 LAN Sync, Presence & Transport | local hosting, QR pairing, host identity, session auth, scoped command transport, WebSocket state projections, reconnect/version negotiation | Parent Catalyst / Claude |
| S10 Permission, Visibility & Redaction | GM/player/device roles, assigned-character authorization, per-action grants, filtered DTOs, approval policy and secret-safe projection | Parent Catalyst / Claude |
| S09 State/Event Log/Undo | command/event envelopes, revisions, device operation IDs, idempotency, conflict queue and approval records | Parent Catalyst / Claude |
| S07 Dice Notation & Outcome | typed remote roll intent, outcome/history with campaign context, no second random evaluator | Parent Catalyst / Claude |
| S24 Audio Playback/Cue | PC-hosted audio state, scene cues, channel/mixer controls and approved remote commands | Parent Catalyst / Claude |
| S23 Scene Presentation & Session Orchestration | scene/encounter/party command interfaces, authorization and player-safe projections | Parent Catalyst / Claude |
| S01/S02 Storage and Stable IDs | durable character/campaign IDs, hosted authority and portable command reconciliation | Parent Catalyst / Claude, existing services first |
| S04–S06 Rules, Calculation & Choices | deterministic portable rules and validation used by Windows/Android; consumed by host for authorization | Character Studio team incubates pure core; parent integrates |
| M01–M04 and C01–C12 Character UI/Engines | Windows builder/sheet/advancement and Android-ready domain contracts | Character Studio team, separate repository |
| New M19 — GM Remote Browser Workspace | mobile responsive browser interface, initially soundboard controls, later GM tools | Parent Catalyst / Claude |
| New M20 — Player Sharing Service Adapter | GM-hosted character-specific projections, commands, approval queue and authorized inventory | Parent Catalyst / Claude |
| New M21 — Android Character Companion | Android player UI, local character replica, offline command queue, sync client and notification/approval UI | Character Studio team, **after Windows is validated** |

M19/M20/M21 are **module ownership additions**, not new global infrastructure engines. The shared transport S25 and permission/event services remain the only underlying LAN engine family. Parent does not implement duplicate Android builder/sheet rules logic; Android reuses the Character Studio domain modules.

## 8. Phased delivery and dependencies

**Phase A — Windows foundation (current first priority):** finish Catalyst Character Studio Windows builder, digital sheet, inventory/resource engines and portable domain data. Preserve testable shared rules contracts. Independently finish the parent Campaign Wizard and global dice as already planned.

**Phase B — Server/security contracts:** implement/verify S10, S09 and minimal S25: host sessions, device pairing, character projections, per-command permissions, revoke, audit, single-character read-only sync. Prototype GM Remote soundboard after S24 has working playback.

**Phase C — Player permissions and remote MVP:** allow approved inventory/resource edits, idempotent sync, GM approval queue, cross-device live refresh; GM browser soundboard controls with saved scenes and local audio. Browser-based player **read-only fallback** can be considered if Android development is delayed, but must use identical host APIs and restrictions.

**Phase D — Android Character Companion:** implement Android interface and safe local storage using reused Character Studio core; support standalone offline characters; connect to campaign via pairing; add queued commands, conflict resolution and tests.

**Phase E — remote expansion:** encounter/initiative, dice, party state, map reveal, references, player-facing display features and optional PWA polish. Future non-LAN networking is explicitly out of scope unless separately approved with threat model.

No phase may claim production readiness until its security and offline synchronization gates pass.

## 9. Acceptance tests and threat cases

- With internet disconnected but local Wi-Fi active, GM phone pairs, controls soundboard, and sees synchronized audio state from PC.
- Pairing expires; an invalid/expired/reused code fails. GM can revoke both GM and player device sessions.
- Player A cannot fetch player B's private character or inventory by guessing IDs, manipulating websocket subscriptions or supplying crafted commands.
- GM-private notes, secret NPC attributes, hidden maps and paid sourcebook data are absent from player response payloads, not merely hidden by UI.
- Only allowed fields/actions sync. GM-only inventory grant and advancement operations trigger approval queue; no forbidden mutation lands.
- Android player may view/edit a standalone character with neither internet nor desktop present.
- A campaign-linked character edited offline reconnects and negotiates schema/pack versions; safe edits merge, contested resources produce explicit conflict; no silent overwrites.
- Retried item consumption/spell-slot updates with the same operation ID apply **once**, not twice.
- Host reboots mid-write or connection drops mid-commit; atomic project files and queued client commands recover deterministically.
- Switching campaign, character owner, GM role or rules-pack version invalidates stale session permissions and prevents cross-campaign leakage.
- Mobile browser cannot call local filesystem, shell, plugin management or arbitrary engine commands.
- UI follows Catalyst Frost Blue tokens, touch sizing, keyboard/screen-reader accessibility and reduced-motion settings.
- All shared audio and character logic works offline, with no proprietary content redistribution or mandatory cloud service.

## 10. Build-session instructions for Claude and Character Studio

**Parent Catalyst / Claude:** audit existing network/security/audio/event services before creating code. Implement S25/S10/S09 shared capability, then M20 player adapter and M19 GM Remote; continue dice S07/S08 and campaign wizard M05 as separate existing workstreams. Coordinate DTOs and fixtures with Character Studio. Do not build a second character builder, character sheet, character-rule engine or Android character schema.

**Character Studio team (separate repo):** build M01–M04 and C01–C12 Windows-first; maintain portable S04–S06 engines and schemas; design remote capabilities and sync command ports without introducing a competing LAN server; after Windows validated, implement M21 Android UI/replica/sync using shared domain code.

**Documentation and integration:** Any new public command, permission or character DTO updates both repos' contract fixtures. This document defines architecture, not an assertion of implemented features. Confirm code paths and tests in each workstream's implementation PR.

## Owner-approved personal-use and content policy (2026-11-24)

Catalyst is for the owner's private use; no commercial distribution is planned. Applicable access, copying and software-license terms remain in force. Personal-use content access does not imply permission to transmit rulebooks or restricted rule payloads to other devices/users. Preserve existing GM/player permissions and independently enforce applicable content rights; exclude unknown/restricted source payloads from sharing unless the specific transfer is authorized. Do not confuse redistribution tags with access-control permissions.
