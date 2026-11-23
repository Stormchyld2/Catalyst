// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import CampaignWizardDialog from "./CampaignWizardDialog.svelte";
import { currentProject } from "../stores/project.svelte";
import { workspace } from "../stores/workspace.svelte";
import { GM_TEMPLATES, GUIDED_STEPS } from "../campaignWizard";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("../utils/nativeDialog", () => ({ open: vi.fn(async () => "C:/Campaigns/Picked") }));
vi.mock("../stores/ui.svelte", () => ({ ui: { setView: vi.fn() } }));

const created = {
  manifest: {
    project_id: "22222222-2222-4222-8222-222222222222",
    title: "Ashes of Varn",
    project_type: "campaign",
    created_at: "2026-11-23T00:00:00Z",
  },
  paths: { root: "C:/Campaigns/Ashes of Varn" },
  overview_path: "Story Development/Campaign/Campaign Overview.md",
  overview_id: "note-33333333-3333-4333-8333-333333333333",
};

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (command) => (command === "create_new_campaign" ? created : null));
  currentProject.setProject(null);
  workspace.leaveProject();
});

afterEach(() => {
  currentProject.setProject(null);
  workspace.leaveProject();
});

function lastInput() {
  const call = vi.mocked(invoke).mock.calls.find(([command]) => command === "create_new_campaign");
  return (call?.[1] as { input: Record<string, any> }).input;
}

async function type(label: string, value: string) {
  await fireEvent.input(screen.getByRole("textbox", { name: label }), { target: { value } });
}

async function fillBasics() {
  await type("Campaign name", "Ashes of Varn");
  await type("Campaign folder", "C:/Campaigns/Ashes of Varn");
}

