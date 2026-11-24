// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import StoryDevelopmentWorkspace from "./StoryDevelopmentWorkspace.svelte";
import { currentProject } from "../stores/project.svelte";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));


type TestMetadataField = {
  key: string;
  value: unknown;
};

interface TestStoryDevelopmentDocument {
  id: string;
  document_type: string;
  title: string;
  relative_path: string;
  body: string;
  source: string;
  metadata_yaml: string;
  metadata: TestMetadataField[];
  source_hash: string;
}

const indexResponse = {
  entries: [
    {
      id: "chr-11111111-1111-4111-8111-111111111111",
      entry_kind: "document",
      document_type: "character",
      title: "Test Character",
      relative_path: "Characters/Test Character.md",
      section: "Characters",
    },
    {
      id: "loc-11111111-1111-4111-8111-111111111112",
      entry_kind: "document",
      document_type: "location",
      title: "Test Location",
      relative_path: "Locations/Test Location.md",
      section: "Locations",
    },
    {
      id: "asset:Assets/Characters/Test Portrait.png",
      entry_kind: "image",
      document_type: "image",
      title: "Test Portrait.png",
      relative_path: "Assets/Characters/Test Portrait.png",
      section: "Assets",
    },
  ],
  issues: [],
};

function documentResponse(relativePath: string): TestStoryDevelopmentDocument {
  const isCharacter = relativePath === "Characters/Test Character.md";
  return {
    id: isCharacter
      ? "chr-11111111-1111-4111-8111-111111111111"
      : "loc-11111111-1111-4111-8111-111111111112",
    document_type: isCharacter ? "character" : "location",
    title: isCharacter ? "Test Character" : "Test Location",
    relative_path: relativePath,
    body: isCharacter ? "# Background\nSynthetic character notes." : "# Overview\nTest Location notes.",
    source: isCharacter ? "# Background\nSynthetic character notes." : "# Overview\nTest Location notes.",
    metadata_yaml: isCharacter
      ? "status: active\ncanon_status: canon\n"
      : "status: active\ncanon_status: canon\n",
    metadata: [
      { key: "status", value: "active" },
      { key: "canon_status", value: "canon" },
    ],
    source_hash: "test-hash",
  };
}

function mockStoryDevelopmentInvoke() {
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === "get_story_development_index") return indexResponse;
    if (command === "get_story_development_folders") return ["Story Development"];
    if (command === "get_entity_template_registry") return { templates: [], issues: [], user_templates_dir: "" };
    if (command === "get_story_development_document") {
      return documentResponse((args as { relativePath: string }).relativePath);
    }
    if (command === "update_story_development_metadata") {
      const input = args as {
        relativePath: string;
        key: string;
        value: unknown;
        expectedSourceHash: string;
      };
      const updated = documentResponse(input.relativePath);
      updated.metadata = updated.metadata.map((field) =>
        field.key === input.key ? { ...field, value: input.value } : field
      );
      updated.metadata_yaml = updated.metadata
        .map((field) => `${field.key}: ${String(field.value)}`)
        .join("\n");
      updated.source_hash = "updated-test-hash";
      return updated;
    }
    if (command === "get_story_development_asset") {
      return {
        relative_path: (args as { relativePath: string }).relativePath,
        mime_type: "image/png",
        bytes: [137, 80, 78, 71],
      };
    }
    if (command === "update_story_development_source" || command === "update_story_development_body") {
      const input = args as {
        relativePath: string;
        body?: string;
        source?: string;
        expectedSourceHash: string;
      };
      const updated = documentResponse(input.relativePath);
      const submitted = input.source ?? input.body ?? "";
      updated.body = submitted.endsWith("\n") ? submitted : `${submitted}\n`;
      updated.source = updated.body;
      updated.source_hash = "updated-body-hash";
      return updated;
    }
    throw new Error(`Unexpected command: ${command}`);
  });
}

