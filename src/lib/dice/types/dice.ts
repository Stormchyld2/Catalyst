/**
 * Shared dice types. These are the contract between the dice engine, the
 * renderer adapters, the floating tray, and future RPG modules (GM Console,
 * Character Studio, Encounter Manager). See
 * docs/architecture/DICE-RESULT-PROTOCOL.md.
 */

/** Standard polyhedral dice. d100 is percentile (rendered as d100 + d10). */
export const DIE_TYPES = [
  "d4",
  "d6",
  "d8",
  "d10",
  "d12",
  "d20",
  "d100",
] as const;
export type DieType = (typeof DIE_TYPES)[number];

export const DIE_SIDES: Record<DieType, number> = {
  d4: 4,
  d6: 6,
  d8: 8,
  d10: 10,
  d12: 12,
  d20: 20,
  d100: 100,
};

/** One term of a parsed expression, e.g. `2d20` -> { type: "d20", count: 2 }. */
export interface DiceTerm {
  type: DieType;
  count: number;
}

/** A validated dice expression. */
export interface DiceExpression {
  /** Canonical notation, e.g. "2d20" or "2d6+1d4+3". */
  notation: string;
  terms: DiceTerm[];
  modifier: number;
}

/** A single settled die. `value` is always an integer in 1..sides. */
export interface DieResult {
  type: DieType;
  value: number;
}

export type RollVisibility = "public" | "gm-private" | "player-private";

/** Where a roll was requested from. Extend as RPG modules land. */
export type RollSource =
  | "dice-tray"
  | "gm-console"
  | "encounter"
  | "character"
  | "campaign";

/** How the outcome was resolved. */
export type RollResolution = "physics" | "random";

/** A resolved, recordable roll. Only created after successful resolution. */
export interface RollRecord {
  id: string;
  sessionId: string | null;
  expression: string;
  dice: DieResult[];
  modifier: number;
  /** Sum of natural dice values, before the modifier. */
  natural: number;
  total: number;
  visibility: RollVisibility;
  source: RollSource;
  resolution: RollResolution;
  timestamp: string;
  characterId?: string;
  encounterId?: string;
  purpose?: string;
}

/** What callers ask the engine for. */
export interface RollRequest {
  expression: string;
  visibility?: RollVisibility;
  source?: RollSource;
  sessionId?: string | null;
  characterId?: string;
  encounterId?: string;
  purpose?: string;
}

export type RollStatus = "idle" | "loading" | "rolling" | "error";