describe("CampaignWizardDialog", () => {
  it("offers Quick Start and Guided Setup before any form", () => {
    render(CampaignWizardDialog, { props: { onClose: vi.fn() } });
    expect(screen.getByRole("dialog", { name: "New Campaign" })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Quick Start/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Guided Setup/ })).toBeTruthy();
    expect(screen.queryByRole("textbox", { name: "Campaign name" })).toBeNull();
  });

  it("Quick Start creates a campaign, activates it and opens Story Development", async () => {
    const onClose = vi.fn();
    const onComplete = vi.fn();
    render(CampaignWizardDialog, { props: { onClose, onComplete } });
    await fireEvent.click(screen.getByRole("button", { name: /Quick Start/ }));
    await fillBasics();
    // A datalist-backed input is exposed as a combobox.
    await fireEvent.input(screen.getByRole("combobox", { name: "Game system" }), { target: { value: "Fifth Edition (5e)" } });
    await fireEvent.click(screen.getByRole("radio", { name: /Mystery & Intrigue/ }));
    await fireEvent.click(screen.getByRole("button", { name: "Create Campaign" }));

    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(lastInput()).toEqual({
      project_root: "C:/Campaigns/Ashes of Varn",
      name: "Ashes of Varn",
      setup_mode: "quick-start",
      game_system: "Fifth Edition (5e)",
      pitch: "",
      starter_id: "mystery",
      gm_template_ids: null,
    });
    expect(currentProject.value?.id).toBe(created.manifest.project_id);
    expect(currentProject.value?.project_type).toBe("campaign");
    expect(currentProject.value?.project_root).toBe(created.paths.root);
    expect(workspace.active).toBe("story-development");
    expect(onComplete).toHaveBeenCalledWith(expect.objectContaining({ id: created.manifest.project_id }));
  });

  it("Quick Start requires a name and folder and writes nothing without them", async () => {
    render(CampaignWizardDialog, { props: { onClose: vi.fn() } });
    await fireEvent.click(screen.getByRole("button", { name: /Quick Start/ }));
    await fireEvent.click(screen.getByRole("button", { name: "Create Campaign" }));
    expect(screen.getByRole("alert").textContent).toContain("campaign name");
    await type("Campaign name", "Ashes of Varn");
    await fireEvent.click(screen.getByRole("button", { name: "Create Campaign" }));
    expect(screen.getByRole("alert").textContent).toContain("folder");
    expect(invoke).not.toHaveBeenCalledWith("create_new_campaign", expect.anything());
  });

  it("shows backend errors and keeps the wizard open", async () => {
    vi.mocked(invoke).mockRejectedValueOnce("Choose an empty folder for the new campaign.");
    const onClose = vi.fn();
    render(CampaignWizardDialog, { props: { onClose } });
    await fireEvent.click(screen.getByRole("button", { name: /Quick Start/ }));
    await fillBasics();
    await fireEvent.click(screen.getByRole("button", { name: "Create Campaign" }));
    expect((await screen.findByRole("alert")).textContent).toContain("empty folder");
    expect(onClose).not.toHaveBeenCalled();
    expect(currentProject.value).toBeNull();
  });

  it("Guided Setup walks all ten steps and sends the full campaign payload", async () => {
    render(CampaignWizardDialog, { props: { onClose: vi.fn() } });
    await fireEvent.click(screen.getByRole("button", { name: /Guided Setup/ }));
    const next = () => fireEvent.click(screen.getByRole("button", { name: /Next/ }));
    const heading = (name: string) => expect(screen.getByRole("heading", { level: 2, name })).toBeTruthy();

    heading(GUIDED_STEPS[0]);
    await fillBasics();
    await type("Genre", "Dark fantasy");
    await next();

    heading(GUIDED_STEPS[1]);
    await type("Number of players", "0");
    await next();
    expect(screen.getByRole("alert").textContent).toContain("Player count");
    await type("Number of players", "5");
    await fireEvent.click(screen.getByRole("checkbox", { name: "X-Card" }));
    await next();

    heading(GUIDED_STEPS[2]);
    await next();
    expect(screen.getByRole("alert").textContent).toContain("world detail");
    await type("Setting", "The Ember Marches");
    await next();

    heading(GUIDED_STEPS[3]);
    await fireEvent.click(screen.getByRole("checkbox", { name: "Factions & Governments" }));
    await type("Factions & Governments names", "Ash Council, Free Blades");
    await next();

    heading(GUIDED_STEPS[4]);
    await fireEvent.click(screen.getByRole("button", { name: /Add location/ }));
    await next();
    expect(screen.getByRole("alert").textContent).toContain("location needs a name");
    await type("Location 1 name", "Cinderford");
    await next();

    heading(GUIDED_STEPS[5]);
    expect(screen.getByText(/Character Studio/)).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: /Add NPC/ }));
    await type("NPC 1 name", "The Ember Queen");
    await fireEvent.click(screen.getByRole("checkbox", { name: "NPC 1 is a villain" }));
    await next();

    heading(GUIDED_STEPS[6]);
    await type("Adventure hooks", "A burned map\nA missing heir\n");
    await fireEvent.click(screen.getByRole("radio", { name: /Dungeon Delve/ }));
    await next();

    heading(GUIDED_STEPS[7]);
    await type("Opening scene", "Smoke on the horizon.");
    await next();

    heading(GUIDED_STEPS[8]);
    await fireEvent.click(screen.getByRole("checkbox", { name: "Random Table" }));
    await type("House rules", "Critical fumbles off.");
    await next();

    heading(GUIDED_STEPS[9]);
    await fireEvent.click(screen.getByRole("button", { name: "Create Campaign" }));
    await waitFor(() => expect(currentProject.value?.id).toBe(created.manifest.project_id));

    const input = lastInput();
    expect(input.setup_mode).toBe("guided");
    expect(input.genre).toBe("Dark fantasy");
    expect(input.table.player_count).toBe(5);
    expect(input.table.safety_tools).toEqual(["X-Card"]);
    expect(input.world.setting).toBe("The Ember Marches");
    expect(input.world_categories).toEqual([{ id: "factions", notes: "", entries: ["Ash Council", "Free Blades"] }]);
    expect(input.locations).toEqual([{ name: "Cinderford", location_type: "", description: "" }]);
    expect(input.npcs[0]).toMatchObject({ name: "The Ember Queen", antagonist: true });
    expect(input.story.hooks).toEqual(["A burned map", "A missing heir"]);
    expect(input.starter_id).toBe("dungeon");
    expect(input.first_session.opening_scene).toBe("Smoke on the horizon.");
    expect(input.gm_template_ids).toHaveLength(GM_TEMPLATES.length - 1);
    expect(input.gm_template_ids).not.toContain("random-table");
    expect(input.house_rules).toBe("Critical fumbles off.");
  });

  it("Back from the first guided step returns to the mode picker", async () => {
    render(CampaignWizardDialog, { props: { onClose: vi.fn() } });
    await fireEvent.click(screen.getByRole("button", { name: /Guided Setup/ }));
    await fireEvent.click(screen.getByRole("button", { name: /Back/ }));
    expect(screen.getByRole("button", { name: /Quick Start/ })).toBeTruthy();
  });

  it("Browse fills the campaign folder from the native picker", async () => {
    render(CampaignWizardDialog, { props: { onClose: vi.fn() } });
    await fireEvent.click(screen.getByRole("button", { name: /Quick Start/ }));
    await fireEvent.click(screen.getByRole("button", { name: /Browse/ }));
    await waitFor(() =>
      expect((screen.getByRole("textbox", { name: "Campaign folder" }) as HTMLInputElement).value).toBe("C:/Campaigns/Picked")
    );
  });
});
