import { describe, expect, it } from "vitest";
import {
  DEFAULT_GLOBAL_WORKSPACE,
  DEFAULT_PROJECT_WORKSPACE,
  WORKSPACES,
  WORKSPACE_GROUPS,
  getWorkspace,
  isPreviewWorkspace,
  resolveWorkspace,
  workspacesForState,
} from "./workspaces";

describe("Catalyst workspace registry", () => {
  it("uses Home globally and Write as the default project workspace", () => {
    expect(DEFAULT_GLOBAL_WORKSPACE).toBe("home");
    expect(DEFAULT_PROJECT_WORKSPACE).toBe("home");
  });

  it("contains each workspace exactly once", () => {
    const ids = WORKSPACES.map((workspace) => workspace.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(ids).toEqual([
      "home",
      "story-development",
      "outline",
      "characters",
      "write",
      "listen",
      "gm-screen",
      "publish",
      "git",
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
    expect(resolveWorkspace("listen", true)).toBe("listen");
  });

  it("only exposes global workspaces before a project is open", () => {
    expect(workspacesForState(false).map((workspace) => workspace.id)).toEqual([
      "home",
      "settings",
    ]);
  });

  it("marks the active Core Preview workspaces", () => {
    expect(WORKSPACES.filter((workspace) => isPreviewWorkspace(workspace.id)).map((w) => w.id))
      .toEqual(["home", "story-development", "outline", "write", "listen", "gm-screen", "git"]);
  });

  it("removes the Terminal and book-only workspaces from the campaign rail", () => {
    const ids = WORKSPACES.map((workspace) => workspace.id as string);
    expect(ids).not.toContain("terminal");
    expect(ids).not.toContain("review");
    expect(ids).not.toContain("audiobook");
  });

  it("uses tabletop campaign labels while keeping stable workspace ids", () => {
    expect(getWorkspace("outline").label).toBe("Adventures");
    expect(getWorkspace("characters").label).toBe("Characters");
    expect(getWorkspace("write").label).toBe("Campaign Notes");
    expect(getWorkspace("listen").label).toBe("Soundboard");
    expect(getWorkspace("gm-screen").label).toBe("GM Screen");
    expect(getWorkspace("publish").label).toBe("Print & Export");
    expect(getWorkspace("git").label).toBe("GitHub Sync");
  });

  it("groups the rail into campaign, table, sharing, and app sections", () => {
    expect(WORKSPACE_GROUPS.map((group) => group.id)).toEqual(["campaign", "table", "share", "app"]);
    expect(WORKSPACES.filter((w) => w.group === "campaign").map((w) => w.id)).toEqual([
      "home",
      "story-development",
      "outline",
      "characters",
      "write",
    ]);
    expect(WORKSPACES.filter((w) => w.group === "table").map((w) => w.id)).toEqual(["listen", "gm-screen"]);
    expect(WORKSPACES.filter((w) => w.group === "share").map((w) => w.id)).toEqual(["publish", "git"]);
    expect(getWorkspace("settings").group).toBe("app");
  });

  it("describes World Builder (Story Development) as the campaign Wiki workspace", () => {
    const workspace = getWorkspace("story-development");
    expect(workspace.label).toBe("World Builder");
    expect(workspace.description).toContain("Wiki");
    expect(workspace.requiresProject).toBe(true);
  });
});
