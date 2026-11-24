<script lang="ts">
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";
  let { onClose }: { onClose: () => void } = $props();
</script>

<div class="dialog-scrim" use:modalFocus={{ onEscape: onClose }}
  role="dialog" aria-modal="true" aria-labelledby="quick-start-title" tabindex="-1">
  <div class="app-dialog-surface ka-dialog-default dialog-shell quick-start" data-testid="quick-start-dialog">
    <DialogHeader title="Catalyst RPG quick start" titleId="quick-start-title"
      {onClose} closeLabel="Close" closeTestId="quick-start-close" />
    <div class="body">
      <section>
        <h3>1. Start a campaign</h3>
        <p>Choose <strong>New Campaign</strong> on Home. Quick Start creates the essential campaign folders; Guided Setup lets you define the ruleset, world, party, NPCs, factions and starting session before creating files.</p>
      </section>
      <section>
        <h3>2. Open The Hollow Crown</h3>
        <p>Use <strong>The Hollow Crown — D&D 5e Demo</strong> on Home for a complete introductory level-one adventure, including GM notes, player reference cards, clues, encounters and offline maps.</p>
      </section>
      <section>
        <h3>3. Build your world and adventure</h3>
        <p><strong>World Builder</strong> contains your file-backed wiki, NPCs, locations, items and lore. <strong>Adventures</strong> organizes playable quests, arcs and scenes. <strong>Campaign Notes</strong> is for session plans, logs, handouts and read-aloud material.</p>
      </section>
      <section>
        <h3>4. Import existing material</h3>
        <p>Home → <strong>Import RPG Content</strong> accepts Markdown, Obsidian vaults, maps, portraits, tokens, Foundry/ Roll20 exports, Fantasy Grounds modules and PDFs. Select an existing campaign and preview the import. Some VTT data stays in preserved source files until full format adapters exist.</p>
      </section>
      <section>
        <h3>5. At the table</h3>
        <p>Open your campaign dashboard to view ruleset, party and actual saved records. Use the Dice Roller for rolls, and update session logs and campaign state after play. Data is stored locally in your campaign folder.</p>
      </section>
    </div>
    <footer><button class="ka-button" type="button" onclick={onClose}>Done</button></footer>
  </div>
</div>

<style>
  .dialog-scrim{position:fixed;inset:0;z-index:80;display:grid;place-items:center;padding:18px;background:rgba(0,0,0,.65)}
  .quick-start{width:min(720px,100%);max-height:min(90vh,900px);overflow:auto;background:var(--color-surface);border:1px solid var(--color-border);border-radius:var(--radius-l);color:var(--color-text)}
  .body{padding:12px 26px 20px;display:grid;gap:16px}
  section{border-top:1px solid var(--color-border);padding-top:14px}
  h3{margin:0;font-size:1rem}
  p{color:var(--color-text-muted);font-size:.89rem;line-height:1.65}
  footer{padding:12px 26px 24px;display:flex;justify-content:flex-end}
  .ka-button{padding:10px 18px;border:1px solid var(--color-accent);background:var(--color-surface);border-radius:var(--radius-m);color:var(--color-text);cursor:pointer}
</style>
