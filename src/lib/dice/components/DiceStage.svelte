<script lang="ts">
  /**
   * The 3D dice surface. Creates one renderer when mounted (lazily, only in
   * expanded mode with animation on) and disposes it on unmount, so closing
   * the tray or switching to compact mode releases GPU and physics resources.
   * The tray re-creates it (keyed) when appearance options change.
   */
  import { onMount } from "svelte";
  import {
    DiceBoxRenderer,
    RendererDisposedError,
    type DiceBoxRendererOptions,
    type DiceRenderer,
  } from "../engine/diceRenderer";
  import { diceTray } from "../stores/diceTray.svelte";

  type StageOptions = Pick<
    DiceBoxRendererOptions,
    "lowPerformance" | "themeColor" | "scale" | "gravity" | "onDieSettled"
  >;

  let {
    options,
    createRenderer = (options: StageOptions) => new DiceBoxRenderer(options),
  }: {
    options: StageOptions;
    createRenderer?: (options: StageOptions) => DiceRenderer;
  } = $props();

  let surface: HTMLDivElement;
  let stageState = $state<"loading" | "ready" | "unavailable">("loading");
  let reason = $state("");

  onMount(() => {
    const renderer = createRenderer({
      ...options,
      onDieSettled: () => diceTray.sound.playSettle(),
    });
    let alive = true;
    const observer =
      typeof ResizeObserver === "undefined" ? null : new ResizeObserver(() => renderer.resize());
    observer?.observe(surface);
    diceTray.rendererLoading();

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
        diceTray.rendererUnavailable();
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
  .dice-stage {
    position: absolute;
    inset: 0;
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
