<script lang="ts">
  import type { DieType } from "../types/dice";

  let { type, size = 30 }: { type: DieType; size?: number } = $props();

  // Silhouettes echo the physical die, as in the approved mockup.
  const SHAPES: Record<DieType, string> = {
    d4: "16,3 30,28 2,28",
    d6: "4,4 28,4 28,28 4,28",
    d8: "16,1 31,16 16,31 1,16",
    d10: "16,2 30,12 26,28 6,28 2,12",
    d12: "16,1 29,9 29,23 16,31 3,23 3,9",
    d20: "16,1 29,8.5 29,23.5 16,31 3,23.5 3,8.5",
    d100: "11,2 22,2 30,16 22,30 11,30 3,16",
  };
  const label = $derived(type === "d100" ? "%" : type.slice(1));
</script>

<svg class="die-glyph" viewBox="0 0 32 32" width={size} height={size} aria-hidden="true">
  <polygon points={SHAPES[type]} />
  <text x="16" y={type === "d4" ? 23 : 17} dominant-baseline="middle" text-anchor="middle"
    font-size={label.length > 1 ? 10.5 : 13}>{label}</text>
</svg>

<style>
  .die-glyph polygon {
    fill: var(--dice-glyph-fill, #e9e6df);
    stroke: var(--dice-glyph-stroke, transparent);
    stroke-width: 1.5;
    stroke-linejoin: round;
  }
  .die-glyph text {
    fill: var(--dice-glyph-text, #1b1b1d);
    font-weight: 700;
    font-family: var(--font-ui, system-ui, sans-serif);
  }
</style>
