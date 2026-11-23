<script lang="ts">
  import { Lock } from "lucide-svelte";
  import { DIE_TYPES, type DieType, type RollRecord } from "../types/dice";

  let {
    records,
    onClear,
    sessionTitle = null,
  }: { records: readonly RollRecord[]; onClear: () => void; sessionTitle?: string | null } =
    $props();

  let filter = $state<DieType | "all">("all");
  const shown = $derived(
    filter === "all" ? records : records.filter((record) => record.dice.some((die) => die.type === filter))
  );
  const KIND = { sum: "", advantage: "ADV", disadvantage: "DIS" } as const;

  const time = (iso: string) =>
    new Date(iso).toLocaleTimeString([], { hour: "numeric", minute: "2-digit", second: "2-digit" });
</script>

<section class="dice-popover" aria-label="Roll history" data-testid="dice-history">
  <header>
    <h3>Roll history</h3>
    <select bind:value={filter} aria-label="Filter by die" data-testid="dice-history-filter">
      <option value="all">All dice</option>
      {#each DIE_TYPES as type (type)}
        <option value={type}>{type}</option>
      {/each}
    </select>
    <button type="button" class="dice-text-button" onclick={onClear} disabled={!records.length}>
      Clear
    </button>
  </header>
  {#if sessionTitle}
    <p class="session" data-testid="dice-history-session">Saving to session: {sessionTitle}</p>
  {/if}
  {#if shown.length}
    <ol>
      {#each shown as record (record.id)}
        <li data-testid="dice-history-item">
          <span class="expr">
            {record.label ?? record.expression}
            {#if KIND[record.kind]}<em>{KIND[record.kind]}</em>{/if}
            {#if record.visibility !== "public"}<Lock size={10} aria-label="GM only" />{/if}
          </span>
          <span class="dice"
            >{#each record.dice as die, index (index)}<span class:dropped={die.dropped}
                >{die.value}</span
              >{index < record.dice.length - 1 ? " · " : ""}{/each}{#if record.modifier}
              {record.modifier > 0 ? "+" : "−"}{Math.abs(record.modifier)}{/if}</span
          >
          <strong class:crit={record.critical === "success"} class:fumble={record.critical === "failure"}
            >{record.total}</strong
          >
          <time datetime={record.timestamp}>{time(record.timestamp)}</time>
        </li>
      {/each}
    </ol>
  {:else}
    <p class="empty">{records.length ? "No rolls with that die." : "No rolls yet this session."}</p>
  {/if}
</section>

<style>
  header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }
  h3 {
    font-family: inherit;
    flex: 1;
    margin: 0;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--dice-muted);
  }
  select {
    padding: 2px 4px;
    border-radius: 6px;
    border: 1px solid var(--dice-line);
    background: #232327;
    color: var(--dice-text);
    font-size: 12px;
  }
  .session {
    margin: 0 0 6px;
    font-size: 11px;
    color: var(--dice-gold);
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
    grid-template-columns: minmax(70px, 1.2fr) 1fr auto auto;
    align-items: baseline;
    gap: 8px;
    padding: 6px 2px;
    border-top: 1px solid var(--dice-line);
    font-variant-numeric: tabular-nums;
  }
  .expr,
  time {
    color: var(--dice-muted);
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  em {
    font-style: normal;
    color: var(--dice-gold);
    font-size: 10px;
    margin-left: 3px;
  }
  .dice {
    font-size: 12px;
  }
  .dropped {
    text-decoration: line-through;
    color: var(--dice-muted);
  }
  strong {
    color: var(--dice-gold);
  }
  strong.crit {
    text-shadow: 0 0 8px rgba(216, 171, 95, 0.7);
  }
  strong.fumble {
    color: #f1b3aa;
  }
  .empty {
    margin: 0;
    color: var(--dice-muted);
    font-size: 13px;
  }
</style>
