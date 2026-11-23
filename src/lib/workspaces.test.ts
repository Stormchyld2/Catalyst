import { describe, expect, it } from "vitest";
import {
  DEFAULT_GLOBAL_WORKSPACE,
  DEFAULT_PROJECT_WORKSPACE,
  WORKSPACES,
  getWorkspace,
  isPreviewWorkspace,
  resolveWorkspace,
  workspacesForState,
} from "./workspaces";

describe("Writer OS workspace registry", () => {
  it("uses Home globally and Write as the default project workspace", () => {
    expect(DEFAULT_GLOBAL_WORKSPACE).toBe("home");
    expect(DEFAULT_PROJECT_WORKSPACE).toBe("write");
  });

  it("contains each workspace exactly once", () => {
    const ids = WORKSPACES.map((workspace) => workspace.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(ids).toEqual([
      "home",
      "story-development",
      "outline",
      "write",
      "review",
      "listen",
      "audiobook",
      "publish",
      "git",
      "terminal",
      "settings",
    ]);
  });

  it("keeps project-bound workspaces behind an active project boundary", () => {
    expect(resolveWorkspace("write", false)).toBe("home");
    expect(resolveWorkspace("story-development", false)).toBe("home");
    expect(resolveWorkspace("outline", false)).toBe("home");
    expect(resolveWorkspace("git", false)).toBe("home");
  });

  it("allows global workspaces without a project", () => {
    expect(resolveWorkspace("home", false)).toBe("home");
    expect(resolveWorkspace("settings", false)).toBe("settings");
  });

  it("allows project workspaces when a project is active", () => {
    expect(resolveWorkspace("write", true)).toBe("write");
    expect(resolveWorkspace("story-development", true)).toBe("story-development");
    expect(resolveWorkspace("outline", true)).toBe("outline");
    expect(resolveWorkspace("review", true)).toBe("review");
  });

  it("only exposes global workspaces before a project is open", () => {
    expect(workspacesForState(false).map((workspace) => workspace.id)).toEqual([
      "home",
      "settings",
    ]);
  });

  it("marks the active Core Preview workspaces", () => {
    expect(WORKSPACES.filter((workspace) => isPreviewWorkspace(workspace.id)).map((w) => w.id))
      .toEqual(["home", "story-development", "outline", "write", "git", "terminal"]);
  });

  it("describes Story Development as the project Wiki/planning workspace", () => {
    const workspace = getWorkspace("story-development");
    expect(workspace.label).toBe("Story Dev");
    expect(workspace.description).toContain("Wiki");
    expect(workspace.requiresProject).toBe(true);
  });
});
