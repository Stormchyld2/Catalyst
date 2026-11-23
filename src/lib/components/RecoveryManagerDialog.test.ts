// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import RecoveryManagerDialog from "./RecoveryManagerDialog.svelte";
import { currentProject } from "../stores/project.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const projectRoot = "C:/Temp/LithoMark/Recovery Test";
const candidate = {
  artifact_path: "Characters/.lithomark-save-recovery.tmp",
  original_path: "Characters/Mara.md",
  kind: "staged",
  valid_document: true,
};
const review = {
  artifact_hash: "artifact-hash",
  destination_hash: "destination-hash",
  original_path: candidate.original_path,
};

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "list_document_recovery_candidates") return [candidate];
    if (command === "preview_document_recovery_candidate") return "Recovered draft";
    if (command === "review_document_recovery_restore") return review;
    if (command === "restore_document_recovery_candidate") return "Characters/safety-copy.bak";
    throw new Error("Unexpected command: " + command);
  });
  currentProject.setProject({ id: "recovery-test", name: "Recovery Test", project_root: projectRoot } as never);
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  currentProject.setProject(null);
});

describe("Recovery Manager", () => {
  it("reviews both hashes and requires confirmation before restoring", async () => {
    const confirm = vi.spyOn(window, "confirm").mockReturnValue(true);
    const restored = vi.fn();
    window.addEventListener("lithomark:recovery-restored", restored);
    try {
      render(RecoveryManagerDialog, { props: { onClose: vi.fn() } });
      await screen.findByText(candidate.original_path);
      await fireEvent.click(screen.getByRole("button", { name: "Preview" }));
      expect(await screen.findByLabelText("Recovery file preview")).toHaveProperty("textContent", "Recovered draft");
      await fireEvent.click(screen.getByRole("button", { name: "Restore…" }));
      await waitFor(() => expect(invoke).toHaveBeenCalledWith("restore_document_recovery_candidate", {
        projectRoot,
        artifactPath: candidate.artifact_path,
        expectedArtifactHash: review.artifact_hash,
        expectedDestinationHash: review.destination_hash,
      }));
      expect(confirm).toHaveBeenCalled();
      expect(restored).toHaveBeenCalled();
    } finally {
      window.removeEventListener("lithomark:recovery-restored", restored);
    }
  });

  it("does not restore when the author declines confirmation", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(false);
    render(RecoveryManagerDialog, { props: { onClose: vi.fn() } });
    await screen.findByText(candidate.original_path);
    await fireEvent.click(screen.getByRole("button", { name: "Restore…" }));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("review_document_recovery_restore", {
      projectRoot, artifactPath: candidate.artifact_path,
    }));
    expect(invoke).not.toHaveBeenCalledWith("restore_document_recovery_candidate", expect.anything());
  });

  it("blocks restoration when the active editor cannot save", async () => {
    const guard = (event: Event) => {
      (event as CustomEvent<{ pendingDocumentSaves: Promise<boolean>[] }>).detail.pendingDocumentSaves.push(Promise.resolve(false));
    };
    window.addEventListener("lithomark:before-workspace-change", guard);
    try {
      render(RecoveryManagerDialog, { props: { onClose: vi.fn() } });
      await screen.findByText(candidate.original_path);
      await fireEvent.click(screen.getByRole("button", { name: "Restore…" }));
      expect(await screen.findByRole("alert")).toHaveProperty("textContent", "Save the current document before restoring a recovery copy.");
      expect(invoke).not.toHaveBeenCalledWith("review_document_recovery_restore", expect.anything());
    } finally {
      window.removeEventListener("lithomark:before-workspace-change", guard);
    }
  });
});
