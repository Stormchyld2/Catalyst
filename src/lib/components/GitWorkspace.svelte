<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { CheckCircle2, CircleAlert, Download, GitBranch, RefreshCw, Upload } from "lucide-svelte";
  import type { Project } from "../types";

  interface GitStatus {
    root: string;
    is_repository: boolean;
    branch: string | null;
    remote: string | null;
    changes: string[];
  }
  interface CommandResult {
    command: string;
    exit_code: number | null;
    stdout: string;
    stderr: string;
  }

  let { project }: { project: Project } = $props();
  let status = $state<GitStatus | null>(null);
  let loading = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let remote = $state("");
  let commitMessage = $state("");
  let output = $state("");
  const root = $derived(project.project_root ?? "");

  onMount(refresh);

  async function refresh() {
    if (!root) return;
    loading = true;
    error = null;
    try {
      status = await invoke<GitStatus>("get_git_workspace_status", { projectRoot: root });
      remote = status.remote ?? "";
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function showResult(result: CommandResult | CommandResult[]) {
    const rows = Array.isArray(result) ? result : [result];
    output = rows.map((item) =>
      [`> ${item.command}`, item.stdout.trim(), item.stderr.trim()]
        .filter(Boolean).join("\n")
    ).join("\n\n");
  }

  async function run(command: string, args: Record<string, unknown> = {}) {
    if (!root || busy) return;
    busy = true;
    error = null;
    try {
      const result = await invoke<CommandResult | CommandResult[]>(command, {
        projectRoot: root,
        ...args,
      });
      showResult(result);
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="workspace" aria-labelledby="git-title">
  <header>
    <div>
      <p class="eyebrow">Version Control</p>
      <h1 id="git-title">Git</h1>
      <p>Commit, pull, and push the normal files in this project. Git remains optional and local-first.</p>
    </div>
    <button class="icon-button" type="button" onclick={refresh} disabled={loading || busy} aria-label="Refresh Git status">
      <span class:spin={loading}><RefreshCw size={17} /></span>
    </button>
  </header>

  {#if !root}
    <div class="notice error"><CircleAlert size={18} />This project does not have a local project folder yet.</div>
  {:else if error}
    <div class="notice error"><CircleAlert size={18} />{error}</div>
  {:else if status}
    <div class="cards">
      <article class="card repo-card">
        <div class="card-title"><GitBranch size={19} /><strong>Repository</strong></div>
        <p class="path">{status.root}</p>
        {#if status.is_repository}
          <div class="facts">
            <span><small>Branch</small><strong>{status.branch ?? "Detached HEAD"}</strong></span>
            <span><small>Origin</small><strong>{status.remote ?? "Not connected"}</strong></span>
          </div>
        {:else}
          <p>This project folder is not a Git repository yet.</p>
          <button class="primary" type="button" onclick={() => run("git_initialize")} disabled={busy}>Initialize Git Repository</button>
        {/if}
      </article>

      {#if status.is_repository}
        <article class="card status-card">
          <div class="card-title">
            {#if status.changes.length === 0}
              <CheckCircle2 size={19} /><strong>Working tree clean</strong>
            {:else}
              <CircleAlert size={19} /><strong>{status.changes.length} changed file{status.changes.length === 1 ? "" : "s"}</strong>
            {/if}
          </div>
          {#if status.changes.length}<pre>{status.changes.join("\n")}</pre>{:else}<p>No local changes to commit.</p>{/if}
        </article>

        <article class="card">
          <strong>GitHub / Remote</strong>
          <label>Origin URL<input bind:value={remote} placeholder="https://github.com/owner/repository.git" /></label>
          <button class="secondary" type="button" onclick={() => run("git_set_origin", { remote: remote.trim() })} disabled={busy || !remote.trim()}>Save Origin</button>
        </article>

        <article class="card">
          <strong>Commit changes</strong>
          <label>Commit message<input bind:value={commitMessage} placeholder="Describe this change" /></label>
          <button class="primary" type="button" onclick={async () => { await run("git_commit_all", { message: commitMessage.trim() }); commitMessage = ""; }} disabled={busy || !commitMessage.trim()}>Commit All Changes</button>
        </article>

        <div class="quick-actions">
          <button class="secondary" type="button" onclick={() => run("git_fetch")} disabled={busy}><RefreshCw size={15}/>Fetch</button>
          <button class="secondary" type="button" onclick={() => run("git_pull")} disabled={busy}><Download size={15}/>Pull</button>
          <button class="primary" type="button" onclick={() => run("git_push")} disabled={busy}><Upload size={15}/>Push</button>
        </div>
      {/if}
    </div>

    {#if output}
      <article class="output-card"><strong>Git output</strong><pre>{output}</pre></article>
    {/if}
  {/if}
</section>

<style>
  .workspace{box-sizing:border-box;width:100%;height:100%;overflow:auto;padding:30px 36px 48px;background:var(--color-bg);color:var(--color-text)}
  header{display:flex;align-items:flex-start;justify-content:space-between;gap:20px;max-width:1100px;margin:0 auto 22px}
  .eyebrow{margin:0 0 5px;color:var(--color-accent-text);font-size:10px;font-weight:700;letter-spacing:.09em;text-transform:uppercase}
  h1{margin:0;font:600 30px/1.2 var(--font-display)}
  header p:not(.eyebrow){margin:7px 0 0;color:var(--color-text-muted)}
  .icon-button{width:34px;height:34px;display:grid;place-items:center;border:1px solid var(--color-border);border-radius:6px;background:var(--color-surface);color:var(--color-text);cursor:pointer}
  .cards,.output-card,.notice{max-width:1100px;margin-inline:auto}
  .cards{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:14px}
  .card,.output-card,.notice{border:1px solid var(--color-border);border-radius:8px;background:var(--color-surface)}
  .card{display:grid;align-content:start;gap:12px;padding:17px}
  .repo-card,.status-card{grid-column:1/-1}
  .card-title{display:flex;align-items:center;gap:9px}
  .card-title :global(svg){color:var(--color-accent-text)}
  .path,.card p{margin:0;color:var(--color-text-muted);line-height:1.5}
  .facts{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px}
  .facts span{display:grid;gap:3px;padding:10px;border:1px solid var(--color-border);border-radius:6px;background:var(--color-surface-sunken)}
  small{color:var(--color-text-muted)}
  label{display:grid;gap:5px;color:var(--color-text-muted);font-size:11px;font-weight:600}
  input{box-sizing:border-box;width:100%;padding:9px;border:1px solid var(--color-border);border-radius:6px;background:var(--color-surface-sunken);color:var(--color-text)}
  button.primary,button.secondary{display:inline-flex;align-items:center;justify-content:center;gap:6px;padding:8px 11px;border:1px solid var(--color-border);border-radius:6px;cursor:pointer}
  .primary{background:var(--color-accent-text);color:var(--color-bg)}
  .secondary{background:var(--color-surface-raised);color:var(--color-text)}
  .quick-actions{grid-column:1/-1;display:flex;gap:8px}
  pre{margin:0;overflow:auto;white-space:pre-wrap;word-break:break-word;color:var(--color-text);font:12px/1.55 ui-monospace,SFMono-Regular,Consolas,monospace}
  .output-card{margin-top:14px;padding:17px}
  .output-card pre{margin-top:10px;max-height:280px}
  .notice{display:flex;align-items:center;gap:8px;padding:13px 15px;color:var(--color-text-muted)}
  .error{color:var(--color-error)}
  .spin{display:inline-flex;animation:spin 1s linear infinite}@keyframes spin{to{transform:rotate(360deg)}}
  @media(max-width:760px){.workspace{padding:22px}.cards{grid-template-columns:1fr}.facts{grid-template-columns:1fr}}
</style>