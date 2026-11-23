<script lang="ts">
  /**
   * The global floating dice tray. Nonblocking: no scrim, no focus trap, and
   * it only receives pointer events inside its own bounds, so the workspace
   * underneath stays usable while it is open.
   */
  import {
    Dices,
    Eraser,
    Eye,
    EyeOff,
    GripHorizontal,
    History,
    Keyboard,
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
  import DiceExpression from "./DiceExpression.svelte";
  import DiceHistory from "./DiceHistory.svelte";
  import DiceResult from "./DiceResult.svelte";
  import DiceSettings from "./DiceSettings.svelte";
  import DiceStage from "./DiceStage.svelte";
  import DieGlyph from "./DieGlyph.svelte";
  import catalystLogo from "../../../assets/catalyst-logo.png";

  let {
    createRenderer,
  }: {
    /** Test seam: replaces the Dice Box renderer. */
    createRenderer?: (options: Record<string, unknown>) => DiceRenderer;
  } = $props();

  type Popover = "history" | "settings" | "expression";
  let popover = $state<Popover | null>(null);
  let tray: HTMLElement;

  const expanded = $derived(diceTray.mode === "expanded");
  const geometry = $derived(diceTray.activeGeometry);
  const rolling = $derived(diceTray.status === "rolling");
  const canRollSelection = $derived(diceTray.selection !== "" && !rolling);
  const showStage = $derived(expanded && diceTray.settings.animate);
  const rendererKey = $derived(JSON.stringify(diceTray.rendererOptions));
  const privateRolls = $derived(diceTray.visibility !== "public");
  const modifierText = $derived(
    diceTray.modifier > 0 ? `+${diceTray.modifier}` : diceTray.modifier < 0 ? `−${-diceTray.modifier}` : "±0"
  );
  const target = $derived(diceTray.modifierSelected ? "modifier" : diceTray.selectedType);

  function dieLabel(type: DieType) {
    return type === "d100" ? "Percentile die (d100)" : type;
  }

  function togglePopover(name: Popover) {
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
    // Compact height follows its content (results, popovers); keep it on screen.
    const measure =
      typeof ResizeObserver === "undefined"
        ? null
        : new ResizeObserver(() => {
            if (diceTray.mode === "compact") diceTray.setCompactHeight(tray.offsetHeight);
          });
    measure?.observe(tray);
    return () => {
      window.removeEventListener("resize", onResize);
      measure?.disconnect();
    };
  });
</script>

