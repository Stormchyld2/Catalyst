use html_to_markdown_rs::{convert, options::ConversionOptions};

/// Converts editor HTML into portable Markdown.
///
/// Markdown is LithoMark's durable prose representation. HTML belongs to the
/// editor layer and must cross this boundary before being written to a
/// portable manuscript or review package.
pub fn html_to_markdown(html: &str) -> Result<String, String> {
    if html.trim().is_empty() {
        return Ok(String::new());
    }

    let result = convert(html, Some(ConversionOptions::default()))
        .map_err(|error| format!("Failed to convert editor HTML to Markdown: {error}"))?;

    let markdown = result.content.unwrap_or_default();
    Ok(normalize_markdown(&markdown))
}

fn normalize_markdown(markdown: &str) -> String {
    let normalized = markdown.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim_end();

    if trimmed.is_empty() {
        String::new()
    } else {
        format!("{trimmed}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::html_to_markdown;

    #[test]
    fn converts_paragraphs_without_losing_text() {
        let markdown =
            html_to_markdown("<p>First paragraph.</p><p>Second paragraph.</p>").unwrap();

        assert!(markdown.contains("First paragraph."));
        assert!(markdown.contains("Second paragraph."));
        assert!(markdown.ends_with('\n'));
    }

    #[test]
    fn preserves_emphasis_and_strong_text() {
        let markdown =
            html_to_markdown("<p>This is <em>important</em> and <strong>critical</strong>.</p>")
                .unwrap();

        assert!(markdown.contains("*important*") || markdown.contains("_important_"));
        assert!(
            markdown.contains("**critical**")
                || markdown.contains("__critical__")
        );
    }

    #[test]
    fn preserves_links() {
        let markdown =
            html_to_markdown(r#"<p>Visit <a href="https://example.com">Example</a>.</p>"#)
                .unwrap();

        assert!(markdown.contains("[Example](https://example.com"));
    }

    #[test]
    fn preserves_unordered_lists() {
        let markdown =
            html_to_markdown("<ul><li>Alpha</li><li>Beta</li></ul>").unwrap();

        assert!(markdown.contains("Alpha"));
        assert!(markdown.contains("Beta"));
        assert!(
            markdown.contains("- Alpha")
                || markdown.contains("* Alpha")
                || markdown.contains("+ Alpha")
        );
    }

    #[test]
    fn preserves_horizontal_rules_as_markdown() {
        let markdown =
            html_to_markdown("<p>Before</p><hr><p>After</p>").unwrap();

        assert!(markdown.contains("Before"));
        assert!(markdown.contains("After"));
        assert!(
            markdown.contains("---")
                || markdown.contains("***")
                || markdown.contains("___")
        );
    }

    #[test]
    fn decodes_html_entities() {
        let markdown =
            html_to_markdown("<p>Smith &amp; Jones &mdash; Review</p>").unwrap();

        assert!(markdown.contains("Smith & Jones"));
        assert!(markdown.contains("Review"));
        assert!(!markdown.contains("&amp;"));
    }

    #[test]
    fn empty_html_produces_empty_markdown() {
        assert_eq!(html_to_markdown("   ").unwrap(), "");
    }

    #[test]
    fn normalizes_output_line_endings() {
        let markdown =
            html_to_markdown("<p>Line one.</p><p>Line two.</p>").unwrap();

        assert!(!markdown.contains('\r'));
    }
}