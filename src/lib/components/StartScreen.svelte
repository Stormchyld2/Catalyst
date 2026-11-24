<script lang="ts">
  import catalystLogo from "../../assets/catalyst-logo.png";
  import { invoke } from "@tauri-apps/api/core";
  import type { ImportType } from "../utils/import";
  import { open as openNative } from "../utils/nativeDialog";
  import {
    BookOpen,
    FileText,
    FolderOpen,
    Loader2,
    Plus,
    TriangleAlert,
    Trash2,
  } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { ui } from "../stores/ui.svelte";
  import { workspace } from "../stores/workspace.svelte";
  import type { Project } from "../types";
  import ConfirmDialog from "./ConfirmDialog.svelte";

  interface Props {
    recentProjects: Project[];
    onImportLongform?: () => void;
    onImportComplete?: (project: Project, type: ImportType) => void;
    onOpenQuickStart?: () => void;
    onNewProject?: () => void;
    onOpenProjectFolder?: () => void;
    onOpenEditorial?: () => void;
    onProjectActivated?: () => void;
  }

  let {
    recentProjects = $bindable(),
    onNewProject,
    onOpenProjectFolder,
    onOpenEditorial,
    onProjectActivated,
  }: Props = $props();

  let deletingProjectId = $state<string | null>(null);
  let openingProjectId = $state<string | null>(null);
  let startingSample = $state(false);
  let openError = $state<{ name: string; message: string } | null>(null);
  let projectToDelete = $state<Project | null>(null);
  let showAllProjects = $state(false);
  let showImportFormats = $state(false);
  let allProjects = $state<Project[] | null>(null);
  interface ProjectDashboardMetrics {
    word_count: number;
    word_target: number | null;
    chapter_count: number;
    scene_count: number;
  }

  const EMPTY_DASHBOARD_METRICS: ProjectDashboardMetrics = {
    word_count: 0,
    word_target: 0,
    chapter_count: 0,
    scene_count: 0,
  };

  let dashboardMetrics = $state<ProjectDashboardMetrics>({ ...EMPTY_DASHBOARD_METRICS });
  let dashboardMetricsProjectId = $state<string | null>(null);
  let loadingDashboardMetrics = $state(false);
  let dashboardMetricsRequestId = 0;

  /** The start screen lists the latest few; "View all" appears only when there are more. */
  const RECENT_SHOWN = 3;
  const listedProjects = $derived(recentProjects.slice(0, RECENT_SHOWN));
  const hasMoreProjects = $derived(recentProjects.length > RECENT_SHOWN);
  const activeHomeProject = $derived(currentProject.value ?? recentProjects[0] ?? null);


  async function loadDashboardMetrics(project: Project) {
    if (dashboardMetricsProjectId === project.id && !loadingDashboardMetrics) return;
    const requestId = ++dashboardMetricsRequestId;

    dashboardMetricsProjectId = project.id;
    dashboardMetrics = { ...EMPTY_DASHBOARD_METRICS };
    loadingDashboardMetrics = true;

    try {
      const metrics = await invoke<ProjectDashboardMetrics>("get_project_dashboard_metrics", {
        projectId: project.id,
      });

      if (requestId !== dashboardMetricsRequestId) return;
      dashboardMetrics = {
        word_count: metrics.word_count ?? 0,
        word_target: metrics.word_target ?? 0,
        chapter_count: metrics.chapter_count ?? 0,
        scene_count: metrics.scene_count ?? 0,
      };
    } catch (e) {
      if (requestId !== dashboardMetricsRequestId) return;
      console.error("Failed to load dashboard metrics:", e);
      dashboardMetrics = { ...EMPTY_DASHBOARD_METRICS };
    } finally {
      if (requestId === dashboardMetricsRequestId) loadingDashboardMetrics = false;
    }
  }

  $effect(() => {
    const latest = activeHomeProject;

    if (!latest) {
      dashboardMetrics = { ...EMPTY_DASHBOARD_METRICS };
      dashboardMetricsProjectId = null;
      return;
    }

    void loadDashboardMetrics(latest);
  });

  const dashboardGoalProgress = $derived(
    dashboardMetrics.word_target && dashboardMetrics.word_target > 0
      ? Math.round((dashboardMetrics.word_count / dashboardMetrics.word_target) * 100)
      : 0
  );
  type RpgFormat = "markdown" | "obsidian" | "assets" | "foundry" | "roll20" | "fantasy_grounds" | "pdf";
  type RpgPreview = {
    format: RpgFormat;
    source: string;
    file_count: number;
    total_bytes: number;
    sample_files: string[];
    warnings: string[];
    behavior: string;
  };
  type RpgResult = { imported: number; documents_created: number; copied_assets: number; destination: string; notes: string[] };
  const RPG_IMPORTS: { type: RpgFormat; label: string; hint: string; directory: boolean; filters?: { name: string; extensions: string[] }[] }[] = [
    { type: "markdown", label: "Markdown", hint: "Single Markdown file", directory: false, filters: [{ name: "Markdown", extensions: ["md", "markdown"] }] },
    { type: "obsidian", label: "Obsidian Vault", hint: "Markdown vault folder", directory: true },
    { type: "assets", label: "Images / Maps / Tokens", hint: "Image folder (PNG, JPG, SVG, WebP, GIF)", directory: true },
    { type: "foundry", label: "Foundry VTT", hint: "Exported JSON data / folder", directory: false, filters: [{ name: "Foundry JSON", extensions: ["json"] }] },
    { type: "roll20", label: "Roll20", hint: "Supported JSON export (not a live campaign connection)", directory: false, filters: [{ name: "Roll20 JSON", extensions: ["json"] }] },
    { type: "fantasy_grounds", label: "Fantasy Grounds", hint: ".mod module or exported XML", directory: false, filters: [{ name: "Fantasy Grounds", extensions: ["mod", "xml"] }] },
    { type: "pdf", label: "PDF Reference", hint: "Original PDF preserved offline; no OCR", directory: false, filters: [{ name: "PDF", extensions: ["pdf"] }] },
  ];
  let rpgImportBusy = $state(false);
  let rpgImportPreview = $state<RpgPreview | null>(null);
  let rpgImportSource = $state("");
  let rpgImportDestination = $state("");
  let rpgImportError = $state("");
  let rpgImportResult = $state<RpgResult | null>(null);
  let selectedRpgFormat = $state<RpgFormat>("markdown");

  async function selectRpgImport(type: RpgFormat) {
    const config = RPG_IMPORTS.find((v) => v.type === type);
    if (!config) return;
    rpgImportError = "";
    rpgImportResult = null;
    rpgImportPreview = null;
    selectedRpgFormat = type;
    try {
      const source = await openNative({
        title: "Choose " + config.label + " source",
        directory: config.directory,
        multiple: false,
        ...(config.filters ? { filters: config.filters } : {}),
      });
      if (!source) return;
      rpgImportSource = source;
      const campaignRoot = currentProject.value?.project_type === "campaign"
        ? currentProject.value.project_root
        : null;
      if (campaignRoot) {
        rpgImportDestination = campaignRoot;
      } else {
        const folder = await openNative({ title: "Choose an existing Catalyst campaign folder", directory: true, multiple: false });
        if (!folder) return;
        rpgImportDestination = folder;
      }
      rpgImportBusy = true;
      rpgImportPreview = await invoke<RpgPreview>("preview_rpg_content_import", { sourcePath: source, format: type });
    } catch (e) {
      rpgImportError = String(e);
    } finally {
      rpgImportBusy = false;
    }
  }

  async function chooseRpgDestination() {
    const folder = await openNative({ title: "Choose an existing Catalyst campaign", directory: true, multiple: false });
    if (folder) rpgImportDestination = folder;
  }

  async function applyRpgImport() {
    if (!rpgImportPreview || !rpgImportDestination || rpgImportBusy) return;
    rpgImportBusy = true;
    rpgImportError = "";
    try {
      rpgImportResult = await invoke<RpgResult>("apply_rpg_content_import", {
        sourcePath: rpgImportSource, destinationRoot: rpgImportDestination,
        format: selectedRpgFormat,
      });
      rpgImportPreview = null;
    } catch (e) {
      rpgImportError = String(e);
    } finally {
      rpgImportBusy = false;
    }
  }

  async function trySampleProject() {
    startingSample = true;
    ui.startImport("Opening D&D 5e adventure demo", "Opening the adventure campaign...");
    try {
      const project = await invoke<Project>("create_sample_project");
      currentProject.setProject(project);
      ui.setView("editor");
      workspace.enterProject("home");
      onProjectActivated?.();
    } catch (e) {
      console.error("Failed to open The Hollow Crown adventure demo:", e);
      ui.showError(`Failed to open The Hollow Crown adventure demo: ${e}`);
    } finally {
      ui.finishImport();
      startingSample = false;
    }
  }

  async function openProject(project: Project, destination: "dashboard" | "write" = "dashboard") {
    if (openingProjectId) return;
    openingProjectId = project.id;
    openError = null;
    try {
      const loaded = await invoke<Project>("get_project", { id: project.id });
      currentProject.setProject(loaded);
      ui.setView("editor");
      workspace.enterProject(destination === "dashboard" ? "home" : "write");
      onProjectActivated?.();
      // Sidebar $effect will run loadChapters
    } catch (e) {
      console.error("Failed to open project:", e);
      openError = { name: project.name, message: String(e) };
    } finally {
      openingProjectId = null;
    }
  }

  // The importer id reads as plain metadata, spelled as each tool spells itself.
  function modifiedDate(value: string) {
    const date = new Date(value);
    const numeric = date.toLocaleDateString("en-US", {
      month: "2-digit",
      day: "2-digit",
      year: "numeric",
    });

    return {
      short: numeric,
      long: numeric,
    };
  }
  let loadingProjectList = $state(false);

  async function openProjectLibrary() {
    if (loadingProjectList) return;
    openError = null;
    loadingProjectList = true;
    try {
      allProjects = await invoke<Project[]>("get_all_projects");
      showAllProjects = true;
    } catch (e) {
      console.error("Failed to load projects:", e);
      ui.showError(`Failed to load projects: ${e}`);
    } finally {
      loadingProjectList = false;
    }
  }

  function closeProjectLibrary() {
    showAllProjects = false;
  }

  function showDeleteConfirmation(event: MouseEvent, project: Project) {
    event.stopPropagation();
    projectToDelete = project;
  }

  async function confirmDeleteProject() {
    if (!projectToDelete) return;

    const project = projectToDelete;
    projectToDelete = null;
    deletingProjectId = project.id;

    try {
      await invoke("delete_project", { projectId: project.id });
      recentProjects = recentProjects.filter((p) => p.id !== project.id);
      allProjects = allProjects?.filter((p) => p.id !== project.id) ?? null;
    } catch (e) {
      console.error("Failed to remove project:", e);
      ui.showError(`Failed to delete project: ${e}`);
    } finally {
      deletingProjectId = null;
    }
  }

  function cancelDeleteProject() {
    projectToDelete = null;
  }
