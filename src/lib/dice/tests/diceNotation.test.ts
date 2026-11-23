import { describe, expect, it } from "vitest";
import {
  assertSupported,
  DiceNotationError,
  formatDiceNotation,
  parseDiceNotation,
  PHASE_1_CAPABILITIES,
} from "../engine/diceNotation";

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

  it("parses mixed dice with modifiers and merges repeated types", () => {
    const parsed = parseDiceNotation("2d6 + 1d4 + 1d6 - 2 + 5");
    expect(parsed.terms).toEqual([
      { type: "d4", count: 1 },
      { type: "d6", count: 3 },
    ]);
    expect(parsed.modifier).toBe(3);
    expect(parsed.notation).toBe("1d4+3d6+3");
  });

  it("reads d% as percentile", () => {
    expect(parseDiceNotation("1d%").terms).toEqual([
      { type: "d100", count: 1 },
    ]);
  });

  it.each([
    "",
    "d",
    "2x20",
    "2d7",
    "0d20",
    "2d20++1",
    "-1d20",
    "2d20*2",
    "abc",
  ])("rejects %j", (input) => {
    expect(() => parseDiceNotation(input)).toThrow(DiceNotationError);
  });

  it("rejects modifiers alone and oversized rolls", () => {
    expect(() => parseDiceNotation("+5")).toThrow(/at least one die/);
    expect(() => parseDiceNotation("21d20")).toThrow(/at most 20/);
    expect(() => parseDiceNotation("2d20+1000")).toThrow(/limited/);
  });
});

describe("assertSupported", () => {
  it("allows d20 rolls in phase 1", () => {
    expect(() =>
      assertSupported(parseDiceNotation("2d20"), PHASE_1_CAPABILITIES),
    ).not.toThrow();
  });

  it("blocks dice and modifiers that phase 1 has not validated", () => {
    expect(() => assertSupported(parseDiceNotation("4d6"))).toThrow(
      /d6 rolls are not available/,
    );
    expect(() => assertSupported(parseDiceNotation("1d20+3"))).toThrow(
      /Modifiers/,
    );
  });

  it("follows wider capabilities when given", () => {
    const wide = { dice: ["d6", "d20"] as const, modifiers: true };
    expect(() =>
      assertSupported(parseDiceNotation("4d6+2"), wide),
    ).not.toThrow();
  });
});

describe("formatDiceNotation", () => {
  it("drops empty terms and signs the modifier", () => {
    expect(
      formatDiceNotation(
        [
          { type: "d20", count: 2 },
          { type: "d6", count: 0 },
        ],
        -1,
      ),
    ).toBe("2d20-1");
  });
});
