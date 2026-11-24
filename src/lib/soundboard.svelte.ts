// Campaign soundboard model and offline playback. The layout file and campaign
// audio are owned by the Rust commands in src-tauri/src/soundboard.rs.

export type SoundChannel = "music" | "ambient" | "effects";

export const CHANNELS: readonly { id: SoundChannel; label: string; hint: string }[] = [
  { id: "music", label: "Music", hint: "One track at a time. Starting another fades the current one out." },
  { id: "ambient", label: "Ambience", hint: "Background layers. Any number can play together." },
  { id: "effects", label: "Effects", hint: "One-shot sounds that can overlap." },
] as const;

export interface SoundPad {
  id: string;
  label: string;
  // Relative to the campaign's Assets/Audio folder, or "builtin:<name>.ogg".
  file: string;
  channel: SoundChannel;
  loop: boolean;
  volume: number;
  // GM Screen pad sets (doc 034).
  tags?: string[];
}

export interface SoundboardLayout {
  format: "catalyst-soundboard";
  version: number;
  master_volume: number;
  channels: Record<SoundChannel, { volume: number }>;
  pads: SoundPad[];
}

export interface AudioFile {
  path: string;
  name: string;
  size: number;
}

export const AUDIO_EXTENSIONS = ["mp3", "ogg", "oga", "opus", "wav", "flac", "m4a", "aac", "webm"] as const;

// Synthesized starter sounds shipped in static/soundboard/ (CC0, made by
// scripts/make-soundboard-sounds.py), so every campaign has pads offline.
export const BUILTIN_SOUNDS: readonly { file: string; label: string; channel: SoundChannel }[] = [
  { file: "builtin:tavern-jig.ogg", label: "Tavern Jig", channel: "music" },
  { file: "builtin:travel-march.ogg", label: "Travel March", channel: "music" },
  { file: "builtin:dungeon-drone.ogg", label: "Dungeon Drone", channel: "music" },
  { file: "builtin:battle-drums.ogg", label: "Battle Drums", channel: "music" },
  { file: "builtin:suspense.ogg", label: "Suspense", channel: "music" },
  { file: "builtin:rain-storm.ogg", label: "Rain & Storm", channel: "ambient" },
  { file: "builtin:wind.ogg", label: "Wind on the Road", channel: "ambient" },
  { file: "builtin:forest.ogg", label: "Forest", channel: "ambient" },
  { file: "builtin:cave.ogg", label: "Cave Drips", channel: "ambient" },
  { file: "builtin:night-camp.ogg", label: "Night Camp", channel: "ambient" },
  { file: "builtin:ocean.ogg", label: "Ocean Waves", channel: "ambient" },
  { file: "builtin:eerie-atmosphere.ogg", label: "Eerie Atmosphere", channel: "ambient" },
  { file: "builtin:tension-bed.ogg", label: "Tension Bed", channel: "ambient" },
  { file: "builtin:epic-swell.ogg", label: "Epic Swell", channel: "ambient" },
  { file: "builtin:sword-clash.ogg", label: "Sword Clash", channel: "effects" },
  { file: "builtin:spell-cast.ogg", label: "Spell Cast", channel: "effects" },
  { file: "builtin:thunder.ogg", label: "Thunder", channel: "effects" },
  { file: "builtin:magic-chime.ogg", label: "Magic Chime", channel: "effects" },
  { file: "builtin:door-creak.ogg", label: "Door Creak", channel: "effects" },
  { file: "builtin:monster-growl.ogg", label: "Monster Growl", channel: "effects" },
] as const;

export function isBuiltin(file: string): boolean {
  return file.startsWith("builtin:");
}

// A campaign without a saved layout starts with every built-in sound on the board.
export function defaultLayout(): SoundboardLayout {
  return {
    format: "catalyst-soundboard",
    version: 1,
    master_volume: 0.8,
    channels: { music: { volume: 1 }, ambient: { volume: 1 }, effects: { volume: 1 } },
    pads: BUILTIN_SOUNDS.map((s) => createPad(s.file, { label: s.label, channel: s.channel })),
  };
}

export function clampVolume(value: number): number {
  return Number.isFinite(value) ? Math.min(1, Math.max(0, value)) : 1;
}

