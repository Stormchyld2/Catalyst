<script lang="ts">
  import type { RollRecord, RollStatus } from "../types/dice";
  import { describeRoll } from "../engine/diceResults";

  let {
    record,
    status,
    error,
    compact = false,
  }: {
    record: RollRecord | null;
    status: RollStatus;
    error: string | null;
    compact?: boolean;
  } = $props();

  const natural = (record: RollRecord, index: number) => {
    const die = record.dice[index];
    if (die.type !== "d20") return "";
    return die.value === 20 ? "nat-high" : die.value === 1 ? "nat-low" : "";
  };
</script>

<div class="dice-result" class:dice-result--compact={compact} data-testid="dice-result">
  {#if error}
    <span class="dice-pill dice-pill--error" role="alert">{error}</span>
  {:else if status === "rolling"}
    <span class="dice-pill" role="status">Rolling…</span>
  {:else if record}
    <span class="dice-pill dice-pill--total" data-testid="dice-total">
      Total: <strong>{record.total}</strong>
    </span>
    <span class="dice-pill" data-testid="dice-breakdown" title={describeRoll(record)}>
      <span class="dice-pill__label">{record.expression}</span>
      {#each record.dice as die, index (index)}
        <span class="dice-value {natural(record, index)}" data-testid="dice-value">{die.value}</span>
      {/each}
      {#if record.modifier}
        <span class="dice-value dice-value--mod">{record.modifier > 0 ? "+" : ""}{record.modifier}</span>
      {/if}
    </span>
  {:else}
    <span class="dice-pill dice-pill--quiet">Ready to roll</span>
  {/if}
</div>

<style>
  .dice-result {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 8px;
    pointer-events: none;
  }
  .dice-pill {
    pointer-events: auto;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    min-height: 34px;
    padding: 4px 16px;
    border-radius: 999px;
    border: 1px solid var(--dice-line);
    background: var(--dice-glass);
    backdrop-filter: blur(10px);
    color: var(--dice-text);
    font-size: 16px;
    font-variant-numeric: tabular-nums;
  }
  .dice-result--compact .dice-pill {
    font-size: 14px;
    min-height: 30px;
    padding: 3px 12px;
  }
  .dice-pill--total strong {
    color: var(--dice-gold);
    font-size: 1.15em;
  }
  .dice-pill--quiet {
    color: var(--dice-muted);
  }
  .dice-pill--error {
    border-color: color-mix(in srgb, #e58b7f 60%, transparent);
    color: #f1b3aa;
    font-size: 13px;
  }
  .dice-pill__label {
    color: var(--dice-muted);
    margin-right: 2px;
  }
  .dice-value {
    min-width: 28px;
    padding: 1px 6px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.07);
    text-align: center;
    font-weight: 600;
  }
  .dice-value.nat-high {
    color: #1b1408;
    background: var(--dice-gold);
  }
  .dice-value.nat-low {
    color: #f1b3aa;
    background: rgba(229, 139, 127, 0.18);
  }
  .dice-value--mod {
    background: transparent;
    color: var(--dice-muted);
  }
</style>
