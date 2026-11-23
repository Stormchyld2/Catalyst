// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import NewBookFoundationDialog from "./NewBookFoundationDialog.svelte";
import { currentProject } from "../stores/project.svelte";
import { workspace } from "../stores/workspace.svelte";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../utils/nativeDialog", () => ({
  open: vi.fn(),
}));

vi.mock("../stores/ui.svelte", () => ({
  ui: {
    setView: vi.fn(),
  },
}));

const registry = {
  templates: [
    { id: "character-profile", name: "Character Profile", category: "character", document_type: "character", subtype: null, source: "built-in" },
    { id: "character-brief", name: "Character Brief", category: "character", document_type: "character", subtype: null, source: "user" },
    { id: "location-profile", name: "Location Profile", category: "location", document_type: "location", subtype: null, source: "built-in" },
    { id: "worldbuilding-profile", name: "Worldbuilding Profile", category: "worldbuilding", document_type: "note", subtype: "worldbuilding", source: "built-in" },
  ],
  issues: [],
  user_templates_dir: "C:/Temp/LithoMark/Templates",
};

const manuscriptTemplates = [
  {
    id: "bundled-three-act",
    name: "Three-Act Structure",
    description: "Setup, confrontation, and resolution.",
    source: "Classical Dramatic Theory",
    project_types: ["novel", "screenplay"],
    structure: [
      { title: "Act I — Setup", children: [] },
      { title: "Act II — Confrontation", children: [] },
      { title: "Act III — Resolution", children: [] },
    ],
  },
];

const created = {
  manifest: {
    project_id: "11111111-1111-4111-8111-111111111111",
    title: "Test Novel",
    project_type: "novel",
    created_at: "2026-09-30T00:00:00Z",
  },
  paths: {
    root: "C:/Temp/LithoMark/Test Novel",
  },
};

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "get_entity_template_registry") return registry;
    if (command === "get_bundled_templates") return manuscriptTemplates;
    if (command === "create_new_book_foundation") return created;
    return null;
  });
  currentProject.setProject(null);
  workspace.leaveProject();
});

afterEach(() => {
  vi.restoreAllMocks();
  cleanup();
  currentProject.setProject(null);
  workspace.leaveProject();
});

async function completeBasics() {
  await fireEvent.input(screen.getByRole("textbox", { name: "Book title" }), {
    target: { value: "Test Novel" },
  });
  await fireEvent.input(screen.getByRole("textbox", { name: "Project folder" }), {
    target: { value: "C:/Temp/LithoMark/Test Novel" },
  });
}

async function next() {
  // Worldbuilding is required; seed only the minimal synthetic test fixture.
  if (screen.queryByRole("heading", { name: "Worldbuilding", level: 2 })) {
    await fireEvent.input(screen.getByRole("textbox", { name: "Setting" }), {
      target: { value: "Synthetic test setting" },
    });
  }
  await fireEvent.click(screen.getByRole("button", { name: /Next/ }));
}

