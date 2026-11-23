// @3d-dice/dice-box 1.1.4 ships no type declarations. The adapter in
// engine/diceRenderer.ts narrows the class to the API it uses.
declare module "@3d-dice/dice-box" {
  const DiceBox: new (config: Record<string, unknown>) => unknown;
  export default DiceBox;
}
