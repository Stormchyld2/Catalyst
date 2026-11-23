import { describe, expect, it } from "vitest";
import { cryptoSource, randomDieValue } from "../engine/diceRandom";

const sequence = (...values: number[]) => {
  let index = 0;
  return () => values[index++];
};

describe("randomDieValue", () => {
  it("maps source values onto faces 1..sides", () => {
    expect(randomDieValue(20, sequence(0))).toBe(1);
    expect(randomDieValue(20, sequence(19))).toBe(20);
    expect(randomDieValue(20, sequence(20))).toBe(1);
  });

  it("rejects the partial top bucket so no face is favored", () => {
    const limit = 2 ** 32 - (2 ** 32 % 20);
    // The first value falls in the biased remainder and must be redrawn.
    expect(randomDieValue(20, sequence(limit, 6))).toBe(7);
  });

  it("gives up rather than looping forever on a broken source", () => {
    expect(() => randomDieValue(20, () => 2 ** 32 - 1)).toThrow(
      /Random source/,
    );
  });

  it("rejects invalid die sizes", () => {
    expect(() => randomDieValue(1)).toThrow(RangeError);
    expect(() => randomDieValue(2.5)).toThrow(RangeError);
  });

  it("produces valid faces from the crypto source", () => {
    for (let i = 0; i < 2000; i++) {
      const value = randomDieValue(20, cryptoSource);
      expect(Number.isInteger(value) && value >= 1 && value <= 20).toBe(true);
    }
  });
});
