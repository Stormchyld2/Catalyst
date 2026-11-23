<script lang="ts">
  import { Lock } from "lucide-svelte";
  import type { DieResult, RollRecord, RollStatus } from "../types/dice";
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

  const KIND = { sum: "", advantage: "Advantage", disadvantage: "Disadvantage" } as const;

  function natural(die: DieResult) {
    if (die.type !== "d20" || die.dropped) return "";
    return die.value === 20 ? "nat-high" : die.value === 1 ? "nat-low" : "";
  }
</script>

<div
  class="dice-result"
  class:dice-result--compact={compact}
  data-testid="dice-result"
  aria-live="off"
>
  {#if error}
    <p class="dice-result__message dice-result__message--error" role="alert">{error}</p>
  {:else if status === "rolling"}
    <p class="dice-result__message" role="status">Rolling…</p>
  {:else if record}
    <div
      class="dice-result__panel"
      class:crit-success={record.critical === "success"}
      class:crit-failure={record.critical === "failure"}
      title={describeRoll(record)}
    >
      <div class="dice-result__meta">
        <span class="dice-result__expr" data-testid="dice-expression">{record.expression}</span>
        {#if record.kind !== "sum"}
          <span class="dice-result__badge" data-testid="dice-kind">{KIND[record.kind]}</span>
        {/if}
        {#if record.visibility !== "public"}
          <span class="dice-result__badge dice-result__badge--private" title="Only the GM sees this roll">
            <Lock size={11} aria-hidden="true" /> GM only
          </span>
        {/if}
        {#if record.label}
          <span class="dice-result__label">{record.label}</span>
        {/if}
        {#if record.critical}
          <span class="dice-result__crit" data-testid="dice-critical">
            {record.critical === "success" ? "Critical success" : "Critical failure"}
          </span>
        {/if}
      </div>
      <div class="dice-result__line" data-testid="dice-breakdown">
        <span class="dice-result__dice">
          {#each record.dice as die, index (index)}
            <span
              class="dice-chip {natural(die)}"
              class:dice-chip--dropped={die.dropped}
              title={`${die.type}${die.dropped ? " (dropped)" : ""}`}
              data-testid="dice-value"
              data-dropped={die.dropped ? "true" : undefined}
            >
              <span class="dice-chip__value">{die.value}</span>
              <span class="dice-chip__type">{die.type}</span>
            </span>
          {/each}
          {#if record.modifier}
            <span class="dice-chip dice-chip--mod" data-testid="dice-modifier">
              <span class="dice-chip__value">{record.modifier > 0 ? "+" : "−"}{Math.abs(record.modifier)}</span>
              <span class="dice-chip__type">mod</span>
            </span>
          {/if}
        </span>
        <span class="dice-result__total" data-testid="dice-total">
          <span class="dice-result__equals" aria-hidden="true">=</span>
          <span class="dice-sr">Total: </span><strong>{record.total}</strong>
        </span>
      </div>
    </div>
  {:else}
    <p class="dice-result__message dice-result__message--quiet">Ready to roll</p>
  {/if}
</div>

<style>
  .dice-result {
    display: flex;
    justify-content: center;
    container-type: inline-size;
    width: 100%;
    min-width: 0;
    max-height: 100%;
  }
  .dice-result__message {
    margin: 0;
    padding: 6px 16px;
    border-radius: 999px;
    border: 1px solid var(--dice-line);
    background: var(--dice-glass);
    color: var(--dice-text);
    font-size: 14px;
  }
  .dice-result__message--quiet {
    color: var(--dice-muted);
  }
  .dice-result__message--error {
    border-color: rgba(229, 139, 127, 0.5);
    color: #f1b3aa;
    font-size: 13px;
    border-radius: 12px;
  }
  .dice-result__panel {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-width: 100%;
    padding: 8px 12px 10px;
    border-radius: 14px;
    border: 1px solid var(--dice-line);
    background: var(--dice-glass);
    backdrop-filter: blur(10px);
    color: var(--dice-text);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  }
  .dice-result__panel.crit-success {
    border-color: color-mix(in srgb, var(--dice-accent) 70%, transparent);
    box-shadow:
      0 0 0 1px color-mix(in srgb, var(--dice-accent) 30%, transparent),
      0 8px 28px rgba(110, 169, 207, 0.25);
  }
  .dice-result__panel.crit-failure {
    border-color: rgba(229, 139, 127, 0.55);
  }
  .dice-result__meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--dice-muted);
  }
  .dice-result__expr {
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.02em;
  }
  .dice-result__badge {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 1px 7px;
    border-radius: 999px;
    background: rgba(110, 169, 207, 0.16);
    color: var(--dice-accent);
    font-weight: 600;
  }
  .dice-result__badge--private {
    background: rgba(201, 206, 214, 0.12);
    color: var(--dice-silver);
  }
  .dice-result__label {
    color: var(--dice-text);
  }
  .dice-result__line {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  .dice-result__dice {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    min-width: 0;
    max-height: 40px;
    overflow-y: auto;
  }
  .dice-chip {
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    min-width: 34px;
    padding: 2px 6px 3px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.07);
    font-variant-numeric: tabular-nums;
    line-height: 1.1;
  }
  .dice-chip__value {
    font-size: 17px;
    font-weight: 650;
  }
  .dice-chip__type {
    font-size: 10px;
    color: var(--dice-muted);
  }
  .dice-chip.nat-high {
    background: var(--dice-accent);
    color: var(--dice-on-accent);
  }
  .dice-chip.nat-high .dice-chip__type {
    color: #14324a;
  }
  .dice-chip.nat-low {
    background: rgba(229, 139, 127, 0.2);
    color: #f6c1b9;
  }
  .dice-chip--dropped {
    opacity: 1;
    background: transparent;
    border: 1px dashed var(--dice-line);
    color: var(--dice-muted);
  }
  .dice-chip--dropped .dice-chip__value {
    text-decoration: line-through;
  }
  .dice-chip--mod {
    background: transparent;
    border: 1px solid var(--dice-line);
  }
  .dice-result__total {
    display: inline-flex;
    align-items: baseline;
    gap: 8px;
    margin-left: auto;
    padding-left: 4px;
    white-space: nowrap;
  }
  .dice-result__equals {
    color: var(--dice-muted);
    font-size: 20px;
  }
  .dice-result__total strong {
    min-width: 2ch;
    padding: 2px 12px;
    border-radius: 10px;
    background: var(--dice-accent);
    color: var(--dice-on-accent);
    font-size: 28px;
    font-weight: 750;
    line-height: 1.15;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }
  .dice-result__crit {
    margin: 0;
    padding: 1px 7px;
    border-radius: 999px;
    background: rgba(110, 169, 207, 0.22);
    font-size: 11px;
    font-weight: 600;
    color: var(--dice-accent);
  }
  .crit-failure .dice-result__crit {
    color: #f1b3aa;
    background: rgba(229, 139, 127, 0.18);
  }
  .dice-sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
  }
  .dice-result--compact .dice-result__panel {
    width: 100%;
  }
  .dice-result--compact .dice-result__dice {
    max-height: 80px;
  }
  .dice-result--compact .dice-result__total strong {
    font-size: 22px;
  }
  @container (max-width: 380px) {
    .dice-chip__value {
      font-size: 14px;
    }
    .dice-result__total strong {
      font-size: 20px;
      padding: 2px 8px;
    }
  }
</style>
