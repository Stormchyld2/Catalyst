<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { ArrowLeft, ArrowRight, FolderOpen, Loader2, Plus, Trash2 } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import { workspace } from "../stores/workspace.svelte";
  import type { Project } from "../types";
  import { open } from "../utils/nativeDialog";
  import DialogHeader from "./DialogHeader.svelte";
  import { modalFocus } from "../utils/modalFocus";

  type CharacterDraft = { name: string; role: string; aliases: string; description: string; template_id: string };
  type LocationDraft = { name: string; location_type: string; aliases: string; description: string; template_id: string };

  interface EntityTemplateOption {
    id: string;
    name: string;
    category: string;
    document_type: string;
    subtype: string | null;
    source: string;
  }

  interface EntityTemplateRegistry {
    templates: EntityTemplateOption[];
    issues: { relative_path: string; message: string }[];
    user_templates_dir: string;
  }

  interface StoryStructureTemplate {
    id: string;
    name: string;
    description?: string | null;
    source?: string | null;
    project_types: string[];
    structure: { title: string; children: { title: string }[] }[];
  }

  interface NewBookProject {
    manifest: {
      project_id: string;
      title: string;
      project_type: string;
      created_at: string;
    };
    paths: { root: string };
  }

  let {
    onClose,
    onComplete,
  }: {
    onClose: () => void;
    onComplete?: (project: Project) => void;
  } = $props();

  const steps = [
    "Project Basics",
    "Characters",
    "Locations",
    "Worldbuilding",
    "Style & Voice",
    "Themes & Core Ideas",
    "Brainstorming",
    "Synopsis",
    "Manuscript Structure",
    "Review & Create",
  ] as const;

  let step = $state(0);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let templateRegistry = $state<EntityTemplateRegistry>({ templates: [], issues: [], user_templates_dir: "" });
  let templatesLoading = $state(false);
  let templateLoadError = $state<string | null>(null);
  let manuscriptTemplates = $state<StoryStructureTemplate[]>([]);
  let manuscriptTemplatesLoading = $state(false);
  let manuscriptTemplateLoadError = $state<string | null>(null);
  let manuscriptTemplateId = $state("decide-later");

  let title = $state("");
  let projectRoot = $state("");
  let primaryGenre = $state("");
  let subgenres = $state("");
  let targetAudience = $state("");
  let targetLength = $state("");
  let series = $state("");

  let characters = $state<CharacterDraft[]>([]);
  let locations = $state<LocationDraft[]>([]);

  let setting = $state("");
  let timePeriod = $state("");
  let organizations = $state("");
  let factions = $state("");
  let systems = $state("");
  let worldNotes = $state("");
  let worldbuildingTemplateId = $state("worldbuilding-profile");

  let pov = $state("");
  let tense = $state("");
  let tone = $state("");
  let styleNotes = $state("");
  let voiceNotes = $state("");
  let influences = $state("");

  let themes = $state("");
  let centralConflict = $state("");
  let stakes = $state("");
  let storyIntent = $state("");
  let motifs = $state("");

  let brainstorming = $state("");
  let synopsis = $state("");

  const isFirst = $derived(step === 0);
  const isLast = $derived(step === steps.length - 1);
  const characterTemplates = $derived(templateRegistry.templates.filter((template) => template.category === "character"));
  const locationTemplates = $derived(templateRegistry.templates.filter((template) => template.category === "location"));
  const worldbuildingTemplates = $derived(templateRegistry.templates.filter((template) => template.category === "worldbuilding"));

  onMount(async () => {
    templatesLoading = true;
    templateLoadError = null;
    try {
      templateRegistry = await invoke<EntityTemplateRegistry>("get_entity_template_registry");
    } catch (e) {
      templateLoadError = e instanceof Error ? e.message : String(e);
    } finally {
      templatesLoading = false;
    }

    manuscriptTemplatesLoading = true;
    manuscriptTemplateLoadError = null;
    try {
      manuscriptTemplates = (await invoke<StoryStructureTemplate[]>("get_bundled_templates"))
        .filter((template) => template.project_types.includes("novel"));
    } catch (e) {
      manuscriptTemplateLoadError = e instanceof Error ? e.message : String(e);
    } finally {
      manuscriptTemplatesLoading = false;
    }
  });

  function csv(value: string): string[] {
    return value.split(",").map((part) => part.trim()).filter(Boolean);
  }

  async function chooseProjectRoot() {
    error = null;
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Choose an empty folder for this Catalyst book",
    });
    if (typeof selected === "string") projectRoot = selected;
  }

  function addCharacter() {
    characters = [...characters, { name: "", role: "", aliases: "", description: "", template_id: "character-profile" }];
  }

  function removeCharacter(index: number) {
    characters = characters.filter((_, current) => current !== index);
  }

  function addLocation() {
    locations = [...locations, { name: "", location_type: "", aliases: "", description: "", template_id: "location-profile" }];
  }

  function removeLocation(index: number) {
    locations = locations.filter((_, current) => current !== index);
  }

  function validateCurrentStep(): boolean {
    error = null;
    if (step === 0) {
      if (!title.trim()) {
        error = "Enter a book title.";
        return false;
      }
      if (!projectRoot.trim()) {
        error = "Choose an empty local folder for the book.";
        return false;
      }
      if (targetLength.trim()) {
        const value = Number(targetLength);
        if (!Number.isInteger(value) || value <= 0) {
          error = "Target word count must be a positive whole number.";
          return false;
        }
      }
    }
    if (step === 1 && characters.some((character) => !character.name.trim())) {
      error = "Each character row needs a name. Remove unused rows or enter a name.";
      return false;
    }
    if (step === 3 && ![setting, timePeriod, organizations, factions, systems, worldNotes].some((value) => value.trim())) {
      error = "Enter at least one worldbuilding detail before continuing.";
      return false;
    }
    if (step === 2 && locations.some((location) => !location.name.trim())) {
      error = "Each location row needs a name. Remove unused rows or enter a name.";
      return false;
    }
    return true;
  }

  function next() {
    if (!validateCurrentStep()) return;
    if (!isLast) step += 1;
  }

  function back() {
    error = null;
    if (!isFirst) step -= 1;
  }

  function toProject(created: NewBookProject): Project {
    return {
      id: created.manifest.project_id,
      name: created.manifest.title,
      source_type: "Blank",
      source_path: null,
      project_root: created.paths.root,
      created_at: created.manifest.created_at,
      modified_at: created.manifest.created_at,
      author_pen_name: null,
      genre: primaryGenre.trim() || null,
      description: synopsis.trim() || null,
      word_target: targetLength.trim() ? Number(targetLength) : null,
      reference_types: [],
      project_type: "novel",
      target_page_count: null,
    };
  }

  async function createBook() {
    if (!validateCurrentStep() || saving) return;
    const canReplace = window.dispatchEvent(
      new CustomEvent("lithomark:before-project-change", { cancelable: true })
    );
    if (!canReplace) return;

    saving = true;
    error = null;
    try {
      const input = {
        project_root: projectRoot.trim(),
        title: title.trim(),
        project_type: "novel",
        primary_genre: primaryGenre.trim(),
        subgenres: csv(subgenres),
        target_audience: targetAudience.trim(),
        target_length: targetLength.trim() ? Number(targetLength) : null,
        series: series.trim(),
        characters: characters.map((character) => ({
          name: character.name.trim(),
          role: character.role.trim(),
          aliases: csv(character.aliases),
          description: character.description.trim(),
          template_id: character.template_id,
        })),
        locations: locations.map((location) => ({
          name: location.name.trim(),
          location_type: location.location_type.trim(),
          aliases: csv(location.aliases),
          description: location.description.trim(),
          template_id: location.template_id,
        })),
        worldbuilding: {
          template_id: worldbuildingTemplateId,
          setting: setting.trim(),
          time_period: timePeriod.trim(),
          organizations: csv(organizations),
          factions: csv(factions),
          systems: systems.trim(),
          notes: worldNotes.trim(),
        },
        style_voice: {
          pov: pov.trim(),
          tense: tense.trim(),
          tone: tone.trim(),
          style_notes: styleNotes.trim(),
          voice_notes: voiceNotes.trim(),
          influences: influences.trim(),
        },
        themes: {
          themes: csv(themes),
          central_conflict: centralConflict.trim(),
          stakes: stakes.trim(),
          story_intent: storyIntent.trim(),
          motifs: csv(motifs),
        },
        brainstorming,
        synopsis,
        manuscript_template_id: manuscriptTemplateId,
      };

      const created = await invoke<NewBookProject>("create_new_book_foundation", { input });
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
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter" && !saving) {
      event.preventDefault();
      if (isLast) void createBook();
      else next();
    }
  }
