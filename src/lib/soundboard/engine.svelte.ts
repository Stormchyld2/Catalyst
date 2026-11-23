/**
 * Offline playback for the campaign soundboard. Audio bytes come from the
 * campaign folder through the backend and play from in-memory blob URLs, so
 * nothing is fetched from the network.
 */

import { audioMimeType, effectiveVolume, isExclusiveChannel, type SoundboardLayout, type SoundPad } from "./layout";

export type AudioLoader = (file: string) => Promise<ArrayBuffer>;
export type AudioFactory = () => HTMLAudioElement;

const FADE_MS = 600;
const FADE_STEP_MS = 40;

interface Voice {
  padId: string;
  audio: HTMLAudioElement;
  fade: ReturnType<typeof setInterval> | null;
}

export class SoundboardEngine {
  /** Pad ids that are currently playing (or loading to play). */
  playing = $state<Record<string, boolean>>({});
  /** Last playback problem per pad, such as a missing or undecodable file. */
  errors = $state<Record<string, string>>({});

  #voices = new Map<string, Voice>();
  #urls = new Map<string, string>();
  #pending = new Map<string, Promise<string>>();
  #layout: SoundboardLayout;
  #loader: AudioLoader;
  #createAudio: AudioFactory;
  #disposed = false;

  constructor(layout: SoundboardLayout, loader: AudioLoader, createAudio: AudioFactory = () => new Audio()) {
    this.#layout = layout;
    this.#loader = loader;
    this.#createAudio = createAudio;
  }

  /** Point the engine at a new layout and re-apply volumes and loop flags. */
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
    // The pad may have been stopped (or another started) while loading.
    if (this.#disposed || !this.isPlaying(pad.id) || this.#voices.has(pad.id)) return;

    const audio = this.#createAudio();
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

  /** Stop one pad; music and ambience fade out unless `fade` is false. */
  stop(padId: string, fade = true) {
    const voice = this.#voices.get(padId);
    if (!voice) {
      if (this.playing[padId]) this.#clearPlaying(padId);
      return;
    }
    const pad = this.#layout.pads.find((p) => p.id === padId);
    if (!fade || !pad || !isExclusiveChannel(pad.channel)) {
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

  stopChannel(channel: SoundPad["channel"], exceptPadId?: string) {
    for (const pad of this.#layout.pads) {
      if (pad.channel === channel && pad.id !== exceptPadId && (this.#voices.has(pad.id) || this.playing[pad.id])) {
        this.stop(pad.id);
      }
    }
  }

  /** Silence everything at once (no fade). */
  stopAll() {
    for (const padId of [...this.#voices.keys()]) this.#release(padId);
    this.playing = {};
  }

  /** Drop cached audio for files that changed or were removed. */
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
    const cached = this.#urls.get(file);
    if (cached) return cached;
    let pending = this.#pending.get(file);
    if (!pending) {
      pending = this.#loader(file).then((bytes) => {
        const url = URL.createObjectURL(new Blob([bytes], { type: audioMimeType(file) }));
        if (this.#disposed) {
          URL.revokeObjectURL(url);
          return url;
        }
        this.#urls.set(file, url);
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
