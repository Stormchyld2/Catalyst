/**
 * Frontend adapter for the authoritative Catalyst project filesystem root.
 *
 * UI code should use this module instead of constructing project paths itself.
 */

import { invoke } from "@tauri-apps/api/core";

export interface ProjectPaths {
  root: string;
  project_manifest: string;
  manuscript: string;
  characters: string;
  locations: string;
  timeline: string;
  research: string;
  notes: string;
  reviews: string;
  review_rounds: string;
  review_distribution: string;
  audiobook: string;
  assets: string;
}

export function initializeProjectRoot(
  projectId: string,
  rootPath: string,
): Promise<ProjectPaths> {
  if (!projectId.trim()) {
    return Promise.reject(new Error("projectId must not be empty."));
  }
  if (!rootPath.trim()) {
    return Promise.reject(new Error("rootPath must not be empty."));
  }

  return invoke<ProjectPaths>("initialize_project_root", {
    projectId,
    rootPath,
  });
}

export function getProjectRoot(projectId: string): Promise<string | null> {
  if (!projectId.trim()) {
    return Promise.reject(new Error("projectId must not be empty."));
  }

  return invoke<string | null>("get_project_root", { projectId });
}

export function getProjectPaths(projectId: string): Promise<ProjectPaths> {
  if (!projectId.trim()) {
    return Promise.reject(new Error("projectId must not be empty."));
  }

  return invoke<ProjectPaths>("get_project_paths", { projectId });
}

/**
 * Create a file-authoritative book in an explicitly chosen empty folder.
 * This does not invoke any legacy SQLite-backed project creation command.
 */
export function createFileProject(
  rootPath: string,
  title: string,
  projectType: "novel" | "screenplay" = "novel",
): Promise<import("../types").Project> {
  if (!rootPath.trim()) {
    return Promise.reject(new Error("Choose a folder for the new book."));
  }
  if (!title.trim()) {
    return Promise.reject(new Error("Book title must not be empty."));
  }
  return invoke<import("../types").Project>("create_file_project", {
    rootPath,
    title,
    projectType,
  });
}
