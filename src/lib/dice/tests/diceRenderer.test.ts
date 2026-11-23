import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { parseDiceNotation } from "../engine/diceNotation";
import {
  DiceBoxRenderer,
  RendererDisposedError,
  RendererUnavailableError,
  RollTimeoutError,
  type DiceBoxDie,
} from "../engine/diceRenderer";

class FakeWorker {
  static created: FakeWorker[] = [];
  terminated = false;
  constructor(public url: string) {
    FakeWorker.created.push(this);
  }
  terminate() {
    this.terminated = true;
  }
}

interface FakeBoxControls {
  config: Record<string, unknown>;
  pending: Array<(dice: DiceBoxDie[]) => void>;
  cleared: number;
  resized: number;
}

let boxes: FakeBoxControls[] = [];
let failInit = false;

class FakeDiceBox {
  canvas: HTMLCanvasElement;
  controls: FakeBoxControls;
  constructor(config: Record<string, unknown>) {
    this.controls = { config, pending: [], cleared: 0, resized: 0 };
    boxes.push(this.controls);
    this.canvas = document.createElement("canvas");
    document.querySelector(String(config.container))!.appendChild(this.canvas);
  }
  async init() {
    // Dice Box spawns the physics worker and the offscreen render worker.
    new Worker("blob:physics");
    await Promise.resolve();
    new Worker("blob:render");
    if (failInit) throw new Error("ammo failed");
    this.resizeWorld();
    return this;
  }
  resizeWorld() {
    window.addEventListener("resize", () => this.controls.resized++);
  }
  roll(): Promise<DiceBoxDie[]> {
    return new Promise((resolve) => this.controls.pending.push(resolve));
  }
  clear() {
    this.controls.cleared++;
  }
}

const twoD20 = parseDiceNotation("2d20");
const load = async () => FakeDiceBox as never;
const settle = (box: FakeBoxControls, values: number[]) =>
  box.pending.shift()!(values.map((value) => ({ sides: 20, value })));

let container: HTMLElement;
let originalWorker: typeof Worker;

beforeEach(() => {
  boxes = [];
  failInit = false;
  FakeWorker.created = [];
  originalWorker = globalThis.Worker;
  globalThis.Worker = FakeWorker as unknown as typeof Worker;
  container = document.createElement("div");
  document.body.appendChild(container);
  // jsdom has no WebGL; release() asks the canvas for its context.
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(null);
});

afterEach(() => {
  vi.restoreAllMocks();
  globalThis.Worker = originalWorker;
  container.remove();
  vi.useRealTimers();
});

function renderer(options = {}) {
  return new DiceBoxRenderer({ load, webgl: () => true, ...options });
}

describe("DiceBoxRenderer", () => {
  it("initializes Dice Box against Catalyst's bundled assets", async () => {
    const r = renderer();
    await r.init(container);
    expect(boxes).toHaveLength(1);
    expect(boxes[0].config).toMatchObject({
      assetPath: "/assets/dice-box/",
      origin: window.location.origin,
      offscreen: true,
      enableShadows: true,
    });
    expect(container.querySelectorAll("canvas")).toHaveLength(1);
    r.dispose();
  });

  it("turns shadows off in low-performance mode", async () => {
    const r = renderer({ lowPerformance: true });
    await r.init(container);
    expect(boxes[0].config.enableShadows).toBe(false);
    r.dispose();
  });

  it("does not create a second canvas when init runs twice", async () => {
    const r = renderer();
    await r.init(container);
    await r.init(container);
    expect(boxes).toHaveLength(1);
    expect(container.querySelectorAll("canvas")).toHaveLength(1);
    r.dispose();
  });

  it("resolves a roll with the settled dice, then rolls again", async () => {
    const r = renderer();
    await r.init(container);
    const first = r.roll(twoD20);
    settle(boxes[0], [14, 7]);
    await expect(first).resolves.toEqual([
      { type: "d20", value: 14 },
      { type: "d20", value: 7 },
    ]);
    const second = r.roll(twoD20);
    settle(boxes[0], [20, 1]);
    await expect(second).resolves.toEqual([
      { type: "d20", value: 20 },
      { type: "d20", value: 1 },
    ]);
    r.dispose();
  });

  it("drives resizes itself instead of leaving a window listener", async () => {
    const r = renderer();
    await r.init(container);
    window.dispatchEvent(new Event("resize"));
    expect(boxes[0].resized).toBe(0);
    r.resize();
    expect(boxes[0].resized).toBe(1);
    r.dispose();
    window.dispatchEvent(new Event("resize"));
    expect(boxes[0].resized).toBe(1);
  });

  it("releases workers and the canvas on dispose, and abandons a pending roll", async () => {
    const r = renderer();
    await r.init(container);
    const pending = r.roll(twoD20);
    r.dispose();
    await expect(pending).rejects.toBeInstanceOf(RendererDisposedError);
    expect(FakeWorker.created).toHaveLength(2);
    expect(FakeWorker.created.every((worker) => worker.terminated)).toBe(true);
    expect(container.querySelector("canvas")).toBeNull();
    expect(globalThis.Worker).toBe(FakeWorker);
    expect(() => r.dispose()).not.toThrow();
    await expect(r.roll(twoD20)).rejects.toBeInstanceOf(RendererDisposedError);
  });

  it("abandons a pending roll on clear", async () => {
    const r = renderer();
    await r.init(container);
    const pending = r.roll(twoD20);
    r.clear();
    await expect(pending).rejects.toBeInstanceOf(RendererDisposedError);
    expect(boxes[0].cleared).toBe(1);
    r.dispose();
  });

  it("times out a roll that never settles", async () => {
    vi.useFakeTimers();
    const r = renderer({ timeoutMs: 1000 });
    await r.init(container);
    const pending = r.roll(twoD20);
    vi.advanceTimersByTime(1001);
    await expect(pending).rejects.toBeInstanceOf(RollTimeoutError);
    r.dispose();
  });

  it("reports WebGL being unavailable without loading Dice Box", async () => {
    const loader = vi.fn(load);
    const r = new DiceBoxRenderer({ load: loader, webgl: () => false });
    await expect(r.init(container)).rejects.toBeInstanceOf(
      RendererUnavailableError,
    );
    expect(loader).not.toHaveBeenCalled();
  });

  it("cleans up when Dice Box fails to start", async () => {
    failInit = true;
    const r = renderer();
    await expect(r.init(container)).rejects.toBeInstanceOf(
      RendererUnavailableError,
    );
    expect(FakeWorker.created.every((worker) => worker.terminated)).toBe(true);
    expect(container.querySelector("canvas")).toBeNull();
    expect(globalThis.Worker).toBe(FakeWorker);
  });

  it("stops if disposed while still loading", async () => {
    let release!: () => void;
    const slowLoad = () =>
      new Promise<never>(
        (resolve) => (release = () => resolve(FakeDiceBox as never)),
      );
    const r = new DiceBoxRenderer({ load: slowLoad, webgl: () => true });
    const init = r.init(container);
    r.dispose();
    release();
    await expect(init).rejects.toBeInstanceOf(RendererDisposedError);
    expect(boxes).toHaveLength(0);
  });
});
