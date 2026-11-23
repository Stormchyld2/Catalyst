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
    await expect(tray.roll({ expression: "2d7" })).resolves.toBeNull();
    expect(tray.status).toBe("error");
    expect(tray.error).toMatch(/Unsupported die/);
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

  it("builds selections from die buttons, + and −, largest die first", () => {
    const tray = new DiceTrayStore(deterministicEngine());
    tray.pickDie("d6");
    expect(tray.selectedType).toBe("d6");
    expect(tray.selection).toBe("2d20+1d6");
    tray.pickDie("d6");
    tray.pickDie("d6");
    expect(tray.selection).toBe("2d20+3d6");
    tray.adjustCount(-1);
    expect(tray.selection).toBe("2d20+2d6");
    tray.adjustCount(-5, "d20");
    tray.adjustCount(50);
    expect(tray.selection).toBe("20d6");
    tray.pickDie("d100");
    tray.pickDie("d8");
    expect(tray.selection).toBe("1d8+20d6+1d100");
  });

  it("adjusts the modifier when MOD is the + and − target", () => {
    const tray = new DiceTrayStore(deterministicEngine());
    tray.selectModifier();
    tray.adjustCount(1);
    tray.adjustCount(1);
    tray.adjustCount(1);
    expect(tray.selection).toBe("2d20+3");
    tray.adjustModifier(-10);
    expect(tray.selection).toBe("2d20-7");
    tray.adjustModifier(500);
    expect(tray.modifier).toBe(99);
    tray.pickDie("d8");
    tray.adjustCount(1);
    expect(tray.modifier).toBe(99);
    expect(tray.selection).toBe("2d20+2d8+99");
    tray.resetSelection();
    expect(tray.selection).toBe("1d20");
  });

  it("rolls advantage and disadvantage as kept d20s, never a sum", async () => {
    const tray = new DiceTrayStore(deterministicEngine());
    tray.open("compact");
    tray.pickDie("d4");
    tray.setRollMode("advantage");
    expect(tray.selection).toBe("2d20kh1+1d4");
    const adv = await tray.rollSelection();
    expect(adv?.kind).toBe("advantage");
    expect(adv?.dice.filter((die) => die.dropped)).toHaveLength(1);
    tray.setRollMode("disadvantage");
    expect(tray.selection).toBe("2d20kl1+1d4");
    tray.setRollMode("disadvantage");
    expect(tray.rollMode).toBe("normal");
    expect(tray.selection).toBe("2d20+1d4");
  });

  it("tells listeners about each recorded roll and uses the tray visibility", async () => {
    const tray = new DiceTrayStore(deterministicEngine());
    tray.open("compact");
    const seen: string[] = [];
    const stop = tray.onRecorded((record) => seen.push(record.visibility));
    tray.visibility = "gm-private";
    await tray.rollSelection();
    stop();
    await tray.rollSelection();
    expect(seen).toEqual(["gm-private"]);
  });

  it("rolls silently where Web Audio is unavailable", async () => {
    const tray = new DiceTrayStore(deterministicEngine());
    tray.updateSettings({ sound: true, volume: 1 });
    tray.open("compact");
    await expect(tray.rollSelection()).resolves.not.toBeNull();
  });

  it("saves rolls to the campaign session file and reloads them", async () => {
    const saved: unknown[] = [];
    const session = { id: "session-1", title: "Night one", startedAt: "2026-11-23T12:00:00Z" };
    const files = {
      start: vi.fn(async () => session),
      append: vi.fn(async (_root: string, _id: string, record: unknown) => {
        saved.push(record);
      }),
      read: vi.fn(async () => ({ session, rolls: saved as never[], skippedLines: 0 })),
      list: vi.fn(async () => [session]),
    };
    const tray = new DiceTrayStore(deterministicEngine(), files);
    tray.open("compact");
    await tray.startCampaignSession("/campaign", "Night one");
    const first = await tray.rollSelection();
    const second = await tray.rollSelection();
    await vi.waitFor(() => expect(files.append).toHaveBeenCalledTimes(2));
    expect(first?.sessionId).toBe("session-1");
    expect(files.append).toHaveBeenCalledWith("/campaign", "session-1", first);
    tray.endCampaignSession();
    await tray.rollSelection();
    expect(files.append).toHaveBeenCalledTimes(2);

    const reopened = new DiceTrayStore(deterministicEngine(), files);
    await reopened.resumeCampaignSession("/campaign", "session-1");
    expect(reopened.campaign?.session.title).toBe("Night one");
    expect(reopened.history.all.map((record) => record.id)).toEqual([second?.id, first?.id]);
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

  it("keeps valid appearance settings and drops invalid ones field by field", () => {
    localStorage.setItem(
      SETTINGS_STORAGE_KEY,
      JSON.stringify({ diceColor: "sapphire", diceSize: "huge", surface: "walnut", volume: 3, speed: "quick" })
    );
    const tray = new DiceTrayStore(deterministicEngine());
    expect(tray.settings).toMatchObject({
      diceColor: "sapphire",
      diceSize: "medium",
      surface: "walnut",
      volume: 0.5,
      speed: "quick",
    });
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

  it("keeps the measured compact tray fully on screen", () => {
    expect(clampGeometry({ x: 0, y: 650, width: 360, height: 0 }, "compact", view, 400).y).toBe(292);
    expect(clampGeometry({ x: 0, y: 650, width: 360, height: 0 }, "compact", view).y).toBe(452);
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
