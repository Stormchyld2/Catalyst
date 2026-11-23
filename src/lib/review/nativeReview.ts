/**
 * Native LithoMark review-package frontend adapter.
 *
 * Keep UI components insulated from raw Tauri command names and Rust transport
 * details. Legacy kindling-editorial commands are intentionally not exposed here.
 */

import { invoke } from "@tauri-apps/api/core";

export interface ReviewProject {
  id: string;
  title: string;
  author?: string | null;
}

export interface ReviewRecipient {
  id: string;
  name: string;
  email?: string | null;
}

export interface ReviewDistribution {
  purpose: string;
  created_at: string;
}

export interface ReviewSourceInput {
  source_id: string;
  relative_path: string;
  manuscript_markdown: string;
}

export interface ReviewSourceIntegrity {
  source_id: string;
  path: string;
  sha256: string;
}

export interface ReviewIntegrity {
  algorithm: string;
  manuscript_sha256: string;
  files: ReviewSourceIntegrity[];
}

export interface ReviewManifest {
  format: string;
  format_version: number;
  review_id: string;
  round_id: string;
  project: ReviewProject;
  reviewer: ReviewRecipient;
  distribution: ReviewDistribution;
  integrity: ReviewIntegrity;
}

export interface CreateReviewPackageRequest {
  output_dir: string;
  ledger_dir: string;
  package_reference: string;
  review_id: string;
  round_id: string;
  project: ReviewProject;
  reviewer: ReviewRecipient;
  distribution: ReviewDistribution;
  sources: ReviewSourceInput[];
}

export interface CreateReviewPackageResult {
  package_path: string;
  ledger_record_path: string;
  manifest: ReviewManifest;
}

export interface DistributionRecord {
  format: string;
  format_version: number;
  review_id: string;
  round_id: string;
  project_id: string;
  project_title: string;
  reviewer_id: string;
  reviewer_name: string;
  reviewer_email?: string | null;
  purpose: string;
  created_at: string;
  manuscript_sha256: string;
  source_count: number;
  package_reference: string;
}

/**
 * Creates a complete native LithoMark review package and records the
 * distribution in the author's durable ledger.
 */
export function createNativeReviewPackage(
  request: CreateReviewPackageRequest,
): Promise<CreateReviewPackageResult> {
  return invoke<CreateReviewPackageResult>("create_native_review_package", {
    request,
  });
}

/**
 * Reads the author's native LithoMark review distribution ledger.
 */
export function readNativeReviewDistributions(
  ledgerDir: string,
): Promise<DistributionRecord[]> {
  if (!ledgerDir.trim()) {
    return Promise.reject(new Error("ledgerDir must not be empty."));
  }

  return invoke<DistributionRecord[]>("read_native_review_distributions", {
    ledgerDir,
  });
}
