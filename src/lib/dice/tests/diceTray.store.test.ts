import { beforeEach, describe, expect, it, vi } from "vitest";
import { DiceEngine } from "../engine/diceEngine";
import {
  RendererDisposedError,
  type DiceRenderer,
} from "../engine/diceRenderer";
import {
  clampGeometry,
  DiceTrayStore,
  SETTINGS_STORAGE_KEY,
  TRAY_STORAGE_KEY,
} from "../stores/diceTray.svelte";
import type { DieResult } from "../types/dice";

function deterministicEngine() {
  let n = 0;
  return new DiceEngine({ random: () => n++, newId: () => `roll-${n}` });
}

function controllableRenderer() {
  const pending: Array<{
    resolve: (dice: DieResult[]) => void;
    reject: (e: Error) => void;
  }> = [];
  const renderer: DiceRenderer = {
    kind: "physics",
    init: vi.fn(async () => {}),
    roll: vi.fn(
      () =>
        new Promise<DieResult[]>((resolve, reject) =>
          pending.push({ resolve, reject }),
        ),
    ),
    clear: vi.fn(() =>
      pending.splice(0).forEach((p) => p.reject(new RendererDisposedError())),
    ),
    resize: vi.fn(),
    dispose: vi.fn(),
  };
  const settle = (values: number[]) =>
    pending
      .shift()!
      .resolve(values.map((value) => ({ type: "d20" as const, value })));
  return { renderer, settle, pending };
}

beforeEach(() => {
  localStorage.clear();
});

describe("DiceTrayStore", () => {
  it("defaults to 2d20 and records an instant roll", async () => {
    const tray = new DiceTrayStore(deterministicEngine());
    tray.open("compact");
    expect(tray.selection).toBe("2d20");
    const record = await tray.rollSelection();
    expect(record?.dice).toEqual([
      { type: "d20", value: 1 },
      { type: "d20", value: 2 },
    ]);
    expect(record?.total).toBe(3);
    expect(record?.resolution).toBe("random");
    expect(tray.current).toBe(record);
    expect(tray.history.all).toEqual([record]);
  });

  it("uses physics in expanded mode once the renderer is ready", async () => {
    const tray = new DiceTrayStore(deterministicEngine());
    const { renderer, settle } = controllableRenderer();
    tray.open("expanded");
    tray.attachRenderer(renderer);
    const roll = tray.rollSelection();
    expect(tray.status).toBe("rolling");
    settle([14, 7]);
    const record = await roll;
    expect(record?.total).toBe(21);
    expect(record?.resolution).toBe("physics");
    expect(tray.status).toBe("idle");
  });

  it("keeps history through close, reopen and mode switches", async () => {
    const tray = new DiceTrayStore(deterministicEngine());
    tray.open("compact");
    await tray.rollSelection();
    tray.close();
    tray.open();
    tray.setMode("expanded");
    tray.setMode("compact");
    await tray.rollSelection();
    expect(tray.history.all).toHaveLength(2);
  });

  it("records nothing when the tray is cleared mid-roll", async () => {
    const tray = new DiceTrayStore(deterministicEngine());
    const { renderer } = controllableRenderer();
    tray.open("expanded");
    tray.attachRenderer(renderer);
    const roll = tray.rollSelection();
    tray.clearTray();
    await expect(roll).resolves.toBeNull();
    expect(tray.history.all).toHaveLength(0);
    expect(tray.status).toBe("idle");
    expect(tray.error).toBeNull();
  });

  it("records nothing when the tray closes mid-roll", async () => {
    const tray = new DiceTrayStore(deterministicEngine());
    const { renderer, settle } = controllableRenderer();
    tray.open("expanded");
    tray.attachRenderer(renderer);
    const roll = tray.rollSelection();
    tray.close();
    settle([3, 4]);
    await expect(roll).resolves.toBeNull();
    expect(tray.history.all).toHaveLength(0);
  });

  it("shows an error and records nothing for an invalid request", async () => {
    const tray = new DiceTrayStore(deterministicEngine());
    tray.open("compact");
    await expect(tray.roll({ expression: "4d6" })).resolves.toBeNull();
    expect(tray.status).toBe("error");
    expect(tray.error).toMatch(/not available/);
    expect(tray.history.all).toHaveLength(0);
  });

  it("rerolls the last expression with its context", async () => {
    const tray = new DiceTrayStore(deterministicEngine());
    tray.open("compact");
    await tray.roll({
      expression: "1d20",
      source: "gm-console",
      visibility: "gm-private",
    });
    const again = await tray.reroll();
    expect(again).toMatchObject({
      expression: "1d20",
      source: "gm-console",
      visibility: "gm-private",
    });
  });

  it("limits dice selection to what this phase supports", () => {
    const tray = new DiceTrayStore(deterministicEngine());
    tray.selectType("d6");
    expect(tray.selectedType).toBe("d20");
    tray.adjustCount(5);
    expect(tray.selection).toBe("7d20");
    tray.adjustCount(-10);
    expect(tray.selection).toBe("");
    tray.adjustCount(50);
    expect(tray.selection).toBe("20d20");
  });

  it("remembers mode, position, size and settings", () => {
    const tray = new DiceTrayStore(deterministicEngine());
    tray.open("expanded");
    tray.moveTo(100, 120);
    tray.resizeTo(700, 500);
    tray.persistTray();
    tray.updateSettings({ lowPerformance: true });
    tray.setMode("compact");

    const reopened = new DiceTrayStore(deterministicEngine());
    expect(reopened.mode).toBe("compact");
    expect(reopened.geometry.expanded).toEqual({
      x: 100,
      y: 120,
      width: 700,
      height: 500,
    });
    expect(reopened.settings.lowPerformance).toBe(true);
    expect(JSON.parse(localStorage.getItem(TRAY_STORAGE_KEY)!).mode).toBe(
      "compact",
    );
    expect(
      JSON.parse(localStorage.getItem(SETTINGS_STORAGE_KEY)!).lowPerformance,
    ).toBe(true);
  });

  it("ignores corrupt stored state", () => {
    localStorage.setItem(TRAY_STORAGE_KEY, "{not json");
    localStorage.setItem(
      SETTINGS_STORAGE_KEY,
      JSON.stringify({ animate: "yes" }),
    );
    const tray = new DiceTrayStore(deterministicEngine());
    expect(tray.mode).toBe("expanded");
    expect(tray.geometry.expanded.width).toBeGreaterThan(0);
    expect(typeof tray.settings.animate).toBe("boolean");
  });
});

describe("clampGeometry", () => {
  const view = { width: 1000, height: 700 };

  it("keeps the tray on screen", () => {
    expect(
      clampGeometry(
        { x: -50, y: 900, width: 600, height: 420 },
        "expanded",
        view,
      ),
    ).toEqual({
      x: 8,
      y: 272,
      width: 600,
      height: 420,
    });
  });

  it("enforces minimum and maximum sizes", () => {
    const small = clampGeometry(
      { x: 0, y: 0, width: 10, height: 10 },
      "expanded",
      view,
    );
    expect(small.width).toBe(560);
    expect(small.height).toBe(400);
    const large = clampGeometry(
      { x: 0, y: 0, width: 5000, height: 5000 },
      "expanded",
      view,
    );
    expect(large.width).toBe(984);
    expect(large.height).toBe(684);
  });
});
