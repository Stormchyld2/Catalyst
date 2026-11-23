/**
 * The shared dice engine. Every Catalyst surface that rolls dice (the floating
 * tray, and the GM Console, encounters and characters through the integration
 * API) goes through here, so parsing, validation, resolution and record
 * building have one owner.
 */
import type { DiceExpression, DieResult, RollMode, RollRecord, RollRequest } from "../types/dice";
import {
  applyRollMode,
  assertSupported,
  FULL_CAPABILITIES,
  parseDiceNotation,
  sidesOf,
  type DiceCapabilities,
} from "./diceNotation";
import { cryptoSource, randomDieValue, type RandomSource } from "./diceRandom";
import { buildRollRecord } from "./diceResults";
import type { DiceRenderer } from "./diceRenderer";

export interface DiceEngineOptions {
  random?: RandomSource;
  now?: () => Date;
  newId?: () => string;
  capabilities?: DiceCapabilities;
}

export class DiceEngine {
  private readonly random: RandomSource;
  private readonly now: () => Date;
  private readonly newId: () => string;
  readonly capabilities: DiceCapabilities;

  constructor(options: DiceEngineOptions = {}) {
    this.random = options.random ?? cryptoSource;
    this.now = options.now ?? (() => new Date());
    this.newId = options.newId ?? (() => `roll-${crypto.randomUUID()}`);
    this.capabilities = options.capabilities ?? FULL_CAPABILITIES;
  }

  /** Parse, apply advantage/disadvantage, and check what this build can roll. */
  parse(notation: string, mode: RollMode = "normal"): DiceExpression {
    const expression = applyRollMode(parseDiceNotation(notation), mode);
    assertSupported(expression, this.capabilities);
    return expression;
  }

  /** Resolve values without animation, from the unbiased random source. */
  resolveRandom(expression: DiceExpression): DieResult[] {
    return expression.terms.flatMap((term) =>
      Array.from({ length: term.count }, () => ({
        type: term.type,
        value: randomDieValue(sidesOf(term.type), this.random),
      }))
    );
  }

  /**
   * Roll `request`. With a renderer, the settled physics faces are the result;
   * without one, the random source is. Rejects (and records nothing) if the
   * expression is invalid, the roll is abandoned, or the renderer reports
   * values that do not match the request.
   */
  async roll(request: RollRequest, renderer: DiceRenderer | null = null): Promise<RollRecord> {
    const expression = this.parse(request.expression, request.mode);
    const dice = renderer ? await renderer.roll(expression) : this.resolveRandom(expression);
    return buildRollRecord(request, expression, dice, {
      id: this.newId(),
      timestamp: this.now().toISOString(),
      resolution: renderer ? "physics" : "random",
    });
  }
}
