/**
 * Shared dice types. These are the contract between the dice engine, the
 * renderer adapters, the floating tray, and future RPG modules (GM Console,
 * Character Studio, Encounter Manager). See
 * docs/architecture/029-DICE-RESULT-PROTOCOL.md.
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

/** Keep only the highest or lowest dice of a term, e.g. `2d20kh1`. */
export interface DiceKeep {
  mode: "highest" | "lowest";
  count: number;
}

/** One term of a parsed expression, e.g. `2d20` -> { type: "d20", count: 2 }. */
export interface DiceTerm {
  type: DieType;
  count: number;
  keep?: DiceKeep;
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
  /** Rolled but not counted (the lower die of an advantage roll, for example). */
  dropped?: boolean;
}

/**
 * `sum` adds the kept dice. `advantage` and `disadvantage` are d20 checks that
 * keep the highest or lowest of two d20s; they are never the same as a 2d20 sum.
 */
export type RollKind = "sum" | "advantage" | "disadvantage";

/** How a d20 check is rolled. */
export type RollMode = "normal" | "advantage" | "disadvantage";

/** A natural 20 or natural 1 on a single counted d20. */
export type CriticalResult = "success" | "failure";

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
  kind: RollKind;
  dice: DieResult[];
  modifier: number;
  /** Sum of the counted (not dropped) dice, before the modifier. */
  natural: number;
  total: number;
  visibility: RollVisibility;
  source: RollSource;
  resolution: RollResolution;
  timestamp: string;
  critical?: CriticalResult;
  /** The player who rolled, for player-private visibility. */
  playerId?: string;
  characterId?: string;
  npcId?: string;
  encounterId?: string;
  /** What the roll was for, e.g. "initiative", "attack", "damage". */
  purpose?: string;
  /** Short human label, e.g. "Mira — Stealth". */
  label?: string;
}

/** What callers ask the engine for. */
export interface RollRequest {
  expression: string;
  /** Advantage or disadvantage turns the expression's single d20 into 2d20kh1 / 2d20kl1. */
  mode?: RollMode;
  visibility?: RollVisibility;
  source?: RollSource;
  sessionId?: string | null;
  playerId?: string;
  characterId?: string;
  npcId?: string;
  encounterId?: string;
  purpose?: string;
  label?: string;
}

export type RollStatus = "idle" | "loading" | "rolling" | "error";
