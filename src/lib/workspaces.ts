/**
 * Canonical Catalyst workspace registry.
 *
 * This module intentionally contains no Svelte state and no persistence.
 * It defines the stable application-level navigation contract that the
 * new shell, command palette, keyboard shortcuts, plugins, and tests can share.
 */

export type WorkspaceId =
  | "home"
  | "write"
  | "story-development"
  | "outline"
  | "characters"
  | "listen"
  | "publish"
  | "git"
  | "settings";

export type WorkspaceStage = "core-preview" | "later-core" | "optional-first-party";

/**
 * Rail grouping for tabletop campaigns. Ids stay stable for compatibility with
 * existing projects and code; labels follow the Catalyst RPG module names.
 */
export type WorkspaceGroup = "campaign" | "table" | "share" | "app";

export const WORKSPACE_GROUPS: readonly { id: WorkspaceGroup; label: string }[] = [
  { id: "campaign", label: "Campaign" },
  { id: "table", label: "At the table" },
  { id: "share", label: "Save & share" },
  { id: "app", label: "App" },
] as const;

export interface WorkspaceDefinition {
  id: WorkspaceId;
  label: string;
  requiresProject: boolean;
  stage: WorkspaceStage;
  group: WorkspaceGroup;
  description: string;
}

export const WORKSPACES: readonly WorkspaceDefinition[] = [
  {
    id: "home",
    label: "Home",
    requiresProject: false,
    stage: "core-preview",
    group: "campaign",
    description: "Campaign library, recent work, campaign creation, opening, import, and clone.",
  },
  {
    id: "story-development",
    label: "World Builder",
    requiresProject: true,
    stage: "core-preview",
    group: "campaign",
    description: "Campaign Wiki: worldbuilding, locations, factions, NPCs, lore, and reference material.",
  },
  {
    id: "outline",
    label: "Adventures",
    requiresProject: true,
    stage: "core-preview",
    group: "campaign",
    description: "Adventure planning: timeline, quests, story arcs, scenes, and beats.",
  },
  {
    id: "characters",
    label: "Characters",
    requiresProject: true,
    stage: "later-core",
    group: "campaign",
    description: "Character Studio: player characters and sheets (coming from the Character Studio milestone).",
  },
  {
    id: "write",
    label: "Campaign Notes",
    requiresProject: true,
    stage: "core-preview",
    group: "campaign",
    description: "File-authoritative campaign writing: session notes, read-aloud text, and handouts.",
  },
  {
    id: "listen",
    label: "Soundboard",
    requiresProject: true,
    stage: "core-preview",
    group: "table",
    description: "Campaign soundboard: offline ambience, music, and sound-effect pads for live play.",
  },
  {
    id: "publish",
    label: "Print & Export",
    requiresProject: true,
    stage: "later-core",
    group: "share",
    description: "Print and export character sheets, handouts, maps, and session reports.",
  },
  {
    id: "git",
    label: "GitHub Sync",
    requiresProject: true,
    stage: "core-preview",
    group: "share",
    description: "Save campaign history and sync it to a GitHub account: status, commit, pull, push.",
  },
  {
    id: "settings",
    label: "Settings",
    requiresProject: false,
    stage: "later-core",
    group: "app",
    description: "Application settings and optional component management.",
  },
] as const;

const WORKSPACE_BY_ID = new Map<WorkspaceId, WorkspaceDefinition>(
  WORKSPACES.map((workspace) => [workspace.id, workspace])
);

export const DEFAULT_GLOBAL_WORKSPACE: WorkspaceId = "home";
export const DEFAULT_PROJECT_WORKSPACE: WorkspaceId = "home";

export function getWorkspace(id: WorkspaceId): WorkspaceDefinition {
  const workspace = WORKSPACE_BY_ID.get(id);
  if (!workspace) {
    throw new Error(`Unknown Catalyst workspace '${id}'.`);
  }
  return workspace;
}

/**
 * Resolve a requested workspace against the active project boundary.
 *
 * Project-bound workspaces must never become active without an active project.
 * Home and Settings remain valid outside a project.
 */
export function resolveWorkspace(
  requested: WorkspaceId,
  hasActiveProject: boolean
): WorkspaceId {
  const workspace = getWorkspace(requested);
  if (workspace.requiresProject && !hasActiveProject) {
    return DEFAULT_GLOBAL_WORKSPACE;
  }
  return requested;
}

export function workspacesForState(hasActiveProject: boolean): readonly WorkspaceDefinition[] {
  return WORKSPACES.filter((workspace) => hasActiveProject || !workspace.requiresProject);
}

export function isPreviewWorkspace(id: WorkspaceId): boolean {
  return getWorkspace(id).stage === "core-preview";
}
