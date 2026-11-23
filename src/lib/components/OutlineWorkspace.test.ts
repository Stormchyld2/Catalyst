// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import OutlineWorkspace from "./OutlineWorkspace.svelte";
import type { Project } from "../types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const project = {
  id: "11111111-1111-4111-8111-111111111111",
  name: "Test Novel",
  source_type: "Blank",
  source_path: null,
  project_root: "C:/Temp/LithoMark/Test Novel",
  created_at: "2026-10-13T12:00:00Z",
  modified_at: "2026-10-13T12:00:00Z",
  author_pen_name: null,
  genre: null,
  description: null,
  word_target: null,
  reference_types: [],
  project_type: "novel",
  target_page_count: null,
} as Project;

const snapshot = {
  source_hash: "outline-hash",
  outline: {
    version: 1,
    plotlines: [{ id: "plot-1", title: "Main Plot" }],
    chapters: [
      {
        id: "chapter-1",
        title: "Chapter One",
        scenes: [
          {
            id: "scene-1",
            title: "Opening Scene",
            synopsis: "The protagonist discovers the first clue.",
            plotline_ids: ["plot-1"],
            beats: [
              { id: "beat-1", content: "A strange letter arrives." },
              { id: "beat-2", content: "The seal is broken." },
            ],
          },
        ],
      },
      {
        id: "chapter-2",
        title: "Chapter Two",
        scenes: [],
      },
    ],
  },
};

