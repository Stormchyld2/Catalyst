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
    const narrow = new DiceEngine({ capabilities: { dice: ["d20"], modifiers: false, keep: false } });
    await expect(narrow.roll({ expression: "4d6" }, renderer)).rejects.toThrow(/not available/);
    expect(renderer.roll).not.toHaveBeenCalled();
  });

  it("rolls every die type and mixed expressions from the random source", async () => {
    const e = new DiceEngine({ random: () => 0 });
    const record = await e.roll({ expression: "1d4+1d6+1d8+1d10+1d12+1d20+1d100+3" });
    expect(record.dice.map((die) => die.type)).toEqual(["d4", "d6", "d8", "d10", "d12", "d20", "d100"]);
    expect(record.dice.every((die) => die.value === 1)).toBe(true);
    expect(record.total).toBe(10);
  });

  it("applies advantage and disadvantage and keeps them apart from a sum", async () => {
    const dice = (a: number, b: number): DieResult[] => [
      { type: "d20", value: a },
      { type: "d20", value: b },
    ];
    const adv = await engine().roll({ expression: "1d20+5", mode: "advantage" }, fakeRenderer(dice(4, 17)));
    expect(adv).toMatchObject({ expression: "2d20kh1+5", kind: "advantage", natural: 17, total: 22 });
    expect(adv.dice[0]).toEqual({ type: "d20", value: 4, dropped: true });
    const dis = await engine().roll({ expression: "1d20", mode: "disadvantage" }, fakeRenderer(dice(4, 17)));
    expect(dis).toMatchObject({ kind: "disadvantage", total: 4 });
    const sum = await engine().roll({ expression: "2d20" }, fakeRenderer(dice(4, 17)));
    expect(sum).toMatchObject({ kind: "sum", total: 21 });
  });

  it("matches Dice Box's per-type results to the written terms", async () => {
    // Dice Box reports d6 before d20; the record still reads 2d20 then 1d6.
    const renderer = fakeRenderer([
      { type: "d6", value: 3 },
      { type: "d20", value: 20 },
      { type: "d20", value: 2 },
    ]);
    const record = await engine().roll({ expression: "2d20kh1+1d6+2" }, renderer);
    expect(record.dice.map((die) => [die.type, die.value, !!die.dropped])).toEqual([
      ["d20", 20, false],
      ["d20", 2, true],
      ["d6", 3, false],
    ]);
    expect(record.total).toBe(25);
    expect(record.critical).toBe("success");
  });

  it("supports repeated rolls", async () => {
    const e = engine([0, 1, 2, 3, 4, 5]);
    const totals = [];
    for (let i = 0; i < 3; i++)
      totals.push((await e.roll({ expression: "2d20" })).total);
    expect(totals).toEqual([3, 7, 11]);
  });
});
