<script lang="ts">
  import type { DiceSettings } from "../stores/diceTray.svelte";

  let {
    settings,
    onChange,
  }: { settings: DiceSettings; onChange: (patch: Partial<DiceSettings>) => void } = $props();
</script>

<section class="dice-popover" aria-label="Dice settings" data-testid="dice-settings">
  <h3>Dice settings</h3>
  <label>
    <input
      type="checkbox"
      checked={settings.animate}
      onchange={(event) => onChange({ animate: event.currentTarget.checked })}
    />
    <span>
      3D physics animation
      <small>Off gives instant results. Follows your reduced-motion setting by default.</small>
    </span>
  </label>
  <label>
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
</section>

<style>
  h3 {
    font-family: inherit;
    margin: 0 0 8px;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--dice-muted);
  }
  label {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    padding: 6px 0;
    cursor: pointer;
  }
  input {
    margin-top: 3px;
    accent-color: var(--dice-gold);
  }
  small {
    display: block;
    color: var(--dice-muted);
    font-size: 12px;
    margin-top: 2px;
  }
</style>
