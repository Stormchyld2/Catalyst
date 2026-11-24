import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import ProjectSettings from "./ProjectSettings.svelte";
import { mockProject } from "../../dev/mock-data";

const campaignProject = {
  ...mockProject,
  project_type: "campaign",
  project_root: "/campaigns/The Hollow Crown",
  word_target: null,
  author_pen_name: null,
};
const loaded = {
  title: "The Hollow Crown", tone: "Mysterious", gm_name: "Demo GM",
  ruleset_name: "Dungeons & Dragons 5e", edition: "5e (2014)", starting_level: "1",
  advancement: "Milestone", play_mode: "physical", schedule: "", session_length: "",
  party_name: "The Bellward Company", expected_players: "4",
};

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === "get_campaign_settings") return { ...loaded };
    if (command === "update_campaign_settings") return (args as { settings: typeof loaded }).settings;
    if (command === "update_project_settings") return campaignProject;
    return [];
  });
});
afterEach(cleanup);

it("edits campaign configuration and saves it to campaign.yaml, not the overview", async () => {
  const onSave = vi.fn();
  render(ProjectSettings, { project: campaignProject, section: "details", onSave });
  await waitFor(() => expect((screen.getByLabelText("Party name") as HTMLInputElement).value).toBe("The Bellward Company"));
  expect(screen.queryByLabelText(/Pen Name/)).toBeNull();
  expect(screen.queryByLabelText(/Word Target/)).toBeNull();
  await fireEvent.input(screen.getByLabelText("Schedule"), { target: { value: "Weekly" } });
  await fireEvent.change(screen.getByLabelText("Play mode"), { target: { value: "hybrid" } });
  await fireEvent.click(screen.getByRole("button", { name: "Save campaign changes" }));
  await waitFor(() => expect(onSave).toHaveBeenCalled());
  expect(invoke).toHaveBeenCalledWith("update_campaign_settings", {
    projectRoot: campaignProject.project_root,
    settings: expect.objectContaining({ schedule: "Weekly", play_mode: "hybrid", party_name: "The Bellward Company" }),
  });
  expect(invoke).not.toHaveBeenCalledWith("get_writing_stats", expect.anything());
});

it("refuses an empty campaign name", async () => {
  render(ProjectSettings, { project: campaignProject, section: "details", onSave: vi.fn() });
  await waitFor(() => expect(screen.getByLabelText("Campaign name")).toBeTruthy());
  await fireEvent.input(screen.getByLabelText("Campaign name"), { target: { value: "  " } });
  await fireEvent.click(screen.getByRole("button", { name: "Save campaign changes" }));
  expect(await screen.findByRole("alert")).toBeTruthy();
  expect(invoke).not.toHaveBeenCalledWith("update_campaign_settings", expect.anything());
});
