/**
 * Building and validating roll records. A record is created only from a
 * completed resolution, never from an animation in progress.
 */
import type {
  DiceExpression,
  DieResult,
  RollRecord,
  RollRequest,
  RollResolution,
} from "../types/dice";
import { sidesOf } from "./diceNotation";

export class DiceResultError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "DiceResultError";
  }
}

export function isValidDieValue(result: DieResult): boolean {
  return (
    Number.isInteger(result.value) &&
    result.value >= 1 &&
    result.value <= sidesOf(result.type)
  );
}

/**
 * Checks that `dice` is exactly what `expression` asked for: the right count of
 * each die type, every value a valid face.
 */
export function validateDice(
  expression: DiceExpression,
  dice: DieResult[],
): void {
  for (const term of expression.terms) {
    const found = dice.filter((die) => die.type === term.type).length;
    if (found !== term.count)
      throw new DiceResultError(
        `Expected ${term.count} ${term.type}, got ${found}.`,
      );
  }
  const expected = expression.terms.reduce((sum, term) => sum + term.count, 0);
  if (dice.length !== expected)
    throw new DiceResultError(`Expected ${expected} dice, got ${dice.length}.`);
  const invalid = dice.find((die) => !isValidDieValue(die));
  if (invalid)
    throw new DiceResultError(
      `Invalid ${invalid.type} value: ${invalid.value}`,
    );
}

export function sumDice(dice: DieResult[]): number {
  return dice.reduce((sum, die) => sum + die.value, 0);
}

export interface RecordContext {
  id: string;
  timestamp: string;
  resolution: RollResolution;
}

export function buildRollRecord(
  request: RollRequest,
  expression: DiceExpression,
  dice: DieResult[],
  context: RecordContext,
): RollRecord {
  validateDice(expression, dice);
  const natural = sumDice(dice);
  const record: RollRecord = {
    id: context.id,
    sessionId: request.sessionId ?? null,
    expression: expression.notation,
    dice: dice.map(({ type, value }) => ({ type, value })),
    modifier: expression.modifier,
    natural,
    total: natural + expression.modifier,
    visibility: request.visibility ?? "public",
    source: request.source ?? "dice-tray",
    resolution: context.resolution,
    timestamp: context.timestamp,
  };
  if (request.characterId) record.characterId = request.characterId;
  if (request.encounterId) record.encounterId = request.encounterId;
  if (request.purpose) record.purpose = request.purpose;
  return record;
}

/** Plain-text summary, e.g. for copying or screen readers. */
export function describeRoll(record: RollRecord): string {
  const lines = [`Roll: ${record.expression}`];
  record.dice.forEach((die, index) =>
    lines.push(`Die ${index + 1} (${die.type}): ${die.value}`),
  );
  if (record.modifier)
    lines.push(`Modifier: ${record.modifier > 0 ? "+" : ""}${record.modifier}`);
  lines.push(`Total: ${record.total}`);
  return lines.join("\n");
}
