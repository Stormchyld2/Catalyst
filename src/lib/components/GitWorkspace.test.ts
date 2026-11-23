// @vitest-environment jsdom

import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import GitWorkspace from "./GitWorkspace.svelte";
import type { Project } from "../types";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const project: Project = {
  id: "p1",
  name: "Test Novel",
  source_type: "Blank",
  source_path: null,
  project_root: "C:/Books/Test Novel",
  created_at: "2026-09-30T00:00:00Z",
  modified_at: "2026-09-30T00:00:00Z",
  author_pen_name: null,
  genre: null,
  description: null,
  word_target: null,
  reference_types: [],
  project_type: "novel",
  target_page_count: null,
};

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "get_git_workspace_status")
      return {
        root: project.project_root,
        is_repository: true,
        branch: "main",
        remote: "https://github.com/example/test.git",
        changes: [" M Characters/Test.md"],
      };
    return { command, exit_code: 0, stdout: "ok", stderr: "" };
  });
});

describe("GitWorkspace", () => {
  it("shows repository state and runs pull", async () => {
    render(GitWorkspace, { props: { project } });
    expect(await screen.findByText("main")).toBeTruthy();
    expect(screen.getByText(/Characters\/Test.md/)).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: /Pull/ }));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("git_pull", { projectRoot: project.project_root })
    );
  });
});
