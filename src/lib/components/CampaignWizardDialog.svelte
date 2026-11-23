<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { ArrowLeft, ArrowRight, Compass, FolderOpen, Loader2, Plus, Trash2, Zap } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import { workspace } from "../stores/workspace.svelte";
  import type { Project } from "../types";
  import { open } from "../utils/nativeDialog";
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";
  import {
    CAMPAIGN_STARTERS,
    GAME_SYSTEMS,
    GM_TEMPLATES,
    GUIDED_STEPS,
    SAFETY_TOOLS,
    WORLD_CATEGORIES,
    buildCampaignInput,
    emptyDraft,
    firstInvalidStep,
    validateBasics,
    validateGuidedStep,
    type SetupMode,
  } from "../campaignWizard";

  interface NewCampaignProject {
    manifest: { project_id: string; title: string; project_type: string; created_at: string };
    paths: { root: string };
    overview_path: string;
    overview_id: string;
  }

  let {
    onClose,
    onComplete,
  }: {
    onClose: () => void;
    onComplete?: (project: Project) => void;
  } = $props();

  // Setup state is temporary until Create Campaign (see 016-NEW-BOOK-FOUNDATION).
  let draft = $state(emptyDraft());
  let mode = $state<SetupMode | null>(null);
  let step = $state(0);
  let saving = $state(false);
  let error = $state<string | null>(null);

  const isLast = $derived(step === GUIDED_STEPS.length - 1);
  const enabledCategories = $derived(WORLD_CATEGORIES.filter((category) => draft.categories[category.id].enabled));

  function choose(next: SetupMode) {
    mode = next;
    draft.mode = next;
    step = 0;
    error = null;
  }

  async function chooseProjectRoot() {
    error = null;
    const selected = await open({ directory: true, multiple: false, title: "Choose an empty folder for this campaign" });
    if (typeof selected === "string") draft.projectRoot = selected;
  }

  function next() {
    error = validateGuidedStep(step, draft);
    if (!error && !isLast) step += 1;
  }

  function back() {
    error = null;
    if (step > 0) step -= 1;
    else mode = null;
  }

  function toggle(list: string[], value: string): string[] {
    return list.includes(value) ? list.filter((item) => item !== value) : [...list, value];
  }

  function toProject(created: NewCampaignProject): Project {
    return {
      id: created.manifest.project_id,
      name: created.manifest.title,
      source_type: "Blank",
      source_path: null,
      project_root: created.paths.root,
      created_at: created.manifest.created_at,
      modified_at: created.manifest.created_at,
      author_pen_name: null,
      genre: draft.genre.trim() || null,
      description: draft.pitch.trim() || null,
      word_target: null,
      reference_types: [],
      project_type: "campaign",
      target_page_count: null,
    };
  }

  async function createCampaign() {
    if (saving) return;
    if (mode === "quick-start") {
      error = validateBasics(draft);
    } else {
      const invalid = firstInvalidStep(draft);
      if (invalid) step = invalid.step;
      error = invalid?.error ?? null;
    }
    if (error) return;

    const canReplace = window.dispatchEvent(new CustomEvent("lithomark:before-project-change", { cancelable: true }));
    if (!canReplace) return;

    saving = true;
    try {
      const created = await invoke<NewCampaignProject>("create_new_campaign", { input: buildCampaignInput(draft) });
      const project = toProject(created);
      currentProject.setProject(null);
      currentProject.setProject(project);
      ui.setView("editor");
      workspace.enterProject("story-development");
      onComplete?.(project);
      onClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter" && !saving && mode) {
      event.preventDefault();
      if (mode === "quick-start" || isLast) void createCampaign();
      else next();
    }
  }
</script>

