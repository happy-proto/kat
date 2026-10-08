use std::path::Path;

use kat::{HighlightStyle, PreparedDocument, RenderOptions};
use serde_json::Value;

fn analysis(path: &str, source: &str) -> Value {
    serde_json::from_str(&kat::debug_analysis_json(Some(Path::new(path)), source).unwrap()).unwrap()
}

fn runtimes(value: &Value) -> Vec<&str> {
    value["nested_regions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|region| {
            region["resolved_document_kind"]["runtime_name"]
                .as_str()
                .unwrap()
        })
        .collect()
}

fn style(source: &str, path: &str, needle: &str) -> HighlightStyle {
    let offset = source.find(needle).unwrap();
    PreparedDocument::detect(
        Some(Path::new(path)),
        source,
        RenderOptions::with_colors((248, 248, 242), (40, 42, 54)),
    )
    .unwrap()
    .spans()
    .into_iter()
    .find(|span| span.range.contains(&offset))
    .unwrap()
    .style
}

#[test]
fn pkl_detection_and_hk_profiles_are_distinct() {
    for path in ["config.pkl", "PklProject", ".config/hk.pkl", "hk.local.pkl"] {
        assert_eq!(
            kat::detected_language_name(Some(Path::new(path)), "enabled = true\n"),
            Some("pkl")
        );
    }
    assert_eq!(
        kat::detected_language_name(None, "#!/usr/bin/env pkl\nenabled = true\n"),
        Some("pkl")
    );
    let source = "steps { [\"test\"] { check = \"echo hello\" } }";
    assert_eq!(
        runtimes(&analysis("config.pkl", source)),
        Vec::<&str>::new()
    );
    for path in ["hk.pkl", "hk.local.pkl", ".config/hk.pkl"] {
        let value = analysis(path, source);
        assert_eq!(value["detected_document_kind"]["profile"], "hk_config");
        assert_eq!(runtimes(&value), ["bash"]);
    }
    for header in [
        "amends \"./hk.pkl\"\n",
        "amends \"package://github.com/jdx/hk/releases/download/v2.5.0/hk@2.5.0#/Config.pkl\"\n",
    ] {
        assert_eq!(
            runtimes(&analysis("shared.pkl", &format!("{header}{source}"))),
            ["bash"]
        );
    }
    for header in [
        "// amends \"./hk.pkl\"\n",
        "import \"./hk.pkl\"\n",
        "local path = \"./hk.pkl\"\n",
    ] {
        assert!(runtimes(&analysis("shared.pkl", &format!("{header}{source}"))).is_empty());
    }
}

#[test]
fn pkl_types_properties_calls_and_interpolations_have_semantic_styles() {
    let source = include_str!("../testdata/fixtures/pkl/config.pkl");
    assert_eq!(
        style(source, "config.pkl", "class").foreground,
        (255, 121, 198)
    );
    assert_eq!(
        style(source, "config.pkl", "host:").foreground,
        (139, 233, 253)
    );
    assert_eq!(
        style(source, "config.pkl", "UInt16").foreground,
        (139, 233, 253)
    );
    assert_eq!(
        style(source, "config.pkl", "8080").foreground,
        (255, 184, 108)
    );
    assert_eq!(
        style(source, "config.pkl", "localhost").foreground,
        (241, 250, 140)
    );
    assert_eq!(
        style(source, "config.pkl", "endpoint():").foreground,
        (80, 250, 123)
    );
    assert_eq!(
        style(source, "config.pkl", "\\(host)").foreground,
        (255, 121, 198)
    );
    // Locals resolve references to the declaration's property style.
    assert_eq!(
        style(source, "config.pkl", "host):").foreground,
        (139, 233, 253)
    );
    assert_eq!(
        style(source, "config.pkl", "\\##n").foreground,
        (255, 121, 198)
    );
    assert_eq!(
        runtimes(&analysis("config.pkl", source)),
        ["markdown", "regex"]
    );
    let document = PreparedDocument::named_language(
        "pkl",
        source,
        RenderOptions::with_colors((248, 248, 242), (40, 42, 54)),
    )
    .unwrap();
    assert!(!document.spans().is_empty());
}