// Only music is single-track; ambience layers and effects overlap freely.
export function isExclusiveChannel(channel: SoundChannel): boolean {
  return channel === "music";
}

export function loopsByDefault(channel: SoundChannel): boolean {
  return channel !== "effects";
}

export function effectiveVolume(layout: SoundboardLayout, pad: SoundPad): number {
  return clampVolume(layout.master_volume) * clampVolume(layout.channels[pad.channel].volume) * clampVolume(pad.volume);
}

export function labelFromFile(file: string): string {
  const base = file.replace(/^builtin:/, "").split("/").pop() ?? file;
  const dot = base.lastIndexOf(".");
  return (dot > 0 ? base.slice(0, dot) : base).trim() || "Sound";
}

export function createPad(file: string, overrides: Partial<SoundPad> = {}): SoundPad {
  const channel = overrides.channel ?? "effects";
  return {
    id: `pad-${crypto.randomUUID()}`,
    label: labelFromFile(file),
    file,
    channel,
    loop: loopsByDefault(channel),
    volume: 1,
    ...overrides,
  };
}

function audioMimeType(file: string): string {
  switch (file.split(".").pop()?.toLowerCase()) {
    case "mp3":
      return "audio/mpeg";
    case "ogg":
    case "oga":
    case "opus":
      return "audio/ogg";
    case "wav":
      return "audio/wav";
    case "flac":
      return "audio/flac";
    case "m4a":
    case "aac":
      return "audio/mp4";
    case "webm":
      return "audio/webm";
    default:
      return "application/octet-stream";
  }
}

const FADE_MS = 600;
const FADE_STEP_MS = 40;

interface Voice {
  padId: string;
  audio: HTMLAudioElement;
  fade: ReturnType<typeof setInterval> | null;
}

/** "combat, Tavern ,combat" → ["combat", "Tavern"]: trimmed, no blanks, no case-insensitive repeats. */
export function parseTags(text: string): string[] {
  const tags: string[] = [];
  for (const tag of text.split(",").map((t) => t.trim())) {
    if (tag && !tags.some((t) => t.toLowerCase() === tag.toLowerCase())) tags.push(tag);
  }
  return tags;
}

/** Every tag used on the board, sorted, for the GM Screen's pad-set dropdown. */
export function padTags(layout: SoundboardLayout): string[] {
  return parseTags(layout.pads.flatMap((p) => p.tags ?? []).join(",")).sort((a, b) => a.localeCompare(b));
}

/** Pads carrying `tag` (case-insensitive); a null tag means all pads. */
export function padsWithTag(layout: SoundboardLayout, tag: string | null): SoundPad[] {
  if (!tag) return layout.pads;
  const wanted = tag.toLowerCase();
  return layout.pads.filter((p) => p.tags?.some((t) => t.toLowerCase() === wanted));
}

export class SoundboardEngine {
  playing = $state<Record<string, boolean>>({});
  errors = $state<Record<string, string>>({});

  #voices = new Map<string, Voice>();
  #urls = new Map<string, string>();
  #pending = new Map<string, Promise<string>>();
  #layout: SoundboardLayout;
  #loadCampaignFile: (file: string) => Promise<ArrayBuffer>;
  #disposed = false;

  constructor(layout: SoundboardLayout, loadCampaignFile: (file: string) => Promise<ArrayBuffer>) {
    this.#layout = layout;
    this.#loadCampaignFile = loadCampaignFile;
  }

  setLayout(layout: SoundboardLayout) {
    this.#layout = layout;
    for (const voice of this.#voices.values()) {
      const pad = layout.pads.find((p) => p.id === voice.padId);
      if (!pad) {
        this.stop(voice.padId, false);
        continue;
      }
      voice.audio.loop = pad.loop;
      if (!voice.fade) voice.audio.volume = effectiveVolume(layout, pad);
    }
  }

  isPlaying(padId: string): boolean {
    return this.playing[padId] === true;
  }

  async toggle(pad: SoundPad) {
    if (this.isPlaying(pad.id)) this.stop(pad.id);
    else await this.play(pad);
  }