{#snippet modeButtons(small: boolean)}
  <button
    type="button"
    class="dice-mode-button"
    class:dice-mode-button--small={small}
    aria-pressed={diceTray.rollMode === "advantage"}
    title="Advantage: roll two d20s and keep the higher"
    data-testid="dice-adv"
    onclick={() => diceTray.setRollMode("advantage")}>ADV</button
  >
  <button
    type="button"
    class="dice-mode-button"
    class:dice-mode-button--small={small}
    aria-pressed={diceTray.rollMode === "disadvantage"}
    title="Disadvantage: roll two d20s and keep the lower"
    data-testid="dice-dis"
    onclick={() => diceTray.setRollMode("disadvantage")}>DIS</button
  >
{/snippet}

{#snippet visibilityButton(size: number)}
  <button
    type="button"
    class="dice-icon-button"
    class:dice-icon-button--small={size < 20}
    aria-pressed={privateRolls}
    aria-label={privateRolls ? "Rolls are GM only. Make public" : "Rolls are public. Make GM only"}
    title={privateRolls ? "GM-only rolls (click for public)" : "Public rolls (click for GM only)"}
    data-testid="dice-visibility"
    onclick={() => (diceTray.visibility = privateRolls ? "public" : "gm-private")}
  >
    {#if privateRolls}<EyeOff size={size} aria-hidden="true" />{:else}<Eye
        size={size}
        aria-hidden="true"
      />{/if}
  </button>
{/snippet}

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
    {#if diceTray.campaign}
      <span class="dice-tray__session" title="Rolls are saved to this campaign session"
        >· {diceTray.campaign.session.title}</span
      >
    {/if}
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
          <button
            type="button"
            class="dice-die-button"
            class:dice-die-button--selected={target === type}
            aria-label={`${dieLabel(type)}, ${diceTray.counts[type] ?? 0} selected`}
            aria-pressed={target === type}
            title={`${dieLabel(type)}: click to add one`}
            data-testid={`dice-type-${type}`}
            onclick={() => diceTray.pickDie(type)}
          >
            <DieGlyph {type} size={28} />
          </button>
        {/each}
        <span class="dice-rail__spacer"></span>
        <button
          type="button"
          class="dice-icon-button"
          aria-label={target === "modifier" ? "Increase modifier" : `Add a ${target}`}
          title={target === "modifier" ? "Increase modifier" : `Add a ${target}`}
          data-testid="dice-add"
          onclick={() => diceTray.adjustCount(1)}
        >
          <Plus size={22} aria-hidden="true" />
        </button>
        <button
          type="button"
          class="dice-icon-button"
          aria-label={target === "modifier" ? "Decrease modifier" : `Remove a ${target}`}
          title={target === "modifier" ? "Decrease modifier" : `Remove a ${target}`}
          data-testid="dice-remove"
          onclick={() => diceTray.adjustCount(-1)}
        >
          <Minus size={22} aria-hidden="true" />
        </button>
      </div>

      <div class="dice-tray__felt" data-surface={diceTray.settings.surface}>
        <div class="dice-tray__top">
          <DiceResult record={diceTray.current} status={diceTray.status} error={diceTray.error} />
        </div>

        <div class="dice-tray__table">
          <img class="dice-tray__mat-logo" src={catalystLogo} alt="" aria-hidden="true" draggable="false" />
          {#if showStage}
            {#key rendererKey}
              <DiceStage options={diceTray.rendererOptions} {createRenderer} />
            {/key}
          {/if}
        </div>

        {#if popover === "history"}
          <DiceHistory
            records={diceTray.history.all}
            sessionTitle={diceTray.campaign?.session.title ?? null}
            onClear={() => diceTray.history.clear()}
          />
        {:else if popover === "settings"}
          <DiceSettings
            settings={diceTray.settings}
            onChange={(patch) => diceTray.updateSettings(patch)}
          />
        {:else if popover === "expression"}
          <DiceExpression onRolled={() => (popover = null)} />
        {/if}

        <div class="dice-strip" role="group" aria-label="Dice in this roll">
          {#each DIE_TYPES as type (type)}
            {@const count = diceTray.counts[type] ?? 0}
            {@const advD20 = type === "d20" && diceTray.rollMode !== "normal"}
            <button
              type="button"
              class="dice-strip__cell"
              class:dice-strip__cell--active={count > 0 || advD20}
              class:dice-strip__cell--target={target === type}
              aria-label={`${advD20 ? `2 (${diceTray.rollMode})` : count} × ${dieLabel(type)}`}
              title="Select, then use + and −"
              data-testid={`dice-count-${type}`}
              onclick={() => diceTray.selectType(type)}
            >
              <span class="dice-strip__count">{advD20 ? "2" : count}</span>
              <DieGlyph {type} size={20} />
            </button>
          {/each}
          <button
            type="button"
            class="dice-strip__cell"
            class:dice-strip__cell--active={diceTray.modifier !== 0}
            class:dice-strip__cell--target={target === "modifier"}
            aria-label={`Modifier ${modifierText}`}
            title="Modifier: select, then use + and −"
            data-testid="dice-count-modifier"
            onclick={() => diceTray.selectModifier()}
          >
            <span class="dice-strip__count">{modifierText}</span>
            <span class="dice-strip__mod">MOD</span>
          </button>
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
          <Settings size={20} aria-hidden="true" />
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
          <History size={20} aria-hidden="true" />
        </button>
        <button
          type="button"
          class="dice-icon-button"
          aria-label="Type a dice expression"
          aria-expanded={popover === "expression"}
          title="Type an expression"
          data-testid="dice-expression-toggle"
          onclick={() => togglePopover("expression")}
        >
          <Keyboard size={20} aria-hidden="true" />
        </button>
        {@render visibilityButton(20)}
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
          <Dices size={22} aria-hidden="true" />
          <span>Roll</span>
        </button>
        <button
          type="button"
          class="dice-icon-button"
          aria-label="Reroll"
          title="Reroll the last roll"
          disabled={rolling || !diceTray.history.latest}
          data-testid="dice-reroll"
          onclick={() => diceTray.reroll()}
        >
          <RotateCcw size={20} aria-hidden="true" />
        </button>
        <button
          type="button"
          class="dice-icon-button"
          aria-label="Clear tray"
          title="Clear tray"
          data-testid="dice-clear"
          onclick={() => diceTray.clearTray()}
        >
          <Eraser size={20} aria-hidden="true" />
        </button>
        {@render modeButtons(false)}
        <span class="dice-rail__spacer"></span>
        <button
          type="button"
          class="dice-icon-button"
          aria-label="Compact dice tray"
          title="Compact mode"
          data-testid="dice-compact"
          onclick={() => diceTray.setMode("compact")}
        >
          <Minimize2 size={20} aria-hidden="true" />
        </button>
        <button
          type="button"
          class="dice-icon-button"
          aria-label="Close dice tray"
          title="Close"
          data-testid="dice-close"
          onclick={() => diceTray.close()}
        >
          <X size={22} aria-hidden="true" />
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
      <div class="dice-compact__row dice-compact__dice">
        {#each DIE_TYPES as type (type)}
          <button
            type="button"
            class="dice-die-button dice-die-button--small"
            class:dice-die-button--selected={target === type}
            class:dice-die-button--counted={(diceTray.counts[type] ?? 0) > 0}
            aria-label={`${dieLabel(type)}, ${diceTray.counts[type] ?? 0} selected`}
            aria-pressed={target === type}
            data-testid={`dice-type-${type}`}
            onclick={() => diceTray.selectType(type)}
          >
            <DieGlyph {type} size={22} />
          </button>
        {/each}
      </div>
      <div class="dice-compact__row">
        <div class="dice-stepper" role="group" aria-label={`Number of ${diceTray.selectedType}`}>
          <button
            type="button"
            class="dice-icon-button dice-icon-button--small"
            aria-label={`Remove a ${diceTray.selectedType}`}
            data-testid="dice-remove"
            onclick={() => diceTray.adjustCount(-1, diceTray.selectedType)}
          >
            <Minus size={16} aria-hidden="true" />
          </button>
          <span class="dice-stepper__value" data-testid="dice-selected-count">
            {diceTray.counts[diceTray.selectedType] ?? 0}{diceTray.selectedType}
          </span>
          <button
            type="button"
            class="dice-icon-button dice-icon-button--small"
            aria-label={`Add a ${diceTray.selectedType}`}
            data-testid="dice-add"
            onclick={() => diceTray.adjustCount(1, diceTray.selectedType)}
          >
            <Plus size={16} aria-hidden="true" />
          </button>
        </div>
        <div class="dice-stepper" role="group" aria-label="Modifier">
          <button
            type="button"
            class="dice-icon-button dice-icon-button--small"
            aria-label="Decrease modifier"
            data-testid="dice-mod-down"
            onclick={() => diceTray.adjustModifier(-1)}
          >
            <Minus size={16} aria-hidden="true" />
          </button>
          <span class="dice-stepper__value" data-testid="dice-modifier-value">{modifierText}</span>
          <button
            type="button"
            class="dice-icon-button dice-icon-button--small"
            aria-label="Increase modifier"
            data-testid="dice-mod-up"
            onclick={() => diceTray.adjustModifier(1)}
          >
            <Plus size={16} aria-hidden="true" />
          </button>
        </div>
        {@render modeButtons(true)}
      </div>
      <div class="dice-compact__row">
        {@render visibilityButton(16)}
        <span class="dice-compact__selection" data-testid="dice-selection">{diceTray.selection || "No dice"}</span>
        <button
          type="button"
          class="dice-roll-button dice-roll-button--compact"
          disabled={!canRollSelection}
          data-testid="dice-roll"
          data-dice-autofocus
          onclick={() => diceTray.rollSelection()}
        >
          <Dices size={18} aria-hidden="true" />
          <span>Roll</span>
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
          aria-expanded={popover === "expression"}
          data-testid="dice-expression-toggle"
          onclick={() => togglePopover("expression")}
        >
          <Keyboard size={14} aria-hidden="true" /> Type
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
        <DiceHistory
          records={diceTray.history.all}
          sessionTitle={diceTray.campaign?.session.title ?? null}
          onClear={() => diceTray.history.clear()}
        />
      {:else if popover === "expression"}
        <DiceExpression onRolled={() => (popover = null)} />
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
  .dice-tray__session {
    font-size: 11px;
    color: var(--dice-gold);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dice-mode-button {
    width: 48px;
    height: 30px;
    flex: 0 0 auto;
    border: 1px solid var(--dice-line);
    border-radius: 9px;
    background: transparent;
    color: var(--dice-silver);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
    cursor: pointer;
  }
  .dice-mode-button--small {
    width: 40px;
    height: 28px;
    font-size: 11px;
  }
  .dice-mode-button:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.07);
  }
  .dice-mode-button[aria-pressed="true"] {
    border-color: var(--dice-gold);
    background: rgba(216, 171, 95, 0.18);
    color: var(--dice-gold);
  }
  .dice-mode-button:focus-visible {
    outline: 2px solid var(--dice-gold);
    outline-offset: 2px;
  }
  .dice-icon-button[aria-pressed="true"] {
    color: var(--dice-gold);
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

  /* The felt is a column: result panel, table, dice strip. The 3D table only
     occupies the middle, so dice can never settle under the panel or strip. */
  .dice-tray__felt {
    position: relative;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 12px 12px;
    overflow: hidden;
    background:
      radial-gradient(ellipse at 50% 40%, rgba(216, 171, 95, 0.06), transparent 60%),
      linear-gradient(rgba(255, 255, 255, 0.025) 1px, transparent 1px) 0 0 / 40px 40px,
      linear-gradient(90deg, rgba(255, 255, 255, 0.025) 1px, transparent 1px) 0 0 / 40px 40px,
      radial-gradient(ellipse at center, #1c1c20, #0f0f11 85%);
    box-shadow: inset 0 0 60px rgba(0, 0, 0, 0.6);
  }
  .dice-tray__felt[data-surface="leather"] {
    background:
      radial-gradient(ellipse at 50% 35%, rgba(120, 72, 40, 0.35), transparent 65%),
      repeating-radial-gradient(circle at 30% 40%, rgba(255, 255, 255, 0.012) 0 2px, transparent 2px 5px),
      radial-gradient(ellipse at center, #2a1c15, #120c09 85%);
  }
  .dice-tray__felt[data-surface="walnut"] {
    background:
      repeating-linear-gradient(
        92deg,
        rgba(255, 255, 255, 0.025) 0 2px,
        transparent 2px 11px,
        rgba(0, 0, 0, 0.08) 11px 13px,
        transparent 13px 23px
      ),
      radial-gradient(ellipse at center, #3a2717, #1a110a 85%);
  }
  .dice-tray__felt[data-surface="stone"] {
    background:
      radial-gradient(circle at 20% 30%, rgba(255, 255, 255, 0.05), transparent 30%),
      radial-gradient(circle at 75% 70%, rgba(255, 255, 255, 0.04), transparent 35%),
      radial-gradient(ellipse at center, #3a3b3f, #1c1d20 85%);
  }
  .dice-tray__felt[data-surface="felt"] {
    background:
      repeating-radial-gradient(circle at 50% 50%, rgba(255, 255, 255, 0.015) 0 1px, transparent 1px 3px),
      radial-gradient(ellipse at center, #2b3a33, #121a16 85%);
  }
  /* Fixed height: a changing result must never resize the 3D table, because
     Dice Box rebuilds its walls on resize and settled dice would be lost. */
  .dice-tray__top {
    position: relative;
    z-index: 2;
    flex: 0 0 82px;
    height: 82px;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    min-width: 0;
  }
  .dice-tray__table {
    position: relative;
    flex: 1 1 auto;
    min-height: 120px;
  }
  /* The Catalyst mark printed into the mat: muted, behind the dice canvas,
     never the focal point and never in the way of input. */
  .dice-tray__mat-logo {
    position: absolute;
    top: 50%;
    left: 50%;
    width: min(70%, 420px);
    max-height: 60%;
    object-fit: contain;
    transform: translate(-50%, -50%);
    filter: grayscale(1) brightness(1.4);
    opacity: 0.09;
    mix-blend-mode: screen;
    pointer-events: none;
    user-select: none;
    z-index: 0;
  }

  .dice-strip {
    position: relative;
    z-index: 2;
    flex: 0 0 auto;
    align-self: center;
    display: flex;
    max-width: 100%;
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
  .dice-strip__cell--target {
    box-shadow: inset 0 -2px 0 var(--dice-gold);
  }
  .dice-strip__mod {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.06em;
    line-height: 20px;
    color: var(--dice-silver);
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
    max-height: calc(100% - 24px);
    overflow-y: auto;
    position: absolute;
    top: 12px;
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
    width: 40px;
    height: 40px;
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
    width: 34px;
    height: 34px;
  }
  .dice-die-button--counted {
    background: rgba(216, 171, 95, 0.08);
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
    flex-wrap: wrap;
  }
  .dice-compact__dice {
    justify-content: space-between;
    gap: 2px;
  }
  .dice-compact__selection {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--dice-muted);
    font-size: 13px;
    font-variant-numeric: tabular-nums;
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
    min-width: 30px;
    font-size: 13px;
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
