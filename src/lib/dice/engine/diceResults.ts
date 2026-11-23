/**
 * Building and validating roll records. A record is created only from a
 * completed resolution, never from an animation in progress.
 */
import type {
  CriticalResult,
  DiceExpression,
  DieResult,
  RollRecord,
  RollRequest,
  RollResolution,
} from "../types/dice";
import { rollKindOf, sidesOf } from "./diceNotation";

export class DiceResultError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "DiceResultError";
  }
}

export function isValidDieValue(result: DieResult): boolean {
  return (
    Number.isInteger(result.value) && result.value >= 1 && result.value <= sidesOf(result.type)
  );
}

/**
 * Checks that `dice` is exactly what `expression` asked for: the right count of
 * each die type, every value a valid face.
 */
export function validateDice(expression: DiceExpression, dice: DieResult[]): void {
  const wanted = new Map<string, number>();
  for (const term of expression.terms)
    wanted.set(term.type, (wanted.get(term.type) ?? 0) + term.count);
  for (const [type, count] of wanted) {
    const found = dice.filter((die) => die.type === type).length;
    if (found !== count) throw new DiceResultError(`Expected ${count} ${type}, got ${found}.`);
  }
  const expected = [...wanted.values()].reduce((sum, count) => sum + count, 0);
  if (dice.length !== expected)
    throw new DiceResultError(`Expected ${expected} dice, got ${dice.length}.`);
  const invalid = dice.find((die) => !isValidDieValue(die));
  if (invalid) throw new DiceResultError(`Invalid ${invalid.type} value: ${invalid.value}`);
}

/**
 * Assign settled dice to the expression's terms and mark the dice that keep
 * rules drop. Dice Box reports dice grouped by type rather than by term, so
 * dice are taken per type in term order. Returns dice in term order.
 */
export function applyKeepRules(expression: DiceExpression, dice: DieResult[]): DieResult[] {
  const pools = new Map<string, DieResult[]>();
  for (const die of dice) {
    const pool = pools.get(die.type) ?? [];
    pool.push({ type: die.type, value: die.value });
    pools.set(die.type, pool);
  }
  return expression.terms.flatMap((term) => {
    const group = (pools.get(term.type) ?? []).splice(0, term.count);
    if (!term.keep) return group;
    const order = group
      .map((die, index) => ({ die, index }))
      .sort((a, b) =>
        term.keep!.mode === "highest" ? b.die.value - a.die.value : a.die.value - b.die.value
      );
    const keep = new Set(order.slice(0, term.keep.count).map((entry) => entry.index));
    return group.map((die, index) => (keep.has(index) ? die : { ...die, dropped: true }));
  });
}

export function sumDice(dice: DieResult[]): number {
  return dice.reduce((sum, die) => sum + (die.dropped ? 0 : die.value), 0);
}

/** A natural 20 or 1 counts as critical when exactly one d20 is counted. */
export function criticalOf(dice: DieResult[]): CriticalResult | undefined {
  const counted = dice.filter((die) => die.type === "d20" && !die.dropped);
  if (counted.length !== 1) return undefined;
  if (counted[0].value === 20) return "success";
  if (counted[0].value === 1) return "failure";
  return undefined;
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
  context: RecordContext
): RollRecord {
  validateDice(expression, dice);
  const settled = applyKeepRules(expression, dice);
  const natural = sumDice(settled);
  const record: RollRecord = {
    id: context.id,
    sessionId: request.sessionId ?? null,
    expression: expression.notation,
    kind: rollKindOf(expression),
    dice: settled,
    modifier: expression.modifier,
    natural,
    total: natural + expression.modifier,
    visibility: request.visibility ?? "public",
    source: request.source ?? "dice-tray",
    resolution: context.resolution,
    timestamp: context.timestamp,
  };
  const critical = criticalOf(settled);
  if (critical) record.critical = critical;
  for (const key of ["playerId", "characterId", "npcId", "encounterId", "purpose", "label"] as const) {
    if (request[key]) record[key] = request[key];
  }
  return record;
}

const KIND_LABEL = { sum: "", advantage: " (advantage)", disadvantage: " (disadvantage)" };

/** Plain-text summary, e.g. for copying or screen readers. */
export function describeRoll(record: RollRecord): string {
  const lines = [`Roll: ${record.expression}${KIND_LABEL[record.kind]}`];
  record.dice.forEach((die, index) =>
    lines.push(`Die ${index + 1} (${die.type}): ${die.value}${die.dropped ? " (dropped)" : ""}`)
  );
  if (record.modifier) lines.push(`Modifier: ${record.modifier > 0 ? "+" : ""}${record.modifier}`);
  lines.push(`Total: ${record.total}`);
  if (record.critical) lines.push(record.critical === "success" ? "Critical success" : "Critical failure");
  return lines.join("\n");
}
