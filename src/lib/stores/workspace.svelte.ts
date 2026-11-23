import {
  DEFAULT_GLOBAL_WORKSPACE,
  DEFAULT_PROJECT_WORKSPACE,
  resolveWorkspace,
  type WorkspaceId,
} from "../workspaces";

/**
 * Writer OS shell state.
 *
 * This is intentionally UI-only state. It never owns project content.
 * The active project itself remains managed by the project layer; this store
 * only decides which top-level workspace is visible.
 */
class WorkspaceStore {
  private _active = $state<WorkspaceId>(DEFAULT_GLOBAL_WORKSPACE);

  get active(): WorkspaceId {
    return this._active;
  }

  setActive(requested: WorkspaceId, hasActiveProject: boolean): WorkspaceId {
    const resolved = resolveWorkspace(requested, hasActiveProject);
    this._active = resolved;
    return resolved;
  }

  enterProject(preferred: WorkspaceId = DEFAULT_PROJECT_WORKSPACE): WorkspaceId {
    return this.setActive(preferred, true);
  }

  leaveProject(): WorkspaceId {
    this._active = DEFAULT_GLOBAL_WORKSPACE;
    return this._active;
  }

  /**
   * Transitional bridge for the Kindling-era start/editor binary.
   *
   * New shell code should use WorkspaceId directly. Existing code can call this
   * while it is being migrated without making "editor" a new source of truth.
   */
  setLegacyView(view: "start" | "editor", hasActiveProject: boolean): WorkspaceId {
    if (view === "start") {
      return this.leaveProject();
    }
    return this.setActive(DEFAULT_PROJECT_WORKSPACE, hasActiveProject);
  }
}

export const workspace = new WorkspaceStore();
