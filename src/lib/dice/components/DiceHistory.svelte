<script lang="ts">
  import type { RollRecord } from "../types/dice";

  let {
    records,
    onClear,
  }: { records: readonly RollRecord[]; onClear: () => void } = $props();

  const time = (iso: string) =>
    new Date(iso).toLocaleTimeString([], { hour: "numeric", minute: "2-digit", second: "2-digit" });
</script>

<section class="dice-popover" aria-label="Roll history" data-testid="dice-history">
  <header>
    <h3>Roll history</h3>
    <button type="button" class="dice-text-button" onclick={onClear} disabled={!records.length}>
      Clear history
    </button>
  </header>
  {#if records.length}
    <ol>
      {#each records as record (record.id)}
        <li data-testid="dice-history-item">
          <span class="expr">{record.expression}</span>
          <span class="dice">{record.dice.map((die) => die.value).join(" + ")}</span>
          <strong>{record.total}</strong>
          <time datetime={record.timestamp}>{time(record.timestamp)}</time>
        </li>
      {/each}
    </ol>
  {:else}
    <p class="empty">No rolls yet this session.</p>
  {/if}
</section>

<style>
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 8px;
  }
  h3 {
    font-family: inherit;
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--dice-muted);
  }
  ol {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 260px;
    overflow-y: auto;
  }
  li {
    display: grid;
    grid-template-columns: 56px 1fr auto auto;
    align-items: baseline;
    gap: 10px;
    padding: 6px 2px;
    border-top: 1px solid var(--dice-line);
    font-variant-numeric: tabular-nums;
  }
  .expr,
  time {
    color: var(--dice-muted);
    font-size: 12px;
  }
  strong {
    color: var(--dice-gold);
  }
  .empty {
    margin: 0;
    color: var(--dice-muted);
    font-size: 13px;
  }
</style>