describe("Story Development document relocation", () => {
  it("moves a document using its saved hash and preserves selection by stable ID", async () => {
    mockStoryDevelopmentInvoke();
    const originalInvoke = vi.mocked(invoke).getMockImplementation()!;
    const movedPath = "Characters/Renamed Character.md";
    const movedIndex = {
      ...indexResponse,
      entries: indexResponse.entries.map((entry) => entry.id === indexResponse.entries[0].id
        ? { ...entry, relative_path: movedPath } : entry),
    };
    let moved = false;
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "move_story_development_document") {
        moved = true;
        return { ...documentResponse("Characters/Test Character.md"), relative_path: movedPath };
      }
      if (command === "get_story_development_index" && moved) return movedIndex;
      if (command === "get_story_development_document" && (args as { relativePath?: string })?.relativePath === movedPath) {
        return { ...documentResponse("Characters/Test Character.md"), relative_path: movedPath };
      }
      return originalInvoke(command, args);
    });

    render(StoryDevelopmentWorkspace);
    expect(await screen.findByRole("heading", { name: "Test Character", level: 1 })).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Rename or move document" }));
    await fireEvent.input(screen.getByLabelText("Destination path"), { target: { value: movedPath } });
    await fireEvent.click(screen.getByRole("button", { name: "Move Document" }));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("move_story_development_document", {
        projectRoot: project.project_root,
        relativePath: "Characters/Test Character.md",
        destinationPath: movedPath,
        expectedSourceHash: "test-hash",
      });
      expect(screen.getByText(movedPath)).toBeTruthy();
    });
  });
});

describe("Story Development recoverable deletion", () => {
  it("requires confirmation and removes the deleted document from tabs and tree", async () => {
    mockStoryDevelopmentInvoke();
    const fallback = vi.mocked(invoke).getMockImplementation()!;
    let deleted = false;
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "delete_story_development_document") {
        deleted = true;
        return indexResponse.entries[0].id;
      }
      if (command === "get_story_development_index" && deleted) {
        return { ...indexResponse, entries: indexResponse.entries.slice(1) };
      }
      return fallback(command, args);
    });

    render(StoryDevelopmentWorkspace);
    expect(await screen.findByRole("heading", { name: "Test Character", level: 1 })).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Delete document" }));
    expect(screen.getByRole("dialog", { name: "Confirm document deletion" })).toBeTruthy();
    expect(invoke).not.toHaveBeenCalledWith("delete_story_development_document", expect.anything());
    await fireEvent.click(screen.getByRole("button", { name: "Move to Trash" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("delete_story_development_document", {
        projectRoot: project.project_root,
        relativePath: "Characters/Test Character.md",
        expectedSourceHash: "test-hash",
      });
      expect(screen.queryByRole("button", { name: "Project file Test Character" })).toBeNull();
    });
  });

  it("cancels deletion without invoking the backend", async () => {
    mockStoryDevelopmentInvoke();
    render(StoryDevelopmentWorkspace);
    expect(await screen.findByRole("heading", { name: "Test Character", level: 1 })).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Delete document" }));
    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("dialog", { name: "Confirm document deletion" })).toBeNull();
    expect(invoke).not.toHaveBeenCalledWith("delete_story_development_document", expect.anything());
  });
});

describe("Story Development trash restoration", () => {
  const trashed = {
    entry_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    original_path: "Characters/Test Character.md",
    document_id: indexResponse.entries[0].id,
    source_hash: "trash-hash",
  };

  it("restores the selected document into the Wiki tree and opens its tab", async () => {
    mockStoryDevelopmentInvoke();
    const fallback = vi.mocked(invoke).getMockImplementation()!;
    let restored = false;
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "get_story_development_index") {
        return restored ? indexResponse : { ...indexResponse, entries: indexResponse.entries.slice(1) };
      }
      if (command === "list_story_development_trash") return [trashed];
      if (command === "restore_story_development_trash") {
        restored = true;
        return documentResponse(trashed.original_path);
      }
      return fallback(command, args);
    });
    render(StoryDevelopmentWorkspace);
    expect(await screen.findByRole("button", { name: "Project file Test Location" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Project file Test Character" })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Restore deleted documents" }));
    expect(await screen.findByRole("dialog", { name: "Restore deleted documents" })).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Restore" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("restore_story_development_trash", {
        projectRoot: project.project_root,
        entryId: trashed.entry_id,
        expectedSourceHash: trashed.source_hash,
      });
      expect(screen.getByRole("button", { name: "Project file Test Character" })).toBeTruthy();
      expect(screen.getByRole("heading", { name: "Test Character", level: 1 })).toBeTruthy();
    });
    expect(screen.queryByRole("dialog", { name: "Restore deleted documents" })).toBeNull();
  });

  it("shows a restore conflict without claiming the file was restored", async () => {
    mockStoryDevelopmentInvoke();
    const fallback = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "get_story_development_index") {
        return { ...indexResponse, entries: indexResponse.entries.slice(1) };
      }
      if (command === "list_story_development_trash") return [trashed];
      if (command === "restore_story_development_trash") {
        throw new Error("Restore destination already exists; no file was overwritten.");
      }
      return fallback(command, args);
    });
    render(StoryDevelopmentWorkspace);
    expect(await screen.findByRole("button", { name: "Project file Test Location" })).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Restore deleted documents" }));
    await fireEvent.click(await screen.findByRole("button", { name: "Restore" }));
    expect(await screen.findByRole("alert")).toHaveProperty("textContent", expect.stringContaining("already exists"));
    expect(screen.getByRole("dialog", { name: "Restore deleted documents" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Project file Test Character" })).toBeNull();
  });
});

