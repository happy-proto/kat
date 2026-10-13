use std::{path::Path, process::Command};

use kat::{HighlightStyle, PreparedDocument, RenderOptions};

const MINIMAL: &str = include_str!("../testdata/fixtures/graph_easy/minimal.ge");
const FEATURES: &str = include_str!("../testdata/fixtures/graph_easy/features.ge");

fn document(path: &str, source: &str) -> PreparedDocument {
    PreparedDocument::detect(
        Some(Path::new(path)),
        source,
        RenderOptions::with_colors((248, 248, 242), (40, 42, 54)),
    )
    .unwrap()
}

fn style(source: &str, needle: &str) -> HighlightStyle {
    style_in("diagram.ge", source, needle)
}

fn style_in(path: &str, source: &str, needle: &str) -> HighlightStyle {
    let offset = source.find(needle).unwrap();
    document(path, source)
        .spans()
        .into_iter()
        .find(|span| span.range.contains(&offset))
        .unwrap_or_else(|| panic!("missing style for {needle}"))
        .style
}

#[test]
fn detects_ge_without_claiming_dot_or_plain_text() {
    assert_eq!(
        kat::detected_language_name(Some(Path::new("minimal.ge")), MINIMAL),
        Some("graph_easy")
    );
    assert_eq!(
        kat::detected_language_name(Some(Path::new("graph.gv")), "digraph { A -> B }"),
        Some("dot")
    );
    assert_ne!(
        kat::detected_language_name(Some(Path::new("notes.txt")), MINIMAL),
        Some("graph_easy")
    );
    for language in ["graph_easy", "graph-easy", "ge"] {
        let named = PreparedDocument::named_language(
            language,
            MINIMAL,
            RenderOptions::with_colors((248, 248, 242), (40, 42, 54)),
        )
        .unwrap();
        assert_eq!(named.spans(), document("minimal.ge", MINIMAL).spans());
    }
}

#[test]
fn graph_easy_constructs_have_distinct_semantic_styles() {
    for label in [
        "Longest",
        "输入",
        "Worker",
        "请求",
        "retry",
        "Hash # inside",
        "request-v2",
        "build-step",
        "deploy.step",
    ] {
        let source = if label == "Longest" {
            MINIMAL
        } else {
            FEATURES
        };
        assert_eq!(style(source, label).foreground, (241, 250, 140), "{label}");
    }
    for operator in ["->", "<-->", "..>", "==>", "~~>", "##>"] {
        assert_eq!(
            style(FEATURES, operator).foreground,
            (255, 121, 198),
            "{operator}"
        );
    }
    assert_eq!(style(FEATURES, "shape").foreground, (139, 233, 253));
    assert_eq!(style(FEATURES, "Pipeline").foreground, (139, 233, 253));
    assert_eq!(style(FEATURES, "node.warning").foreground, (255, 121, 198));
    let comment = style(FEATURES, "# Graph");
    assert_eq!(comment.foreground, (98, 114, 164));
    assert!(comment.italic);
}

#[test]
fn markdown_fences_share_graph_easy_runtime() {
    for alias in ["graph-easy", "graph_easy", "ge"] {
        let source = format!("```{alias}\n{MINIMAL}```\n");
        let value: serde_json::Value = serde_json::from_str(
            &kat::debug_analysis_json(Some(Path::new("README.md")), &source).unwrap(),
        )
        .unwrap();
        assert!(
            value["nested_regions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|region| { region["resolved_document_kind"]["runtime_name"] == "graph_easy" }),
            "{value}"
        );
        assert_eq!(
            style_in("README.md", &source, "Longest").foreground,
            (241, 250, 140)
        );
    }
}

#[test]
fn escaped_labels_and_incomplete_input_preserve_earlier_highlighting() {
    let source = "[ literal\\]bracket ] -> [ complete ]\n[ B ] -> [ unfinished";
    assert_eq!(style(source, "literal").foreground, (241, 250, 140));
    assert_eq!(style(source, "complete").foreground, (241, 250, 140));
    assert_eq!(style(source, "->").foreground, (255, 121, 198));
}

#[test]
fn cli_preserves_graph_easy_source_and_adds_highlighting() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/fixtures/graph_easy/minimal.ge");
    let output = Command::new(env!("CARGO_BIN_EXE_kat"))
        .args(["--hyperlinks", "never"])
        .arg(path)
        .env("COLORTERM", "truecolor")
        .env_remove("NO_COLOR")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered = String::from_utf8(output.stdout).unwrap();
    assert!(rendered.contains("\x1b[38;2;241;250;140m"), "{rendered:?}");
    assert!(rendered.contains("Longest"));
    assert!(rendered.contains("->"));
    let plain = Command::new(env!("CARGO_BIN_EXE_kat"))
        .args(["--hyperlinks", "never"])
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/fixtures/graph_easy/minimal.ge"))
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    assert!(plain.status.success());
    assert_eq!(String::from_utf8(plain.stdout).unwrap(), MINIMAL);
}
