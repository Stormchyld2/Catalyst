import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import StartScreen from "./StartScreen.svelte";
import { currentProject } from "../stores/project.svelte";
import { ui } from "../stores/ui.svelte";
import { workspace } from "../stores/workspace.svelte";
import type { Project } from "../types";

vi.hoisted(() => {
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (k: string) => values.get(k) ?? null,
    setItem: (k: string, v: string) => values.set(k, v),
    removeItem: (k: string) => values.delete(k),
    clear: () => values.clear(),
  });
});

function project(overrides: Partial<Project> = {}): Project {
  return {
    id: "project-1",
    name: "The Letter",
    source_type: "Markdown",
    source_path: null,
    project_root: null,
    created_at: "2026-09-10T12:00:00Z",
    modified_at: "2026-09-10T12:00:00Z",
    author_pen_name: null,
    genre: null,
    description: null,
    word_target: null,
    reference_types: ["characters"],
    project_type: "novel",
    target_page_count: null,
    ...overrides,
  };
}

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "get_project_dashboard_metrics") {
      return { word_count: 0, word_target: null, chapter_count: 0, scene_count: 0 };
    }
    return undefined;
  });
  currentProject.setProject(null);
  ui.setView("start");
  workspace.leaveProject();
});

afterEach(() => {
  cleanup();
  ui.finishImport();
});

