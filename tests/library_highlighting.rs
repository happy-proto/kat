use std::path::Path;

use kat::{HighlightStyle, highlight_source_spans};

fn style_at(source: &str, needle: &str, offset: usize, path: &str) -> HighlightStyle {
    let position = source.find(needle).expect("needle in source") + offset;
    highlight_source_spans(Some(Path::new(path)), source)
        .expect("highlight source")
        .into_iter()
        .find(|span| span.range.start <= position && position < span.range.end)
        .expect("styled needle")
        .style
}

#[test]
fn source_spans_use_original_utf8_offsets_and_dracula_colors() {
    let source = "fn main() { let café = \"hello\"; }\n";
    let spans = highlight_source_spans(Some(Path::new("main.rs")), source).unwrap();

    assert!(!spans.is_empty());
    for span in &spans {
        assert!(span.range.start < span.range.end);
        assert!(source.get(span.range.clone()).is_some());
    }
    assert_eq!(
        style_at(source, "hello", 0, "main.rs").foreground,
        (241, 250, 140)
    );
}

#[test]
fn nested_markdown_rust_uses_the_same_syntax_style() {
    let rust = "fn nested() {}\n";
    let markdown = "```rust\nfn nested() {}\n```\n";
    assert_eq!(
        style_at(rust, "fn", 0, "main.rs"),
        style_at(markdown, "fn", 0, "README.md"),
    );
}

#[test]
fn unknown_file_has_no_syntax_spans() {
    assert!(
        highlight_source_spans(Some(Path::new("mystery.unknown")), "hello\n")
            .unwrap()
            .is_empty()
    );
}
