/**
 * Soundboard layout model. The authoritative copy is the campaign's
 * `Assets/Audio/soundboard.json`, read and written by the Rust backend; these
 * helpers are pure so the UI and tests can share them.
 */

export type SoundChannel = "music" | "ambient" | "effects";

export const CHANNELS: readonly { id: SoundChannel; label: string; hint: string }[] = [
  { id: "music", label: "Music", hint: "One track at a time. Starting another fades the current one out." },
  { id: "ambient", label: "Ambience", hint: "One background loop at a time, such as rain or a tavern." },
  { id: "effects", label: "Effects", hint: "One-shot sounds that can overlap." },
] as const;

export interface SoundPad {
  id: string;
  label: string;
  /** Path relative to the campaign's Assets/Audio folder. */
  file: string;
  channel: SoundChannel;
  loop: boolean;
  volume: number;
  /** GM Screen pad sets (doc 034). */
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

export function emptyLayout(): SoundboardLayout {
  return {
    format: "catalyst-soundboard",
    version: 1,
    master_volume: 0.8,
    channels: { music: { volume: 1 }, ambient: { volume: 1 }, effects: { volume: 1 } },
    pads: [],
  };
}

export function clampVolume(value: number): number {
  return Number.isFinite(value) ? Math.min(1, Math.max(0, value)) : 1;
}

/** Music and ambience are exclusive: one pad plays per channel at a time. */
export function isExclusiveChannel(channel: SoundChannel): boolean {
  return channel !== "effects";
}

export function effectiveVolume(layout: SoundboardLayout, pad: SoundPad): number {
  return clampVolume(layout.master_volume) * clampVolume(layout.channels[pad.channel].volume) * clampVolume(pad.volume);
}

/** "Music/Tavern night.ogg" → "Tavern night". */
export function labelFromFile(file: string): string {
  const base = file.split("/").pop() ?? file;
  const dot = base.lastIndexOf(".");
  return (dot > 0 ? base.slice(0, dot) : base).trim() || "Sound";
}

/** Sensible defaults from a file's folder: Music/ loops on music, Ambience/ loops on ambient. */
export function guessChannel(file: string): SoundChannel {
  const lower = file.toLowerCase();
  if (/(^|\/)(music|songs?|themes?)\//.test(lower)) return "music";
  if (/(^|\/)(ambien(t|ce)s?|atmosphere|environment|weather)\//.test(lower)) return "ambient";
  return "effects";
}

export function newPadId(): string {
  const uuid =
    typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
      ? crypto.randomUUID()
      : "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (c) => {
          const r = (Math.random() * 16) | 0;
          return (c === "x" ? r : (r & 0x3) | 0x8).toString(16);
        });
  return `pad-${uuid}`;
}

export function createPad(file: string, overrides: Partial<SoundPad> = {}): SoundPad {
  const channel = overrides.channel ?? guessChannel(file);
  return {
    id: newPadId(),
    label: labelFromFile(file),
    file,
    channel,
    loop: isExclusiveChannel(channel),
    volume: 1,
    ...overrides,
  };
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

export function audioMimeType(file: string): string {
  const ext = file.split(".").pop()?.toLowerCase();
  switch (ext) {
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