  async play(pad: SoundPad) {
    if (this.#disposed) return;
    if (isExclusiveChannel(pad.channel)) this.stopChannel(pad.channel, pad.id);
    this.stop(pad.id, false);
    this.playing = { ...this.playing, [pad.id]: true };
    const { [pad.id]: _cleared, ...rest } = this.errors;
    this.errors = rest;

    let url: string;
    try {
      url = await this.#urlFor(pad.file);
    } catch (e) {
      this.#fail(pad.id, e instanceof Error ? e.message : String(e));
      return;
    }
    // The pad may have been stopped, or started again, while its file loaded.
    if (this.#disposed || !this.isPlaying(pad.id) || this.#voices.has(pad.id)) return;

    const audio = new Audio();
    audio.src = url;
    audio.loop = pad.loop;
    audio.volume = effectiveVolume(this.#layout, pad);
    const voice: Voice = { padId: pad.id, audio, fade: null };
    audio.onended = () => {
      if (this.#voices.get(pad.id) === voice) this.#release(pad.id);
    };
    this.#voices.set(pad.id, voice);
    try {
      await audio.play();
    } catch (e) {
      if (this.#voices.get(pad.id) === voice) {
        this.#release(pad.id);
        this.#fail(pad.id, e instanceof Error ? e.message : "This file could not be played.");
      }
    }
  }

  // Looping pads fade out unless `fade` is false; one-shots stop at once.
  stop(padId: string, fade = true) {
    const voice = this.#voices.get(padId);
    if (!voice) {
      if (this.playing[padId]) this.#clearPlaying(padId);
      return;
    }
    const pad = this.#layout.pads.find((p) => p.id === padId);
    if (!fade || !pad || !pad.loop) {
      this.#release(padId);
      return;
    }
    this.#clearPlaying(padId);
    if (voice.fade) return;
    const start = voice.audio.volume;
    const steps = Math.max(1, Math.round(FADE_MS / FADE_STEP_MS));
    let step = 0;
    voice.fade = setInterval(() => {
      step += 1;
      voice.audio.volume = Math.max(0, start * (1 - step / steps));
      if (step >= steps && this.#voices.get(padId) === voice) this.#release(padId);
    }, FADE_STEP_MS);
  }

  stopChannel(channel: SoundChannel, exceptPadId?: string) {
    for (const pad of this.#layout.pads) {
      if (pad.channel === channel && pad.id !== exceptPadId && (this.#voices.has(pad.id) || this.playing[pad.id])) {
        this.stop(pad.id);
      }
    }
  }

  stopAll() {
    for (const padId of [...this.#voices.keys()]) this.#release(padId);
    this.playing = {};
  }

  // Drop cached audio so a rescanned file is read again.
  forget(file: string) {
    const url = this.#urls.get(file);
    if (url) URL.revokeObjectURL(url);
    this.#urls.delete(file);
  }

  dispose() {
    this.#disposed = true;
    this.stopAll();
    for (const url of this.#urls.values()) URL.revokeObjectURL(url);
    this.#urls.clear();
    this.#pending.clear();
  }

  async #urlFor(file: string): Promise<string> {
    if (isBuiltin(file)) return `/soundboard/${file.slice("builtin:".length)}`;
    const cached = this.#urls.get(file);
    if (cached) return cached;
    let pending = this.#pending.get(file);
    if (!pending) {
      pending = this.#loadCampaignFile(file).then((bytes) => {
        const url = URL.createObjectURL(new Blob([bytes], { type: audioMimeType(file) }));
        if (this.#disposed) URL.revokeObjectURL(url);
        else this.#urls.set(file, url);
        return url;
      });
      this.#pending.set(file, pending);
      pending.finally(() => this.#pending.delete(file)).catch(() => {});
    }
    return pending;
  }

  #release(padId: string) {
    const voice = this.#voices.get(padId);
    if (voice) {
      if (voice.fade) clearInterval(voice.fade);
      voice.audio.onended = null;
      voice.audio.pause();
      voice.audio.removeAttribute("src");
      this.#voices.delete(padId);
    }
    this.#clearPlaying(padId);
  }

  #clearPlaying(padId: string) {
    if (!(padId in this.playing)) return;
    const { [padId]: _removed, ...rest } = this.playing;
    this.playing = rest;
  }

  #fail(padId: string, message: string) {
    this.#clearPlaying(padId);
    this.errors = { ...this.errors, [padId]: message };
  }
}
