// @vitest-environment jsdom

import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import GmScreenWorkspace from "./GmScreenWorkspace.svelte";
import type { Project } from "../types";
import { defaultLayout, type SoundboardLayout } from "../soundboard.svelte";
import { soundboard } from "../soundboardBoard.svelte";
import { diceTray } from "../dice";
import type { GmScreenLayout } from "../gmScreen";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const project = { id: "p1", name: "Vael", project_root: "C:/Campaigns/Vael", project_type: "campaign" } as Project;

let board: SoundboardLayout;
let screenFile: GmScreenLayout | null;

beforeEach(async () => {
  await soundboard.close();
  diceTray.history.clear();
  board = defaultLayout();
  board.pads = [
    { id: "pad-1", label: "Battle drums", file: "drums.ogg", channel: "music", loop: true, volume: 1, tags: ["Combat"] },
    { id: "pad-2", label: "Tavern", file: "tavern.ogg", channel: "ambient", loop: true, volume: 1, tags: ["Town"] },
  ];
  screenFile = null;
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    const a = args as Record<string, unknown>;
    switch (command) {
      case "load_soundboard_layout":
        return structuredClone(board);
      case "list_soundboard_audio":
        return [{ path: "drums.ogg", name: "drums.ogg", size: 1 }, { path: "tavern.ogg", name: "tavern.ogg", size: 1 }];
      case "save_soundboard_layout":
        board = structuredClone(a.layout as SoundboardLayout);
        return board;
      case "load_gm_screen":
        return screenFile && structuredClone(screenFile);
      case "save_gm_screen":
        screenFile = structuredClone(a.layout as GmScreenLayout);
        return null;
      default:
        throw new Error(`unexpected ${command}`);
    }
  });
});

describe("GmScreenWorkspace", () => {
  it("shows the chosen pad set and saves the choice in gm-screen.json", async () => {
    render(GmScreenWorkspace, { project });
    const pads = screen.getByRole("region", { name: "Sound pads" });
    await within(pads).findByRole("button", { name: /Battle drums/ });
    expect(within(pads).getByRole("button", { name: /Tavern/ })).toBeTruthy();

    await fireEvent.change(screen.getByLabelText("Pad set"), { target: { value: "Combat" } });
    expect(within(pads).queryByRole("button", { name: /Tavern/ })).toBeNull();
    await waitFor(() => expect(screenFile?.layouts.default.panels.find((p) => p.id === "soundboard")?.pad_tag).toBe("Combat"));
  });

  it("opens every pad in a modal", async () => {
    screenFile = {
      format: "catalyst-gm-screen",
      version: 1,
      active_layout: "default",
      layouts: { default: { panels: [{ id: "soundboard", visible: true, pad_tag: "Combat" }] } },
    };
    render(GmScreenWorkspace, { project });
    await screen.findByRole("button", { name: /Battle drums/ });
    expect(screen.queryByRole("button", { name: /Tavern/ })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "All pads" }));
    const dialog = screen.getByRole("dialog", { name: "All pads" });
    expect(within(dialog).getByRole("button", { name: /Tavern/ })).toBeTruthy();
  });

  it("writes master volume to the campaign soundboard, not the screen file", async () => {
    render(GmScreenWorkspace, { project });
    await screen.findByRole("button", { name: /Battle drums/ });
    await fireEvent.input(screen.getByLabelText("Master volume"), { target: { value: "0.3" } });
    await waitFor(() => expect(board.master_volume).toBe(0.3), { timeout: 2000 });
    expect(screenFile).toBeNull();
  });

  it("keeps the latest roll frozen above the history", async () => {
    render(GmScreenWorkspace, { project });
    await fireEvent.click(screen.getByRole("button", { name: "Roll d6" }));
    await waitFor(() => expect(screen.getByTestId("gm-current-roll")).toBeTruthy());
    expect(screen.queryAllByTestId("gm-roll")).toHaveLength(0);
    await fireEvent.click(screen.getByRole("button", { name: "Roll d20" }));
    await waitFor(() => expect(screen.queryAllByTestId("gm-roll")).toHaveLength(1));
    expect(screen.getByTestId("gm-current-roll").textContent).toContain("1d20");
    expect(diceTray.history.all[0].source).toBe("gm-console");
  });

  it("hides a panel and remembers it", async () => {
    render(GmScreenWorkspace, { project });
    await screen.findByRole("button", { name: /Battle drums/ });
    await fireEvent.click(screen.getByRole("button", { name: "Dice" }));
    expect(screen.queryByRole("button", { name: "Roll d6" })).toBeNull();
    await waitFor(() => expect(screenFile?.layouts.default.panels.find((p) => p.id === "dice")?.visible).toBe(false));
  });
});