{#snippet basics(showDetails: boolean)}
  <section class="form-grid">
    <label>Campaign name<input aria-label="Campaign name" bind:value={draft.name} placeholder="Untitled campaign" /></label>
    <label>
      Game system
      <input aria-label="Game system" list="campaign-game-systems" bind:value={draft.gameSystem} placeholder="Choose or type a system" />
      <datalist id="campaign-game-systems">{#each GAME_SYSTEMS as system}<option value={system}></option>{/each}</datalist>
    </label>
    <label class="wide">
      Campaign folder
      <div class="folder-picker">
        <input aria-label="Campaign folder" bind:value={draft.projectRoot} placeholder="Choose an empty local folder…" />
        <button type="button" onclick={chooseProjectRoot}><FolderOpen size={15} aria-hidden="true" />Browse</button>
      </div>
      <small>Campaign files become authoritative here when you press Create Campaign.</small>
    </label>
    {#if showDetails}
      <label>Edition / version<input aria-label="Edition" bind:value={draft.edition} /></label>
      <label>Genre<input aria-label="Genre" bind:value={draft.genre} placeholder="Dark fantasy, space opera…" /></label>
      <label class="wide">Tone<input aria-label="Tone" bind:value={draft.tone} placeholder="Heroic, grim, comedic…" /></label>
    {/if}
    <label class="wide">Campaign pitch<textarea aria-label="Campaign pitch" bind:value={draft.pitch} rows="3" placeholder="One or two sentences you'd tell your players."></textarea></label>
  </section>
{/snippet}

{#snippet starterPicker()}
  <div class="structure-options" role="radiogroup" aria-label="Campaign starter">
    {#each CAMPAIGN_STARTERS as starter (starter.id)}
      <label class="structure-option" class:selected={draft.starterId === starter.id}>
        <input type="radio" name="campaign-starter" value={starter.id} bind:group={draft.starterId} />
        <span><strong>{starter.name}</strong><small>{starter.description}</small></span>
      </label>
    {/each}
  </div>
{/snippet}

<div
  class="dialog-scrim"
  use:modalFocus={{ onEscape: onClose, onKeydown: handleKeydown }}
  role="dialog"
  aria-modal="true"
  aria-labelledby="new-campaign-title"
  tabindex="-1"
>
  <div class="foundation-shell">
    <DialogHeader title="New Campaign" titleId="new-campaign-title" {onClose} closeLabel="Close" />

    {#if mode === null}
      <main class="mode-picker">
        <p>How would you like to start? Both create the same campaign folder, and everything stays editable afterward.</p>
        <div class="mode-cards">
          <button type="button" class="mode-card" onclick={() => choose("quick-start")}>
            <Zap size={22} aria-hidden="true" />
            <strong>Quick Start</strong>
            <small>Name, system and a starter. Ready to prep in under a minute.</small>
          </button>
          <button type="button" class="mode-card" onclick={() => choose("guided")}>
            <Compass size={22} aria-hidden="true" />
            <strong>Guided Setup</strong>
            <small>Ten steps covering your table, world, NPCs, hooks and first session.</small>
          </button>
        </div>
      </main>
    {:else if mode === "quick-start"}
      <main class="step-content single">
        <header class="step-header"><p>Quick Start</p><h2>Create a campaign</h2></header>
        {@render basics(false)}
        <h3 class="sub-heading">Starter</h3>
        {@render starterPicker()}
        <p class="review-note">Quick Start also adds the Game Master toolkit templates, a Session Zero page and a Session 1 prep page.</p>
        {#if error}<p class="foundation-error" role="alert">{error}</p>{/if}
      </main>
    {:else}
      <div class="foundation-body">
        <nav class="steps" aria-label="Guided Setup steps">
          {#each GUIDED_STEPS as label, index}
            <button
              type="button"
              class:active={step === index}
              class:complete={index < step}
              disabled={saving || index > step}
              onclick={() => { error = null; step = index; }}
            >
              <span>{index + 1}</span>
              <strong>{label}</strong>
            </button>
          {/each}
        </nav>

        <main class="step-content">
          <header class="step-header">
            <p>Step {step + 1} of {GUIDED_STEPS.length}</p>
            <h2>{GUIDED_STEPS[step]}</h2>
          </header>

          {#if step === 0}
            {@render basics(true)}
          {:else if step === 1}
            <section class="form-grid">
              <label>Number of players<input aria-label="Number of players" inputmode="numeric" bind:value={draft.playerCount} placeholder="4" /></label>
              <label>Starting level / tier<input aria-label="Starting level" bind:value={draft.startingLevel} placeholder="1" /></label>
              <label>Session length<input aria-label="Session length" bind:value={draft.sessionLength} placeholder="3 hours" /></label>
              <label>Cadence<input aria-label="Cadence" bind:value={draft.cadence} placeholder="Weekly" /></label>
              <label class="wide">Play mode<input aria-label="Play mode" bind:value={draft.playMode} placeholder="In person, online, hybrid" /></label>
              <fieldset class="wide checks">
                <legend>Safety tools</legend>
                {#each SAFETY_TOOLS as tool}
                  <label class="check"><input type="checkbox" checked={draft.safetyTools.includes(tool)} onchange={() => (draft.safetyTools = toggle(draft.safetyTools, tool))} />{tool}</label>
                {/each}
              </fieldset>
              <label class="wide">Lines (hard limits)<input aria-label="Lines" bind:value={draft.lines} placeholder="Comma separated" /></label>
              <label class="wide">Veils (fade to black)<input aria-label="Veils" bind:value={draft.veils} placeholder="Comma separated" /></label>
            </section>
          {:else if step === 2}
            <section class="form-grid">
              <p class="wide hint">Add at least one detail to establish your world. You can expand it later in the wiki.</p>
              <label>Setting<input aria-label="Setting" bind:value={draft.setting} placeholder="The Shattered Isles" /></label>
              <label>Era<input aria-label="Era" bind:value={draft.era} /></label>
              <label>Magic level<input aria-label="Magic level" bind:value={draft.magicLevel} placeholder="None, low, high…" /></label>
              <label>Technology level<input aria-label="Technology level" bind:value={draft.technologyLevel} /></label>
              <label class="wide">World notes<textarea aria-label="World notes" bind:value={draft.worldNotes} rows="8"></textarea></label>
            </section>
          {:else if step === 3}
            <section class="entity-step">
              <p class="hint">Pick the categories you want seeded. Each one gets an overview page; names you list become their own pages.</p>
              {#each WORLD_CATEGORIES as category (category.id)}
                <article class="entity-card" class:selected={draft.categories[category.id].enabled}>
                  <label class="check strong"><input type="checkbox" aria-label={category.label} bind:checked={draft.categories[category.id].enabled} />{category.label}</label>
                  {#if draft.categories[category.id].enabled}
                    <div class="form-grid nested">
                      <label class="wide">Names<input aria-label={`${category.label} names`} bind:value={draft.categories[category.id].entries} placeholder="Comma separated (optional)" /></label>
                      <label class="wide">Notes<textarea aria-label={`${category.label} notes`} bind:value={draft.categories[category.id].notes} rows="3"></textarea></label>
                    </div>
                  {/if}
                </article>
              {/each}
            </section>
          {:else if step === 4}
            <section class="entity-step">
              <div class="step-intro">
                <p>Add the places your players are likely to visit first. You can skip this.</p>
                <button class="secondary" type="button" onclick={() => (draft.locations = [...draft.locations, { name: "", location_type: "", description: "" }])}><Plus size={14} aria-hidden="true" />Add location</button>
              </div>
              {#if draft.locations.length === 0}<div class="empty-card">No locations added.</div>{/if}
              {#each draft.locations as location, index}
                <article class="entity-card">
                  <header><strong>Location {index + 1}</strong><button type="button" aria-label={`Remove location ${index + 1}`} onclick={() => (draft.locations = draft.locations.filter((_, i) => i !== index))}><Trash2 size={14} /></button></header>
                  <div class="form-grid">
                    <label>Name<input aria-label={`Location ${index + 1} name`} bind:value={location.name} /></label>
                    <label>Type<input bind:value={location.location_type} placeholder="City, dungeon, tavern…" /></label>
                    <label class="wide">Description<textarea bind:value={location.description} rows="3"></textarea></label>
                  </div>
                </article>
              {/each}
            </section>
          {:else if step === 5}
            <section class="entity-step">
              <div class="step-intro">
                <p>Add key NPCs and villains. Player characters will come from Character Studio, which isn't available yet; a Player Characters folder is created for them.</p>
                <button class="secondary" type="button" onclick={() => (draft.npcs = [...draft.npcs, { name: "", role: "", affiliation: "", description: "", antagonist: false }])}><Plus size={14} aria-hidden="true" />Add NPC</button>
              </div>
              {#if draft.npcs.length === 0}<div class="empty-card">No NPCs added.</div>{/if}
              {#each draft.npcs as npc, index}
                <article class="entity-card">
                  <header><strong>NPC {index + 1}</strong><button type="button" aria-label={`Remove NPC ${index + 1}`} onclick={() => (draft.npcs = draft.npcs.filter((_, i) => i !== index))}><Trash2 size={14} /></button></header>
                  <div class="form-grid">
                    <label>Name<input aria-label={`NPC ${index + 1} name`} bind:value={npc.name} /></label>
                    <label>Role<input bind:value={npc.role} placeholder="Patron, rival, shopkeeper…" /></label>
                    <label>Affiliation<input bind:value={npc.affiliation} /></label>
                    <label class="check"><input type="checkbox" aria-label={`NPC ${index + 1} is a villain`} bind:checked={npc.antagonist} />Villain / antagonist</label>
                    <label class="wide">Description<textarea bind:value={npc.description} rows="3"></textarea></label>
                  </div>
                </article>
              {/each}
            </section>
          {:else if step === 6}
            <section class="form-grid">
              <label class="wide">Central conflict<textarea aria-label="Central conflict" bind:value={draft.centralConflict} rows="3"></textarea></label>
              <label>Main threat<input aria-label="Main threat" bind:value={draft.mainThreat} /></label>
              <label>Themes<input aria-label="Themes" bind:value={draft.themes} placeholder="Comma separated" /></label>
              <label class="wide">Adventure hooks<textarea aria-label="Adventure hooks" bind:value={draft.hooks} rows="5" placeholder="One hook per line"></textarea></label>
              <div class="wide"><h3 class="sub-heading">Campaign starter</h3>{@render starterPicker()}</div>
            </section>
          {:else if step === 7}
            <section class="form-grid">
              <label class="wide">Session Zero notes<textarea aria-label="Session Zero notes" bind:value={draft.sessionZeroNotes} rows="5"></textarea></label>
              <label class="wide">Opening scene<textarea aria-label="Opening scene" bind:value={draft.openingScene} rows="4" placeholder="How does Session 1 begin?"></textarea></label>
              <label class="wide">Session 1 notes<textarea aria-label="Session 1 notes" bind:value={draft.sessionOneNotes} rows="5"></textarea></label>
            </section>
          {:else if step === 8}
            <section class="form-grid">
              <fieldset class="wide checks">
                <legend>Starter templates for the Game Master folder</legend>
                {#each GM_TEMPLATES as template (template.id)}
                  <label class="check"><input type="checkbox" checked={draft.gmTemplateIds.includes(template.id)} onchange={() => (draft.gmTemplateIds = toggle(draft.gmTemplateIds, template.id))} />{template.title}</label>
                {/each}
              </fieldset>
              <label class="wide">House rules<textarea aria-label="House rules" bind:value={draft.houseRules} rows="6"></textarea></label>
              <p class="wide hint">Map Studio, Character Studio and the encounter tools aren't available yet. Their folders are created now so they can pick up this campaign later.</p>
            </section>
          {:else}
            <section class="review">
              <div class="review-hero">
                <h3>{draft.name || "Untitled campaign"}</h3>
                <p>{[draft.gameSystem, draft.edition, draft.genre].filter(Boolean).join(" · ") || "System not specified"}</p>
                <span>{draft.projectRoot}</span>
              </div>
              <div class="review-grid">
                <article><strong>Players</strong><span>{draft.playerCount || "Not set"}</span></article>
                <article><strong>World</strong><span>{draft.setting || draft.era || draft.worldNotes ? "Included" : "Required"}</span></article>
                <article><strong>World categories</strong><span>{enabledCategories.length}</span></article>
                <article><strong>Locations</strong><span>{draft.locations.length}</span></article>
                <article><strong>NPCs</strong><span>{draft.npcs.length}</span></article>
                <article><strong>Hooks</strong><span>{draft.hooks.split("\n").filter((line) => line.trim()).length}</span></article>
                <article><strong>Starter</strong><span>{CAMPAIGN_STARTERS.find((starter) => starter.id === draft.starterId)?.name}</span></article>
                <article><strong>GM templates</strong><span>{draft.gmTemplateIds.length}</span></article>
              </div>
              <p class="review-note">Create Campaign writes ordinary Catalyst Markdown/YAML files with stable IDs, then opens the campaign in Story Development. Setup data does not remain a second source of truth.</p>
            </section>
          {/if}

          {#if error}<p class="foundation-error" role="alert">{error}</p>{/if}
        </main>
      </div>
    {/if}

    {#if mode !== null}
      <footer class="foundation-footer">
        <button type="button" class="secondary" onclick={back} disabled={saving}>
          <ArrowLeft size={15} aria-hidden="true" />Back
        </button>
        <span>{mode === "quick-start" ? "Quick Start" : GUIDED_STEPS[step]}</span>
        {#if mode === "quick-start" || isLast}
          <button type="button" class="primary" onclick={createCampaign} disabled={saving}>
            {#if saving}<Loader2 class="spin" size={15} aria-hidden="true" />Creating…{:else}Create Campaign{/if}
          </button>
        {:else}
          <button type="button" class="primary" onclick={next} disabled={saving}>Next<ArrowRight size={15} aria-hidden="true" /></button>
        {/if}
      </footer>
    {/if}
  </div>
</div>

<style>
  .foundation-shell { width:min(1180px,96vw); height:min(860px,94vh); display:flex; flex-direction:column; overflow:hidden; border:1px solid var(--color-border); border-radius:var(--radius-l); background:var(--color-bg); box-shadow:0 24px 70px rgb(0 0 0 / .45); }
  .foundation-body { flex:1; min-height:0; display:grid; grid-template-columns:250px minmax(0,1fr); }
  .mode-picker { flex:1; display:grid; align-content:center; justify-items:center; gap:22px; padding:32px; }
  .mode-picker > p { max-width:560px; margin:0; text-align:center; color:var(--color-text-muted); line-height:1.55; }
  .mode-cards { display:grid; grid-template-columns:repeat(2,minmax(0,280px)); gap:16px; }
  .mode-card { display:grid; justify-items:start; gap:8px; padding:22px; border:1px solid var(--color-border); border-radius:10px; background:var(--color-surface); color:var(--color-text); text-align:left; cursor:pointer; }
  .mode-card:hover,.mode-card:focus-visible { border-color:var(--color-accent-text); color:var(--color-accent-text); }
  .mode-card strong { font:600 19px/1.2 var(--font-display); }
  .mode-card small { color:var(--color-text-muted); line-height:1.45; }
  .steps { min-height:0; overflow:auto; padding:12px 8px; border-right:1px solid var(--color-border); background:var(--color-surface); }
  .steps button { width:100%; display:grid; grid-template-columns:26px minmax(0,1fr); align-items:center; gap:8px; padding:8px; border:0; border-radius:6px; background:transparent; color:var(--color-text-muted); text-align:left; cursor:pointer; }
  .steps button span { width:22px; height:22px; display:grid; place-items:center; border:1px solid var(--color-border); border-radius:50%; font-size:10px; }
  .steps button.active { background:var(--color-surface-raised); color:var(--color-accent-text); }
  .steps button.complete span { border-color:var(--color-accent-text); color:var(--color-accent-text); }
  .steps button:disabled { cursor:default; opacity:.55; }
  .step-content { flex:1; min-height:0; overflow:auto; padding:28px 34px 40px; }
  .step-content.single { max-width:860px; width:100%; box-sizing:border-box; margin:0 auto; }
  .step-header { margin-bottom:22px; }
  .step-header p { margin:0 0 3px; color:var(--color-accent-text); font-size:10px; font-weight:600; letter-spacing:.08em; text-transform:uppercase; }
  .step-header h2 { margin:0; color:var(--color-text); font:600 27px/1.2 var(--font-display); }
  .sub-heading { margin:22px 0 10px; color:var(--color-text); font:600 15px/1.25 var(--font-display); }
  .form-grid { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:14px; }
  .form-grid.nested { margin-top:12px; }
  .form-grid label { display:grid; gap:5px; color:var(--color-text-muted); font-size:11px; font-weight:600; }
  .form-grid .wide { grid-column:1 / -1; }
  .form-grid input:not([type="checkbox"]):not([type="radio"]),.form-grid textarea { width:100%; box-sizing:border-box; padding:9px 10px; border:1px solid var(--color-border); border-radius:6px; background:var(--color-surface-sunken); color:var(--color-text); font:13px/1.5 var(--font-ui); }
  .form-grid textarea { resize:vertical; }
  .form-grid small { font-weight:400; }
  .checks { display:flex; flex-wrap:wrap; gap:8px 18px; margin:0; padding:12px; border:1px solid var(--color-border); border-radius:7px; }
  .checks legend { padding:0 4px; color:var(--color-text-muted); font-size:11px; font-weight:600; }
  label.check { display:flex; align-items:center; gap:7px; color:var(--color-text); font-size:12px; font-weight:400; cursor:pointer; }
  label.check.strong { font-weight:600; font-size:13px; }
  label.check input { accent-color:var(--color-accent-text); }
  .hint { margin:0; color:var(--color-text-muted); line-height:1.55; }
  .folder-picker { display:flex; gap:7px; }
  .folder-picker input { flex:1; }
  .folder-picker button,.secondary,.primary { display:inline-flex; align-items:center; justify-content:center; gap:6px; padding:8px 11px; border:1px solid var(--color-border); border-radius:6px; cursor:pointer; }
  .folder-picker button,.secondary { background:var(--color-surface-raised); color:var(--color-text); }
  .primary { background:var(--color-accent-text); color:var(--color-bg); }
  .entity-step { display:grid; gap:12px; }
  .step-intro { display:flex; align-items:center; justify-content:space-between; gap:16px; }
  .step-intro p,.review-note { color:var(--color-text-muted); line-height:1.55; }
  .entity-card,.empty-card,.review-hero,.review-grid article { border:1px solid var(--color-border); border-radius:7px; background:var(--color-surface); }
  .entity-card { padding:13px; }
  .entity-card.selected { border-color:var(--color-accent-text); }
  .entity-card > header { display:flex; align-items:center; justify-content:space-between; margin-bottom:12px; color:var(--color-text); }
  .entity-card > header button { display:grid; place-items:center; width:28px; height:28px; border:0; background:transparent; color:var(--color-text-muted); cursor:pointer; }
  .empty-card { padding:18px; color:var(--color-text-muted); }
  .structure-options { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:10px; }
  .structure-option { display:grid; grid-template-columns:auto 1fr; align-items:start; gap:11px; padding:12px; border:1px solid var(--color-border); border-radius:7px; background:var(--color-surface); color:var(--color-text); cursor:pointer; }
  .structure-option.selected { border-color:var(--color-accent-text); box-shadow:inset 3px 0 0 var(--color-accent-text); }
  .structure-option input { margin-top:3px; accent-color:var(--color-accent-text); }
  .structure-option > span { display:grid; gap:4px; }
  .structure-option small { color:var(--color-text-muted); font-weight:400; line-height:1.45; }
  .review { display:grid; gap:16px; }
  .review-hero { padding:18px; }
  .review-hero h3 { margin:0 0 5px; color:var(--color-text); font:600 22px/1.2 var(--font-display); }
  .review-hero p,.review-hero span { margin:0; color:var(--color-text-muted); }
  .review-grid { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:10px; }
  .review-grid article { display:flex; align-items:center; justify-content:space-between; padding:11px 12px; color:var(--color-text); }
  .review-grid article span { color:var(--color-text-muted); }
  .foundation-error { margin:16px 0 0; padding:9px 10px; border:1px solid var(--color-error); border-radius:6px; color:var(--color-error); }
  .foundation-footer { min-height:58px; display:grid; grid-template-columns:140px 1fr 160px; align-items:center; gap:10px; padding:8px 14px; border-top:1px solid var(--color-border); background:var(--color-surface); }
  .foundation-footer > span { text-align:center; color:var(--color-text-muted); font-size:11px; }
  .foundation-footer .primary { justify-self:end; min-width:140px; }
  @media (max-width: 760px) {
    .foundation-body { grid-template-columns:1fr; }
    .steps { display:none; }
    .mode-cards,.structure-options,.form-grid,.review-grid { grid-template-columns:1fr; }
  }
</style>
