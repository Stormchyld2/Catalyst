<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onDestroy } from "svelte";
  import {
    AudioLines,
    CircleAlert,
    FolderInput,
    Music,
    Pencil,
    Plus,
    RefreshCw,
    Repeat,
    Square,
    Trash2,
    Wind,
    Zap,
  } from "lucide-svelte";
  import type { Project } from "../types";
  import {
    AUDIO_EXTENSIONS,
    CHANNELS,
    clampVolume,
    createPad,
    isExclusiveChannel,
    parseTags,
    type SoundChannel,
    type SoundPad,
    type SoundboardLayout,
  } from "../soundboard/layout";
  import { soundboard } from "../soundboard/board.svelte";

  let { project }: { project: Project } = $props();

  const root = $derived(project.project_root ?? "");
  const CHANNEL_ICONS = { music: Music, ambient: Wind, effects: Zap } as const;

  // Shared with the GM Screen so playback survives switching workspaces (doc 034).
  const engine = soundboard.engine;
  const layout = $derived(soundboard.layout);
  const files = $derived(soundboard.files);
  const loading = $derived(soundboard.loading);
  const error = $derived(soundboard.error);
  const saveState = $derived(soundboard.saveState);
  let editing = $state<SoundPad | null>(null);
  let editingIsNew = $state(false);
  let editingTags = $state("");

  const fileSet = $derived(new Set(files.map((f) => f.path)));
  const unusedFiles = $derived(files.filter((f) => !layout.pads.some((p) => p.file === f.path)));
  const anyPlaying = $derived(Object.keys(engine.playing).length > 0);

  $effect(() => {
    const next = root;
    if (next && next !== soundboard.root) {
      editing = null;
      void soundboard.open(next);
    }
  });

  onDestroy(() => void soundboard.flush());

  const refreshFiles = () => soundboard.refreshFiles();
  const commit = (next: SoundboardLayout) => soundboard.commit(next);

  function setMaster(value: number) {
    commit({ ...layout, master_volume: clampVolume(value) });
  }

  function setChannelVolume(channel: SoundChannel, value: number) {
    commit({ ...layout, channels: { ...layout.channels, [channel]: { volume: clampVolume(value) } } });
  }

  function padsIn(channel: SoundChannel) {
    return layout.pads.filter((p) => p.channel === channel);
  }

  function startAdd(channel: SoundChannel) {
    const file = unusedFiles[0]?.path ?? files[0]?.path ?? "";
    editing = file
      ? createPad(file, { channel, loop: isExclusiveChannel(channel) })
      : { id: "", label: "", file: "", channel, loop: isExclusiveChannel(channel), volume: 1 };
    editingIsNew = true;
    editingTags = "";
  }

  function startEdit(pad: SoundPad) {
    editing = { ...pad };
    editingIsNew = false;
    editingTags = (pad.tags ?? []).join(", ");
  }

  function saveEdit() {
    if (!editing || !editing.file) return;
    const pad: SoundPad = {
      ...editing,
      id: editing.id || createPad(editing.file).id,
      label: editing.label.trim() || createPad(editing.file).label,
      volume: clampVolume(editing.volume),
      tags: parseTags(editingTags),
    };
    if (pad.tags?.length === 0) delete pad.tags;
    const pads = editingIsNew ? [...layout.pads, pad] : layout.pads.map((p) => (p.id === pad.id ? pad : p));
    const previous = layout.pads.find((p) => p.id === pad.id);
    if (previous && (previous.file !== pad.file || previous.channel !== pad.channel)) engine.stop(pad.id, false);
    commit({ ...layout, pads });
    editing = null;
  }

  function deletePad(id: string) {
    engine.stop(id, false);
    commit({ ...layout, pads: layout.pads.filter((p) => p.id !== id) });
    editing = null;
  }

  function onEditorFileChange(file: string) {
    if (!editing) return;
    const oldDefault = editing.file ? createPad(editing.file).label : "";
    editing = {
      ...editing,
      file,
      label: !editing.label || editing.label === oldDefault ? createPad(file).label : editing.label,
    };
  }

  /** Add one pad per audio file that is not on the board yet. */
  function addAllUnused() {
    if (unusedFiles.length === 0) return;
    commit({ ...layout, pads: [...layout.pads, ...unusedFiles.map((f) => createPad(f.path))] });
  }

  async function importAudio() {
    if (!soundboard.root) return;
    soundboard.error = null;
    try {
      const picked = await open({
        multiple: true,
        directory: false,
        title: "Import audio into this campaign",
        filters: [{ name: "Audio", extensions: [...AUDIO_EXTENSIONS] }],
      });
      const sources = picked == null ? [] : Array.isArray(picked) ? picked : [picked];
      if (sources.length === 0) return;
      const imported = await invoke<string[]>("import_soundboard_audio", {
        projectRoot: soundboard.root,
        sourcePaths: sources,
      });
      await refreshFiles();
      commit({ ...layout, pads: [...layout.pads, ...imported.map((file) => createPad(file))] });
    } catch (e) {
      soundboard.error = `Import failed: ${String(e)}`;
    }
  }

  function percent(value: number) {
    return Math.round(clampVolume(value) * 100);
  }

  function playingIn(channel: SoundChannel) {
    return layout.pads.filter((p) => p.channel === channel && engine.playing[p.id]);
  }
