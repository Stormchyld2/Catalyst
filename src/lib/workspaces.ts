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
  | "review"
  | "listen"
  | "audiobook"
  | "publish"
  | "git"
  | "terminal"
  | "settings";

export type WorkspaceStage = "core-preview" | "later-core" | "optional-first-party";

export interface WorkspaceDefinition {
  id: WorkspaceId;
  label: string;
  requiresProject: boolean;
  stage: WorkspaceStage;
  description: string;
}

export const WORKSPACES: readonly WorkspaceDefinition[] = [
  {
    id: "home",
    label: "Home",
    requiresProject: false,
    stage: "core-preview",
    description: "Project library, recent work, project creation, opening, import, and clone.",
  },
  {
    id: "story-development",
    label: "Story Dev",
    requiresProject: true,
    stage: "core-preview",
    description: "Project Wiki, characters, locations, worldbuilding, and reference material.",
  },
  {
    id: "outline",
    label: "Outline",
    requiresProject: true,
    stage: "core-preview",
    description: "Visual timeline, plotlines, chapters, scenes, and beats.",
  },
  {
    id: "write",
    label: "Write",
    requiresProject: true,
    stage: "core-preview",
    description: "File-authoritative manuscript navigation and writing.",
  },
  {
    id: "review",
    label: "Review",
    requiresProject: true,
    stage: "later-core",
    description: "Developmental review, beta reading, ARC, and revision workflows.",
  },
  {
    id: "listen",
    label: "Listen",
    requiresProject: true,
    stage: "later-core",
    description: "Core text-to-speech reader for the active manuscript.",
  },
  {
    id: "audiobook",
    label: "Audiobook",
    requiresProject: true,
    stage: "optional-first-party",
    description: "Optional first-party audiobook production workspace.",
  },
  {
    id: "publish",
    label: "Publish",
    requiresProject: true,
    stage: "later-core",
    description: "Book formatting, export, and publishing preparation.",
  },
  {
    id: "git",
    label: "Git",
    requiresProject: true,
    stage: "core-preview",
    description: "Writer-friendly version-control status, commit, pull, push, and conflicts.",
  },
  {
    id: "terminal",
    label: "Terminal",
    requiresProject: true,
    stage: "core-preview",
    description: "Integrated terminal scoped to the active project.",
  },
  {
    id: "settings",
    label: "Settings",
    requiresProject: false,
    stage: "later-core",
    description: "Application settings and optional component management.",
  },
] as const;

const WORKSPACE_BY_ID = new Map<WorkspaceId, WorkspaceDefinition>(
  WORKSPACES.map((workspace) => [workspace.id, workspace])
);

export const DEFAULT_GLOBAL_WORKSPACE: WorkspaceId = "home";
export const DEFAULT_PROJECT_WORKSPACE: WorkspaceId = "write";

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
