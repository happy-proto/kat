use std::path::Path;

use kat::{HighlightStyle, PreparedDocument, RenderOptions};

fn document(path: &str, source: &str) -> PreparedDocument {
    PreparedDocument::detect(
        Some(Path::new(path)),
        source,
        RenderOptions::with_colors((248, 248, 242), (40, 42, 54)),
    )
    .unwrap()
}

#[test]
fn library_exposes_backgrounds_for_every_nested_region_kind() {
    let cases = [
        ("justfile", "build:\n    echo hello\n", "echo"),
        ("README.md", "```rust\nfn main() {}\n```\n", "fn main"),
    ];
    for (path, source, body) in cases {
        let document = document(path, source);
        let line = source[..source.find(body).unwrap()]
            .bytes()
            .filter(|b| *b == b'\n')
            .count();
        let backgrounds = document.background_lines(4);
        assert!(
            backgrounds[line]
                .iter()
                .any(|run| run.end_column > run.start_column),
            "{path}"
        );
        assert!(
            backgrounds[0].is_empty(),
            "header should remain unshaded: {path}"
        );
    }
}

#[test]
fn just_recipe_nested_lines_share_a_rectangular_background() {
    let source = "demo:\n    echo outer\n        echo nested\n";
    let backgrounds = document("justfile", source).background_lines(4);
    assert_eq!(
        backgrounds[1][0].start_column,
        backgrounds[2][0].start_column
    );
    assert_eq!(backgrounds[1][0].end_column, backgrounds[2][0].end_column);
}

fn style_at(source: &str, needle: &str, offset: usize, path: &str) -> HighlightStyle {
    let position = source.find(needle).expect("needle in source") + offset;
    document(path, source)
        .spans()
        .into_iter()
        .find(|span| span.range.start <= position && position < span.range.end)
        .expect("styled needle")
        .style
}

#[test]
fn source_spans_use_original_utf8_offsets_and_dracula_colors() {
    let source = "fn main() { let café = \"hello\"; }\n";
    let spans = document("main.rs", source).spans();

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
    assert!(document("mystery.unknown", "hello\n").spans().is_empty());
}
