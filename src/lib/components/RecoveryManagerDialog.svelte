<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { currentProject } from "../stores/project.svelte";

  let { onClose }: { onClose: () => void } = $props();
  interface Candidate {
    artifact_path: string;
    original_path: string | null;
    kind: string;
    valid_document: boolean;
  }
  let candidates = $state<Candidate[]>([]);
  let loading = $state(false);
  let restoring = $state(false);
  let error = $state<string | null>(null);
  let notice = $state<string | null>(null);
  let preview = $state<string | null>(null);
  let previewPath = $state<string | null>(null);

  async function scan() {
    const projectRoot = currentProject.value?.project_root;
    if (!projectRoot) return;
    loading = true;
    error = null;
    preview = null;
    try {
      const found = await invoke<Candidate[]>("list_document_recovery_candidates", { projectRoot });
      if (currentProject.value?.project_root === projectRoot) candidates = found;
    } catch (cause) { error = String(cause); }
    finally { loading = false; }
  }

  async function showPreview(candidate: Candidate) {
    const projectRoot = currentProject.value?.project_root;
    if (!projectRoot || !candidate.valid_document) return;
    preview = null;
    previewPath = candidate.artifact_path;
    error = null;
    try {
      const result = await invoke<string>("preview_document_recovery_candidate", {
        projectRoot, artifactPath: candidate.artifact_path,
      });
      if (currentProject.value?.project_root === projectRoot && previewPath === candidate.artifact_path)
        preview = result;
    } catch (cause) { error = String(cause); }
  }

  async function restore(candidate: Candidate) {
    const projectRoot = currentProject.value?.project_root;
    if (!projectRoot || !candidate.valid_document || !candidate.original_path || restoring) return;
    const pendingDocumentSaves: Promise<boolean>[] = [];
    window.dispatchEvent(new CustomEvent("catalyst:before-workspace-change", {
      detail: { pendingDocumentSaves },
    }));
    if ((await Promise.all(pendingDocumentSaves)).some((ok) => !ok)) {
      error = "Save the current document before restoring a recovery copy.";
      return;
    }
    restoring = true;
    error = null;
    notice = null;
    try {
      const review = await invoke<{ artifact_hash: string; destination_hash: string; original_path: string }>(
        "review_document_recovery_restore", { projectRoot, artifactPath: candidate.artifact_path }
      );
      if (currentProject.value?.project_root !== projectRoot) return;
      if (!window.confirm(
        'Restore the interrupted-save copy to "' + review.original_path + '"?\n\n' +
        "The current document will be preserved as a separate safety backup. " +
        "The recovery artifact will also be retained. Continue only if you recognize it."
      )) return;
      if (currentProject.value?.project_root !== projectRoot) return;
      const backup = await invoke<string>("restore_document_recovery_candidate", {
        projectRoot, artifactPath: candidate.artifact_path,
        expectedArtifactHash: review.artifact_hash,
        expectedDestinationHash: review.destination_hash,
      });
      if (currentProject.value?.project_root !== projectRoot) return;
      notice = "Restored " + review.original_path + ". Safety backup: " + backup;
      window.dispatchEvent(new CustomEvent("catalyst:recovery-restored", {
        detail: { relativePath: review.original_path, projectRoot },
      }));
      await scan();
    } catch (cause) { error = String(cause); }
    finally { restoring = false; }
  }

  $effect(() => { void scan(); });
</script>

<div class="recovery-overlay" role="presentation">
  <div class="recovery-dialog" role="dialog" aria-modal="true" aria-label="Recovery Manager" tabindex="-1">
    <header>
      <h2>Recovery Manager</h2>
      <button type="button" onclick={onClose} aria-label="Close Recovery Manager">Close</button>
    </header>
    <p>Inspect interrupted saves for the current project. Restoring requires confirmation and creates a safety backup.</p>
    <button type="button" disabled={loading || restoring} onclick={scan}>{loading ? "Scanning…" : "Rescan recovery files"}</button>
    {#if error}<p role="alert">{error}</p>{/if}
    {#if notice}<p role="status">{notice}</p>{/if}
    {#if !loading && candidates.length === 0 && !error}<p>No recovery files found.</p>{/if}
    {#each candidates as candidate (candidate.artifact_path)}
      <article class="candidate">
        <strong>{candidate.original_path ?? "Original document unknown"}</strong>
        <small>{candidate.kind} · {candidate.valid_document ? "Valid document" : "Invalid document"}</small>
        <small title={candidate.artifact_path}>{candidate.artifact_path}</small>
        <div class="actions">
          <button type="button" disabled={!candidate.valid_document} onclick={() => showPreview(candidate)}>Preview</button>
          <button type="button" disabled={!candidate.valid_document || !candidate.original_path || restoring} onclick={() => restore(candidate)}>
            {restoring ? "Restoring…" : "Restore…"}
          </button>
        </div>
        {#if previewPath === candidate.artifact_path && preview !== null}
          <pre aria-label="Recovery file preview">{preview}</pre>
        {/if}
      </article>
    {/each}
  </div>
</div>

<style>
  .recovery-overlay { position:fixed; inset:0; z-index:2500; background:rgb(0 0 0 / .7); display:flex; align-items:center; justify-content:center; padding:24px; }
  .recovery-dialog { width:min(760px,100%); max-height:85vh; overflow:auto; background:var(--color-surface); border:1px solid var(--color-border); border-radius:12px; padding:24px; color:var(--color-text); }
  header { display:flex; justify-content:space-between; align-items:center; gap:16px; }
  h2 { margin:0; }
  p { line-height:1.5; }
  button { padding:8px 12px; border:1px solid var(--color-border); border-radius:6px; background:var(--color-surface-raised); color:var(--color-text); cursor:pointer; }
  button:disabled { opacity:.5; cursor:not-allowed; }
  .candidate { border-top:1px solid var(--color-border); margin-top:16px; padding-top:16px; display:flex; flex-direction:column; gap:8px; }
  small { overflow-wrap:anywhere; color:var(--color-text-muted); }
  .actions { display:flex; gap:8px; }
  pre { max-height:280px; overflow:auto; white-space:pre-wrap; padding:12px; background:var(--color-surface-raised); }
</style>