</script>

<section class="soundboard" aria-labelledby="soundboard-title">
  <header class="sb-header">
    <div class="sb-title">
      <AudioLines size={22} aria-hidden="true" />
      <div>
        <h1 id="soundboard-title">Soundboard</h1>
        <p class="sb-sub">Plays audio from this campaign's <code>Assets/Audio</code> folder. Works fully offline.</p>
      </div>
    </div>
    <div class="sb-actions">
      <span class="sb-save" data-state={saveState} aria-live="polite">
        {#if saveState === "saving"}Saving…{:else if saveState === "saved"}Saved to campaign{:else if saveState === "error"}Not saved{/if}
      </span>
      <button type="button" class="sb-btn" onclick={importAudio} disabled={!root}>
        <FolderInput size={16} aria-hidden="true" /> Import audio
      </button>
      <button type="button" class="sb-btn" onclick={refreshFiles} disabled={!root} title="Rescan Assets/Audio">
        <RefreshCw size={16} aria-hidden="true" /> Rescan
      </button>
      <button type="button" class="sb-btn sb-btn--stop" onclick={() => engine.stopAll()} disabled={!anyPlaying}>
        <Square size={16} aria-hidden="true" /> Stop all
      </button>
    </div>
  </header>

  {#if !root}
    <p class="sb-empty">Open a campaign to use the soundboard.</p>
  {:else}
    {#if error}
      <p class="sb-error" role="alert"><CircleAlert size={16} aria-hidden="true" /> {error}</p>
    {/if}

    <div class="sb-mixer" role="group" aria-label="Mixer">
      <label class="sb-fader sb-fader--master">
        <span>Master <output>{percent(layout.master_volume)}%</output></span>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={layout.master_volume}
          oninput={(e) => setMaster(Number(e.currentTarget.value))}
          aria-label="Master volume"
        />
      </label>
      {#each CHANNELS as channel (channel.id)}
        {@const Icon = CHANNEL_ICONS[channel.id]}
        {@const nowPlaying = playingIn(channel.id)}
        <div class="sb-fader">
          <label>
            <span><Icon size={14} aria-hidden="true" /> {channel.label} <output>{percent(layout.channels[channel.id].volume)}%</output></span>
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={layout.channels[channel.id].volume}
              oninput={(e) => setChannelVolume(channel.id, Number(e.currentTarget.value))}
              aria-label={`${channel.label} volume`}
            />
          </label>
          <div class="sb-now">
            <span class="sb-now__label">{nowPlaying.length ? nowPlaying.map((p) => p.label).join(", ") : "Silent"}</span>
            {#if nowPlaying.length}
              <button type="button" class="sb-icon" onclick={() => engine.stopChannel(channel.id)} aria-label={`Stop ${channel.label}`}>
                <Square size={12} aria-hidden="true" />
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>

    {#if loading}
      <p class="sb-empty">Loading soundboard…</p>
    {:else}
      {#if files.length === 0 && layout.pads.length === 0}
        <div class="sb-empty sb-empty--card">
          <p><strong>No audio yet.</strong> Use <em>Import audio</em> to copy files into this campaign, or put them in the campaign's <code>Assets/Audio</code> folder and press <em>Rescan</em>.</p>
          <p class="sb-hint">Tip: files inside <code>Assets/Audio/Music</code> or <code>Assets/Audio/Ambience</code> become looping music or ambience pads.</p>
        </div>
      {:else if unusedFiles.length > 0}
        <p class="sb-hint">
          {unusedFiles.length} audio file{unusedFiles.length === 1 ? " is" : "s are"} not on the board yet.
          <button type="button" class="sb-link" onclick={addAllUnused}>Add {unusedFiles.length === 1 ? "it" : "them all"}</button>
        </p>
      {/if}

      {#each CHANNELS as channel (channel.id)}
        {@const Icon = CHANNEL_ICONS[channel.id]}
        <section class="sb-section" aria-labelledby={`sb-${channel.id}`}>
          <h2 id={`sb-${channel.id}`}><Icon size={16} aria-hidden="true" /> {channel.label}</h2>
          <p class="sb-hint">{channel.hint}</p>
          <div class="sb-grid">
            {#each padsIn(channel.id) as pad (pad.id)}
              {@const missing = !fileSet.has(pad.file)}
              {@const playing = engine.playing[pad.id] === true}
              {@const problem = engine.errors[pad.id]}
              <div class="sb-pad" class:is-playing={playing} class:is-missing={missing}>
                <button
                  type="button"
                  class="sb-pad__play"
                  aria-pressed={playing}
                  disabled={missing}
                  onclick={() => engine.toggle(pad)}
                  title={missing ? `${pad.file} is missing from Assets/Audio` : pad.file}
                >
                  <span class="sb-pad__label">{pad.label}</span>
                  <span class="sb-pad__meta">
                    {#if pad.loop}<Repeat size={12} aria-label="Loops" />{/if}
                    {#if missing}missing file{:else if problem}can't play{:else if playing}playing{/if}
                  </span>
                </button>
                <button type="button" class="sb-pad__edit" onclick={() => startEdit(pad)} aria-label={`Edit ${pad.label}`}>
                  <Pencil size={14} aria-hidden="true" />
                </button>
                {#if problem}<span class="sr-only" role="status">{pad.label}: {problem}</span>{/if}
              </div>
            {/each}
            <button type="button" class="sb-pad sb-pad--add" onclick={() => startAdd(channel.id)} disabled={files.length === 0}>
              <Plus size={18} aria-hidden="true" /> Add pad
            </button>
          </div>
        </section>
      {/each}
    {/if}
  {/if}

  {#if editing}
    <div class="sb-dialog-backdrop" role="presentation" onclick={() => (editing = null)}></div>
    <div
      class="sb-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="sb-edit-title"
      tabindex="-1"
      onkeydown={(e) => {
        if (e.key === "Escape") {
          e.stopPropagation();
          editing = null;
        }
      }}
    >
      <h2 id="sb-edit-title">{editingIsNew ? "Add pad" : "Edit pad"}</h2>
      <form
        onsubmit={(e) => {
          e.preventDefault();
          saveEdit();
        }}
      >
        <label>
          <span>Name</span>
          <input type="text" bind:value={editing.label} maxlength="80" placeholder="Tavern night" />
        </label>
        <label>
          <span>Audio file</span>
          <select value={editing.file} onchange={(e) => onEditorFileChange(e.currentTarget.value)} required>
            {#if !fileSet.has(editing.file)}
              <option value={editing.file} disabled>{editing.file || "Choose a file"}{editing.file ? " (missing)" : ""}</option>
            {/if}
            {#each files as f (f.path)}
              <option value={f.path}>{f.path}</option>
            {/each}
          </select>
        </label>
        <label>
          <span>Channel</span>
          <select bind:value={editing.channel}>
            {#each CHANNELS as c (c.id)}
              <option value={c.id}>{c.label}</option>
            {/each}
          </select>
        </label>
        <label>
          <span>Tags <small>(comma separated, for GM Screen pad sets)</small></span>
          <input type="text" bind:value={editingTags} maxlength="200" placeholder="combat, tavern" />
        </label>
        <label class="sb-check">
          <input type="checkbox" bind:checked={editing.loop} />
          <span>Loop until stopped</span>
        </label>
        <label>
          <span>Pad volume <output>{percent(editing.volume)}%</output></span>
          <input type="range" min="0" max="1" step="0.01" bind:value={editing.volume} />
        </label>
        <div class="sb-dialog__actions">
          {#if !editingIsNew}
            <button type="button" class="sb-btn sb-btn--danger" onclick={() => editing && deletePad(editing.id)}>
              <Trash2 size={14} aria-hidden="true" /> Remove pad
            </button>
          {/if}
          <span class="sb-spacer"></span>
          <button type="button" class="sb-btn" onclick={() => (editing = null)}>Cancel</button>
          <button type="submit" class="sb-btn sb-btn--primary" disabled={!editing.file || !fileSet.has(editing.file)}>
            {editingIsNew ? "Add pad" : "Save"}
          </button>
        </div>
      </form>
    </div>
  {/if}
</section>

<style>
  .soundboard {
    height: 100%;
    overflow: auto;
    padding: 24px 28px 40px;
    background: var(--cat-bg);
    color: var(--cat-text);
    box-sizing: border-box;
  }
  .sb-header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 18px;
  }
  .sb-title {
    display: flex;
    gap: 12px;
    align-items: center;
    color: var(--cat-accent);
  }
  .sb-title h1 {
    margin: 0;
    font-size: 1.35rem;
    color: var(--cat-text);
  }
  .sb-sub,
  .sb-hint {
    margin: 2px 0 0;
    color: var(--cat-muted);
    font-size: 0.85rem;
  }
  code {
    background: var(--cat-inset);
    border: 1px solid var(--cat-border-subtle);
    border-radius: 4px;
    padding: 0 4px;
    font-size: 0.8rem;
  }
  .sb-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }
  .sb-save {
    color: var(--cat-muted);
    font-size: 0.8rem;
    min-width: 7em;
    text-align: right;
  }
  .sb-save[data-state="error"] {
    color: var(--cat-error);
  }
  .sb-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 7px 12px;
    border-radius: 8px;
    border: 1px solid var(--cat-border);
    background: var(--cat-raised);
    color: var(--cat-text);
    font: inherit;
    font-size: 0.85rem;
    cursor: pointer;
  }
  .sb-btn:hover:not(:disabled) {
    background: var(--cat-hover);
  }
  .sb-btn:disabled {
    color: var(--cat-disabled);
    cursor: default;
  }
  .sb-btn--stop:not(:disabled) {
    border-color: var(--cat-error);
    color: var(--cat-error);
  }
  .sb-btn--primary {
    background: var(--cat-accent);
    border-color: var(--cat-accent);
    color: var(--cat-on-accent);
  }
  .sb-btn--primary:hover:not(:disabled) {
    background: var(--cat-accent-hover);
  }
  .sb-btn--danger {
    color: var(--cat-error);
  }
  .sb-btn:focus-visible,
  .sb-pad__play:focus-visible,
  .sb-pad__edit:focus-visible,
  .sb-icon:focus-visible,
  .sb-link:focus-visible {
    outline: 2px solid var(--cat-focus);
    outline-offset: 2px;
  }
  .sb-error {
    display: flex;
    gap: 8px;
    align-items: center;
    color: var(--cat-error);
    background: var(--cat-inset);
    border: 1px solid var(--cat-error);
    border-radius: 8px;
    padding: 8px 12px;
  }
  .sb-mixer {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
    gap: 12px;
    background: var(--cat-panel);
    border: 1px solid var(--cat-border-subtle);
    border-radius: 12px;
    padding: 14px 16px;
    margin-bottom: 18px;
  }
  .sb-fader label,
  .sb-fader--master {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 0.85rem;
    color: var(--cat-silver);
  }
  .sb-fader span {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  output {
    margin-left: auto;
    color: var(--cat-muted);
    font-variant-numeric: tabular-nums;
  }
  input[type="range"] {
    accent-color: var(--cat-accent);
    width: 100%;
  }
  .sb-now {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 4px;
    min-height: 20px;
  }
  .sb-now__label {
    font-size: 0.75rem;
    color: var(--cat-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sb-icon {
    border: 1px solid var(--cat-border);
    background: var(--cat-raised);
    color: var(--cat-text);
    border-radius: 6px;
    padding: 2px 4px;
    cursor: pointer;
    display: inline-flex;
  }
  .sb-link {
    border: none;
    background: none;
    color: var(--cat-accent);
    cursor: pointer;
    font: inherit;
    padding: 0;
    text-decoration: underline;
  }
  .sb-section {
    margin-top: 22px;
  }
  .sb-section h2 {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 1rem;
    margin: 0;
    color: var(--cat-silver);
  }
  .sb-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 10px;
    margin-top: 10px;
  }
  .sb-pad {
    position: relative;
    min-height: 92px;
    border-radius: 12px;
    border: 1px solid var(--cat-border);
    background: var(--cat-panel);
  }
  .sb-pad__play {
    width: 100%;
    height: 100%;
    min-height: 92px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: flex-start;
    gap: 8px;
    padding: 12px 34px 10px 12px;
    border: none;
    border-radius: 12px;
    background: transparent;
    color: var(--cat-text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .sb-pad:hover {
    background: var(--cat-raised);
  }
  .sb-pad.is-playing {
    border-color: var(--cat-accent);
    background: var(--cat-accent-soft);
    box-shadow: 0 0 0 1px var(--cat-accent) inset;
  }
  .sb-pad.is-missing .sb-pad__play {
    color: var(--cat-disabled);
    cursor: not-allowed;
  }
  .sb-pad__label {
    font-weight: 600;
    word-break: break-word;
  }
  .sb-pad__meta {
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: 0.75rem;
    color: var(--cat-muted);
    min-height: 14px;
  }
  .is-playing .sb-pad__meta {
    color: var(--cat-accent-hover);
  }
  .is-missing .sb-pad__meta {
    color: var(--cat-warning);
  }
  .sb-pad__edit {
    position: absolute;
    top: 8px;
    right: 8px;
    border: none;
    background: transparent;
    color: var(--cat-muted);
    border-radius: 6px;
    padding: 4px;
    cursor: pointer;
    display: inline-flex;
  }
  .sb-pad__edit:hover {
    color: var(--cat-text);
    background: var(--cat-hover);
  }
  .sb-pad--add {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border-style: dashed;
    color: var(--cat-muted);
    font: inherit;
    cursor: pointer;
  }
  .sb-pad--add:disabled {
    cursor: default;
    color: var(--cat-disabled);
  }
  .sb-empty {
    color: var(--cat-muted);
  }
  .sb-empty--card {
    background: var(--cat-panel);
    border: 1px dashed var(--cat-border);
    border-radius: 12px;
    padding: 16px 18px;
  }
  .sb-dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 0.5);
    z-index: 40;
  }
  .sb-dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: min(440px, calc(100vw - 32px));
    background: var(--cat-panel);
    border: 1px solid var(--cat-border);
    border-radius: 14px;
    padding: 18px 20px;
    z-index: 41;
    box-shadow: 0 20px 60px rgb(0 0 0 / 0.5);
  }
  .sb-dialog h2 {
    margin: 0 0 12px;
    font-size: 1.05rem;
  }
  .sb-dialog form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .sb-dialog label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 0.85rem;
    color: var(--cat-silver);
  }
  .sb-dialog label span {
    display: flex;
  }
  .sb-dialog input[type="text"],
  .sb-dialog select {
    background: var(--cat-inset);
    border: 1px solid var(--cat-border);
    border-radius: 8px;
    color: var(--cat-text);
    padding: 7px 9px;
    font: inherit;
  }
  .sb-dialog .sb-check {
    flex-direction: row;
    align-items: center;
    gap: 8px;
  }
  .sb-check input {
    accent-color: var(--cat-accent);
  }
  .sb-dialog__actions {
    display: flex;
    gap: 8px;
    margin-top: 6px;
  }
  .sb-spacer {
    flex: 1;
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
    white-space: nowrap;
  }
</style>