#[test]
fn hk_fixture_dispatches_commands_and_preserves_host_interpolations() {
    let source = include_str!("../testdata/fixtures/pkl/hk.pkl");
    let value = analysis("hk.pkl", source);
    assert_eq!(runtimes(&value), ["bash", "bash", "fish"]);
    assert_eq!(value["nested_regions"][2]["visual_kind"], "rect_block");
    assert_eq!(style(source, "hk.pkl", "echo").foreground, (80, 250, 123));
    assert_eq!(
        style(source, "hk.pkl", "set label").foreground,
        (139, 233, 253)
    );
    assert_eq!(
        style(source, "hk.pkl", "{{files}}").foreground,
        (255, 121, 198)
    );
    assert_eq!(
        style(source, "hk.pkl", "files}}").foreground,
        (189, 147, 249)
    );
    assert_eq!(
        style(source, "hk.pkl", "\\(label)").foreground,
        (255, 121, 198)
    );
    assert_eq!(
        style(source, "hk.pkl", "label)\\").foreground,
        (139, 233, 253)
    );
    let document = PreparedDocument::detect(
        Some(Path::new("hk.pkl")),
        source,
        RenderOptions::with_colors((248, 248, 242), (40, 42, 54)),
    )
    .unwrap();
    for span in document.spans() {
        assert!(
            source.get(span.range).is_some(),
            "mapped spans must stay on UTF-8 boundaries"
        );
    }
}

#[test]
fn hk_shell_selection_is_local_and_unknown_shells_are_not_guessed() {
    let source = r#"steps {
      ["pwsh"] { shell = "pwsh -Command"; check = "Write-Host hello" }
      ["zsh"] { shell = "/bin/zsh -c"; fix = "print hello" }
      ["python"] { shell = "python3 -c"; check = "print('hello')" }
      ["cmd"] { shell = "cmd /c"; check = "echo hello" }
      ["unknown"] { shell = "custom-shell -c"; check = "echo hello" }
      ["dynamic"] { shell = read("env:SHELL"); check = "echo hello" }
      ["argv"] { check = List("echo", "$HOME") }
      ["plain"] { check = "echo hello"; env { check = "not a command" } }
    }"#;
    assert_eq!(
        runtimes(&analysis("hk.pkl", source)),
        ["powershell", "zsh", "python", "batch", "bash"]
    );
}

#[test]
fn hk_step_helpers_and_platform_scripts_are_supported() {
    let source = r#"local lint = new Step { check = "echo lint" }
    local linters = new Mapping<String, Step> {
      ["lint"] { check_diff = "git diff" }
    }
    steps { ["platform"] {
      check = new Script {
        linux = "echo linux"
        macos = "echo macos"
        windows = "echo windows"
        other = "echo other"
      }
    } }"#;
    assert_eq!(
        runtimes(&analysis("hk.pkl", source)),
        ["bash", "bash", "bash", "bash", "batch", "bash"]
    );
}

#[test]
fn pkl_fences_use_the_same_runtime_and_hk_schema_dispatch() {
    let source =
        "```pkl\namends \"./hk.pkl\"\nsteps { [\"test\"] { check = \"echo hello\" } }\n```\n";
    let value = analysis("README.md", source);
    assert_eq!(runtimes(&value), ["pkl"]);
    assert_eq!(
        value["nested_regions"][0]["child_nested_regions"][0]["resolved_document_kind"]["runtime_name"],
        "bash"
    );
}

#[test]
fn raw_and_escaped_multiline_commands_keep_structure_after_projection() {
    for source in [
        "steps { [\"test\"] { check = \"echo \\\"蓝色\\\"\\nif true; then echo ok; fi\" } }",
        "steps { [\"test\"] { check = ###\"echo 蓝色\\###nif true; then echo ok; fi\"### } }",
        "steps { [\"test\"] { check = \"\"\"\r\n  echo 蓝色\r\n  if true; then echo ok; fi\r\n  \"\"\" } }",
    ] {
        assert_eq!(runtimes(&analysis("hk.pkl", source)), ["bash"]);
        assert_eq!(
            style(source, "hk.pkl", "if true").foreground,
            (255, 121, 198)
        );
    }
    // A command expression must not be interpreted as an independently executable fragment.
    for source in [
        "steps { [\"test\"] { check = \"echo\" + \" hello\" } }",
        "steps { [\"test\"] { check = List(\"echo\", \"hello\") } }",
        "steps { [\"test\"] { check = \"unterminated } }",
    ] {
        assert!(runtimes(&analysis("hk.pkl", source)).is_empty());
    }
}
