// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import ModuleRail from "./ModuleRail.svelte";
import { workspace } from "../stores/workspace.svelte";
import { diceTray } from "../dice";

afterEach(() => {
  document.body.innerHTML = "";
});

beforeEach(() => {
  workspace.leaveProject();
});

describe("ModuleRail", () => {
  it("toggles the global dice tray without changing workspace", async () => {
    diceTray.close();
    workspace.enterProject();
    render(ModuleRail, { props: { hasActiveProject: true } });
    const before = workspace.active;
    const button = screen.getByTestId("utility-dice");
    await fireEvent.click(button);
    expect(diceTray.isOpen).toBe(true);
    expect(button.getAttribute("aria-pressed")).toBe("true");
    expect(workspace.active).toBe(before);
    await fireEvent.click(button);
    expect(diceTray.isOpen).toBe(false);
  });


  it("disables project workspaces when no project is active", () => {
    render(ModuleRail, { props: { hasActiveProject: false } });
    expect((screen.getByTestId("module-home") as HTMLButtonElement).disabled).toBe(false);
    expect((screen.getByTestId("module-write") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByTestId("module-outline") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByTestId("module-story-development") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByTestId("module-settings") as HTMLButtonElement).disabled).toBe(false);
  });

  it("switches between project workspaces when a project is active", async () => {
    workspace.enterProject();
    render(ModuleRail, { props: { hasActiveProject: true } });

    await fireEvent.click(screen.getByTestId("module-story-development"));

    expect(workspace.active).toBe("story-development");
    const buttons = Array.from(document.querySelectorAll("[data-testid^=module-]"));
    const ids = buttons.map((button) => button.getAttribute("data-testid"));
    expect(ids.indexOf("module-story-development")).toBeLessThan(ids.indexOf("module-outline"));
    expect(ids.indexOf("module-outline")).toBeLessThan(ids.indexOf("module-write"));
    await fireEvent.click(screen.getByTestId("module-outline"));
    expect(workspace.active).toBe("outline");
    expect(screen.getByTestId("module-story-development").getAttribute("aria-current")).toBeNull();
    expect(screen.getByTestId("module-outline").getAttribute("aria-current")).toBe("page");
  });


  it("starts the New Book Foundation from Story Development when no project is active", async () => {
    const onStartStoryDevelopment = vi.fn();
    render(ModuleRail, {
      props: { hasActiveProject: false, onStartStoryDevelopment },
    });

    const storyDevelopment = screen.getByTestId("module-story-development") as HTMLButtonElement;
    expect(storyDevelopment.disabled).toBe(false);

    await fireEvent.click(storyDevelopment);

    expect(onStartStoryDevelopment).toHaveBeenCalledTimes(1);
    expect(workspace.active).toBe("home");
  });

  it("opens the existing Settings dialog through the shell callback", async () => {
    const onOpenSettings = vi.fn();
    render(ModuleRail, { props: { hasActiveProject: false, onOpenSettings } });

    await fireEvent.click(screen.getByTestId("module-settings"));

    expect(onOpenSettings).toHaveBeenCalledTimes(1);
    expect(workspace.active).toBe("home");
  });
});
