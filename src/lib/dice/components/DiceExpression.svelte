<script lang="ts">
  import { onMount } from "svelte";
  import { DiceNotationError } from "../engine/diceNotation";
  import { diceTray } from "../stores/diceTray.svelte";

  let { onRolled }: { onRolled?: () => void } = $props();

  let text = $state(diceTray.current?.expression ?? diceTray.selection ?? "");
  let problem = $state<string | null>(null);
  let input: HTMLInputElement;
  const EXAMPLES = ["1d20", "2d20", "4d6", "3d8+4", "2d20+1d6", "1d100", "4d6kh3"];

  function check(value: string): boolean {
    try {
      diceTray.engine.parse(value);
      problem = null;
      return true;
    } catch (error) {
      problem = error instanceof DiceNotationError ? error.message : String(error);
      return false;
    }
  }

  function roll(value = text) {
    text = value;
    if (!check(value)) return;
    void diceTray.roll({ expression: value, source: "dice-tray", visibility: diceTray.visibility });
    onRolled?.();
  }

  onMount(() => input.focus());
</script>

<section class="dice-popover" aria-label="Roll an expression" data-testid="dice-expression-panel">
  <h3>Roll an expression</h3>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      roll();
    }}
  >
    <input
      bind:this={input}
      bind:value={text}
      type="text"
      spellcheck="false"
      autocomplete="off"
      placeholder="e.g. 3d8+4"
      aria-label="Dice expression"
      aria-invalid={problem ? "true" : undefined}
      data-testid="dice-expression-input"
      oninput={() => (problem = null)}
    />
    <button type="submit" class="dice-text-button go" data-testid="dice-expression-roll">Roll</button>
  </form>
  {#if problem}
    <p class="problem" role="alert">{problem}</p>
  {/if}
  <div class="examples">
    {#each EXAMPLES as example (example)}
      <button type="button" class="dice-text-button" onclick={() => roll(example)}>{example}</button>
    {/each}
  </div>
  <p class="hint">
    d4, d6, d8, d10, d12, d20 and d100, combined with + and a modifier. <code>kh</code>/<code>kl</code> keep
    the highest or lowest dice.
  </p>
</section>

<style>
  h3 {
    font-family: inherit;
    margin: 0 0 8px;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--dice-muted);
  }
  form {
    display: flex;
    gap: 6px;
  }
  input {
    flex: 1;
    min-width: 0;
    padding: 6px 10px;
    border-radius: 8px;
    border: 1px solid var(--dice-line);
    background: #141416;
    color: var(--dice-text);
    font: 15px/1.2 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  }
  input:focus-visible {
    outline: 2px solid var(--dice-accent);
    outline-offset: 0;
  }
  .go {
    background: var(--dice-accent) !important;
    color: var(--dice-on-accent) !important;
    font-weight: 700;
  }
  .problem {
    margin: 6px 0 0;
    color: #f1b3aa;
    font-size: 12px;
  }
  .examples {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 8px;
  }
  .examples button {
    border: 1px solid var(--dice-line) !important;
    font-variant-numeric: tabular-nums;
  }
  .hint {
    margin: 8px 0 0;
    color: var(--dice-muted);
    font-size: 11px;
  }
  code {
    color: var(--dice-silver);
  }
</style>
