<script lang="ts">
  /**
   * The global floating dice tray. Nonblocking: no scrim, no focus trap, and
   * it only receives pointer events inside its own bounds, so the workspace
   * underneath stays usable while it is open.
   */
  import {
    Dices,
    Eraser,
    GripHorizontal,
    History,
    Maximize2,
    Minimize2,
    Minus,
    Plus,
    RotateCcw,
    Settings,
    X,
  } from "lucide-svelte";
  import { onMount } from "svelte";
  import { DIE_TYPES, type DieType } from "../types/dice";
  import { describeRoll } from "../engine/diceResults";
  import type { DiceRenderer } from "../engine/diceRenderer";
  import { diceTray } from "../stores/diceTray.svelte";
  import DiceHistory from "./DiceHistory.svelte";
  import DiceResult from "./DiceResult.svelte";
  import DiceSettings from "./DiceSettings.svelte";
  import DiceStage from "./DiceStage.svelte";
  import DieGlyph from "./DieGlyph.svelte";

  let {
    createRenderer,
  }: {
    /** Test seam: replaces the Dice Box renderer. */
    createRenderer?: (options: { lowPerformance: boolean }) => DiceRenderer;
  } = $props();

  let popover = $state<"history" | "settings" | null>(null);
  let tray: HTMLElement;

  const expanded = $derived(diceTray.mode === "expanded");
  const geometry = $derived(diceTray.activeGeometry);
  const rolling = $derived(diceTray.status === "rolling");
  const canRollSelection = $derived(diceTray.selection !== "" && !rolling);
  const showStage = $derived(expanded && diceTray.settings.animate);
  const rollableTypes = $derived(DIE_TYPES.filter((type) => diceTray.canRoll(type)));

  function dieLabel(type: DieType) {
    return type === "d100" ? "Percentile die (d100)" : type;
  }

  function togglePopover(name: "history" | "settings") {
    popover = popover === name ? null : name;
  }

  function onTrayKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape") return;
    event.stopPropagation();
    if (popover) popover = null;
    else diceTray.close();
  }

  // --- drag and resize -------------------------------------------------------

  function track(
    event: PointerEvent,
    apply: (dx: number, dy: number, start: typeof geometry) => void
  ) {
    if (event.button !== 0) return;
    if ((event.target as Element).closest("button")) return;
    event.preventDefault();
    const target = event.currentTarget as HTMLElement;
    const start = { ...diceTray.activeGeometry };
    const originX = event.clientX;
    const originY = event.clientY;
    target.setPointerCapture?.(event.pointerId);
    const move = (next: PointerEvent) => apply(next.clientX - originX, next.clientY - originY, start);
    const end = () => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", end);
      target.removeEventListener("pointercancel", end);
      diceTray.persistTray();
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", end);
    target.addEventListener("pointercancel", end);
  }

  const startDrag = (event: PointerEvent) =>
    track(event, (dx, dy, start) => diceTray.moveTo(start.x + dx, start.y + dy));

  const startResize = (event: PointerEvent) =>
    track(event, (dx, dy, start) => diceTray.resizeTo(start.width + dx, start.height + dy));

  function onGripKeydown(event: KeyboardEvent) {
    const step = event.shiftKey ? 48 : 16;
    const moves: Record<string, [number, number]> = {
      ArrowLeft: [-step, 0],
      ArrowRight: [step, 0],
      ArrowUp: [0, -step],
      ArrowDown: [0, step],
    };
    const delta = moves[event.key];
    if (!delta) return;
    event.preventDefault();
    diceTray.moveTo(geometry.x + delta[0], geometry.y + delta[1]);
    diceTray.persistTray();
  }

  onMount(() => {
    diceTray.refit();
    const onResize = () => diceTray.refit();
    window.addEventListener("resize", onResize);
    tray.addEventListener("keydown", onTrayKeydown);
    tray.querySelector<HTMLElement>("[data-dice-autofocus]")?.focus({ preventScroll: true });
    return () => window.removeEventListener("resize", onResize);
  });
</script>

<section
  bind:this={tray}
  class="dice-tray"
  class:dice-tray--expanded={expanded}
  class:dice-tray--compact={!expanded}
  aria-label="Dice tray"
  data-testid="dice-tray"
  data-mode={diceTray.mode}
  style:left="{geometry.x}px"
  style:top="{geometry.y}px"
  style:width="{geometry.width}px"
  style:height={expanded ? `${geometry.height}px` : undefined}
