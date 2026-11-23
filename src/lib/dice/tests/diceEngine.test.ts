import { describe, expect, it, vi } from "vitest";
import { DiceEngine } from "../engine/diceEngine";
import type { DiceRenderer } from "../engine/diceRenderer";
import type { DieResult } from "../types/dice";

function engine(values: number[] = []) {
  let index = 0;
  return new DiceEngine({
    random: () => values[index++],
    now: () => new Date("2026-11-23T12:00:00.000Z"),
    newId: () => "roll-test",
  });
}

function fakeRenderer(results: DieResult[]): DiceRenderer {
  return {
    kind: "physics",
    init: vi.fn(async () => {}),
    roll: vi.fn(async () => results),
    clear: vi.fn(),
    resize: vi.fn(),
    dispose: vi.fn(),
  };
}

describe("DiceEngine", () => {
  it("uses the settled physics values as the result", async () => {
    const renderer = fakeRenderer([
      { type: "d20", value: 14 },
      { type: "d20", value: 7 },
    ]);
    const record = await engine().roll({ expression: "2d20" }, renderer);
    expect(renderer.roll).toHaveBeenCalledWith(
      expect.objectContaining({ notation: "2d20" }),
    );
    expect(record.dice.map((die) => die.value)).toEqual([14, 7]);
    expect(record.total).toBe(21);
    expect(record.resolution).toBe("physics");
    expect(record.timestamp).toBe("2026-11-23T12:00:00.000Z");
  });

  it("resolves from the random source without a renderer", async () => {
    // Raw values 13 and 6 map to faces 14 and 7.
    const record = await engine([13, 6]).roll({ expression: "2d20" });
    expect(record.dice).toEqual([
      { type: "d20", value: 14 },
      { type: "d20", value: 7 },
    ]);
    expect(record.total).toBe(21);
    expect(record.resolution).toBe("random");
  });

  it("refuses a renderer result that does not match the request", async () => {
    const renderer = fakeRenderer([
      { type: "d20", value: 25 },
      { type: "d20", value: 3 },
    ]);
    await expect(
      engine().roll({ expression: "2d20" }, renderer),
    ).rejects.toThrow(/Invalid d20/);
  });

  it("rejects invalid or unsupported notation before rolling", async () => {
    const renderer = fakeRenderer([]);
    await expect(
      engine().roll({ expression: "2d7" }, renderer),
    ).rejects.toThrow();
    await expect(
      engine().roll({ expression: "4d6" }, renderer),
    ).rejects.toThrow(/not available/);
    expect(renderer.roll).not.toHaveBeenCalled();
  });

  it("supports repeated rolls", async () => {
    const e = engine([0, 1, 2, 3, 4, 5]);
    const totals = [];
    for (let i = 0; i < 3; i++)
      totals.push((await e.roll({ expression: "2d20" })).total);
    expect(totals).toEqual([3, 7, 11]);
  });
});
