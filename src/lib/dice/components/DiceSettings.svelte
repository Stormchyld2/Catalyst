<script lang="ts">
  import {
    DICE_COLORS,
    DICE_SIZES,
    ROLL_SPEEDS,
    TRAY_SURFACES,
    type DiceColor,
    type DiceSettings,
  } from "../stores/diceTray.svelte";

  let {
    settings,
    onChange,
  }: { settings: DiceSettings; onChange: (patch: Partial<DiceSettings>) => void } = $props();

  const SURFACE_LABELS = {
    charcoal: "Charcoal",
    leather: "Dark leather",
    walnut: "Walnut",
    stone: "Stone",
    felt: "Neutral felt",
  } as const;
  const SIZE_LABELS = { small: "Small", medium: "Medium", large: "Large" } as const;
  const SPEED_LABELS = { relaxed: "Relaxed", normal: "Normal", quick: "Quick" } as const;
  const colorEntries = Object.entries(DICE_COLORS) as [DiceColor, (typeof DICE_COLORS)[DiceColor]][];
</script>

<section class="dice-popover dice-settings" aria-label="Dice settings" data-testid="dice-settings">
  <h3>Dice appearance</h3>
  <div class="row" role="radiogroup" aria-label="Dice colour">
    {#each colorEntries as [key, color] (key)}
      <button
        type="button"
        class="swatch"
        role="radio"
        aria-checked={settings.diceColor === key}
        aria-label={color.label}
        title={color.label}
        style:--swatch={color.value}
        data-testid={`dice-color-${key}`}
        onclick={() => onChange({ diceColor: key })}
      ></button>
    {/each}
  </div>
  <div class="field">
    <span>Dice size</span>
    <div class="segmented" role="radiogroup" aria-label="Dice size">
      {#each Object.keys(DICE_SIZES) as size (size)}
        <button
          type="button"
          role="radio"
          aria-checked={settings.diceSize === size}
          onclick={() => onChange({ diceSize: size as DiceSettings["diceSize"] })}
          >{SIZE_LABELS[size as keyof typeof SIZE_LABELS]}</button
        >
      {/each}
    </div>
  </div>
  <label class="field">
    <span>Tray surface</span>
    <select
      value={settings.surface}
      data-testid="dice-surface"
      onchange={(event) =>
        onChange({ surface: event.currentTarget.value as DiceSettings["surface"] })}
    >
      {#each TRAY_SURFACES as surface (surface)}
        <option value={surface}>{SURFACE_LABELS[surface]}</option>
      {/each}
    </select>
  </label>

  <h3>Animation</h3>
  <label class="check">
    <input
      type="checkbox"
      checked={settings.animate}
      data-testid="dice-animate"
      onchange={(event) => onChange({ animate: event.currentTarget.checked })}
    />
    <span>
      3D physics animation
      <small>Off gives instant results. Follows your reduced-motion setting by default.</small>
    </span>
  </label>
  <div class="field">
    <span>Roll speed</span>
    <div class="segmented" role="radiogroup" aria-label="Roll speed">
      {#each Object.keys(ROLL_SPEEDS) as speed (speed)}
        <button
          type="button"
          role="radio"
          aria-checked={settings.speed === speed}
          disabled={!settings.animate}
          onclick={() => onChange({ speed: speed as DiceSettings["speed"] })}
          >{SPEED_LABELS[speed as keyof typeof SPEED_LABELS]}</button
        >
      {/each}
    </div>
  </div>
  <label class="check">
    <input
      type="checkbox"
      checked={settings.lowPerformance}
      onchange={(event) => onChange({ lowPerformance: event.currentTarget.checked })}
    />
    <span>
      Low-performance mode
      <small>Turns off shadows for slower graphics hardware.</small>
    </span>
  </label>

  <h3>Sound</h3>
  <label class="check">
    <input
      type="checkbox"
      checked={settings.sound}
      data-testid="dice-sound"
      onchange={(event) => onChange({ sound: event.currentTarget.checked })}
    />
    <span>Rolling sounds</span>
  </label>
  <label class="field">
    <span>Volume</span>
    <input
      type="range"
      min="0"
      max="100"
      step="5"
      value={Math.round(settings.volume * 100)}
      disabled={!settings.sound}
      aria-label="Volume"
      data-testid="dice-volume"
      oninput={(event) => onChange({ volume: Number(event.currentTarget.value) / 100 })}
    />
  </label>
</section>

<style>
  .dice-settings {
    max-height: calc(100% - 70px);
    overflow-y: auto;
  }
  h3 {
    font-family: inherit;
    margin: 10px 0 6px;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--dice-muted);
  }
  h3:first-child {
    margin-top: 0;
  }
  .row {
    display: flex;
    gap: 8px;
    margin-bottom: 8px;
  }
  .swatch {
    width: 24px;
    height: 24px;
    padding: 0;
    border-radius: 50%;
    border: 2px solid transparent;
    background: var(--swatch);
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.25);
    cursor: pointer;
  }
  .swatch[aria-checked="true"] {
    border-color: var(--dice-accent);
  }
  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 4px 0;
  }
  .segmented {
    display: inline-flex;
    border: 1px solid var(--dice-line);
    border-radius: 8px;
    overflow: hidden;
  }
  .segmented button {
    padding: 3px 9px;
    border: 0;
    background: transparent;
    color: var(--dice-silver);
    font-size: 12px;
    cursor: pointer;
  }
  .segmented button[aria-checked="true"] {
    background: rgba(110, 169, 207, 0.2);
    color: var(--dice-accent);
  }
  select {
    padding: 3px 6px;
    border-radius: 6px;
    border: 1px solid var(--dice-line);
    background: var(--dice-raised);
    color: var(--dice-text);
    font-size: 12px;
  }
  input[type="range"] {
    width: 140px;
    accent-color: var(--dice-accent);
  }
  .check {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    padding: 5px 0;
    cursor: pointer;
  }
  .check input {
    margin-top: 3px;
    accent-color: var(--dice-accent);
  }
  small {
    display: block;
    color: var(--dice-muted);
    font-size: 12px;
    margin-top: 2px;
  }
</style>
