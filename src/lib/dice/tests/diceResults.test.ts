import { describe, expect, it } from "vitest";
import { parseDiceNotation } from "../engine/diceNotation";
import {
  buildRollRecord,
  DiceResultError,
  describeRoll,
  validateDice,
} from "../engine/diceResults";

const context = {
  id: "roll-1",
  timestamp: "2026-11-23T12:00:00.000Z",
  resolution: "physics" as const,
};

describe("buildRollRecord", () => {
  it("records each die and the summed total for 2d20", () => {
    const record = buildRollRecord(
      { expression: "2d20" },
      parseDiceNotation("2d20"),
      [
        { type: "d20", value: 14 },
        { type: "d20", value: 7 },
      ],
      context,
    );
    expect(record).toMatchObject({
      id: "roll-1",
      expression: "2d20",
      dice: [
        { type: "d20", value: 14 },
        { type: "d20", value: 7 },
      ],
      modifier: 0,
      natural: 21,
      total: 21,
      visibility: "public",
      source: "dice-tray",
      resolution: "physics",
      sessionId: null,
    });
    expect(describeRoll(record)).toBe(
      "Roll: 2d20\nDie 1 (d20): 14\nDie 2 (d20): 7\nTotal: 21",
    );
  });

  it("keeps the modifier separate from the natural dice", () => {
    const record = buildRollRecord(
      {
        expression: "2d6+1d4+3",
        visibility: "gm-private",
        source: "gm-console",
        purpose: "damage",
      },
      parseDiceNotation("2d6+1d4+3"),
      [
        { type: "d6", value: 6 },
        { type: "d6", value: 1 },
        { type: "d4", value: 2 },
      ],
      context,
    );
    expect(record.natural).toBe(9);
    expect(record.modifier).toBe(3);
    expect(record.total).toBe(12);
    expect(record.visibility).toBe("gm-private");
    expect(record.purpose).toBe("damage");
  });
});

describe("validateDice", () => {
  const twoD20 = parseDiceNotation("2d20");

  it.each([0, 21, 3.5, Number.NaN])("rejects a d20 value of %s", (value) => {
    expect(() =>
      validateDice(twoD20, [
        { type: "d20", value: 10 },
        { type: "d20", value },
      ]),
    ).toThrow(DiceResultError);
  });

  it("rejects missing, extra or wrong dice", () => {
    expect(() => validateDice(twoD20, [{ type: "d20", value: 3 }])).toThrow(
      /Expected 2 d20/,
    );
    expect(() =>
      validateDice(twoD20, [
        { type: "d20", value: 3 },
        { type: "d6", value: 3 },
      ]),
    ).toThrow(DiceResultError);
  });
});
