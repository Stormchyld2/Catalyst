/**
 * Transition bridge from the current editor's EditorialSource shape to the
 * native Catalyst review service.
 *
 * This file is deliberately separate from the native review adapter. The
 * native package format must not depend on legacy Kindling editorial models.
 * Delete this bridge once the editor itself supplies Markdown/file sources.
 */

import type { EditorialSource } from "../utils/editorial";
import type { ReviewSourceInput } from "./nativeReview";

export interface NativeReviewSourceSelection {
  sources: EditorialSource[];
  chapterIds?: string[];
}

/**
 * Converts the current editor source inventory into stable native review
 * source inputs. Source IDs remain the existing stable scene/beat UUIDs.
 *
 * NOTE: Current EditorialSource prose is HTML. We preserve that fact rather
 * than pretending it is Markdown. The actual HTML -> Markdown conversion
 * belongs at the manuscript-storage boundary and must be lossless enough for
 * review. Until that converter exists, this bridge refuses rich HTML instead
 * of silently corrupting manuscript formatting.
 */
export function prepareNativeReviewSources({
  sources,
  chapterIds = [],
}: NativeReviewSourceSelection): ReviewSourceInput[] {
  const selected = chapterIds.length
    ? sources.filter((source) => chapterIds.includes(source.chapter_id))
    : sources;

  const duplicateIds = new Set<string>();
  const seenIds = new Set<string>();
  for (const source of selected) {
    if (seenIds.has(source.id)) duplicateIds.add(source.id);
    seenIds.add(source.id);
  }
  if (duplicateIds.size) {
    throw new Error("The manuscript contains duplicate review source IDs.");
  }

  return selected.map((source, index) => ({
    source_id: source.id,
    relative_path: sourcePath(source, index, selected),
    manuscript_markdown: htmlToPlainMarkdown(source.html),
  }));
}

function sourcePath(
  source: EditorialSource,
  index: number,
  sources: EditorialSource[],
): string {
  const chapterNumber =
    sources.findIndex((candidate) => candidate.chapter_id === source.chapter_id) + 1;
  const scenePeers = sources.filter(
    (candidate) =>
      candidate.chapter_id === source.chapter_id &&
      candidate.scene_id === source.scene_id,
  );
  const sourceNumber = scenePeers.findIndex((candidate) => candidate.id === source.id) + 1;

  const chapter = safeSegment(source.chapter || `Chapter ${chapterNumber}`);
  const scene = safeSegment(source.scene || `Scene ${index + 1}`);
  const suffix = scenePeers.length > 1 ? ` - Part ${sourceNumber}` : "";

  return `Manuscript/${chapter}/${scene}${suffix}.md`;
}

function safeSegment(value: string): string {
  const cleaned = value
    .replace(/[<>:"/\\|?*\u0000-\u001f]/g, " ")
    .replace(/\s+/g, " ")
    .replace(/[. ]+$/g, "")
    .trim();

  return cleaned || "Untitled";
}

/**
 * Transitional conservative HTML converter.
 *
 * The current editor stores prose as HTML. This converter supports the block
 * and inline markup produced by the normal manuscript editor and rejects
 * unsupported structures rather than exporting misleading Markdown.
 */
function htmlToPlainMarkdown(html: string): string {
  const root = document.createElement("div");
  root.innerHTML = html;

  const markdown = Array.from(root.childNodes)
    .map((node) => renderNode(node, 0))
    .join("")
    .replace(/\n{3,}/g, "\n\n")
    .trim();

  if (!markdown) {
    throw new Error("A selected manuscript source contains no reviewable prose.");
  }

  return `${markdown}\n`;
}

function renderNode(node: Node, listDepth: number): string {
  if (node.nodeType === Node.TEXT_NODE) return node.textContent ?? "";
  if (!(node instanceof HTMLElement)) return "";

  const inner = () =>
    Array.from(node.childNodes)
      .map((child) => renderNode(child, listDepth))
      .join("");

  switch (node.tagName.toLowerCase()) {
    case "p":
      return `${inner().trim()}\n\n`;
    case "br":
      return "\n";
    case "strong":
    case "b":
      return `**${inner()}**`;
    case "em":
    case "i":
      return `*${inner()}*`;
    case "u":
      return inner();
    case "s":
    case "strike":
      return `~~${inner()}~~`;
    case "blockquote":
      return (
        inner()
          .trim()
          .split("\n")
          .map((line) => `> ${line}`)
          .join("\n") + "\n\n"
      );
    case "h1":
    case "h2":
    case "h3":
    case "h4":
    case "h5":
    case "h6": {
      const level = Number(node.tagName.slice(1));
      return `${"#".repeat(level)} ${inner().trim()}\n\n`;
    }
    case "a": {
      const href = node.getAttribute("href") ?? "";
      if (!/^(https?:|mailto:)/i.test(href))
        throw new Error("The manuscript contains an unsupported link.");
      return `[${inner()}](${href})`;
    }
    case "ul":
    case "ol":
      return `${Array.from(node.children)
        .map((child) => renderNode(child, listDepth + 1))
        .join("")}\n`;
    case "li": {
      const parent = node.parentElement?.tagName.toLowerCase();
      const prefix =
        parent === "ol"
          ? `${Array.from(node.parentElement?.children ?? []).indexOf(node) + 1}. `
          : "- ";
      return `${"  ".repeat(Math.max(0, listDepth - 1))}${prefix}${inner().trim()}\n`;
    }
    case "code":
      return `\`${inner()}\``;
    case "pre":
      return `\n\`\`\`\n${node.textContent ?? ""}\n\`\`\`\n\n`;
    case "div":
    case "span":
      return inner();
    default:
      throw new Error(
        `The manuscript contains unsupported <${node.tagName.toLowerCase()}> formatting. ` +
          "Native review export stopped rather than altering the manuscript.",
      );
  }
}
