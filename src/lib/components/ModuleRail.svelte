<script lang="ts">
  import {
    BookOpenText,
    BookOpen,
    GitBranch,
    Headphones,
    Home,
    LibraryBig,
    ListTree,
    Mic2,
    Settings,
    SquareTerminal,
    Upload,
  } from "lucide-svelte";
  import { WORKSPACES, type WorkspaceId } from "../workspaces";
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
    write: BookOpenText,
    "story-development": LibraryBig,
    outline: ListTree,
    review: BookOpen,
    listen: Headphones,
    audiobook: Mic2,
    publish: Upload,
    git: GitBranch,
    terminal: SquareTerminal,
    settings: Settings,
  };

  let switchingWorkspace = false;

  async function selectWorkspace(id: WorkspaceId) {
    if (switchingWorkspace || id === workspace.active) return;
    switchingWorkspace = true;
    try {
      const pendingDocumentSaves: Promise<boolean>[] = [];
      const guard = new CustomEvent("lithomark:before-workspace-change", {
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

<nav class="module-rail" aria-label="Catalyst workspaces">
  <div class="module-rail__brand" aria-hidden="true">L</div>

  <div class="module-rail__modules">
    {#each WORKSPACES.filter((item) => item.stage !== "optional-first-party") as item (item.id)}
      {@const Icon = icons[item.id]}
      {@const unavailable =
        item.requiresProject &&
        !hasActiveProject &&
        !(item.id === "story-development" && onStartStoryDevelopment)}
      <button
        type="button"
        class="module-rail__button"
        class:module-rail__button--active={workspace.active === item.id && !unavailable}
        class:module-rail__button--future={item.stage !== "core-preview"}
        aria-label={item.label}
        aria-current={workspace.active === item.id && !unavailable ? "page" : undefined}
        disabled={unavailable}
        title={unavailable ? `${item.label} — open a project first` : item.label}
        data-testid={`module-${item.id}`}
        onclick={() => selectWorkspace(item.id)}
      >
        <Icon size={20} aria-hidden="true" />
        <span>{item.label}</span>
      </button>
    {/each}
  </div>
</nav>

<style>
  .module-rail {
    flex: 0 0 72px;
    width: 72px;
    min-height: 0;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    border-right: 1px solid var(--color-border);
    background: var(--color-surface);
    overflow-y: auto;
    overflow-x: hidden;
  }

  .module-rail__brand {
    height: 54px;
    flex: 0 0 54px;
    display: grid;
    place-items: center;
    border-bottom: 1px solid var(--color-border);
    color: var(--color-accent-text);
    font: 700 22px/1 var(--font-display);
  }

  .module-rail__modules {
    display: grid;
    gap: 2px;
    padding: 6px;
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
    border-radius: var(--radius-m);
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .module-rail__button span {
    max-width: 62px;
    font: 500 9px/1.15 var(--font-ui);
    text-align: center;
    white-space: normal;
    overflow-wrap: anywhere;
  }

  .module-rail__button:hover:not(:disabled) {
    background: var(--color-surface-raised);
    color: var(--color-text);
  }

  .module-rail__button--active {
    background: var(--color-surface-raised);
    color: var(--color-accent-text);
  }

  .module-rail__button--active::before {
    content: "";
    position: absolute;
    inset: 8px auto 8px -6px;
    width: 3px;
    border-radius: 0 3px 3px 0;
    background: var(--color-accent-text);
  }

  .module-rail__button--future:not(.module-rail__button--active) {
    color: var(--color-text-muted);
  }

  .module-rail__button:disabled {
    background: transparent !important;
  }
</style>
