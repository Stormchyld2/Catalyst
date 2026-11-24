<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { ArrowLeft, ArrowRight, Compass, FilePlus2, FolderOpen, Loader2, Plus, SkipForward, Trash2, Zap } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import { workspace } from "../stores/workspace.svelte";
  import type { Project } from "../types";
  import { open } from "../utils/nativeDialog";
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";
  import {
    ERAS,
    GENRES,
    GUIDED_STEPS,
    HOUSE_RULE_MODES,
    RULES_PACKS,
    SETTING_ORIGINS,
    SETTING_SCALES,
    TONES,
    WORLD_CATEGORIES,
    WORLD_MODES,
    buildCampaignInput,
    emptyDraft,
    firstInvalidStep,
    genreLabel,
    plannedFiles,
    rulesName as packName,
    rulesPack,
    toneLabel,
    validateEssentials,
    validateGuidedStep,
    type CategorySpec,
    type Choice,
    type EntryDraft,
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

  // Wizard state is temporary until Create (020 §2). Cancel writes nothing.
  let draft = $state(emptyDraft());
  let mode = $state<SetupMode | null>(null);
  let step = $state(0);
  let furthest = $state(0);
  let saving = $state(false);
  let error = $state<string | null>(null);

  const isLast = $derived(step === GUIDED_STEPS.length - 1);
  const planned = $derived(plannedFiles(draft));
  const rulesName = $derived(packName(draft));
  const pack = $derived(rulesPack(draft.rulesPackId));
  const progression = $derived(pack.progressions.find((p) => p.id === draft.progressionType));
  const MODE_TITLES: Record<SetupMode, string> = { "quick-start": "Quick Start", guided: "Guided Campaign Builder", blank: "Blank Campaign" };

  function choose(next: SetupMode) {
    mode = next;
    draft.mode = next;
    step = 0;
    furthest = 0;
    error = null;
  }

  async function chooseProjectRoot() {
    error = null;
    const selected = await open({ directory: true, multiple: false, title: "Choose a new or empty folder for this campaign" });
    if (typeof selected === "string") draft.projectRoot = selected;
  }

  function goTo(index: number) {
    step = index;
    furthest = Math.max(furthest, index);
  }

  function next() {
    error = validateGuidedStep(step, draft);
    if (!error && !isLast) goTo(step + 1);
  }

  /** Optional steps can be skipped; answers already entered are kept. */
  function skip() {
    error = null;
    if (!isLast) goTo(step + 1);
  }

  function toggleTone(id: string) {
    draft.tones = draft.tones.includes(id) ? draft.tones.filter((t) => t !== id) : [...draft.tones, id];
  }

  function selectPack(id: string) {
    draft.rulesPackId = id;
    const next = rulesPack(id);
    if (!next.editions.some((e) => e.id === draft.edition)) draft.edition = next.editions[0]?.id ?? "";
    if (!next.progressions.some((p) => p.id === draft.progressionType)) draft.progressionType = "";
    if (!next.advancement.some((a) => a.id === draft.advancement)) draft.advancement = "";
  }

  function addWorldEntry(category: CategorySpec) {
    const value = draft.categories[category.id];
    value.entries = [...value.entries, { name: "", fields: {} }];
  }

  function back() {
    error = null;
    if (step > 0) step -= 1;
    else mode = null;
  }

  const blankEntry = (): EntryDraft => ({ name: "", kind: "", description: "" });

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
      genre: genreLabel(draft) || null,
      description: null,
      word_target: null,
      reference_types: [],
      project_type: "campaign",
      target_page_count: null,
    };
  }

  async function createCampaign() {
    if (saving) return;
    if (mode !== "guided") {
      error = validateEssentials(draft);
    } else {
      const invalid = firstInvalidStep(draft);
      if (invalid) step = invalid.step;
      error = invalid?.error ?? null;
    }
    if (error) return;

    const canReplace = window.dispatchEvent(new CustomEvent("catalyst:before-project-change", { cancelable: true }));
    if (!canReplace) return;

    saving = true;
    try {
      const created = await invoke<NewCampaignProject>("create_new_campaign", { input: buildCampaignInput(draft) });
      const project = toProject(created);
      currentProject.setProject(null);
      currentProject.setProject(project);
      ui.setView("editor");
      // No Campaign Dashboard exists yet; the campaign opens in the wiki.
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
      if (mode !== "guided" || isLast) void createCampaign();
      else next();
    }
  }
