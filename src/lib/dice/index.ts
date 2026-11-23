/**
 * Public entry point for Catalyst's shared dice roller. RPG modules import
 * from here, not from the internal folders.
 */
export {
  diceTray,
  type TrayMode,
  type DiceSettings,
  type RollOptions,
} from "./stores/diceTray.svelte";
export { DiceEngine } from "./engine/diceEngine";
export { parseDiceNotation, DiceNotationError } from "./engine/diceNotation";
export { describeRoll } from "./engine/diceResults";
export {
  abilityCheck,
  attackRoll,
  d20Check,
  damageRoll,
  encounterRoll,
  initiativeRoll,
  savingThrow,
  skillCheck,
  type CheckOptions,
  type RollActor,
} from "./integration/rollRequests";
export { canView, visibleTo, type RollViewer } from "./integration/visibility";
export type { DiceSessionInfo, DiceSessionHistory } from "./integration/campaignSessions";
export type {
  DieType,
  DieResult,
  RollKind,
  RollMode,
  RollRecord,
  RollRequest,
  RollSource,
  RollVisibility,
} from "./types/dice";
