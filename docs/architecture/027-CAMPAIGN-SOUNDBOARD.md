# 027 — Campaign Soundboard

Status: implemented (replaces the old Listen placeholder; workspace id stays `listen`, label "Soundboard").

## What it does

A grid of sound pads for live play, grouped into three channels:

| Channel  | Behaviour                                                                 |
|----------|---------------------------------------------------------------------------|
| Music    | One pad at a time. Starting another fades the current one out (600 ms).   |
| Ambience | One pad at a time, same fade. Meant for looping beds such as rain.        |
| Effects  | One-shots that overlap freely.                                            |

Each pad has a name, an audio file, a channel, a loop flag and its own volume. The mixer has a master
fader plus one fader per channel; a pad plays at `master × channel × pad`. **Stop all** silences
everything immediately. Pads whose file disappeared are shown as "missing file" and cannot play.

## Storage (file-authoritative, no database)

```text
<Campaign>/Assets/Audio/
├── soundboard.json        pad layout and mixer levels
├── Music/…                optional subfolders, any depth
└── *.mp3|ogg|opus|wav|flac|m4a|aac|webm
```

`soundboard.json`:

```json
{
  "format": "catalyst-soundboard",
  "version": 1,
  "master_volume": 0.8,
  "channels": { "music": { "volume": 1 }, "ambient": { "volume": 1 }, "effects": { "volume": 1 } },
  "pads": [
    { "id": "pad-<uuid v4>", "label": "Tavern", "file": "Music/Tavern.ogg",
      "channel": "music", "loop": true, "volume": 1 }
  ]
}
```

- `file` is relative to `Assets/Audio/`. The backend refuses absolute paths, `..`, non-audio
  extensions and symlinks that resolve outside the folder.
- The file is written atomically. A layout from a newer version is refused rather than overwritten.
- Deleting `soundboard.json` only resets the board; the audio files are plain files any player can use.
- **Import audio** copies picked files into `Assets/Audio/` and never overwrites (a clash becomes
  `Name (2).ext`). Files dropped into the folder by hand appear after **Rescan**. Files under a
  `Music/` or `Ambience/` subfolder default to looping music or ambience pads.
- Pad ids use a `pad-` prefix with a UUID v4. They are board-local, not core document IDs.

## Offline playback

Audio bytes are read by the Rust command `read_soundboard_audio` and returned as a raw binary IPC
payload. The UI wraps them in an in-memory `blob:` URL and plays them with `HTMLAudioElement`. No
remote assets, no network requests. The CSP gains `media-src 'self' blob:` for this.

Commands: `list_soundboard_audio`, `read_soundboard_audio`, `load_soundboard_layout`,
`save_soundboard_layout`, `import_soundboard_audio` (`src-tauri/src/soundboard.rs`,
`src-tauri/src/commands/soundboard.rs`). UI: `src/lib/components/SoundboardWorkspace.svelte`,
`src/lib/soundboard/`.

## Prior art reviewed

| Project | License | State | Decision |
|---|---|---|---|
| Ascapes Mixer (ilyaly/ascapes-mixer) | MIT | Svelte 5 + Tauri 2, last commit March 2025 | Design reference for the three-channel music/ambience/effects model and the pad grid. No code copied: it is SvelteKit/plain JS, stores metadata in IndexedDB and copies audio into app data, all of which conflict with Catalyst's storage rules. |
| Ambora (xetorthio/ambora) | MIT | Electron + React, active (Oct 2026) | Reference only for scene moods/crossfades. Different stack. |
| SonicDeck | Unclear license | Tauri + React | Not used. |
| Immerse Yourself, pedro-ibs RPG Soundboard | Listed as MIT, not verified | — | Not used. |

See `THIRD_PARTY_NOTICES.md`.

## Later

Scene presets (save which pads are playing), hotkeys, a native Rust audio backend for very low
latency, and LAN remote control (doc 026) can build on this layout file without changing it.
