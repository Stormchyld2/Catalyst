<script lang="ts">
  import { Loader2 } from "lucide-svelte";
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";

  export type StructureDraft = {
    title: string;
    synopsis: string;
    pov: string;
    location: string;
    purpose: string;
    reveal: string;
  };

  let {
    kind,
    onSave,
    onClose,
  }: {
    kind: "chapter" | "scene";
    onSave: (draft: StructureDraft) => Promise<void>;
    onClose: () => void;
  } = $props();

  let title = $state("");
  let synopsis = $state("");
  let pov = $state("");
  let location = $state("");
  let purpose = $state("");
  let reveal = $state("");
  let saving = $state(false);
  let error = $state<string | null>(null);

  const heading = $derived(kind === "chapter" ? "New chapter" : "New scene");

  async function handleSave() {
    if (!title.trim()) {
      error = "Title is required.";
      return;
    }
    saving = true;
    error = null;
    try {
      await onSave({
        title: title.trim(),
        synopsis: synopsis.trim(),
        pov: pov.trim(),
        location: location.trim(),
        purpose: purpose.trim(),
        reveal: reveal.trim(),
      });
      onClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e || `Failed to create ${kind}`);
    } finally {
      saving = false;
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget && !saving) onClose();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="dialog-scrim"
  use:modalFocus={{ onEscape: () => !saving && onClose(), initialFocus: "#structure-title" }}
  onclick={handleBackdropClick}
  role="dialog"
  aria-modal="true"
  aria-labelledby="structure-dialog-title"
  tabindex="-1"
>
  <div class="app-dialog-surface dialog-shell" style="width:min(620px, calc(100vw - 48px));">
    <DialogHeader
      title={heading}
      titleId="structure-dialog-title"
      {onClose}
      closeLabel="Close"
    />

    <div class="ka-dialog-body" style="display:grid; gap:16px;">
      <div class="ka-field od-field">
        <label for="structure-title">Title</label>
        <input id="structure-title" bind:value={title} disabled={saving} placeholder={kind === "chapter" ? "Chapter title…" : "Scene title…"} />
      </div>

      <div class="ka-field od-field">
        <label for="structure-synopsis">Synopsis</label>
        <textarea id="structure-synopsis" bind:value={synopsis} disabled={saving} rows="4" placeholder={kind === "chapter" ? "What happens in this chapter?" : "What happens in this scene?"}></textarea>
      </div>

      {#if kind === "scene"}
        <div style="display:grid; grid-template-columns:1fr 1fr; gap:12px;">
          <div class="ka-field od-field">
            <label for="structure-pov">POV <span style="opacity:.65">(optional)</span></label>
            <input id="structure-pov" bind:value={pov} disabled={saving} placeholder="Point-of-view character" />
          </div>
          <div class="ka-field od-field">
            <label for="structure-location">Location <span style="opacity:.65">(optional)</span></label>
            <input id="structure-location" bind:value={location} disabled={saving} placeholder="Scene location" />
          </div>
        </div>
        <div class="ka-field od-field">
          <label for="structure-purpose">Purpose <span style="opacity:.65">(optional)</span></label>
          <input id="structure-purpose" bind:value={purpose} disabled={saving} placeholder="What this scene accomplishes" />
        </div>
        <div class="ka-field od-field">
          <label for="structure-reveal">Reveal <span style="opacity:.65">(optional)</span></label>
          <input id="structure-reveal" bind:value={reveal} disabled={saving} placeholder="Information revealed in this scene" />
        </div>
      {/if}

      {#if error}
        <p class="ka-error" role="alert">{error}</p>
      {/if}
    </div>

    <footer class="ka-dialog-footer">
      <button type="button" onclick={onClose} class="ka-button ka-button--secondary" disabled={saving}>Cancel</button>
      <button type="button" onclick={handleSave} class="ka-button" disabled={saving || !title.trim()} aria-busy={saving || undefined}>
        {#if saving}
          <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
          Creating…
        {:else}
          Create {kind}
        {/if}
      </button>
    </footer>
  </div>
</div>
