// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import CampaignWizardDialog from "./CampaignWizardDialog.svelte";
import { currentProject } from "../stores/project.svelte";
import { workspace } from "../stores/workspace.svelte";
import { GUIDED_STEPS } from "../campaignWizard";

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
  overview_path: "Adventures/Main Campaign/Campaign Overview.md",
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

async function fillEssentials() {
  await type("Campaign name", "Ashes of Varn");
  await type("Campaign folder", "C:/Campaigns/Ashes of Varn");
}

describe("CampaignWizardDialog", () => {
  it("offers Quick Start, the Guided Campaign Builder and a Blank Campaign before any form", () => {
    render(CampaignWizardDialog, { props: { onClose: vi.fn() } });
    expect(screen.getByRole("dialog", { name: "New Campaign" })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Quick Start/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Guided Campaign Builder/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Blank Campaign/ })).toBeTruthy();
    expect(screen.queryByRole("textbox", { name: "Campaign name" })).toBeNull();
  });

  it("Quick Start creates a System-Neutral campaign, activates it and opens the wiki", async () => {
    const onClose = vi.fn();
    const onComplete = vi.fn();
    render(CampaignWizardDialog, { props: { onClose, onComplete } });
    await fireEvent.click(screen.getByRole("button", { name: /Quick Start/ }));
    expect((screen.getByRole("radio", { name: /System-Neutral/ }) as HTMLInputElement).checked).toBe(true);
    await fillEssentials();
    await type("Premise", "A border kingdom burns.");
    await fireEvent.click(screen.getByRole("button", { name: "Create Campaign" }));

    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(lastInput()).toEqual({
      project_root: "C:/Campaigns/Ashes of Varn",
      name: "Ashes of Varn",
      setup_mode: "quick-start",
      basics: { premise: "A border kingdom burns." },
      rules: { pack_id: "system-neutral", custom_system_name: "" },
    });
    expect(currentProject.value?.id).toBe(created.manifest.project_id);
    expect(currentProject.value?.project_type).toBe("campaign");
    expect(currentProject.value?.project_root).toBe(created.paths.root);
    expect(workspace.active).toBe("story-development");
    expect(onComplete).toHaveBeenCalledWith(expect.objectContaining({ id: created.manifest.project_id }));
  });

  it("Quick Start requires a name, a folder, and a name for a homebrew system", async () => {
    render(CampaignWizardDialog, { props: { onClose: vi.fn() } });
    await fireEvent.click(screen.getByRole("button", { name: /Quick Start/ }));
    const create = () => fireEvent.click(screen.getByRole("button", { name: "Create Campaign" }));
    await create();
    expect(screen.getByRole("alert").textContent).toContain("campaign name");
    await type("Campaign name", "Ashes of Varn");
    await create();
    expect(screen.getByRole("alert").textContent).toContain("folder");
    await type("Campaign folder", "C:/Campaigns/Ashes");
    await fireEvent.click(screen.getByRole("radio", { name: /Custom \/ Homebrew/ }));
    await create();
    expect(screen.getByRole("alert").textContent).toContain("homebrew");
    expect(invoke).not.toHaveBeenCalledWith("create_new_campaign", expect.anything());
  });

  it("Blank Campaign needs only a name and folder and defaults to System-Neutral", async () => {
    const onClose = vi.fn();
    render(CampaignWizardDialog, { props: { onClose } });
    await fireEvent.click(screen.getByRole("button", { name: /Blank Campaign/ }));
    expect(screen.queryByRole("textbox", { name: "Premise" })).toBeNull();
    await fillEssentials();
    await fireEvent.click(screen.getByRole("button", { name: "Create Campaign" }));
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(lastInput()).toEqual({
      project_root: "C:/Campaigns/Ashes of Varn",
      name: "Ashes of Varn",
      setup_mode: "blank",
      basics: { premise: "" },
      rules: { pack_id: "system-neutral", custom_system_name: "" },
    });
  });

  it("lets optional steps be skipped without losing answers", async () => {
    render(CampaignWizardDialog, { props: { onClose: vi.fn() } });
    await fireEvent.click(screen.getByRole("button", { name: /Guided Campaign Builder/ }));
    expect(screen.queryByRole("button", { name: /Skip this step/ })).toBeNull();
    await fillEssentials();
    await fireEvent.click(screen.getByRole("button", { name: /Next/ }));
    await fireEvent.click(screen.getByRole("radio", { name: /Milestones/ }));
    await fireEvent.click(screen.getByRole("button", { name: /Skip this step/ }));
    expect(screen.getByRole("heading", { level: 2, name: GUIDED_STEPS[2] })).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: new RegExp(GUIDED_STEPS[1]) }));
    expect((screen.getByRole("radio", { name: /Milestones/ }) as HTMLInputElement).checked).toBe(true);
  });

  it("shows backend errors and keeps the wizard open", async () => {
    vi.mocked(invoke).mockRejectedValueOnce("Choose an empty or new folder for the campaign.");
    const onClose = vi.fn();
    render(CampaignWizardDialog, { props: { onClose } });
    await fireEvent.click(screen.getByRole("button", { name: /Quick Start/ }));
    await fillEssentials();
    await fireEvent.click(screen.getByRole("button", { name: "Create Campaign" }));
    expect((await screen.findByRole("alert")).textContent).toContain("empty or new folder");
    expect(onClose).not.toHaveBeenCalled();
    expect(currentProject.value).toBeNull();
  });

  it("Guided Campaign Builder walks the ten steps and sends the full payload", async () => {
    render(CampaignWizardDialog, { props: { onClose: vi.fn() } });
    await fireEvent.click(screen.getByRole("button", { name: /Guided Campaign Builder/ }));
    const next = () => fireEvent.click(screen.getByRole("button", { name: /Next/ }));
    const heading = (name: string) => expect(screen.getByRole("heading", { level: 2, name })).toBeTruthy();
    const pick = (name: RegExp | string) => fireEvent.click(screen.getByRole("radio", { name }));
    const check = (name: RegExp | string) => fireEvent.click(screen.getByRole("checkbox", { name }));

    heading(GUIDED_STEPS[0]);
    await next();
    expect(screen.getByRole("alert").textContent).toContain("campaign name");
    await fillEssentials();
    await pick("Dark Fantasy");
    await check("Dark / Gritty");
    await check("Political / Intrigue");
    await type("Premise", "A border kingdom burns.");
    await fireEvent.input(screen.getByLabelText("Campaign start date"), { target: { value: "2026-12-05" } });
    await next();

    heading(GUIDED_STEPS[1]);
    await pick(/Custom \/ Homebrew.*Your own game/);
    await next();
    expect(screen.getByRole("alert").textContent).toContain("homebrew");
    await type("System name", "Ember d6");
    await type("Edition", "Second printing");
    await pick("Tier");
    await type("Starting tier", "Tier 2");
    await pick("Milestones");
    await pick(/Standard Rules with Modifications/);
    await type("House rules", "Critical fumbles off.");
    await type("Permitted content", "Core book, Ember Companion");
    await next();

    heading(GUIDED_STEPS[2]);
    expect((screen.getByRole("radio", { name: /Use an Existing World/ }) as HTMLInputElement).disabled).toBe(true);
    expect((screen.getByRole("radio", { name: /Import a World/ }) as HTMLInputElement).disabled).toBe(true);
    await type("World name", "The Ember Marches");
    await pick(/Original \/ Homebrew/);
    await pick("Region");
    await pick("Medieval");
    await type("What makes this setting distinctive", "Ash falls like snow.");
    await next();

    heading(GUIDED_STEPS[3]);
    expect(screen.queryByRole("textbox", { name: "What is the general climate?" })).toBeNull();
    await check("Geography & Climate");
    await type("What is the general climate?", "Cold and ashen");
    await check("Settlements & Locations");
    await fireEvent.click(screen.getByRole("button", { name: /Add settlement/ }));
    await next();
    expect(screen.getByRole("alert").textContent).toContain("settlement");
    await type("Settlement 1 name", "Cinderford");
    await type("Settlement 1 What purpose does it serve?", "River trade");
    await check("Religions & Deities");
    await fireEvent.click(screen.getByRole("button", { name: /Add faith/ }));
    await type("Faith 1 name", "The Ash Mother");
    await next();

    heading(GUIDED_STEPS[4]);
    expect(screen.getByText(/placeholder record, not a character sheet/)).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: /Add player/ }));
    await type("Player 1 name", "Sam");
    await type("Player 1 character", "Kestrel");
    await next();

    heading(GUIDED_STEPS[5]);
    await fireEvent.click(screen.getByRole("button", { name: /Add NPC/ }));
    await next();
    expect(screen.getByRole("alert").textContent).toContain("NPC needs a name");
    await type("NPC 1 name", "The Ember Queen");
    await fireEvent.change(screen.getByRole("combobox", { name: "NPC 1 disposition" }), { target: { value: "villain" } });
    await fireEvent.click(screen.getByRole("button", { name: /Add creature/ }));
    await type("Creature 1 name", "Cinder Hound");
    await fireEvent.click(screen.getByRole("button", { name: /Add faction/ }));
    await type("Faction 1 name", "Ash Council");
    await type("Faction 1 goal", "Rule the Marches");
    await next();

    heading(GUIDED_STEPS[6]);
    await type("Starting adventure", "Smoke on the Horizon");
    await type("Side quests", "A burned map\nA missing heir\n");
    await next();

    heading(GUIDED_STEPS[7]);
    await type("Starting location", "Cinderford");
    await fireEvent.click(screen.getByRole("button", { name: /Add location/ }));
    await next();
    expect(screen.getByRole("alert").textContent).toContain("location needs a name");
    await type("Location 1 name", "Cinderford");
    await next();

    heading(GUIDED_STEPS[8]);
    await type("First session title", "Into the Smoke");
    await fireEvent.change(screen.getByRole("combobox", { name: "Tabletop style" }), { target: { value: "hybrid" } });
    await fireEvent.click(screen.getByRole("checkbox", { name: "Use a player display" }));
    await next();

    heading(GUIDED_STEPS[9]);
    const planned = within(screen.getByRole("list", { name: "Planned files" }));
    for (const file of [
      "campaign.yaml",
      "Worldbuilding/Religions & Deities/The Ash Mother.md",
      "Worldbuilding/Geography & Climate/Geography & Climate Overview.md",
      "Characters/Player Characters/Kestrel.md",
      "Characters/Non-Player Characters/The Ember Queen.md",
      "Worldbuilding/Settlements & Locations/Cinderford.md",
      "Sessions/Session Plans/Into the Smoke.md",
    ]) {
      expect(planned.getByText(file)).toBeTruthy();
    }
    // A settlement from step 4 and a location from step 8 with the same name get numbered files.
    expect(planned.getAllByText("Worldbuilding/Settlements & Locations/Cinderford.md")).toHaveLength(1);
    expect(planned.getByText("Worldbuilding/Settlements & Locations/Cinderford 2.md")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Create Campaign" }));
    await waitFor(() => expect(currentProject.value?.id).toBe(created.manifest.project_id));

    const input = lastInput();
    expect(input.setup_mode).toBe("guided");
    expect(input.basics).toMatchObject({ genre: "Dark Fantasy", tone: "Dark / Gritty, Political / Intrigue", start_date: "2026-12-05" });
    expect(input.rules).toMatchObject({
      pack_id: "custom-homebrew",
      custom_system_name: "Ember d6",
      edition: "Second printing",
      progression_type: "tier",
      starting_level: "Tier 2",
      advancement: "Milestones",
      house_rules_mode: "modified",
      house_rules: "Critical fumbles off.",
      allowed_content: ["Core book", "Ember Companion"],
    });
    expect(input.world).toMatchObject({ mode: "new", name: "The Ember Marches", origin: "original", scale: "Region", era: "Medieval", notes: "Ash falls like snow." });
    expect(input.worldbuilding).toEqual([
      { id: "geography", answers: [{ question: "What is the general climate?", answer: "Cold and ashen" }], entries: [], notes: "" },
      { id: "settlements", answers: [], entries: [{ name: "Cinderford", kind: "", description: "**What purpose does it serve?** River trade" }], notes: "" },
      { id: "religions", answers: [], entries: [{ name: "The Ash Mother", kind: "", description: "" }], notes: "" },
    ]);
    expect(input.party.players).toEqual([{ player_name: "Sam", character_name: "Kestrel" }]);
    expect(input.npcs[0]).toMatchObject({ name: "The Ember Queen", disposition: "villain" });
    expect(input.creatures).toEqual([{ name: "Cinder Hound", kind: "", description: "" }]);
    expect(input.factions).toEqual([{ name: "Ash Council", kind: "Rule the Marches", description: "" }]);
    expect(input.story).toMatchObject({ starting_adventure: "Smoke on the Horizon", side_quests: ["A burned map", "A missing heir"] });
    expect(input.locations).toMatchObject({ starting_location: "Cinderford", entries: [{ name: "Cinderford", kind: "", description: "" }] });
    expect(input.session).toMatchObject({ first_session_title: "Into the Smoke", play_mode: "hybrid", dice: "physical", player_display: true });
  });

  it("keeps entered values when moving back and forth between steps", async () => {
    render(CampaignWizardDialog, { props: { onClose: vi.fn() } });
    await fireEvent.click(screen.getByRole("button", { name: /Guided Campaign Builder/ }));
    await fillEssentials();
    await fireEvent.click(screen.getByRole("button", { name: /Next/ }));
    await fireEvent.click(screen.getByRole("button", { name: /Back/ }));
    expect((screen.getByRole("textbox", { name: "Campaign name" }) as HTMLInputElement).value).toBe("Ashes of Varn");
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