beforeEach(() => {
  vi.mocked(invoke).mockReset();
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("OutlineWorkspace", () => {
  it("loads the registered file-backed outline and renders chapters and scenes", async () => {
    vi.mocked(invoke).mockResolvedValue(snapshot);

    render(OutlineWorkspace, { project });

    expect(await screen.findByRole("heading", { name: "Chapter One" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Chapter Two" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Opening Scene" })).toBeTruthy();
    expect(screen.getByLabelText("Main Plot, Chapter Two")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("get_file_outline", { projectId: project.id });
  });

  it("places unassigned scenes in a visible fallback lane", async () => {
    const unassigned = structuredClone(snapshot);
    unassigned.outline.plotlines = [];
    unassigned.outline.chapters[0].scenes[0].plotline_ids = [];
    vi.mocked(invoke).mockResolvedValue(unassigned);

    render(OutlineWorkspace, { project });

    expect(await screen.findByText("Unassigned")).toBeTruthy();
    expect(screen.getByLabelText("Unassigned, Chapter One")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Opening Scene" })).toBeTruthy();
  });

  it("shows scene synopsis and beats after a scene is selected", async () => {
    vi.mocked(invoke).mockResolvedValue(snapshot);

    render(OutlineWorkspace, { project });
    const sceneButton = await screen.findByRole("button", { name: "Opening Scene" });
    await fireEvent.click(sceneButton);

    expect(screen.getByRole("heading", { name: "Opening Scene", level: 2 })).toBeTruthy();
    const details = screen.getByLabelText("Scene details");
    expect(details.textContent).toContain("The protagonist discovers the first clue.");
    expect(screen.getByText("A strange letter arrives.")).toBeTruthy();
    expect(screen.getByText("The seal is broken.")).toBeTruthy();
    expect(sceneButton.getAttribute("aria-pressed")).toBe("true");
  });

  it("persists plotline renames using the source hash", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(snapshot).mockResolvedValueOnce({
      ...snapshot,
      source_hash: "updated-hash",
      outline: { ...snapshot.outline, plotlines: [{ id: "plot-1", title: "Mystery Thread" }] },
    });

    render(OutlineWorkspace, { project });
    await fireEvent.click(await screen.findByRole("button", { name: "Rename Main Plot" }));
    await fireEvent.input(screen.getByRole("textbox", { name: "Rename plotline" }), {
      target: { value: "Mystery Thread" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save name" }));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("save_file_outline", {
      projectId: project.id,
      expectedHash: "outline-hash",
      outline: { ...snapshot.outline, plotlines: [{ id: "plot-1", title: "Mystery Thread" }] },
    }));
    expect(await screen.findByText("Mystery Thread")).toBeTruthy();
  });

  it("persists scene plotline removal without changing scene identity or beats", async () => {
    const updated = structuredClone(snapshot);
    updated.outline.chapters[0].scenes[0].plotline_ids = [];
    vi.mocked(invoke).mockResolvedValueOnce(snapshot).mockResolvedValueOnce(updated);

    render(OutlineWorkspace, { project });
    await fireEvent.click(await screen.findByRole("button", { name: "Opening Scene" }));
    await fireEvent.click(screen.getByRole("checkbox", { name: "Main Plot" }));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("save_file_outline", {
      projectId: project.id,
      expectedHash: "outline-hash",
      outline: updated.outline,
    }));
    expect(await screen.findByText("Unassigned")).toBeTruthy();
  });

  it("creates a plotline with a stable UUID and guarded save", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(snapshot).mockImplementationOnce(async (_command, args) => ({
      source_hash: "next-hash",
      outline: (args as { outline: typeof snapshot.outline }).outline,
    }));
    render(OutlineWorkspace, { project });
    await screen.findByRole("heading", { name: "Chapter One" });
    await fireEvent.input(screen.getByRole("textbox", { name: "New plotline title" }), {
      target: { value: "Secondary Arc" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Add plotline" }));
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(2));
    const [command, args] = vi.mocked(invoke).mock.calls[1];
    expect(command).toBe("save_file_outline");
    const payload = args as { projectId: string; expectedHash: string; outline: typeof snapshot.outline };
    expect(payload.projectId).toBe(project.id);
    expect(payload.expectedHash).toBe("outline-hash");
    expect(payload.outline.plotlines[1].title).toBe("Secondary Arc");
    expect(payload.outline.plotlines[1].id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
  });

  it("saves synopsis edits while preserving scene IDs and beats", async () => {
    const updated = structuredClone(snapshot);
    updated.source_hash = "synopsis-hash";
    updated.outline.chapters[0].scenes[0].synopsis = "A revised opening.";
    vi.mocked(invoke).mockResolvedValueOnce(snapshot).mockResolvedValueOnce(updated);
    render(OutlineWorkspace, { project });
    await fireEvent.click(await screen.findByRole("button", { name: "Opening Scene" }));
    await fireEvent.click(screen.getByRole("button", { name: "Edit synopsis" }));
    await fireEvent.input(screen.getByRole("textbox", { name: "Scene synopsis" }), {
      target: { value: "A revised opening." },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save synopsis" }));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("save_file_outline", {
      projectId: project.id,
      expectedHash: "outline-hash",
      outline: updated.outline,
    }));
    expect(screen.getByLabelText("Scene details").textContent).toContain("A revised opening.");
  });

  it("retains original synopsis and shows an error on rejected save", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(snapshot).mockRejectedValueOnce("Outline changed on disk.");
    render(OutlineWorkspace, { project });
    await fireEvent.click(await screen.findByRole("button", { name: "Opening Scene" }));
    await fireEvent.click(screen.getByRole("button", { name: "Edit synopsis" }));
    await fireEvent.input(screen.getByRole("textbox", { name: "Scene synopsis" }), {
      target: { value: "Unsaved revision." },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save synopsis" }));
    expect((await screen.findByRole("alert")).textContent).toContain("Outline changed on disk.");
    expect(screen.getByRole("textbox", { name: "Scene synopsis" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Opening Scene" }).textContent).toContain("The protagonist discovers the first clue.");
  });

  it("reorders scenes only within their chapter while retaining identities and beats", async () => {
    const twoScenes = structuredClone(snapshot);
    twoScenes.outline.chapters[0].scenes.push({
      id: "scene-2", title: "Second Scene", synopsis: "Next clue.",
      plotline_ids: ["plot-1"], beats: [],
    });
    const reordered = structuredClone(twoScenes);
    reordered.source_hash = "reordered-hash";
    reordered.outline.chapters[0].scenes.reverse();
    vi.mocked(invoke).mockResolvedValueOnce(twoScenes).mockResolvedValueOnce(reordered);
    render(OutlineWorkspace, { project });
    await fireEvent.click(await screen.findByRole("button", { name: "Second Scene" }));
    expect((screen.getByRole("button", { name: "Move scene down" }) as HTMLButtonElement).disabled).toBe(true);
    await fireEvent.click(screen.getByRole("button", { name: "Move scene up" }));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("save_file_outline", {
      projectId: project.id,
      expectedHash: "outline-hash",
      outline: reordered.outline,
    }));
    expect(screen.getByLabelText("Scene details").textContent).toContain("1 of 2");
    expect(reordered.outline.chapters[0].scenes[1].beats).toEqual(twoScenes.outline.chapters[0].scenes[0].beats);
    expect(reordered.outline.chapters[1]).toEqual(twoScenes.outline.chapters[1]);
  });

  it("does not submit an invalid move beyond the chapter boundary", async () => {
    vi.mocked(invoke).mockResolvedValue(snapshot);
    render(OutlineWorkspace, { project });
    await fireEvent.click(await screen.findByRole("button", { name: "Opening Scene" }));
    expect((screen.getByRole("button", { name: "Move scene up" }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole("button", { name: "Move scene down" }) as HTMLButtonElement).disabled).toBe(true);
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("offers explicit initialization when no outline exists and adopts the returned snapshot", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce(snapshot);

    render(OutlineWorkspace, { project });

    const initialize = await screen.findByRole("button", { name: "Initialize from Write" });
    expect(screen.getByText(/reuse existing chapter, scene, and beat identities/i)).toBeTruthy();

    await fireEvent.click(initialize);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("initialize_file_outline", { projectId: project.id });
      expect(screen.getByRole("heading", { name: "Chapter One" })).toBeTruthy();
    });
  });

  it("surfaces backend load failures without inventing outline data", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("Outline file could not be read."));

    render(OutlineWorkspace, { project });

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Outline file could not be read.");
    expect(screen.queryByRole("heading", { name: "Chapter One" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Initialize from Write" })).toBeNull();
  });

  it("refreshes through the same file-backed command", async () => {
    vi.mocked(invoke).mockResolvedValue(snapshot);

    render(OutlineWorkspace, { project });
    await screen.findByRole("heading", { name: "Chapter One" });
    await fireEvent.click(screen.getByRole("button", { name: "Refresh" }));

    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(2));
    expect(invoke).toHaveBeenNthCalledWith(2, "get_file_outline", { projectId: project.id });
  });
});
