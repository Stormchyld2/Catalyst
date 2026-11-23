<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import type { Project } from "../types";

  interface OutlineBeat { id: string; content: string }
  interface OutlineScene { id: string; title: string; synopsis: string; plotline_ids: string[]; beats: OutlineBeat[] }
  interface OutlineChapter { id: string; title: string; scenes: OutlineScene[] }
  interface Plotline { id: string; title: string }
  interface Outline { version: number; plotlines: Plotline[]; chapters: OutlineChapter[] }
  interface Snapshot { outline: Outline; source_hash: string }

  let { project }: { project: Project } = $props();
  let snapshot = $state<Snapshot | null>(null);
  let loading = $state(true);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let selectedSceneId = $state<string | null>(null);
  let newPlotlineTitle = $state("");
  let editPlotlineId = $state<string | null>(null);
  let editPlotlineTitle = $state("");
  let editSynopsisSceneId = $state<string | null>(null);
  let editSynopsisText = $state("");

  const selectedScene = $derived(snapshot?.outline.chapters.flatMap((chapter) => chapter.scenes)
    .find((scene) => scene.id === selectedSceneId) ?? null);
  const hasUnassignedScenes = $derived(snapshot?.outline.chapters.some((chapter) =>
    chapter.scenes.some((scene) => scene.plotline_ids.length === 0)
  ) ?? false);

  async function persistOutline(outline: Outline) {
    if (!snapshot || busy) return;
    busy = true;
    error = null;
    try {
      snapshot = await invoke<Snapshot>("save_file_outline", {
        projectId: project.id,
        outline,
        expectedHash: snapshot.source_hash,
      });
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      busy = false;
    }
  }

  async function addPlotline() {
    const title = newPlotlineTitle.trim();
    if (!snapshot || busy || !title) return;
    const id = crypto.randomUUID();
    await persistOutline({
      ...snapshot.outline,
      plotlines: [...snapshot.outline.plotlines, { id, title }],
    });
    if (!error) newPlotlineTitle = "";
  }

  async function renamePlotline() {
    const title = editPlotlineTitle.trim();
    if (!snapshot || busy || !editPlotlineId || !title) return;
    await persistOutline({
      ...snapshot.outline,
      plotlines: snapshot.outline.plotlines.map((plotline) =>
        plotline.id === editPlotlineId ? { ...plotline, title } : plotline
      ),
    });
    if (!error) editPlotlineId = null;
  }

  async function setScenePlotline(sceneId: string, plotlineId: string, assigned: boolean) {
    if (!snapshot || busy) return;
    await persistOutline({
      ...snapshot.outline,
      chapters: snapshot.outline.chapters.map((chapter) => ({
        ...chapter,
        scenes: chapter.scenes.map((scene) => {
          if (scene.id !== sceneId) return scene;
          return {
            ...scene,
            plotline_ids: assigned
              ? [...new Set([...scene.plotline_ids, plotlineId])]
              : scene.plotline_ids.filter((id) => id !== plotlineId),
          };
        }),
      })),
    });
  }

  function selectScene(id: string) {
    selectedSceneId = id;
    editSynopsisSceneId = null;
  }

  async function saveSynopsis() {
    if (!snapshot || busy || !editSynopsisSceneId) return;
    const sceneId = editSynopsisSceneId;
    const synopsis = editSynopsisText.trim();
    await persistOutline({
      ...snapshot.outline,
      chapters: snapshot.outline.chapters.map((chapter) => ({
        ...chapter,
        scenes: chapter.scenes.map((scene) =>
          scene.id === sceneId ? { ...scene, synopsis } : scene
        ),
      })),
    });
    if (!error) editSynopsisSceneId = null;
  }

  async function reorderScene(sceneId: string, direction: -1 | 1) {
    if (!snapshot || busy) return;
    const chapter = snapshot.outline.chapters.find((item) => item.scenes.some((scene) => scene.id === sceneId));
    if (!chapter) return;
    const from = chapter.scenes.findIndex((scene) => scene.id === sceneId);
    const to = from + direction;
    if (to < 0 || to >= chapter.scenes.length) return;
    const scenes = [...chapter.scenes];
    [scenes[from], scenes[to]] = [scenes[to], scenes[from]];
    await persistOutline({
      ...snapshot.outline,
      chapters: snapshot.outline.chapters.map((item) =>
        item.id === chapter.id ? { ...item, scenes } : item
      ),
    });
  }

  onMount(() => { void refresh(); });

  async function refresh() {
    loading = true;
    error = null;
    try {
      snapshot = await invoke<Snapshot | null>("get_file_outline", { projectId: project.id });
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      loading = false;
    }
  }

  async function initialize() {
    if (busy) return;
    busy = true;
    error = null;
    try {
      snapshot = await invoke<Snapshot>("initialize_file_outline", { projectId: project.id });
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      busy = false;
    }
  }
