import { beforeEach, describe, expect, it } from "vitest";
import { workspace } from "./workspace.svelte";

describe("Writer OS workspace state", () => {
  beforeEach(() => {
    workspace.leaveProject();
  });

  it("starts and resets at Home", () => {
    expect(workspace.active).toBe("home");
    expect(workspace.leaveProject()).toBe("home");
  });

  it("enters an active project in Write by default", () => {
    expect(workspace.enterProject()).toBe("write");
    expect(workspace.active).toBe("write");
  });

  it("can enter an active project directly in Story Development", () => {
    expect(workspace.enterProject("story-development")).toBe("story-development");
    expect(workspace.active).toBe("story-development");
  });

  it("does not allow a project-only workspace without a project", () => {
    expect(workspace.setActive("review", false)).toBe("home");
    expect(workspace.active).toBe("home");
  });

  it("allows Settings without an active project", () => {
    expect(workspace.setActive("settings", false)).toBe("settings");
    expect(workspace.active).toBe("settings");
  });

  it("bridges legacy start to Home", () => {
    workspace.enterProject();
    expect(workspace.setLegacyView("start", true)).toBe("home");
  });

  it("bridges legacy editor to Write only when a project is active", () => {
    expect(workspace.setLegacyView("editor", false)).toBe("home");
    expect(workspace.setLegacyView("editor", true)).toBe("write");
  });
});
