// @vitest-environment jsdom

import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import TerminalWorkspace from "./TerminalWorkspace.svelte";
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
  vi.mocked(invoke).mockResolvedValue({
    command: "git status",
    exit_code: 0,
    stdout: "On branch main",
    stderr: "",
  });
});

describe("TerminalWorkspace", () => {
  it("runs commands in the active project folder", async () => {
    render(TerminalWorkspace, { props: { project } });
    await fireEvent.input(screen.getByRole("textbox", { name: "Terminal command" }), {
      target: { value: "git status" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Run command" }));

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("run_project_terminal_command", {
        projectRoot: project.project_root,
        command: "git status",
      })
    );
    expect(await screen.findByText("On branch main")).toBeTruthy();
  });
});