const project = {
  id: "11111111-1111-4111-8111-111111111111",
  name: "Test Novel",
  project_root: "C:/Temp/Catalyst/Test Novel",
};

let testProjectSequence = 0;

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  // Session state is intentionally keyed by project ID and survives component unmounts.
  // Isolate test projects so tab/selection state cannot leak between test cases.
  testProjectSequence += 1;
  currentProject.setProject({ ...project, id: `test-story-development-${testProjectSequence}` } as never);
});

afterEach(async () => {
  vi.restoreAllMocks();
  cleanup();
  currentProject.setProject(null);
});

describe("StoryDevelopmentWorkspace", () => {
  it("keeps indexed Wiki documents available if folder discovery fails", async () => {
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "get_story_development_index") return indexResponse;
      if (command === "get_story_development_folders") throw new Error("Folder scan unavailable");
      if (command === "get_entity_template_registry") return { templates: [], issues: [], user_templates_dir: "" };
      if (command === "get_story_development_document") {
        return documentResponse((args as { relativePath: string }).relativePath);
      }
      throw new Error(`Unexpected command: ${command}`);
    });

    render(StoryDevelopmentWorkspace);
    expect(await screen.findByRole("button", { name: "Project file Test Character" })).toBeTruthy();
    expect(await screen.findByRole("heading", { name: "Test Character", level: 1 })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Project file Test Location" })).toBeTruthy();
  });
  it("keeps indexed Wiki documents available if folder discovery returns invalid data", async () => {
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "get_story_development_index") return indexResponse;
      if (command === "get_story_development_folders") return null;
      if (command === "get_entity_template_registry") return { templates: [], issues: [], user_templates_dir: "" };
      if (command === "get_story_development_document") {
        return documentResponse((args as { relativePath: string }).relativePath);
      }
      throw new Error(`Unexpected command: ${command}`);
    });

    render(StoryDevelopmentWorkspace);
    expect(await screen.findByRole("button", { name: "Project file Test Character" })).toBeTruthy();
    expect(await screen.findByRole("heading", { name: "Test Character", level: 1 })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Folder Story Development" })).toBeTruthy();
  });

  it("loads and displays file-authoritative Wiki entries", async () => {
    mockStoryDevelopmentInvoke();

    render(StoryDevelopmentWorkspace);

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("get_story_development_index", {
        projectRoot: project.project_root,
      })
    );

    expect(await screen.findByRole("button", { name: "Project file Test Character" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Project file Test Location" })).toBeTruthy();
    expect(await screen.findByRole("heading", { name: "Test Character", level: 1 })).toBeTruthy();
  });

  it("opens a resolved wikilink through the shared project resolver", async () => {
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "get_story_development_index") return indexResponse;
      if (command === "get_entity_template_registry") return { templates: [], issues: [], user_templates_dir: "" };
      if (command === "get_story_development_document") {
        const result = documentResponse((args as { relativePath: string }).relativePath);
        if (result.relative_path === "Characters/Test Character.md") {
          result.body = result.source = "See [[Test Location|the location]].";
        }
        return result;
      }
      if (command === "resolve_project_link") {
        return {
          status: "resolved",
          target: {
            id: "loc-11111111-1111-4111-8111-111111111112",
            document_type: "location",
            title: "Test Location",
            aliases: [],
            relative_path: "Locations/Test Location.md",
          },
        };
      }
      throw new Error(`Unexpected command: ${command}`);
    });

    render(StoryDevelopmentWorkspace);
    const link = await screen.findByRole("button", { name: "Open linked document the location" });
    await fireEvent.click(link);

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("resolve_project_link", {
        projectRoot: project.project_root,
        query: "Test Location",
      })
    );
    expect(await screen.findByRole("heading", { name: "Test Location", level: 1 })).toBeTruthy();
  });

  it("surfaces unresolved wikilinks without guessing", async () => {
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "get_story_development_index") return indexResponse;
      if (command === "get_entity_template_registry") return { templates: [], issues: [], user_templates_dir: "" };
      if (command === "get_story_development_document") {
        const result = documentResponse((args as { relativePath: string }).relativePath);
        result.body = result.source = "See [[Missing Place]].";
        return result;
      }
      if (command === "resolve_project_link") {
        return { status: "unresolved", query: "Missing Place" };
      }
      throw new Error(`Unexpected command: ${command}`);
    });

    render(StoryDevelopmentWorkspace);
    await fireEvent.click(await screen.findByRole("button", { name: "Open linked document Missing Place" }));
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Unresolved link: Missing Place");
  });

  it("requires explicit stable-ID selection for ambiguous wikilinks", async () => {
    const selectedId = "loc-11111111-1111-4111-8111-111111111112";
    const outsideId = "loc-22222222-2222-4222-8222-222222222222";
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "get_story_development_index") return indexResponse;
      if (command === "get_entity_template_registry") return { templates: [], issues: [], user_templates_dir: "" };
      if (command === "get_story_development_document") {
        const result = documentResponse((args as { relativePath: string }).relativePath);
        if (result.relative_path === "Characters/Test Character.md") result.body = result.source = "See [[Shared Place]].";
        return result;
      }
      if (command === "resolve_project_link") {
        const query = (args as { query: string }).query;
        if (query === "Shared Place") {
          return {
            status: "ambiguous",
            query,
            candidates: [
              { id: selectedId, document_type: "location", title: "Test Location", aliases: ["Shared Place"], relative_path: "Locations/Test Location.md" },
              { id: outsideId, document_type: "location", title: "Other Location", aliases: ["Shared Place"], relative_path: "Notes/Other Location.md" },
            ],
          };
        }
        if (query === selectedId) {
          return { status: "resolved", target: { id: selectedId, document_type: "location", title: "Test Location", aliases: ["Shared Place"], relative_path: "Locations/Test Location.md" } };
        }
      }
      throw new Error(`Unexpected command: ${command}`);
    });

    render(StoryDevelopmentWorkspace);
    await fireEvent.click(await screen.findByRole("button", { name: "Open linked document Shared Place" }));

    expect(await screen.findByRole("dialog", { name: "Choose “Shared Place”" })).toBeTruthy();
    expect(screen.queryByRole("heading", { name: "Test Location", level: 1 })).toBeNull();
    expect(screen.getAllByText("Aliases: Shared Place")).toHaveLength(2);
    expect(screen.getByText(selectedId)).toBeTruthy();

    const choice = screen.getByText(selectedId).closest("button");
    expect(choice).toBeTruthy();
    await fireEvent.click(choice!);

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("resolve_project_link", {
      projectRoot: project.project_root,
      query: selectedId,
    }));
    expect(await screen.findByRole("heading", { name: "Test Location", level: 1 })).toBeTruthy();
    expect(screen.queryByRole("dialog", { name: "Choose “Shared Place”" })).toBeNull();
    expect(vi.mocked(invoke).mock.calls.some(([command]) => command === "update_story_development_body" || command === "update_story_development_metadata")).toBe(false);
  });

  it("dismisses ambiguous wikilink choices without navigating", async () => {
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "get_story_development_index") return indexResponse;
      if (command === "get_entity_template_registry") return { templates: [], issues: [], user_templates_dir: "" };
      if (command === "get_story_development_document") {
        const result = documentResponse((args as { relativePath: string }).relativePath);
        result.body = result.source = "See [[Shared Place]].";
        return result;
      }
      if (command === "resolve_project_link") return {
        status: "ambiguous", query: "Shared Place", candidates: [
          { id: "loc-22222222-2222-4222-8222-222222222222", document_type: "location", title: "Other Location", aliases: [], relative_path: "Notes/Other Location.md" },
        ],
      };
      throw new Error(`Unexpected command: ${command}`);
    });

    render(StoryDevelopmentWorkspace);
    await fireEvent.click(await screen.findByRole("button", { name: "Open linked document Shared Place" }));
    await screen.findByRole("dialog", { name: "Choose “Shared Place”" });
    await fireEvent.click(screen.getByRole("button", { name: "Dismiss ambiguous link choices" }));

    expect(screen.queryByRole("dialog", { name: "Choose “Shared Place”" })).toBeNull();
    expect(screen.getByRole("heading", { name: "Test Character", level: 1 })).toBeTruthy();
  });

  it("does not fall back to path when a resolved stable ID is outside Story Development", async () => {
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "get_story_development_index") return indexResponse;
      if (command === "get_entity_template_registry") return { templates: [], issues: [], user_templates_dir: "" };
      if (command === "get_story_development_document") {
        const result = documentResponse((args as { relativePath: string }).relativePath);
        result.body = result.source = "See [[Test Location]].";
        return result;
      }
      if (command === "resolve_project_link") return {
        status: "resolved",
        target: {
          id: "loc-99999999-9999-4999-8999-999999999999",
          document_type: "location",
          title: "Test Location",
          aliases: [],
          relative_path: "Locations/Test Location.md",
        },
      };
      throw new Error(`Unexpected command: ${command}`);
    });

    render(StoryDevelopmentWorkspace);
    await fireEvent.click(await screen.findByRole("button", { name: "Open linked document Test Location" }));

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("outside Story Development: Test Location");
    expect(screen.getByRole("heading", { name: "Test Character", level: 1 })).toBeTruthy();
    expect(screen.queryByRole("heading", { name: "Test Location", level: 1 })).toBeNull();
  });

  it("changes the selected Wiki page from the navigation", async () => {
    mockStoryDevelopmentInvoke();

    render(StoryDevelopmentWorkspace);
    const royalStreet = await screen.findByRole("button", { name: "Project file Test Location" });
    await fireEvent.click(royalStreet);

    expect(await screen.findByRole("heading", { name: "Test Location", level: 1 })).toBeTruthy();
    expect(royalStreet.getAttribute("aria-current")).toBe("page");
  });

  it("keeps opened documents in top tabs", async () => {
    mockStoryDevelopmentInvoke();

    render(StoryDevelopmentWorkspace);

    await screen.findByRole("heading", { name: "Test Character", level: 1 });
    await fireEvent.click(screen.getByRole("button", { name: "Project file Test Location" }));

    expect(screen.getByRole("tab", { name: /Test Character/ })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Test Location/ }).getAttribute("aria-selected")).toBe("true");
  });

  it("opens local project images in the Story Development image viewer", async () => {
    mockStoryDevelopmentInvoke();

    render(StoryDevelopmentWorkspace);

    const assetsCharacters = await screen.findByRole("button", { name: "Folder Assets/Characters" });
    await fireEvent.click(assetsCharacters);
    const imageEntry = await screen.findByRole("button", { name: "Project image Test Portrait.png" });
    await fireEvent.click(imageEntry);

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("get_story_development_asset", {
        projectRoot: project.project_root,
        relativePath: "Assets/Characters/Test Portrait.png",
      })
    );

    expect(screen.getByRole("img", { name: "Test Portrait.png" })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Test Portrait/ }).getAttribute("aria-selected")).toBe("true");
  });

  it("loads Overview images from filename-only metadata references", async () => {
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "get_story_development_index") return indexResponse;
      if (command === "get_entity_template_registry") {
        return { templates: [], issues: [], user_templates_dir: "C:/Temp/Templates" };
      }
      if (command === "get_story_development_document") {
        const result = documentResponse((args as { relativePath: string }).relativePath);
        result.metadata = [
          ...result.metadata,
          { key: "image", value: "Test Portrait.png" },
        ];
        return result;
      }
      if (command === "get_story_development_asset") {
        return {
          relative_path: "Assets/Characters/Test Portrait.png",
          mime_type: "image/png",
          bytes: [137, 80, 78, 71],
        };
      }
      throw new Error(`Unexpected command: ${command}`);
    });

    render(StoryDevelopmentWorkspace);
    await screen.findByRole("heading", { name: "Test Character", level: 1 });

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("get_story_development_asset", {
        projectRoot: project.project_root,
        relativePath: "Test Portrait.png",
      })
    );

    expect(screen.getByRole("img", { name: "Test Character reference" })).toBeTruthy();
  });

  it("surfaces ambiguous Overview image references instead of silently choosing", async () => {
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "get_story_development_index") return indexResponse;
      if (command === "get_entity_template_registry") {
        return { templates: [], issues: [], user_templates_dir: "C:/Temp/Templates" };
      }
      if (command === "get_story_development_document") {
        const result = documentResponse((args as { relativePath: string }).relativePath);
        result.metadata = [
          ...result.metadata,
          { key: "image", value: "portrait.png" },
        ];
        return result;
      }
      if (command === "get_story_development_asset") {
        throw new Error(
          "Image reference 'portrait.png' is ambiguous. Use a project-root-relative path."
        );
      }
      throw new Error(`Unexpected command: ${command}`);
    });

    render(StoryDevelopmentWorkspace);
    await screen.findByRole("heading", { name: "Test Character", level: 1 });

    expect(
      await screen.findByText(/Image reference 'portrait\.png' is ambiguous/)
    ).toBeTruthy();
  });

  it("filters the local project tree with file search", async () => {
    mockStoryDevelopmentInvoke();

    render(StoryDevelopmentWorkspace);
    await screen.findByRole("heading", { name: "Test Character", level: 1 });

    const search = screen.getByRole("searchbox", { name: "Search Story Development files" });
    await fireEvent.input(search, { target: { value: "Location" } });

    expect(screen.getByRole("button", { name: "Project file Test Location" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Project file Test Character" })).toBeNull();
  });

  it("shows document headings in Contents navigation", async () => {
    mockStoryDevelopmentInvoke();

    render(StoryDevelopmentWorkspace);
    await screen.findByRole("heading", { name: "Test Character", level: 1 });

    await fireEvent.click(screen.getByRole("button", { name: "Contents" }));

    expect(screen.getByRole("navigation", { name: "Document contents" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Background" })).toBeTruthy();
  });

  it("formats selected Markdown from the editor toolbar", async () => {
    mockStoryDevelopmentInvoke();

    render(StoryDevelopmentWorkspace);
    await screen.findByRole("heading", { name: "Test Character", level: 1 });
    await fireEvent.click(screen.getByRole("button", { name: "Edit" }));

    const editor = screen.getByRole("textbox", { name: "Document Markdown source including YAML frontmatter" }) as HTMLTextAreaElement;
    editor.setSelectionRange(2, 12);
    await fireEvent.click(screen.getByRole("button", { name: "Bold" }));

    expect(editor.value).toContain("**Background**");
    expect(screen.getByText("Unsaved")).toBeTruthy();
  });

  it("renders live Markdown preview in Split mode", async () => {
    mockStoryDevelopmentInvoke();

    render(StoryDevelopmentWorkspace);
    await screen.findByRole("heading", { name: "Test Character", level: 1 });
    await fireEvent.click(screen.getByRole("button", { name: "Split" }));

    expect(screen.getByText("Editor")).toBeTruthy();
    expect(screen.getByText("Preview")).toBeTruthy();

    const editor = screen.getByRole("textbox", { name: "Document Markdown source including YAML frontmatter" });
    await fireEvent.input(editor, { target: { value: "## Live Preview\n\nUpdated." } });

    expect(await screen.findByRole("heading", { name: "Live Preview", level: 2 })).toBeTruthy();
  });

  it("closes and reopens a clean tab", async () => {
    mockStoryDevelopmentInvoke();

    render(StoryDevelopmentWorkspace);
    await screen.findByRole("heading", { name: "Test Character", level: 1 });
    await fireEvent.click(screen.getByRole("button", { name: "Project file Test Location" }));
    await screen.findByRole("heading", { name: "Test Location", level: 1 });

    await fireEvent.click(screen.getByRole("button", { name: "Close Test Location" }));
    expect(screen.queryByRole("tab", { name: /Test Location/ })).toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: "Reopen closed tab" }));
    expect(await screen.findByRole("heading", { name: "Test Location", level: 1 })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /Test Location/ })).toBeTruthy();
  });

  it("shows dirty state and blocks closing an unsaved active tab when cancelled", async () => {
    mockStoryDevelopmentInvoke();
    vi.spyOn(window, "confirm").mockReturnValue(false);

    render(StoryDevelopmentWorkspace);
    await screen.findByRole("heading", { name: "Test Character", level: 1 });
    await fireEvent.click(screen.getByRole("button", { name: "Edit" }));

    const editor = screen.getByRole("textbox", { name: "Document Markdown source including YAML frontmatter" });
    await fireEvent.input(editor, { target: { value: "Unsaved tab draft" } });

    expect(screen.getByLabelText("Unsaved changes")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Close Test Character" }));

    expect(screen.getByRole("tab", { name: /Test Character/ })).toBeTruthy();
  });

  it("uses template-defined Overview fields for an explicitly assigned entity template", async () => {
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "get_story_development_index") return indexResponse;
      if (command === "get_entity_template_registry") {
        return {
          templates: [
            {
              id: "test-character-profile",
              name: "Test Character Profile",
              category: "character",
              document_type: "character",
              subtype: null,
              overview_fields: ["role"],
              image_field: "portrait",
              source: "user",
            },
          ],
          issues: [],
          user_templates_dir: "C:/Temp/Templates",
        };
      }
      if (command === "get_story_development_document") {
        const result = documentResponse((args as { relativePath: string }).relativePath);
        result.metadata = [
          { key: "template_id", value: "test-character-profile" },
          { key: "role", value: "Protagonist" },
          { key: "private_note", value: "Not shown in Overview" },
        ];
        return result;
      }
      throw new Error(`Unexpected command: ${command}`);
    });

    render(StoryDevelopmentWorkspace);
    await screen.findByRole("heading", { name: "Test Character", level: 1 });

    expect(screen.getByText("Test Character Profile")).toBeTruthy();
    expect(screen.getByText("Protagonist")).toBeTruthy();
    expect(screen.queryByText("Not shown in Overview")).toBeNull();
  });

  it("keeps overview metadata read-only while exposing full-source editing", async () => {
    mockStoryDevelopmentInvoke();
    render(StoryDevelopmentWorkspace);

    await screen.findByRole("heading", { name: "Test Character", level: 1 });
    expect(screen.getByRole("complementary", { name: "Document overview" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Edit Status" })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    expect(screen.getByRole("textbox", { name: "Document Markdown source including YAML frontmatter" })).toBeTruthy();
  });

  it("saves Markdown body through the local document update command", async () => {
    mockStoryDevelopmentInvoke();

    render(StoryDevelopmentWorkspace);

    await screen.findByRole("heading", { name: "Test Character", level: 1 });
    await fireEvent.click(screen.getByRole("button", { name: "Edit" }));

    const editor = screen.getByRole("textbox", { name: "Document Markdown source including YAML frontmatter" });
    await fireEvent.input(editor, { target: { value: "# Revised\n\nLocal file content." } });

    const save = screen.getByRole("button", { name: "Save" });
    await fireEvent.click(save);

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("update_story_development_source", {
        projectRoot: project.project_root,
        relativePath: "Characters/Test Character.md",
        source: "# Revised\n\nLocal file content.",
        expectedSourceHash: "test-hash",
      })
    );

    expect(screen.queryByText("Unsaved")).toBeNull();
  });

  it("saves the active Markdown body with Ctrl+S", async () => {
    mockStoryDevelopmentInvoke();

    render(StoryDevelopmentWorkspace);

    await screen.findByRole("heading", { name: "Test Character", level: 1 });
    await fireEvent.click(screen.getByRole("button", { name: "Edit" }));

    const editor = screen.getByRole("textbox", { name: "Document Markdown source including YAML frontmatter" });
    await fireEvent.input(editor, { target: { value: "# Shortcut save" } });
    await fireEvent.keyDown(window, { key: "s", ctrlKey: true });

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("update_story_development_source", {
        projectRoot: project.project_root,
        relativePath: "Characters/Test Character.md",
        source: "# Shortcut save",
        expectedSourceHash: "test-hash",
      })
    );
  });

  it("keeps the current file selected when unsaved navigation is cancelled", async () => {
    mockStoryDevelopmentInvoke();
    vi.spyOn(window, "confirm").mockReturnValue(false);

    render(StoryDevelopmentWorkspace);

    await screen.findByRole("heading", { name: "Test Character", level: 1 });
    await fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    const editor = screen.getByRole("textbox", { name: "Document Markdown source including YAML frontmatter" });
    await fireEvent.input(editor, { target: { value: "Unsaved draft" } });

    await fireEvent.click(screen.getByRole("button", { name: "Project file Test Location" }));

    expect(screen.getByRole("button", { name: "Project file Test Character" }).getAttribute("aria-current")).toBe("page");
    expect(screen.queryByRole("heading", { name: "Test Location", level: 1 })).toBeNull();
  });

  it("cancels a project change when the active body has unsaved edits", async () => {
    mockStoryDevelopmentInvoke();
    vi.spyOn(window, "confirm").mockReturnValue(false);

    render(StoryDevelopmentWorkspace);

    await screen.findByRole("heading", { name: "Test Character", level: 1 });
    await fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    const editor = screen.getByRole("textbox", { name: "Document Markdown source including YAML frontmatter" });
    await fireEvent.input(editor, { target: { value: "Unsaved project draft" } });

    const event = new CustomEvent("catalyst:before-project-change", { cancelable: true });
    expect(window.dispatchEvent(event)).toBe(false);
  });

  it("keeps the draft when a workspace-change save fails", async () => {
    mockStoryDevelopmentInvoke();
    const previous = vi.mocked(invoke).getMockImplementation()!;
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "update_story_development_source") {
        throw new Error("Stale document revision");
      }
      return previous(command, args);
    });

    render(StoryDevelopmentWorkspace);
    await screen.findByRole("heading", { name: "Test Character", level: 1 });
    await fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    const editor = screen.getByRole("textbox", { name: "Document Markdown source including YAML frontmatter" });
    await fireEvent.input(editor, { target: { value: "Keep this draft" } });

    const pendingDocumentSaves: Promise<boolean>[] = [];
    window.dispatchEvent(new CustomEvent("catalyst:before-workspace-change", {
      cancelable: true,
      detail: { pendingDocumentSaves },
    }));
    expect(pendingDocumentSaves).toHaveLength(1);
    expect(await pendingDocumentSaves[0]).toBe(false);
    expect((editor as HTMLTextAreaElement).value).toBe("Keep this draft");
    expect(screen.getByLabelText("Unsaved changes")).toBeTruthy();
  });

  it("surfaces invalid project files as Wiki issues", async () => {
    vi.mocked(invoke).mockResolvedValue({
      entries: [],
      issues: [
        {
          relative_path: "Locations/Broken.md",
          message: "Invalid document YAML",
        },
      ],
    });

    render(StoryDevelopmentWorkspace);

    expect(await screen.findByText("Locations/Broken.md")).toBeTruthy();
    expect(screen.getByText("Invalid document YAML")).toBeTruthy();
  });
});


