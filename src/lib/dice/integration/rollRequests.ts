/**
 * Roll requests for RPG modules: GM Console, Character Studio and Adventure
 * Studio (encounters). Each helper builds a RollRequest with the right purpose, label and
 * context; modifiers come from Catalyst's rules engine, not from the dice
 * roller. Roll them with the shared tray:
 *
 *   const record = await diceTray.roll(attackRoll(goblin, { bonus: 4 }), { show: true });
 *
 * See docs/architecture/030-DICE-GM-CONSOLE-INTEGRATION.md.
 */
import { applyRollMode, formatDiceNotation, parseDiceNotation } from "../engine/diceNotation";
import type { RollMode, RollRequest, RollSource, RollVisibility } from "../types/dice";

/** Who is rolling. `id` is the character's or NPC's stable ID. */
export interface RollActor {
  kind: "character" | "npc";
  id: string;
  name?: string;
  /** The player controlling a character, for player-private rolls. */
  playerId?: string;
}

export interface CheckOptions {
  bonus?: number;
  mode?: RollMode;
  visibility?: RollVisibility;
  source?: RollSource;
  encounterId?: string;
}

/** NPC rolls default to GM-private; character rolls default to public. */
function defaultVisibility(actor: RollActor | undefined): RollVisibility {
  return actor?.kind === "npc" ? "gm-private" : "public";
}

function withBonus(base: string, bonus = 0): string {
  if (!Number.isInteger(bonus)) throw new RangeError(`Bonus must be a whole number: ${bonus}`);
  if (!bonus) return base;
  return `${base}${bonus > 0 ? "+" : "-"}${Math.abs(bonus)}`;
}

function context(actor: RollActor | undefined, purpose: string, label: string, options: CheckOptions) {
  const request: Omit<RollRequest, "expression"> = {
    purpose,
    label: actor?.name ? `${actor.name} — ${label}` : label,
    visibility: options.visibility ?? defaultVisibility(actor),
    source: options.source ?? (options.encounterId ? "encounter" : actor ? actor.kind === "npc" ? "gm-console" : "character" : "gm-console"),
  };
  if (options.mode && options.mode !== "normal") request.mode = options.mode;
  if (options.encounterId) request.encounterId = options.encounterId;
  if (actor?.kind === "character") request.characterId = actor.id;
  if (actor?.kind === "npc") request.npcId = actor.id;
  if (actor?.playerId) request.playerId = actor.playerId;
  return request;
}

/** A d20 check with a bonus: the shared shape of attacks, saves and skills. */
export function d20Check(
  actor: RollActor | undefined,
  purpose: string,
  label: string,
  options: CheckOptions = {}
): RollRequest {
  return { expression: withBonus("1d20", options.bonus), ...context(actor, purpose, label, options) };
}

export function initiativeRoll(actor: RollActor, options: CheckOptions = {}): RollRequest {
  return d20Check(actor, "initiative", "Initiative", options);
}

export function attackRoll(actor: RollActor, options: CheckOptions & { weapon?: string } = {}): RollRequest {
  return d20Check(actor, "attack", options.weapon ? `${options.weapon} attack` : "Attack", options);
}

export function savingThrow(actor: RollActor, ability: string, options: CheckOptions = {}): RollRequest {
  return d20Check(actor, "saving-throw", `${ability} save`, options);
}

export function skillCheck(actor: RollActor, skill: string, options: CheckOptions = {}): RollRequest {
  return d20Check(actor, "skill-check", skill, options);
}

export function abilityCheck(actor: RollActor, ability: string, options: CheckOptions = {}): RollRequest {
  return d20Check(actor, "ability-check", `${ability} check`, options);
}

/**
 * Damage, e.g. `damageRoll(actor, "1d8+3")`. On a critical hit the dice are
 * doubled and the modifier is not (`2d8+3`), the common d20 rule; pass
 * `critical: false` to opt out.
 */
export function damageRoll(
  actor: RollActor,
  expression: string,
  options: Omit<CheckOptions, "bonus" | "mode"> & { critical?: boolean; label?: string } = {}
): RollRequest {
  const parsed = parseDiceNotation(expression);
  const terms = options.critical ? parsed.terms.map((term) => ({ ...term, count: term.count * 2 })) : parsed.terms;
  return {
    expression: formatDiceNotation(terms, parsed.modifier),
    ...context(actor, "damage", options.label ?? (options.critical ? "Critical damage" : "Damage"), options),
  };
}

/** Any expression rolled for an encounter, e.g. a random-encounter table or a trap. */
export function encounterRoll(
  encounterId: string,
  expression: string,
  purpose: string,
  options: Omit<CheckOptions, "encounterId" | "bonus"> & { label?: string } = {}
): RollRequest {
  applyRollMode(parseDiceNotation(expression), options.mode); // validate early
  return {
    expression,
    ...context(undefined, purpose, options.label ?? purpose, {
      ...options,
      encounterId,
      visibility: options.visibility ?? "gm-private",
    }),
  };
}
