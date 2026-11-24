// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import EntityTemplateManager from "./EntityTemplateManager.svelte";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const registry = {
  templates: [
    {
      id: "character-profile",
      name: "Character Profile",
      category: "character",
      scope: "story-development",
      document_type: "character",
      subtype: null,
      version: 1,
      description: "Built-in character profile.",
      overview_fields: ["role"],
      image_field: "image",
      fields: [
        { key: "image", label: "Image", type: "image", required: false, options: [] },
        { key: "role", label: "Role", type: "text", required: false, options: [] },
      ],
      body: "# Character Summary",
      source: "built-in",
      relative_path: null,
    },
    {
      id: "custom-place",
      name: "Custom Place",
      category: "location",
      scope: "worldbuilding",
      document_type: "location",
      subtype: null,
      version: 1,
      description: "User location template.",
      overview_fields: ["region"],
      image_field: "image",
      fields: [
        { key: "image", label: "Image", type: "image", required: false, options: [] },
        { key: "region", label: "Region", type: "text", required: false, options: [] },
      ],
      body: "# Location Summary",
      source: "user",
      relative_path: "Locations/Custom Place.md",
    },
  ],
  issues: [],
  user_templates_dir: "C:/Temp/Catalyst/Templates",
};

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "get_entity_template_registry") return registry;
    if (command === "save_user_entity_template") return registry;
    if (command === "delete_user_entity_template") return {
      ...registry,
      templates: registry.templates.filter((template) => template.id !== "custom-place"),
    };
    throw new Error(`Unexpected command: ${command}`);
  });
});

afterEach(() => {
  vi.restoreAllMocks();
  cleanup();
});

describe("EntityTemplateManager", () => {
  it("shows built-in templates as read-only and user templates as editable", async () => {
    render(EntityTemplateManager, { props: { onClose: vi.fn() } });

    await fireEvent.click(await screen.findByRole("button", { name: /Character Profile/ }));
    expect(screen.getByText(/Built-in templates are read-only/)).toBeTruthy();
    expect((screen.getByDisplayValue("character-profile") as HTMLInputElement).disabled).toBe(true);

    await fireEvent.click(screen.getByRole("button", { name: /Custom Place/ }));
    expect((screen.getByDisplayValue("custom-place") as HTMLInputElement).disabled).toBe(false);
    expect(screen.getByRole("button", { name: "Delete" })).toBeTruthy();
  });

  it("supports category filtering and copying a built-in template", async () => {
    render(EntityTemplateManager, { props: { onClose: vi.fn() } });
    await screen.findByRole("button", { name: /Character Profile/ });
    await fireEvent.change(screen.getByLabelText("Filter template category"), { target: { value: "location" } });
    expect(screen.queryByRole("button", { name: /Character Profile/ })).toBeNull();
    expect(screen.getByRole("button", { name: /Custom Place/ })).toBeTruthy();
    await fireEvent.change(screen.getByLabelText("Filter template category"), { target: { value: "all" } });
    await fireEvent.click(screen.getByRole("button", { name: /Character Profile/ }));
    await fireEvent.click(screen.getByRole("button", { name: "Duplicate selected template to customize" }));
    expect((screen.getByDisplayValue("Character Profile (Custom)") as HTMLInputElement).disabled).toBe(false);
    expect(screen.getByRole("button", { name: /Save template/i })).toBeTruthy();
  });

  it("creates a user template through the file-backed save command", async () => {
    render(EntityTemplateManager, { props: { onClose: vi.fn() } });

    await screen.findByRole("button", { name: /Character Profile/ });
    await fireEvent.click(screen.getByRole("button", { name: /New user template/i }));

    await fireEvent.input(screen.getByLabelText("ID"), { target: { value: "test-template" } });
    await fireEvent.input(screen.getByLabelText("Name"), { target: { value: "Test Template" } });
    await fireEvent.input(screen.getByLabelText("Template file"), { target: { value: "Tests/Test Template.md" } });
    await fireEvent.input(screen.getByLabelText("Overview fields"), { target: { value: "" } });
    await fireEvent.click(screen.getByRole("button", { name: /Save template/i }));

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith(
        "save_user_entity_template",
        expect.objectContaining({
          input: expect.objectContaining({
            relative_path: "Tests/Test Template.md",
            previous_relative_path: null,
            manifest: expect.objectContaining({
              id: "test-template",
              name: "Test Template",
            }),
          }),
        })
      )
    );
  });

  it("deletes a user template through the file-backed delete command", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(true);
    render(EntityTemplateManager, { props: { onClose: vi.fn() } });

    await fireEvent.click(await screen.findByRole("button", { name: /Custom Place/ }));
    await fireEvent.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("delete_user_entity_template", {
        relativePath: "Locations/Custom Place.md",
      })
    );
  });
});
