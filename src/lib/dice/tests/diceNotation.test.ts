import { describe, expect, it } from "vitest";
import {
  applyRollMode,
  assertSupported,
  DiceNotationError,
  formatDiceNotation,
  FULL_CAPABILITIES,
  parseDiceNotation,
  rollKindOf,
} from "../engine/diceNotation";
import { DIE_TYPES } from "../types/dice";

describe("parseDiceNotation", () => {
  it("parses 2d20", () => {
    expect(parseDiceNotation("2d20")).toEqual({
      notation: "2d20",
      terms: [{ type: "d20", count: 2 }],
      modifier: 0,
    });
  });

  it("treats a bare die as one die and normalizes case and spaces", () => {
    expect(parseDiceNotation(" D20 ").notation).toBe("1d20");
  });

  it.each(DIE_TYPES)("parses every supported die: %s", (type) => {
    const parsed = parseDiceNotation(`3${type}`);
    expect(parsed.terms).toEqual([{ type, count: 3 }]);
  });

  it.each([
    ["2d20", "2d20"],
    ["4d6", "4d6"],
    ["3d8+4", "3d8+4"],
    ["2d20+1d6", "2d20+1d6"],
    ["1d4+1d6+1d8+1d10+1d12+1d20+1d100", "1d4+1d6+1d8+1d10+1d12+1d20+1d100"],
    ["1d20 - 2", "1d20-2"],
  ])("keeps %j as written", (input, notation) => {
    expect(parseDiceNotation(input).notation).toBe(notation);
  });

  it("merges repeated plain types into the first term, in written order", () => {
    const parsed = parseDiceNotation("2d6 + 1d4 + 1d6 - 2 + 5");
    expect(parsed.terms).toEqual([
      { type: "d6", count: 3 },
      { type: "d4", count: 1 },
    ]);
    expect(parsed.modifier).toBe(3);
    expect(parsed.notation).toBe("3d6+1d4+3");
  });

  it("reads d% as percentile", () => {
    expect(parseDiceNotation("1d%").terms).toEqual([{ type: "d100", count: 1 }]);
  });

  it("parses keep-highest and keep-lowest, which never merge", () => {
    const parsed = parseDiceNotation("4d6kh3+1d6");
    expect(parsed.terms).toEqual([
      { type: "d6", count: 4, keep: { mode: "highest", count: 3 } },
      { type: "d6", count: 1 },
    ]);
    expect(parseDiceNotation("2d20KL1").terms[0].keep).toEqual({ mode: "lowest", count: 1 });
  });

  it.each(["", "d", "2x20", "2d7", "0d20", "2d20++1", "-1d20", "2d20*2", "abc", "2d20kh2", "2d20kh0", "1d20kh1"])(
    "rejects %j",
    (input) => {
      expect(() => parseDiceNotation(input)).toThrow(DiceNotationError);
    }
  );

  it("rejects modifiers alone and oversized rolls", () => {
    expect(() => parseDiceNotation("+5")).toThrow(/at least one die/);
    expect(() => parseDiceNotation("21d20")).toThrow(/at most 20/);
    expect(() => parseDiceNotation("20d20+20d6+1d4")).toThrow(/at most 40/);
    expect(() => parseDiceNotation("2d20+1000")).toThrow(/limited/);
  });
});

describe("rollKindOf and applyRollMode", () => {
  it("tells advantage and disadvantage apart from a 2d20 sum", () => {
    expect(rollKindOf(parseDiceNotation("2d20"))).toBe("sum");
    expect(rollKindOf(parseDiceNotation("2d20kh1+5"))).toBe("advantage");
    expect(rollKindOf(parseDiceNotation("2d20kl1"))).toBe("disadvantage");
    expect(rollKindOf(parseDiceNotation("4d6kh3"))).toBe("sum");
  });

  it("turns a single d20 into 2d20kh1 or 2d20kl1 and keeps the rest", () => {
    const base = parseDiceNotation("1d20+1d6+3");
    expect(applyRollMode(base, "advantage").notation).toBe("2d20kh1+1d6+3");
    expect(applyRollMode(base, "disadvantage").notation).toBe("2d20kl1+1d6+3");
    expect(applyRollMode(base, "normal")).toBe(base);
  });

  it("refuses advantage on anything but one d20", () => {
    expect(() => applyRollMode(parseDiceNotation("2d20"), "advantage")).toThrow(/single d20/);
    expect(() => applyRollMode(parseDiceNotation("4d6"), "disadvantage")).toThrow(/single d20/);
  });
});

describe("assertSupported", () => {
  it("allows every die, modifiers and keep rules by default", () => {
    expect(() => assertSupported(parseDiceNotation("1d4+1d6+1d8+1d10+1d12+1d20+1d100+3"))).not.toThrow();
    expect(() => assertSupported(parseDiceNotation("4d6kh3"), FULL_CAPABILITIES)).not.toThrow();
  });

  it("follows narrower capabilities when given", () => {
    const narrow = { dice: ["d20"] as const, modifiers: false, keep: false };
    expect(() => assertSupported(parseDiceNotation("4d6"), narrow)).toThrow(/d6 rolls are not available/);
    expect(() => assertSupported(parseDiceNotation("1d20+3"), narrow)).toThrow(/Modifiers/);
    expect(() => assertSupported(parseDiceNotation("2d20kh1"), narrow)).toThrow(/Keep-highest/);
  });
});

describe("formatDiceNotation", () => {
  it("drops empty terms, signs the modifier and writes keep rules", () => {
    expect(
      formatDiceNotation(
        [
          { type: "d20", count: 2, keep: { mode: "highest", count: 1 } },
          { type: "d6", count: 0 },
        ],
        -1
      )
    ).toBe("2d20kh1-1");
  });
});