</script>

{#snippet folderPicker()}
  <label class="wide">
    Campaign folder
    <div class="folder-picker">
      <input aria-label="Campaign folder" bind:value={draft.projectRoot} placeholder="Choose a new or empty local folder…" />
      <button type="button" onclick={chooseProjectRoot}><FolderOpen size={15} aria-hidden="true" />Browse</button>
    </div>
    <small>Nothing is written until you press Create Campaign.</small>
  </label>
{/snippet}

{#snippet rulesPicker()}
  <div class="structure-options wide" role="radiogroup" aria-label="Rules system">
    {#each RULES_PACKS as option (option.id)}
      <label class="structure-option" class:selected={draft.rulesPackId === option.id}>
        <input type="radio" name="rules-pack" value={option.id} checked={draft.rulesPackId === option.id} onchange={() => selectPack(option.id)} />
        <span><strong>{option.name}</strong><small>{option.description}</small></span>
      </label>
    {/each}
  </div>
  {#if draft.rulesPackId === "custom-homebrew"}
    <label class="wide">System name<input aria-label="System name" bind:value={draft.customSystemName} placeholder="The game you're running" /></label>
  {/if}
{/snippet}

{#snippet question(text: string, hint?: string)}
  <div class="question wide"><strong>{text}</strong>{#if hint}<small>{hint}</small>{/if}</div>
{/snippet}

{#snippet chipRadio(name: string, choices: Choice[], value: string, onPick: (id: string) => void)}
  <div class="chips wide" role="radiogroup" aria-label={name}>
    {#each choices as choice (choice.id)}
      <label class="chip" class:selected={value === choice.id}>
        <input type="radio" name={name} value={choice.id} checked={value === choice.id} onchange={() => onPick(choice.id)} />{choice.label}
      </label>
    {/each}
  </div>
{/snippet}

{#snippet optionCards(name: string, choices: (Choice & { hint: string; available?: boolean })[], value: string, onPick: (id: string) => void)}
  <div class="structure-options wide" role="radiogroup" aria-label={name}>
    {#each choices as choice (choice.id)}
      <label class="structure-option" class:selected={value === choice.id} class:unavailable={choice.available === false}>
        <input type="radio" name={name} value={choice.id} checked={value === choice.id} disabled={choice.available === false} onchange={() => onPick(choice.id)} />
        <span><strong>{choice.label}</strong><small>{choice.hint}</small></span>
      </label>
    {/each}
  </div>
{/snippet}

{#snippet entryList(rows: EntryDraft[], noun: string, kindLabel: string, onChange: (rows: EntryDraft[]) => void)}
  <div class="step-intro">
    <h3 class="sub-heading">{noun}s</h3>
    <button class="secondary" type="button" onclick={() => onChange([...rows, blankEntry()])}><Plus size={14} aria-hidden="true" />Add {noun.toLowerCase()}</button>
  </div>
  {#if rows.length === 0}<div class="empty-card">No {noun.toLowerCase()}s added.</div>{/if}
  {#each rows as row, index}
    <article class="entity-card">
      <header><strong>{noun} {index + 1}</strong><button type="button" aria-label={`Remove ${noun.toLowerCase()} ${index + 1}`} onclick={() => onChange(rows.filter((_, i) => i !== index))}><Trash2 size={14} /></button></header>
      <div class="form-grid">
        <label>Name<input aria-label={`${noun} ${index + 1} name`} bind:value={row.name} /></label>
        <label>{kindLabel}<input aria-label={`${noun} ${index + 1} ${kindLabel.toLowerCase()}`} bind:value={row.kind} /></label>
        <label class="wide">Description<textarea aria-label={`${noun} ${index + 1} description`} bind:value={row.description} rows="2"></textarea></label>
      </div>
    </article>
  {/each}
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
        <p>How would you like to start? Every option creates the same campaign folders and files, and everything stays editable afterward.</p>
        <div class="mode-cards">
          <button type="button" class="mode-card" onclick={() => choose("quick-start")}>
            <Zap size={22} aria-hidden="true" />
            <strong>Quick Start</strong>
            <small>Name, folder, rules system and an optional premise. Ready to play in under a minute, with GM guides and random tables included.</small>
          </button>
          <button type="button" class="mode-card" onclick={() => choose("guided")}>
            <Compass size={22} aria-hidden="true" />
            <strong>Guided Campaign Builder</strong>
            <small>Ten steps that walk you through your system, world, party, NPCs, story and first session. Skip anything you haven't decided.</small>
          </button>
          <button type="button" class="mode-card" onclick={() => choose("blank")}>
            <FilePlus2 size={22} aria-hidden="true" />
            <strong>Blank Campaign</strong>
            <small>Just the folders and campaign files. No starter content.</small>
          </button>
        </div>
      </main>
    {:else if mode === "quick-start" || mode === "blank"}
      <main class="step-content single">
        <header class="step-header"><p>{MODE_TITLES[mode]}</p><h2>{mode === "blank" ? "Create an empty campaign" : "Create a campaign"}</h2></header>
        <section class="form-grid">
          <label class="wide">Campaign name<input aria-label="Campaign name" bind:value={draft.name} placeholder="Untitled campaign" /></label>
          {@render folderPicker()}
          {@render question("Which tabletop RPG system are you using?", mode === "blank" ? "Optional. System-Neutral is used if you don't choose." : undefined)}
          {@render rulesPicker()}
          {#if mode === "quick-start"}
            <label class="wide">Premise<textarea aria-label="Premise" bind:value={draft.premise} rows="3" placeholder="Optional. One or two sentences you'd tell your players."></textarea></label>
          {/if}
        </section>
        {#if error}<p class="foundation-error" role="alert">{error}</p>{/if}
      </main>
    {:else}
      <div class="foundation-body">
        <nav class="steps" aria-label="Campaign Builder steps">
          {#each GUIDED_STEPS as label, index}
            <button
              type="button"
              class:active={step === index}
              class:complete={index < furthest || (index < step)}
              disabled={saving || index > furthest}
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
            <section class="form-grid">
              <label class="wide">What is the name of your campaign?<input aria-label="Campaign name" bind:value={draft.name} placeholder="The Ashes of Varn" /></label>
              {@render question("What genre best describes your campaign?")}
              {@render chipRadio("Genre", GENRES, draft.genre, (id) => (draft.genre = id))}
              {#if draft.genre === "custom" || draft.genre === "mixed"}
                <label class="wide">Describe the genre<input aria-label="Custom genre" bind:value={draft.genreCustom} placeholder="Weird west with cosmic horror" /></label>
              {/if}
              {@render question("What tone or atmosphere do you want?", "Pick as many as fit.")}
              <div class="chips wide" role="group" aria-label="Tone">
                {#each TONES as tone (tone.id)}
                  <label class="chip" class:selected={draft.tones.includes(tone.id)}>
                    <input type="checkbox" checked={draft.tones.includes(tone.id)} onchange={() => toggleTone(tone.id)} />{tone.label}
                  </label>
                {/each}
              </div>
              {#if draft.tones.includes("custom")}
                <label class="wide">Describe the tone<input aria-label="Custom tone" bind:value={draft.toneCustom} /></label>
              {/if}
              <label class="wide">What is the basic premise of your campaign?
                <textarea aria-label="Premise" bind:value={draft.premise} rows="3" placeholder="Example: A border kingdom burns as an exiled queen raises an army of ash. The heroes are the last scouts of a fallen watch."></textarea>
                <small>Optional. A sentence or two you'd tell your players at the table.</small>
              </label>
              <label>Who is the Game Master?<input aria-label="GM display name" bind:value={draft.gmName} placeholder="Optional" /></label>
              <label>Planned starting date<input type="date" aria-label="Campaign start date" bind:value={draft.startDate} /><small>Optional. When your first session is planned.</small></label>
              <div class="wide">{@render question("Where should Catalyst save the campaign?")}</div>
              {@render folderPicker()}
              <p class="wide hint">Campaign artwork can be added to the Campaign Overview's image after creation.</p>
            </section>
          {:else if step === 1}
            <section class="form-grid">
              {@render question("Which tabletop RPG system are you using?", "No verified rules packs are installed yet, so only System-Neutral and Custom / Homebrew are offered. Installed systems will appear here.")}
              {@render rulesPicker()}
              {#if pack.editions.length > 1 || (pack.editions.length === 1 && pack.id !== "system-neutral")}
                {@render question("Which edition or ruleset?")}
                {@render chipRadio("Edition", pack.editions, draft.edition, (id) => (draft.edition = id))}
              {:else if pack.editions.length === 0}
                <label class="wide">Which edition or ruleset?<input aria-label="Edition" bind:value={draft.edition} placeholder="Optional, such as 2nd edition or your version number" /></label>
              {/if}
              {@render question("What starting progression should characters use?")}
              {@render chipRadio("Progression", pack.progressions, draft.progressionType, (id) => (draft.progressionType = id))}
              {#if progression}
                <label class="wide">{progression.input}<input aria-label={progression.input} bind:value={draft.startingLevel} placeholder={progression.placeholder} /></label>
              {/if}
              {@render question("How will characters advance?")}
              {@render chipRadio("Advancement", pack.advancement, draft.advancement, (id) => (draft.advancement = id))}
              {#if draft.advancement === "custom"}
                <label class="wide">Describe how characters advance<input aria-label="Custom advancement" bind:value={draft.advancementCustom} /></label>
              {/if}
              {@render question("Will you use house rules?")}
              {@render optionCards("House rules", HOUSE_RULE_MODES, draft.houseRulesMode, (id) => (draft.houseRulesMode = id))}
              {#if draft.houseRulesMode === "modified" || draft.houseRulesMode === "custom"}
                <label class="wide">House rules notes<textarea aria-label="House rules" bind:value={draft.houseRules} rows="4" placeholder="Optional. One rule per line, such as: Critical fumbles are off."></textarea><small>Saved as a House Rules page under Rules & Mechanics.</small></label>
              {/if}
              {@render question("Are there character-creation restrictions?", "Optional. Saved with the campaign settings so Character Studio can check them later. Catalyst doesn't enforce them yet.")}
              <label class="wide">Permitted content<textarea aria-label="Permitted content" bind:value={draft.allowedContent} rows="2" placeholder="One per line or comma separated, such as: Core rules only"></textarea></label>
              <label class="wide">Character option notes<textarea aria-label="Character creation notes" bind:value={draft.creationNotes} rows="2" placeholder="Banned options, required backgrounds, ties to the setting…"></textarea></label>
            </section>
          {:else if step === 2}
            <section class="form-grid">
              {@render question("What kind of world will the campaign use?")}
              {@render optionCards("World", WORLD_MODES, draft.worldMode, (id) => (draft.worldMode = id as typeof draft.worldMode))}
              {#if draft.worldMode === "new"}
                <label class="wide">What is the name of the setting or world?<input aria-label="World name" bind:value={draft.worldName} placeholder="Optional" /></label>
                {@render question("Is the setting original or based on an existing setting?")}
                {@render optionCards("Setting origin", SETTING_ORIGINS, draft.worldOrigin, (id) => (draft.worldOrigin = id))}
                {@render question("What is the scale of the setting?")}
                {@render chipRadio("Setting scale", SETTING_SCALES, draft.worldScale, (id) => (draft.worldScale = id))}
                {#if draft.worldScale === "custom"}
                  <label class="wide">Describe the scale<input aria-label="Custom scale" bind:value={draft.worldScaleCustom} /></label>
                {/if}
                {@render question("What is the general technological or historical era?")}
                {@render chipRadio("Era", ERAS, draft.era, (id) => (draft.era = id))}
                {#if draft.era === "custom"}
                  <label class="wide">Describe the era<input aria-label="Custom era" bind:value={draft.eraCustom} /></label>
                {/if}
                <label class="wide">What makes this setting distinctive?<textarea aria-label="What makes this setting distinctive" bind:value={draft.worldNotes} rows="4" placeholder="Optional. The sun never sets; the gods are dead; every city floats…"></textarea></label>
              {:else}
                <p class="wide hint">Worldbuilding stays empty for now. You can build the world from the wiki at any time.</p>
              {/if}
            </section>
          {:else if step === 3}
            <section class="entity-step">
              {@render question("Which parts of your world would you like to develop now?", "Pick any areas you already have ideas for. Each answer is saved to that area's wiki page. Nothing is created for areas you leave off.")}
              <div class="chips" role="group" aria-label="Worldbuilding areas">
                {#each WORLD_CATEGORIES as category (category.id)}
                  <label class="chip" class:selected={draft.categories[category.id].enabled}>
                    <input type="checkbox" aria-label={category.label} bind:checked={draft.categories[category.id].enabled} />{category.label}
                  </label>
                {/each}
              </div>
              {#each WORLD_CATEGORIES.filter((c) => draft.categories[c.id].enabled) as category (category.id)}
                {@const value = draft.categories[category.id]}
                <article class="entity-card selected">
                  <h3 class="card-title">{category.label}</h3>
                  <div class="form-grid">
                    {#each category.questions as q (q.key)}
                      <label class="wide">{q.label}<textarea aria-label={q.label} bind:value={value.answers[q.key]} rows="2" placeholder={q.placeholder ?? "Optional"}></textarea></label>
                    {/each}
                  </div>
                  {#if category.entries}
                    {@const spec = category.entries}
                    <div class="step-intro">
                      <strong class="question-inline">{spec.prompt}</strong>
                      <button class="secondary" type="button" onclick={() => addWorldEntry(category)}><Plus size={14} aria-hidden="true" />Add {spec.noun.toLowerCase()}</button>
                    </div>
                    {#each value.entries as row, index}
                      <div class="entry-row">
                        <header><strong>{spec.noun} {index + 1}</strong><button type="button" aria-label={`Remove ${spec.noun.toLowerCase()} ${index + 1}`} onclick={() => (value.entries = value.entries.filter((_, i) => i !== index))}><Trash2 size={14} /></button></header>
                        <div class="form-grid">
                          <label class="wide">Name<input aria-label={`${spec.noun} ${index + 1} name`} bind:value={row.name} /></label>
                          {#each spec.fields as field (field.key)}
                            <label class:wide={spec.fields.length === 1}>{field.label}
                              {#if field.options}
                                <select aria-label={`${spec.noun} ${index + 1} ${field.label}`} bind:value={row.fields[field.key]}>
                                  <option value="">Not decided</option>
                                  {#each field.options as option}<option value={option}>{option}</option>{/each}
                                </select>
                              {:else}
                                <input aria-label={`${spec.noun} ${index + 1} ${field.label}`} bind:value={row.fields[field.key]} />
                              {/if}
                            </label>
                          {/each}
                        </div>
                      </div>
                    {/each}
                  {/if}
                </article>
              {/each}
            </section>
          {:else if step === 4}
            <section class="entity-step">
              <div class="form-grid">
                <label>Party name<input aria-label="Party name" bind:value={draft.partyName} /></label>
                <label>Starting progression<input aria-label="Starting progression" bind:value={draft.startingProgression} /></label>
              </div>
              <p class="hint">Character Studio isn't available yet. A character name here creates a placeholder record, not a character sheet.</p>
              <div class="step-intro">
                <h3 class="sub-heading">Players</h3>
                <button class="secondary" type="button" onclick={() => (draft.players = [...draft.players, { player_name: "", character_name: "" }])}><Plus size={14} aria-hidden="true" />Add player</button>
              </div>
              {#if draft.players.length === 0}<div class="empty-card">No players added.</div>{/if}
              {#each draft.players as player, index}
                <article class="entity-card">
                  <header><strong>Player {index + 1}</strong><button type="button" aria-label={`Remove player ${index + 1}`} onclick={() => (draft.players = draft.players.filter((_, i) => i !== index))}><Trash2 size={14} /></button></header>
                  <div class="form-grid">
                    <label>Player name<input aria-label={`Player ${index + 1} name`} bind:value={player.player_name} /></label>
                    <label>Character name<input aria-label={`Player ${index + 1} character`} bind:value={player.character_name} placeholder="Optional" /></label>
                  </div>
                </article>
              {/each}
            </section>
          {:else if step === 5}
            <section class="entity-step">
              <div class="step-intro">
                <h3 class="sub-heading">NPCs</h3>
                <button class="secondary" type="button" onclick={() => (draft.npcs = [...draft.npcs, { name: "", disposition: "neutral", role: "", motivation: "", description: "" }])}><Plus size={14} aria-hidden="true" />Add NPC</button>
              </div>
              {#if draft.npcs.length === 0}<div class="empty-card">No NPCs added.</div>{/if}
              {#each draft.npcs as npc, index}
                <article class="entity-card">
                  <header><strong>NPC {index + 1}</strong><button type="button" aria-label={`Remove NPC ${index + 1}`} onclick={() => (draft.npcs = draft.npcs.filter((_, i) => i !== index))}><Trash2 size={14} /></button></header>
                  <div class="form-grid">
                    <label>Name<input aria-label={`NPC ${index + 1} name`} bind:value={npc.name} /></label>
                    <label>Disposition
                      <select aria-label={`NPC ${index + 1} disposition`} bind:value={npc.disposition}>
                        <option value="ally">Ally</option><option value="neutral">Neutral</option><option value="villain">Villain</option>
                      </select>
                    </label>
                    <label>Role<input aria-label={`NPC ${index + 1} role`} bind:value={npc.role} /></label>
                    <label>Motivation<input aria-label={`NPC ${index + 1} motivation`} bind:value={npc.motivation} /></label>
                    <label class="wide">Description<textarea aria-label={`NPC ${index + 1} description`} bind:value={npc.description} rows="2"></textarea></label>
                  </div>
                </article>
              {/each}
              {@render entryList(draft.creatures, "Creature", "Creature type", (rows) => { draft.creatures = rows; })}
              {@render entryList(draft.factions, "Faction", "Goal", (rows) => { draft.factions = rows; })}
            </section>
          {:else if step === 6}
            <section class="form-grid">
              <label class="wide">Central conflict<textarea aria-label="Central conflict" bind:value={draft.centralConflict} rows="3"></textarea></label>
              <label>Starting adventure<input aria-label="Starting adventure" bind:value={draft.startingAdventure} /></label>
              <label>Adventure summary<input aria-label="Adventure summary" bind:value={draft.startingAdventureSummary} /></label>
              <label class="wide">Story arcs<textarea aria-label="Story arcs" bind:value={draft.arcs} rows="4" placeholder="One per line"></textarea></label>
              <label class="wide">Side quests<textarea aria-label="Side quests" bind:value={draft.sideQuests} rows="4" placeholder="One per line"></textarea></label>
            </section>
          {:else if step === 7}
            <section class="entity-step">
              <div class="form-grid">
                <label class="wide">Starting location<input aria-label="Starting location" bind:value={draft.startingLocation} /></label>
              </div>
              {@render entryList(draft.locations, "Location", "Type", (rows) => { draft.locations = rows; })}
              <div class="form-grid">
                <label class="wide">Existing map files<textarea aria-label="Existing map files" bind:value={draft.mapReferences} rows="3" placeholder="One path or note per line"></textarea></label>
              </div>
              <p class="hint">Map Studio isn't available yet, so no maps are generated. The Maps folders are created and ready for your files.</p>
            </section>
          {:else if step === 8}
            <section class="form-grid">
              <label>First session title<input aria-label="First session title" bind:value={draft.firstSessionTitle} placeholder="Session 1" /></label>
              <label>Tabletop style
                <select aria-label="Tabletop style" bind:value={draft.playMode}>
                  <option value="physical">Physical</option><option value="hybrid">Hybrid</option><option value="digital">Digital</option>
                </select>
              </label>
              <label class="wide">First session notes<textarea aria-label="First session notes" bind:value={draft.firstSessionNotes} rows="4"></textarea></label>
              <label class="wide">GM notes<textarea aria-label="GM notes" bind:value={draft.gmNotes} rows="4"></textarea></label>
              <label>Dice
                <select aria-label="Dice" bind:value={draft.dice}>
                  <option value="physical">Physical dice</option><option value="digital">Digital dice</option>
                </select>
              </label>
              <label class="check"><input type="checkbox" aria-label="Use a player display" bind:checked={draft.playerDisplay} />Use a player display</label>
            </section>
          {:else}
            <section class="review">
              <div class="review-hero">
                <h3>{draft.name || "Untitled campaign"}</h3>
                <p>{[rulesName, genreLabel(draft), toneLabel(draft)].filter(Boolean).join(" · ")}</p>
                <span>{draft.projectRoot}</span>
              </div>
              <div class="review-grid">
                <article><strong>Rules</strong><span>{rulesName}</span></article>
                <article><strong>World</strong><span>{draft.worldMode === "new" ? draft.worldName || "New world" : "Decide later"}</span></article>
                <article><strong>Worldbuilding areas</strong><span>{WORLD_CATEGORIES.filter((c) => draft.categories[c.id].enabled).length}</span></article>
                <article><strong>Players</strong><span>{draft.players.length}</span></article>
                <article><strong>NPCs / creatures / factions</strong><span>{draft.npcs.length} / {draft.creatures.length} / {draft.factions.length}</span></article>
                <article><strong>Locations</strong><span>{draft.locations.length}</span></article>
                <article><strong>Tabletop style</strong><span>{draft.playMode}, {draft.dice} dice</span></article>
                <article><strong>First session</strong><span>{draft.firstSessionTitle || draft.firstSessionNotes ? "Planned" : "Not set"}</span></article>
              </div>
              <div class="planned">
                <h3 class="sub-heading">Files to be created ({planned.length})</h3>
                <ul aria-label="Planned files">{#each planned as file}<li>{file}</li>{/each}</ul>
                <p class="review-note">Plus the full campaign folder tree: Worldbuilding, Characters, Creatures & Monsters, Adventures, Encounters, Maps, Items & Equipment, Rules & Mechanics, Sessions, Game Master, Assets, and Exports & Print.</p>
              </div>
              {#if draft.players.some((p) => p.character_name.trim())}
                <p class="review-note warning">Player characters are placeholders until Character Studio is available.</p>
              {/if}
              <p class="review-note">Create Campaign builds everything in a temporary folder, checks it, and only then moves it into place. It never overwrites existing files.</p>
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
        <span class="footer-middle">
          {mode === "guided" ? GUIDED_STEPS[step] : MODE_TITLES[mode]}
          {#if mode === "guided" && step > 0 && !isLast}
            <button type="button" class="link" onclick={skip} disabled={saving}><SkipForward size={13} aria-hidden="true" />Skip this step</button>
          {/if}
        </span>
        {#if mode !== "guided" || isLast}
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
  .mode-cards { display:grid; grid-template-columns:repeat(3,minmax(0,260px)); gap:16px; }
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
  .form-grid label { display:grid; gap:5px; color:var(--color-text-muted); font-size:11px; font-weight:600; }
  .form-grid .wide { grid-column:1 / -1; }
  .form-grid input:not([type="checkbox"]):not([type="radio"]),.form-grid textarea { width:100%; box-sizing:border-box; padding:9px 10px; border:1px solid var(--color-border); border-radius:6px; background:var(--color-surface-sunken); color:var(--color-text); font:13px/1.5 var(--font-ui); }
  .form-grid textarea { resize:vertical; }
  .form-grid small { font-weight:400; }
  label.check { display:flex; align-items:center; gap:7px; color:var(--color-text); font-size:12px; font-weight:400; cursor:pointer; }
  label.check input { accent-color:var(--color-accent-text); }
  .hint { margin:0; color:var(--color-text-muted); line-height:1.55; }
  .folder-picker { display:flex; gap:7px; }
  .folder-picker input { flex:1; }
  .folder-picker button,.secondary,.primary { display:inline-flex; align-items:center; justify-content:center; gap:6px; padding:8px 11px; border:1px solid var(--color-border); border-radius:6px; cursor:pointer; }
  .folder-picker button,.secondary { background:var(--color-surface-raised); color:var(--color-text); }
  .primary { background:var(--color-accent-text); color:var(--color-bg); }
  .entity-step { display:grid; gap:12px; }
  .step-intro { display:flex; align-items:center; justify-content:space-between; gap:16px; }
  .review-note { color:var(--color-text-muted); line-height:1.55; }
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
  .form-grid select { width:100%; box-sizing:border-box; padding:9px 10px; border:1px solid var(--color-border); border-radius:6px; background:var(--color-surface-sunken); color:var(--color-text); font:13px/1.5 var(--font-ui); }
  .step-intro h3 { margin:8px 0 0; }
  .planned ul { max-height:220px; overflow:auto; margin:0; padding:10px 12px 10px 28px; border:1px solid var(--color-border); border-radius:7px; background:var(--color-surface-sunken); color:var(--color-text-muted); font:12px/1.6 ui-monospace,SFMono-Regular,Consolas,monospace; }
  .review-note.warning { color:var(--color-warning, var(--color-accent-text)); }
  .foundation-error { margin:16px 0 0; padding:9px 10px; border:1px solid var(--color-error); border-radius:6px; color:var(--color-error); }
  .foundation-footer { min-height:58px; display:grid; grid-template-columns:140px 1fr 160px; align-items:center; gap:10px; padding:8px 14px; border-top:1px solid var(--color-border); background:var(--color-surface); }
  .foundation-footer > span { text-align:center; color:var(--color-text-muted); font-size:11px; }
  .foundation-footer .primary { justify-self:end; min-width:140px; }
  .question { display:grid; gap:3px; margin-top:6px; color:var(--color-text); font-size:13px; }
  .question strong { font-weight:600; }
  .question small { color:var(--color-text-muted); font-size:11.5px; line-height:1.45; }
  .question-inline { color:var(--color-text); font-size:13px; font-weight:600; }
  .form-grid > label { color:var(--color-text); font-size:12.5px; }
  .form-grid > label small { color:var(--color-text-muted); }
  .chips { display:flex; flex-wrap:wrap; gap:7px; }
  .chip { display:inline-flex; align-items:center; gap:6px; padding:6px 11px; border:1px solid var(--color-border); border-radius:999px; background:var(--color-surface); color:var(--color-text); font-size:12px; cursor:pointer; }
  .chip input { position:absolute; opacity:0; width:1px; height:1px; }
  .chip.selected { border-color:var(--color-accent-text); background:color-mix(in srgb, var(--color-accent-text) 16%, var(--color-surface)); color:var(--color-accent-text); }
  .chip:focus-within { outline:2px solid var(--color-accent-text); outline-offset:2px; }
  .structure-option.unavailable { opacity:.6; cursor:not-allowed; }
  .card-title { margin:0 0 12px; color:var(--color-accent-text); font:600 15px/1.25 var(--font-display); }
  .entry-row { margin-top:10px; padding:11px; border:1px solid var(--color-border); border-radius:6px; background:var(--color-surface-sunken); }
  .entry-row > header { display:flex; align-items:center; justify-content:space-between; margin-bottom:9px; color:var(--color-text); font-size:12px; }
  .entry-row > header button { display:grid; place-items:center; width:26px; height:26px; border:0; background:transparent; color:var(--color-text-muted); cursor:pointer; }
  .footer-middle { display:flex; align-items:center; justify-content:center; gap:12px; }
  .link { display:inline-flex; align-items:center; gap:4px; padding:4px 6px; border:0; background:transparent; color:var(--color-accent-text); font-size:11px; cursor:pointer; }
  @media (max-width: 760px) {
    .foundation-body { grid-template-columns:1fr; }
    .steps { display:none; }
    .mode-cards,.structure-options,.form-grid,.review-grid { grid-template-columns:1fr; }
  }
</style>
