<script lang="ts">
  import {
    Earth,
    FileOutput,
    GitBranch,
    Home,
    Map as MapIcon,
    NotebookPen,
    Settings,
    Users,
    AudioLines,
  } from "lucide-svelte";
  import {
    WORKSPACES,
    WORKSPACE_GROUPS,
    type WorkspaceDefinition,
    type WorkspaceId,
  } from "../workspaces";
  import { workspace } from "../stores/workspace.svelte";

  let {
    hasActiveProject,
    onOpenSettings,
    onOpenHome,
    onStartStoryDevelopment,
    onOpenStoryDevelopment,
    onProjectWorkspaceActivated,
  }: {
    hasActiveProject: boolean;
    onOpenSettings?: () => void;
    onOpenHome?: () => void;
    onStartStoryDevelopment?: () => void;
    onOpenStoryDevelopment?: () => boolean;
    onProjectWorkspaceActivated?: () => void;
  } = $props();

  const icons: Record<WorkspaceId, typeof Home> = {
    home: Home,
    "story-development": Earth,
    outline: MapIcon,
    characters: Users,
    write: NotebookPen,
    listen: AudioLines,
    publish: FileOutput,
    git: GitBranch,
    settings: Settings,
  };

  const visible = WORKSPACES.filter((item) => item.stage !== "optional-first-party");
  const groups = WORKSPACE_GROUPS.filter((group) => group.id !== "app")
    .map((group) => ({ ...group, items: visible.filter((item) => item.group === group.id) }))
    .filter((group) => group.items.length > 0);
  const appItems = visible.filter((item) => item.group === "app");

  function isUnavailable(item: WorkspaceDefinition) {
    return (
      item.requiresProject &&
      !hasActiveProject &&
      !(item.id === "story-development" && onStartStoryDevelopment)
    );
  }

  let switchingWorkspace = false;

  async function selectWorkspace(id: WorkspaceId) {
    if (switchingWorkspace || id === workspace.active) return;
    switchingWorkspace = true;
    try {
      const pendingDocumentSaves: Promise<boolean>[] = [];
      const guard = new CustomEvent("catalyst:before-workspace-change", {
        cancelable: true,
        detail: { from: workspace.active, to: id, pendingDocumentSaves },
      });
      if (!window.dispatchEvent(guard)) return;
      const results = await Promise.allSettled(pendingDocumentSaves);
      if (results.some((result) => result.status === "rejected" || !result.value)) return;
      activateWorkspace(id);
    } finally {
      switchingWorkspace = false;
    }
  }

  function activateWorkspace(id: WorkspaceId) {
    if (id === "home") {
      onOpenHome?.();
      workspace.setActive("home", hasActiveProject);
      return;
    }


    if (id === "settings" && onOpenSettings) {
      onOpenSettings();
      return;
    }
    if (id === "story-development") {
      if (!hasActiveProject && onStartStoryDevelopment) {
        onStartStoryDevelopment();
        return;
      }
      if (hasActiveProject && onOpenStoryDevelopment && !onOpenStoryDevelopment()) return;
    }
    workspace.setActive(id, hasActiveProject);
    if (hasActiveProject) onProjectWorkspaceActivated?.();
  }
</script>

{#snippet railButton(item: WorkspaceDefinition)}
  {@const Icon = icons[item.id]}
  {@const unavailable = isUnavailable(item)}
  {@const active = workspace.active === item.id && !unavailable}
  <button
    type="button"
    class="module-rail__button"
    class:module-rail__button--active={active}
    class:module-rail__button--future={item.stage !== "core-preview"}
    aria-label={item.label}
    aria-current={active ? "page" : undefined}
    disabled={unavailable}
    title={unavailable ? `${item.label} — open a campaign first` : item.label}
    data-testid={`module-${item.id}`}
    onclick={() => selectWorkspace(item.id)}
  >
    <Icon size={20} aria-hidden="true" />
    <span>{item.label}</span>
  </button>
{/snippet}

<nav class="module-rail" aria-label="Catalyst workspaces">
  <div class="module-rail__brand" aria-hidden="true">C</div>

  <div class="module-rail__modules">
    {#each groups as group (group.id)}
      <div class="module-rail__group" role="group" aria-label={group.label} data-testid={`rail-group-${group.id}`}>
        <p class="module-rail__group-label" aria-hidden="true">{group.label}</p>
        {#each group.items as item (item.id)}
          {@render railButton(item)}
        {/each}
      </div>
    {/each}
  </div>

  <!-- Global utilities and app settings stay pinned to the bottom of the rail. -->
  <div class="module-rail__utilities">
    {#each appItems as item (item.id)}
      {@render railButton(item)}
    {/each}
  </div>
</nav>

<style>
  /* Obsidian + Frost Blue (docs/architecture/025-CATALYST-DESIGN-SYSTEM.md). */
  .module-rail {
    flex: 0 0 80px;
    width: 80px;
    min-height: 0;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    border-right: 1px solid var(--cat-border-subtle);
    background: var(--cat-sidebar);
    color: var(--cat-silver);
    overflow-y: auto;
    overflow-x: hidden;
  }

  .module-rail__brand {
    height: 54px;
    flex: 0 0 54px;
    display: grid;
    place-items: center;
    border-bottom: 1px solid var(--cat-border-subtle);
    color: var(--cat-accent);
    font: 700 22px/1 var(--font-ui);
    letter-spacing: 0.02em;
  }

  .module-rail__modules {
    display: grid;
    gap: 4px;
    padding: 6px;
  }

  .module-rail__group {
    display: grid;
    gap: 2px;
  }

  .module-rail__group + .module-rail__group {
    margin-top: 2px;
    padding-top: 6px;
    border-top: 1px solid var(--cat-border-subtle);
  }

  .module-rail__group-label {
    margin: 2px 0 2px;
    color: var(--cat-muted);
    font: 600 8.5px/1.2 var(--font-ui);
    letter-spacing: 0.08em;
    text-align: center;
    text-transform: uppercase;
  }

  .module-rail__utilities {
    margin-top: auto;
    display: grid;
    gap: 2px;
    padding: 6px;
    border-top: 1px solid var(--cat-border-subtle);
  }

  .module-rail__button {
    position: relative;
    min-height: 54px;
    display: grid;
    justify-items: center;
    align-content: center;
    gap: 4px;
    padding: 6px 4px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--cat-silver);
    cursor: pointer;
    transition: background-color 140ms ease, color 140ms ease;
  }

  .module-rail__button span {
    max-width: 70px;
    font: 500 9.5px/1.15 var(--font-ui);
    text-align: center;
    white-space: normal;
    overflow-wrap: normal;
  }

  .module-rail__button:hover:not(:disabled) {
    background: var(--cat-hover);
    color: var(--cat-text);
  }

  .module-rail__button:focus-visible {
    outline: 2px solid var(--cat-focus);
    outline-offset: -2px;
  }

  .module-rail__button--active {
    background: var(--cat-accent-soft);
    color: var(--cat-text);
  }

  .module-rail__button--active :global(svg) {
    color: var(--cat-accent);
  }

  .module-rail__button--active::before {
    content: "";
    position: absolute;
    inset: 8px auto 8px -6px;
    width: 3px;
    border-radius: 0 3px 3px 0;
    background: var(--cat-accent);
  }

  /* Overrides the global disabled token pair in app.css, which uses !important. */
  .module-rail__button:disabled {
    background: transparent !important;
    color: var(--cat-disabled) !important;
    cursor: not-allowed;
  }

  @media (prefers-reduced-motion: reduce) {
    .module-rail__button {
      transition: none;
    }
  }
</style>
