<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy } from "svelte";
  import { Dices, Eye, EyeOff, Lock, PanelsTopLeft, Square, X } from "lucide-svelte";
  import type { Project } from "../types";
  import { BUILTIN_SOUNDS, CHANNELS, clampVolume, padTags, padsWithTag, type SoundChannel, type SoundPad } from "../soundboard.svelte";
  import { soundboard } from "../soundboardBoard.svelte";
  import { diceTray, type RollMode, type RollRecord, type RollVisibility } from "../dice";
  import { GM_PANELS, defaultGmScreen, gmPanel, updateGmPanel, type GmPanelId, type GmScreenLayout } from "../gmScreen";

  let { project }: { project: Project } = $props();

  const root = $derived(project.project_root ?? "");
  const SAVE_DELAY_MS = 400;
  const DICE = ["d4", "d6", "d8", "d10", "d12", "d20", "d100"] as const;

  let screenLayout = $state<GmScreenLayout>(defaultGmScreen());
  let loadedRoot = "";
  let screenError = $state<string | null>(null);
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let allPadsOpen = $state(false);
  let rollMode = $state<RollMode>("normal");
  let visibility = $state<RollVisibility>("public");

  const engine = soundboard.engine;
  const board = $derived(soundboard.layout);
  const fileSet = $derived(new Set([...BUILTIN_SOUNDS.map((s) => s.file), ...soundboard.files.map((f) => f.path)]));
  const tags = $derived(padTags(board));
  const padTag = $derived(gmPanel(screenLayout, "soundboard").pad_tag ?? null);
  const shownPads = $derived(padsWithTag(board, padTag));
  const anyPlaying = $derived(Object.keys(engine.playing).length > 0);
  const rolls = $derived(diceTray.history.all);
  const visible = (id: GmPanelId) => gmPanel(screenLayout, id).visible;

  $effect(() => {
    const next = root;
    if (!next) return;
    void soundboard.open(next);
    if (next !== loadedRoot) void loadScreen(next);
  });

  onDestroy(() => {
    if (saveTimer) {
      clearTimeout(saveTimer);
      void persist();
    }
    void soundboard.flush();
  });

  async function loadScreen(projectRoot: string) {
    loadedRoot = projectRoot;
    screenError = null;
    try {
      const saved = await invoke<GmScreenLayout | null>("load_gm_screen", { projectRoot });
      if (projectRoot === loadedRoot) screenLayout = saved ?? defaultGmScreen();
    } catch (e) {
      // Keep the defaults on screen but never overwrite a file we couldn't read.
      loadedRoot = "";
      screenError = String(e);
    }
  }

  function commitScreen(next: GmScreenLayout) {
    screenLayout = next;
    if (!loadedRoot) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => void persist(), SAVE_DELAY_MS);
  }

  async function persist() {
    saveTimer = null;
    if (!loadedRoot) return;
    try {
      await invoke("save_gm_screen", { projectRoot: loadedRoot, layout: $state.snapshot(screenLayout) });
    } catch (e) {
      screenError = `Could not save the GM Screen layout: ${String(e)}`;
    }
  }

  function setMaster(value: number) {
    soundboard.commit({ ...board, master_volume: clampVolume(value) });
  }

  function setChannel(channel: SoundChannel, value: number) {
    soundboard.commit({ ...board, channels: { ...board.channels, [channel]: { volume: clampVolume(value) } } });
  }

  function rollDie(die: (typeof DICE)[number]) {
    void diceTray.roll({
      expression: `1${die}`,
      mode: die === "d20" ? rollMode : "normal",
      visibility,
      source: "gm-console",
    });
  }

  function toggleMode(mode: RollMode) {
    rollMode = rollMode === mode ? "normal" : mode;
  }

  /** Hidden scrollbars: mark the edges that have more content so the panel can fade them. */
  function scrollCue(node: HTMLElement) {
    const update = () => {
      node.classList.toggle("more-above", node.scrollTop > 1);
      node.classList.toggle("more-below", node.scrollTop + node.clientHeight < node.scrollHeight - 1);
    };
    const observer = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(update);
    observer?.observe(node);
    const mutations = new MutationObserver(update);
    mutations.observe(node, { childList: true, subtree: true });
    node.addEventListener("scroll", update, { passive: true });
    update();
    return {
      destroy() {
        observer?.disconnect();
        mutations.disconnect();
        node.removeEventListener("scroll", update);
      },
    };
  }

  const percent = (value: number) => Math.round(clampVolume(value) * 100);
  const KIND = { sum: "", advantage: "ADV", disadvantage: "DIS" } as const;
