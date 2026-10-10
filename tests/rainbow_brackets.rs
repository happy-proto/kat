use std::path::Path;

use kat::{HighlightSpan, HighlightStyle, PreparedDocument, RenderOptions};

const COLORS: [(u8, u8, u8); 6] = [
    (241, 250, 140),
    (189, 147, 249),
    (139, 233, 253),
    (255, 121, 198),
    (80, 250, 123),
    (255, 184, 108),
];

fn options(enabled: bool) -> RenderOptions {
    RenderOptions::with_colors((248, 248, 242), (40, 42, 54)).with_rainbow_brackets(enabled)
}

fn style(spans: &[HighlightSpan], offset: usize) -> HighlightStyle {
    spans
        .iter()
        .find(|span| span.range.contains(&offset))
        .unwrap()
        .style
}

fn assert_pair(source: &str, spans: &[HighlightSpan], open: &str, close: &str, depth: usize) {
    let a = source.find(open).unwrap();
    let b = source.find(close).unwrap();
    assert_eq!(
        style(spans, a).foreground,
        COLORS[depth % COLORS.len()],
        "{open} in {source}"
    );
    assert_eq!(
        style(spans, b).foreground,
        COLORS[depth % COLORS.len()],
        "{close} in {source}"
    );
}

#[test]
fn supported_runtimes_color_pairs_by_structural_depth() {
    for (language, source, depths) in [
        (
            "json",
            "{\"items\": [1, {\"x\": 2}]}",
            vec![0, 1, 2, 2, 1, 0],
        ),
        (
            "rust",
            "fn main() { let a = [Some((1, 2))]; }",
            vec![0, 0, 0, 1, 2, 3, 3, 2, 1, 0],
        ),
        (
            "javascript",
            "const a = [wrap((1 + 2))];",
            vec![0, 1, 2, 2, 1, 0],
        ),
        (
            "typescript",
            "const a: number[] = [wrap((1 + 2))];",
            vec![0, 0, 0, 1, 2, 2, 1, 0],
        ),
        (
            "tsx",
            "const a = <div>{wrap([1])}</div>;",
            vec![0, 1, 2, 2, 1, 0],
        ),
        ("python", "values = [wrap((1, 2))]", vec![0, 1, 2, 2, 1, 0]),
        ("toml", "values = [{ x = [1, 2] }]", vec![0, 1, 2, 2, 1, 0]),
        (
            "pkl",
            "values = new { item = List(List(1)) }",
            vec![0, 1, 2, 2, 1, 0],
        ),
        ("sql", "SELECT coalesce((1 + 2), 0);", vec![0, 1, 1, 0]),
        (
            "sql_postgres",
            "SELECT coalesce((1 + 2), 0);",
            vec![0, 1, 1, 0],
        ),
        (
            "sql_mysql",
            "SELECT coalesce((1 + 2), 0);",
            vec![0, 1, 1, 0],
        ),
        (
            "sql_sqlite",
            "SELECT coalesce((1 + 2), 0);",
            vec![0, 1, 1, 0],
        ),
        (
            "bash",
            "echo \"$(printf %s \"$(echo nested)\")\"",
            vec![0, 1, 1, 0],
        ),
        (
            "fish",
            "echo (string join \x27\x27 (echo nested))",
            vec![0, 1, 1, 0],
        ),
    ] {
        let spans = PreparedDocument::named_language(language, source, options(true))
            .unwrap()
            .spans();
        let offsets: Vec<_> = source
            .char_indices()
            .filter(|(_, ch)| "()[]{}".contains(*ch))
            .map(|(offset, _)| offset)
            .collect();
        assert_eq!(offsets.len(), depths.len(), "{language}");
        for (offset, depth) in offsets.into_iter().zip(depths) {
            assert_eq!(
                style(&spans, offset).foreground,
                COLORS[depth],
                "{language} at {offset}"
            );
        }
    }
}

