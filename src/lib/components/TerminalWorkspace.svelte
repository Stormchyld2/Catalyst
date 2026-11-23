<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { CornerDownLeft, Trash2 } from "lucide-svelte";
  import type { Project } from "../types";

  interface CommandResult {
    command: string;
    exit_code: number | null;
    stdout: string;
    stderr: string;
  }

  let { project }: { project: Project } = $props();
  let command = $state("");
  let busy = $state(false);
  let history = $state<CommandResult[]>([]);
  let error = $state<string | null>(null);
  const root = $derived(project.project_root ?? "");

  async function run() {
    const value = command.trim();
    if (!root || !value || busy) return;
    busy = true;
    error = null;
    command = "";
    try {
      const result = await invoke<CommandResult>("run_project_terminal_command", {
        projectRoot: root,
        command: value,
      });
      history = [...history, result];
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  function keydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void run();
    }
  }
</script>

<section class="terminal-workspace" aria-labelledby="terminal-title">
  <header>
    <div>
      <p class="eyebrow">Project Tools</p>
      <h1 id="terminal-title">Terminal</h1>
      <p>Commands run with the active project folder as the working directory.</p>
    </div>
    <button type="button" class="clear" onclick={() => (history = [])} disabled={!history.length}><Trash2 size={15}/>Clear</button>
  </header>

  <div class="terminal-shell">
    {#if !root}
      <p class="error">This project does not have a local project folder yet.</p>
    {:else}
      <div class="terminal-meta"><span>{navigator.userAgent.includes("Windows") ? "PowerShell" : "Shell"}</span><code>{root}</code></div>
      <div class="history" aria-live="polite">
        {#if history.length === 0}<p class="hint">Type a command below. Try <code>git status</code> or <code>dir</code>.</p>{/if}
        {#each history as item}
          <section class="entry">
            <div class="prompt"><span>›</span><code>{item.command}</code></div>
            {#if item.stdout}<pre>{item.stdout}</pre>{/if}
            {#if item.stderr}<pre class="stderr">{item.stderr}</pre>{/if}
            {#if item.exit_code !== 0 && item.exit_code !== null}<small>Exit code {item.exit_code}</small>{/if}
          </section>
        {/each}
        {#if error}<p class="error">{error}</p>{/if}
      </div>
      <div class="command-row">
        <span class="prompt-mark">PS›</span>
        <input aria-label="Terminal command" bind:value={command} onkeydown={keydown} disabled={busy} placeholder="Enter command…" />
        <button type="button" onclick={run} disabled={busy || !command.trim()} aria-label="Run command"><CornerDownLeft size={16}/></button>
      </div>
    {/if}
  </div>
</section>

<style>
  .terminal-workspace{box-sizing:border-box;width:100%;height:100%;overflow:auto;padding:30px 36px 48px;background:var(--color-bg);color:var(--color-text)}
  header{display:flex;align-items:flex-start;justify-content:space-between;gap:20px;max-width:1180px;margin:0 auto 18px}
  .eyebrow{margin:0 0 5px;color:var(--color-accent-text);font-size:10px;font-weight:700;letter-spacing:.09em;text-transform:uppercase}
  h1{margin:0;font:600 30px/1.2 var(--font-display)}
  header p:not(.eyebrow){margin:7px 0 0;color:var(--color-text-muted)}
  .clear{display:inline-flex;align-items:center;gap:6px;padding:7px 10px;border:1px solid var(--color-border);border-radius:6px;background:var(--color-surface);color:var(--color-text);cursor:pointer}
  .terminal-shell{max-width:1180px;min-height:560px;margin:0 auto;border:1px solid var(--color-border);border-radius:9px;background:#0f1115;box-shadow:var(--shadow-md);overflow:hidden}
  .terminal-meta{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:9px 12px;border-bottom:1px solid rgb(255 255 255 / .09);color:#c7d0d9;font-size:11px}
  .terminal-meta code{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:#8ca7bf}
  .history{box-sizing:border-box;min-height:480px;max-height:calc(100vh - 290px);overflow:auto;padding:15px}
  .hint{color:#7f8a96}.hint code{color:#c7d0d9}
  .entry{margin-bottom:18px}.prompt{display:flex;gap:8px;color:#9fb8d0}.prompt code{color:#e6edf3}
  pre{margin:7px 0 0;white-space:pre-wrap;word-break:break-word;color:#d7dde4;font:12px/1.55 ui-monospace,SFMono-Regular,Consolas,monospace}
  .stderr,.error{color:#ef9a91}
  small{color:#f0bd6d}
  .command-row{display:grid;grid-template-columns:auto minmax(0,1fr) auto;align-items:center;gap:8px;padding:10px 12px;border-top:1px solid rgb(255 255 255 / .09);background:#151920}
  .prompt-mark{color:#d8ab5f;font:600 12px ui-monospace,SFMono-Regular,Consolas,monospace}
  input{width:100%;box-sizing:border-box;border:0;outline:0;background:transparent;color:#f0f3f5;font:13px ui-monospace,SFMono-Regular,Consolas,monospace}
  .command-row button{width:32px;height:32px;display:grid;place-items:center;border:1px solid rgb(255 255 255 / .12);border-radius:5px;background:#20262e;color:#dbe4ec;cursor:pointer}
  .error{padding:16px}
  @media(max-width:760px){.terminal-workspace{padding:22px}}
</style>