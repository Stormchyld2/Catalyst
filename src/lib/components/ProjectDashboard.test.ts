// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import ProjectDashboard from "./ProjectDashboard.svelte";
import type { Project } from "../types";

const campaign: Project = {
  id: "campaign-1", name: "The Hollow Crown", source_type: "Blank",
  source_path: null, project_root: "C:/Campaigns/The Hollow Crown",
  created_at: "2026-11-23T12:00:00Z", modified_at: "2026-11-23T12:00:00Z",
  author_pen_name: null, genre: "Fantasy mystery", description: "Level one adventure",
  word_target: null, reference_types: [], project_type: "campaign", target_page_count: null,
};

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockResolvedValue({
    ruleset: "D&D 5e", edition: "2014", starting_level: "1",
    party_name: "The Bellward Company", expected_players: "4", play_mode: "physical",
    session_schedule: null, sessions_planned: 2, sessions_logged: 0,
    adventure_records: 7, encounters: 3, npcs: 5, world_entries: 6,
    latest_session_plan: "Session 1 - The Silent Bell",
  });
});
afterEach(cleanup);

describe("ProjectDashboard", () => {
  it("reads and shows actual campaign summary instead of writing goals", async () => {
    render(ProjectDashboard, { props: { project: campaign,
      onOpenWrite: vi.fn(), onOpenStoryDevelopment: vi.fn(), onOpenAdventures: vi.fn() } });
    expect(await screen.findByText(/D&D 5e/)).toBeTruthy();
    expect(screen.getByText("The Bellward Company · 4 expected")).toBeTruthy();
    expect(screen.getByText("Session Plans")).toBeTruthy();
    expect(screen.getByText("Worldbuilding Records")).toBeTruthy();
    expect(screen.queryByText(/manuscript word target|Writing Activity|Book brief/)).toBeNull();
    expect(invoke).toHaveBeenCalledWith("get_campaign_dashboard_summary", { projectRoot: campaign.project_root });
  });

  it("opens Campaign Notes, World Builder and Adventures", async () => {
    const onOpenWrite = vi.fn(), onOpenStoryDevelopment = vi.fn(), onOpenAdventures = vi.fn();
    render(ProjectDashboard, { props: { project: campaign,
      onOpenWrite, onOpenStoryDevelopment, onOpenAdventures } });
    await screen.findByText("Session Plans");
    await fireEvent.click(screen.getByRole("button", { name: /Open Campaign Notes/ }));
    await fireEvent.click(screen.getByRole("button", { name: /Open World Builder/ }));
    await fireEvent.click(screen.getByRole("button", { name: /Open Adventures/ }));
    expect(onOpenWrite).toHaveBeenCalled();
    expect(onOpenStoryDevelopment).toHaveBeenCalledOnce();
    expect(onOpenAdventures).toHaveBeenCalledOnce();
  });

  it("fails visibly on an invalid campaign rather than showing invented numbers", async () => {
    vi.mocked(invoke).mockRejectedValue("Missing campaign.yaml");
    render(ProjectDashboard, { props: { project: campaign,
      onOpenWrite: vi.fn(), onOpenStoryDevelopment: vi.fn(), onOpenAdventures: vi.fn() } });
    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("Missing campaign.yaml"));
  });
});