describe("StartScreen", () => {
  it("opens The Hollow Crown D&D adventure in the campaign dashboard", async () => {
    const sample = project({
      id: "hollow-crown",
      name: "The Hollow Crown",
      source_type: "Blank",
      project_root: "C:/Temp/Catalyst/The Hollow Crown",
      genre: "D&D 5e Fantasy Mystery",
      word_target: null,
    });
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "create_sample_project") return sample;
      return [];
    });

    render(StartScreen, { recentProjects: [], onNewProject: vi.fn() });

    expect(screen.getByRole("button", { name: /The Hollow Crown — D&D 5e Demo/ })).toBeTruthy();
    expect(screen.getByText(/level-1 D&D 5e adventure/)).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: /The Hollow Crown/ }));
    await waitFor(() => expect(currentProject.value?.name).toBe("The Hollow Crown"));

    expect(invoke).toHaveBeenCalledWith("create_sample_project");
    expect(workspace.active).toBe("home");
  });

  it("lists recent projects as rows with plain metadata and a labelled delete action", () => {
    render(StartScreen, { recentProjects: [project()], onNewProject: vi.fn() });

    const list = screen.getByTestId("recent-projects");
    expect(within(list).getByRole("heading", { name: "Recent Projects" })).toBeTruthy();
    const row = within(list).getByTestId("project-card");
    expect(row.textContent).toContain("The Letter");
    // The importer id reads as sentence-case metadata, not "(markdown)".
    expect(row.textContent).toContain("The Letter");
    expect(row.textContent).not.toContain("(");
    expect(row.querySelector("time")?.getAttribute("datetime")).toBe("2026-09-10T12:00:00Z");
    expect(row.querySelector("time")?.getAttribute("aria-label")).toMatch(/^Modified /);
    expect(within(list).getByRole("button", { name: "Remove The Letter from Catalyst" })).toBeTruthy();
  });

  const many = (n: number) =>
    Array.from({ length: n }, (_, i) => project({ id: `p${i}`, name: `Book ${i + 1}` }));

  it("keeps View all as the first control when there are more projects", () => {
    render(StartScreen, { recentProjects: many(6) });
    const first = screen.getByTestId("recent-projects").querySelector("button");
    expect(first?.textContent?.trim()).toBe("View All");
    expect(first?.getAttribute("aria-haspopup")).toBe("dialog");
    expect(screen.getAllByTestId("project-card")).toHaveLength(3);
    expect(screen.queryByText("Book 6")).toBeNull();
  });

  it("offers no View all when every project is already listed", () => {
    render(StartScreen, { recentProjects: many(3) });
    expect(screen.queryByRole("button", { name: "View All" })).toBeNull();
    expect(screen.getByText("Book 3")).toBeTruthy();
  });

  it("lists every project, then returns to recent without refetching", async () => {
    const all = many(12);
    vi.mocked(invoke).mockImplementation(async (command) => command === "get_all_projects" ? all : { word_count: 0, word_target: null, chapter_count: 0, scene_count: 0 });
    render(StartScreen, { recentProjects: all.slice(0, 10) });
    await fireEvent.click(screen.getByRole("button", { name: "View All" }));
    expect(await screen.findByRole("heading", { name: "All Projects" })).toBeTruthy();
    expect(screen.getByText("Book 12")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("get_all_projects");

    await fireEvent.click(screen.getByRole("button", { name: "Close project library" }));
    expect(screen.getByRole("heading", { name: "Recent Projects" })).toBeTruthy();
    expect(screen.queryByText("Book 6")).toBeNull();
    expect(invoke).toHaveBeenCalledWith("get_all_projects");
  });

  it("shows all seven RPG content import choices instead of manuscript importers", async () => {
    render(StartScreen, { recentProjects: [], onNewProject: vi.fn() });
    await fireEvent.click(screen.getByRole("button", { name: /Import RPG Content/ }));
    const dialog = screen.getByRole("dialog", { name: "Import RPG Content" });
    for (const label of ["Markdown", "Obsidian", "Images", "Foundry", "Roll20", "Fantasy Grounds", "PDF"]) {
      expect(within(dialog).getByRole("button", { name: new RegExp(label) })).toBeTruthy();
    }
    expect(within(dialog).queryByText(/Import chapters, scenes/)).toBeNull();
    expect(within(dialog).queryByRole("button", { name: /Scrivener/ })).toBeNull();
  });

  it("explains the empty library", () => {")) })).toBeTruthy();
    }
    expect(within(dialog).queryByText(/Import chapters, scenes/)).toBeNull();
    expect(within(dialog).queryByRole("button", { name: /Scrivener/ })).toBeNull();
  });

  it("explains the empty library", () => {
    render(StartScreen, { recentProjects: [] });
    expect(screen.getByText("Your projects will appear here")).toBeTruthy();
    expect(screen.queryByTestId("recent-projects")).toBeNull();
    expect(screen.getByRole("button", { name: /Import RPG Content/ })).toBeTruthy();
  });

  it("shows an open failure beside the list and keeps the row", async () => {
    vi.mocked(invoke).mockImplementation(async (command) => { if (command === "get_project") throw "The project file could not be read."; return { word_count: 0, word_target: null, chapter_count: 0, scene_count: 0 }; });
    render(StartScreen, { recentProjects: [project()] });

    await fireEvent.click(screen.getByTestId("project-card"));

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain('Could not open "The Letter"');
    expect(alert.textContent).toContain("The project file could not be read.");
    expect(screen.getByTestId("project-card")).toBeTruthy();
    expect(currentProject.value).toBeNull();
  });

  it("marks the row busy while a project opens", async () => {
    let resolve!: (value: Project) => void;
    vi.mocked(invoke).mockImplementation((command) => command === "get_project" ? new Promise<Project>((r) => (resolve = r)) : Promise.resolve({ word_count: 0, word_target: null, chapter_count: 0, scene_count: 0 }));
    render(StartScreen, { recentProjects: [project()] });

    await fireEvent.click(screen.getByTestId("project-card"));
    expect(within(screen.getByTestId("project-card")).getByText("Opening...")).toBeTruthy();
    expect(screen.getByTestId("project-card").closest("li")?.getAttribute("aria-busy")).toBe(
      "true"
    );

    resolve(project());
    await waitFor(() => expect(currentProject.value?.id).toBe("project-1"));
    expect(ui.currentView).toBe("editor");
  });

  it("spells source tools the way they spell themselves", () => {
    render(StartScreen, {
      recentProjects: [
        project({ id: "a", source_type: "YWriter" }),
        project({ id: "b", source_type: "NovelWriter" }),
      ],
    });
    const rows = screen.getAllByTestId("project-card").map((row) => row.textContent);
    expect(rows[0]).toContain("The Letter");
    expect(rows[1]).toContain("The Letter");
  });

  it("switches to all projects only after they load", async () => {
    vi.mocked(invoke).mockImplementation(async (command) => { if (command === "get_all_projects") throw "offline"; return { word_count: 0, word_target: null, chapter_count: 0, scene_count: 0 }; });
    render(StartScreen, { recentProjects: many(6) });
    await fireEvent.click(screen.getByRole("button", { name: "View All" }));
    await waitFor(() => expect(ui.toast?.message).toContain("Failed to load projects"));
    expect(screen.getByRole("heading", { name: "Recent Projects" })).toBeTruthy();
  });

  it("names the consequence when confirming a delete", async () => {
    render(StartScreen, { recentProjects: [project()] });
    await fireEvent.click(screen.getByRole("button", { name: "Remove The Letter from Catalyst" }));
    expect(screen.getByText('Remove "The Letter" from Catalyst?')).toBeTruthy();
    expect(screen.getByTestId("dialog-message").textContent).toContain("local files remain untouched");
  });
});