>
  <div
    class="dice-tray__grip"
    role="toolbar"
    tabindex="0"
    aria-label="Move dice tray (drag, or use arrow keys)"
    data-testid="dice-grip"
    onpointerdown={startDrag}
    onkeydown={onGripKeydown}
  >
    <GripHorizontal size={16} aria-hidden="true" />
    <span class="dice-tray__title">Dice</span>
    {#if !expanded}
      <span class="dice-tray__spacer"></span>
      <button
        type="button"
        class="dice-icon-button dice-icon-button--small"
        aria-label="Expand dice tray"
        title="Expand"
        data-testid="dice-expand"
        onclick={() => diceTray.setMode("expanded")}
      >
        <Maximize2 size={15} aria-hidden="true" />
      </button>
      <button
        type="button"
        class="dice-icon-button dice-icon-button--small"
        aria-label="Close dice tray"
        title="Close"
        data-testid="dice-close"
        onclick={() => diceTray.close()}
      >
        <X size={15} aria-hidden="true" />
      </button>
    {/if}
  </div>

  {#if expanded}
    <div class="dice-tray__body">
      <div class="dice-rail" role="group" aria-label="Choose dice">
        {#each DIE_TYPES as type (type)}
          {@const enabled = diceTray.canRoll(type)}
          <button
            type="button"
            class="dice-die-button"
            class:dice-die-button--selected={diceTray.selectedType === type && enabled}
            aria-label={dieLabel(type)}
            aria-pressed={diceTray.selectedType === type && enabled}
            title={enabled ? dieLabel(type) : `${dieLabel(type)}: not available yet`}
            disabled={!enabled}
            data-testid={`dice-type-${type}`}
            onclick={() => diceTray.selectType(type)}
          >
            <DieGlyph {type} size={30} />
          </button>
        {/each}
        <span class="dice-rail__spacer"></span>
        <button
          type="button"
          class="dice-icon-button"
          aria-label={`Add a ${diceTray.selectedType}`}
          title="Add a die"
          data-testid="dice-add"
          onclick={() => diceTray.adjustCount(1)}
        >
          <Plus size={22} aria-hidden="true" />
        </button>
        <button
          type="button"
          class="dice-icon-button"
          aria-label={`Remove a ${diceTray.selectedType}`}
          title="Remove a die"
          data-testid="dice-remove"
          onclick={() => diceTray.adjustCount(-1)}
        >
          <Minus size={22} aria-hidden="true" />
        </button>
      </div>

      <div class="dice-tray__felt">
        {#if showStage}
          {#key diceTray.settings.lowPerformance}
            <DiceStage lowPerformance={diceTray.settings.lowPerformance} {createRenderer} />
          {/key}
        {/if}

        <div class="dice-tray__top">
          <DiceResult record={diceTray.current} status={diceTray.status} error={diceTray.error} />
        </div>

        {#if popover === "history"}
          <DiceHistory records={diceTray.history.all} onClear={() => diceTray.history.clear()} />
        {:else if popover === "settings"}
          <DiceSettings
            settings={diceTray.settings}
            onChange={(patch) => diceTray.updateSettings(patch)}
          />
        {/if}

        <div class="dice-strip" role="group" aria-label="Dice in this roll">
          {#each DIE_TYPES as type (type)}
            {@const enabled = diceTray.canRoll(type)}
            <button
              type="button"
              class="dice-strip__cell"
              class:dice-strip__cell--active={(diceTray.counts[type] ?? 0) > 0}
              disabled={!enabled}
              aria-label={`${diceTray.counts[type] ?? 0} × ${dieLabel(type)}`}
              data-testid={`dice-count-${type}`}
              onclick={() => diceTray.selectType(type)}
            >
              <span class="dice-strip__count">{diceTray.counts[type] ?? 0}</span>
              <DieGlyph {type} size={20} />
            </button>
          {/each}
        </div>
      </div>

      <div class="dice-rail dice-rail--right" role="toolbar" aria-label="Dice tray controls">
        <button
          type="button"
          class="dice-icon-button"
          aria-label="Dice settings"
          aria-expanded={popover === "settings"}
          title="Settings"
          data-testid="dice-settings-toggle"
          onclick={() => togglePopover("settings")}
        >
          <Settings size={22} aria-hidden="true" />
        </button>
        <button
          type="button"
          class="dice-icon-button"
          aria-label="Roll history"
          aria-expanded={popover === "history"}
          title="History"
          data-testid="dice-history-toggle"
          onclick={() => togglePopover("history")}
        >
          <History size={22} aria-hidden="true" />
        </button>
        <span class="dice-rail__spacer"></span>
        <button
          type="button"
          class="dice-roll-button"
          disabled={!canRollSelection}
          title={`Roll ${diceTray.selection || "dice"}`}
          data-testid="dice-roll"
          data-dice-autofocus
          onclick={() => diceTray.rollSelection()}
        >
          <Dices size={24} aria-hidden="true" />
          <span>Roll {diceTray.selection}</span>
        </button>
        <button
          type="button"
          class="dice-icon-button"
          aria-label="Reroll"
          title="Reroll"
          disabled={rolling || !diceTray.history.latest}
          data-testid="dice-reroll"
          onclick={() => diceTray.reroll()}
        >
          <RotateCcw size={22} aria-hidden="true" />
        </button>
        <button
          type="button"
          class="dice-icon-button"
          aria-label="Clear tray"
          title="Clear tray"
          data-testid="dice-clear"
          onclick={() => diceTray.clearTray()}
        >
          <Eraser size={22} aria-hidden="true" />
        </button>
        <span class="dice-rail__spacer"></span>
        <button
          type="button"
          class="dice-icon-button"
          aria-label="Compact dice tray"
          title="Compact mode"
          data-testid="dice-compact"
          onclick={() => diceTray.setMode("compact")}
        >
          <Minimize2 size={22} aria-hidden="true" />
        </button>
        <button
          type="button"
          class="dice-icon-button"
          aria-label="Close dice tray"
          title="Close"
          data-testid="dice-close"
          onclick={() => diceTray.close()}
        >
          <X size={24} aria-hidden="true" />
        </button>
      </div>
    </div>
    <div
      class="dice-tray__resize"
      aria-hidden="true"
      data-testid="dice-resize"
      onpointerdown={startResize}
    ></div>
  {:else}
    <div class="dice-compact">
      <div class="dice-compact__row">
        {#each rollableTypes as type (type)}
          <button
            type="button"
            class="dice-die-button dice-die-button--small"
            class:dice-die-button--selected={diceTray.selectedType === type}
            aria-label={dieLabel(type)}
            aria-pressed={diceTray.selectedType === type}
            onclick={() => diceTray.selectType(type)}
          >
            <DieGlyph {type} size={24} />
          </button>
        {/each}
        <div class="dice-stepper" role="group" aria-label="Number of dice">
          <button
            type="button"
            class="dice-icon-button dice-icon-button--small"
            aria-label={`Remove a ${diceTray.selectedType}`}
            data-testid="dice-remove"
            onclick={() => diceTray.adjustCount(-1)}
          >
            <Minus size={16} aria-hidden="true" />
          </button>
          <span class="dice-stepper__value" data-testid="dice-selected-count">
            {diceTray.counts[diceTray.selectedType] ?? 0}
          </span>
          <button
            type="button"
            class="dice-icon-button dice-icon-button--small"
            aria-label={`Add a ${diceTray.selectedType}`}
            data-testid="dice-add"
            onclick={() => diceTray.adjustCount(1)}
          >
            <Plus size={16} aria-hidden="true" />
          </button>
        </div>
        <button
          type="button"
          class="dice-roll-button dice-roll-button--compact"
          disabled={!canRollSelection}
          data-testid="dice-roll"
          data-dice-autofocus
          onclick={() => diceTray.rollSelection()}
        >
          <Dices size={18} aria-hidden="true" />
          <span>Roll {diceTray.selection}</span>
        </button>
      </div>
      <DiceResult
        record={diceTray.current}
        status={diceTray.status}
        error={diceTray.error}
        compact
      />
      <div class="dice-compact__row dice-compact__row--end">
        <button
          type="button"
          class="dice-text-button"
          disabled={!diceTray.history.latest}
          data-testid="dice-reroll"
          onclick={() => diceTray.reroll()}
        >
          <RotateCcw size={14} aria-hidden="true" /> Reroll
        </button>
        <button
          type="button"
          class="dice-text-button"
          data-testid="dice-clear"
          onclick={() => diceTray.clearTray()}
        >
          <Eraser size={14} aria-hidden="true" /> Clear
        </button>
        <button
          type="button"
          class="dice-text-button"
          aria-expanded={popover === "history"}
          data-testid="dice-history-toggle"
          onclick={() => togglePopover("history")}
        >
          <History size={14} aria-hidden="true" /> History ({diceTray.history.all.length})
        </button>
      </div>
      {#if popover === "history"}
        <DiceHistory records={diceTray.history.all} onClear={() => diceTray.history.clear()} />
      {/if}
    </div>
  {/if}

  <p class="dice-sr-only" aria-live="polite">
    {diceTray.current ? describeRoll(diceTray.current) : ""}
  </p>
</section>

<style>
  .dice-tray {
    /* Charcoal, metallic silver and amber-gold, as in the approved Catalyst identity. */
    --dice-charcoal: #18181b;
    --dice-panel: #1f1f23;
    --dice-glass: rgba(24, 24, 27, 0.72);
    --dice-line: rgba(214, 218, 224, 0.16);
    --dice-text: #e7e9ec;
    --dice-muted: #a3a8b0;
    --dice-silver: #c9ced6;
    --dice-gold: #d8ab5f;
    --dice-gold-strong: #c4914a;
    --dice-glyph-fill: #e7e9ec;
    --dice-glyph-text: #18181b;
    /* Catalyst draws disabled buttons from this token pair (app.css), never opacity. */
    --color-disabled-text: #6b7078;
    --color-disabled-bg: rgba(255, 255, 255, 0.04);
    --color-disabled-border: transparent;

    position: fixed;
    z-index: var(--z-overlay, 40);
    display: flex;
    flex-direction: column;
    border-radius: 18px;
    border: 1px solid var(--dice-line);
    background: var(--dice-panel);
    color: var(--dice-text);
    box-shadow:
      0 24px 60px rgba(0, 0, 0, 0.45),
      0 2px 8px rgba(0, 0, 0, 0.35);
    overflow: hidden;
    font-family: var(--font-ui, system-ui, sans-serif);
    user-select: none;
  }

  .dice-tray__grip {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    flex: 0 0 32px;
    padding: 0 8px 0 12px;
    color: var(--dice-muted);
    cursor: grab;
    touch-action: none;
    border-bottom: 1px solid var(--dice-line);
  }
  .dice-tray__grip:active {
    cursor: grabbing;
  }
  .dice-tray__grip:focus-visible {
    outline: 2px solid var(--dice-gold);
    outline-offset: -2px;
  }
  .dice-tray__title {
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .dice-tray__spacer {
    flex: 1;
  }

  .dice-tray__body {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 64px 1fr 72px;
  }

  .dice-rail {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 10px 0;
    overflow-y: auto;
    background: rgba(0, 0, 0, 0.18);
    border-right: 1px solid var(--dice-line);
  }
  .dice-rail--right {
    border-right: 0;
    border-left: 1px solid var(--dice-line);
  }
  .dice-rail__spacer {
    flex: 1;
    min-height: 6px;
  }

  .dice-tray__felt {
    position: relative;
    min-width: 0;
    overflow: hidden;
    background:
      radial-gradient(ellipse at 50% 40%, rgba(216, 171, 95, 0.06), transparent 60%),
      linear-gradient(rgba(255, 255, 255, 0.025) 1px, transparent 1px) 0 0 / 40px 40px,
      linear-gradient(90deg, rgba(255, 255, 255, 0.025) 1px, transparent 1px) 0 0 / 40px 40px,
      radial-gradient(ellipse at center, #1c1c20, #0f0f11 85%);
    box-shadow: inset 0 0 60px rgba(0, 0, 0, 0.6);
  }
  .dice-tray__top {
    position: absolute;
    top: 12px;
    left: 12px;
    right: 12px;
    z-index: 2;
    pointer-events: none;
  }

  .dice-strip {
    position: absolute;
    left: 50%;
    bottom: 12px;
    z-index: 2;
    transform: translateX(-50%);
    display: flex;
    max-width: calc(100% - 24px);
    overflow-x: auto;
    border-radius: 12px;
    border: 1px solid var(--dice-line);
    background: var(--dice-glass);
    backdrop-filter: blur(10px);
  }
  .dice-strip__cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    min-width: 44px;
    padding: 4px 6px 6px;
    border: 0;
    border-right: 1px solid var(--dice-line);
    background: transparent;
    color: var(--dice-text);
    cursor: pointer;
  }
  .dice-strip__cell:last-child {
    border-right: 0;
  }
  .dice-strip__cell--active {
    background: linear-gradient(rgba(216, 171, 95, 0.28), rgba(216, 171, 95, 0.1));
  }
  .dice-strip__cell:disabled {
    cursor: default;
    --dice-glyph-fill: #4a4d53;
  }
  .dice-strip__count {
    font-size: 14px;
    font-variant-numeric: tabular-nums;
  }

  .dice-tray :global(.dice-popover) {
    position: absolute;
    top: 58px;
    right: 12px;
    z-index: 3;
    width: min(320px, calc(100% - 24px));
    padding: 12px 14px;
    border-radius: 12px;
    border: 1px solid var(--dice-line);
    background: rgba(24, 24, 27, 0.94);
    backdrop-filter: blur(12px);
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.4);
    font-size: 13px;
    user-select: text;
  }
  .dice-tray--compact :global(.dice-popover) {
    position: static;
    width: auto;
    margin-top: 4px;
  }

  .dice-icon-button,
  .dice-die-button {
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    flex: 0 0 auto;
    border: 1px solid transparent;
    border-radius: 12px;
    background: transparent;
    color: var(--dice-silver);
    cursor: pointer;
    transition:
      background 120ms ease,
      color 120ms ease,
      border-color 120ms ease;
  }
  .dice-icon-button--small {
    width: 28px;
    height: 28px;
    border-radius: 8px;
  }
  .dice-die-button--small {
    width: 36px;
    height: 36px;
  }
  .dice-icon-button:hover:not(:disabled),
  .dice-die-button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.07);
    color: #fff;
  }
  .dice-icon-button[aria-expanded="true"] {
    color: var(--dice-gold);
    background: rgba(216, 171, 95, 0.12);
  }
  .dice-die-button--selected {
    border-color: var(--dice-gold);
    background: rgba(216, 171, 95, 0.12);
    --dice-glyph-fill: var(--dice-gold);
  }
  .dice-die-button:disabled {
    cursor: default;
    --dice-glyph-fill: #4a4d53;
  }
  .dice-icon-button:disabled {
    cursor: default;
  }
  .dice-icon-button:focus-visible,
  .dice-die-button:focus-visible,
  .dice-roll-button:focus-visible,
  .dice-strip__cell:focus-visible,
  .dice-tray :global(.dice-text-button:focus-visible) {
    outline: 2px solid var(--dice-gold);
    outline-offset: 2px;
  }

  .dice-roll-button {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    width: 58px;
    min-height: 64px;
    padding: 6px 2px;
    border: 0;
    border-radius: 14px;
    background: linear-gradient(180deg, var(--dice-gold), var(--dice-gold-strong));
    color: #1b1408;
    font-size: 11px;
    font-weight: 700;
    line-height: 1.1;
    text-align: center;
    cursor: pointer;
    box-shadow: 0 6px 16px rgba(216, 171, 95, 0.22);
  }
  .dice-roll-button:hover:not(:disabled) {
    filter: brightness(1.08);
  }
  .dice-roll-button:disabled {
    cursor: default;
    box-shadow: none;
  }
  .dice-roll-button--compact {
    flex-direction: row;
    width: auto;
    min-height: 36px;
    padding: 0 14px;
    margin-left: auto;
    font-size: 13px;
  }

  .dice-compact {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 10px 12px 12px;
  }
  .dice-compact__row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .dice-compact__row--end {
    justify-content: flex-end;
  }
  .dice-stepper {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 2px;
    border-radius: 10px;
    border: 1px solid var(--dice-line);
  }
  .dice-stepper__value {
    min-width: 22px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  .dice-tray :global(.dice-text-button) {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 8px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--dice-silver);
    font-size: 12px;
    cursor: pointer;
  }
  .dice-tray :global(.dice-text-button:hover:not(:disabled)) {
    background: rgba(255, 255, 255, 0.07);
    color: #fff;
  }
  .dice-tray :global(.dice-text-button:disabled) {
    cursor: default;
  }

  .dice-tray__resize {
    position: absolute;
    right: 0;
    bottom: 0;
    width: 18px;
    height: 18px;
    cursor: nwse-resize;
    touch-action: none;
    z-index: 4;
    background: linear-gradient(
      135deg,
      transparent 50%,
      var(--dice-line) 50%,
      var(--dice-line) 60%,
      transparent 60%,
      transparent 70%,
      var(--dice-line) 70%,
      var(--dice-line) 80%,
      transparent 80%
    );
  }

  .dice-sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
    white-space: nowrap;
  }

  @media (prefers-reduced-motion: reduce) {
    .dice-icon-button,
    .dice-die-button {
      transition: none;
    }
  }
</style>
