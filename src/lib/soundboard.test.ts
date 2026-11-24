import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  BUILTIN_SOUNDS,
  SoundboardEngine,
  createPad,
  defaultLayout,
  effectiveVolume,
  labelFromFile,
  type SoundboardLayout,
} from "./soundboard.svelte";

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
  return new SoundboardEngine(layout, loader);
}

beforeEach(() => {
  created = [];
  loader.mockClear();
  layout = { ...defaultLayout(), pads: [] };
  vi.stubGlobal(
    "Audio",
    vi.fn(function () {
      const audio = new FakeAudio();
      created.push(audio);
      return audio;
    }),
  );
  vi.stubGlobal("URL", { ...URL, createObjectURL: vi.fn(() => "blob:test"), revokeObjectURL: vi.fn() });
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe("soundboard layout", () => {
  it("starts a new campaign with every built-in sound, music and ambience looping", () => {
    const pads = defaultLayout().pads;
    expect(pads.map((p) => p.file)).toEqual(BUILTIN_SOUNDS.map((s) => s.file));
    expect(pads.filter((p) => p.channel === "effects").every((p) => !p.loop)).toBe(true);
    expect(pads.filter((p) => p.channel !== "effects").every((p) => p.loop)).toBe(true);
    expect(new Set(pads.map((p) => p.id)).size).toBe(pads.length);
  });

  it("names pads from file paths", () => {
    expect(labelFromFile("Music/Tavern night.mp3")).toBe("Tavern night");
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
  it("layers music, several ambience loops and one-shots at the same time", async () => {
    const music = createPad("eerie.mp3", { channel: "music" });
    const ocean = createPad("builtin:ocean.ogg", { channel: "ambient" });
    const rain = createPad("builtin:rain-storm.ogg", { channel: "ambient" });
    const fx1 = createPad("builtin:sword-clash.ogg");
    const fx2 = createPad("builtin:sword-clash.ogg");
    layout.pads = [music, ocean, rain, fx1, fx2];
    const e = engine();
    for (const pad of layout.pads) await e.play(pad);
    expect(Object.keys(e.playing).sort()).toEqual(layout.pads.map((p) => p.id).sort());
    expect(created.every((a) => !a.paused)).toBe(true);
  });

  it("plays built-in sounds from the app bundle without touching the campaign", async () => {
    const pad = createPad("builtin:thunder.ogg");
    layout.pads = [pad];
    await engine().play(pad);
    expect(loader).not.toHaveBeenCalled();
    expect(created[0].src).toBe("/soundboard/thunder.ogg");
  });

  it("keeps one music track at a time and fades the previous out", async () => {
    vi.useFakeTimers();
    const one = createPad("one.ogg", { channel: "music" });
    const two = createPad("two.ogg", { channel: "music" });
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
  });

  it("honours each pad's loop or one-shot setting in any channel", async () => {
    const loopingFx = createPad("a.wav", { loop: true });
    const onceAmbient = createPad("b.wav", { channel: "ambient", loop: false });
    layout.pads = [loopingFx, onceAmbient];
    const e = engine();
    await e.play(loopingFx);
    await e.play(onceAmbient);
    expect(created.map((a) => a.loop)).toEqual([true, false]);
    created[1].onended?.();
    expect(e.isPlaying(onceAmbient.id)).toBe(false);
    e.setLayout({ ...layout, pads: [{ ...loopingFx, loop: false }, onceAmbient] });
    expect(created[0].loop).toBe(false);
  });

  it("stop all silences every channel at once", async () => {
    const music = createPad("one.ogg", { channel: "music" });
    const fx = createPad("a.wav");
    layout.pads = [music, fx];
    const e = engine();
    await e.play(music);
    await e.play(fx);
    e.stopAll();
    expect(e.playing).toEqual({});
    expect(created.every((a) => a.paused)).toBe(true);
  });

  it("reuses loaded campaign audio and applies new volumes live", async () => {
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
});