</script>

<section class="outline-workspace" aria-labelledby="outline-title">
  <header class="outline-header">
    <div>
      <p class="eyebrow">Planning</p>
      <h1 id="outline-title">Outline</h1>
      <p>Chapters, scenes, and beats from your project's own files.</p>
    </div>
    <button type="button" onclick={refresh} disabled={loading || busy}>Refresh</button>
  </header>

  {#if error}
    <p role="alert" class="outline-error">{error}</p>
  {/if}
  {#if loading}
    <p role="status">Loading outline…</p>
  {:else if error && !snapshot}
    <p>Unable to determine whether an outline exists. Refresh to try again.</p>
  {:else if !snapshot}
    <div class="outline-empty">
      <h2>No outline yet</h2>
      <p>Initialize from Write to reuse existing chapter, scene, and beat identities. Manuscript prose is not modified.</p>
      <button type="button" onclick={initialize} disabled={busy}>{busy ? "Initializing…" : "Initialize from Write"}</button>
    </div>
  {:else}
    <div class="plotline-toolbar">
      <form onsubmit={(event) => { event.preventDefault(); void addPlotline(); }}>
        <label for="new-plotline-title">New plotline</label>
        <input id="new-plotline-title" aria-label="New plotline title" bind:value={newPlotlineTitle} placeholder="Plotline name" disabled={busy} />
        <button type="submit" disabled={busy || !newPlotlineTitle.trim()}>Add plotline</button>
      </form>
      {#each snapshot.outline.plotlines as plotline (plotline.id)}
        {#if editPlotlineId === plotline.id}
          <form onsubmit={(event) => { event.preventDefault(); void renamePlotline(); }}>
            <input aria-label="Rename plotline" bind:value={editPlotlineTitle} disabled={busy} />
            <button type="submit" disabled={busy || !editPlotlineTitle.trim()}>Save name</button>
            <button type="button" onclick={() => editPlotlineId = null} disabled={busy}>Cancel</button>
          </form>
        {:else}
          <button type="button" disabled={busy} onclick={() => { editPlotlineId = plotline.id; editPlotlineTitle = plotline.title; }}>Rename {plotline.title}</button>
        {/if}
      {/each}
    </div>
    <div class="outline-layout">
      <section class="timeline-shell" aria-label="Outline timeline">
        {#if snapshot.outline.chapters.length === 0}
          <p>No chapters yet. Add chapters in Write, then refresh the project structure.</p>
        {:else}
          <div
            class="timeline-grid"
            style:grid-template-columns={`11rem repeat(${snapshot.outline.chapters.length}, minmax(13rem, 1fr))`}
          >
            <div class="timeline-corner" aria-hidden="true">Plotline</div>
            {#each snapshot.outline.chapters as chapter (chapter.id)}
              <div class="chapter-header">
                <span class="chapter-kicker">Chapter</span>
                <h2>{chapter.title}</h2>
              </div>
            {/each}

            {#each snapshot.outline.plotlines as plotline (plotline.id)}
              <div class="lane-label">{plotline.title}</div>
              {#each snapshot.outline.chapters as chapter (chapter.id)}
                <div class="timeline-cell" aria-label={`${plotline.title}, ${chapter.title}`}>
                  {#each chapter.scenes.filter((scene) => scene.plotline_ids.includes(plotline.id)) as scene (scene.id)}
                    <button
                      type="button"
                      class="scene-card"
                      aria-label={scene.title}
                      class:selected={selectedSceneId === scene.id}
                      aria-pressed={selectedSceneId === scene.id}
                      onclick={() => selectScene(scene.id)}
                    >
                      <strong>{scene.title}</strong>
                      {#if scene.synopsis}<span>{scene.synopsis}</span>{/if}
                      {#if scene.beats.length}<small>{scene.beats.length} beat{scene.beats.length === 1 ? "" : "s"}</small>{/if}
                    </button>
                  {/each}
                </div>
              {/each}
            {/each}

            {#if hasUnassignedScenes || snapshot.outline.plotlines.length === 0}
              <div class="lane-label lane-label--muted">Unassigned</div>
              {#each snapshot.outline.chapters as chapter (chapter.id)}
                <div class="timeline-cell" aria-label={`Unassigned, ${chapter.title}`}>
                  {#each chapter.scenes.filter((scene) => scene.plotline_ids.length === 0) as scene (scene.id)}
                    <button
                      type="button"
                      class="scene-card"
                      aria-label={scene.title}
                      class:selected={selectedSceneId === scene.id}
                      aria-pressed={selectedSceneId === scene.id}
                      onclick={() => selectScene(scene.id)}
                    >
                      <strong>{scene.title}</strong>
                      {#if scene.synopsis}<span>{scene.synopsis}</span>{/if}
                      {#if scene.beats.length}<small>{scene.beats.length} beat{scene.beats.length === 1 ? "" : "s"}</small>{/if}
                    </button>
                  {/each}
                </div>
              {/each}
            {/if}
          </div>
        {/if}
      </section>

      <aside class="outline-detail" aria-label="Scene details">
        {#if selectedScene}
          <p class="detail-kicker">Scene</p>
          <h2>{selectedScene.title}</h2>
          {#if editSynopsisSceneId === selectedScene.id}
            <form class="synopsis-form" onsubmit={(event) => { event.preventDefault(); void saveSynopsis(); }}>
              <label for="outline-scene-synopsis">Scene synopsis</label>
              <textarea id="outline-scene-synopsis" bind:value={editSynopsisText} disabled={busy} rows="5"></textarea>
              <div class="synopsis-actions">
                <button type="submit" disabled={busy}>Save synopsis</button>
                <button type="button" disabled={busy} onclick={() => editSynopsisSceneId = null}>Cancel</button>
              </div>
            </form>
          {:else}
            {#if selectedScene.synopsis}<p>{selectedScene.synopsis}</p>{/if}
            <button type="button" disabled={busy} onclick={() => { editSynopsisSceneId = selectedScene.id; editSynopsisText = selectedScene.synopsis; }}>Edit synopsis</button>
          {/if}
          {#if selectedScene.plotline_ids.length}
            <div class="plotline-chips" aria-label="Scene plotlines">
              {#each snapshot.outline.plotlines.filter((plotline) => selectedScene?.plotline_ids.includes(plotline.id)) as plotline (plotline.id)}
                <span>{plotline.title}</span>
              {/each}
            </div>
          {/if}
          {@const selectedChapter = snapshot.outline.chapters.find((chapter) => chapter.scenes.some((scene) => scene.id === selectedScene.id))}
          {#if selectedChapter}
            {@const sceneIndex = selectedChapter.scenes.findIndex((scene) => scene.id === selectedScene.id)}
            <div class="scene-order-controls" aria-label="Scene order">
              <span>Order in {selectedChapter.title}: {sceneIndex + 1} of {selectedChapter.scenes.length}</span>
              <button type="button" disabled={busy || sceneIndex === 0} onclick={() => void reorderScene(selectedScene.id, -1)}>Move scene up</button>
              <button type="button" disabled={busy || sceneIndex === selectedChapter.scenes.length - 1} onclick={() => void reorderScene(selectedScene.id, 1)}>Move scene down</button>
            </div>
          {/if}
          <h3>Plotline assignments</h3>
          {#if snapshot.outline.plotlines.length === 0}
            <p class="muted">Add a plotline to assign this scene.</p>
          {/if}
          <div class="assignment-list">
            {#each snapshot.outline.plotlines as plotline (plotline.id)}
              <label>
                <input
                  type="checkbox"
                  checked={selectedScene.plotline_ids.includes(plotline.id)}
                  disabled={busy}
                  onchange={(event) => void setScenePlotline(selectedScene.id, plotline.id, event.currentTarget.checked)}
                />
                {plotline.title}
              </label>
            {/each}
          </div>
          <h3>Beats</h3>
          {#if selectedScene.beats.length === 0}<p class="muted">No beats</p>{/if}
          <ol>{#each selectedScene.beats as beat (beat.id)}<li>{beat.content}</li>{/each}</ol>
        {:else}
          <p>Select a scene card to inspect its synopsis, plotlines, and beats.</p>
        {/if}
      </aside>
    </div>
  {/if}
</section>

<style>
  .outline-workspace { box-sizing: border-box; padding: 1.5rem; height: 100%; overflow: auto; color: var(--color-text, #eee); background: var(--color-bg, #242424); }
  .outline-header { display: flex; justify-content: space-between; align-items: start; gap: 1rem; margin-bottom: 1.25rem; }
  .outline-header h1 { margin: .25rem 0; }
  .outline-header p:not(.eyebrow) { margin: .35rem 0 0; color: var(--color-text-muted, #aaa); }
  .eyebrow, .chapter-kicker, .detail-kicker { margin: 0; color: var(--color-accent-text, #c7a457); font-size: .68rem; font-weight: 700; letter-spacing: .08em; text-transform: uppercase; }
  .muted { opacity: .7; }
  button { cursor: pointer; padding: .55rem .8rem; border: 1px solid var(--color-border, #555); border-radius: 6px; background: var(--color-surface, #343434); color: inherit; }
  button:disabled { opacity: .5; cursor: default; }
  .outline-error { color: var(--color-error, #ffb4a8); }
  .outline-empty { max-width: 34rem; padding: 1.5rem; border: 1px solid var(--color-border, #555); border-radius: 8px; background: var(--color-surface, #303030); }
  .plotline-toolbar { display: flex; flex-wrap: wrap; gap: .5rem; align-items: center; margin-bottom: 1rem; }
  .plotline-toolbar form { display: flex; gap: .4rem; align-items: center; }
  .plotline-toolbar label { font-size: .8rem; }
  .plotline-toolbar input { min-width: 9rem; padding: .5rem; border: 1px solid #666; border-radius: 5px; background: #292929; color: inherit; }
  .scene-order-controls { display: flex; flex-wrap: wrap; gap: .45rem; align-items: center; margin: .85rem 0; }
  .scene-order-controls span { flex-basis: 100%; font-size: .8rem; color: var(--color-text-muted, #aaa); }
  .synopsis-form { display: grid; gap: .5rem; margin: .75rem 0; }
  .synopsis-form label { font-size: .8rem; }
  .synopsis-form textarea { resize: vertical; padding: .6rem; border: 1px solid #666; border-radius: 5px; background: #292929; color: inherit; font: inherit; }
  .synopsis-actions { display: flex; gap: .5rem; }
  .assignment-list { display: grid; gap: .5rem; }
  .assignment-list label { display: flex; gap: .5rem; align-items: center; font-size: .85rem; }
  .outline-layout { display: grid; grid-template-columns: minmax(0, 1fr) minmax(18rem, 23rem); gap: 1rem; min-height: 0; }
  .timeline-shell { overflow: auto; border: 1px solid var(--color-border, #555); border-radius: 8px; background: var(--color-surface-sunken, #202020); }
  .timeline-grid { display: grid; min-width: max-content; align-items: stretch; }
  .timeline-corner, .chapter-header, .lane-label, .timeline-cell { border-right: 1px solid var(--color-border, #555); border-bottom: 1px solid var(--color-border, #555); }
  .timeline-corner { position: sticky; left: 0; z-index: 3; padding: .85rem 1rem; color: var(--color-text-muted, #aaa); background: var(--color-surface-raised, #303030); font-size: .72rem; font-weight: 700; text-transform: uppercase; letter-spacing: .06em; }
  .chapter-header { position: sticky; top: 0; z-index: 2; padding: .75rem .9rem; background: var(--color-surface-raised, #303030); }
  .chapter-header h2 { margin: .15rem 0 0; font-size: .95rem; }
  .lane-label { position: sticky; left: 0; z-index: 1; padding: .9rem 1rem; background: var(--color-surface, #292929); font-weight: 650; }
  .lane-label--muted { color: var(--color-text-muted, #aaa); font-style: italic; }
  .timeline-cell { min-height: 7rem; padding: .65rem; display: flex; flex-direction: column; gap: .55rem; background: color-mix(in srgb, var(--color-surface-sunken, #202020) 88%, transparent); }
  .scene-card { width: 100%; display: grid; gap: .35rem; text-align: left; padding: .7rem .75rem; border-color: var(--color-border, #555); background: var(--color-surface, #343434); box-shadow: 0 1px 0 rgba(0,0,0,.2); }
  .scene-card strong { font-size: .85rem; }
  .scene-card span { color: var(--color-text-muted, #aaa); font-size: .76rem; line-height: 1.35; }
  .scene-card small { color: var(--color-accent-text, #c7a457); font-size: .68rem; }
  .scene-card.selected { border-color: var(--color-accent-text, #c7a457); background: color-mix(in srgb, var(--color-accent-text, #c7a457) 14%, var(--color-surface, #343434)); }
  .outline-detail { position: sticky; top: 0; align-self: start; border: 1px solid var(--color-border, #555); border-radius: 8px; padding: 1rem; min-height: 14rem; background: var(--color-surface, #303030); }
  .outline-detail h2 { margin: .2rem 0 .65rem; }
  .outline-detail h3 { margin: 1.1rem 0 .5rem; font-size: .85rem; }
  .outline-detail ol { padding-left: 1.25rem; }
  .outline-detail li + li { margin-top: .45rem; }
  .plotline-chips { display: flex; flex-wrap: wrap; gap: .4rem; margin-top: .85rem; }
  .plotline-chips span { padding: .25rem .45rem; border: 1px solid var(--color-border, #555); border-radius: 999px; color: var(--color-accent-text, #c7a457); font-size: .7rem; }
  @media (max-width: 900px) { .outline-layout { grid-template-columns: 1fr; } .outline-detail { position: static; } }
</style>
