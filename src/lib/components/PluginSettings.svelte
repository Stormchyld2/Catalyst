<script lang="ts">
  let { mode = "installed" }: { mode?: "installed" | "development" } = $props();
</script>

<div class="settings-pane plugin-settings">
  {#if mode === "installed"}
    <p class="settings-lede">
      Extend LithoMark without making plugins part of the authoritative project format.
      Projects remain ordinary local files even when plugins are disabled.
    </p>
    <section class="plugin-card">
      <div>
        <span class="status-dot"></span>
        <div>
          <strong>LithoMark Plugin API</strong>
          <p>Core plugin contracts are active. API version 1.</p>
        </div>
      </div>
      <span class="badge">Core</span>
    </section>
    <section class="plugin-card muted">
      <div>
        <span class="status-dot inactive"></span>
        <div>
          <strong>Installed plugins</strong>
          <p>No third-party plugins are installed in this preview build.</p>
        </div>
      </div>
      <span class="badge">0 installed</span>
    </section>
    <p class="ka-help">
      Plugin installation and browsing will use the same permission-gated API already enforced by Core.
    </p>
  {:else}
    <p class="settings-lede">
      Plugin development targets the stable LithoMark Core boundary rather than project internals.
    </p>
    <div class="dev-grid">
      <article>
        <strong>Manifest version</strong>
        <span>1</span>
      </article>
      <article>
        <strong>Plugin API</strong>
        <span>1</span>
      </article>
      <article>
        <strong>Project storage</strong>
        <span>Local files only</span>
      </article>
      <article>
        <strong>Permissions</strong>
        <span>Explicit grants</span>
      </article>
    </div>
    <div class="permission-list">
      <strong>Permission families</strong>
      <p>Project · Documents · Story Development · Editor · UI · Import / Export · Network · External filesystem · Process execution</p>
    </div>
  {/if}
</div>

<style>
  .plugin-settings{display:grid;gap:var(--space-s)}
  .plugin-card{display:flex;align-items:center;justify-content:space-between;gap:18px;padding:14px;border:var(--border-hair);border-radius:var(--radius-m);background:var(--color-surface)}
  .plugin-card>div{display:flex;align-items:flex-start;gap:10px}
  .plugin-card strong{display:block;color:var(--color-text)}
  .plugin-card p{margin:4px 0 0;color:var(--color-text-muted);font-size:var(--text-small)}
  .plugin-card.muted{opacity:.82}
  .status-dot{width:9px;height:9px;margin-top:6px;border-radius:50%;background:var(--color-success)}
  .status-dot.inactive{background:var(--color-text-muted)}
  .badge{flex:none;padding:3px 7px;border:var(--border-hair);border-radius:var(--radius-pill);color:var(--color-text-muted);font-size:var(--text-eyebrow)}
  .dev-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px}
  .dev-grid article,.permission-list{display:grid;gap:5px;padding:13px;border:var(--border-hair);border-radius:var(--radius-m);background:var(--color-surface)}
  .dev-grid span,.permission-list p{color:var(--color-text-muted);font-size:var(--text-small)}
  .permission-list p{margin:0;line-height:1.55}
</style>