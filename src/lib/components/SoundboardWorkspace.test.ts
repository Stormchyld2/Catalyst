// @vitest-environment jsdom

import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import SoundboardWorkspace from "./SoundboardWorkspace.svelte";
import type { Project } from "../types";
import { emptyLayout, type SoundboardLayout } from "../soundboard/layout";
import { soundboard } from "../soundboard/board.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

const project: Project = {
  id: "p1",
  name: "Shadows of Vael",
  source_type: "Blank",
  source_path: null,
  project_root: "C:/Campaigns/Shadows of Vael",
  created_at: "2026-11-23T00:00:00Z",
  modified_at: "2026-11-23T00:00:00Z",
  author_pen_name: null,
  genre: null,
  description: null,
  word_target: null,
  reference_types: [],
  project_type: "campaign",
  target_page_count: null,
};

let stored: SoundboardLayout;
let files: { path: string; name: string; size: number }[];

beforeEach(async () => {
  await soundboard.close();
  stored = emptyLayout();
  files = [
    { path: "Music/Tavern.ogg", name: "Tavern.ogg", size: 10 },
    { path: "door.wav", name: "door.wav", size: 5 },
  ];
  vi.mocked(open).mockReset();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    const a = args as Record<string, unknown>;
    switch (command) {
      case "load_soundboard_layout":
        return structuredClone(stored);
      case "list_soundboard_audio":
        return files;
      case "save_soundboard_layout":
        stored = structuredClone(a.layout as SoundboardLayout);
        return stored;
      case "import_soundboard_audio":
        files = [...files, { path: "Rain.mp3", name: "Rain.mp3", size: 3 }];
        return ["Rain.mp3"];
      default:
        throw new Error(`unexpected ${command}`);
    }
  });
});

describe("SoundboardWorkspace", () => {
  it("loads the campaign layout and audio list from the campaign folder", async () => {
    render(SoundboardWorkspace, { project });
    await screen.findByText(/2 audio files are not on the board yet/);
    expect(invoke).toHaveBeenCalledWith("load_soundboard_layout", { projectRoot: project.project_root });
    expect(invoke).toHaveBeenCalledWith("list_soundboard_audio", { projectRoot: project.project_root });
  });

  it("adds pads for unused files and saves the layout into the campaign", async () => {
    render(SoundboardWorkspace, { project });
    await fireEvent.click(await screen.findByRole("button", { name: "Add them all" }));
    expect(await screen.findByRole("button", { name: /^Tavern/ })).toBeTruthy();
    await waitFor(() => expect(stored.pads).toHaveLength(2), { timeout: 2000 });
    const music = stored.pads.find((p) => p.file === "Music/Tavern.ogg");
    expect(music).toMatchObject({ channel: "music", loop: true, label: "Tavern" });
    expect(stored.pads.find((p) => p.file === "door.wav")).toMatchObject({ channel: "effects", loop: false });
  });

  it("marks a pad whose file was removed from Assets/Audio", async () => {
    stored.pads = [{ id: "pad-1", label: "Gone", file: "gone.mp3", channel: "effects", loop: false, volume: 1 }];
    render(SoundboardWorkspace, { project });
    expect(await screen.findByText("missing file")).toBeTruthy();
    expect((screen.getByRole("button", { name: /^Gone/ }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("imports audio through the file picker and puts it on the board", async () => {
    vi.mocked(open).mockResolvedValue(["/home/gm/Rain.mp3"]);
    render(SoundboardWorkspace, { project });
    await screen.findByText(/not on the board yet/);
    await fireEvent.click(screen.getByRole("button", { name: /Import audio/ }));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("import_soundboard_audio", {
        projectRoot: project.project_root,
        sourcePaths: ["/home/gm/Rain.mp3"],
      }),
    );
    expect(await screen.findByRole("button", { name: /^Rain/ })).toBeTruthy();
    await waitFor(() => expect(stored.pads.map((p) => p.file)).toEqual(["Rain.mp3"]), { timeout: 2000 });
  });

  it("edits a pad's name and loop setting", async () => {
    stored.pads = [{ id: "pad-1", label: "Door", file: "door.wav", channel: "effects", loop: false, volume: 1 }];
    render(SoundboardWorkspace, { project });
    await fireEvent.click(await screen.findByRole("button", { name: "Edit Door" }));
    const name = screen.getByPlaceholderText("Tavern night") as HTMLInputElement;
    await fireEvent.input(name, { target: { value: "Front door" } });
    await fireEvent.click(screen.getByLabelText("Loop until stopped"));
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(stored.pads[0]).toMatchObject({ label: "Front door", loop: true }), { timeout: 2000 });
  });

  it("saves pad tags for GM Screen pad sets", async () => {
    stored.pads = [{ id: "pad-1", label: "Door", file: "door.wav", channel: "effects", loop: false, volume: 1 }];
    render(SoundboardWorkspace, { project });
    await fireEvent.click(await screen.findByRole("button", { name: "Edit Door" }));
    await fireEvent.input(screen.getByPlaceholderText("combat, tavern"), { target: { value: "Combat, tavern, combat" } });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(stored.pads[0].tags).toEqual(["Combat", "tavern"]), { timeout: 2000 });
  });

  it("keeps the board loaded when the workspace is closed and reopened", async () => {
    stored.pads = [{ id: "pad-1", label: "Door", file: "door.wav", channel: "effects", loop: false, volume: 1 }];
    const first = render(SoundboardWorkspace, { project });
    await screen.findByRole("button", { name: "Edit Door" });
    first.unmount();
    render(SoundboardWorkspace, { project });
    await screen.findByRole("button", { name: "Edit Door" });
    expect(vi.mocked(invoke).mock.calls.filter(([c]) => c === "load_soundboard_layout")).toHaveLength(1);
  });
});
