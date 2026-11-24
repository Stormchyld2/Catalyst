// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/svelte";
import ModuleRail from "./ModuleRail.svelte";
import { workspace } from "../stores/workspace.svelte";

afterEach(() => {
  document.body.innerHTML = "";
});

beforeEach(() => {
  workspace.leaveProject();
});

describe("ModuleRail", () => {
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

  it("renders the campaign rail groups with Settings pinned at the bottom", () => {
    render(ModuleRail, { props: { hasActiveProject: true } });
    const campaign = screen.getByTestId("rail-group-campaign");
    expect(campaign.getAttribute("aria-label")).toBe("Campaign");
    expect(campaign.querySelector("[data-testid=module-story-development]")?.textContent).toContain(
      "World Builder"
    );
    expect(campaign.querySelector("[data-testid=module-outline]")?.textContent).toContain("Adventures");
    expect(campaign.querySelector("[data-testid=module-characters]")).not.toBeNull();
    expect(campaign.querySelector("[data-testid=module-write]")?.textContent).toContain("Campaign Notes");
    expect(
      screen.getByTestId("rail-group-table").querySelector("[data-testid=module-listen]")?.textContent
    ).toContain("Soundboard");
    const share = screen.getByTestId("rail-group-share");
    expect(share.querySelector("[data-testid=module-publish]")?.textContent).toContain("Print & Export");
    expect(share.querySelector("[data-testid=module-git]")?.textContent).toContain("GitHub Sync");
    expect(document.querySelector(".module-rail__utilities [data-testid=module-settings]")).not.toBeNull();
  });

  it("no longer offers the Terminal or book-only workspaces", () => {
    render(ModuleRail, { props: { hasActiveProject: true } });
    expect(screen.queryByTestId("module-terminal")).toBeNull();
    expect(screen.queryByTestId("module-review")).toBeNull();
    expect(screen.queryByTestId("module-audiobook")).toBeNull();
  });

  it("opens the existing Settings dialog through the shell callback", async () => {
    const onOpenSettings = vi.fn();
    render(ModuleRail, { props: { hasActiveProject: false, onOpenSettings } });

    await fireEvent.click(screen.getByTestId("module-settings"));

    expect(onOpenSettings).toHaveBeenCalledTimes(1);
    expect(workspace.active).toBe("home");
  });
});
