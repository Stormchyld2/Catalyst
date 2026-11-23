/**
 * Dice notation parsing and validation.
 *
 * Supported: `NdX` terms for d4, d6, d8, d10, d12, d20 and d100 (`d%`), keep
 * highest/lowest (`2d20kh1`, `4d6kh3`), flat modifiers, and combinations such
 * as `2d20+1d6+3`. Terms keep the order they were written in. Plain terms of
 * the same die type merge into the first one (`1d6+1d6` is `2d6`); keep terms
 * never merge, because keeping applies to that group only.
 */
import {
  DIE_SIDES,
  DIE_TYPES,
  type DiceExpression,
  type DiceTerm,
  type DieType,
  type RollKind,
  type RollMode,
} from "../types/dice";

/** What the tray and engine can roll. */
export interface DiceCapabilities {
  dice: readonly DieType[];
  modifiers: boolean;
  keep: boolean;
}

/** Every die and feature, each checked end to end against Dice Box. */
export const FULL_CAPABILITIES: DiceCapabilities = {
  dice: DIE_TYPES,
  modifiers: true,
  keep: true,
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

const TERM = /^(\d*)d(\d+|%)(?:k([hl])(\d+))?$/i;
const FLAT = /^\d+$/;

function dieType(sides: string): DieType {
  const key = (sides === "%" ? "d100" : `d${Number(sides)}`) as DieType;
  if (!DIE_TYPES.includes(key)) throw new DiceNotationError(`Unsupported die: d${sides}`);
  return key;
}

/** Parse `notation` into terms and a modifier. */
export function parseDiceNotation(notation: string): DiceExpression {
  const source = notation.replace(/\s+/g, "").toLowerCase();
  if (!source) throw new DiceNotationError("Enter a dice expression, such as 2d20.");
  if (!/^[+-]?[0-9dkhl%]+([+-][0-9dkhl%]+)*$/.test(source))
    throw new DiceNotationError(`Invalid dice expression: ${notation}`);

  const terms: DiceTerm[] = [];
  let modifier = 0;
  for (const match of source.matchAll(/([+-]?)([^+-]+)/g)) {
    const sign = match[1] === "-" ? -1 : 1;
    const part = match[2];
    const term = TERM.exec(part);
    if (term) {
      if (sign < 0) throw new DiceNotationError("Subtracting dice is not supported.");
      const count = term[1] === "" ? 1 : Number(term[1]);
      if (!Number.isInteger(count) || count < 1)
        throw new DiceNotationError(`Invalid dice count in ${part}`);
      const type = dieType(term[2]);
      if (term[3]) {
        const keepCount = Number(term[4]);
        if (!Number.isInteger(keepCount) || keepCount < 1 || keepCount >= count)
          throw new DiceNotationError(`Keep fewer dice than you roll in ${part}`);
        terms.push({
          type,
          count,
          keep: { mode: term[3] === "h" ? "highest" : "lowest", count: keepCount },
        });
      } else {
        const same = terms.find((existing) => existing.type === type && !existing.keep);
        if (same) same.count += count;
        else terms.push({ type, count });
      }
    } else if (FLAT.test(part)) {
      modifier += sign * Number(part);
    } else {
      throw new DiceNotationError(`Invalid dice expression: ${notation}`);
    }
  }

  if (!terms.length) throw new DiceNotationError("A roll needs at least one die.");
  if (terms.some((term) => term.count > MAX_DICE_PER_TERM))
    throw new DiceNotationError(`Roll at most ${MAX_DICE_PER_TERM} dice of one type.`);
  if (terms.reduce((sum, term) => sum + term.count, 0) > MAX_DICE_PER_ROLL)
    throw new DiceNotationError(`Roll at most ${MAX_DICE_PER_ROLL} dice at once.`);
  if (Math.abs(modifier) > MAX_MODIFIER)
    throw new DiceNotationError(`Modifiers are limited to ±${MAX_MODIFIER}.`);

  return { notation: formatDiceNotation(terms, modifier), terms, modifier };
}

function formatTerm(term: DiceTerm): string {
  const keep = term.keep ? `k${term.keep.mode === "highest" ? "h" : "l"}${term.keep.count}` : "";
  return `${term.count}${term.type}${keep}`;
}

export function formatDiceNotation(terms: DiceTerm[], modifier = 0): string {
  const dice = terms
    .filter((term) => term.count > 0)
    .map(formatTerm)
    .join("+");
  if (!modifier) return dice;
  return `${dice}${modifier > 0 ? "+" : "-"}${Math.abs(modifier)}`;
}

/** Throws unless every part of the expression is enabled. */
export function assertSupported(
  expression: DiceExpression,
  capabilities: DiceCapabilities = FULL_CAPABILITIES
): void {
  const unsupported = expression.terms.find((term) => !capabilities.dice.includes(term.type));
  if (unsupported)
    throw new DiceNotationError(`${unsupported.type} rolls are not available yet.`);
  if (expression.modifier !== 0 && !capabilities.modifiers)
    throw new DiceNotationError("Modifiers are not available yet.");
  if (expression.terms.some((term) => term.keep) && !capabilities.keep)
    throw new DiceNotationError("Keep-highest and keep-lowest are not available yet.");
}

/** `2d20kh1` alone is advantage, `2d20kl1` disadvantage; anything else is a sum. */
export function rollKindOf(expression: DiceExpression): RollKind {
  const d20 = expression.terms.filter((term) => term.type === "d20");
  if (d20.length !== 1) return "sum";
  const [term] = d20;
  if (term.count !== 2 || term.keep?.count !== 1) return "sum";
  return term.keep.mode === "highest" ? "advantage" : "disadvantage";
}

/**
 * Apply advantage or disadvantage to a d20 check. The expression must contain
 * exactly one d20 (for example `1d20+5`); that die becomes `2d20kh1` or
 * `2d20kl1` and everything else is kept as written.
 */
export function applyRollMode(expression: DiceExpression, mode: RollMode = "normal"): DiceExpression {
  if (mode === "normal") return expression;
  const d20 = expression.terms.filter((term) => term.type === "d20");
  if (d20.length !== 1 || d20[0].count !== 1 || d20[0].keep)
    throw new DiceNotationError("Advantage and disadvantage apply to a single d20 roll.");
  const terms = expression.terms.map((term) =>
    term.type === "d20"
      ? { type: "d20" as const, count: 2, keep: { mode: mode === "advantage" ? ("highest" as const) : ("lowest" as const), count: 1 } }
      : term
  );
  return { notation: formatDiceNotation(terms, expression.modifier), terms, modifier: expression.modifier };
}

export function sidesOf(type: DieType): number {
  return DIE_SIDES[type];
}
