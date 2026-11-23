import { describe, expect, it } from "vitest";
import { renderMarkdown } from "./markdownPreview";

describe("renderMarkdown", () => {
  it("renders headings, paragraphs, lists, emphasis, code, and wikilinks", () => {
    const html = renderMarkdown(
      "# Title\n\nParagraph with **bold**, *italics*, ~~strike~~, `code`, and [[Test Link]].\n\n- One\n- Two"
    );

    expect(html).toContain('<h1 id="title">Title</h1>');
    expect(html).toContain("<strong>bold</strong>");
    expect(html).toContain("<em>italics</em>");
    expect(html).toContain("<del>strike</del>");
    expect(html).toContain("<code>code</code>");
    expect(html).toContain('class="wikilink"');
    expect(html).toContain('data-wikilink="Test Link"');
    expect(html).toContain('aria-label="Open linked document Test Link"');
    expect(html).toContain(">Test Link</button>");
    expect(html).toContain("<ul>");
  });

  it("renders custom wikilink labels while preserving the resolver target", () => {
    const html = renderMarkdown("[[Test & Place|the <place>]]");

    expect(html).toContain('data-wikilink="Test &amp; Place"');
    expect(html).toContain('aria-label="Open linked document the &lt;place&gt;"');
    expect(html).toContain(">the &lt;place&gt;</button>");
  });

  it("escapes raw HTML before rendering markdown", () => {
    expect(renderMarkdown("<script>alert(1)</script>")).toContain("&lt;script&gt;");
  });
});