describe("Campaign Overview direct navigation", () => {
  it("opens the canonical overview rather than the first adventure document", async () => {
    mockStoryDevelopmentInvoke();
    const original = vi.mocked(invoke).getMockImplementation()!;
    const path = "Adventures/Main Campaign/Campaign Overview.md";
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "get_story_development_index") {
        return { entries: [
          { id: "note-aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", entry_kind: "document", document_type: "note",
            title: "A Different Adventure", relative_path: "Adventures/Main Campaign/A Different Adventure.md", section: "Adventures" },
          { id: "note-bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", entry_kind: "document", document_type: "note",
            title: "Campaign Overview", relative_path: path, section: "Adventures" },
        ], issues: [] };
      }
      if (command === "get_story_development_folders") return ["Adventures", "Adventures/Main Campaign"];
      if (command === "get_story_development_document") {
        const relativePath = (args as { relativePath: string }).relativePath;
        return { ...documentResponse(relativePath), id: "note-bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
          title: relativePath === path ? "Campaign Overview" : "A Different Adventure", relative_path: relativePath };
      }
      return original(command, args);
    });
    render(StoryDevelopmentWorkspace, { props: { focusRoot: "Adventures", initialDocumentPath: path } });
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("get_story_development_document",
        expect.objectContaining({ relativePath: path }));
    });
    // The first adventure in the tree is never opened instead of the overview.
    expect(invoke).not.toHaveBeenCalledWith("get_story_development_document",
      expect.objectContaining({ relativePath: "Adventures/Main Campaign/A Different Adventure.md" }));
    expect(await screen.findByRole("button", { name: "Close Campaign Overview" })).toBeTruthy();
  });
});

describe("Campaign workspace labels", () => {
  it.each([
    [undefined, "World Builder"],
    ["Adventures", "Adventures"],
    ["Sessions", "Campaign Notes"],
  ] as const)("labels the %s view as %s", async (focusRoot, label) => {
    mockStoryDevelopmentInvoke();
    render(StoryDevelopmentWorkspace, { props: focusRoot ? { focusRoot } : {} });
    expect(await screen.findByText(label, { selector: ".eyebrow" })).toBeTruthy();
    expect(screen.queryByText("Story Development", { selector: ".eyebrow" })).toBeNull();
  });
});