</script>

{#snippet pad(item: SoundPad)}
  {@const missing = !fileSet.has(item.file)}
  {@const playing = engine.playing[item.id] === true}
  <button
    type="button"
    class="gm-pad"
    class:is-playing={playing}
    aria-pressed={playing}
    disabled={missing}
    title={missing ? `${item.file} is missing from Assets/Audio` : item.file}
    onclick={() => engine.toggle(item)}
  >
    <span class="gm-pad__label">{item.label}</span>
    <span class="gm-pad__meta">{missing ? "missing file" : engine.errors[item.id] ? "can't play" : playing ? "playing" : ""}</span>
  </button>
{/snippet}

{#snippet roll(record: RollRecord, big: boolean)}
  <div class="gm-roll" class:gm-roll--big={big} data-testid={big ? "gm-current-roll" : "gm-roll"}>
    <span class="gm-roll__what">
      {record.label ?? record.expression}
      {#if KIND[record.kind]}<em>{KIND[record.kind]}</em>{/if}
      {#if record.visibility !== "public"}<Lock size={11} aria-label="GM only" />{/if}
    </span>
    <span class="gm-roll__dice"
      >{#each record.dice as die, index (index)}<span class:dropped={die.dropped}>{die.value}</span
        >{index < record.dice.length - 1 ? " · " : ""}{/each}{#if record.modifier}
        {record.modifier > 0 ? "+" : "−"}{Math.abs(record.modifier)}{/if}</span
    >
    <strong class:crit={record.critical === "success"} class:fumble={record.critical === "failure"}>{record.total}</strong>
    {#if big && record.critical}
      <span class="gm-roll__crit">{record.critical === "success" ? "Critical success" : "Critical failure"}</span>
    {/if}
  </div>
{/snippet}

<section class="gm" aria-labelledby="gm-title">
  <header class="gm-header">
    <h1 id="gm-title"><PanelsTopLeft size={20} aria-hidden="true" /> GM Screen</h1>
    <div class="gm-toggles" role="group" aria-label="Show panels">
      {#each GM_PANELS as panel (panel.id)}
        <button
          type="button"
          class="gm-chip"
          aria-pressed={visible(panel.id)}
          onclick={() => commitScreen(updateGmPanel(screenLayout, panel.id, { visible: !visible(panel.id) }))}
        >
          {panel.label}
        </button>
      {/each}
    </div>
  </header>

  {#if !root}
    <p class="gm-empty">Open a campaign to use the GM Screen.</p>
  {:else}
    {#if screenError || soundboard.error}
      <p class="gm-error" role="alert">{screenError ?? soundboard.error}</p>
    {/if}
    <div class="gm-grid">
      {#if visible("soundboard")}
        <section class="gm-panel gm-panel--sound" aria-labelledby="gm-sound-title">
          <header class="gm-panel__head">
            <h2 id="gm-sound-title">Soundboard</h2>
            <select
              aria-label="Pad set"
              value={padTag ?? ""}
              onchange={(e) =>
                commitScreen(updateGmPanel(screenLayout, "soundboard", { pad_tag: e.currentTarget.value || null }))}
            >
              <option value="">All pads</option>
              {#each tags as tag (tag)}
                <option value={tag}>{tag}</option>
              {/each}
              {#if padTag && !tags.some((t) => t.toLowerCase() === padTag.toLowerCase())}
                <option value={padTag}>{padTag} (no pads)</option>
              {/if}
            </select>
            <button type="button" class="gm-btn" onclick={() => (allPadsOpen = true)}>All pads</button>
          </header>
          <div class="gm-frozen gm-mixer" role="group" aria-label="Mixer">
            <label class="gm-fader gm-fader--master">
              <span>Master <output>{percent(board.master_volume)}%</output></span>
              <input type="range" min="0" max="1" step="0.01" value={board.master_volume}
                oninput={(e) => setMaster(Number(e.currentTarget.value))} aria-label="Master volume" />
            </label>
            {#each CHANNELS as channel (channel.id)}
              <label class="gm-fader">
                <span>{channel.label} <output>{percent(board.channels[channel.id].volume)}%</output></span>
                <input type="range" min="0" max="1" step="0.01" value={board.channels[channel.id].volume}
                  oninput={(e) => setChannel(channel.id, Number(e.currentTarget.value))} aria-label={`${channel.label} volume`} />
              </label>
            {/each}
            <button type="button" class="gm-btn gm-btn--stop" onclick={() => engine.stopAll()} disabled={!anyPlaying}>
              <Square size={14} aria-hidden="true" /> Stop all
            </button>
          </div>
          <!-- svelte-ignore a11y_no_noninteractive_tabindex (scrollbars are hidden, so the region must take keyboard focus to scroll) -->
          <div class="gm-scroll" use:scrollCue tabindex="0" role="region" aria-label="Sound pads">
            {#if shownPads.length}
              <div class="gm-pads">
                {#each shownPads as item (item.id)}{@render pad(item)}{/each}
              </div>
            {:else}
              <p class="gm-empty">{board.pads.length ? "No pads have this tag. Add tags in the Soundboard." : "No pads yet. Add them in the Soundboard."}</p>
            {/if}
          </div>
        </section>
      {/if}

      {#if visible("dice") || visible("rolls")}
        <div class="gm-column">
          {#if visible("dice")}
            <section class="gm-panel gm-panel--dice" aria-labelledby="gm-dice-title">
              <header class="gm-panel__head">
                <h2 id="gm-dice-title">Dice</h2>
                <button type="button" class="gm-btn" onclick={() => diceTray.open("expanded")}>
                  <Dices size={14} aria-hidden="true" /> Open tray
                </button>
              </header>
              <div class="gm-dice">
                {#each DICE as die (die)}
                  <button type="button" class="gm-die" onclick={() => rollDie(die)} aria-label={`Roll ${die}`}>{die}</button>
                {/each}
              </div>
              <div class="gm-dice-options">
                <button type="button" class="gm-chip" aria-pressed={rollMode === "advantage"} onclick={() => toggleMode("advantage")}
                  title="d20 rolls with advantage">ADV</button>
                <button type="button" class="gm-chip" aria-pressed={rollMode === "disadvantage"} onclick={() => toggleMode("disadvantage")}
                  title="d20 rolls with disadvantage">DIS</button>
                <button
                  type="button"
                  class="gm-chip"
                  aria-pressed={visibility === "gm-private"}
                  onclick={() => (visibility = visibility === "public" ? "gm-private" : "public")}
                >
                  {#if visibility === "public"}<Eye size={13} aria-hidden="true" /> Public{:else}<EyeOff size={13} aria-hidden="true" /> GM only{/if}
                </button>
              </div>
            </section>
          {/if}

          {#if visible("rolls")}
            <section class="gm-panel gm-panel--rolls" aria-labelledby="gm-rolls-title">
              <header class="gm-panel__head"><h2 id="gm-rolls-title">Rolls</h2></header>
              <div class="gm-frozen" aria-live="polite">
                {#if rolls[0]}{@render roll(rolls[0], true)}{:else}<p class="gm-empty">No rolls yet.</p>{/if}
              </div>
              <!-- svelte-ignore a11y_no_noninteractive_tabindex (scrollbars are hidden, so the region must take keyboard focus to scroll) -->
              <div class="gm-scroll" use:scrollCue tabindex="0" role="region" aria-label="Roll history">
                <ol class="gm-rolls">
                  {#each rolls.slice(1) as record (record.id)}<li>{@render roll(record, false)}</li>{/each}
                </ol>
              </div>
            </section>
          {/if}
        </div>
      {/if}
    </div>
  {/if}

  {#if allPadsOpen}
    <div class="gm-backdrop" role="presentation" onclick={() => (allPadsOpen = false)}></div>
    <div
      class="gm-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="gm-all-pads-title"
      tabindex="-1"
      onkeydown={(e) => {
        if (e.key === "Escape") {
          e.stopPropagation();
          allPadsOpen = false;
        }
      }}
    >
      <header class="gm-panel__head">
        <h2 id="gm-all-pads-title">All pads</h2>
        <button type="button" class="gm-icon" onclick={() => (allPadsOpen = false)} aria-label="Close"><X size={16} /></button>
      </header>
      <!-- svelte-ignore a11y_no_noninteractive_tabindex (scrollbars are hidden, so the region must take keyboard focus to scroll) -->
      <div class="gm-scroll" use:scrollCue tabindex="0" role="region" aria-label="All sound pads">
        {#each CHANNELS as channel (channel.id)}
          {@const pads = board.pads.filter((p) => p.channel === channel.id)}
          {#if pads.length}
            <h3>{channel.label}</h3>
            <div class="gm-pads">{#each pads as item (item.id)}{@render pad(item)}{/each}</div>
          {/if}
        {/each}
      </div>
    </div>
  {/if}
</section>

<style>
  .gm {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 16px 20px;
    gap: 12px;
    box-sizing: border-box;
    background: var(--cat-bg);
    color: var(--cat-text);
    overflow: hidden;
  }
  .gm-header,
  .gm-panel__head {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .gm-header h1 {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    margin: 0;
    font-size: 1.2rem;
  }
  .gm-header h1 :global(svg) {
    color: var(--cat-accent);
  }
  .gm-toggles,
  .gm-dice-options {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .gm-grid {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 12px;
  }
  .gm-column {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-height: 0;
  }
  .gm-panel {
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 12px;
    border: 1px solid var(--cat-border);
    border-radius: 12px;
    background: var(--cat-panel);
  }
  .gm-panel--rolls {
    flex: 1;
  }
  .gm-panel__head {
    margin-bottom: 10px;
  }
  .gm-panel__head h2 {
    flex: 1;
    margin: 0;
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--cat-muted);
  }
  .gm-frozen {
    flex: none;
    padding-bottom: 10px;
    margin-bottom: 8px;
    border-bottom: 1px solid var(--cat-border-subtle);
  }
  .gm-scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overscroll-behavior: contain;
    border-radius: 8px;
  }
  .gm-scroll:focus-visible {
    outline: 2px solid var(--cat-focus);
    outline-offset: 2px;
  }
  /* Edge fades stand in for the hidden scrollbar. */
  .gm-scroll:global(.more-below) {
    mask-image: linear-gradient(to bottom, #000 calc(100% - 28px), transparent);
  }
  .gm-scroll:global(.more-above) {
    mask-image: linear-gradient(to top, #000 calc(100% - 28px), transparent);
  }
  .gm-scroll:global(.more-above.more-below) {
    mask-image: linear-gradient(to bottom, transparent, #000 28px, #000 calc(100% - 28px), transparent);
  }
  .gm-mixer {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(110px, 1fr));
    gap: 8px 12px;
    align-items: end;
  }
  .gm-fader {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 0.78rem;
    color: var(--cat-muted);
  }
  .gm-fader--master {
    grid-column: 1 / -1;
    color: var(--cat-text);
  }
  .gm-fader input {
    width: 100%;
    accent-color: var(--cat-accent);
  }
  .gm-pads {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
    gap: 8px;
  }
  .gm-pad {
    min-height: 64px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: flex-start;
    gap: 4px;
    padding: 8px 10px;
    border: 1px solid var(--cat-border);
    border-radius: 10px;
    background: var(--cat-raised);
    color: var(--cat-text);
    text-align: left;
    cursor: pointer;
  }
  .gm-pad:hover:not(:disabled) {
    background: var(--cat-hover);
  }
  .gm-pad.is-playing {
    border-color: var(--cat-accent);
    box-shadow: inset 0 0 0 1px var(--cat-accent);
    background: var(--cat-accent-soft);
  }
  .gm-pad:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
  .gm-pad__label {
    font-weight: 600;
    font-size: 0.85rem;
  }
  .gm-pad__meta {
    font-size: 0.72rem;
    color: var(--cat-muted);
  }
  .gm-btn,
  .gm-chip,
  .gm-die,
  .gm-icon,
  select {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    border: 1px solid var(--cat-border);
    border-radius: 8px;
    background: var(--cat-raised);
    color: var(--cat-text);
    font: inherit;
    font-size: 0.8rem;
    cursor: pointer;
  }
  .gm-btn:hover:not(:disabled),
  .gm-chip:hover,
  .gm-die:hover {
    background: var(--cat-hover);
  }
  .gm-btn:disabled {
    color: var(--cat-disabled);
    cursor: not-allowed;
  }
  .gm-btn--stop {
    justify-content: center;
  }
  .gm-chip[aria-pressed="true"] {
    border-color: var(--cat-accent);
    background: var(--cat-accent-soft);
    color: var(--cat-accent-hover);
  }
  .gm-dice {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(56px, 1fr));
    gap: 6px;
    margin-bottom: 8px;
  }
  .gm-die {
    justify-content: center;
    padding: 10px 0;
    font-weight: 600;
  }
  .gm-rolls {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .gm-roll {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 2px 10px;
    padding: 6px 4px;
    border-bottom: 1px solid var(--cat-border-subtle);
    font-size: 0.82rem;
  }
  .gm-roll--big {
    border-bottom: 0;
    font-size: 0.95rem;
  }
  .gm-roll__what em {
    margin-left: 4px;
    font-style: normal;
    font-size: 0.7rem;
    color: var(--cat-accent);
  }
  .gm-roll__dice {
    grid-column: 1;
    color: var(--cat-muted);
    font-variant-numeric: tabular-nums;
  }
  .gm-roll__dice .dropped {
    text-decoration: line-through;
    opacity: 0.6;
  }
  .gm-roll strong {
    grid-column: 2;
    grid-row: 1 / span 2;
    align-self: center;
    font-size: 1.1rem;
    font-variant-numeric: tabular-nums;
  }
  .gm-roll--big strong {
    font-size: 2rem;
    color: var(--cat-accent-hover);
  }
  .gm-roll strong.crit {
    color: var(--cat-success);
  }
  .gm-roll strong.fumble {
    color: var(--cat-error);
  }
  .gm-roll__crit {
    grid-column: 1;
    font-size: 0.75rem;
    color: var(--cat-muted);
  }
  .gm-empty {
    margin: 0;
    color: var(--cat-muted);
    font-size: 0.85rem;
  }
  .gm-error {
    margin: 0;
    color: var(--cat-error);
    font-size: 0.85rem;
  }
  .gm-backdrop {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 0.5);
    z-index: 40;
  }
  .gm-dialog {
    position: fixed;
    inset: 8vh 10vw;
    z-index: 41;
    display: flex;
    flex-direction: column;
    padding: 16px;
    border: 1px solid var(--cat-border);
    border-radius: 14px;
    background: var(--cat-panel);
  }
  .gm-dialog h3 {
    margin: 12px 0 8px;
    font-size: 0.8rem;
    color: var(--cat-muted);
  }
  .gm-icon {
    padding: 4px;
  }
  @media (prefers-reduced-motion: reduce) {
    .gm-pad {
      transition: none;
    }
  }
</style>
