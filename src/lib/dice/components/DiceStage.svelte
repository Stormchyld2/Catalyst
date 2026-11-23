<script lang="ts">
  /**
   * The 3D dice surface. Creates one renderer when mounted (lazily, only in
   * expanded mode with animation on) and disposes it on unmount, so closing
   * the tray or switching to compact mode releases GPU and physics resources.
   */
  import { onMount } from "svelte";
  import {
    DiceBoxRenderer,
    RendererDisposedError,
    type DiceRenderer,
  } from "../engine/diceRenderer";
  import { diceTray } from "../stores/diceTray.svelte";

  let {
    lowPerformance,
    createRenderer = (options: { lowPerformance: boolean }) => new DiceBoxRenderer(options),
  }: {
    lowPerformance: boolean;
    createRenderer?: (options: { lowPerformance: boolean }) => DiceRenderer;
  } = $props();

  let surface: HTMLDivElement;
  let stageState = $state<"loading" | "ready" | "unavailable">("loading");
  let reason = $state("");

  onMount(() => {
    const renderer = createRenderer({ lowPerformance });
    let alive = true;
    const observer =
      typeof ResizeObserver === "undefined" ? null : new ResizeObserver(() => renderer.resize());
    observer?.observe(surface);

    renderer
      .init(surface)
      .then(() => {
        if (!alive) return;
        stageState = "ready";
        diceTray.attachRenderer(renderer);
      })
      .catch((error: unknown) => {
        if (!alive || error instanceof RendererDisposedError) return;
        console.warn("3D dice unavailable, using instant rolls:", error);
        reason = error instanceof Error ? error.message : String(error);
        stageState = "unavailable";
      });

    return () => {
      alive = false;
      observer?.disconnect();
      diceTray.detachRenderer(renderer);
      renderer.dispose();
    };
  });
</script>

<div class="dice-stage" bind:this={surface} data-testid="dice-stage" data-state={stageState}>
  {#if stageState === "loading"}
    <p class="dice-stage__note" role="status">Preparing the dice tray…</p>
  {:else if stageState === "unavailable"}
    <p class="dice-stage__note" role="status" title={reason}>
      3D dice are unavailable here. Rolls still work, without animation.
    </p>
  {/if}
</div>

<style>
  /* Leave room for the result pills above and the dice strip below, so
     settled dice are never hidden behind the overlays. */
  .dice-stage {
    position: absolute;
    inset: 58px 0 72px;
    overflow: hidden;
  }
  .dice-stage :global(.dice-renderer-host),
  .dice-stage :global(canvas) {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    display: block;
  }
  .dice-stage__note {
    position: absolute;
    inset: auto 0 50% 0;
    margin: 0;
    text-align: center;
    color: var(--dice-muted);
    font-size: 13px;
    pointer-events: none;
  }
</style>
