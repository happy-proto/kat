use std::{path::Path, process::Command};

use kat::{HighlightStyle, PreparedDocument, RenderOptions};
use serde_json::Value;

const FLOWCHART: &str = include_str!("../testdata/fixtures/mermaid/flowchart.mmd");
const SEQUENCE: &str = include_str!("../testdata/fixtures/mermaid/sequence.mermaid");
const MARKDOWN: &str = include_str!("../testdata/fixtures/markdown/mermaid_fences.md");

fn document(path: &str, source: &str) -> PreparedDocument {
    PreparedDocument::detect(
        Some(Path::new(path)),
        source,
        RenderOptions::with_colors((248, 248, 242), (40, 42, 54)),
    )
    .unwrap()
}

fn style(path: &str, source: &str, needle: &str) -> HighlightStyle {
    let offset = source.find(needle).unwrap();
    document(path, source)
        .spans()
        .into_iter()
        .find(|span| span.range.contains(&offset))
        .unwrap_or_else(|| panic!("missing style for {needle}"))
        .style
}

#[test]
fn detects_mermaid_extensions_and_supports_explicit_language() {
    for path in ["flow.mmd", "diagram.mermaid"] {
        assert_eq!(
            kat::detected_language_name(Some(Path::new(path)), FLOWCHART),
            Some("mermaid")
        );
    }
    let named = PreparedDocument::named_language(
        "mermaid",
        FLOWCHART,
        RenderOptions::with_colors((248, 248, 242), (40, 42, 54)),
    )
    .unwrap();
    assert_eq!(named.spans(), document("flow.mmd", FLOWCHART).spans());
    assert_ne!(
        kat::detected_language_name(Some(Path::new("notes.txt")), FLOWCHART),
        Some("mermaid")
    );
}

#[test]
fn flowchart_and_sequence_tokens_use_semantic_colors() {
    for (source, path, tokens) in [
        (
            FLOWCHART,
            "flow.mmd",
            vec!["flowchart", "subgraph", "classDef"],
        ),
        (
            SEQUENCE,
            "sequence.mermaid",
            vec!["sequenceDiagram", "participant", "alt healthy", "else"],
        ),
    ] {
        for token in tokens {
            assert_eq!(
                style(path, source, token).foreground,
                (255, 121, 198),
                "{token}"
            );
        }
    }
    let comment = style("flow.mmd", FLOWCHART, "%%");
    assert_eq!(comment.foreground, (98, 114, 164));
    assert!(comment.italic);
    for text in ["客户端", "请求", "数据库"] {
        assert_eq!(
            style("flow.mmd", FLOWCHART, text).foreground,
            (241, 250, 140)
        );
    }
    assert_eq!(
        style("sequence.mermaid", SEQUENCE, "GET /health").foreground,
        (241, 250, 140)
    );
    assert_eq!(
        style("flow.mmd", FLOWCHART, "-->").foreground,
        (255, 121, 198)
    );
}

#[test]
fn markdown_fences_resolve_mermaid_without_claiming_unknown_languages() {
    let value: Value = serde_json::from_str(
        &kat::debug_analysis_json(Some(Path::new("README.md")), MARKDOWN).unwrap(),
    )
    .unwrap();
    let regions: Vec<_> = value["nested_regions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|region| region["resolved_document_kind"]["runtime_name"] != "markdown_inline")
        .collect();
    assert_eq!(regions.len(), 3);
    for region in regions {
        assert_eq!(region["resolved_document_kind"]["runtime_name"], "mermaid");
        assert_eq!(region["visual_kind"], "rect_block");
    }
    for (token, color) in [
        ("flowchart", (255, 121, 198)),
        ("sequenceDiagram", (255, 121, 198)),
        ("读者", (241, 250, 140)),
        ("你好", (241, 250, 140)),
        ("嵌套", (241, 250, 140)),
    ] {
        assert_eq!(
            style("README.md", MARKDOWN, token).foreground,
            color,
            "{token}"
        );
    }
    for prose in ["正文在代码块外", "代码块后的正文"] {
        assert_eq!(
            style("README.md", MARKDOWN, prose).foreground,
            (248, 248, 242)
        );
    }
}

#[test]
fn markdown_and_standalone_files_share_all_mermaid_token_styles() {
    for source in [FLOWCHART, SEQUENCE] {
        let markdown = format!("# 图\n\n```mermaid\n{source}```\n\n结束。\n");
        let body_start = markdown.find(source).unwrap();
        let nested = document("README.md", &markdown).spans();
        let direct = document("diagram.mmd", source).spans();
        for span in direct {
            for offset in span.range {
                assert_eq!(
                    nested
                        .iter()
                        .find(|s| s.range.contains(&(body_start + offset)))
                        .map(|s| s.style),
                    Some(span.style),
                    "source offset {offset}"
                );
            }
        }
    }
}

#[test]
fn markdown_container_prefixes_do_not_enter_the_fenced_language() {
    for source in [
        "> ```mermaid\n> flowchart TD\n>     A --> B[\"标签 > 内容\"]\n> ```\n",
        "> > ```mermaid\n> > flowchart TD\n> >     A --> B[\"标签 > 内容\"]\n> > ```\n",
        "- Diagram:\n\n  ```mermaid\n  flowchart TD\n      A --> B[\"标签 > 内容\"]\n  ```\n",
    ] {
        assert_eq!(
            style("README.md", source, "标签 > 内容").foreground,
            (241, 250, 140)
        );
        assert_eq!(
            style("README.md", source, "-->").foreground,
            (255, 121, 198)
        );
    }
    let rust = "> ```rust\n> fn main() {\n>     let message = \"text > content\";\n> }\n> ```\n";
    assert_eq!(style("README.md", rust, "let").foreground, (255, 121, 198));
    assert_eq!(
        style("README.md", rust, "text > content").foreground,
        (241, 250, 140)
    );
}

#[test]
fn supports_other_common_diagrams_and_recovers_incomplete_input() {
    for (source, token, color) in [
        (
            "classDiagram\nclass Animal\nAnimal <|-- Duck\n",
            "Animal",
            (139, 233, 253),
        ),
        (
            "stateDiagram-v2\n[*] --> Ready\nReady --> Done: 完成\n",
            "完成",
            (241, 250, 140),
        ),
        (
            "erDiagram\nCUSTOMER ||--o{ ORDER : places\n",
            "CUSTOMER",
            (139, 233, 253),
        ),
        (
            "gantt\nsection Build\nCompile :a1, 2026-01-01, 1d\n",
            "section",
            (255, 121, 198),
        ),
        (
            "pie title Traffic\n\"Success\" : 90\n",
            "90",
            (255, 184, 108),
        ),
        (
            "mindmap\n  root((系统))\n    API\n",
            "系统",
            (241, 250, 140),
        ),
        ("flowchart TD\nA[\"unfinished", "flowchart", (255, 121, 198)),
    ] {
        assert_eq!(
            style("diagram.mmd", source, token).foreground,
            color,
            "{source}"
        );
    }
}

#[test]
fn cli_highlights_mermaid_source_instead_of_rendering_a_diagram() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/fixtures/markdown/mermaid_fences.md");
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
    assert!(
        rendered.contains("\x1b[38;2;255;121;198mflowchart"),
        "{rendered:?}"
    );
    assert!(rendered.contains("sequenceDiagram"));
    assert!(rendered.contains("读者"));
    assert!(rendered.contains("-->"));
    assert!(rendered.contains("代码块后的正文"));
}
