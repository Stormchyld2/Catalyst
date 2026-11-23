// @vitest-environment jsdom

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ProjectDashboard from "./ProjectDashboard.svelte";
import type { Project } from "../types";

afterEach(() => {
  cleanup();
});

const project: Project = {
  id: "project-1",
  name: "The Glass Meridian",
  source_type: "Blank",
  source_path: null,
  project_root: "C:/Books/The Glass Meridian",
  created_at: "2026-09-30T12:00:00Z",
  modified_at: "2026-09-30T20:00:00Z",
  author_pen_name: null,
  genre: "Speculative mystery",
  description: null,
  word_target: 88000,
  reference_types: [],
  project_type: "novel",
  target_page_count: null,
};

describe("ProjectDashboard", () => {
  it("shows the active project and its local project details", () => {
    render(ProjectDashboard, {
      props: {
        project,
        onOpenWrite: vi.fn(),
        onOpenStoryDevelopment: vi.fn(),
      },
    });

    expect(screen.getByRole("heading", { name: "The Glass Meridian" })).toBeTruthy();
    expect(screen.getByText(/Speculative mystery · Author not set · Updated/)).toBeTruthy();
    expect(screen.getByText("0% of 88,000 word goal")).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Book brief" })).toBeTruthy();
  });

  it("navigates directly to Write and Story Development", async () => {
    const onOpenWrite = vi.fn();
    const onOpenStoryDevelopment = vi.fn();

    render(ProjectDashboard, {
      props: { project, onOpenWrite, onOpenStoryDevelopment },
    });

    await fireEvent.click(screen.getByRole("button", { name: "Continue Writing" }));
    await fireEvent.click(screen.getByRole("button", { name: "Open Story Development" }));

    expect(onOpenWrite).toHaveBeenCalledTimes(1);
    expect(onOpenStoryDevelopment).toHaveBeenCalledTimes(1);
  });
});
