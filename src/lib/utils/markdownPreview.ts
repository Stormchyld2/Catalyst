export function renderMarkdown(markdown: string | null | undefined): string {
  const escaped = escapeHtml((markdown ?? "").replace(/\r\n?/g, "\n"));
  const lines = escaped.split("\n");
  const output: string[] = [];
  let inList = false;
  let tableRows: string[][] = [];

  const closeTable = () => {
    if (!tableRows.length) return;
    const [header, ...body] = tableRows;
    output.push("<div class=\"markdown-table-wrap\"><table><thead><tr>");
    for (const cell of header) output.push(`<th>${inlineMarkdown(cell)}</th>`);
    output.push("</tr></thead><tbody>");
    for (const row of body) {
      output.push("<tr>");
      for (let i = 0; i < header.length; i++) output.push(`<td>${inlineMarkdown(row[i] ?? "")}</td>`);
      output.push("</tr>");
    }
    output.push("</tbody></table></div>");
    tableRows = [];
  };

  const closeList = () => {
    if (inList) {
      output.push("</ul>");
      inList = false;
    }
  };

  for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
    const rawLine = lines[lineIndex];
    const line = rawLine.trimEnd();

    if (!line.trim()) {
      closeList();
      closeTable();
      continue;
    }

    if (isTableRow(line)) {
      closeList();
      const cells = parseTableRow(line);
      const nextLine = lines[lineIndex + 1];
      if (!tableRows.length && nextLine && isTableSeparator(nextLine)) {
        tableRows.push(cells);
        continue;
      }
      if (tableRows.length) {
        if (isTableSeparator(line)) continue;
        tableRows.push(cells);
        continue;
      }
    } else {
      closeTable();
    }

    const heading = /^(#{1,6})\s+(.+)$/.exec(line);
    if (heading) {
      closeList();
      const level = heading[1].length;
      const plainHeading = decodeEscapedText(heading[2]);
      const id = markdownHeadingId(plainHeading);
      output.push(`<h${level} id="${id}">${inlineMarkdown(heading[2])}</h${level}>`);
      continue;
    }

    const bullet = /^[-*]\s+(.+)$/.exec(line);
    if (bullet) {
      if (!inList) {
        output.push("<ul>");
        inList = true;
      }
      output.push(`<li>${inlineMarkdown(bullet[1])}</li>`);
      continue;
    }

    closeList();
    output.push(`<p>${inlineMarkdown(line.trim())}</p>`);
  }

  closeList();
  closeTable();
  return output.join("\n");
}

export function markdownHeadingId(value: string): string {
  return value
    .toLowerCase()
    .replace(/&[a-z0-9#]+;/gi, "")
    .replace(/[^a-z0-9\s-]/g, "")
    .trim()
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-") || "section";
}

function decodeEscapedText(value: string): string {
  return value
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#039;/g, "'");
}

function inlineMarkdown(value: string): string {
  return value
    .replace(/~~(.+?)~~/g, "<del>$1</del>")
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/\*(.+?)\*/g, "<em>$1</em>")
    .replace(/\`([^\`]+)\`/g, "<code>$1</code>")
    .replace(/\[\[([^\]]+)\]\]/g, (_match, raw: string) => {
      const [target, label] = raw.split("|", 2);
      const decodedTarget = decodeEscapedText(target.trim());
      const display = label?.trim() || target.trim();
      const decodedDisplay = decodeEscapedText(display);
      return `<button type="button" class="wikilink" data-wikilink="${escapeAttribute(decodedTarget)}" aria-label="Open linked document ${escapeAttribute(decodedDisplay)}">${display}</button>`;
    });
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

function escapeAttribute(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}


function isTableRow(value: string): boolean {
  const line = value.trim();
  return line.startsWith("|") && line.endsWith("|") && line.slice(1, -1).includes("|");
}

function isTableSeparator(value: string): boolean {
  if (!isTableRow(value)) return false;
  return parseTableRow(value).every((cell) => /^:?-{3,}:?$/.test(cell.trim()));
}

function parseTableRow(value: string): string[] {
  return value.trim().slice(1, -1).split("|").map((cell) => cell.trim());
}
