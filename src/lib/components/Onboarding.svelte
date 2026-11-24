<script lang="ts">
  import catalystLogo from "../../assets/catalyst-logo.png";
  import { ui } from "../stores/ui.svelte";
  import { modalFocus } from "../utils/modalFocus";
  const tour = [
    { step: "welcome", title: "Welcome to Catalyst", description: "Create campaigns, build worlds, prepare adventures and run the tabletop experience with local files." },
    { step: "tour-sidebar", title: "Campaign navigation", description: "Use the workspace rail for World Builder, Adventures, Characters and Campaign Notes. The global dice tray is available wherever you work." },
    { step: "tour-editor", title: "Build a playable adventure", description: "Prepare locations, NPCs, clues, encounters and sessions as editable documents. The Hollow Crown is a complete level-one demonstration." },
    { step: "tour-references", title: "World Builder and references", description: "Store lore and connect related records in your campaign wiki. Campaign files remain authoritative on your device." },
    { step: "import", title: "Import RPG material", description: "Use Import RPG Content on the Home screen for Markdown, Obsidian, images, maps, tokens, VTT exports and PDF reference files." },
  ] as const;
  const position = $derived(Math.max(0, tour.findIndex((item) => item.step === ui.onboardingStep)));
  const current = $derived(tour[position]);
  function next() {
    if (position === tour.length - 1) ui.completeOnboarding();
    else ui.nextStep();
  }
  function previous() { ui.previousStep(); }
</script>

{#if ui.showOnboarding}
  <div class="dialog-scrim" role="dialog" aria-modal="true" aria-labelledby="onboarding-title"
    data-testid="onboarding" tabindex="-1" use:modalFocus={{ onEscape: () => ui.skipOnboarding() }}>
    <div class="app-dialog-surface onboarding-dialog">
      <header class="header">
        <img src={catalystLogo} alt="Catalyst" width="54" height="54" />
        <button type="button" aria-label="Skip introduction" data-testid="skip-onboarding" onclick={() => ui.skipOnboarding()}>Close</button>
      </header>
      <p class="eyebrow">Catalyst RPG • {position + 1} of {tour.length}</p>
      <h2 id="onboarding-title">{current.title}</h2>
      <p>{current.description}</p>
      {#if position === 0}
        <p>Everything starts with a campaign. New Campaign provides Quick Start or Guided Setup.</p>
      {:else if position === 4}
        <p>Importing requires an existing campaign destination. Original files are preserved where appropriate, and imported data never replaces your existing records without your consent.</p>
      {/if}
      <footer>
        <button type="button" class="secondary" onclick={() => ui.skipOnboarding()}>Skip for now</button>
        {#if position > 0}<button type="button" class="secondary" onclick={previous}>Back</button>{/if}
        <button type="button" class="primary" onclick={next}>{position === tour.length - 1 ? "Finish" : "Next"}</button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .dialog-scrim{position:fixed;inset:0;z-index:80;display:grid;place-items:center;background:rgba(0,0,0,.64);padding:20px}
  .onboarding-dialog{width:min(560px,100%);padding:26px;background:var(--color-surface);border:1px solid var(--color-border);border-radius:var(--radius-l);color:var(--color-text);box-shadow:0 25px 90px rgba(0,0,0,.3)}
  .header{display:flex;justify-content:space-between;align-items:center}
  .header button,.secondary,.primary{padding:10px 13px;background:var(--color-surface);border:1px solid var(--color-border);border-radius:var(--radius-m);color:var(--color-text);cursor:pointer}
  .primary{background:color-mix(in srgb,var(--color-accent) 22%,var(--color-surface));border-color:var(--color-accent)}
  .eyebrow{font-size:.75rem;letter-spacing:.08em;text-transform:uppercase;color:var(--color-accent-text)}
  h2{font-size:1.6rem;font-family:var(--font-display)}
  p{font-size:.92rem;line-height:1.6}
  footer{display:flex;justify-content:flex-end;gap:10px;margin-top:25px}
  footer .secondary:first-child{margin-right:auto}
</style>