describe("NewBookFoundationDialog", () => {
  it("uses the ten-step New Book Foundation sequence", async () => {
    render(NewBookFoundationDialog, { props: { onClose: vi.fn() } });

    expect(screen.getByRole("navigation", { name: "New Book Foundation steps" })).toBeTruthy();
    expect(screen.getByText("Step 1 of 10")).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Project Basics", level: 2 })).toBeTruthy();

    await completeBasics();
    await next();

    expect(screen.getByText("Step 2 of 10")).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Characters", level: 2 })).toBeTruthy();
  });

  it("allows optional development steps to be skipped", async () => {
    render(NewBookFoundationDialog, { props: { onClose: vi.fn() } });
    await completeBasics();

    for (let i = 0; i < 7; i += 1) await next();

    expect(screen.getByText("Step 8 of 10")).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Synopsis", level: 2 })).toBeTruthy();
    await next();

    expect(screen.getByText("Step 9 of 10")).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Manuscript Structure", level: 2 })).toBeTruthy();
    await next();

    expect(screen.getByRole("heading", { name: "Review & Create", level: 2 })).toBeTruthy();
  });

  it("requires a worldbuilding detail before advancing", async () => {
    render(NewBookFoundationDialog, { props: { onClose: vi.fn() } });
    await completeBasics();
    await next(); // Characters
    await next(); // Locations
    await next(); // Worldbuilding

    await fireEvent.click(screen.getByRole("button", { name: /Next/ }));
    expect(screen.getByRole("alert").textContent).toContain("at least one worldbuilding detail");
    expect(screen.getByRole("heading", { name: "Worldbuilding", level: 2 })).toBeTruthy();

    await next();
    expect(screen.getByRole("heading", { name: "Style & Voice", level: 2 })).toBeTruthy();
  });

  it("validates named character rows before continuing", async () => {
    render(NewBookFoundationDialog, { props: { onClose: vi.fn() } });
    await completeBasics();
    await next();

    await fireEvent.click(screen.getByRole("button", { name: /Add character/ }));
    await next();

    expect(screen.getByRole("alert").textContent).toContain("needs a name");
    expect(screen.getByRole("heading", { name: "Characters", level: 2 })).toBeTruthy();
  });

  it("reviews and changes proposed entity templates before creating the book", async () => {
    render(NewBookFoundationDialog, { props: { onClose: vi.fn() } });
    await completeBasics();
    await next();

    await fireEvent.click(screen.getByRole("button", { name: /Add character/ }));
    await fireEvent.input(screen.getByRole("textbox", { name: "Character 1 name" }), {
      target: { value: "Mara Vale" },
    });

    for (let i = 0; i < 8; i += 1) await next();

    const selector = await screen.findByRole("combobox", { name: "Mara Vale template" });
    expect((selector as HTMLSelectElement).value).toBe("character-profile");

    await fireEvent.change(selector, { target: { value: "character-brief" } });
    await fireEvent.click(screen.getByRole("button", { name: "Create Book" }));

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith(
        "create_new_book_foundation",
        expect.objectContaining({
          input: expect.objectContaining({
            characters: [
              expect.objectContaining({
                name: "Mara Vale",
                template_id: "character-brief",
              }),
            ],
          }),
        })
      )
    );
  });

  it("passes the selected manuscript structure to the file-authoritative creator", async () => {
    render(NewBookFoundationDialog, { props: { onClose: vi.fn() } });
    await completeBasics();

    for (let i = 0; i < 8; i += 1) await next();

    const threeAct = await screen.findByRole("radio", { name: /Three-Act Structure/ });
    await fireEvent.click(threeAct);
    await next();

    await fireEvent.click(screen.getByRole("button", { name: "Create Book" }));

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith(
        "create_new_book_foundation",
        expect.objectContaining({
          input: expect.objectContaining({
            manuscript_template_id: "bundled-three-act",
          }),
        })
      )
    );
  });

  it("keeps the wizard open and does not activate a project when creation fails", async () => {
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "get_entity_template_registry") return registry;
      if (command === "get_bundled_templates") return manuscriptTemplates;
      if (command === "create_new_book_foundation") throw new Error("Destination already exists");
      return null;
    });
    const onClose = vi.fn();
    const onComplete = vi.fn();
    render(NewBookFoundationDialog, { props: { onClose, onComplete } });

    await completeBasics();
    for (let i = 0; i < 9; i += 1) await next();
    await fireEvent.click(screen.getByRole("button", { name: "Create Book" }));

    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain("Destination already exists")
    );
    expect(currentProject.value).toBeNull();
    expect(workspace.active).not.toBe("story-development");
    expect(onComplete).not.toHaveBeenCalled();
    expect(onClose).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Create Book" })).toBeTruthy();
  });

  it("creates the file-authoritative book and opens Story Development", async () => {
    const onClose = vi.fn();
    const onComplete = vi.fn();
    render(NewBookFoundationDialog, { props: { onClose, onComplete } });

    await completeBasics();
    for (let i = 0; i < 9; i += 1) await next();

    await fireEvent.click(screen.getByRole("button", { name: "Create Book" }));

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith(
        "create_new_book_foundation",
        expect.objectContaining({
          input: expect.objectContaining({
            project_root: "C:/Temp/LithoMark/Test Novel",
            title: "Test Novel",
            project_type: "novel",
          }),
        })
      )
    );

    expect(currentProject.value?.id).toBe(created.manifest.project_id);
    expect(currentProject.value?.project_root).toBe(created.paths.root);
    expect(workspace.active).toBe("story-development");
    expect(onComplete).toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();
  });
});
