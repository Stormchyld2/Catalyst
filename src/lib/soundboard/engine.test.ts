import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SoundboardEngine } from "./engine.svelte";
import { createPad, effectiveVolume, emptyLayout, guessChannel, labelFromFile, type SoundboardLayout } from "./layout";

class FakeAudio {
  src = "";
  loop = false;
  volume = 1;
  paused = true;
  onended: (() => void) | null = null;
  play = vi.fn(async () => {
    this.paused = false;
  });
  pause = vi.fn(() => {
    this.paused = true;
  });
  removeAttribute(name: string) {
    if (name === "src") this.src = "";
  }
}

let created: FakeAudio[];
let layout: SoundboardLayout;
const loader = vi.fn(async (_file: string) => new ArrayBuffer(4));

function engine() {
  return new SoundboardEngine(layout, loader, () => {
    const audio = new FakeAudio();
    created.push(audio);
    return audio as unknown as HTMLAudioElement;
  });
}

beforeEach(() => {
  created = [];
  loader.mockClear();
  layout = emptyLayout();
  vi.stubGlobal("URL", { ...URL, createObjectURL: vi.fn(() => "blob:test"), revokeObjectURL: vi.fn() });
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe("soundboard layout helpers", () => {
  it("derives names and channels from campaign file paths", () => {
    expect(labelFromFile("Music/Tavern night.ogg")).toBe("Tavern night");
    expect(guessChannel("Music/Tavern night.ogg")).toBe("music");
    expect(guessChannel("Ambience/Rain.mp3")).toBe("ambient");
    expect(guessChannel("door-slam.wav")).toBe("effects");
    expect(createPad("Ambience/Rain.mp3")).toMatchObject({ channel: "ambient", loop: true, label: "Rain" });
    expect(createPad("door.wav")).toMatchObject({ channel: "effects", loop: false });
    expect(createPad("door.wav").id).toMatch(/^pad-[0-9a-f-]{36}$/);
  });

  it("multiplies master, channel and pad volume", () => {
    const pad = createPad("door.wav", { volume: 0.5 });
    layout.master_volume = 0.8;
    layout.channels.effects.volume = 0.5;
    expect(effectiveVolume(layout, pad)).toBeCloseTo(0.2);
  });
});

describe("SoundboardEngine", () => {
  it("plays effects on top of each other", async () => {
    const a = createPad("a.wav");
    const b = createPad("b.wav");
    layout.pads = [a, b];
    const e = engine();
    await e.play(a);
    await e.play(b);
    expect(e.isPlaying(a.id)).toBe(true);
    expect(e.isPlaying(b.id)).toBe(true);
    expect(created).toHaveLength(2);
  });

  it("keeps one music track at a time and fades the previous out", async () => {
    vi.useFakeTimers();
    const one = createPad("Music/one.ogg");
    const two = createPad("Music/two.ogg");
    layout.pads = [one, two];
    const e = engine();
    await e.play(one);
    await e.play(two);
    expect(e.isPlaying(one.id)).toBe(false);
    expect(e.isPlaying(two.id)).toBe(true);
    expect(created[0].paused).toBe(false);
    vi.advanceTimersByTime(1000);
    expect(created[0].paused).toBe(true);
    expect(created[1].paused).toBe(false);
    expect(created[1].loop).toBe(true);
  });

  it("stop all silences every channel at once", async () => {
    const music = createPad("Music/one.ogg");
    const fx = createPad("a.wav");
    layout.pads = [music, fx];
    const e = engine();
    await e.play(music);
    await e.play(fx);
    e.stopAll();
    expect(e.playing).toEqual({});
    expect(created.every((a) => a.paused)).toBe(true);
  });

  it("reuses loaded audio and applies new volumes live", async () => {
    const pad = createPad("a.wav");
    layout.pads = [pad];
    const e = engine();
    await e.play(pad);
    e.stop(pad.id);
    await e.play(pad);
    expect(loader).toHaveBeenCalledTimes(1);
    e.setLayout({ ...layout, master_volume: 0.5 });
    expect(created[1].volume).toBeCloseTo(0.5);
  });

  it("reports a file that cannot be loaded", async () => {
    const pad = createPad("missing.wav");
    layout.pads = [pad];
    loader.mockRejectedValueOnce(new Error("Audio file 'missing.wav' is missing from Assets/Audio."));
    const e = engine();
    await e.play(pad);
    expect(e.isPlaying(pad.id)).toBe(false);
    expect(e.errors[pad.id]).toContain("missing");
  });

  it("ends one-shot pads when the sound finishes", async () => {
    const pad = createPad("a.wav");
    layout.pads = [pad];
    const e = engine();
    await e.play(pad);
    created[0].onended?.();
    expect(e.isPlaying(pad.id)).toBe(false);
  });
});
