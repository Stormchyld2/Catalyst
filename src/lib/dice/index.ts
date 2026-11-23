/**
 * Public entry point for Catalyst's shared dice roller. RPG modules import
 * from here, not from the internal folders.
 */
export {
  diceTray,
  type TrayMode,
  type DiceSettings,
} from "./stores/diceTray.svelte";
export { DiceEngine } from "./engine/diceEngine";
export { parseDiceNotation, DiceNotationError } from "./engine/diceNotation";
export { describeRoll } from "./engine/diceResults";
export type {
  DieType,
  DieResult,
  RollRecord,
  RollRequest,
  RollSource,
  RollVisibility,
} from "./types/dice";