</script>

<div class="start">
  <div class="start-body">
    <div class="start-intro">
      <h1 class="ka-sr">Catalyst</h1>

      <div class="signature">
        <figure class="brand">
          <img src={catalystLogo} alt="Catalyst" />
        </figure>
        <p class="tagline">Build the world. Run the table.</p>
      </div>

      <section class="start-panel" aria-labelledby="start-heading">
        <h2 id="start-heading" class="section-title">Start</h2>

        <div class="start-actions">
          {#if onNewProject}
            <button
              type="button"
              data-testid="new-project-button"
              onclick={onNewProject}
              class="ka-button start-action"
            >
              <Plus class="w-5 h-5" aria-hidden="true" />
              <span>New Campaign <small>Quick Start or Guided Setup, then open your campaign wiki</small></span>
            </button>

            {#if ui.showSampleProject}
            <button
              type="button"
              onclick={trySampleProject}
              disabled={startingSample}
              aria-busy={startingSample || undefined}
              class="ka-button ka-button--secondary start-action"
            >
              <BookOpen class="w-5 h-5" aria-hidden="true" />
              <span>
                {startingSample ? "Opening The Hollow Crown..." : "The Hollow Crown — D&D 5e Demo"}
                <small>Explore a playable level-1 D&D 5e adventure</small>
              </span>
            </button>
            {/if}
          {/if}

          {#if onOpenProjectFolder}
            <button type="button" onclick={onOpenProjectFolder} class="ka-button ka-button--secondary start-action">
              <FolderOpen class="w-5 h-5" aria-hidden="true" />
              <span>Open Project Folder <small>Open an existing Catalyst project folder</small></span>
            </button>
          {/if}

          <button
            type="button"
            class="ka-button ka-button--secondary start-action"
            aria-haspopup="dialog"
            onclick={() => (showImportFormats = true)}
          >
            <FolderOpen class="w-5 h-5" aria-hidden="true" />
            <span>Import RPG Content <small>Markdown, Obsidian, maps, VTT exports and PDFs</small></span>
          </button>

          {#if onOpenEditorial}
            <button
              type="button"
              onclick={onOpenEditorial}
              class="ka-button ka-button--secondary start-action"
            >
              <FileText class="w-5 h-5" aria-hidden="true" />
              <span>Import Review <small>Import reviewer feedback or a review package</small></span>
            </button>
          {/if}
        </div>
      </section>
    </div>

    <div class="start-main">
      {#if recentProjects.length > 0}
        <section class="dashboard-card" data-testid="recent-projects" aria-labelledby="recent-projects-heading">
          <div class="section-head">
            <h2 id="recent-projects-heading" class="section-title">
              Recent Projects
            </h2>
            {#if hasMoreProjects}
              <button
                type="button"
                class="ka-button ka-button--ghost"
                aria-haspopup="dialog"
                aria-busy={loadingProjectList || undefined}
                onclick={openProjectLibrary}
              >
                View All
              </button>
            {/if}
          </div>

          {#if openError}
            <div class="ka-notice ka-notice--error od-row-top open-error" role="alert">
              <TriangleAlert class="w-5 h-5 shrink-0" aria-hidden="true" />
              <div class="od-field od-fill">
                <strong>Could not open "{openError.name}"</strong>
                <p>{openError.message}</p>
              </div>
            </div>
          {/if}

          <ul class="recent-list">
            {#each listedProjects as project (project.id)}
              {@const modified = modifiedDate(project.modified_at)}
              <li class="recent-row" aria-busy={openingProjectId === project.id || undefined}>
                <button
                  type="button"
                  data-testid="project-card"
                  onclick={() => openProject(project)}
                  class="recent-open"
                >
                  <span class="recent-title">
                    <span class="recent-name">{project.name}</span>
                  </span>
                  {#if openingProjectId === project.id}
                    <span class="recent-date">Opening...</span>
                  {:else}
                    <time
                      class="recent-date"
                      datetime={project.modified_at}
                      aria-label={`Modified ${modified.long}`}>{modified.short}</time
                    >
                  {/if}
                </button>
                <button
                  type="button"
                  onclick={(e) => showDeleteConfirmation(e, project)}
                  disabled={deletingProjectId === project.id || openingProjectId === project.id}
                  class="ka-button ka-button--ghost ka-icon-button recent-delete"
                  class:is-busy={deletingProjectId === project.id}
                  aria-label={`Remove ${project.name} from Catalyst`}
                  title="Remove from Catalyst"
                >
                  {#if deletingProjectId === project.id}
                    <Loader2 class="w-5 h-5 animate-spin" aria-hidden="true" />
                  {:else}
                    <Trash2 class="w-5 h-5" aria-hidden="true" />
                  {/if}
                </button>
              </li>
            {/each}
          </ul>
        </section>

        {@const latestProject = activeHomeProject ?? recentProjects[0]}
        {@const latestModified = modifiedDate(latestProject.modified_at)}
        <section class="dashboard-card continue-card" aria-labelledby="continue-heading">
          <div class="continue-copy">
            <span class="dashboard-eyebrow">Continue Writing</span>
            <h2 id="continue-heading">{latestProject.name}</h2>
            <p>Last worked {latestModified.short}</p>
          </div>
          <button
            type="button"
            class="ka-button continue-button"
            onclick={() => openProject(latestProject, "write")}
            disabled={openingProjectId !== null}
          >
            {openingProjectId === latestProject.id ? "Opening..." : "Continue"}
          </button>
        </section>

        <section
          class="dashboard-card metrics-card"
          aria-labelledby="metrics-heading"
          aria-busy={loadingDashboardMetrics || undefined}
        >
          <div class="section-head">
            <h2 id="metrics-heading" class="section-title">Project Overview</h2>
          </div>

          <div class="metrics-grid">
            <div class="metric">
              <span class="metric-label">Words</span>
              <strong class="metric-value">{dashboardMetrics.word_count.toLocaleString()}</strong>
            </div>

            <div class="metric">
              <span class="metric-label">Goal</span>
              <strong class="metric-value">{(dashboardMetrics.word_target ?? 0).toLocaleString()}</strong>
              <span>{dashboardGoalProgress}% of goal</span>
            </div>

            <div class="metric">
              <span class="metric-label">Chapters</span>
              <strong class="metric-value">{dashboardMetrics.chapter_count.toLocaleString()}</strong>
            </div>

            <div class="metric">
              <span class="metric-label">Scenes</span>
              <strong class="metric-value">{dashboardMetrics.scene_count.toLocaleString()}</strong>
            </div>
          </div>
        </section>
      {:else}
        <section class="dashboard-card empty-dashboard" aria-label="Projects">
          <FileText class="w-7 h-7" aria-hidden="true" />
          <div>
            <h2 class="section-title">Your projects will appear here</h2>
            <p>Start a campaign, explore the demo adventure, or import existing tabletop material.</p>
          </div>
        </section>
      {/if}
    </div>
  </div>
</div>

{#if showImportFormats}
  <div class="import-backdrop" role="presentation" onclick={() => (showImportFormats = false)}>
    <div class="import-dialog" tabindex="-1" role="dialog" aria-modal="true"
      aria-labelledby="import-dialog-heading" onclick={(event) => event.stopPropagation()}
      onkeydown={(event) => event.stopPropagation()}>
      <header class="import-dialog__header">
        <div>
          <span class="dashboard-eyebrow">Offline campaign importer</span>
          <h2 id="import-dialog-heading">Import RPG Content</h2>
        </div>
        <button type="button" class="ka-button ka-button--ghost" onclick={() => (showImportFormats = false)}
          aria-label="Close import dialog">Close</button>
      </header>
      <div class="import-dialog__body">
        <p class="ka-help import-help">Select an export, preview the files, and choose an existing Catalyst campaign. Originals are preserved where appropriate. No imported content overwrites existing records.</p>
        {#if rpgImportError}
          <p role="alert" class="ka-help">{rpgImportError}</p>
        {/if}
        {#if rpgImportResult}
          <div role="status">
            <strong>Import complete</strong>
            <p>{rpgImportResult.imported} source file(s); {rpgImportResult.documents_created} notes and {rpgImportResult.copied_assets} original assets.</p>
            <p>Destination: {rpgImportResult.destination}</p>
            {#each rpgImportResult.notes as note}<p>{note}</p>{/each}
          </div>
          <button type="button" class="ka-button" onclick={() => { rpgImportResult = null; rpgImportPreview = null; }}>Import more</button>
        {:else if rpgImportPreview}
          <div class="od-stack">
            <p><strong>{rpgImportPreview.file_count} file(s)</strong> · {Math.round(rpgImportPreview.total_bytes / 1024)} KB</p>
            <p>{rpgImportPreview.behavior}</p>
            {#each rpgImportPreview.warnings as warning}<p class="ka-help">{warning}</p>{/each}
            <details><summary>Files to import (first {rpgImportPreview.sample_files.length})</summary>
              <ul>{#each rpgImportPreview.sample_files as file}<li>{file}</li>{/each}</ul>
            </details>
            <p><strong>Source:</strong> {rpgImportSource}</p>
            <p><strong>Campaign:</strong> {rpgImportDestination || "Not selected"}</p>
            <button type="button" class="ka-button ka-button--ghost" onclick={chooseRpgDestination}>Change destination</button>
            <button type="button" class="ka-button" disabled={rpgImportBusy || !rpgImportDestination}
              onclick={applyRpgImport}>{rpgImportBusy ? "Importing…" : "Import into campaign"}</button>
            <button type="button" class="ka-button ka-button--ghost" onclick={() => rpgImportPreview = null}>Back to formats</button>
          </div>
        {:else}
          <ul class="import-list">
            {#each RPG_IMPORTS as format (format.type)}
              <li><button type="button" class="ka-button ka-button--ghost import-format"
                disabled={rpgImportBusy} onclick={() => selectRpgImport(format.type)}>
                {format.label}<small>{format.hint}</small>
              </button></li>
            {/each}
          </ul>
          <p class="ka-help">Foundry, Roll20 and Fantasy Grounds exports are preserved as source references; complete mechanics and VTT automation are not imported.</p>
        {/if}
      </div>
    </div>
  </div>
{/if}

{#if showAllProjects && allProjects}
  <div class="project-library-backdrop" role="presentation" onclick={closeProjectLibrary}>
    <div
      class="project-library"
      tabindex="-1"
      role="dialog"
      aria-modal="true"
      aria-labelledby="project-library-heading"
      onclick={(event) => event.stopPropagation()}
      onkeydown={(event) => event.stopPropagation()}
    >
      <header class="project-library__header">
        <div>
          <span class="dashboard-eyebrow">Project Library</span>
          <h2 id="project-library-heading">All Projects</h2>
        </div>
        <button type="button" class="ka-button ka-button--ghost" onclick={closeProjectLibrary} aria-label="Close project library">Close</button>
      </header>
      <ul class="project-library__list">
        {#each allProjects as project (project.id)}
          {@const modified = modifiedDate(project.modified_at)}
          <li class="recent-row">
            <button type="button" class="recent-open" onclick={() => openProject(project)}>
              <span class="recent-title"><span class="recent-name">{project.name}</span></span>
              <time class="recent-date" datetime={project.modified_at}>{modified.short}</time>
            </button>
            <button
              type="button"
              class="ka-button ka-button--ghost ka-icon-button recent-delete"
              onclick={(event) => showDeleteConfirmation(event, project)}
              aria-label={`Remove ${project.name} from Catalyst`}
              title="Remove from Catalyst"
            >
              <Trash2 class="w-5 h-5" aria-hidden="true" />
            </button>
          </li>
        {/each}
      </ul>
    </div>
  </div>
{/if}

<!-- Delete Project Confirmation -->
{#if projectToDelete}
  <ConfirmDialog
    title={`Remove "${projectToDelete.name}" from Catalyst?`}
    message="This removes the book from your Catalyst project list only. Your manuscript, project folder, and all local files remain untouched. You can reopen the folder later."
    confirmLabel="Remove from list"
    onConfirm={confirmDeleteProject}
    onCancel={cancelDeleteProject}
  />
{/if}

<style>
  .start {
    box-sizing: border-box;
    width: 100%;
    height: 100%;
    min-height: 0;
    overflow-y: auto;
    scrollbar-gutter: stable;
    padding: var(--space-l) clamp(24px, 4vw, 64px) clamp(32px, 5vh, 56px);
    background: var(--color-bg);
    color: var(--color-text);
  }
.start-body {
    display: grid;
    grid-template-columns: minmax(300px, 0.82fr) minmax(420px, 1.35fr);
    gap: clamp(40px, 6vw, 88px);
    width: min(1180px, 100%);
    margin: 0 auto;
    align-items: stretch;
  }

  .start-intro {
    min-width: 0;
  }

  .signature {
    width: 100%;
    margin-bottom: clamp(28px, 4vh, 44px);
  }

  .brand {
    display: flex;
    width: 100%;
    margin: 0;
    justify-content: center;
  }

  .brand img {
    display: block;
    width: min(300px, 88%);
    height: auto;
  }

  .tagline {
    margin: 12px 0 0;
    color: var(--color-text-muted);
    font-size: 0.95rem;
    line-height: 1.5;
    text-align: center;
  }

  .start-panel,
  .dashboard-card {
    box-sizing: border-box;
    border: var(--border-hair);
    border-radius: var(--radius-l);
    background: var(--color-surface);
  }

  .start-panel {
    padding: clamp(20px, 2.5vw, 28px);
  }

  .section-title {
    margin: 0;
    color: var(--color-text);
    font-size: 0.78rem;
    font-weight: 700;
    line-height: 1.2;
    letter-spacing: 0.09em;
    text-transform: uppercase;
  }

  .start-actions {
    display: grid;
    gap: 10px;
    margin-top: 18px;
  }

  .start-action {
    display: flex;
    width: 100%;
    min-height: 58px;
    justify-content: flex-start;
    align-items: center;
    gap: 14px;
    padding: 11px 14px;
    text-align: left;
  }

  .start-action > span {
    display: flex;
    min-width: 0;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    font-weight: 650;
  }

  .start-action small {
    color: var(--color-text-muted);
    font-size: 0.76rem;
    font-weight: 400;
    line-height: 1.35;
  }

  .start-action:not(.ka-button--secondary) {
    color: var(--color-on-accent);
  }

  .start-action:not(.ka-button--secondary) small {
    color: var(--color-on-accent);
    opacity: 0.82;
  }

  .import-backdrop {
    position: fixed;
    inset: 0;
    z-index: 85;
    display: grid;
    place-items: center;
    padding: 32px;
    background: rgb(0 0 0 / 0.68);
    backdrop-filter: blur(4px);
  }

  .import-dialog {
    box-sizing: border-box;
    width: min(640px, 94vw);
    max-height: min(680px, 86vh);
    overflow: hidden;
    border: var(--border-hair);
    border-radius: var(--radius-l);
    background: var(--color-surface);
    box-shadow: 0 24px 80px rgb(0 0 0 / 0.45);
  }

  .import-dialog__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    padding: 22px 24px;
    border-bottom: var(--border-hair);
  }

  .import-dialog__header h2 {
    margin: 3px 0 0;
    font-family: var(--font-display);
    font-size: 1.35rem;
  }

  .import-dialog__body {
    padding: 20px 24px 24px;
    overflow-y: auto;
  }

  .import-help {
    margin: 5px 0 12px;
    line-height: 1.4;
  }

  .import-list {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 6px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .import-format {
    display: flex;
    width: 100%;
    height: 100%;
    min-height: 48px;
    flex-direction: column;
    align-items: flex-start;
    justify-content: center;
    gap: 2px;
    padding: 8px 10px;
    text-align: left;
  }

  .import-format small {
    color: var(--color-text-muted);
    font-size: 0.7rem;
    font-weight: 400;
    line-height: 1.25;
  }

  .start-main {
    display: grid;
    min-width: 0;
    gap: 22px;
    grid-template-rows: auto auto 1fr;
  }

  .dashboard-card {
    padding: clamp(20px, 2.5vw, 28px);
  }

  .section-head {
    display: flex;
    min-height: 34px;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 12px;
  }

  .open-error {
    margin-bottom: 12px;
  }

  .recent-list {
    margin: 0;
    padding: 0;
    list-style: none;
    border-top: var(--border-hair);
  }

  .recent-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: stretch;
    border-bottom: var(--border-hair);
  }

  .recent-open {
    display: flex;
    min-width: 0;
    min-height: 68px;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    padding: 10px 8px;
    border: 0;
    background: transparent;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .recent-open:hover {
    background: var(--color-surface-hover);
  }

  .recent-open:focus-visible {
    outline: var(--focus-ring);
    outline-offset: -2px;
  }

  .recent-title {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 4px;
  }

  .recent-name {
    overflow: hidden;
    color: var(--color-text);
    font-size: 0.98rem;
    font-weight: 650;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .recent-date {
    color: var(--color-text-muted);
    font-size: 0.76rem;
  }

  .recent-date {
    flex: none;
    white-space: nowrap;
  }

  .recent-delete {
    align-self: center;
    margin-left: 4px;
  }

  .continue-card {
    display: flex;
    min-height: 158px;
    align-items: center;
    justify-content: space-between;
    gap: 28px;
  }

  .continue-copy {
    min-width: 0;
  }

  .dashboard-eyebrow {
    display: block;
    margin-bottom: 8px;
    color: var(--color-text-muted);
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.09em;
    text-transform: uppercase;
  }

  .continue-copy h2 {
    margin: 0;
    overflow: hidden;
    color: var(--color-text);
    font-size: clamp(1.25rem, 2vw, 1.65rem);
    font-weight: 650;
    line-height: 1.2;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .continue-copy p {
    margin: 8px 0 0;
    color: var(--color-text-muted);
    font-size: 0.82rem;
    line-height: 1.5;
  }
.continue-button {
    flex: none;
    min-width: 110px;
  }

  .empty-dashboard {
    display: flex;
    min-height: 180px;
    align-items: center;
    gap: 18px;
  }

  .empty-dashboard :global(svg) {
    flex: none;
    color: var(--color-text-muted);
  }

  .empty-dashboard p {
    margin: 8px 0 0;
    color: var(--color-text-muted);
    line-height: 1.5;
  }

  @media (max-width: 900px) {
    .start {
      padding-inline: 28px;
    }

    .start-body {
      grid-template-columns: minmax(260px, 0.8fr) minmax(340px, 1.2fr);
      gap: 32px;
    }

    .brand img {
      width: min(260px, 90%);
    }

    .import-list {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 720px) {
    .start {
      padding: 18px 20px 32px;
    }
.start-body {
      grid-template-columns: 1fr;
      gap: 24px;
    }

    .signature {
      margin-bottom: 24px;
    }

    .brand img {
      width: min(280px, 82%);
    }

    .continue-card {
      align-items: flex-start;
      flex-direction: column;
      gap: 18px;
    }

    .continue-button {
      width: 100%;
    }
  }

  /* Keep the home dashboard stable whether 1, 2, or 3 recent projects exist. */
  [data-testid="recent-projects"] {
    min-height: 248px;
  }

  [data-testid="recent-projects"] .recent-list {
    min-height: 168px;
  }


  .metrics-card {
    min-height: 148px;
    height: 100%;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    margin-top: 8px;
    border-top: var(--border-hair);
  }

  .metric {
    min-width: 0;
    padding: 18px 18px 8px 0;
    text-align: center;
  }

  .metric + .metric {
    padding-left: 18px;
    border-left: var(--border-hair);
  }

  .metric-label {
    display: block;
    margin-bottom: 7px;
    color: var(--color-text-muted);
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .metric-value {
    display: block;
    overflow: hidden;
    color: var(--color-text);
    font-family: var(--font-display);
    font-size: 1.45rem;
    font-weight: 650;
    line-height: 1.2;
    text-overflow: ellipsis;
    white-space: nowrap;
  }


  .project-library-backdrop {
    position: fixed;
    inset: 0;
    z-index: 80;
    display: grid;
    place-items: center;
    padding: 32px;
    background: rgb(0 0 0 / 0.68);
    backdrop-filter: blur(4px);
  }

  .project-library {
    box-sizing: border-box;
    width: min(860px, 94vw);
    max-height: min(760px, 86vh);
    overflow: hidden;
    border: var(--border-hair);
    border-radius: var(--radius-l);
    background: var(--color-surface);
    box-shadow: 0 24px 80px rgb(0 0 0 / 0.45);
  }

  .project-library__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    padding: 22px 24px;
    border-bottom: var(--border-hair);
  }

  .project-library__header h2 {
    margin: 3px 0 0;
    font-family: var(--font-display);
    font-size: 1.45rem;
  }

  .project-library__list {
    max-height: calc(min(760px, 86vh) - 88px);
    overflow-y: auto;
    margin: 0;
    padding: 0 24px 24px;
    list-style: none;
  }

  @media (max-width: 720px) {
    [data-testid="recent-projects"],
    [data-testid="recent-projects"] .recent-list {
      min-height: 0;
    }

    .metrics-grid {
      grid-template-columns: 1fr;
    }

    .metric {
      padding: 14px 0;
    }

    .metric + .metric {
      padding-left: 0;
      border-top: var(--border-hair);
      border-left: 0;
    }
  }
</style>