</script>

<div
  class="dialog-scrim"
  use:modalFocus={{ onEscape: onClose, onKeydown: handleKeydown }}
  role="dialog"
  aria-modal="true"
  aria-labelledby="new-book-foundation-title"
  tabindex="-1"
>
  <div class="foundation-shell">
    <DialogHeader
      title="New Book Foundation"
      titleId="new-book-foundation-title"
      {onClose}
      closeLabel="Close"
    />

    <div class="foundation-body">
      <nav class="steps" aria-label="New Book Foundation steps">
        {#each steps as label, index}
          <button
            type="button"
            class:active={step === index}
            class:complete={index < step}
            disabled={saving || index > step}
            onclick={() => {
              error = null;
              step = index;
            }}
          >
            <span>{index + 1}</span>
            <strong>{label}</strong>
          </button>
        {/each}
      </nav>

      <main class="step-content">
        <header class="step-header">
          <p>Step {step + 1} of {steps.length}</p>
          <h2>{steps[step]}</h2>
        </header>

        {#if step === 0}
          <section class="form-grid">
            <label>Book title<input aria-label="Book title" bind:value={title} placeholder="Untitled book" /></label>
            <label class="wide">
              Project folder
              <div class="folder-picker">
                <input aria-label="Project folder" bind:value={projectRoot} placeholder="Choose an empty local folder…" />
                <button type="button" onclick={chooseProjectRoot}><FolderOpen size={15} aria-hidden="true" />Browse</button>
              </div>
              <small>Files become authoritative here when you press Create Book.</small>
            </label>
            <label>Primary genre<input bind:value={primaryGenre} /></label>
            <label>Subgenres<input bind:value={subgenres} placeholder="Comma separated" /></label>
            <label>Target audience<input bind:value={targetAudience} /></label>
            <label>Target word count<input aria-label="Target word count" type="text" inputmode="numeric" bind:value={targetLength} placeholder="90000" /></label>
            <label class="wide">Series<input bind:value={series} placeholder="Optional" /></label>
          </section>
        {:else if step === 1}
          <section class="entity-step">
            <div class="step-intro">
              <p>Add only the characters you already know. You can skip this and build profiles later.</p>
              <button class="secondary" type="button" onclick={addCharacter}><Plus size={14} aria-hidden="true" />Add character</button>
            </div>
            {#if characters.length === 0}<div class="empty-card">No characters added.</div>{/if}
            {#each characters as character, index}
              <article class="entity-card">
                <header><strong>Character {index + 1}</strong><button type="button" aria-label={`Remove character ${index + 1}`} onclick={() => removeCharacter(index)}><Trash2 size={14} /></button></header>
                <div class="form-grid">
                  <label>Name<input aria-label={`Character ${index + 1} name`} bind:value={character.name} /></label>
                  <label>Role<input bind:value={character.role} /></label>
                  <label class="wide">Aliases<input bind:value={character.aliases} placeholder="Comma separated" /></label>
                  <label class="wide">Description<textarea bind:value={character.description} rows="4"></textarea></label>
                </div>
              </article>
            {/each}
          </section>
        {:else if step === 2}
          <section class="entity-step">
            <div class="step-intro">
              <p>Add known locations now or skip this step and create them later.</p>
              <button class="secondary" type="button" onclick={addLocation}><Plus size={14} aria-hidden="true" />Add location</button>
            </div>
            {#if locations.length === 0}<div class="empty-card">No locations added.</div>{/if}
            {#each locations as location, index}
              <article class="entity-card">
                <header><strong>Location {index + 1}</strong><button type="button" aria-label={`Remove location ${index + 1}`} onclick={() => removeLocation(index)}><Trash2 size={14} /></button></header>
                <div class="form-grid">
                  <label>Name<input aria-label={`Location ${index + 1} name`} bind:value={location.name} /></label>
                  <label>Type<input bind:value={location.location_type} /></label>
                  <label class="wide">Aliases<input bind:value={location.aliases} placeholder="Comma separated" /></label>
                  <label class="wide">Description<textarea bind:value={location.description} rows="4"></textarea></label>
                </div>
              </article>
            {/each}
          </section>
        {:else if step === 3}
          <section class="form-grid">
            <p class="wide">Add at least one worldbuilding detail to establish your book's world. You can expand it later.</p>
            <label>Setting<input bind:value={setting} /></label>
            <label>Time period<input bind:value={timePeriod} /></label>
            <label class="wide">Organizations<input bind:value={organizations} placeholder="Comma separated" /></label>
            <label class="wide">Factions<input bind:value={factions} placeholder="Comma separated" /></label>
            <label class="wide">Systems / rules<textarea bind:value={systems} rows="5"></textarea></label>
            <label class="wide">Worldbuilding notes<textarea bind:value={worldNotes} rows="8"></textarea></label>
          </section>
        {:else if step === 4}
          <section class="form-grid">
            <label>POV<input bind:value={pov} placeholder="Third person limited" /></label>
            <label>Tense<input bind:value={tense} placeholder="Past" /></label>
            <label class="wide">Tone<input bind:value={tone} /></label>
            <label class="wide">Style notes<textarea bind:value={styleNotes} rows="5"></textarea></label>
            <label class="wide">Voice notes<textarea bind:value={voiceNotes} rows="5"></textarea></label>
            <label class="wide">Influences<textarea bind:value={influences} rows="4"></textarea></label>
          </section>
        {:else if step === 5}
          <section class="form-grid">
            <label class="wide">Themes<input bind:value={themes} placeholder="Comma separated" /></label>
            <label class="wide">Central conflict<textarea bind:value={centralConflict} rows="4"></textarea></label>
            <label class="wide">Stakes<textarea bind:value={stakes} rows="4"></textarea></label>
            <label class="wide">Story intent<textarea bind:value={storyIntent} rows="4"></textarea></label>
            <label class="wide">Motifs<input bind:value={motifs} placeholder="Comma separated" /></label>
          </section>
        {:else if step === 6}
          <section class="long-form">
            <p>Capture fragments, questions, possibilities, and rough ideas. This can stay messy.</p>
            <textarea aria-label="Brainstorming" bind:value={brainstorming} rows="22"></textarea>
          </section>
        {:else if step === 7}
          <section class="long-form">
            <p>Write the current version of what the story is. It does not need to be final.</p>
            <textarea aria-label="Synopsis" bind:value={synopsis} rows="22"></textarea>
          </section>
        {:else if step === 8}
          <section class="manuscript-structure-step">
            <div class="step-intro">
              <p>Choose how Catalyst should organize the Manuscript folder. You can also leave it undecided and set it up later from the Write workspace.</p>
            </div>

            <div class="structure-options" role="radiogroup" aria-label="Manuscript structure">
              <label class="structure-option" class:selected={manuscriptTemplateId === "decide-later"}>
                <input type="radio" name="manuscript-structure" value="decide-later" bind:group={manuscriptTemplateId} />
                <span>
                  <strong>Decide Later</strong>
                  <small>Create only the Manuscript folder. No internal manuscript structure is assumed.</small>
                </span>
              </label>

              {#if manuscriptTemplatesLoading}
                <div class="empty-card">Loading manuscript structures…</div>
              {:else if manuscriptTemplateLoadError}
                <p class="foundation-error" role="alert">Could not load manuscript structures: {manuscriptTemplateLoadError}</p>
              {:else}
                {#each manuscriptTemplates as template (template.id)}
                  <label class="structure-option" class:selected={manuscriptTemplateId === template.id}>
                    <input type="radio" name="manuscript-structure" value={template.id} bind:group={manuscriptTemplateId} />
                    <span>
                      <strong>{template.name}</strong>
                      {#if template.description}<small>{template.description}</small>{/if}
                      <small>{template.structure.map((part) => part.title).join(" · ")}</small>
                    </span>
                  </label>
                {/each}
              {/if}
            </div>
          </section>
        {:else}
          <section class="review">
            <div class="review-hero">
              <h3>{title || "Untitled book"}</h3>
              <p>{primaryGenre || "Genre not specified"}{targetLength ? ` · ${Number(targetLength).toLocaleString()} words` : ""}</p>
              <span>{projectRoot}</span>
            </div>
            <div class="review-grid">
              <article><strong>Characters</strong><span>{characters.length}</span></article>
              <article><strong>Locations</strong><span>{locations.length}</span></article>
              <article><strong>Worldbuilding</strong><span>{setting || timePeriod || organizations || factions || systems || worldNotes ? "Included" : "Required"}</span></article>
              <article><strong>Style & Voice</strong><span>{pov || tense || tone ? "Included" : "Skipped"}</span></article>
              <article><strong>Themes</strong><span>{themes || centralConflict ? "Included" : "Skipped"}</span></article>
              <article><strong>Brainstorming</strong><span>{brainstorming.trim() ? "Included" : "Skipped"}</span></article>
              <article><strong>Synopsis</strong><span>{synopsis.trim() ? "Included" : "Skipped"}</span></article>
              <article><strong>Manuscript Structure</strong><span>{manuscriptTemplateId === "decide-later" ? "Decide Later" : manuscriptTemplates.find((template) => template.id === manuscriptTemplateId)?.name ?? "Selected"}</span></article>
            </div>

            <section class="template-review" aria-labelledby="template-review-heading">
              <div class="template-review__heading">
                <div>
                  <h3 id="template-review-heading">Entity templates</h3>
                  <p>Review Catalyst's proposed template for each file. Change any assignment before the book is created.</p>
                </div>
                {#if templatesLoading}<span>Loading templates…</span>{/if}
              </div>

              {#if templateLoadError}
                <p class="foundation-error" role="alert">Could not load entity templates: {templateLoadError}</p>
              {:else}
                {#if characters.length}
                  <div class="template-group">
                    <h4>Characters</h4>
                    {#each characters as character, index}
                      <label class="template-assignment">
                        <span><strong>{character.name}</strong><small>Character</small></span>
                        <select aria-label={`${character.name} template`} bind:value={characters[index].template_id}>
                          {#each characterTemplates as template (template.id)}
                            <option value={template.id}>{template.name}{template.source === "user" ? " · User" : ""}</option>
                          {/each}
                        </select>
                      </label>
                    {/each}
                  </div>
                {/if}

                {#if locations.length}
                  <div class="template-group">
                    <h4>Locations</h4>
                    {#each locations as location, index}
                      <label class="template-assignment">
                        <span><strong>{location.name}</strong><small>Location</small></span>
                        <select aria-label={`${location.name} template`} bind:value={locations[index].template_id}>
                          {#each locationTemplates as template (template.id)}
                            <option value={template.id}>{template.name}{template.source === "user" ? " · User" : ""}</option>
                          {/each}
                        </select>
                      </label>
                    {/each}
                  </div>
                {/if}

                {#if setting || timePeriod || worldNotes || systems || organizations || factions}
                  <div class="template-group">
                    <h4>Worldbuilding</h4>
                    <label class="template-assignment">
                      <span><strong>Worldbuilding Overview</strong><small>Worldbuilding</small></span>
                      <select aria-label="Worldbuilding Overview template" bind:value={worldbuildingTemplateId}>
                        {#each worldbuildingTemplates as template (template.id)}
                          <option value={template.id}>{template.name}{template.source === "user" ? " · User" : ""}</option>
                        {/each}
                      </select>
                    </label>
                  </div>
                {/if}

                {#if !templatesLoading && !characters.length && !locations.length && !(setting || timePeriod || worldNotes || systems || organizations || factions)}
                  <p class="review-note">No template-backed entities will be created from this foundation.</p>
                {/if}
              {/if}
            </section>

            <p class="review-note">Create Book writes the normal Catalyst project files, preserves these template assignments in YAML, assigns stable IDs, then opens the project in Story Development. Setup data does not remain a second source of truth.</p>
          </section>
        {/if}

        {#if error}<p class="foundation-error" role="alert">{error}</p>{/if}
      </main>
    </div>

    <footer class="foundation-footer">
      <button type="button" class="secondary" onclick={back} disabled={isFirst || saving}>
        <ArrowLeft size={15} aria-hidden="true" />Back
      </button>
      <span>{steps[step]}</span>
      {#if isLast}
        <button type="button" class="primary" onclick={createBook} disabled={saving}>
          {#if saving}<Loader2 class="spin" size={15} aria-hidden="true" />Creating…{:else}Create Book{/if}
        </button>
      {:else}
        <button type="button" class="primary" onclick={next} disabled={saving}>
          Next<ArrowRight size={15} aria-hidden="true" />
        </button>
      {/if}
    </footer>
  </div>
</div>

<style>
  .foundation-shell { width:min(1180px,96vw); height:min(860px,94vh); display:flex; flex-direction:column; overflow:hidden; border:1px solid var(--color-border); border-radius:var(--radius-l); background:var(--color-bg); box-shadow:0 24px 70px rgb(0 0 0 / .45); }
  .foundation-body { flex:1; min-height:0; display:grid; grid-template-columns:250px minmax(0,1fr); }
  .steps { min-height:0; overflow:auto; padding:12px 8px; border-right:1px solid var(--color-border); background:var(--color-surface); }
  .steps button { width:100%; display:grid; grid-template-columns:26px minmax(0,1fr); align-items:center; gap:8px; padding:8px; border:0; border-radius:6px; background:transparent; color:var(--color-text-muted); text-align:left; cursor:pointer; }
  .steps button span { width:22px; height:22px; display:grid; place-items:center; border:1px solid var(--color-border); border-radius:50%; font-size:10px; }
  .steps button.active { background:var(--color-surface-raised); color:var(--color-accent-text); }
  .steps button.complete span { border-color:var(--color-accent-text); color:var(--color-accent-text); }
  .steps button:disabled { cursor:default; opacity:.55; }
  .step-content { min-height:0; overflow:auto; padding:28px 34px 40px; }
  .step-header { margin-bottom:22px; }
  .step-header p { margin:0 0 3px; color:var(--color-accent-text); font-size:10px; font-weight:600; letter-spacing:.08em; text-transform:uppercase; }
  .step-header h2 { margin:0; color:var(--color-text); font:600 27px/1.2 var(--font-display); }
  .form-grid { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:14px; }
  .form-grid label { display:grid; gap:5px; color:var(--color-text-muted); font-size:11px; font-weight:600; }
  .form-grid label.wide { grid-column:1 / -1; }
  .form-grid input,.form-grid textarea,.long-form textarea { width:100%; box-sizing:border-box; padding:9px 10px; border:1px solid var(--color-border); border-radius:6px; background:var(--color-surface-sunken); color:var(--color-text); font:13px/1.5 var(--font-ui); }
  .form-grid textarea,.long-form textarea { resize:vertical; }
  .form-grid small { font-weight:400; }
  .folder-picker { display:flex; gap:7px; }
  .folder-picker input { flex:1; }
  .folder-picker button,.secondary,.primary { display:inline-flex; align-items:center; justify-content:center; gap:6px; padding:8px 11px; border:1px solid var(--color-border); border-radius:6px; cursor:pointer; }
  .folder-picker button,.secondary { background:var(--color-surface-raised); color:var(--color-text); }
  .primary { background:var(--color-accent-text); color:var(--color-bg); }
  .entity-step { display:grid; gap:14px; }
  .step-intro { display:flex; align-items:center; justify-content:space-between; gap:16px; }
  .step-intro p,.long-form p,.review-note { color:var(--color-text-muted); line-height:1.55; }
  .entity-card,.empty-card,.review-hero,.review-grid article { border:1px solid var(--color-border); border-radius:7px; background:var(--color-surface); }
  .entity-card { padding:13px; }
  .entity-card > header { display:flex; align-items:center; justify-content:space-between; margin-bottom:12px; color:var(--color-text); }
  .entity-card > header button { display:grid; place-items:center; width:28px; height:28px; border:0; background:transparent; color:var(--color-text-muted); cursor:pointer; }
  .empty-card { padding:18px; color:var(--color-text-muted); }
  .long-form textarea { min-height:430px; font-family:ui-monospace,SFMono-Regular,Consolas,monospace; }
  .manuscript-structure-step { display:grid; gap:16px; }
  .structure-options { display:grid; gap:10px; }
  .structure-option { display:grid; grid-template-columns:auto 1fr; align-items:start; gap:11px; padding:14px; border:1px solid var(--color-border); border-radius:7px; background:var(--color-surface); color:var(--color-text); cursor:pointer; }
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
  .template-review { display:grid; gap:12px; padding:16px; border:1px solid var(--color-border); border-radius:7px; background:var(--color-surface); }
  .template-review__heading { display:flex; align-items:flex-start; justify-content:space-between; gap:16px; }
  .template-review__heading h3 { margin:0; color:var(--color-text); font:600 17px/1.25 var(--font-display); }
  .template-review__heading p { margin:5px 0 0; color:var(--color-text-muted); font-size:11px; line-height:1.5; }
  .template-review__heading > span { flex:none; color:var(--color-text-muted); font-size:11px; }
  .template-group { display:grid; gap:7px; padding-top:10px; border-top:1px solid var(--color-border); }
  .template-group h4 { margin:0 0 2px; color:var(--color-text-muted); font-size:10px; letter-spacing:.08em; text-transform:uppercase; }
  .template-assignment { display:grid; grid-template-columns:minmax(0,1fr) minmax(220px,.9fr); align-items:center; gap:14px; padding:8px 0; color:var(--color-text); }
  .template-assignment > span { display:grid; gap:2px; min-width:0; }
  .template-assignment small { color:var(--color-text-muted); font-size:10px; font-weight:400; }
  .template-assignment select { width:100%; box-sizing:border-box; padding:8px 9px; border:1px solid var(--color-border); border-radius:6px; background:var(--color-surface-sunken); color:var(--color-text); font:12px/1.4 var(--font-ui); }
  .foundation-error { margin:16px 0 0; padding:9px 10px; border:1px solid var(--color-error); border-radius:6px; color:var(--color-error); }
  .foundation-footer { min-height:58px; display:grid; grid-template-columns:140px 1fr 140px; align-items:center; gap:10px; padding:8px 14px; border-top:1px solid var(--color-border); background:var(--color-surface); }
  .foundation-footer > span { text-align:center; color:var(--color-text-muted); font-size:11px; }
  .foundation-footer .primary { justify-self:end; min-width:120px; }
</style>
