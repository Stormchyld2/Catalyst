/**
 * Unbiased random die values for rolls that are not resolved by physics
 * (compact mode, reduced motion, or when WebGL is unavailable).
 *
 * Uses crypto.getRandomValues with rejection sampling, so every face is
 * equally likely. Tests inject a deterministic source instead.
 */

/** Returns uniformly distributed unsigned 32-bit integers. */
export type RandomSource = () => number;

export const cryptoSource: RandomSource = () => {
  const buffer = new Uint32Array(1);
  crypto.getRandomValues(buffer);
  return buffer[0];
};

const RANGE = 0x1_0000_0000; // 2^32

/** An integer in 1..sides with no modulo bias. */
export function randomDieValue(
  sides: number,
  source: RandomSource = cryptoSource,
): number {
  if (!Number.isInteger(sides) || sides < 2)
    throw new RangeError(`Invalid die size: ${sides}`);
  // Reject the top partial bucket so every face maps to the same number of inputs.
  const limit = RANGE - (RANGE % sides);
  for (let attempt = 0; attempt < 64; attempt++) {
    const value = source() >>> 0;
    if (value < limit) return (value % sides) + 1;
  }
  throw new Error("Random source failed to produce a usable value.");
}
