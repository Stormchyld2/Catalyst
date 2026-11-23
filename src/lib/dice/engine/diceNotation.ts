/**
 * Dice notation parsing and validation.
 *
 * Phase 1 enables `Nd20`. The parser already understands `NdX` terms for the
 * standard dice plus a flat modifier (`2d6+1d4+3`), but the tray only offers
 * what has been validated end to end (see PHASE_1_CAPABILITIES).
 */
import {
  DIE_SIDES,
  DIE_TYPES,
  type DiceExpression,
  type DiceTerm,
  type DieType,
} from "../types/dice";

/** What the tray can roll today. Phase 2 widens this. */
export interface DiceCapabilities {
  dice: readonly DieType[];
  modifiers: boolean;
}

export const PHASE_1_CAPABILITIES: DiceCapabilities = {
  dice: ["d20"],
  modifiers: false,
};

export const MAX_DICE_PER_TERM = 20;
export const MAX_DICE_PER_ROLL = 40;
export const MAX_MODIFIER = 999;

export class DiceNotationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "DiceNotationError";
  }
}

const TERM = /^(\d*)d(\d+|%)$/i;
const FLAT = /^\d+$/;

function dieType(sides: string): DieType {
  const key = (sides === "%" ? "d100" : `d${Number(sides)}`) as DieType;
  if (!DIE_TYPES.includes(key))
    throw new DiceNotationError(`Unsupported die: d${sides}`);
  return key;
}

/**
 * Parse `notation` into terms and a modifier. Terms of the same die type are
 * merged, so `1d20+1d20` and `2d20` are the same expression.
 */
export function parseDiceNotation(notation: string): DiceExpression {
  const source = notation.replace(/\s+/g, "").toLowerCase();
  if (!source)
    throw new DiceNotationError("Enter a dice expression, such as 2d20.");
  if (!/^[+-]?[0-9d%]+([+-][0-9d%]+)*$/.test(source))
    throw new DiceNotationError(`Invalid dice expression: ${notation}`);

  const counts = new Map<DieType, number>();
  let modifier = 0;
  for (const match of source.matchAll(/([+-]?)([^+-]+)/g)) {
    const sign = match[1] === "-" ? -1 : 1;
    const part = match[2];
    const term = TERM.exec(part);
    if (term) {
      if (sign < 0)
        throw new DiceNotationError("Subtracting dice is not supported yet.");
      const count = term[1] === "" ? 1 : Number(term[1]);
      if (!Number.isInteger(count) || count < 1)
        throw new DiceNotationError(`Invalid dice count in ${part}`);
      const type = dieType(term[2]);
      counts.set(type, (counts.get(type) ?? 0) + count);
    } else if (FLAT.test(part)) {
      modifier += sign * Number(part);
    } else {
      throw new DiceNotationError(`Invalid dice expression: ${notation}`);
    }
  }

  const terms: DiceTerm[] = DIE_TYPES.filter((type) => counts.has(type)).map(
    (type) => ({
      type,
      count: counts.get(type)!,
    }),
  );
  if (!terms.length)
    throw new DiceNotationError("A roll needs at least one die.");
  if (terms.some((term) => term.count > MAX_DICE_PER_TERM))
    throw new DiceNotationError(
      `Roll at most ${MAX_DICE_PER_TERM} dice of one type.`,
    );
  if (terms.reduce((sum, term) => sum + term.count, 0) > MAX_DICE_PER_ROLL)
    throw new DiceNotationError(
      `Roll at most ${MAX_DICE_PER_ROLL} dice at once.`,
    );
  if (Math.abs(modifier) > MAX_MODIFIER)
    throw new DiceNotationError(`Modifiers are limited to ±${MAX_MODIFIER}.`);

  return { notation: formatDiceNotation(terms, modifier), terms, modifier };
}

export function formatDiceNotation(terms: DiceTerm[], modifier = 0): string {
  const dice = terms
    .filter((term) => term.count > 0)
    .map((term) => `${term.count}${term.type}`)
    .join("+");
  if (!modifier) return dice;
  return `${dice}${modifier > 0 ? "+" : "-"}${Math.abs(modifier)}`;
}

/** Throws unless every part of the expression is enabled. */
export function assertSupported(
  expression: DiceExpression,
  capabilities: DiceCapabilities = PHASE_1_CAPABILITIES,
): void {
  const unsupported = expression.terms.find(
    (term) => !capabilities.dice.includes(term.type),
  );
  if (unsupported)
    throw new DiceNotationError(
      `${unsupported.type} rolls are not available yet.`,
    );
  if (expression.modifier !== 0 && !capabilities.modifiers)
    throw new DiceNotationError("Modifiers are not available yet.");
}

export function sidesOf(type: DieType): number {
  return DIE_SIDES[type];
}
