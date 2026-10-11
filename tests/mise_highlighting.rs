use std::path::Path;

use kat::{PreparedDocument, RenderOptions};
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

#[test]
fn common_mise_paths_select_a_profile_without_claiming_generic_toml() {
    let source = "[tasks.test]\nrun = 'echo hello'\n";
    for path in [
        "mise.toml",
        ".mise.toml",
        "mise.local.toml",
        ".mise.local.toml",
        "mise.production.toml",
        ".mise.dev.local.toml",
        "mise.linux.toml",
        ".config/mise.toml",
        "mise/config.toml",
        ".mise/config.local.toml",
        ".config/mise/config.production.local.toml",
        "/etc/mise/config.toml",
        ".config/mise/conf.d/tasks.toml",
        "mise/conf.d/10-build.toml",
    ] {
        let value = analysis(path, source);
        assert_eq!(
            value["detected_document_kind"]["runtime_name"], "toml",
            "{path}"
        );
        assert_eq!(
            value["detected_document_kind"]["profile"], "mise_config",
            "{path}"
        );
        assert_eq!(runtimes(&value), ["bash"], "{path}");
    }
    for path in [
        "config.toml",
        "config.local.toml",
        "other/config.toml",
        "mise.lock",
        ".miserc.toml",
        "mise/conf.d/nested/arbitrary.toml",
        "misleading.toml",
        "mise..toml",
        "..mise.toml",
    ] {
        assert!(runtimes(&analysis(path, source)).is_empty(), "{path}");
    }
}

#[test]
fn fixture_dispatches_only_task_commands() {
    let source = include_str!("../testdata/fixtures/toml/mise.toml");
    let value = analysis("mise.toml", source);
    assert_eq!(
        runtimes(&value),
        [
            "bash", "bash", "bash", "bash", "batch", "python", "fish", "zsh"
        ]
    );
    assert!(runtimes(&analysis("ordinary.toml", source)).is_empty());
    assert_eq!(value["nested_regions"][5]["visual_kind"], "rect_block");
    assert_eq!(value["nested_regions"][0]["visual_kind"], "transparent");
}

#[test]
fn quoted_task_names_dotted_keys_and_inline_tables_use_the_same_sinks() {
    let source = r#"
tasks."test.unit".run = 'echo dotted'
tasks.inline = { run = ['echo first', { task = 'other' }, 'echo last'], shell = 'zsh -c' }
[tasks]
short = 'echo short'
"inline.more" = { run = 'print("hello")', shell = 'python3 -c', env = { run = 'not code' } }
[tasks."quoted\u002ename"]
run = 'echo quoted'
[tasks.other.env]
run = 'not code'
"#;
    assert_eq!(
        runtimes(&analysis("mise.toml", source)),
        ["bash", "zsh", "zsh", "bash", "python", "bash"]
    );
}

#[test]
fn explicit_and_inherited_shells_and_shebangs_select_interpreters() {
    let source = r#"
[task_config]
shell = 'fish -c'
[tasks.default]
run = 'echo default'
[tasks.explicit]
shell = '/bin/zsh -c'
run = 'print hello'
[tasks.python]
shell = 'bash -c'
run = '''
#!/usr/bin/env -S python3 -u
print("hello")
'''
[tasks.uv]
run = '''
#!/usr/bin/env -S uv run --script
print("hello")
'''
[tasks.node]
shell = 'node -e'
run = 'console.log("hello")'
[tasks.powershell]
shell = 'pwsh -Command'
run_windows = 'Write-Host hello'
[tasks.unknown]
shell = 'unknown-shell -c'
run = 'echo unknown'
[tasks.dynamic]
shell = '{{vars.shell}} -c'
run = 'echo dynamic'
[tasks.unknown_shebang]
run = '''
#!/usr/bin/env custom-interpreter
echo unknown
'''
"#;
    assert_eq!(
        runtimes(&analysis("mise.toml", source)),
        [
            "fish",
            "zsh",
            "python",
            "python",
            "javascript",
            "powershell"
        ]
    );
}

#[test]
fn task_templates_resolve_only_same_document_shells_and_stop_at_cycles() {
    let source = r#"
[task_templates.python]
shell = 'python -c'
[task_templates.child]
extends = 'python'
[tasks.local]
extends = 'child'
run = 'print("hello")'
[tasks.external]
extends = 'not-in-this-file'
run = 'echo external'
[task_templates.a]
extends = 'b'
[task_templates.b]
extends = 'a'
[tasks.cycle]
extends = 'a'
run = 'echo cycle'
[tasks.override]
extends = 'not-in-this-file'
shell = 'bash -c'
run = 'echo explicit'
"#;
    assert_eq!(runtimes(&analysis("mise.toml", source)), ["python", "bash"]);
}

#[test]
fn toml_escapes_continuations_and_templates_keep_source_mapping_valid() {
    let source = r#"
[tasks.escaped]
run = "echo \"中文\"; printf '\u03bb\\n'"
[tasks.multiline]
run = """
echo \
    hello
printf '%s' "{{ vars.name | default(value=\"中文\") }}"
{% for file in vars.files %}
echo "{{ file }}"
{% endfor %}
"""
"#;
    let value = analysis("mise.toml", source);
    assert_eq!(runtimes(&value), ["bash", "bash"]);
    let document = PreparedDocument::detect(
        Some(Path::new("mise.toml")),
        source,
        RenderOptions::with_colors((248, 248, 242), (40, 42, 54)),
    )
    .unwrap();
    let spans = document.spans();
    for span in &spans {
        assert!(source.get(span.range.clone()).is_some());
    }
    for (needle, color) in [("echo", (80, 250, 123)), ("printf", (139, 233, 253))] {
        let offset = source.find(needle).unwrap();
        assert_eq!(
            spans
                .iter()
                .find(|span| span.range.contains(&offset))
                .unwrap()
                .style
                .foreground,
            color,
            "{needle}",
        );
    }
    // Template expressions retain their TOML style instead of receiving Bash styles.
    let offset = source.find("vars.name").unwrap();
    assert_eq!(
        spans
            .iter()
            .find(|span| span.range.contains(&offset))
            .unwrap()
            .style
            .foreground,
        (241, 250, 140)
    );
}

#[test]
fn decoded_shebangs_and_literal_backslashes_have_distinct_meaning() {
    let source = r##"
[tasks.escaped]
run = "#!/usr/bin/env python3\nprint(\"\u4e2d\U00006587\")"
[tasks.literal]
run = 'echo \n literal'
[tasks.crlf]
run = """
#!/usr/bin/env ruby
puts 'hello'
"""
"##;
    assert_eq!(
        runtimes(&analysis("mise.toml", source)),
        ["python", "bash", "ruby"]
    );
    assert_eq!(
        runtimes(&analysis("mise.toml", &source.replace('\n', "\r\n"))),
        ["python", "bash", "ruby"]
    );
}
