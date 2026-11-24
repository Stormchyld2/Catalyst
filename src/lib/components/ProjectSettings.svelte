<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { writing } from "../stores/writing.svelte";
  import type { WritingStats } from "../types";
  import { REFERENCE_FIELD_TYPES, REFERENCE_TYPE_OPTIONS } from "../referenceTypes";
  import { invoke } from "@tauri-apps/api/core";
  import { Loader2 } from "lucide-svelte";
  import type { Project } from "../types";
  import { normalizeReferenceTypes, DEFAULT_REFERENCE_TYPES } from "../referenceTypes";
  import FieldDefinitionManager from "./FieldDefinitionManager.svelte";
  import TagManager from "./TagManager.svelte";

  let {
    project,
    section,
    onSave,
    dirty = $bindable(false),
    busy = $bindable(false),
  }: {
    project: Project;
    section: string;
    onSave: (project: Project) => void;
    dirty?: boolean;
    busy?: boolean;
  } = $props();
  // The parent keys this form by project id; drafts survive section navigation.
  const initial = untrack(() => project);
  const isCampaign = initial.project_type === "campaign";
  let baseline = $state("");
  let saved = $state(false);
  let referenceTypes = $state(
    normalizeReferenceTypes(initial.reference_types ?? DEFAULT_REFERENCE_TYPES)
  );
  // Form fields initialized from current project
  let authorPenName = $state(initial.author_pen_name ?? "");
  let genre = $state(initial.genre ?? "");
  let description = $state(initial.description ?? "");
  let wordTarget = $state(
    initial.word_target !== null && initial.word_target !== undefined
      ? String(initial.word_target)
      : ""
  );
  const projectId = initial.id;
  // Campaign configuration: title in project.yaml, the rest in campaign.yaml.
  type CampaignSettings = {
    title: string; tone: string; gm_name: string; ruleset_name: string; edition: string;
    starting_level: string; advancement: string; play_mode: string; schedule: string;
    session_length: string; party_name: string; expected_players: string;
  };
  let campaign = $state<CampaignSettings | null>(null);
  let campaignBaseline = $state("");
  let campaignLoadError = $state<string | null>(null);
  baseline = untrack(() =>
    JSON.stringify([authorPenName, genre, description, wordTarget, referenceTypes])
  );
  let dailyGoal = $state<number | undefined>(undefined);
  let goalLoaded = $state(false);
  onMount(async () => {
    if (isCampaign) {
      if (!initial.project_root) return;
      try {
        campaign = await invoke<CampaignSettings>("get_campaign_settings", { projectRoot: initial.project_root });
        campaignBaseline = JSON.stringify(campaign);
      } catch (e) {
        campaignLoadError = `Could not load campaign settings: ${String(e)}`;
      }
      return;
    }
    try {
      const stats = await invoke<WritingStats>("get_writing_stats", { projectId });
      dailyGoal = stats.daily_goal;
      savedGoal = dailyGoal;
      goalLoaded = true;
    } catch (e) {
      error = `Could not load daily goal: ${String(e)}`;
    }
  });
  let saving = $state(false);
  let error = $state<string | null>(null);

  const snapshot = $derived(
    JSON.stringify([authorPenName, genre, description, wordTarget, referenceTypes])
  );
  let tagState = $state({ dirty: false, busy: false });
  let fieldStates = $state<Record<string, { dirty: boolean; busy: boolean }>>({});
  let tagsVisited = $state(false);
  let fieldsVisited = $state(false);
  $effect(() => {
    if (section === "tags") tagsVisited = true;
    if (section === "fields") fieldsVisited = true;
  });
  let savedGoal = $state<number | undefined>(undefined);
  $effect(() => {
    dirty =
      snapshot !== baseline ||
      (campaign !== null && JSON.stringify(campaign) !== campaignBaseline) ||
      (goalLoaded && dailyGoal !== savedGoal) ||
      tagState.dirty ||
      Object.values(fieldStates).some((state) => state.dirty);
    busy = saving || tagState.busy || Object.values(fieldStates).some((state) => state.busy);
  });

  async function handleSave() {
    if (saving) return;

    saving = true;
    saved = false;
    error = null;

    try {
      const parsedWordTarget = isCampaign
        ? project.word_target
        : String(wordTarget ?? "").trim().length
          ? Number(String(wordTarget ?? "").trim())
          : null;
      if (
        parsedWordTarget !== null &&
        (!Number.isSafeInteger(parsedWordTarget) || parsedWordTarget < 0)
      ) {
        throw new Error("Word target must be a non-negative whole number");
      }

      if (
        goalLoaded &&
        (dailyGoal === undefined ||
          !Number.isInteger(dailyGoal) ||
          dailyGoal < 0 ||
          dailyGoal > 1000000)
      ) {
        throw new Error("Daily goal must be a whole number between 0 and 1,000,000");
      }

      const disabledDraft = REFERENCE_TYPE_OPTIONS.find(
        (option) => fieldStates[option.id]?.dirty && !referenceTypes.includes(option.id)
      );
      if (disabledDraft) {
        throw new Error(
          `Save or cancel the ${disabledDraft.label} custom field draft before disabling that reference type.`
        );
      }

      // Convert empty strings to null for optional fields
      const settings = {
        author_pen_name: isCampaign ? project.author_pen_name : authorPenName.trim() || null,
        genre: genre.trim() || null,
        description: description.trim() || null,
        word_target: isCampaign ? project.word_target : parsedWordTarget,
        reference_types: referenceTypes,
        // This command replaces nullable metadata; preserve the screenplay target.
        target_page_count: project.target_page_count,
        ...(!isCampaign && goalLoaded ? { daily_writing_goal: dailyGoal } : {}),
      };

      if (isCampaign && campaign && initial.project_root && JSON.stringify(campaign) !== campaignBaseline) {
        if (!campaign.title.trim()) throw new Error("Campaign name cannot be empty.");
        campaign = await invoke<CampaignSettings>("update_campaign_settings", {
          projectRoot: initial.project_root,
          settings: $state.snapshot(campaign),
        });
        campaignBaseline = JSON.stringify(campaign);
      }

      const updatedProject = await invoke<Project>("update_project_settings", {
        projectId,
        settings,
      });

      if (!isCampaign) void writing.refresh(projectId);
      baseline = snapshot;
      savedGoal = dailyGoal;
      saved = true;
      onSave(updatedProject);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }
</script>

<div class="settings-pane" aria-busy={busy}>
  <p class="settings-lede">
    Changes here apply only to <strong>{project.name}</strong>.
    {#if section === "tags"}
      Tag changes save as you make them.
    {:else if section === "fields"}
      Custom field changes save as you make them. Fields are listed for saved, enabled reference
      types.
    {/if}
  </p>
  <div hidden={section !== "details"} class="settings-fields">
    {#if !isCampaign}
    <!-- Pen Name -->
    <div class="ka-field od-field">
      <label for="author-pen-name">
        Pen Name <span class="ka-optional">(optional)</span>
      </label>
      <input
        id="author-pen-name"
        type="text"
        bind:value={authorPenName}
        placeholder="Leave blank to use your author name"
        disabled={saving}
      />
      <p class="ka-help">
        If provided, this will be used as the byline on title pages instead of your author name.
      </p>
    </div>

    {/if}
    {#if isCampaign}
      {#if campaignLoadError}<p role="alert" class="ka-error">{campaignLoadError}</p>{/if}
      {#if campaign}
        <div class="ka-field od-field">
          <label for="campaign-title">Campaign name</label>
          <input id="campaign-title" type="text" bind:value={campaign.title} disabled={saving} />
        </div>
      {/if}
    {/if}
    <!-- Genre -->
    <div class="ka-field od-field">
      <label for="genre">
        {isCampaign ? "Campaign Genre" : "Genre"} <span class="ka-optional">(optional)</span>
      </label>
      <input
        id="genre"
        type="text"
        bind:value={genre}
        placeholder={isCampaign ? "e.g., Fantasy, Horror, Science Fiction" : "e.g., Literary Fiction, Science Fiction, Mystery"}
        disabled={saving}
      />
      <p class="ka-help">{isCampaign ? "Shown on the Campaign Dashboard." : "Genre will be displayed on manuscript title pages."}</p>
    </div>

    {#if isCampaign && campaign}
      <div class="ka-field od-field">
        <label for="campaign-tone">Tone <span class="ka-optional">(optional)</span></label>
        <input id="campaign-tone" type="text" bind:value={campaign.tone} placeholder="e.g., Grim, heroic, mysterious" disabled={saving} />
      </div>
      <div class="ka-field od-field">
        <label for="campaign-gm">Game Master <span class="ka-optional">(optional)</span></label>
        <input id="campaign-gm" type="text" bind:value={campaign.gm_name} disabled={saving} />
      </div>
      <fieldset class="campaign-group">
        <legend>Ruleset</legend>
        <div class="ka-field od-field">
          <label for="campaign-ruleset">Game system</label>
          <input id="campaign-ruleset" type="text" bind:value={campaign.ruleset_name} placeholder="e.g., Dungeons & Dragons 5e" disabled={saving} />
        </div>
        <div class="ka-field od-field">
          <label for="campaign-edition">Edition</label>
          <input id="campaign-edition" type="text" bind:value={campaign.edition} disabled={saving} />
        </div>
        <div class="ka-field od-field">
          <label for="campaign-level">Starting level</label>
          <input id="campaign-level" type="text" bind:value={campaign.starting_level} disabled={saving} />
        </div>
        <div class="ka-field od-field">
          <label for="campaign-advancement">Advancement</label>
          <input id="campaign-advancement" type="text" bind:value={campaign.advancement} placeholder="e.g., Milestone, XP" disabled={saving} />
        </div>
      </fieldset>
      <fieldset class="campaign-group">
        <legend>Party</legend>
        <div class="ka-field od-field">
          <label for="campaign-party">Party name</label>
          <input id="campaign-party" type="text" bind:value={campaign.party_name} disabled={saving} />
        </div>
        <div class="ka-field od-field">
          <label for="campaign-players">Expected players</label>
          <input id="campaign-players" type="text" inputmode="numeric" bind:value={campaign.expected_players} disabled={saving} />
        </div>
      </fieldset>
      <fieldset class="campaign-group">
        <legend>Sessions</legend>
        <div class="ka-field od-field">
          <label for="campaign-play-mode">Play mode</label>
          <select id="campaign-play-mode" bind:value={campaign.play_mode} disabled={saving}>
            <option value="physical">In person</option>
            <option value="hybrid">Hybrid</option>
            <option value="digital">Online</option>
          </select>
        </div>
        <div class="ka-field od-field">
          <label for="campaign-schedule">Schedule</label>
          <input id="campaign-schedule" type="text" bind:value={campaign.schedule} placeholder="e.g., Weekly" disabled={saving} />
        </div>
        <div class="ka-field od-field">
          <label for="campaign-session-length">Session length</label>
          <input id="campaign-session-length" type="text" bind:value={campaign.session_length} placeholder="e.g., 3 hours" disabled={saving} />
        </div>
      </fieldset>
      <p class="ka-help">House rules are a campaign document: Rules &amp; Mechanics/House Rules in World Builder.</p>
    {/if}

    <!-- Description -->
    <div class="ka-field od-field">
      <label for="project-description">
        {isCampaign ? "Campaign Description" : "Project Description"} <span class="ka-optional">(optional)</span>
      </label>
      <textarea
        id="project-description"
        rows="4"
        bind:value={description}
        placeholder="Short summary or notes about this project"
        disabled={saving}
      ></textarea>
    </div>

    {#if !isCampaign}
    <div class="ka-field od-field">
      <label for="daily-writing-goal">Daily writing goal</label>
      <input
        id="daily-writing-goal"
        type="number"
        min="0"
        max="1000000"
        step="1"
        bind:value={dailyGoal}
        disabled={saving || !goalLoaded}
        aria-describedby="daily-goal-help"
      />
      <p id="daily-goal-help" class="ka-help">
        Net words added per day in this project. Set to 0 to turn off the goal. Changes apply today;
        earlier streak days keep their original goals.
      </p>
    </div>

    <!-- Word Target -->
    <div class="ka-field od-field">
      <label for="word-target">
        Word Target <span class="ka-optional">(optional)</span>
      </label>
      <input
        id="word-target"
        type="number"
        min="0"
        inputmode="numeric"
        bind:value={wordTarget}
        placeholder="e.g., 80000"
        disabled={saving}
      />
    </div>
    {/if}
  </div>
  <div hidden={section !== "references"} class="settings-fields">
    <p class="ka-help">
      Choose which reference types appear in this project’s References panel. Disabling a type keeps
      its existing entries.
    </p>
    {#each REFERENCE_TYPE_OPTIONS as option (option.id)}
      <label class="ka-check">
        <input type="checkbox" value={option.id} bind:group={referenceTypes} disabled={saving} />
        {option.label}
      </label>
    {/each}
  </div>
  <div hidden={section !== "tags"}>
    {#if tagsVisited}
      <TagManager
        projectId={project.id}
        onState={(state) => (tagState = state)}
        onChange={() => onSave({ ...project })}
      />
    {/if}
  </div>
  <div hidden={section !== "fields"}>
    {#if fieldsVisited}
      {#each normalizeReferenceTypes(project.reference_types ?? DEFAULT_REFERENCE_TYPES) as refType (refType)}
        {@const mapping = REFERENCE_TYPE_OPTIONS.find((option) => option.id === refType)}
        {#if mapping}<FieldDefinitionManager
            projectId={project.id}
            entityType={REFERENCE_FIELD_TYPES[refType]}
            entityLabel={mapping.label}
            onState={(state) => (fieldStates[refType] = state)}
            onChange={() => onSave({ ...project })}
          />{/if}
      {:else}<p class="ka-help">Enable reference types to configure custom fields.</p>{/each}
    {/if}
  </div>
  {#if section === "details" || section === "references"}
    {#if error}<p role="alert" class="ka-error">{error}</p>{/if}
    <div class="settings-actions">
      {#if saved && !dirty}<p role="status" class="ka-help">{isCampaign ? "Campaign changes saved." : "Project changes saved."}</p>{/if}
      <button
        type="button"
        onclick={handleSave}
        disabled={saving}
        aria-busy={saving || undefined}
        class="ka-button"
        >{#if saving}<Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
          Saving…{:else}{isCampaign ? "Save campaign changes" : "Save project changes"}{/if}</button
      >
    </div>
  {/if}
</div>

<style>
  .campaign-group { border: 1px solid var(--color-border); border-radius: var(--radius-m); padding: 12px 14px; display: grid; gap: 10px; }
  .campaign-group legend { padding: 0 6px; font-weight: 650; }
</style>