#[test]
fn strings_comments_and_non_bracket_styles_are_preserved() {
    let source = include_str!("../testdata/fixtures/brackets/nested.rs");
    let colored = PreparedDocument::named_language("rust", source, options(true))
        .unwrap()
        .spans();
    let plain = PreparedDocument::named_language("rust", source, options(false))
        .unwrap()
        .spans();
    let literal_start = source.find('"').unwrap();
    let comment_start = source.find("//").unwrap();
    let comment_end = comment_start + source[comment_start..].find('\n').unwrap();
    for (offset, ch) in source.char_indices() {
        if (literal_start..comment_end).contains(&offset) || !"()[]{}".contains(ch) {
            assert_eq!(
                style(&colored, offset),
                style(&plain, offset),
                "byte {offset}"
            );
        }
    }
    assert_ne!(
        style(&colored, source.find('{').unwrap()),
        style(&plain, source.find('{').unwrap())
    );
}

#[test]
fn embedded_languages_reset_depth_and_preserve_source_offsets() {
    for (path, source) in [
        (
            "example.md",
            include_str!("../testdata/fixtures/brackets/embedded.md"),
        ),
        (
            "Justfile",
            include_str!("../testdata/fixtures/brackets/Justfile"),
        ),
    ] {
        let spans = PreparedDocument::detect(Some(Path::new(path)), source, options(true))
            .unwrap()
            .spans();
        if path.ends_with(".md") {
            assert_pair(source, &spans, "{\"items", "}\n```", 0);
            assert_pair(source, &spans, "[1", "]}", 1);
        }
        assert_pair(source, &spans, "[wrap", "]\n", 0);
        assert_pair(source, &spans, "((1", ")]", 1);
        assert_pair(source, &spans, "(1", "))", 2);
        let disabled = PreparedDocument::detect(Some(Path::new(path)), source, options(false))
            .unwrap()
            .spans();
        assert_eq!(
            style(&disabled, source.find("[wrap").unwrap()).foreground,
            (248, 248, 242)
        );
    }
}

#[test]
fn unmatched_delimiters_do_not_shift_complete_inner_pairs() {
    let source = "const a = [wrap((1 + 2));";
    let spans = PreparedDocument::named_language("javascript", source, options(true))
        .unwrap()
        .spans();
    assert_pair(source, &spans, "((", ");", 0);
    assert_pair(source, &spans, "(1", "))", 1);
    let disabled = PreparedDocument::named_language("javascript", source, options(false))
        .unwrap()
        .spans();
    assert_eq!(
        style(&spans, source.find('[').unwrap()),
        style(&disabled, source.find('[').unwrap())
    );
}

#[test]
fn palette_cycles_and_pairs_remain_stable_across_layout_widths() {
    let source = "((((((((1))))))))";
    let document = PreparedDocument::named_language("python", source, options(true)).unwrap();
    let spans = document.spans();
    for depth in 0..8 {
        assert_eq!(
            style(&spans, depth).foreground,
            COLORS[depth % COLORS.len()]
        );
        assert_eq!(
            style(&spans, source.len() - 1 - depth).foreground,
            COLORS[depth % COLORS.len()]
        );
    }
    for width in [80, 8, 80] {
        document.render(Some(width));
        assert_eq!(document.spans(), spans);
    }
}

#[test]
fn generic_angles_are_colored_but_comparisons_keep_their_style() {
    for (language, source) in [
        (
            "rust",
            "fn demo<T>() { let x: Vec<Option<T>> = vec![]; let less = 1 < 2; }",
        ),
        (
            "typescript",
            "function demo<T>(x: Array<Array<T>>) { return 1 < 2; }",
        ),
        ("pkl", "class Demo<T> { x: List<List<T>> }\nless = 1 < 2"),
    ] {
        let colored = PreparedDocument::named_language(language, source, options(true))
            .unwrap()
            .spans();
        let plain = PreparedDocument::named_language(language, source, options(false))
            .unwrap()
            .spans();
        let comparison = source.find("< 2").unwrap();
        assert_eq!(
            style(&colored, comparison),
            style(&plain, comparison),
            "{language}"
        );
        let generic = source.find("<T>").unwrap();
        assert_eq!(style(&colored, generic).foreground, COLORS[0], "{language}");
        assert_eq!(
            style(&colored, generic + 2).foreground,
            COLORS[0],
            "{language}"
        );
    }
}

#[test]
fn multi_character_delimiters_form_one_pair_and_keep_inner_depths() {
    for (language, source, open, close, inner) in [
        (
            "toml",
            "[[servers]]\nports = [80, 443]\n",
            "[[",
            "]]",
            "[80",
        ),
        ("bash", "[[ $(echo ok) == ok ]]", "[[", "]]", "$("),
        ("bash", "(( 1 + (2 * 3) ))", "((", "))", "(2"),
        ("bash", "echo ${name:-$(echo fallback)}", "${", "}", "$("),
    ] {
        let spans = PreparedDocument::named_language(language, source, options(true))
            .unwrap()
            .spans();
        for delimiter in [open, close] {
            let start = source.find(delimiter).unwrap();
            for offset in start..start + delimiter.len() {
                assert_eq!(
                    style(&spans, offset).foreground,
                    COLORS[0],
                    "{language} at {offset}"
                );
            }
        }
        let depth = usize::from(language != "toml");
        assert_eq!(
            style(&spans, source.find(inner).unwrap()).foreground,
            COLORS[depth],
            "{language}"
        );
    }
}

#[test]
fn injected_sql_and_decoded_hk_commands_color_original_ranges() {
    let shell = "psql <<'SQL'\nSELECT coalesce((1 + 2), 0);\nSQL\n";
    let spans = PreparedDocument::named_language("bash", shell, options(true))
        .unwrap()
        .spans();
    assert_pair(shell, &spans, "((", ");", 0);
    assert_pair(shell, &spans, "(1", "),", 1);

    let pkl = r#"steps { ["demo"] { check = "echo \"$(echo $(echo nested))\"" } }"#;
    let spans = PreparedDocument::detect(Some(Path::new("hk.pkl")), pkl, options(true))
        .unwrap()
        .spans();
    assert_pair(pkl, &spans, "$(echo $(", ")\\\"", 0);
    assert_pair(pkl, &spans, "$(echo nested", "))", 1);
}

#[test]
fn no_color_cli_output_remains_plain_text() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let source = include_str!("../testdata/fixtures/brackets/nested.py");
    let mut child = Command::new(env!("CARGO_BIN_EXE_kat"))
        .args(["--language", "python", "-"])
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, source.as_bytes());
}

#[test]
fn same_runtime_macro_injections_keep_the_host_nesting_depth() {
    let source = "fn main() { let values = vec![Some((1, 2))]; }";
    let spans = PreparedDocument::named_language("rust", source, options(true))
        .unwrap()
        .spans();
    assert_pair(source, &spans, "{", "}", 0);
    assert_pair(source, &spans, "[Some", "]", 1);
    assert_pair(source, &spans, "((", ")];", 2);
    assert_pair(source, &spans, "(1", "))", 3);
}

#[test]
fn literal_brackets_do_not_change_depth_or_string_and_comment_colors() {
    for (language, source) in [
        ("json", r#""([{}])""#),
        ("javascript", "const text = \"([{}])\"; // ([{}])"),
        ("typescript", "const text = \"([{}])\"; // ([{}])"),
        ("tsx", "const text = \"([{}])\"; // ([{}])"),
        ("python", "text = r\"([{}])\" # ([{}])"),
        ("toml", "text = \"([{}])\" # ([{}])"),
        ("pkl", "text = \"([{}])\" // ([{}])"),
        ("bash", "echo '([{}])' # ([{}])"),
        ("fish", "echo '([{}])' # ([{}])"),
        ("sql", "SELECT '([{}])'; -- ([{}])"),
    ] {
        let colored = PreparedDocument::named_language(language, source, options(true))
            .unwrap()
            .spans();
        let plain = PreparedDocument::named_language(language, source, options(false))
            .unwrap()
            .spans();
        assert_eq!(colored, plain, "{language}");
    }
}
