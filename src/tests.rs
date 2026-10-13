#[test]
fn debug_semantics_is_empty_for_languages_without_overlays() {
    for language in ["pem", "openpgp_armor", "python_docstring", "json"] {
        assert_eq!(
            crate::api::debug_semantics(language, "plain text").unwrap(),
            "",
            "{language} has no semantic overlays"
        );
    }
}

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    analysis::{AnalysisSnapshot, NestedRegionSnapshot, RegionSegmentSnapshot},
    display_geometry::{display_width, strip_ansi},
    document_kind::DocumentKind,
    document_kind::{DocumentProfile, yaml_document_kind},
    language_runtime::runtime,
    layout::{LayoutRowSnapshot, LayoutSnapshot},
    render_ops::{RenderOpSnapshot, RenderPlanSnapshot},
    sql_dialect::detect_sql_dialect,
    theme::{ColorMode, Theme},
};
use crate::{
    api::analyze_with_theme, api::debug_analysis_json, api::debug_layout_json,
    api::debug_render_ops_json, api::debug_semantics, api::debug_terminal_json,
    api::debug_visual_json, api::highlight_named_language, api::render_plan_with_theme,
    detection::SupportedLanguage, detection::detect_document_kind, detection::detect_language,
    document::render_with_theme, document_kind::plain_document_kind,
    host_injections::InjectionVisualKind, injection::collect_top_level_injection_regions,
    render_data::NestedRegion, render_data::RegionSegment, render_data::StyledSpan,
    semantic_overlays::semantic_capture_spans, span_ops::overlay_nested_region,
    span_ops::overlay_style_spans,
};
use anstyle::RgbColor;
use serde_json::Value;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

struct FixtureCase {
    relative_path: &'static str,
    expect_highlight: bool,
    expected_fragments: &'static [&'static str],
}

const FIXTURE_DATA_AND_MARKUP_FAMILIES: &[&str] = &[
    "cabal",
    "csv",
    "email",
    "json",
    "query",
    "syslog",
    "toml",
    "yaml",
    "markdown",
    "mermaid",
    "asciidoc",
    "armor",
    "bibtex",
    "html",
    "css",
    "vue",
    "svelte",
    "graphql",
    "sql",
    "proto",
    "cel",
    "textproto",
    "latex",
    "textile",
    "todotxt",
    "dotenv",
    "ini",
    "tsv",
    "typst",
    "vimhelp",
    "varlink",
    "xml",
    "plain",
];
const FIXTURE_SHELL_AND_CONFIG_FAMILIES: &[&str] = &[
    "ghostty",
    "pkl",
    "bash",
    "fish",
    "zsh",
    "powershell",
    "batch",
    "dockerfile",
    "just",
    "git_config",
    "ignore",
    "git_link",
    "git_mailmap",
    "git_log",
    "authorized_keys",
    "known_hosts",
    "ssh_config",
    "gitattributes",
    "git_commit",
    "git_rebase",
    "cmakecache",
    "command_help",
    "cpuinfo",
    "crontab",
    "debsources",
    "fortran_namelist",
    "fstab",
    "requirements",
    "strace",
    "todotxt",
];
const FIXTURE_SYSTEMS_PROGRAMMING_FAMILIES: &[&str] = &[
    "rust",
    "c",
    "cpp",
    "go",
    "vhdl",
    "verilog",
    "systemverilog",
    "wgsl",
    "ada",
    "asm",
    "nasm",
    "crystal",
    "d",
    "fortran",
    "fsharp",
    "sml",
];
const FIXTURE_MANAGED_PROGRAMMING_FAMILIES: &[&str] = &[
    "java",
    "kotlin",
    "csharp",
    "groovy",
    "scala",
    "solidity",
    "swift",
    "dart",
    "elixir",
    "vyper",
    "zig",
    "actionscript",
    "erlang",
];
const FIXTURE_SCRIPTING_AND_WEB_PROGRAMMING_FAMILIES: &[&str] = &[
    "cfml",
    "clojure",
    "coffeescript",
    "elm",
    "python",
    "php",
    "javascript",
    "typescript",
    "tsx",
    "ruby",
    "lua",
    "applescript",
    "awk",
    "tcl",
    "vim",
    "stylus",
];
const FIXTURE_INFRA_AND_TEMPLATE_FAMILIES: &[&str] = &[
    "graph_easy",
    "adp",
    "asp",
    "eex",
    "hcl",
    "properties",
    "diff",
    "apache",
    "scss",
    "sass",
    "jq",
    "less",
    "dot",
    "nginx",
    "nix",
    "make",
    "cmake",
    "ninja",
    "jinja",
    "jsp",
    "twig",
    "erb",
];

const FIXTURE_CASES: &[FixtureCase] = &[
    FixtureCase {
        relative_path: "graph_easy/minimal.ge",
        expect_highlight: true,
        expected_fragments: &["R", "Longest", "->"],
    },
    FixtureCase {
        relative_path: "graph_easy/features.ge",
        expect_highlight: true,
        expected_fragments: &["Pipeline", "输入", "请求", "##>", "shape"],
    },
    FixtureCase {
        relative_path: "mermaid/flowchart.mmd",
        expect_highlight: true,
        expected_fragments: &["flowchart", "客户端", "-->", "数据库"],
    },
    FixtureCase {
        relative_path: "mermaid/sequence.mermaid",
        expect_highlight: true,
        expected_fragments: &["sequenceDiagram", "GET /health", "-->>"],
    },
    FixtureCase {
        relative_path: "markdown/mermaid_fences.md",
        expect_highlight: true,
        expected_fragments: &["flowchart", "sequenceDiagram", "读者", "代码块后的正文"],
    },
    FixtureCase {
        relative_path: "pkl/config.pkl",
        expect_highlight: true,
        expected_fragments: &["Server", "host", "8080", "Regex"],
    },
    FixtureCase {
        relative_path: "pkl/hk.pkl",
        expect_highlight: true,
        expected_fragments: &["steps", "check", "{{files}}", "echo"],
    },
    FixtureCase {
        relative_path: "pkl/hk.local.pkl",
        expect_highlight: true,
        expected_fragments: &["steps", "check", "{{files}}", "set"],
    },
    FixtureCase {
        relative_path: "json/basic.json",
        expect_highlight: true,
        expected_fragments: &["\"name\"", "// comment", "true", "3"],
    },
    FixtureCase {
        relative_path: "json/rich.json",
        expect_highlight: true,
        expected_fragments: &["\"theme\"", "\"Dracula\"", "true", "null", "second"],
    },
    FixtureCase {
        relative_path: "query/grammars/lua/queries/highlights.scm",
        expect_highlight: true,
        expected_fragments: &["function_item", "@function", "eq?", "\"lua\""],
    },
    FixtureCase {
        relative_path: "bash/script.sh",
        expect_highlight: true,
        expected_fragments: &["if", "echo", "\"kat\""],
    },
    FixtureCase {
        relative_path: "bash/rich.sh",
        expect_highlight: true,
        expected_fragments: &["#!/usr/bin/env bash", "pipefail", "printf", "=~", "second"],
    },
    FixtureCase {
        relative_path: "bash/sql_heredoc.sh",
        expect_highlight: true,
        expected_fragments: &["psql", "RETURNING", "slug", "sqlite3", "AUTO_INCREMENT"],
    },
    FixtureCase {
        relative_path: "fish/rich.fish",
        expect_highlight: true,
        expected_fragments: &[
            "#!/usr/bin/env fish",
            "function",
            "set",
            "string",
            "render_theme",
        ],
    },
    FixtureCase {
        relative_path: "zsh/rich.zsh",
        expect_highlight: true,
        expected_fragments: &["#!/usr/bin/env zsh", "emulate", "autoload", "[[", "(#i)"],
    },
    FixtureCase {
        relative_path: "toml/Cargo.toml",
        expect_highlight: true,
        expected_fragments: &["package", "name", "\"kat\""],
    },
    FixtureCase {
        relative_path: "toml/Cargo.lock",
        expect_highlight: true,
        expected_fragments: &["version", "package", "\"kat\""],
    },
    FixtureCase {
        relative_path: "toml/rich.toml",
        expect_highlight: true,
        expected_fragments: &["display-name", "released_at", "meta", "showcase"],
    },
    FixtureCase {
        relative_path: "yaml/site-config.yaml",
        expect_highlight: true,
        expected_fragments: &["title", "Dracula", "true", "theme"],
    },
    FixtureCase {
        relative_path: "yaml/actions.yaml",
        expect_highlight: true,
        expected_fragments: &[
            "actions/github-script",
            "console",
            "log",
            "&defaults",
            "*defaults",
        ],
    },
    FixtureCase {
        relative_path: "cel/policy.cel",
        expect_highlight: true,
        expected_fragments: &["request", "admin", "exists", "public"],
    },
    FixtureCase {
        relative_path: "proto/schema.proto",
        expect_highlight: true,
        expected_fragments: &[
            "syntax",
            "proto3",
            "message",
            "ThemePreview",
            "service",
            "GetTheme",
        ],
    },
    FixtureCase {
        relative_path: "textproto/theme.textproto",
        expect_highlight: true,
        expected_fragments: &["name", "\"Dracula\"", "enabled", "true", "tags"],
    },
    FixtureCase {
        relative_path: "latex/theme.tex",
        expect_highlight: true,
        expected_fragments: &[
            "\\documentclass",
            "\\section",
            "\\textbf",
            "\\label",
            "https://example.com/kat",
        ],
    },
    FixtureCase {
        relative_path: "latex/theme.sty",
        expect_highlight: true,
        expected_fragments: &["\\ProvidesPackage", "\\newcommand", "\\texttt", "katTheme"],
    },
    FixtureCase {
        relative_path: "textile/theme.textile",
        expect_highlight: true,
        expected_fragments: &["h1.", "Kat Textile Preview", "*", "bc.", "pre."],
    },
    FixtureCase {
        relative_path: "csv/themes.csv",
        expect_highlight: true,
        expected_fragments: &["name", "Dracula", "true", "Solarized Dark"],
    },
    FixtureCase {
        relative_path: "tsv/themes.tsv",
        expect_highlight: true,
        expected_fragments: &["name", "Dracula", "true", "Monokai"],
    },
    FixtureCase {
        relative_path: "typst/theme.typ",
        expect_highlight: true,
        expected_fragments: &["#let", "= Theme Preview", "*bold*", "```json", "#for"],
    },
    FixtureCase {
        relative_path: "hcl/nomad-job.hcl",
        expect_highlight: true,
        expected_fragments: &[
            "job",
            "datacenters",
            "driver",
            "ENABLE_METRICS",
            "destination",
            "endif",
        ],
    },
    FixtureCase {
        relative_path: "rust/main.rs",
        expect_highlight: true,
        expected_fragments: &["fn", "42", "println"],
    },
    FixtureCase {
        relative_path: "rust/rich.rs",
        expect_highlight: true,
        expected_fragments: &["Renderable", "render", "macro_rules!", "themed!", "true"],
    },
    FixtureCase {
        relative_path: "rust/injections.rs",
        expect_highlight: true,
        expected_fragments: &["SELECT", "regex!", "Regex", "new", "kind"],
    },
    FixtureCase {
        relative_path: "rust/doc_comments.rs",
        expect_highlight: true,
        expected_fragments: &["Preview", "markdown", "list item", "inline code"],
    },
    FixtureCase {
        relative_path: "rust/doc_comments_nested.rs",
        expect_highlight: true,
        expected_fragments: &["Guide", "nested", "Nested", "return", "42"],
    },
    FixtureCase {
        relative_path: "actionscript/ThemePreview.as",
        expect_highlight: true,
        expected_fragments: &["package", "class", "function", "return"],
    },
    FixtureCase {
        relative_path: "ada/theme_preview.adb",
        expect_highlight: true,
        expected_fragments: &["procedure", "Theme_Preview", "begin", "Put_Line"],
    },
    FixtureCase {
        relative_path: "asm/theme.s",
        expect_highlight: true,
        expected_fragments: &["theme_preview:", "mov", "x0", "#42"],
    },
    FixtureCase {
        relative_path: "nasm/theme.asm",
        expect_highlight: true,
        expected_fragments: &[
            "section .text",
            "global theme_preview",
            "mov",
            "rax",
            "syscall",
        ],
    },
    FixtureCase {
        relative_path: "cabal/kat.cabal",
        expect_highlight: true,
        expected_fragments: &["name:", "kat", "build-depends", "base"],
    },
    FixtureCase {
        relative_path: "cfml/theme.cfm",
        expect_highlight: true,
        expected_fragments: &["<cfset", "renderTheme", "<cfoutput>", "#theme#"],
    },
    FixtureCase {
        relative_path: "clojure/theme.clj",
        expect_highlight: true,
        expected_fragments: &["defn", "render-theme", ":theme/name", "println"],
    },
    FixtureCase {
        relative_path: "cmakecache/CMakeCache.txt",
        expect_highlight: true,
        expected_fragments: &["KAT_THEME", "STRING", "Dracula", "ON"],
    },
    FixtureCase {
        relative_path: "coffeescript/theme.coffee",
        expect_highlight: true,
        expected_fragments: &["class", "ThemePreview", "render", "console.log"],
    },
    FixtureCase {
        relative_path: "coffeescript/theme.coffee.erb",
        expect_highlight: true,
        expected_fragments: &["<%=", "ThemePreview", "->", "@theme.name"],
    },
    FixtureCase {
        relative_path: "command_help/kat.help",
        expect_highlight: true,
        expected_fragments: &["NAME", "SYNOPSIS", "kat render", "--theme"],
    },
    FixtureCase {
        relative_path: "cpuinfo/cpuinfo",
        expect_highlight: true,
        expected_fragments: &["processor", "cpu MHz", "2400.000", "model name"],
    },
    FixtureCase {
        relative_path: "crontab/root.tab",
        expect_highlight: true,
        expected_fragments: &["*/5", "PATH=/usr/local/bin", "kat render", "MAILTO"],
    },
    FixtureCase {
        relative_path: "crystal/theme.cr",
        expect_highlight: true,
        expected_fragments: &["class", "ThemePreview", "def", "puts"],
    },
    FixtureCase {
        relative_path: "d/theme.d",
        expect_highlight: true,
        expected_fragments: &["module", "ThemePreview", "string", "writeln"],
    },
    FixtureCase {
        relative_path: "debsources/sources.list",
        expect_highlight: true,
        expected_fragments: &["deb", "bookworm", "main", "deb.debian.org"],
    },
    FixtureCase {
        relative_path: "elm/Main.elm",
        expect_highlight: true,
        expected_fragments: &["module", "main", "ThemePreview", "text"],
    },
    FixtureCase {
        relative_path: "email/theme.eml",
        expect_highlight: true,
        expected_fragments: &["From:", "Subject:", "<dcjanus@dcjanus.com>", "Dracula"],
    },
    FixtureCase {
        relative_path: "erlang/theme.erl",
        expect_highlight: true,
        expected_fragments: &["-module", "render", "io:format", "\"kat~n\""],
    },
    FixtureCase {
        relative_path: "python/main.py",
        expect_highlight: true,
        expected_fragments: &["class", "def", "return", "42"],
    },
    FixtureCase {
        relative_path: "python/rich_features.py",
        expect_highlight: true,
        expected_fragments: &["dataclass", "ThemePreview", "render", "self", "\"kat\""],
    },
    FixtureCase {
        relative_path: "python/advanced.py",
        expect_highlight: true,
        expected_fragments: &[
            "classmethod",
            "build_default",
            "__init__",
            "property",
            "isinstance",
        ],
    },
    FixtureCase {
        relative_path: "python/injections.py",
        expect_highlight: true,
        expected_fragments: &["CREATE", "AUTOINCREMENT", "WITHOUT", "re", "compile"],
    },
    FixtureCase {
        relative_path: "python/docstrings.py",
        expect_highlight: true,
        expected_fragments: &[
            "Theme preview helpers.",
            ":class:`ThemePreview`",
            "Attributes:",
            "Args:",
            "ThemePreview",
            "from_name",
            "Parameters",
            "Raises:",
            "uppercase : bool",
        ],
    },
    FixtureCase {
        relative_path: "c/rich.c",
        expect_highlight: true,
        expected_fragments: &[
            "#include",
            "Theme",
            "preview_count",
            "\"Dracula\"",
            "printf",
        ],
    },
    FixtureCase {
        relative_path: "c/theme_config.h.in",
        expect_highlight: true,
        expected_fragments: &["#define", "KAT_THEME", "\"Dracula\""],
    },
    FixtureCase {
        relative_path: "cpp/rich.cpp",
        expect_highlight: true,
        expected_fragments: &[
            "class",
            "ThemePreview",
            "render",
            "R\"(theme:dracula)\"",
            "std",
        ],
    },
    FixtureCase {
        relative_path: "cpp/theme_config.hpp.in",
        expect_highlight: true,
        expected_fragments: &["namespace", "kat", "inline", "theme_name"],
    },
    FixtureCase {
        relative_path: "java/ThemePreview.java",
        expect_highlight: true,
        expected_fragments: &[
            "package",
            "ThemePreview",
            "DEFAULT_THEME",
            "String",
            "println",
        ],
    },
    FixtureCase {
        relative_path: "kotlin/ThemePreview.kt",
        expect_highlight: true,
        expected_fragments: &["package", "data", "ThemePreview", "render", "println"],
    },
    FixtureCase {
        relative_path: "typescript/rich.ts",
        expect_highlight: true,
        expected_fragments: &[
            "interface",
            "ThemePreview",
            "Renderer",
            "constructor",
            "render",
        ],
    },
    FixtureCase {
        relative_path: "tsx/component.tsx",
        expect_highlight: true,
        expected_fragments: &["CardProps", "PreviewCard", "section", "className", "active"],
    },
    FixtureCase {
        relative_path: "go/rich.go",
        expect_highlight: true,
        expected_fragments: &[
            "package",
            "Renderer",
            "DefaultTheme",
            "regexp",
            "SELECT",
            "RETURNING",
            "AUTO_INCREMENT",
            "WITHOUT",
            "themes",
            "MustCompile",
            "console",
            "log",
        ],
    },
    FixtureCase {
        relative_path: "go/go.mod",
        expect_highlight: true,
        expected_fragments: &[
            "module",
            "github.com/example/kat",
            "toolchain",
            "go1.24.1",
            "replace",
            "../forks/lipgloss",
            "retract",
        ],
    },
    FixtureCase {
        relative_path: "go/go.work",
        expect_highlight: true,
        expected_fragments: &[
            "go",
            "1.24.0",
            "use",
            "../shared/theme-kit",
            "replace",
            "github.com/example/theme-kit",
        ],
    },
    FixtureCase {
        relative_path: "go/go.sum",
        expect_highlight: true,
        expected_fragments: &[
            "github.com/charmbracelet/lipgloss",
            "v1.0.0",
            "go.mod",
            "h1",
            "+incompatible",
        ],
    },
    FixtureCase {
        relative_path: "fortran/theme.f90",
        expect_highlight: true,
        expected_fragments: &["program", "theme_preview", "print", "end program"],
    },
    FixtureCase {
        relative_path: "fortran_namelist/theme.namelist",
        expect_highlight: true,
        expected_fragments: &["&theme", "name", ".true.", "/"],
    },
    FixtureCase {
        relative_path: "fsharp/Theme.fs",
        expect_highlight: true,
        expected_fragments: &["module", "let", "render", "printfn"],
    },
    FixtureCase {
        relative_path: "fsharp/Theme.fsi",
        expect_highlight: true,
        expected_fragments: &["module", "type", "member", "val", "render"],
    },
    FixtureCase {
        relative_path: "sml/theme.sml",
        expect_highlight: true,
        expected_fragments: &["structure", "ThemePreview", "fun", "print"],
    },
    FixtureCase {
        relative_path: "fstab/fstab",
        expect_highlight: true,
        expected_fragments: &["UUID=0123-ABCD", "/", "ext4", "defaults"],
    },
    FixtureCase {
        relative_path: "html/index.html",
        expect_highlight: true,
        expected_fragments: &["style", "--accent", "@param", "console", "log"],
    },
    FixtureCase {
        relative_path: "html/nested_regions.html",
        expect_highlight: true,
        expected_fragments: &["style", "--accent", "@param", "console", "greet"],
    },
    FixtureCase {
        relative_path: "html/attribute_injections.html",
        expect_highlight: true,
        expected_fragments: &["style", "--accent", "onclick", "console", "clicked"],
    },
    FixtureCase {
        relative_path: "html/rich.html",
        expect_highlight: true,
        expected_fragments: &["Tom", "&amp;", "theme-card", "accent", "Dracula"],
    },
    FixtureCase {
        relative_path: "html/page.xhtml",
        expect_highlight: true,
        expected_fragments: &["section", "data-theme", "const", "title"],
    },
    FixtureCase {
        relative_path: "html/page.shtml",
        expect_highlight: true,
        expected_fragments: &["section", "panel", "SSI host"],
    },
    FixtureCase {
        relative_path: "html/component.htc",
        expect_highlight: true,
        expected_fragments: &["PUBLIC:COMPONENT", "script", "function", "attach"],
    },
    FixtureCase {
        relative_path: "html/template.yaws",
        expect_highlight: true,
        expected_fragments: &["section", "panel", "YAWS host"],
    },
    FixtureCase {
        relative_path: "vue/component.vue",
        expect_highlight: true,
        expected_fragments: &["template", "section", "Props", "title", "color"],
    },
    FixtureCase {
        relative_path: "svelte/component.svelte",
        expect_highlight: true,
        expected_fragments: &["export", "title", "if", "panel", "color"],
    },
    FixtureCase {
        relative_path: "css/theme.css",
        expect_highlight: true,
        expected_fragments: &["@media", "--accent", "panel"],
    },
    FixtureCase {
        relative_path: "css/rich.css",
        expect_highlight: true,
        expected_fragments: &["@supports", "app", "panel", "before", "1.5", "rem"],
    },
    FixtureCase {
        relative_path: "javascript/report.js",
        expect_highlight: true,
        expected_fragments: &["const", "render", "console", "log"],
    },
    FixtureCase {
        relative_path: "javascript/rich.js",
        expect_highlight: true,
        expected_fragments: &[
            "#theme",
            "constructor",
            "/dracula/",
            "gi",
            "html",
            "ThemePreview",
        ],
    },
    FixtureCase {
        relative_path: "javascript/component.jsx",
        expect_highlight: true,
        expected_fragments: &["Panel", "section", "className", "PreviewCard", "title"],
    },
    FixtureCase {
        relative_path: "javascript/tagged_templates.js",
        expect_highlight: true,
        expected_fragments: &["css", "--accent", "\"enabled\"", "html", "section"],
    },
    FixtureCase {
        relative_path: "javascript/graphql.js",
        expect_highlight: true,
        expected_fragments: &["fragment", "ThemeFields", "ThemeBySlug", "$", "themes"],
    },
    FixtureCase {
        relative_path: "javascript/injections.js",
        expect_highlight: true,
        expected_fragments: &["SELECT", "snapshotQuery", "RETURNING", "kind", "slug"],
    },
    FixtureCase {
        relative_path: "javascript/userscript.user.js",
        expect_highlight: true,
        expected_fragments: &[
            "==UserScript==",
            "name",
            "GM_addStyle",
            "https://example.com/lib.js",
            "document-start",
        ],
    },
    FixtureCase {
        relative_path: "markdown/sample.md",
        expect_highlight: true,
        expected_fragments: &[
            "Preview",
            "layout",
            "\"solarized\"",
            "emphasis",
            "strong",
            "strikethrough",
            "done item",
            "pending item",
            "Column",
            "site",
            "let value = 42",
            "```",
            "return",
            "\"mode\"",
            "\"showcase\"",
        ],
    },
    FixtureCase {
        relative_path: "asciidoc/guide.adoc",
        expect_highlight: true,
        expected_fragments: &["= Kat", "*strong*", "https://example.com", "NOTE:"],
    },
    FixtureCase {
        relative_path: "bibtex/references.bib",
        expect_highlight: true,
        expected_fragments: &["@article", "kat", "title", "Dracula"],
    },
    FixtureCase {
        relative_path: "markdown/nested_runtime.md",
        expect_highlight: true,
        expected_fragments: &[
            "Nested Runtime",
            "draft",
            "emphasis",
            "strong",
            "strike",
            "Preview",
            "render",
            "\"enabled\"",
            "inline html block",
        ],
    },
    FixtureCase {
        relative_path: "markdown/userscript_fence.md",
        expect_highlight: true,
        expected_fragments: &[
            "Userscript Fence",
            "==UserScript==",
            "grant",
            "GM_addStyle",
            "document-end",
        ],
    },
    FixtureCase {
        relative_path: "markdown/go_fence.md",
        expect_highlight: true,
        expected_fragments: &["Go Fence", "package", "Renderer", "Render", "NewRenderer"],
    },
    FixtureCase {
        relative_path: "markdown/graphql_fence.md",
        expect_highlight: true,
        expected_fragments: &["GraphQL Fence", "query", "ThemeBySlug", "$", "fragment"],
    },
    FixtureCase {
        relative_path: "markdown/sql_dialects.md",
        expect_highlight: true,
        expected_fragments: &["SQL Dialects", "RETURNING", "AUTO_INCREMENT", "WITHOUT"],
    },
    FixtureCase {
        relative_path: "just/Justfile",
        expect_highlight: true,
        expected_fragments: &["build", "cargo", "--target", "target"],
    },
    FixtureCase {
        relative_path: "just/injections.just",
        expect_highlight: true,
        expected_fragments: &["import", "json", "print", "source", ".env"],
    },
    FixtureCase {
        relative_path: "just/nested_recipes.just",
        expect_highlight: true,
        expected_fragments: &["python-demo", "Preview", "render", "source", "HOME"],
    },
    FixtureCase {
        relative_path: "just/heredoc_recipes.just",
        expect_highlight: true,
        expected_fragments: &["SELECT", "enabled", "ORDER"],
    },
    FixtureCase {
        relative_path: "csharp/ThemePreview.cs",
        expect_highlight: true,
        expected_fragments: &[
            "namespace",
            "ThemePreview",
            "DefaultTheme",
            "Render",
            "Obsolete",
        ],
    },
    FixtureCase {
        relative_path: "groovy/build.gradle",
        expect_highlight: true,
        expected_fragments: &[
            "plugins",
            "ThemePreview",
            "DEFAULT_THEME",
            "render",
            "println",
        ],
    },
    FixtureCase {
        relative_path: "diff/theme_preview.patch",
        expect_highlight: true,
        expected_fragments: &["diff", "src/theme.rs", "@@", "println!(\"kat\");"],
    },
    FixtureCase {
        relative_path: "properties/gradle.properties",
        expect_highlight: true,
        expected_fragments: &[
            "theme",
            "name",
            "Dracula",
            "preview",
            "count",
            "example.com",
        ],
    },
    FixtureCase {
        relative_path: "php/ThemePreview.php",
        expect_highlight: true,
        expected_fragments: &[
            "namespace",
            "ThemePreview",
            "DEFAULT_THEME",
            "render",
            "echo",
        ],
    },
    FixtureCase {
        relative_path: "scala/ThemePreview.scala",
        expect_highlight: true,
        expected_fragments: &[
            "object",
            "ThemePreview",
            "DefaultTheme",
            "render",
            "println",
        ],
    },
    FixtureCase {
        relative_path: "swift/ThemePreview.swift",
        expect_highlight: true,
        expected_fragments: &[
            "Foundation",
            "nil",
            "ThemePreview",
            "defaultTheme",
            "render",
            "print",
        ],
    },
    FixtureCase {
        relative_path: "solidity/theme.sol",
        expect_highlight: true,
        expected_fragments: &["pragma", "contract", "DEFAULT_THEME", "render"],
    },
    FixtureCase {
        relative_path: "dart/theme_preview.dart",
        expect_highlight: true,
        expected_fragments: &["ThemePreview", "defaultTheme", "render", "main", "print"],
    },
    FixtureCase {
        relative_path: "elixir/theme_preview.ex",
        expect_highlight: true,
        expected_fragments: &[
            "defmodule",
            "ThemePreview",
            "@default_theme",
            "render",
            "IO",
        ],
    },
    FixtureCase {
        relative_path: "vyper/theme.vy",
        expect_highlight: true,
        expected_fragments: &["@deploy", "@external", "self.name", "return"],
    },
    FixtureCase {
        relative_path: "zig/theme.zig",
        expect_highlight: true,
        expected_fragments: &["const", "@import", "std", "main", "debug"],
    },
    FixtureCase {
        relative_path: "vhdl/theme_preview.vhd",
        expect_highlight: true,
        expected_fragments: &[
            "library",
            "entity",
            "theme_preview",
            "rising_edge",
            "std_logic_vector",
        ],
    },
    FixtureCase {
        relative_path: "verilog/theme.v",
        expect_highlight: true,
        expected_fragments: &["module", "always", "active", "1'b1"],
    },
    FixtureCase {
        relative_path: "systemverilog/theme.sv",
        expect_highlight: true,
        expected_fragments: &["module", "logic", "always_ff", "1'b1"],
    },
    FixtureCase {
        relative_path: "wgsl/theme.wgsl",
        expect_highlight: true,
        expected_fragments: &["struct", "@fragment", "vec4<f32>", "return"],
    },
    FixtureCase {
        relative_path: "vim/theme_preview.vim",
        expect_highlight: true,
        expected_fragments: &[
            "let",
            "RenderTheme",
            "autocmd",
            "echohl",
            "embedded lua from vim",
        ],
    },
    FixtureCase {
        relative_path: "stylus/theme.styl",
        expect_highlight: true,
        expected_fragments: &["@import", "$accent", ".panel", "&:hover", "rgba"],
    },
    FixtureCase {
        relative_path: "vimhelp/theme.vimhelp",
        expect_highlight: true,
        expected_fragments: &["Theme Preview", "*theme-preview*", ":KatRender", "Note:"],
    },
    FixtureCase {
        relative_path: "varlink/theme.varlink",
        expect_highlight: true,
        expected_fragments: &["interface", "Theme", "GetTheme", "NotFound"],
    },
    FixtureCase {
        relative_path: "ghostty/config.ghostty",
        expect_highlight: true,
        expected_fragments: &["font-family", "font-size", "keybind", "# literal"],
    },
    FixtureCase {
        relative_path: "known_hosts/known_hosts",
        expect_highlight: true,
        expected_fragments: &[
            "example.com",
            "ssh-ed25519",
            "@cert-authority",
            "@revoked",
            "|1|",
        ],
    },
    FixtureCase {
        relative_path: "ssh_config/.ssh/config",
        expect_highlight: true,
        expected_fragments: &["Host", "HostName", "IdentityFile", "Match", "Port"],
    },
    FixtureCase {
        relative_path: "ssh_config/sshd_config",
        expect_highlight: true,
        expected_fragments: &["Port", "PermitRootLogin", "Match", "ForceCommand"],
    },
    FixtureCase {
        relative_path: "gitattributes/.gitattributes",
        expect_highlight: true,
        expected_fragments: &["rs", "text", "binary", "linguist-generated"],
    },
    FixtureCase {
        relative_path: "git_link/worktree.gitdir",
        expect_highlight: true,
        expected_fragments: &["gitdir", ".git/worktrees/git-language-support"],
    },
    FixtureCase {
        relative_path: "git_mailmap/.mailmap",
        expect_highlight: true,
        expected_fragments: &["DCjanus", "codex@openai.com", "OpenAI Codex"],
    },
    FixtureCase {
        relative_path: "git_log/history.gitlog",
        expect_highlight: true,
        expected_fragments: &[
            "commit",
            "Author:",
            "a/src/lib.rs b/src/lib.rs",
            "@@",
            "Add git metadata",
        ],
    },
    FixtureCase {
        relative_path: "authorized_keys/id_ed25519.pub",
        expect_highlight: true,
        expected_fragments: &["ssh-ed25519", "dcjanusmacbook-pro tailnet", "Example/8"],
    },
    FixtureCase {
        relative_path: "authorized_keys/authorized_keys",
        expect_highlight: true,
        expected_fragments: &["restrict", "command=", "ssh-ed25519", "kat@example"],
    },
    FixtureCase {
        relative_path: "armor/server.pem",
        expect_highlight: true,
        expected_fragments: &["BEGIN CERTIFICATE", "END CERTIFICATE"],
    },
    FixtureCase {
        relative_path: "armor/public.asc",
        expect_highlight: true,
        expected_fragments: &["PGP PUBLIC KEY BLOCK", "Version", "=abcd"],
    },
    FixtureCase {
        relative_path: "git_commit/COMMIT_EDITMSG",
        expect_highlight: true,
        expected_fragments: &[
            "Add theme preview runtime support",
            "Reviewed-by",
            "DCjanus",
            "Please enter the commit message",
        ],
    },
    FixtureCase {
        relative_path: "git_rebase/git-rebase-todo",
        expect_highlight: true,
        expected_fragments: &["pick", "abc1234", "reword", "cargo test"],
    },
    FixtureCase {
        relative_path: "requirements/requirements.txt",
        expect_highlight: true,
        expected_fragments: &["-r", "httpx", "0.27", "python_version", "example.com"],
    },
    FixtureCase {
        relative_path: "strace/strace",
        expect_highlight: true,
        expected_fragments: &["read", "\"theme\"", "openat", "exited"],
    },
    FixtureCase {
        relative_path: "syslog/syslog",
        expect_highlight: true,
        expected_fragments: &["Oct", "kat-host", "sshd", "Accepted publickey"],
    },
    FixtureCase {
        relative_path: "todotxt/todo.txt",
        expect_highlight: true,
        expected_fragments: &[
            "(A)",
            "Finish",
            "+syntax",
            "@terminal",
            "project:kat",
            "x 2026-04-11",
        ],
    },
    FixtureCase {
        relative_path: "apache/httpd.conf",
        expect_highlight: true,
        expected_fragments: &[
            "VirtualHost",
            "ServerName",
            "DocumentRoot",
            "RewriteRule",
            "index.html",
        ],
    },
    FixtureCase {
        relative_path: "scss/theme.scss",
        expect_highlight: true,
        expected_fragments: &["@mixin", "preview-card", "base-color", "@include", "12px"],
    },
    FixtureCase {
        relative_path: "sass/theme.sass",
        expect_highlight: true,
        expected_fragments: &["base-color", "preview-card", "padding", "preview", "color"],
    },
    FixtureCase {
        relative_path: "jq/theme.jq",
        expect_highlight: true,
        expected_fragments: &["def", "preview", "Dracula", "themes", "select"],
    },
    FixtureCase {
        relative_path: "less/theme.less",
        expect_highlight: true,
        expected_fragments: &["base-color", "preview-card", "spacing", "padding"],
    },
    FixtureCase {
        relative_path: "dot/theme.dot",
        expect_highlight: true,
        expected_fragments: &["digraph", "ThemeGraph", "preview", "render", "rankdir"],
    },
    FixtureCase {
        relative_path: "nginx/nginx.conf",
        expect_highlight: true,
        expected_fragments: &["http", "map", "http_x_theme", "location", "try_files"],
    },
    FixtureCase {
        relative_path: "ruby/theme_preview.rb",
        expect_highlight: true,
        expected_fragments: &["class", "DEFAULT_THEME", "initialize", "@name", "puts"],
    },
    FixtureCase {
        relative_path: "applescript/theme.applescript",
        expect_highlight: true,
        expected_fragments: &["on run", "display dialog", "\"kat\"", "end run"],
    },
    FixtureCase {
        relative_path: "awk/theme.awk",
        expect_highlight: true,
        expected_fragments: &["BEGIN", "function render", "printf", "theme"],
    },
    FixtureCase {
        relative_path: "tcl/theme.tcl",
        expect_highlight: true,
        expected_fragments: &["#!/usr/bin/env tclsh", "proc", "set", "puts", "if"],
    },
    FixtureCase {
        relative_path: "lua/theme_preview.lua",
        expect_highlight: true,
        expected_fragments: &["local", "ThemePreview", "function", "setmetatable", "print"],
    },
    FixtureCase {
        relative_path: "nix/flake.nix",
        expect_highlight: true,
        expected_fragments: &[
            "description",
            "outputs",
            "inherit",
            "writeShellScriptBin",
            "Dracula",
        ],
    },
    FixtureCase {
        relative_path: "sql/rich.sql",
        expect_highlight: true,
        expected_fragments: &[
            "WITH",
            "recent_themes",
            "preview_count",
            "$kat$dracula$kat$",
            "featured",
        ],
    },
    FixtureCase {
        relative_path: "sql/schema.postgres.sql",
        expect_highlight: true,
        expected_fragments: &["jsonb", "CONFLICT", "RETURNING", "$tag$dracula$tag$"],
    },
    FixtureCase {
        relative_path: "sql/schema.mysql.sql",
        expect_highlight: true,
        expected_fragments: &["AUTO_INCREMENT", "ENGINE", "REPLACE", "UNSIGNED"],
    },
    FixtureCase {
        relative_path: "sql/schema.sqlite.sql",
        expect_highlight: true,
        expected_fragments: &["AUTOINCREMENT", "STRICT", "WITHOUT", "REPLACE"],
    },
    FixtureCase {
        relative_path: "graphql/schema.graphql",
        expect_highlight: true,
        expected_fragments: &["schema", "Query", "Theme", "fragment", "ThemeBySlug", "$"],
    },
    FixtureCase {
        relative_path: "plain/notes.txt",
        expect_highlight: false,
        expected_fragments: &["plain text"],
    },
    FixtureCase {
        relative_path: "dotenv/.env",
        expect_highlight: true,
        expected_fragments: &[
            "APP_NAME",
            "\"kat\"",
            "ENABLED",
            "DATABASE_URL",
            "# comment",
        ],
    },
    FixtureCase {
        relative_path: "ini/.editorconfig",
        expect_highlight: true,
        expected_fragments: &["root", "*.rs", "indent_style", "indent_size"],
    },
    FixtureCase {
        relative_path: "xml/layout.xml",
        expect_highlight: true,
        expected_fragments: &["layout", "theme", "dracula", "panel", "Preview"],
    },
    FixtureCase {
        relative_path: "make/Makefile",
        expect_highlight: true,
        expected_fragments: &["build", "cargo", "install", "/usr/local/bin/kat"],
    },
    FixtureCase {
        relative_path: "cmake/CMakeLists.txt",
        expect_highlight: true,
        expected_fragments: &["cmake_minimum_required", "project", "kat", "add_executable"],
    },
    FixtureCase {
        relative_path: "ninja/build.ninja",
        expect_highlight: true,
        expected_fragments: &["rule", "command", "build", "main.o", "default"],
    },
    FixtureCase {
        relative_path: "jinja/page.html.jinja",
        expect_highlight: true,
        expected_fragments: &["section", "{{", "title", "{%", "endif"],
    },
    FixtureCase {
        relative_path: "jinja/page.html.j2",
        expect_highlight: true,
        expected_fragments: &["main", "section", "script", "export", "style", "color"],
    },
    FixtureCase {
        relative_path: "jinja/feed.xml.j2",
        expect_highlight: true,
        expected_fragments: &["feed", "title", "{{", "title", "}}"],
    },
    FixtureCase {
        relative_path: "jinja/config.yaml.j2",
        expect_highlight: true,
        expected_fragments: &["sources", "endpoint", "sinks", "healthcheck", "false"],
    },
    FixtureCase {
        relative_path: "jinja/query.sql.j2",
        expect_highlight: true,
        expected_fragments: &["SELECT", "column", "FROM", "WHERE", "TRUE"],
    },
    FixtureCase {
        relative_path: "jinja/theme.css.j2",
        expect_highlight: true,
        expected_fragments: &["preview", "variant", "color", "background", "#282a36"],
    },
    FixtureCase {
        relative_path: "jinja/app.js.j2",
        expect_highlight: true,
        expected_fragments: &["const", "theme_json", "export", "previewName", "name"],
    },
    FixtureCase {
        relative_path: "jinja/app.coffee.j2",
        expect_highlight: true,
        expected_fragments: &["theme_json", "class", "Preview", "render", "theme.name"],
    },
    FixtureCase {
        relative_path: "twig/page.html.twig",
        expect_highlight: true,
        expected_fragments: &["section", "{{", "title", "{%", "endif"],
    },
    FixtureCase {
        relative_path: "jinja/webserver.sls",
        expect_highlight: true,
        expected_fragments: &[
            "webserver",
            "pkg.installed",
            "service.running",
            "enable",
            "true",
        ],
    },
    FixtureCase {
        relative_path: "eex/page.html.eex",
        expect_highlight: true,
        expected_fragments: &["section", "<%=", "@title", "%>"],
    },
    FixtureCase {
        relative_path: "jsp/page.jsp",
        expect_highlight: true,
        expected_fragments: &["section", "<%=", "title", "%>"],
    },
    FixtureCase {
        relative_path: "asp/page.asp",
        expect_highlight: true,
        expected_fragments: &["section", "<%=", "title", "%>"],
    },
    FixtureCase {
        relative_path: "asp/global.asa",
        expect_highlight: true,
        expected_fragments: &["OBJECT", "RUNAT", "Session", "Theme"],
    },
    FixtureCase {
        relative_path: "adp/page.adp",
        expect_highlight: true,
        expected_fragments: &["section", "<%=", "title", "%>"],
    },
    FixtureCase {
        relative_path: "erb/page.html.erb",
        expect_highlight: true,
        expected_fragments: &["section", "<%=", "title", "<%", "Preview"],
    },
    FixtureCase {
        relative_path: "erb/app.js.erb",
        expect_highlight: true,
        expected_fragments: &["const", "theme_name", "document", "dataset"],
    },
    FixtureCase {
        relative_path: "erb/theme.css.erb",
        expect_highlight: true,
        expected_fragments: &["--accent", "accent", ":root"],
    },
    FixtureCase {
        relative_path: "erb/schema.sql.erb",
        expect_highlight: true,
        expected_fragments: &["CREATE", "TABLE", "themes", "<%= accent %>"],
    },
];

#[test]
fn fixture_suite_groups_cover_all_fixtures() {
    let covered_families = FIXTURE_DATA_AND_MARKUP_FAMILIES
        .iter()
        .chain(FIXTURE_SHELL_AND_CONFIG_FAMILIES.iter())
        .chain(FIXTURE_SYSTEMS_PROGRAMMING_FAMILIES.iter())
        .chain(FIXTURE_MANAGED_PROGRAMMING_FAMILIES.iter())
        .chain(FIXTURE_SCRIPTING_AND_WEB_PROGRAMMING_FAMILIES.iter())
        .chain(FIXTURE_INFRA_AND_TEMPLATE_FAMILIES.iter())
        .copied()
        .collect::<std::collections::BTreeSet<_>>();

    for case in FIXTURE_CASES {
        let family = fixture_family(case.relative_path);
        assert!(
            covered_families.contains(family),
            "fixture family {family:?} is not assigned to any fixture suite group"
        );
    }
}

#[test]
fn fixture_suite_matches_data_and_markup_rendering_behavior() {
    run_fixture_suite(FIXTURE_DATA_AND_MARKUP_FAMILIES);
}

#[test]
fn proto_annotations_highlight_options_http_templates_and_cel() {
    let path = fixture_path("proto/annotations.proto");
    let source = read_file(&path);
    let json = debug_analysis_json(Some(path.as_path()), &source)
        .expect("proto annotations should be analyzed");
    let analysis: Value = serde_json::from_str(&json).expect("analysis JSON should parse");
    let color_at = |needle: &str| {
        let offset = source.find(needle).expect("fixture fragment should exist");
        analysis["spans"]
            .as_array()
            .expect("analysis should have spans")
            .iter()
            .find(|span| {
                span["start"]
                    .as_u64()
                    .is_some_and(|start| start <= offset as u64)
                    && span["end"].as_u64().is_some_and(|end| end > offset as u64)
            })
            .and_then(|span| span["style"]["color_name"].as_str())
            .unwrap_or("none")
            .to_owned()
    };

    for option in ["buf.validate.field", "validate.rules", "google.api.http"] {
        assert_ne!(
            color_at(option),
            "foreground",
            "option {option} should be styled"
        );
    }
    for key in ["min_len", "additional_bindings", "expression:"] {
        assert_ne!(
            color_at(key),
            "foreground",
            "option key {key} should be styled"
        );
    }
    assert_ne!(color_at("parent=publishers/*"), "yellow");
    assert_eq!(color_at("**"), "pink");
    assert_ne!(color_at("create\""), "yellow");
    assert_ne!(color_at("has(this.title)"), "yellow");
    assert_eq!(color_at("parent != ''"), "cyan");
    assert_ne!(color_at("matches(\\\""), "yellow");
    assert_eq!(color_at("body: \"*\""), "cyan");
    assert_eq!(color_at("\"*\""), "yellow");
    let cel_regions = analysis["nested_regions"]
        .as_array()
        .expect("expected nested regions")
        .iter()
        .filter(|region| region["resolved_document_kind"]["runtime_name"] == "cel")
        .count();
    assert_eq!(
        cel_regions, 2,
        "field and message CEL should both be injected"
    );
}

#[test]
fn cel_file_detects_and_highlights_standalone_expression() {
    let path = fixture_path("cel/policy.cel");
    let source = read_file(&path);
    assert_eq!(
        detect_language(Some(path.as_path()), &source),
        Some(SupportedLanguage::Cel)
    );
    let json = debug_analysis_json(Some(path.as_path()), &source)
        .expect("standalone CEL should be analyzed");
    let analysis: Value = serde_json::from_str(&json).expect("analysis JSON should parse");
    assert_eq!(analysis["detected_document_kind"]["runtime_name"], "cel");
    let exists = source
        .find("exists")
        .expect("fixture should contain exists");
    assert!(
        analysis["spans"]
            .as_array()
            .is_some_and(|spans| spans.iter().any(|span| {
                span["start"]
                    .as_u64()
                    .is_some_and(|start| start <= exists as u64)
                    && span["end"].as_u64().is_some_and(|end| end > exists as u64)
                    && span["style"]["color_name"] == "green"
            }))
    );
}

#[test]
fn fixture_suite_matches_shell_and_config_rendering_behavior() {
    run_fixture_suite(FIXTURE_SHELL_AND_CONFIG_FAMILIES);
}

#[test]
fn fixture_suite_matches_systems_programming_rendering_behavior() {
    run_fixture_suite(FIXTURE_SYSTEMS_PROGRAMMING_FAMILIES);
}

#[test]
fn swift_nil_literal_is_highlighted_as_a_builtin_constant() {
    let path = fixture_path("swift/ThemePreview.swift");
    let source = read_file(&path);
    let json =
        debug_analysis_json(Some(path.as_path()), &source).expect("Swift should be analyzed");
    let analysis: Value = serde_json::from_str(&json).expect("analysis JSON should parse");
    let start = source.find("nil").expect("fixture should contain nil") as u64;
    assert!(analysis["spans"].as_array().unwrap().iter().any(|span| {
        span["start"].as_u64().is_some_and(|offset| offset <= start)
            && span["end"]
                .as_u64()
                .is_some_and(|offset| offset >= start + 3)
            && span["style"]["color_name"] == "purple"
    }));
}

#[test]
fn fixture_suite_matches_managed_programming_rendering_behavior() {
    run_fixture_suite(FIXTURE_MANAGED_PROGRAMMING_FAMILIES);
}

#[test]
fn fixture_suite_matches_scripting_and_web_programming_rendering_behavior() {
    run_fixture_suite(FIXTURE_SCRIPTING_AND_WEB_PROGRAMMING_FAMILIES);
}

#[test]
fn fixture_suite_matches_infra_and_template_rendering_behavior() {
    run_fixture_suite(FIXTURE_INFRA_AND_TEMPLATE_FAMILIES);
}

fn run_fixture_suite(families: &[&str]) {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let mut matched_cases = 0usize;

    for case in FIXTURE_CASES
        .iter()
        .filter(|case| families.contains(&fixture_family(case.relative_path)))
    {
        matched_cases += 1;
        let path = fixture_path(case.relative_path);
        let source = read_file(&path);
        let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
        let render_plan = render_plan_snapshot_for_path(path.as_path(), &source, &theme);
        let rendered_text = render_plan_text(&render_plan);

        if case.expect_highlight {
            assert!(
                analysis.detected_document_kind.is_some(),
                "expected analysis document kind for {}",
                path.display()
            );
            assert!(
                !analysis.spans.is_empty() || !analysis.nested_regions.is_empty(),
                "expected structural IR for {}",
                path.display()
            );
        } else {
            assert_eq!(
                analysis.detected_document_kind,
                None,
                "plain fixture should stay as passthrough at analysis layer: {}",
                path.display()
            );
            assert!(
                analysis.spans.is_empty() && analysis.nested_regions.is_empty(),
                "plain fixture should not synthesize IR spans or nested regions: {}",
                path.display()
            );
        }

        for fragment in case.expected_fragments {
            assert!(
                rendered_text.contains(fragment),
                "missing fragment {fragment:?} for {}",
                path.display()
            );
        }
    }

    assert!(
        matched_cases > 0,
        "fixture suite group should include at least one fixture"
    );
}

const SHOWCASE_MARKUP_AND_CONFIG_FAMILIES: &[&str] = &[
    "graph_easy",
    "ghostty",
    "pkl",
    "cel",
    "csv",
    "css",
    "graphql",
    "hcl",
    "html",
    "jinja",
    "json",
    "markdown",
    "mermaid",
    "latex",
    "proto",
    "query",
    "sql",
    "textile",
    "textproto",
    "toml",
    "todotxt",
    "tsv",
    "typst",
    "syslog",
    "varlink",
    "vimhelp",
    "yaml",
];
const SHOWCASE_SHELL_AND_TOOLING_FAMILIES: &[&str] = &[
    "bash",
    "dockerfile",
    "fish",
    "git_config",
    "git_link",
    "git_mailmap",
    "git_log",
    "ignore",
    "just",
    "strace",
    "todotxt",
    "zsh",
];
const SHOWCASE_PROGRAMMING_LANGUAGE_FAMILIES: &[&str] = &[
    "go",
    "javascript",
    "nasm",
    "python",
    "rust",
    "sml",
    "solidity",
    "stylus",
    "tcl",
    "vhdl",
    "verilog",
    "vim",
    "systemverilog",
    "vyper",
    "wgsl",
];

#[test]
fn showcase_suite_groups_cover_all_showcase_families() {
    let covered_families = SHOWCASE_MARKUP_AND_CONFIG_FAMILIES
        .iter()
        .chain(SHOWCASE_SHELL_AND_TOOLING_FAMILIES.iter())
        .chain(SHOWCASE_PROGRAMMING_LANGUAGE_FAMILIES.iter())
        .copied()
        .collect::<std::collections::BTreeSet<_>>();

    for path in collect_files("testdata/showcase") {
        let family = showcase_family(&path);
        assert!(
            covered_families.contains(family),
            "showcase family {family:?} is not assigned to any showcase suite group"
        );
    }
}

#[test]
fn showcase_suite_renders_markup_and_config_without_errors() {
    run_showcase_suite(SHOWCASE_MARKUP_AND_CONFIG_FAMILIES);
}

#[test]
fn showcase_suite_renders_shell_and_tooling_without_errors() {
    run_showcase_suite(SHOWCASE_SHELL_AND_TOOLING_FAMILIES);
}

#[test]
fn showcase_suite_renders_programming_languages_without_errors() {
    run_showcase_suite(SHOWCASE_PROGRAMMING_LANGUAGE_FAMILIES);
}

fn run_showcase_suite(families: &[&str]) {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let mut matched_paths = 0usize;

    for path in collect_files("testdata/showcase")
        .into_iter()
        .filter(|path| families.contains(&showcase_family(path)))
    {
        matched_paths += 1;
        let source = read_file(&path);
        let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
        let render_plan = render_plan_snapshot_for_path(path.as_path(), &source, &theme);

        if is_supported_highlight_path(&path) {
            assert!(
                analysis.detected_document_kind.is_some(),
                "expected structural analysis output for supported showcase {}",
                path.display()
            );
            assert!(
                !render_plan.ops.is_empty(),
                "expected render plan ops for supported showcase {}",
                path.display()
            );
        } else {
            assert!(
                !render_plan_text(&render_plan).is_empty(),
                "unsupported showcase should still preserve text ops: {}",
                path.display()
            );
        }
    }

    assert!(
        matched_paths > 0,
        "showcase suite group should include at least one showcase file"
    );
}

#[test]
fn just_injections_highlight_nested_languages() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("just/injections.just");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198mimport"),
        "expected injected Python keyword styling for import"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108mTrue"),
        "expected injected Python boolean styling for True"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected injected Bash builtin styling for source"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected bun shebang to reuse injected JavaScript builtin styling"
    );
}

#[test]
fn just_shell_aliases_reuse_supported_runtimes() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("just/runtime_matrix.just");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    let zsh_source_count = rendered.matches("\x1b[38;2;139;233;253msource").count();
    assert!(
        zsh_source_count >= 2,
        "expected global zsh shell setting to route both recipe bodies and external commands into zsh runtime"
    );
    assert!(
        rendered.matches("\x1b[38;2;139;233;253mprintf").count() >= 2,
        "expected zsh global shell setting to keep zsh builtin styling across recipe bodies and external commands"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected fish shebang recipe to route recipe body into fish runtime"
    );
}

#[test]
fn fish_and_zsh_detect_by_extension_filename_and_shebang() {
    assert!(matches!(
        detect_language(Some(Path::new(".envrc")), "export THEME=Dracula\n"),
        Some(SupportedLanguage::Bash)
    ));
    assert!(matches!(
        detect_language(Some(Path::new(".bash_aliases")), "alias k='kat'\n"),
        Some(SupportedLanguage::Bash)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new(".bash_completions")),
            "complete -W 'render' kat\n"
        ),
        Some(SupportedLanguage::Bash)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("bashrc")), "export KAT_THEME=dracula\n"),
        Some(SupportedLanguage::Bash)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.bashrc")),
            "export KAT_THEME=dracula\n"
        ),
        Some(SupportedLanguage::Bash)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("PKGBUILD")), "pkgname=kat\n"),
        Some(SupportedLanguage::Bash)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("pkg/kat.ebuild")), "src_install() { :; }\n"),
        Some(SupportedLanguage::Bash)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("pkg/eclass/kat.eclass")),
            "EXPORT_FUNCTIONS src_install\n"
        ),
        Some(SupportedLanguage::Bash)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("/etc/profile")),
            "export PATH=$PATH:/opt/kat\n"
        ),
        Some(SupportedLanguage::Bash)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("/etc/os-release")), "NAME=\"katOS\"\n"),
        Some(SupportedLanguage::Bash)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("/var/run/os-release")), "ID=kat\n"),
        Some(SupportedLanguage::Bash)
    ));
    assert!(matches!(
        detect_language(Some(Path::new(".config/bat/config")), "--theme=Dracula\n"),
        Some(SupportedLanguage::Bash)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("config.fish")), "set theme Dracula\n"),
        Some(SupportedLanguage::Fish)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("fish_variables")),
            "SETUVAR __fish_initialized:4300\n"
        ),
        Some(SupportedLanguage::Fish)
    ));
    assert!(matches!(
        detect_language(Some(Path::new(".zshrc")), "emulate -L zsh\n"),
        Some(SupportedLanguage::Zsh)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("zshrc")), "emulate -L zsh\n"),
        Some(SupportedLanguage::Zsh)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.zshrc")), "emulate -L zsh\n"),
        Some(SupportedLanguage::Zsh)
    ));
    assert!(matches!(
        detect_language(Some(Path::new(".zsh_aliases")), "setopt EXTENDED_GLOB\n"),
        Some(SupportedLanguage::Zsh)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("dracula.zsh-theme")),
            "autoload -Uz colors\n"
        ),
        Some(SupportedLanguage::Zsh)
    ));
    assert!(matches!(
        detect_language(None, "#!/usr/bin/env fish\nset theme Dracula\n"),
        Some(SupportedLanguage::Fish)
    ));
    assert_eq!(
        detect_document_kind(
            Some(Path::new("fish_variables")),
            "SETUVAR fish_user_paths:/opt/homebrew/bin\\x1e/Users/example/bin\n"
        ),
        Some(DocumentKind::with_profile(
            "fish",
            DocumentProfile::FishVariables
        ))
    );
    assert!(matches!(
        detect_language(None, "#!/usr/bin/env zsh\nemulate -L zsh\n"),
        Some(SupportedLanguage::Zsh)
    ));
    assert!(matches!(
        detect_language(Some(Path::new(".gitignore")), "target/\n"),
        Some(SupportedLanguage::Ignore)
    ));
    assert!(matches!(
        detect_language(Some(Path::new(".dockerignore")), "node_modules/\n"),
        Some(SupportedLanguage::Ignore)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("Dockerfile.dockerignore")), ".git\n"),
        Some(SupportedLanguage::Ignore)
    ));
    assert!(matches!(
        detect_language(Some(Path::new(".npmignore")), "dist/\n"),
        Some(SupportedLanguage::Ignore)
    ));
    assert_eq!(
        detect_document_kind(Some(Path::new(".git/info/exclude")), ".direnv/\n")
            .map(|kind| kind.runtime_name()),
        Some("ignore")
    );
    assert_eq!(
        detect_document_kind(Some(Path::new(".config/git/ignore")), "*.local\n")
            .map(|kind| kind.runtime_name()),
        Some("ignore")
    );
    assert!(matches!(
        detect_language(Some(Path::new(".gitconfig")), "[core]\n  editor = nvim\n"),
        Some(SupportedLanguage::GitConfig)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new(".gitmodules")),
            "[submodule \"kat\"]\n  path = vendor/kat\n"
        ),
        Some(SupportedLanguage::GitConfig)
    ));
    assert!(matches!(
        detect_language(Some(Path::new(".git/config")), "[core]\n  bare = false\n"),
        Some(SupportedLanguage::GitConfig)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new(".config/git/config")),
            "[include]\n  path = ~/.gitconfig.local\n"
        ),
        Some(SupportedLanguage::GitConfig)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new(".git/worktrees/demo/config.worktree")),
            "[core]\n  sparseCheckout = true\n"
        ),
        Some(SupportedLanguage::GitConfig)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("Dockerfile")), "FROM alpine:3.21\n"),
        Some(SupportedLanguage::Dockerfile)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("Dockerfile.dev")),
            "RUN printf '%s\\n' \"$HOME\"\n"
        ),
        Some(SupportedLanguage::Dockerfile)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("example.hcl")),
            "job \"api\" { datacenters = [\"dc1\"] }\n"
        ),
        Some(SupportedLanguage::Hcl)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("example.nomad")),
            "job \"api\" { datacenters = [\"dc1\"] }\n"
        ),
        Some(SupportedLanguage::Hcl)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("main.tf")),
            "resource \"null_resource\" \"preview\" {}\n"
        ),
        Some(SupportedLanguage::Hcl)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("terraform.tfvars")),
            "region = \"us-east-1\"\n"
        ),
        Some(SupportedLanguage::Hcl)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("scripts/profile.ps1")),
            "Write-Host \"kat\"\n"
        ),
        Some(SupportedLanguage::Powershell)
    ));
    assert!(matches!(
        detect_language(None, "#!/usr/bin/env pwsh\nWrite-Host \"kat\"\n"),
        Some(SupportedLanguage::Powershell)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("scripts/build.cmd")), "@echo off\r\n"),
        Some(SupportedLanguage::Batch)
    ));

    let workflow_kind = detect_document_kind(
        Some(Path::new(".github/workflows/build.yml")),
        "name: Build\njobs: {}\n",
    )
    .expect("expected GitHub workflow path to detect as YAML");
    assert_eq!(workflow_kind.runtime_name(), "yaml");
    assert_eq!(
        workflow_kind.profile(),
        DocumentProfile::GitHubActionsWorkflow
    );

    let action_kind = detect_document_kind(
        Some(Path::new("action.yml")),
        "name: Example\nruns:\n  using: composite\n",
    )
    .expect("expected action metadata path to detect as YAML");
    assert_eq!(action_kind.runtime_name(), "yaml");
    assert_eq!(action_kind.profile(), DocumentProfile::GitHubActionMetadata);

    let prek_kind = detect_document_kind(
        Some(Path::new("prek.toml")),
        "minimum_prek_version = \"0.3.8\"\n",
    )
    .expect("expected prek.toml path to detect as TOML");
    assert_eq!(prek_kind.runtime_name(), "toml");
    assert_eq!(prek_kind.profile(), DocumentProfile::PrekConfig);

    assert!(matches!(
        detect_language(
            Some(Path::new("proto/theme.proto")),
            "syntax = \"proto3\";\nmessage Theme {}\n",
        ),
        Some(SupportedLanguage::Proto)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("proto/theme.textproto")),
            "name: \"Dracula\"\nenabled: true\n",
        ),
        Some(SupportedLanguage::Textproto)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("docs/theme.tex")),
            "\\section{Theme Preview}\\label{sec:theme}\n",
        ),
        Some(SupportedLanguage::Latex)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.sty")),
            "\\ProvidesPackage{kat-theme}\n",
        ),
        Some(SupportedLanguage::Latex)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.textile")),
            "h1. Kat Textile Preview\n",
        ),
        Some(SupportedLanguage::Textile)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("themes.csv")), "name,active\nDracula,true\n",),
        Some(SupportedLanguage::Csv)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("themes.tsv")),
            "name\tactive\nDracula\ttrue\n",
        ),
        Some(SupportedLanguage::Tsv)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.typ")),
            "#let theme = \"Dracula\"\n= Theme Preview\n",
        ),
        Some(SupportedLanguage::Typst)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.tcl")),
            "#!/usr/bin/env tclsh\nputs \"kat\"\n",
        ),
        Some(SupportedLanguage::Tcl)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("src/theme.c")),
            "typedef struct Theme { int enabled; } Theme;\n",
        ),
        Some(SupportedLanguage::C)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("include/theme.hpp")),
            "class ThemePreview {};\n",
        ),
        Some(SupportedLanguage::Cpp)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("src/ThemePreview.java")),
            "public final class ThemePreview {}\n",
        ),
        Some(SupportedLanguage::Java)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("build.gradle.kts")),
            "plugins { kotlin(\"jvm\") }\n",
        ),
        Some(SupportedLanguage::Kotlin)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("src/theme.ts")),
            "export interface Theme { title: string }\n",
        ),
        Some(SupportedLanguage::TypeScript)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("src/ThemePreview.cs")),
            "public sealed class ThemePreview {}\n",
        ),
        Some(SupportedLanguage::Csharp)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("build.gradle")), "plugins { id 'java' }\n",),
        Some(SupportedLanguage::Groovy)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("Jenkinsfile")), "pipeline { agent any }\n",),
        Some(SupportedLanguage::Groovy)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.patch")), "diff --git a/a b/b\n",),
        Some(SupportedLanguage::Diff)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("gradle.properties")), "theme.name=Dracula\n",),
        Some(SupportedLanguage::Properties)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("filters/theme.jq")), ".themes[] | .name\n",),
        Some(SupportedLanguage::Jq)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("styles/theme.less")), "@color: #6272a4;\n",),
        Some(SupportedLanguage::Less)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("graph/theme.dot")),
            "digraph ThemeGraph { a -> b }\n",
        ),
        Some(SupportedLanguage::Dot)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("/etc/nginx/sites-enabled/default.conf")),
            "server { listen 8080; }\n",
        ),
        Some(SupportedLanguage::Nginx)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("src/component.tsx")),
            "export const View = () => <section />;\n",
        ),
        Some(SupportedLanguage::Tsx)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("src/App.vue")),
            "<template><section /></template>\n",
        ),
        Some(SupportedLanguage::Vue)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("src/App.svelte")),
            "<script>let title = 'kat';</script>\n",
        ),
        Some(SupportedLanguage::Svelte)
    ));
    assert!(matches!(
        detect_language(Some(Path::new(".env")), "APP=kat\n"),
        Some(SupportedLanguage::Dotenv)
    ));
    assert!(matches!(
        detect_language(Some(Path::new(".env.local")), "APP=kat\n"),
        Some(SupportedLanguage::Dotenv)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("env.example")), "APP=kat\n"),
        Some(SupportedLanguage::Dotenv)
    ));
    assert!(matches!(
        detect_language(Some(Path::new(".editorconfig")), "root = true\n"),
        Some(SupportedLanguage::Ini)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("layout.xml")), "<layout />\n"),
        Some(SupportedLanguage::Xml)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("page.xhtml")), "<section />\n"),
        Some(SupportedLanguage::Html)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("page.shtml")), "<section />\n"),
        Some(SupportedLanguage::Html)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("legacy/component.htc")),
            "<PUBLIC:COMPONENT></PUBLIC:COMPONENT>\n",
        ),
        Some(SupportedLanguage::Html)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("templates/page.yaws")),
            "<section></section>\n",
        ),
        Some(SupportedLanguage::Html)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("Makefile")), "build:\n\tcargo test\n"),
        Some(SupportedLanguage::Make)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("CMakeLists.txt")),
            "cmake_minimum_required(VERSION 3.28)\n",
        ),
        Some(SupportedLanguage::Cmake)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("build.ninja")), "build a: phony\n"),
        Some(SupportedLanguage::Ninja)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("templates/page.html.j2")),
            "<section>{{ title }}</section>\n",
        ),
        Some(SupportedLanguage::Jinja)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("templates/feed.xml.j2")),
            "<feed>{{ title }}</feed>\n",
        ),
        Some(SupportedLanguage::Jinja)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("states/webserver.sls")),
            "webserver:\n  test.configurable_test_state:\n",
        ),
        Some(SupportedLanguage::Jinja)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("templates/page.html.twig")),
            "<section>{{ title }}</section>\n",
        ),
        Some(SupportedLanguage::Twig)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("templates/page.html.eex")),
            "<section><%= @title %></section>\n",
        ),
        Some(SupportedLanguage::Eex)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("templates/page.jsp")),
            "<section><%= title %></section>\n",
        ),
        Some(SupportedLanguage::Jsp)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("templates/page.asp")),
            "<section><%= title %></section>\n",
        ),
        Some(SupportedLanguage::Asp)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("templates/global.asa")),
            "<OBJECT RUNAT=Server SCOPE=Session ID=Theme></OBJECT>\n",
        ),
        Some(SupportedLanguage::Asp)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("templates/page.adp")),
            "<section><%= title %></section>\n",
        ),
        Some(SupportedLanguage::Adp)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("templates/page.html.erb")),
            "<section><%= title %></section>\n",
        ),
        Some(SupportedLanguage::Erb)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("templates/app.js.erb")),
            "const theme = \"<%= theme_name %>\";\n",
        ),
        Some(SupportedLanguage::Erb)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("templates/theme.css.erb")),
            ":root { --accent: <%= accent %>; }\n",
        ),
        Some(SupportedLanguage::Erb)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("db/schema.sql.erb")),
            "CREATE TABLE themes (accent TEXT DEFAULT '<%= accent %>');\n",
        ),
        Some(SupportedLanguage::Erb)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("db/schema.erbsql")),
            "SELECT '<%= accent %>';\n",
        ),
        Some(SupportedLanguage::Erb)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("notebook.ipynb")), "{\n  \"cells\": []\n}\n"),
        Some(SupportedLanguage::Json)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("events.jsonl")), "{\"kind\":\"theme\"}\n"),
        Some(SupportedLanguage::Json)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("flake.lock")), "{\n  \"nodes\": {}\n}\n"),
        Some(SupportedLanguage::Json)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("CITATION.cff")), "cff-version: 1.2.0\n"),
        Some(SupportedLanguage::Yaml)
    ));
    assert!(matches!(
        detect_language(Some(Path::new(".clang-format")), "BasedOnStyle: LLVM\n"),
        Some(SupportedLanguage::Yaml)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("README.mkd")), "# kat\n"),
        Some(SupportedLanguage::Markdown)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("Gemfile")),
            "source \"https://rubygems.org\"\n"
        ),
        Some(SupportedLanguage::Ruby)
    ));
    assert!(matches!(
        detect_language(None, "#!/usr/bin/env ruby\nputs ThemePreview.new.render\n"),
        Some(SupportedLanguage::Ruby)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("scripts/theme.nse")),
            "return { action = function() end }\n"
        ),
        Some(SupportedLanguage::Lua)
    ));
    assert!(matches!(
        detect_language(None, "#!/usr/bin/env lua\nprint('kat')\n"),
        Some(SupportedLanguage::Lua)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("flake.nix")),
            "{ outputs = { self }: {}; }\n"
        ),
        Some(SupportedLanguage::Nix)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("schema.ddl")), "CREATE TABLE themes ();"),
        Some(SupportedLanguage::Sql)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("seed.dml")),
            "INSERT INTO themes VALUES (1);"
        ),
        Some(SupportedLanguage::Sql)
    ));
}

#[test]
fn newer_languages_and_special_files_detect_by_path() {
    assert!(matches!(
        detect_language(
            Some(Path::new("ThemePreview.as")),
            "package { public class ThemePreview { public function render():String { return \"kat\"; } } }\n",
        ),
        Some(SupportedLanguage::ActionScript)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme_preview.adb")),
            "procedure Theme_Preview is begin null; end Theme_Preview;\n",
        ),
        Some(SupportedLanguage::Ada)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.ads")),
            "package Theme is end Theme;\n"
        ),
        Some(SupportedLanguage::Ada)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("kat.gpr")), "project Kat is end Kat;\n"),
        Some(SupportedLanguage::Ada)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.applescript")),
            "on run\n  display dialog \"kat\"\nend run\n",
        ),
        Some(SupportedLanguage::AppleScript)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("Script Editor")),
            "tell application \"Finder\" to activate\n",
        ),
        Some(SupportedLanguage::AppleScript)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.s")),
            "theme_preview:\n  mov x0, #42\n",
        ),
        Some(SupportedLanguage::Asm)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.asm")),
            "section .text\nglobal theme_preview\n",
        ),
        Some(SupportedLanguage::Nasm)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.yasm")),
            "section .text\nglobal theme_preview\n",
        ),
        Some(SupportedLanguage::Nasm)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("macros.mac")),
            "%macro save_regs 0\n%endmacro\n",
        ),
        Some(SupportedLanguage::Nasm)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("macros.inc")),
            "%define THEME_NAME \"Dracula\"\n",
        ),
        Some(SupportedLanguage::Nasm)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("guide.adoc")),
            "= Kat\n\nNOTE: render all the things.\n",
        ),
        Some(SupportedLanguage::AsciiDoc)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new(".ssh/authorized_keys")),
            "restrict ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIKb0pQ1wTq8mC9oG0qD1nVwXnQ7lYQd+O4Wn6H2YfA kat@example\n",
        ),
        Some(SupportedLanguage::AuthorizedKeys)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("id_ed25519.pub")),
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIKb0pQ1wTq8mC9oG0qD1nVwXnQ7lYQd+O4Wn6H2YfA kat@example\n",
        ),
        Some(SupportedLanguage::AuthorizedKeys)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("server.pem")),
            "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n",
        ),
        Some(SupportedLanguage::Pem)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("private.key")),
            concat!(
                "-----BEGIN ENCRYPTED PRIVATE ",
                "KEY-----\nMIIB\n-----END ENCRYPTED PRIVATE ",
                "KEY-----\n",
            ),
        ),
        Some(SupportedLanguage::Pem)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("public.asc")),
            "-----BEGIN PGP PUBLIC KEY BLOCK-----\n\nmQEN\n=abcd\n-----END PGP PUBLIC KEY BLOCK-----\n",
        ),
        Some(SupportedLanguage::OpenPgpArmor)
    ));
    assert_eq!(
        detect_language(Some(Path::new("archive.gpg")), "not ascii armor\n"),
        None
    );
    assert!(matches!(
        detect_language(Some(Path::new("theme.awk")), "BEGIN { print \"kat\" }\n"),
        Some(SupportedLanguage::Awk)
    ));
    assert!(matches!(
        detect_language(None, "#!/usr/bin/env awk -f\nBEGIN { print \"kat\" }\n"),
        Some(SupportedLanguage::Awk)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("references.bib")),
            "@article{kat, title = {Dracula}}\n",
        ),
        Some(SupportedLanguage::Bibtex)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("kat.cabal")), "name: kat\n"),
        Some(SupportedLanguage::Cabal)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.cfm")), "<cfset theme = \"kat\">\n"),
        Some(SupportedLanguage::Cfml)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.coffee")), "class ThemePreview\n"),
        Some(SupportedLanguage::CoffeeScript)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("Cakefile")),
            "task 'build', -> console.log 'kat'\n",
        ),
        Some(SupportedLanguage::CoffeeScript)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.clj")),
            "(defn render-theme [] (println \"kat\"))\n",
        ),
        Some(SupportedLanguage::Clojure)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("ThemeCache/CMakeCache.txt")),
            "KAT_THEME:STRING=kat\n",
        ),
        Some(SupportedLanguage::CmakeCache)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("kat.help")), "NAME\n  kat render\n"),
        Some(SupportedLanguage::CommandHelp)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.cpuinfo")), "processor : 0\n"),
        Some(SupportedLanguage::CpuInfo)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("root.tab")), "*/5 * * * * kat render\n"),
        Some(SupportedLanguage::Crontab)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.cr")), "class ThemePreview\nend\n"),
        Some(SupportedLanguage::Crystal)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("/etc/cron.d/kat")),
            "*/5 * * * * root kat render\n",
        ),
        Some(SupportedLanguage::Crontab)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.d")), "module theme;\n"),
        Some(SupportedLanguage::D)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("sources.list")),
            "deb https://deb.debian.org/debian bookworm main\n",
        ),
        Some(SupportedLanguage::Debsources)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("Main.elm")), "module Main exposing (main)\n"),
        Some(SupportedLanguage::Elm)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.eml")), "From: theme@example.com\n"),
        Some(SupportedLanguage::Email)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("/var/mail/dcjanus")),
            "From: theme@example.com\n"
        ),
        Some(SupportedLanguage::Email)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.erl")), "-module(theme).\n"),
        Some(SupportedLanguage::Erlang)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("Emakefile")), "{deps, []}.\n"),
        Some(SupportedLanguage::Erlang)
    ));
    assert!(matches!(
        detect_language(None, "#!/usr/bin/env escript\nmain(_) -> ok.\n"),
        Some(SupportedLanguage::Erlang)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("Theme.fs")),
            "module Theme\nlet render = printfn \"kat\"\n",
        ),
        Some(SupportedLanguage::Fsharp)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("Theme.fsi")),
            "module Theme\nval render: unit -> unit\n",
        ),
        Some(SupportedLanguage::Fsharp)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.f90")),
            "program theme_preview\nend program theme_preview\n",
        ),
        Some(SupportedLanguage::Fortran)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.f")),
            "      program theme_preview\n      end\n",
        ),
        Some(SupportedLanguage::Fortran)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.namelist")), "&theme\n/\n"),
        Some(SupportedLanguage::FortranNamelist)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("Theme.sml")),
            "structure ThemePreview = struct end\n",
        ),
        Some(SupportedLanguage::Sml)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("fstab")),
            "UUID=0123-ABCD / ext4 defaults 0 1\n",
        ),
        Some(SupportedLanguage::Fstab)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme_config.h.in")),
            "#define KAT_THEME \"kat\"\n",
        ),
        Some(SupportedLanguage::C)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme_config.hpp.in")), "namespace kat {}\n"),
        Some(SupportedLanguage::Cpp)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("ThemePreview.php")), "<?php echo 'kat';\n"),
        Some(SupportedLanguage::Php)
    ));
    assert!(matches!(
        detect_language(None, "#!/usr/bin/env php\n<?php echo 'kat';\n"),
        Some(SupportedLanguage::Php)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("build.sbt")), "name := \"kat\"\n"),
        Some(SupportedLanguage::Scala)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("ThemePreview.sol")),
            "pragma solidity ^0.8.28;\ncontract ThemePreview {}\n",
        ),
        Some(SupportedLanguage::Solidity)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("ThemePreview.swift")), "print(\"kat\")\n"),
        Some(SupportedLanguage::Swift)
    ));
    assert!(matches!(
        detect_language(None, "#!/usr/bin/env swift\nprint(\"kat\")\n"),
        Some(SupportedLanguage::Swift)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.dart")), "void main() {}\n"),
        Some(SupportedLanguage::Dart)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.exs")), "IO.puts(\"kat\")\n"),
        Some(SupportedLanguage::Elixir)
    ));
    assert!(matches!(
        detect_language(None, "#!/usr/bin/env elixir\nIO.puts(\"kat\")\n"),
        Some(SupportedLanguage::Elixir)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("build.zig.zon")), ".{ .name = \"kat\" }\n"),
        Some(SupportedLanguage::Zig)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.vy")),
            "@external\ndef render() -> String[32]:\n    return \"kat\"\n",
        ),
        Some(SupportedLanguage::Vyper)
    ));
    assert!(matches!(
        detect_language(Some(Path::new(".ssh/config")), "Host github.com\n"),
        Some(SupportedLanguage::SshConfig)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("ssh_config")), "Host *\n"),
        Some(SupportedLanguage::SshConfig)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("sshd_config")), "Port 22\n"),
        Some(SupportedLanguage::SshConfig)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new(".config/git/attributes")),
            "*.rs text eol=lf\n"
        ),
        Some(SupportedLanguage::GitAttributes)
    ));
    assert_eq!(
        detect_document_kind(
            Some(Path::new(".git")),
            "gitdir: /Users/dcjanus/Code/kat/.git/worktrees/git-language-support\n"
        )
        .map(|kind| kind.runtime_name()),
        Some("git_link")
    );
    assert_eq!(
        detect_document_kind(
            Some(Path::new(".mailmap")),
            "DCjanus <DCjanus@dcjanus.com>\n"
        )
        .map(|kind| kind.runtime_name()),
        Some("git_mailmap")
    );
    assert_eq!(
        detect_document_kind(
            Some(Path::new("history.gitlog")),
            "commit deadbeef\nAuthor: DCjanus <DCjanus@dcjanus.com>\n"
        )
        .map(|kind| kind.runtime_name()),
        Some("git_log")
    );
    assert!(matches!(
        detect_language(Some(Path::new("COMMIT_EDITMSG")), "Add support\n"),
        Some(SupportedLanguage::GitCommit)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("git-rebase-todo")),
            "pick abc1234 add support\n"
        ),
        Some(SupportedLanguage::GitRebase)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("requirements.txt")), "httpx>=0.27\n"),
        Some(SupportedLanguage::Requirements)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("strace")), "read(3, \"theme\", 5) = 5\n"),
        Some(SupportedLanguage::Strace)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.styl")), ".panel\n  color #fff\n"),
        Some(SupportedLanguage::Stylus)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("syslog")),
            "Oct 15 10:42:00 kat-host sshd[1234]: Accepted publickey\n",
        ),
        Some(SupportedLanguage::Syslog)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("todo.txt")),
            "(A) 2026-04-12 Finish kat +syntax @terminal due:2026-04-13\n"
        ),
        Some(SupportedLanguage::TodoTxt)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme_preview.vhd")),
            "entity theme_preview is\nend entity theme_preview;\n"
        ),
        Some(SupportedLanguage::Vhdl)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.v")),
            "module theme_preview; endmodule\n",
        ),
        Some(SupportedLanguage::Verilog)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.sv")),
            "module theme_preview; logic active; endmodule\n",
        ),
        Some(SupportedLanguage::SystemVerilog)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.vh")), "`define THEME_ACCENT 8'hbd\n",),
        Some(SupportedLanguage::SystemVerilog)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("theme.wgsl")),
            "@fragment\nfn main() -> @location(0) vec4<f32> {\n  return vec4<f32>(1.0);\n}\n",
        ),
        Some(SupportedLanguage::Wgsl)
    ));
    assert!(matches!(
        detect_language(Some(Path::new(".vimrc")), "let g:kat_theme = 'dracula'\n"),
        Some(SupportedLanguage::Vim)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.vimhelp")), "= Theme Preview =\n",),
        Some(SupportedLanguage::VimHelp)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.varlink")), "interface io.kat.theme\n",),
        Some(SupportedLanguage::Varlink)
    ));
    assert!(matches!(
        detect_language(
            Some(Path::new("/etc/apache2/sites-enabled/kat.conf")),
            "ServerName kat.example.com\n"
        ),
        Some(SupportedLanguage::Apache)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.scss")), "$color: #fff;\n"),
        Some(SupportedLanguage::Scss)
    ));
    assert!(matches!(
        detect_language(Some(Path::new("theme.sass")), "$color: #fff\n"),
        Some(SupportedLanguage::Sass)
    ));
}

#[test]
fn tree_sitter_query_files_detect_by_grammar_queries_path() {
    let source = read_file(&fixture_path("query/grammars/lua/queries/highlights.scm"));

    assert_eq!(
        crate::api::detected_language_name(
            Some(Path::new("grammars/lua/queries/highlights.scm")),
            &source
        ),
        Some("query")
    );
    assert_eq!(
        crate::api::detected_language_name(
            Some(Path::new("grammars/lua/queries/injections.scm")),
            &source
        ),
        Some("query")
    );
    assert_eq!(
        crate::api::detected_language_name(
            Some(Path::new("grammars/lua/queries/locals.scm")),
            &source
        ),
        Some("query")
    );
    assert_eq!(
        crate::api::detected_language_name(Some(Path::new("queries/highlights.scm")), &source),
        None
    );
}

#[test]
fn tree_sitter_query_files_highlight_captures_predicates_and_comments() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("query/grammars/lua/queries/highlights.scm");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;98;114;164m; Tree-sitter query fixture for kat"),
        "expected query fixture comment styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mfunction_item"),
        "expected query node names to use type styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123m@function"),
        "expected query capture names to use function-like styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253meq?"),
        "expected query predicates to use builtin-function styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"lua\""),
        "expected query string parameters to keep string styling"
    );
}

#[test]
fn fish_and_zsh_highlight_shell_specific_constructs() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let fish_path = fixture_path("fish/rich.fish");
    let fish_source = read_file(&fish_path);
    let fish_rendered = render_with_theme(Some(fish_path.as_path()), &fish_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", fish_path.display()));
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m#!/usr/bin/env fish"),
        "expected fish shebang directive styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;255;121;198mfunction"),
        "expected fish function keyword styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;139;233;253mset"),
        "expected fish builtin command styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;139;233;253mcontains"),
        "expected fish contains builtin styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;189;147;249m--argument-names"),
        "expected fish option styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;139;233;253memit"),
        "expected fish emit builtin styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;139;233;253mfunctions"),
        "expected fish functions builtin styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;139;233;253mtype"),
        "expected fish type builtin styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;80;250;123mcurrent-filename"),
        "expected fish status subcommand directive styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;255;85;85mDra*"),
        "expected fish case glob pattern styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;255;184;108mnew_theme"),
        "expected fish function argument names styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;255;184;108mTHEME_NAME"),
        "expected fish function handler target styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;80;250;123mreplace"),
        "expected fish string subcommand directive styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;189;147;249m$argv"),
        "expected fish argv special variable styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;189;147;249m$status"),
        "expected fish status special variable styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;189;147;249m$fish_pid"),
        "expected fish fish_pid special variable styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;189;147;249m$last_pid"),
        "expected fish last_pid special variable styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;255;121;198m.."),
        "expected fish list range operator styling"
    );

    let zsh_path = fixture_path("zsh/rich.zsh");
    let zsh_source = read_file(&zsh_path);
    let zsh_rendered = render_with_theme(Some(zsh_path.as_path()), &zsh_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", zsh_path.display()));
    assert!(
        zsh_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m#!/usr/bin/env zsh"),
        "expected zsh shebang directive styling"
    );
    assert!(
        zsh_rendered.contains("\x1b[38;2;139;233;253memulate"),
        "expected zsh builtin command styling"
    );
    assert!(
        strip_ansi(&zsh_rendered).contains("[[ $theme_name"),
        "expected zsh test command bracket styling"
    );
    assert!(
        zsh_rendered.contains("\x1b[3m\x1b[38;2;80;250;123mEXTENDED_GLOB"),
        "expected zsh setopt option-name styling"
    );
    assert!(
        zsh_rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected zsh source builtin styling"
    );
    assert!(
        zsh_rendered.contains("\x1b[38;2;139;233;253mread"),
        "expected zsh read builtin styling"
    );
    assert!(
        zsh_rendered.contains("\x1b[3m\x1b[38;2;255;184;108mtheme_line"),
        "expected zsh read target variable styling"
    );
    assert!(
        strip_ansi(&zsh_rendered).contains("$palette[1]"),
        "expected zsh subscript bracket styling"
    );
    assert!(
        zsh_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m:l"),
        "expected zsh expansion modifier styling"
    );
    assert!(
        zsh_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m(I)"),
        "expected zsh subscript flag styling"
    );
}

#[test]
fn fish_variables_profile_detects_and_highlights_persistent_record_syntax() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("fish/fish_variables");
    let source = read_file(&path);
    let document_kind = detect_document_kind(Some(path.as_path()), &source)
        .expect("expected fish_variables fixture to detect as fish");
    let overlays = semantic_capture_spans(document_kind, &source)
        .expect("expected fish_variables semantic overlays to resolve");
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert_eq!(document_kind.runtime_name(), "fish");
    assert_eq!(document_kind.profile(), DocumentProfile::FishVariables);
    assert!(
        overlays.iter().any(|span| {
            span.capture == "keyword.directive" && &source[span.range.clone()] == "SETUVAR"
        }),
        "expected fish_variables profile to capture record directives"
    );
    assert!(
        overlays.iter().any(|span| {
            span.capture == "string.special.key" && &source[span.range.clone()] == "fish_user_paths"
        }),
        "expected fish_variables profile to capture variable names as keys"
    );
    assert!(
        overlays.iter().any(|span| {
            span.capture == "punctuation.special" && &source[span.range.clone()] == "\\x1e"
        }),
        "expected fish_variables profile to treat list separators specially"
    );
    assert!(
        overlays.iter().any(|span| {
            span.capture == "string.escape" && &source[span.range.clone()] == "\\x20"
        }),
        "expected fish_variables profile to capture hex escapes inside values"
    );
    assert!(
        overlays.iter().any(|span| {
            span.capture == "string.special.path"
                && &source[span.range.clone()] == "/opt/homebrew/bin"
        }),
        "expected fish_variables profile to classify path-like list entries"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123mSETUVAR"),
        "expected fish_variables directives to receive directive styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mfish_user_paths"),
        "expected fish_variables keys to reuse config-key styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108m4300"),
        "expected numeric persistent values to receive number styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m/opt/homebrew/bin"),
        "expected path-like values to remain visible and string-highlighted"
    );
}

#[test]
fn powershell_and_batch_highlight_shell_specific_constructs() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let powershell_path = fixture_path("powershell/rich.ps1");
    let powershell_source = read_file(&powershell_path);
    let powershell_rendered =
        render_with_theme(Some(powershell_path.as_path()), &powershell_source, &theme)
            .unwrap_or_else(|error| {
                panic!("failed to render {}: {error}", powershell_path.display())
            });
    assert!(
        powershell_rendered.contains("\x1b[38;2;255;121;198mparam"),
        "expected PowerShell param keyword styling"
    );
    assert!(
        powershell_rendered.contains("\x1b[38;2;139;233;253mstring"),
        "expected PowerShell type styling"
    );
    assert!(
        powershell_rendered.contains("\x1b[38;2;139;233;253mWrite-Host"),
        "expected PowerShell builtin command styling"
    );
    assert!(
        powershell_rendered.contains("\x1b[3m\x1b[38;2;189;147;249m$env:GITHUB_REF"),
        "expected PowerShell environment variables to receive special-variable styling"
    );

    let batch_path = fixture_path("batch/rich.cmd");
    let batch_source = read_file(&batch_path);
    let batch_rendered = render_with_theme(Some(batch_path.as_path()), &batch_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", batch_path.display()));
    assert!(
        batch_rendered.contains("\x1b[38;2;255;121;198m@echo off"),
        "expected batch echo-off keyword styling"
    );
    assert!(
        batch_rendered.contains("\x1b[38;2;255;121;198mset"),
        "expected batch set keyword styling"
    );
    assert!(
        batch_rendered.contains("\x1b[38;2;139;233;253mecho"),
        "expected batch builtin command styling"
    );
    assert!(
        batch_rendered.contains("\x1b[38;2;80;250;123m:build"),
        "expected batch labels to receive dedicated label styling"
    );
    assert!(
        batch_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m:eof"),
        "expected batch :eof targets to receive directive styling"
    );
}

#[test]
fn vhdl_vim_and_todotxt_highlight_language_specific_constructs() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let vhdl_path = fixture_path("vhdl/theme_preview.vhd");
    let vhdl_source = read_file(&vhdl_path);
    let vhdl_rendered = render_with_theme(Some(vhdl_path.as_path()), &vhdl_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", vhdl_path.display()));
    assert!(
        vhdl_rendered.contains("\x1b[38;2;255;121;198mentity"),
        "expected VHDL entity keyword styling"
    );
    assert!(
        vhdl_rendered.contains("\x1b[38;2;139;233;253mstd_logic_vector"),
        "expected VHDL builtin type styling"
    );
    assert!(
        vhdl_rendered.contains("\x1b[38;2;139;233;253mrising_edge"),
        "expected VHDL builtin function styling"
    );

    let vim_path = fixture_path("vim/theme_preview.vim");
    let vim_source = read_file(&vim_path);
    let vim_rendered = render_with_theme(Some(vim_path.as_path()), &vim_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", vim_path.display()));
    assert!(
        vim_rendered.contains("\x1b[38;2;255;121;198mlet"),
        "expected Vim let command styling"
    );
    assert!(
        vim_rendered.contains("\x1b[38;2;80;250;123mRenderTheme"),
        "expected Vim function declaration styling"
    );
    assert!(
        vim_rendered.contains("embedded lua from vim"),
        "expected Vim embedded Lua chunk to render"
    );

    let todotxt_path = fixture_path("todotxt/todo.txt");
    let todotxt_source = read_file(&todotxt_path);
    let todotxt_rendered = render_with_theme(Some(todotxt_path.as_path()), &todotxt_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", todotxt_path.display()));
    assert!(
        todotxt_rendered.contains("\x1b[38;2;255;121;198m(A)"),
        "expected todo.txt priority styling"
    );
    assert!(
        todotxt_rendered.contains("\x1b[38;2;241;250;140m+syntax"),
        "expected todo.txt project styling"
    );
    assert!(
        todotxt_rendered.contains("\x1b[38;2;139;233;253m@terminal"),
        "expected todo.txt context styling"
    );
}

#[test]
fn justfile_windows_shells_reuse_batch_and_powershell_runtimes() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("just/windows_shells.just");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198m@echo off"),
        "expected cmd-backed Justfile recipes to reuse batch highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mWrite-Host"),
        "expected pwsh shebang recipes to reuse PowerShell builtin highlighting"
    );
}

#[test]
fn html_injections_highlight_script_and_style_content() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("html/index.html");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198mstyle"),
        "expected html tag-name styling for style tag"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m#8be9fd"),
        "expected injected css color styling for custom property value"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected injected javascript builtin styling for console"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m@param"),
        "expected jsdoc tag styling for @param"
    );
}

#[test]
fn bash_heredoc_interpreter_blocks_highlight_nested_languages() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("bash/heredoc_injections.sh");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253mPreview"),
        "expected python heredoc to reuse python class styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mrender"),
        "expected python heredoc to reuse python method styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected node heredoc to reuse javascript builtin styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected bash heredoc to reuse bash builtin styling"
    );
}

#[test]
fn html_attribute_injections_highlight_css_and_javascript() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("html/attribute_injections.html");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;80;250;123mvar"),
        "expected injected CSS function styling inside style attribute"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected injected JavaScript builtin styling inside onclick attribute"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198mreturn"),
        "expected injected JavaScript keyword styling inside onclick attribute"
    );
}

#[test]
fn embedded_template_profiles_route_host_content_into_matching_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let js_path = fixture_path("erb/app.js.erb");
    let js_source = read_file(&js_path);
    let js_analysis = analyze_with_theme(Some(js_path.as_path()), &js_source, &theme)
        .expect("failed to analyze js.erb fixture")
        .snapshot();
    assert!(
        nested_runtime_names(&js_analysis).contains(&"javascript"),
        "expected js.erb content to inject into javascript runtime"
    );
    assert!(
        !nested_runtime_names(&js_analysis).contains(&"html"),
        "expected js.erb content to avoid falling back to html runtime"
    );

    let css_path = fixture_path("erb/theme.css.erb");
    let css_source = read_file(&css_path);
    let css_analysis = analyze_with_theme(Some(css_path.as_path()), &css_source, &theme)
        .expect("failed to analyze css.erb fixture")
        .snapshot();
    assert!(
        nested_runtime_names(&css_analysis).contains(&"css"),
        "expected css.erb content to inject into css runtime"
    );

    let xml_path = fixture_path("jinja/feed.xml.j2");
    let xml_source = read_file(&xml_path);
    let xml_analysis = analyze_with_theme(Some(xml_path.as_path()), &xml_source, &theme)
        .expect("failed to analyze xml.j2 fixture")
        .snapshot();
    assert!(
        nested_runtime_names(&xml_analysis).contains(&"xml"),
        "expected xml.j2 content to inject into xml runtime"
    );
    assert!(
        !nested_runtime_names(&xml_analysis).contains(&"html"),
        "expected xml.j2 content to avoid falling back to html runtime"
    );

    let sls_path = fixture_path("jinja/webserver.sls");
    let sls_source = read_file(&sls_path);
    let sls_analysis = analyze_with_theme(Some(sls_path.as_path()), &sls_source, &theme)
        .expect("failed to analyze sls fixture")
        .snapshot();
    assert!(
        nested_runtime_names(&sls_analysis).contains(&"yaml"),
        "expected .sls content to inject into yaml runtime"
    );

    let sql_path = fixture_path("erb/schema.sql.erb");
    let sql_source = read_file(&sql_path);
    let sql_analysis = analyze_with_theme(Some(sql_path.as_path()), &sql_source, &theme)
        .expect("failed to analyze sql.erb fixture")
        .snapshot();
    assert!(
        nested_runtime_names(&sql_analysis).contains(&"sql"),
        "expected sql.erb content to inject into sql runtime"
    );
    assert!(
        !nested_runtime_names(&sql_analysis).contains(&"html"),
        "expected sql.erb content to avoid falling back to html runtime"
    );
}

#[test]
fn jinja_yaml_projection_preserves_mapping_structure_across_expressions() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("jinja/config.yaml.j2");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(&path, &source, &theme);
    let yaml_region = find_nested_region(&analysis, "yaml", "sources:", &source);
    let property_style = theme
        .token_style_for("property.yaml", "key")
        .map(|style| style.snapshot(ColorMode::TrueColor));

    for key in ["scrape_interval_secs", "sinks", "healthcheck", "enabled"] {
        let offset = source
            .find(key)
            .unwrap_or_else(|| panic!("expected source to contain key {key:?}"));
        assert!(
            yaml_region.overlays.iter().any(|span| {
                span.start <= offset && offset < span.end && span.style == property_style
            }),
            "expected YAML key {key:?} after Jinja expressions to retain property styling"
        );
    }
}

#[test]
fn jinja_template_projection_preserves_each_registered_host_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let cases = [
        ("jinja/page.html.j2", "html", "section", "tag"),
        ("jinja/feed.xml.j2", "xml", "feed", "tag"),
        ("jinja/query.sql.j2", "sql", "FROM", "keyword"),
        ("jinja/theme.css.j2", "css", "color", "property"),
        ("jinja/app.js.j2", "javascript", "export", "keyword.import"),
        (
            "jinja/app.coffee.j2",
            "coffeescript",
            "class",
            "keyword.control",
        ),
    ];

    for (relative_path, runtime_name, needle, capture) in cases {
        let path = fixture_path(relative_path);
        let source = read_file(&path);
        let analysis = analysis_snapshot_for_path(&path, &source, &theme);
        let region = find_nested_region(&analysis, runtime_name, needle, &source);
        let offset = source
            .find(needle)
            .unwrap_or_else(|| panic!("expected source to contain {needle:?}"));
        let expected_style = theme
            .token_style_for(capture, needle)
            .map(|style| style.snapshot(ColorMode::TrueColor));

        assert!(
            region.overlays.iter().any(|span| {
                span.start <= offset && offset < span.end && span.style == expected_style
            }),
            "expected {runtime_name} token {needle:?} after a Jinja expression to retain {capture:?} styling"
        );
    }
}

#[test]
fn jinja_html_projection_preserves_script_and_style_injections() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("jinja/page.html.j2");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(&path, &source, &theme);

    for (runtime_name, needle, capture) in [
        ("javascript", "export", "keyword.import"),
        ("css", "color", "property"),
    ] {
        let region = find_nested_region(&analysis, runtime_name, needle, &source);
        let offset = source
            .find(needle)
            .unwrap_or_else(|| panic!("expected source to contain {needle:?}"));
        let expected_style = theme
            .token_style_for(capture, needle)
            .map(|style| style.snapshot(ColorMode::TrueColor));

        assert!(
            region.overlays.iter().any(|span| {
                span.start <= offset && offset < span.end && span.style == expected_style
            }),
            "expected nested {runtime_name} token {needle:?} to retain {capture:?} styling"
        );
    }
}

#[test]
fn template_projection_preserves_html_across_supported_template_engines() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let cases = [
        (
            "templates/page.html.j2",
            "<main>{{ title }}<section>after</section></main>\n",
        ),
        (
            "templates/page.html.twig",
            "<main>{{ title }}<section>after</section></main>\n",
        ),
        (
            "templates/page.html.erb",
            "<main><%= title %><section>after</section></main>\n",
        ),
        (
            "templates/page.html.eex",
            "<main><%= @title %><section>after</section></main>\n",
        ),
        (
            "templates/page.jsp",
            "<main><%= title %><section>after</section></main>\n",
        ),
        (
            "templates/page.asp",
            "<main><%= title %><section>after</section></main>\n",
        ),
        (
            "templates/page.adp",
            "<main><%= $title %><section>after</section></main>\n",
        ),
    ];
    let tag_style = theme
        .token_style_for("tag", "section")
        .map(|style| style.snapshot(ColorMode::TrueColor));

    for (path, source) in cases {
        let analysis = analysis_snapshot_for_path(Path::new(path), source, &theme);
        let region = find_nested_region(&analysis, "html", "section", source);
        let offset = source
            .find("section")
            .expect("expected source to contain section tag");
        assert!(
            region.overlays.iter().any(|span| {
                span.start <= offset && offset < span.end && span.style == tag_style
            }),
            "expected HTML tag after template output to retain styling for {path}"
        );
    }
}

#[test]
fn embedded_template_html_family_reuses_html_host_and_runtime_specific_code() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let eex_path = fixture_path("eex/page.html.eex");
    let eex_source = read_file(&eex_path);
    let eex_analysis = analyze_with_theme(Some(eex_path.as_path()), &eex_source, &theme)
        .expect("failed to analyze eex fixture")
        .snapshot();
    assert!(
        nested_runtime_names(&eex_analysis).contains(&"html"),
        "expected html.eex content to inject into html runtime"
    );
    assert!(
        nested_runtime_names(&eex_analysis).contains(&"elixir"),
        "expected html.eex code to inject into elixir runtime"
    );

    let jsp_path = fixture_path("jsp/page.jsp");
    let jsp_source = read_file(&jsp_path);
    let jsp_analysis = analyze_with_theme(Some(jsp_path.as_path()), &jsp_source, &theme)
        .expect("failed to analyze jsp fixture")
        .snapshot();
    assert!(
        nested_runtime_names(&jsp_analysis).contains(&"html"),
        "expected jsp content to inject into html runtime"
    );
    assert!(
        nested_runtime_names(&jsp_analysis).contains(&"java"),
        "expected jsp code to inject into java runtime"
    );
}

#[test]
fn asp_and_adp_templates_still_reuse_html_host_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    for relative_path in ["asp/page.asp", "adp/page.adp"] {
        let path = fixture_path(relative_path);
        let source = read_file(&path);
        let analysis = analyze_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to analyze {}: {error}", path.display()))
            .snapshot();
        assert!(
            nested_runtime_names(&analysis).contains(&"html"),
            "expected {relative_path} content to inject into html runtime"
        );
    }
}

#[test]
fn coffee_erb_templates_reuse_coffeescript_host_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("coffeescript/theme.coffee.erb");
    let source = read_file(&path);
    let analysis = analyze_with_theme(Some(path.as_path()), &source, &theme)
        .expect("failed to analyze coffee.erb fixture")
        .snapshot();

    assert!(
        nested_runtime_names(&analysis).contains(&"coffeescript"),
        "expected coffee.erb content to inject into coffeescript runtime"
    );
    assert!(
        nested_runtime_names(&analysis).contains(&"ruby"),
        "expected coffee.erb code to inject into ruby runtime"
    );
}

#[test]
fn nomad_template_destination_routes_data_into_target_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("hcl/nomad-template-python.nomad");
    let source = read_file(&path);
    let analysis = analyze_with_theme(Some(path.as_path()), &source, &theme)
        .expect("failed to analyze nomad python template fixture")
        .snapshot();

    assert!(
        nested_runtime_names(&analysis).contains(&"python"),
        "expected destination-driven nomad template content to inject into python runtime"
    );
}

#[test]
fn nomad_template_keeps_hcl_template_syntax_while_highlighting_python_content() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("hcl/nomad-template-python.nomad");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let python_region = find_nested_region(&analysis, "python", "def render_message():", &source);
    let keyword_style = theme
        .token_style_for("keyword", "def")
        .expect("expected python keyword style")
        .snapshot(ColorMode::TrueColor);
    let return_style = theme
        .token_style_for("keyword", "return")
        .expect("expected python return style")
        .snapshot(ColorMode::TrueColor);
    let template_punctuation_style = theme
        .token_style_for("punctuation.special", "${")
        .expect("expected hcl template punctuation style")
        .snapshot(ColorMode::TrueColor);
    let template_keyword_style = theme
        .token_style_for("keyword.control.conditional", "if")
        .expect("expected hcl template conditional style")
        .snapshot(ColorMode::TrueColor);
    let def_offset = source
        .find("def render_message")
        .expect("expected python def");
    let return_offset = source
        .find("return os.environ.get")
        .expect("expected python return statement");
    let interpolation_offset = source
        .find("${NOMAD_ALLOC_ID}")
        .expect("expected interpolation");
    let if_offset = source
        .find("%{ if")
        .expect("expected template if directive")
        + 3;

    assert!(
        region_covers_line(python_region, &source, "def render_message():"),
        "expected python region to cover the function definition line"
    );
    assert!(
        region_covers_line(python_region, &source, "print(\"hello ${NOMAD_ALLOC_ID}\")"),
        "expected python region to keep covering lines that contain hcl interpolations"
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= def_offset
                && def_offset < span.end
                && span.style.as_ref() == Some(&keyword_style)
        }),
        "expected def to keep python keyword styling, got {:?}",
        analysis.spans
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= return_offset
                && return_offset < span.end
                && span.style.as_ref() == Some(&return_style)
        }),
        "expected return after template directives to keep python keyword styling, got {:?}",
        analysis.spans
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= interpolation_offset
                && interpolation_offset < span.end
                && span.style.as_ref() == Some(&template_punctuation_style)
        }),
        "expected ${{...}} to keep hcl template punctuation styling, got {:?}",
        analysis.spans
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= if_offset
                && if_offset < span.end
                && span.style.as_ref() == Some(&template_keyword_style)
        }),
        "expected %{{ if ... }} to keep hcl conditional styling, got {:?}",
        analysis.spans
    );
}

#[test]
fn nomad_template_python_projection_renders_without_layout_panics() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("hcl/nomad-template-python.nomad");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .expect("expected nomad template projection fixture to render");

    assert!(rendered.contains("render_message"));
    assert!(rendered.contains("NOMAD_ALLOC_ID"));
    assert!(rendered.contains("endif"));
}

#[test]
fn nomad_template_python_projection_uses_rectangular_block_tint() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("hcl/nomad-template-python.nomad");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let region = find_nested_region(&analysis, "python", "def render_message():", &source);
    assert_eq!(region.visual_kind, "rect_block");

    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 120);
    let import_row = layout_row_containing(&layout.rows, "import os");
    let blank_row = layout
        .rows
        .iter()
        .find(|row| row.source_line_start == line_start_for_line_number(&source, 7))
        .expect("expected layout to retain the blank line inside the Nomad template body");
    let endif_row = layout_row_containing(&layout.rows, "%{ endif }");
    let expected_bounds = first_background_bounds(import_row)
        .expect("expected projected Python block to have a background");

    assert_eq!(
        expected_bounds.0, 0,
        "expected projected Python block to start at the source left edge"
    );
    assert_eq!(
        first_background_bounds(blank_row),
        Some(expected_bounds),
        "expected blank lines inside the projected template body to keep the same block tint"
    );
    assert_eq!(
        first_background_bounds(endif_row),
        Some(expected_bounds),
        "expected HCL template directive lines inside the projection to keep the same block tint"
    );
}

#[test]
fn go_highlights_generics_methods_builtins_and_directives() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("go/rich.go");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m//go:build linux"),
        "expected go build directive styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;139;233;253mshowcase"),
        "expected package namespace styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mRenderer"),
        "expected go type styling for Renderer"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mNewRenderer"),
        "expected go function definition styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mRender"),
        "expected go method definition styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mmake"),
        "expected go builtin function styling for make"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;139;233;253mstring"),
        "expected go builtin type styling for string"
    );
}

#[test]
fn go_injections_and_markdown_fences_reuse_same_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let go_path = fixture_path("go/rich.go");
    let go_source = read_file(&go_path);
    let go_rendered = render_with_theme(Some(go_path.as_path()), &go_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", go_path.display()));

    assert!(
        go_rendered.contains("\x1b[38;2;255;184;108mtrue"),
        "expected injected JSON boolean styling in Go"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected injected SQL keyword styling in Go"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;255;121;198mRETURNING"),
        "expected Go sql:postgres hint to route through postgres SQL runtime"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;255;121;198mAUTO_INCREMENT"),
        "expected Go sql:mysql hint to route through mysql SQL runtime"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;255;121;198mWITHOUT"),
        "expected Go sql:sqlite hint to route through sqlite SQL runtime"
    );
    assert!(
        go_rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected injected JavaScript builtin styling in Go"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;255;121;198msection"),
        "expected injected HTML tag styling in Go"
    );
    assert!(
        go_rendered.contains("--accent"),
        "expected injected CSS custom property content in Go"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;139;233;253mprintf"),
        "expected injected Bash builtin styling in Go"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;139;233;253msection"),
        "expected injected regex named group styling in Go"
    );
    assert!(
        go_rendered.contains("\x1b[3m\x1b[38;2;139;233;253malpha"),
        "expected Go regexp.MustCompilePOSIX case to reuse posix regex styling"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;139;233;253mescapedSection"),
        "expected Go escaped regex string case to reuse regex named group styling"
    );

    let markdown_path = fixture_path("markdown/go_fence.md");
    let markdown_source = read_file(&markdown_path);
    let markdown_rendered =
        render_with_theme(Some(markdown_path.as_path()), &markdown_source, &theme).unwrap_or_else(
            |error| panic!("failed to render {}: {error}", markdown_path.display()),
        );

    assert!(
        markdown_rendered.contains("\x1b[38;2;139;233;253mRenderer"),
        "expected fenced go block to reuse go type styling"
    );
    assert!(
        markdown_rendered.contains("\x1b[38;2;80;250;123mRender"),
        "expected fenced go block to reuse go method styling"
    );
    assert!(
        markdown_rendered.contains("\x1b[38;2;80;250;123mNewRenderer"),
        "expected fenced golang block alias to reuse go function styling"
    );

    let sql_markdown_path = fixture_path("markdown/sql_dialects.md");
    let sql_markdown_source = read_file(&sql_markdown_path);
    let sql_markdown_rendered = render_with_theme(
        Some(sql_markdown_path.as_path()),
        &sql_markdown_source,
        &theme,
    )
    .unwrap_or_else(|error| panic!("failed to render {}: {error}", sql_markdown_path.display()));
    assert!(
        sql_markdown_rendered.contains("\x1b[38;2;255;121;198mRETURNING"),
        "expected fenced postgres block alias to route through postgres SQL runtime"
    );
    assert!(
        sql_markdown_rendered.contains("\x1b[38;2;255;121;198mAUTO_INCREMENT"),
        "expected fenced mysql block alias to route through mysql SQL runtime"
    );
    assert!(
        sql_markdown_rendered.contains("\x1b[38;2;255;121;198mWITHOUT"),
        "expected fenced sqlite block alias to route through sqlite SQL runtime"
    );
}

#[test]
fn go_module_workspace_and_sum_files_use_dedicated_runtimes() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let gomod_path = fixture_path("go/go.mod");
    let gomod_source = read_file(&gomod_path);
    let gomod_rendered = render_with_theme(Some(gomod_path.as_path()), &gomod_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", gomod_path.display()));
    assert!(
        gomod_rendered.contains("\x1b[38;2;255;121;198mmodule"),
        "expected go.mod directive keyword styling"
    );
    assert!(
        gomod_rendered.contains("\x1b[38;2;139;233;253mgithub.com/example/kat"),
        "expected go.mod module path styling"
    );
    assert!(
        gomod_rendered.contains("\x1b[38;2;241;250;140mgo1.24.1"),
        "expected go.mod toolchain value styling"
    );
    assert!(
        gomod_rendered.contains("\x1b[38;2;139;233;253m../forks/lipgloss"),
        "expected go.mod replacement file path styling"
    );

    let gowork_path = fixture_path("go/go.work");
    let gowork_source = read_file(&gowork_path);
    let gowork_rendered = render_with_theme(Some(gowork_path.as_path()), &gowork_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", gowork_path.display()));
    assert!(
        gowork_rendered.contains("\x1b[38;2;255;121;198muse"),
        "expected go.work use directive styling"
    );
    assert!(
        gowork_rendered.contains("\x1b[38;2;139;233;253m../shared/theme-kit"),
        "expected go.work workspace path styling"
    );

    let gosum_path = fixture_path("go/go.sum");
    let gosum_source = read_file(&gosum_path);
    let gosum_rendered = render_with_theme(Some(gosum_path.as_path()), &gosum_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", gosum_path.display()));
    assert!(
        gosum_rendered.contains("\x1b[38;2;139;233;253mgithub.com/charmbracelet/lipgloss"),
        "expected go.sum module path styling"
    );
    assert!(
        gosum_rendered.contains("\x1b[3m\x1b[38;2;80;250;123mh1"),
        "expected go.sum hash version styling"
    );
    assert!(
        gosum_rendered.contains("\x1b[38;2;255;121;198m+incompatible"),
        "expected go.sum compatibility suffix styling"
    );
}

#[test]
fn nested_fixtures_lock_supported_injection_scenarios() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let markdown_path = fixture_path("markdown/nested_runtime.md");
    let markdown_source = read_file(&markdown_path);
    let markdown_rendered =
        render_with_theme(Some(markdown_path.as_path()), &markdown_source, &theme).unwrap_or_else(
            |error| panic!("failed to render {}: {error}", markdown_path.display()),
        );
    assert!(
        markdown_rendered.contains("title"),
        "expected TOML frontmatter property styling in nested markdown fixture"
    );
    assert!(
        markdown_rendered.contains("\x1b[38;2;139;233;253mPreview"),
        "expected injected Python class styling in nested markdown fixture"
    );
    assert!(
        markdown_rendered.contains("\x1b[38;2;255;184;108mtrue"),
        "expected injected JSON boolean styling in nested markdown fixture"
    );

    let html_path = fixture_path("html/nested_regions.html");
    let html_source = read_file(&html_path);
    let html_rendered = render_with_theme(Some(html_path.as_path()), &html_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", html_path.display()));
    assert!(
        html_rendered.contains("ff79c6"),
        "expected injected CSS styling in nested HTML fixture"
    );
    assert!(
        html_rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected injected JavaScript builtin styling in nested HTML fixture"
    );
    assert!(
        html_rendered.contains("\x1b[38;2;255;121;198m@param"),
        "expected injected JSDoc styling in nested HTML fixture"
    );

    let html_attr_path = fixture_path("html/attribute_injections.html");
    let html_attr_source = read_file(&html_attr_path);
    let html_attr_rendered =
        render_with_theme(Some(html_attr_path.as_path()), &html_attr_source, &theme)
            .unwrap_or_else(|error| {
                panic!("failed to render {}: {error}", html_attr_path.display())
            });
    assert!(
        html_attr_rendered.contains("\x1b[38;2;80;250;123mvar"),
        "expected injected CSS styling in HTML attribute fixture"
    );
    assert!(
        html_attr_rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected injected JavaScript styling in HTML attribute fixture"
    );

    let just_path = fixture_path("just/nested_recipes.just");
    let just_source = read_file(&just_path);
    let just_rendered = render_with_theme(Some(just_path.as_path()), &just_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", just_path.display()));
    assert!(
        just_rendered.contains("\x1b[38;2;139;233;253mPreview"),
        "expected injected Python class styling in nested Just fixture"
    );
    assert!(
        just_rendered.contains("\x1b[38;2;80;250;123mrender"),
        "expected injected Python method styling in nested Just fixture"
    );
    assert!(
        just_rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected injected Bash builtin styling in nested Just fixture"
    );

    let just_heredoc_path = fixture_path("just/heredoc_recipes.just");
    let just_heredoc_source = read_file(&just_heredoc_path);
    let just_heredoc_rendered = render_with_theme(
        Some(just_heredoc_path.as_path()),
        &just_heredoc_source,
        &theme,
    )
    .unwrap_or_else(|error| panic!("failed to render {}: {error}", just_heredoc_path.display()));
    assert!(
        just_heredoc_rendered.contains("\x1b[38;2;139;233;253mPreview"),
        "expected bash heredoc injections inside Justfile to reuse python class styling"
    );
    assert!(
        just_heredoc_rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected bash heredoc injections inside Justfile to reuse javascript builtin styling"
    );
    assert!(
        just_heredoc_rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected bash heredoc injections inside Justfile to reuse bash builtin styling"
    );

    let js_path = fixture_path("javascript/tagged_templates.js");
    let js_source = read_file(&js_path);
    let js_rendered = render_with_theme(Some(js_path.as_path()), &js_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", js_path.display()));
    assert!(
        js_rendered.contains("--accent"),
        "expected injected CSS styling in JavaScript tagged template fixture"
    );
    assert!(
        js_rendered.contains("\x1b[38;2;255;184;108mtrue"),
        "expected injected JSON boolean styling in JavaScript tagged template fixture"
    );
    assert!(
        js_rendered.contains("\x1b[38;2;255;121;198msection"),
        "expected injected HTML tag styling in JavaScript tagged template fixture"
    );
}

#[test]
fn markdown_injections_highlight_inline_and_fenced_content() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("markdown/sample.md");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));
    let stripped = strip_ansi(&rendered);

    assert!(
        rendered.contains("layout"),
        "expected markdown YAML frontmatter property styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"solarized\""),
        "expected markdown YAML frontmatter string styling"
    );
    assert!(
        rendered.contains("\x1b[1m\x1b[38;2;189;147;249mPreview"),
        "expected markdown heading styling"
    );
    assert!(
        rendered.contains("\x1b[1m\x1b[38;2;189;147;249mSecondary Heading"),
        "expected markdown setext heading styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;241;250;140memphasis"),
        "expected markdown inline emphasis styling"
    );
    assert!(
        rendered.contains("\x1b[1m\x1b[38;2;255;184;108mstrong"),
        "expected markdown strong-emphasis styling"
    );
    assert!(
        rendered.contains("\x1b[9m\x1b[38;2;98;114;164mstrikethrough"),
        "expected markdown strikethrough styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m- [x]"),
        "expected markdown task-list marker styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m1."),
        "expected markdown ordered-list marker styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mhttps://example.com"),
        "expected markdown link uri styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m<https://example.com/autolink>"),
        "expected markdown autolink styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m<hello@example.com>"),
        "expected markdown email autolink styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m[docs-ref]"),
        "expected markdown reference link label styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"Docs Title\""),
        "expected markdown link title styling"
    );
    assert!(
        stripped.contains("| Column | Value |") && stripped.contains("| Code | `let value = 42` |"),
        "expected markdown pipe table delimiter styling"
    );
    assert!(
        stripped.contains("```rust"),
        "expected markdown fenced code delimiter line styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mlet value = 42"),
        "expected markdown inline code-span styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108mplain fence without syntax"),
        "expected markdown plain fenced code block styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;241;250;140m>"),
        "expected markdown blockquote marker styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198mkbd"),
        "expected markdown inline HTML to route through HTML highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198mreturn"),
        "expected fenced rust block injection styling"
    );
}

#[test]
fn markdown_frontmatter_supports_yaml_and_toml_metadata() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let yaml_path = fixture_path("markdown/sample.md");
    let yaml_source = read_file(&yaml_path);
    let yaml_rendered = render_with_theme(Some(yaml_path.as_path()), &yaml_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", yaml_path.display()));

    assert!(
        yaml_rendered.contains("layout"),
        "expected YAML frontmatter property styling"
    );
    assert!(
        yaml_rendered.contains("\x1b[38;2;255;184;108mtrue"),
        "expected YAML frontmatter boolean styling"
    );

    let toml_path = Path::new("testdata/showcase/markdown/mixed-code-blocks.md").to_path_buf();
    let toml_source = read_file(&toml_path);
    let toml_rendered = render_with_theme(Some(toml_path.as_path()), &toml_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", toml_path.display()));

    assert!(
        toml_rendered.contains("title"),
        "expected TOML frontmatter property styling"
    );
    assert!(
        toml_rendered.contains("\x1b[38;2;255;184;108mtrue"),
        "expected TOML frontmatter boolean styling"
    );
}

#[test]
fn cargo_lock_is_detected_as_toml() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("toml/Cargo.lock");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b["),
        "expected ANSI highlighting for {}",
        path.display()
    );
    assert!(
        rendered.contains("\"kat\""),
        "expected TOML string content to remain visible for {}",
        path.display()
    );
}

#[test]
fn uv_lock_is_detected_as_toml() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("toml/uv.lock");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b["),
        "expected ANSI highlighting for {}",
        path.display()
    );
    assert!(
        rendered.contains("\"kat\""),
        "expected TOML string content to remain visible for {}",
        path.display()
    );
}

#[test]
fn ignore_files_are_detected_and_highlight_ignore_syntax() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("ignore/.gitignore");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;98;114;164m# ignore build output"),
        "expected ignore-file comments to reuse comment styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m!"),
        "expected ignore-file negation to use operator styling"
    );
}

#[test]
fn git_config_files_are_detected_and_highlight_core_syntax() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("git_config/.gitconfig");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253minclude"),
        "expected include section names to reuse builtin styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m~/.gitconfig.local"),
        "expected include paths to receive path-oriented string styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m!.git/hooks/post-checkout"),
        "expected shell-command style values to remain string-highlighted"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mtrue"),
        "expected boolean values to receive builtin constant styling"
    );
}

#[test]
fn git_ignore_extra_paths_reuse_ignore_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("ignore/git-info-exclude");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(Path::new(".git/info/exclude")), &source, &theme)
        .unwrap_or_else(|error| {
            panic!(
                "failed to render {} as .git/info/exclude: {error}",
                path.display()
            )
        });

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;98;114;164m# local-only ignore rules"),
        "expected exclude comments to reuse ignore comment styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m.direnv"),
        "expected local exclude patterns to reuse ignore string styling"
    );
}

#[test]
fn armor_files_detect_and_highlight_textual_boundaries() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let cert_path = fixture_path("armor/server.pem");
    let cert_source = read_file(&cert_path);
    let cert_rendered = render_with_theme(Some(cert_path.as_path()), &cert_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", cert_path.display()));
    assert!(
        cert_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m-----BEGIN "),
        "expected PEM boundaries to receive directive styling"
    );
    assert!(
        cert_rendered.contains("\x1b[38;2;139;233;253mCERTIFICATE"),
        "expected PEM labels to receive type styling"
    );

    let private_source = concat!(
        "-----BEGIN ENCRYPTED PRIVATE ",
        "KEY-----\n",
        "Proc-Type: 4,ENCRYPTED\n",
        "DEK-Info: AES-256-CBC,0123456789ABCDEF0123456789ABCDEF\n\n",
        "MIIBvTBXBgkqhkiG9w0BBQ0wSjApBgkqhkiG9w0BBQwwHAQIkatprivatefixture\n",
        "-----END ENCRYPTED PRIVATE ",
        "KEY-----\n",
    );
    let private_analysis =
        analysis_snapshot_for_path(Path::new("private.key"), private_source, &theme);
    assert_eq!(
        private_analysis
            .detected_document_kind
            .expect("kind")
            .runtime_name,
        "pem"
    );
    let private_label = private_source
        .find("ENCRYPTED PRIVATE KEY")
        .expect("private label should exist");
    let private_style = private_analysis
        .spans
        .iter()
        .find(|span| span.start <= private_label && private_label < span.end)
        .and_then(|span| span.style.as_ref())
        .expect("private label should be styled");
    assert_eq!(private_style.color_name, "orange");
    assert!(private_style.bold);

    let pgp_path = fixture_path("armor/public.asc");
    let pgp_source = read_file(&pgp_path);
    let pgp_analysis = analysis_snapshot_for_path(pgp_path.as_path(), &pgp_source, &theme);
    assert_eq!(
        pgp_analysis
            .detected_document_kind
            .expect("kind")
            .runtime_name,
        "openpgp_armor"
    );

    let version_start = pgp_source.find("Version").expect("header should exist");
    let version_style = pgp_analysis
        .spans
        .iter()
        .find(|span| span.start <= version_start && version_start < span.end)
        .and_then(|span| span.style.as_ref())
        .expect("header key should be styled");
    assert_eq!(version_style.color_name, "foreground");

    let checksum_start = pgp_source.find("=abcd").expect("checksum should exist");
    let checksum_style = pgp_analysis
        .spans
        .iter()
        .find(|span| span.start <= checksum_start && checksum_start < span.end)
        .and_then(|span| span.style.as_ref())
        .expect("checksum should be styled");
    assert_eq!(checksum_style.color_name, "yellow");
}

#[test]
fn git_link_files_are_detected_and_highlight_gitdir_paths() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("git_link/worktree.gitdir");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198mgitdir"),
        "expected gitdir keys to receive keyword styling"
    );
    assert!(
        rendered.contains(
            "\x1b[38;2;241;250;140m/Users/dcjanus/Code/kat/.git/worktrees/git-language-support"
        ),
        "expected gitdir targets to receive path-oriented string styling"
    );
}

#[test]
fn git_mailmap_files_are_detected_and_highlight_names_and_emails() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("git_mailmap/.mailmap");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;80;250;123mDCjanus"),
        "expected canonical names to receive tag-like styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m<DCjanus@dcjanus.com>"),
        "expected email addresses to receive builtin-like styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;98;114;164m# canonical identities"),
        "expected mailmap comments to reuse comment styling"
    );
}

#[test]
fn git_log_files_are_detected_and_highlight_metadata_and_diff_blocks() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("git_log/history.gitlog");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198mcommit"),
        "expected commit headers to receive keyword styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198mAuthor:"),
        "expected metadata keys to receive keyword styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m<DCjanus@dcjanus.com>"),
        "expected metadata emails to receive builtin-like styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123muse std::ffi::OsStr;"),
        "expected embedded diff hunks to reuse diff addition styling"
    );
}

#[test]
fn git_modules_profile_preserves_git_config_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("git_config/.gitmodules");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));
    let document_kind = detect_document_kind(Some(path.as_path()), &source)
        .expect("expected .gitmodules to detect as git_config");

    assert_eq!(document_kind.runtime_name(), "git_config");
    assert_eq!(document_kind.profile(), DocumentProfile::GitModules);
    assert!(
        rendered.contains("\x1b[38;2;241;250;140mvendor/kat"),
        "expected submodule paths to remain visible and highlighted"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140mhttps://github.com/happy-proto/kat.git"),
        "expected submodule URLs to be string-highlighted"
    );
}

#[test]
fn dockerfile_highlights_directives_and_injects_shell_forms() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198mFROM"),
        "expected Dockerfile instruction keyword styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mprintf"),
        "expected shell-form RUN/CMD/ENTRYPOINT content to reuse bash builtin styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249m--version"),
        "expected shell-form command options to reuse bash parameter styling"
    );
}

#[test]
fn dockerfile_healthcheck_and_heredoc_reuse_bash_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.advanced");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198mHEALTHCHECK"),
        "expected HEALTHCHECK instruction keyword styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mprintf"),
        "expected HEALTHCHECK CMD shell form to reuse bash builtin styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected Dockerfile heredoc lines to reuse bash builtin styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m$"),
        "expected Dockerfile heredoc bash expansions to be highlighted"
    );
}

#[test]
fn dockerfile_shell_instruction_routes_shell_regions_to_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.shells");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected SHELL [\"zsh\", ...] to route shell-form RUN into zsh runtime"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mset"),
        "expected SHELL [\"fish\", ...] to route shell-form RUN into fish runtime"
    );
}

#[test]
fn dockerfile_highlights_keys_ports_and_paths_with_more_specific_semantics() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.semantics");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253mAPP_HOME"),
        "expected ENV / ARG keys to use key-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253morg.opencontainers.image.source"),
        "expected LABEL keys to use key-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108m8080"),
        "expected EXPOSE ports to use number styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m/app"),
        "expected Dockerfile paths to use string styling"
    );
}

#[test]
fn dockerfile_highlights_param_names_separately_from_values() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.params");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m--from"),
        "expected Dockerfile param names to keep attribute styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140mbuilder"),
        "expected Dockerfile param values to use string-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mBUILDPLATFORM"),
        "expected generic param expansions to split out the variable name semantics"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m5s"),
        "expected HEALTHCHECK param values to stop inheriting attribute styling"
    );
    assert!(
        (rendered.contains("\x1b[38;2;189;147;249mCACHE_DIR")
            || rendered.contains("\x1b[38;2;189;147;249mCACHE_STAGE"))
            && rendered.contains("\x1b[38;2;255;184;108m1000"),
        "expected mount param payload to keep expansion and numeric semantics"
    );
}

#[test]
fn dockerfile_highlights_mount_param_pairs_with_key_value_semantics() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.params");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253mtype"),
        "expected mount param keys to use key-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mtarget"),
        "expected mount param keys to use key-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mcache"),
        "expected mount param enum-like values to use type styling"
    );
    assert!(
        rendered.contains("CACHE_DIR") && rendered.contains("CACHE_STAGE"),
        "expected mount param expansion-based values to render"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mreadonly"),
        "expected bare mount flags to use constant styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108m1000"),
        "expected mount param numeric values to use number styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mlocked")
            && rendered.contains("\x1b[38;2;139;233;253mcache"),
        "expected mount param enum-like values to use type-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140mcache-") && rendered.contains("TARGETARCH"),
        "expected mount id values to keep string fragments while splitting embedded expansions"
    );
    assert!(
        rendered.contains("APP_ENV"),
        "expected env mount values to render as an independent identifier-like token"
    );
}

#[test]
fn dockerfile_highlights_json_form_commands_and_paths_with_host_semantics() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.exec");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253m\"python\""),
        "expected JSON-form CMD executable to use command-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m\"sh\""),
        "expected JSON-form ENTRYPOINT executable to use command-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m\"zsh\""),
        "expected SHELL executable to use command-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"app\"")
            && rendered.contains("\x1b[38;2;241;250;140m\"echo hi\""),
        "expected non-option JSON-form argv entries to remain string styled"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"/data\""),
        "expected JSON-form VOLUME paths to keep string styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m\"-m\"")
            && rendered.contains("\x1b[3m\x1b[38;2;80;250;123m\"-lc\"")
            && rendered.contains("\x1b[3m\x1b[38;2;80;250;123m\"--config\""),
        "expected JSON-form option argv entries to use attribute styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"./bin/server\""),
        "expected path-like JSON-form executables to keep path/string styling"
    );
}

#[test]
fn dockerfile_highlights_mount_type_families_with_key_aware_semantics() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.mount-kinds");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253msecret")
            && rendered.contains("\x1b[38;2;139;233;253mssh")
            && rendered.contains("\x1b[38;2;139;233;253mbind"),
        "expected mount type values to keep family/type styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140mapi-key")
            && rendered.contains("\x1b[38;2;241;250;140mdefault"),
        "expected secret/ssh ids to use string-like styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mrequired")
            && rendered.contains("\x1b[38;2;189;147;249mreadonly"),
        "expected bare flags in mount families to keep constant styling"
    );
    assert!(
        rendered.contains("API_KEY"),
        "expected env mount values to remain an independently rendered identifier-like token"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m./hack")
            && rendered.contains("\x1b[38;2;241;250;140m/work"),
        "expected bind mount source and target paths to keep string/path styling"
    );
}

#[test]
fn dockerfile_highlights_json_form_with_unified_command_option_path_env_and_expansion_semantics() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.exec-advanced");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"./bin/server\""),
        "expected path-like JSON-form executable to stay string styled"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m\"--config\"")
            && rendered.contains("\x1b[3m\x1b[38;2;80;250;123m\"--root\""),
        "expected JSON-form options to keep attribute styling across commands"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"/etc/app.toml\"")
            && rendered.contains("\x1b[38;2;241;250;140m\"/workspace\""),
        "expected path-like JSON-form argv values to use string/path styling"
    );
    assert!(
        rendered.contains("APP_ENV=prod"),
        "expected env-style JSON-form argv to render as its own structured token"
    );
    assert!(
        rendered.contains("PORT"),
        "expected JSON-form expansion argv to keep expansion semantics visible"
    );
}

#[test]
fn rustdoc_comments_highlight_markdown_content() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("rust/doc_comments.rs");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[1m\x1b[38;2;189;147;249mPreview"),
        "expected rustdoc heading styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;241;250;140mmarkdown"),
        "expected rustdoc emphasis styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123minline code"),
        "expected rustdoc code-span styling"
    );
}

#[test]
fn rustdoc_markdown_fenced_code_reuses_nested_language_runtimes() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("rust/doc_comments_nested.rs");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let markdown_region =
        find_nested_region(&analysis, "markdown", "Rustdoc should support", &source);

    assert!(
        markdown_region
            .child_nested_regions
            .iter()
            .any(|region| region.resolved_document_kind.runtime_name == "rust"),
        "expected rustdoc markdown region to preserve fenced rust child runtime"
    );
    assert!(
        markdown_region
            .child_nested_regions
            .iter()
            .any(|region| region.resolved_document_kind.runtime_name == "python"),
        "expected rustdoc markdown region to preserve fenced python child runtime"
    );
}

#[test]
fn rustdoc_nested_regions_use_different_tint_levels() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("rust/doc_comments_nested.rs");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let markdown_region =
        find_nested_region(&analysis, "markdown", "Rustdoc should support", &source);
    let rust_region = find_nested_region(&analysis, "rust", "fn nested() -> i32 {", &source);

    assert!(
        markdown_region.visual_level == 1,
        "outer rustdoc markdown prose should stay at the first nested visual level"
    );
    assert!(
        rust_region.visual_level == 2,
        "expected fenced code inside rustdoc to use a stronger nested visual level"
    );
}

#[test]
fn rustdoc_outer_markdown_prose_uses_tint_without_rectangular_padding() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("rust/doc_comments_nested.rs");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let markdown_region = find_nested_region(
        &analysis,
        "markdown",
        "same nested runtimes as top-level Markdown.",
        &source,
    );
    assert!(
        markdown_region.visual_level == 1,
        "outer rustdoc markdown prose should still receive the outer nested level"
    );
    assert_ne!(
        markdown_region.visual_kind, "rect_block",
        "outer rustdoc prose should not become a rectangular block"
    );
}

#[test]
fn rustdoc_inner_fenced_block_keeps_its_own_block_width() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("rust/doc_comments_nested.rs");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let markdown_region = find_nested_region(&analysis, "markdown", "class Nested:", &source);
    let python_region = find_nested_region(&analysis, "python", "class Nested:", &source);
    assert!(
        python_region.visual_level == 2,
        "inner fenced block line should use its own stronger block level"
    );
    assert!(
        region_covers_line(markdown_region, &source, "class Nested:"),
        "the same line should still preserve the weaker outer rustdoc container coverage"
    );
    assert_eq!(python_region.visual_kind, "rect_block");
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 120);
    let rows = [
        layout_row_containing(&layout.rows, "class Nested:"),
        layout_row_containing(&layout.rows, "def render(self)"),
        layout_row_containing(&layout.rows, "return 42"),
    ];
    let expected_bounds = background_bounds_at_level(rows[0], 2);
    assert!(
        expected_bounds.is_some()
            && rows
                .iter()
                .all(|row| background_bounds_at_level(row, 2) == expected_bounds),
        "inner fenced rows should share their own level-2 layout-time block bbox"
    );
}

#[test]
fn rustdoc_fenced_rust_does_not_inherit_markdown_literal_color_for_plain_identifiers() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = Path::new("testdata/showcase/rust/macros.rs").to_path_buf();
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        strip_ansi(&rendered).contains(" rendered "),
        "expected rustdoc fenced rust local binding to keep foreground styling"
    );
    assert!(
        !rendered.contains("\x1b[38;2;241;250;140m rendered = "),
        "plain rust identifier in rustdoc fence should not inherit markdown literal yellow"
    );
}

#[test]
fn python_docstrings_use_dedicated_runtime_and_detect_styles() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let module_docstring = find_nested_region(
        &analysis,
        "python_docstring",
        "Theme preview helpers.",
        &source,
    );
    let init_docstring = find_nested_region(
        &analysis,
        "python_docstring",
        "Initialize the preview instance.",
        &source,
    );
    let classmethod_docstring = find_nested_region(
        &analysis,
        "python_docstring",
        "Create a preview from a plain theme name.",
        &source,
    );
    let render_docstring = find_nested_region(
        &analysis,
        "python_docstring",
        "Render the preview value.",
        &source,
    );

    assert!(
        !nested_runtime_names(&analysis).contains(&"markdown"),
        "expected python docstrings to stop defaulting to markdown runtime"
    );
    assert!(
        module_docstring.resolved_document_kind.profile == "python_docstring_rest",
        "expected module docstring to resolve to reST-style profile"
    );
    assert!(
        init_docstring.resolved_document_kind.profile == "python_docstring_google",
        "expected __init__ docstring to resolve to Google-style profile"
    );
    assert!(
        classmethod_docstring.resolved_document_kind.profile == "python_docstring_rest",
        "expected classmethod docstring to resolve to reST-style profile"
    );
    assert!(
        render_docstring.resolved_document_kind.profile == "python_docstring_numpy",
        "expected render docstring to resolve to NumPy-style profile"
    );
}

#[test]
fn python_docstrings_highlight_rest_google_and_numpy_structures() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let keyword_style = theme
        .token_style_for("keyword.directive", "Args")
        .expect("expected keyword.directive style")
        .snapshot(ColorMode::TrueColor);
    let parameter_style = theme
        .token_style_for("variable.parameter", "name")
        .expect("expected variable.parameter style")
        .snapshot(ColorMode::TrueColor);
    let type_style = theme
        .token_style_for("type", "ThemePreview")
        .expect("expected type style")
        .snapshot(ColorMode::TrueColor);
    let punctuation_style = theme
        .token_style_for("punctuation.special", ":")
        .expect("expected punctuation.special style")
        .snapshot(ColorMode::TrueColor);

    let args_offset = source.find("Args:").expect("expected Args section");
    let param_field_offset = source
        .find(":param name:")
        .expect("expected reST param field");
    let role_offset = source
        .find(":class:`ThemePreview`")
        .expect("expected reST class role");
    let numpy_section_offset = source
        .find("Parameters\n        ----------")
        .expect("expected NumPy section header");
    let numpy_param_offset = source
        .find("uppercase : bool")
        .expect("expected NumPy parameter entry");

    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= args_offset
                && args_offset < span.end
                && span.style == Some(keyword_style.clone())
        }),
        "expected Google section header to receive directive styling"
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= param_field_offset + 1
                && param_field_offset + 1 < span.end
                && span.style == Some(keyword_style.clone())
        }),
        "expected reST field name to receive directive styling"
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= param_field_offset + 7
                && param_field_offset + 7 < span.end
                && span.style == Some(parameter_style.clone())
        }),
        "expected reST parameter name to receive parameter styling"
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= role_offset + 8
                && role_offset + 8 < span.end
                && span.style == Some(type_style.clone())
        }),
        "expected reST role target to receive type styling"
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= numpy_section_offset
                && numpy_section_offset < span.end
                && span.style == Some(keyword_style.clone())
        }),
        "expected NumPy section title to receive directive styling"
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= numpy_param_offset
                && numpy_param_offset < span.end
                && span.style == Some(parameter_style.clone())
        }),
        "expected NumPy parameter name to receive parameter styling"
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= numpy_param_offset + "uppercase ".len()
                && numpy_param_offset + "uppercase ".len() < span.end
                && span.style == Some(punctuation_style.clone())
        }),
        "expected NumPy parameter separator to receive punctuation styling"
    );
}

#[test]
fn python_docstring_doctest_reuses_python_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let doctest_region = find_nested_region(
        &analysis,
        "python",
        ">>> preview = ThemePreview(\"Dracula\")",
        &source,
    );

    assert!(
        doctest_region.visual_level >= 2,
        "expected doctest code to render as a deeper nested python runtime"
    );
    assert!(
        doctest_region
            .layout_segments
            .iter()
            .all(|segment| segment.left > segment.line_start),
        "expected doctest runtime to begin after the doctest prompt"
    );
}

#[test]
fn python_docstring_doctest_highlights_inner_python_literals() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let string_style = theme
        .token_style_for("string", "\"Dracula\"")
        .expect("expected string style")
        .snapshot(ColorMode::TrueColor);
    let string_offset = source
        .find("\"Dracula\"")
        .expect("expected doctest string literal");

    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= string_offset
                && string_offset < span.end
                && span.style == Some(string_style.clone())
        }),
        "expected doctest string literal to reuse Python string styling"
    );
}

#[test]
fn python_docstring_runtime_uses_rectangular_block_visuals() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let docstring_region = find_nested_region(
        &analysis,
        "python_docstring",
        "Theme preview helpers.",
        &source,
    );
    assert_eq!(
        docstring_region.visual_kind, "rect_block",
        "expected python docstring runtime to render as a rectangular block"
    );
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 120);
    let rows = [
        layout_row_containing(&layout.rows, "Theme preview helpers."),
        layout_row_containing(
            &layout.rows,
            "This module exercises several common docstring styles.",
        ),
        layout_row_containing(
            &layout.rows,
            "The public entrypoint is :class:`ThemePreview`.",
        ),
    ];
    let expected_bounds = first_background_bounds(rows[0]);
    assert!(expected_bounds.is_some());
    assert!(
        rows.iter()
            .all(|row| first_background_bounds(row) == expected_bounds),
        "expected python docstring rows to share one layout-time block bbox"
    );
}

#[test]
fn python_docstring_wraps_blank_and_content_rows_into_one_exact_bbox() {
    let source = "def demo():\n    \"\"\"短说明。\n\n    这里是一段足够长的中文内容用于验证文档字符串换行后的统一背景边界\n    \"\"\"\n";
    let theme =
        Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(RgbColor(1, 2, 3)));
    let layout =
        crate::api::layout_with_theme(Some(Path::new("demo.py")), source, &theme, Some(49))
            .expect("expected Python docstring layout")
            .snapshot();
    let block_start = layout
        .rows
        .iter()
        .position(|row| row.text.contains("短说明"))
        .expect("expected opening docstring row");
    let block_end = layout
        .rows
        .iter()
        .rposition(|row| row.text.trim() == "\"\"\"")
        .expect("expected closing docstring row");
    let rows = &layout.rows[block_start..=block_end];

    assert!(rows.len() > 4, "expected the long docstring row to wrap");
    assert!(
        rows.iter()
            .all(|row| background_bounds_at_level(row, 1) == Some((0, 48))),
        "expected docstring content, blank, and continuation rows to share the exact non-viewport-wide bbox: {rows:?}"
    );
}

#[test]
fn python_docstring_block_left_edge_starts_at_triple_quotes() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let docstring_region = find_nested_region(
        &analysis,
        "python_docstring",
        "Initialize the preview instance.",
        &source,
    );
    let first_segment = segment_for_line(
        docstring_region,
        &source,
        "\"\"\"Initialize the preview instance.",
    );
    let line_start = line_start_containing(&source, "\"\"\"Initialize the preview instance.");
    let quote_column = display_width(
        &source[line_start
            ..source[line_start..]
                .find("\"\"\"Initialize the preview instance.")
                .map(|offset| line_start + offset)
                .expect("expected init docstring line")],
    )
    .as_usize();

    assert_eq!(
        segment_left_column(&source, first_segment),
        quote_column,
        "expected python docstring block left edge to align with opening triple quotes"
    );
}

#[test]
fn python_docstring_blank_lines_keep_block_tint_geometry() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 120);
    let args_row = layout_row_containing(&layout.rows, "        Args:");
    let raises_row = layout_row_containing(&layout.rows, "        Raises:");
    let blank_after_intro = layout
        .rows
        .iter()
        .find(|row| row.source_line_start == line_start_for_line_number(&source, 24))
        .expect("expected blank line between init docstring intro and Args section");
    let blank_before_raises = layout
        .rows
        .iter()
        .find(|row| row.source_line_start == line_start_for_line_number(&source, 28))
        .expect("expected blank line between init docstring Args and Raises sections");

    assert_eq!(
        blank_after_intro
            .background_runs
            .first()
            .map(|run| (run.start_column, run.end_column)),
        args_row
            .background_runs
            .first()
            .map(|run| (run.start_column, run.end_column)),
        "expected blank line after the init summary to keep the same docstring block tint as Args"
    );
    assert_eq!(
        blank_before_raises
            .background_runs
            .first()
            .map(|run| (run.start_column, run.end_column)),
        raises_row
            .background_runs
            .first()
            .map(|run| (run.start_column, run.end_column)),
        "expected blank line before Raises to keep the same docstring block tint as Raises"
    );
}

#[test]
fn python_docstring_blank_lines_render_background_width_as_spaces() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let render_plan = render_plan_snapshot_for_path(path.as_path(), &source, &theme);
    let rendered_text = render_plan_text(&render_plan);
    let rendered_lines: Vec<_> = rendered_text.lines().collect();
    let blank_after_intro = rendered_lines
        .get(23)
        .copied()
        .expect("expected render plan to retain blank init docstring line");
    let blank_before_raises = rendered_lines
        .get(27)
        .copied()
        .expect("expected render plan to retain blank Raises separator line");

    assert!(
        !blank_after_intro.is_empty()
            && blank_after_intro.chars().all(|ch| ch == ' ')
            && blank_after_intro.starts_with("        "),
        "expected blank init docstring line to render as background-bearing spaces, got {blank_after_intro:?}"
    );
    assert!(
        !blank_before_raises.is_empty()
            && blank_before_raises.chars().all(|ch| ch == ' ')
            && blank_before_raises.starts_with("        "),
        "expected blank Raises separator line to render as background-bearing spaces, got {blank_before_raises:?}"
    );
}

#[test]
fn python_docstring_blank_lines_switch_into_background_after_indent() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let render_plan = render_plan_snapshot_for_path(path.as_path(), &source, &theme);
    let blank_after_intro = render_plan_lines(&render_plan)
        .get(23)
        .cloned()
        .expect("expected init docstring blank line in render plan");

    assert!(
        matches!(
            blank_after_intro.as_slice(),
            [
                RenderOpSnapshot::Text { text: indent },
                RenderOpSnapshot::SetStyle { .. },
                RenderOpSnapshot::Text { text: tinted },
                RenderOpSnapshot::ResetStyle,
            ] if indent == "        " && !tinted.is_empty() && tinted.chars().all(|ch| ch == ' ')
        ),
        "expected blank init docstring line to switch into a tinted space run after the shared indent, got {blank_after_intro:?}"
    );
}

#[test]
fn python_highlights_class_and_method_names() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/main.py");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253mGreeter"),
        "expected python class name styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mhello"),
        "expected python method name styling"
    );
}

#[test]
fn python_highlights_decorators_parameters_and_calls() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/rich_features.py");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198m@"),
        "expected decorator marker styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mdataclass"),
        "expected decorator name styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249mself"),
        "expected self parameter styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mprint"),
        "expected builtin call styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mThemePreview"),
        "expected class call styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mrender"),
        "expected method call styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mpreview"),
        "expected local binding styling for preview"
    );
}

#[test]
fn python_docstring_doctest_highlights_local_bindings_and_receivers() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let doctest_region = find_nested_region(
        &analysis,
        "python",
        ">>> preview = ThemePreview(\"Dracula\")",
        &source,
    );
    let local_style = theme
        .token_style_for("variable.local", "preview")
        .expect("expected local variable style")
        .snapshot(ColorMode::TrueColor);
    let assignment_preview = source
        .find("preview = ThemePreview(\"Dracula\")")
        .expect("expected doctest assignment preview");
    let receiver_preview = source
        .find("preview.render()")
        .expect("expected doctest receiver preview");

    for offset in [assignment_preview, receiver_preview] {
        assert!(
            doctest_region.overlays.iter().any(|span| {
                span.start <= offset && offset < span.end && span.style == Some(local_style.clone())
            }),
            "expected doctest preview at byte offset {offset} to reuse local variable styling"
        );
    }
}

#[test]
fn python_docstring_doctest_child_python_overlays_reach_final_analysis_spans() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let local_style = theme
        .token_style_for("variable.local", "preview")
        .expect("expected local variable style")
        .snapshot(ColorMode::TrueColor);
    let method_style = theme
        .token_style_for("function.method.call", "render")
        .expect("expected method call style")
        .snapshot(ColorMode::TrueColor);

    for (needle, style) in [
        ("preview = ThemePreview(\"Dracula\")", local_style.clone()),
        ("preview.render()", local_style.clone()),
        ("render()", method_style),
    ] {
        let offset = source.find(needle).expect("expected doctest fragment");
        assert!(
            analysis.spans.iter().any(|span| {
                span.start <= offset && offset < span.end && span.style == Some(style.clone())
            }),
            "expected final analysis spans to retain doctest child overlay for {needle}"
        );
    }
}

#[test]
fn python_highlights_advanced_semantics() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/advanced.py");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;80;250;123mclassmethod"),
        "expected decorator builtin styling for classmethod"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mbuild_default"),
        "expected function definition styling for build_default"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123m__init__"),
        "expected constructor method styling for __init__"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;139;233;253mstr"),
        "expected builtin type styling for str annotations"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253misinstance"),
        "expected builtin predicate styling for isinstance"
    );
}

#[test]
fn sql_highlights_keywords_types_parameters_and_dollar_strings() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("sql/rich.sql");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;98;114;164m-- Dashboard query"),
        "expected SQL comment styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198mWITH"),
        "expected SQL keyword styling for WITH"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mCOUNT"),
        "expected SQL function-call styling for COUNT"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;255;184;108m$1"),
        "expected SQL parameter styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m$kat$dracula$kat$"),
        "expected SQL dollar-quoted string styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108m10"),
        "expected SQL numeric literal styling"
    );
}

#[test]
fn sql_dialect_detector_uses_path_and_content_signals() {
    assert_eq!(
        detect_sql_dialect(Some(Path::new("db/schema.postgres.sql")), "SELECT 1;").runtime_name(),
        "sql_postgres"
    );
    assert_eq!(
        detect_sql_dialect(Some(Path::new("db/schema.psql")), "SELECT 1;").runtime_name(),
        "sql_postgres"
    );
    assert_eq!(
        detect_sql_dialect(
            Some(Path::new("db/schema.sql")),
            "INSERT INTO themes VALUES ($1) ON CONFLICT (id) DO NOTHING RETURNING id;"
        )
        .runtime_name(),
        "sql_postgres"
    );
    assert_eq!(
        detect_sql_dialect(Some(Path::new("db/schema.mysql")), "SELECT 1;").runtime_name(),
        "sql_mysql"
    );
    assert_eq!(
        detect_sql_dialect(
            Some(Path::new("db/schema.sql")),
            "CREATE TABLE `themes` (`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT) ENGINE=InnoDB;"
        )
        .runtime_name(),
        "sql_mysql"
    );
    assert_eq!(
        detect_sql_dialect(Some(Path::new("db/schema.sqlite3")), "SELECT 1;").runtime_name(),
        "sql_sqlite"
    );
    assert_eq!(
        detect_sql_dialect(
            Some(Path::new("db/schema.sql")),
            "CREATE TABLE themes (id INTEGER PRIMARY KEY AUTOINCREMENT) STRICT, WITHOUT ROWID;"
        )
        .runtime_name(),
        "sql_sqlite"
    );
    assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "CREATE FUNCTION refresh_theme_cache() RETURNS void LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE AS $$ BEGIN RETURN; END; $$; CREATE INDEX theme_payload_gin ON theme_snapshots USING GIN (payload);"
            )
            .runtime_name(),
            "sql_postgres"
        );
    assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "CREATE UNLOGGED TABLE theme_cache (id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY); CREATE EXTENSION IF NOT EXISTS pgcrypto;"
            )
            .runtime_name(),
            "sql_postgres"
        );
    assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "CREATE TABLE theme_snapshots (id INT UNSIGNED NOT NULL AUTO_INCREMENT, name VARCHAR(255) CHARACTER SET utf8mb4, PRIMARY KEY (id)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4; INSERT INTO theme_snapshots (id, name) VALUES (1, 'Dracula') ON DUPLICATE KEY UPDATE name = VALUES(name);"
            )
            .runtime_name(),
            "sql_mysql"
        );
    assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "CREATE TABLE `theme_snapshots` (`id` BIGINT UNSIGNED ZEROFILL NOT NULL AUTO_INCREMENT) ENGINE=InnoDB; INSERT IGNORE INTO `theme_snapshots` (`id`) VALUES (1); SHOW CREATE TABLE `theme_snapshots`;"
            )
            .runtime_name(),
            "sql_mysql"
        );
    assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "PRAGMA foreign_keys = ON; VACUUM; INSERT OR IGNORE INTO theme_cache (slug) VALUES ('dracula');"
            )
            .runtime_name(),
            "sql_sqlite"
        );
    assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "BEGIN IMMEDIATE; ATTACH DATABASE 'theme-cache.db' AS cache; REINDEX; INSERT OR FAIL INTO theme_cache (slug) VALUES ('kat');"
            )
            .runtime_name(),
            "sql_sqlite"
        );
    assert_eq!(
        detect_sql_dialect(
            Some(Path::new("db/schema.mysql.sql")),
            "INSERT INTO theme_snapshots (payload) VALUES ($1::jsonb) RETURNING id;"
        )
        .runtime_name(),
        "sql_postgres"
    );
    assert_eq!(
        detect_sql_dialect(
            Some(Path::new("db/schema.sql")),
            "SELECT id, name FROM themes;"
        )
        .runtime_name(),
        "sql"
    );
}

#[test]
fn sql_dialect_runtimes_highlight_postgres_mysql_and_sqlite_specific_tokens() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let postgres_path = fixture_path("sql/schema.postgres.sql");
    let postgres_source = read_file(&postgres_path);
    let postgres_rendered =
        render_with_theme(Some(postgres_path.as_path()), &postgres_source, &theme).unwrap_or_else(
            |error| panic!("failed to render {}: {error}", postgres_path.display()),
        );
    assert!(
        postgres_rendered.contains("\x1b[38;2;139;233;253mjsonb"),
        "expected postgres runtime to highlight jsonb as builtin type"
    );
    assert!(
        postgres_rendered.contains("\x1b[38;2;255;121;198mUNLOGGED"),
        "expected postgres runtime to highlight UNLOGGED"
    );
    assert!(
        postgres_rendered.contains("\x1b[38;2;255;121;198mALWAYS"),
        "expected postgres runtime to highlight ALWAYS"
    );
    assert!(
        postgres_rendered.contains("\x1b[38;2;255;121;198mRETURNING"),
        "expected postgres runtime to highlight RETURNING"
    );
    assert!(
        postgres_rendered.contains("\x1b[38;2;241;250;140m$tag$dracula$tag$"),
        "expected postgres runtime to highlight dollar-quoted strings"
    );

    let mysql_path = fixture_path("sql/schema.mysql.sql");
    let mysql_source = read_file(&mysql_path);
    let mysql_rendered = render_with_theme(Some(mysql_path.as_path()), &mysql_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", mysql_path.display()));
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mAUTO_INCREMENT"),
        "expected mysql runtime to highlight AUTO_INCREMENT"
    );
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mENGINE"),
        "expected mysql runtime to highlight ENGINE"
    );
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mZEROFILL"),
        "expected mysql runtime to highlight ZEROFILL"
    );
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mREPLACE"),
        "expected mysql runtime to highlight REPLACE"
    );

    let sqlite_path = fixture_path("sql/schema.sqlite.sql");
    let sqlite_source = read_file(&sqlite_path);
    let sqlite_rendered = render_with_theme(Some(sqlite_path.as_path()), &sqlite_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", sqlite_path.display()));
    assert!(
        sqlite_rendered.contains("\x1b[38;2;255;121;198mWITHOUT"),
        "expected sqlite runtime to highlight WITHOUT"
    );
}

#[test]
fn sql_dialect_runtimes_highlight_deeper_dialect_specific_constructs() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let postgres_rendered = highlight_named_language(
            "sql_postgres",
            "CREATE FUNCTION refresh_theme_cache() RETURNS void LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE AS $$ BEGIN RETURN; END; $$;\nCREATE INDEX theme_payload_gin ON theme_snapshots USING GIN (payload jsonb_path_ops);",
            &theme,
        )
        .expect("failed to highlight postgres SQL source");
    assert!(
        postgres_rendered.contains("\x1b[38;2;255;121;198mLANGUAGE"),
        "expected postgres runtime to highlight LANGUAGE"
    );
    assert!(
        postgres_rendered.contains("\x1b[38;2;255;121;198mIMMUTABLE"),
        "expected postgres runtime to highlight IMMUTABLE"
    );
    assert!(
        postgres_rendered.contains("\x1b[38;2;255;121;198mPARALLEL"),
        "expected postgres runtime to highlight PARALLEL"
    );
    assert!(
        postgres_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mGIN"),
        "expected postgres runtime to highlight GIN as a builtin index method"
    );

    let mysql_rendered = highlight_named_language(
            "sql_mysql",
            "CREATE TABLE theme_snapshots (id INT UNSIGNED NOT NULL AUTO_INCREMENT, name VARCHAR(255) CHARACTER SET utf8mb4, PRIMARY KEY (id)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;\nINSERT HIGH_PRIORITY IGNORE INTO theme_snapshots (id, name) VALUES (1, 'Dracula');\nREPLACE DELAYED LOW_PRIORITY INTO theme_snapshots (id, name) VALUES (2, 'Kat');",
            &theme,
        )
        .expect("failed to highlight mysql SQL source");
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mSET"),
        "expected mysql runtime to highlight SET inside CHARACTER SET"
    );
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mHIGH_PRIORITY"),
        "expected mysql runtime to highlight HIGH_PRIORITY"
    );
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mIGNORE"),
        "expected mysql runtime to highlight IGNORE"
    );
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mDELAYED"),
        "expected mysql runtime to highlight DELAYED"
    );
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mUNSIGNED"),
        "expected mysql runtime to highlight UNSIGNED as a type qualifier"
    );

    let sqlite_rendered = highlight_named_language(
            "sql_sqlite",
            "PRAGMA foreign_keys = ON;\nVACUUM;\nINSERT OR IGNORE INTO theme_cache (slug) VALUES ('dracula');\nSELECT slug FROM theme_cache WHERE slug GLOB 'dr*';",
            &theme,
        )
        .expect("failed to highlight sqlite SQL source");
    assert!(
        sqlite_rendered.contains("\x1b[38;2;255;121;198mVACUUM"),
        "expected sqlite runtime to highlight VACUUM"
    );
    assert!(
        sqlite_rendered.contains("\x1b[38;2;255;121;198mIGNORE"),
        "expected sqlite runtime to highlight IGNORE"
    );
}

#[test]
fn javascript_python_rust_and_regex_hosts_reuse_nested_sql_and_regex_runtimes() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let js_path = fixture_path("javascript/injections.js");
    let js_source = read_file(&js_path);
    let js_analysis = analysis_snapshot_for_path(js_path.as_path(), &js_source, &theme);
    let js_runtimes = nested_runtime_names(&js_analysis);
    assert!(js_runtimes.contains(&"sql_postgres"));
    assert!(js_runtimes.contains(&"sql_mysql"));
    assert!(js_runtimes.contains(&"regex_javascript"));

    let python_path = fixture_path("python/injections.py");
    let python_source = read_file(&python_path);
    let python_analysis = analysis_snapshot_for_path(python_path.as_path(), &python_source, &theme);
    let python_runtimes = nested_runtime_names(&python_analysis);
    assert!(python_runtimes.contains(&"sql_sqlite"));
    assert!(python_runtimes.contains(&"regex_python"));

    let rust_path = fixture_path("rust/injections.rs");
    let rust_source = read_file(&rust_path);
    let rust_analysis = analysis_snapshot_for_path(rust_path.as_path(), &rust_source, &theme);
    let rust_runtimes = nested_runtime_names(&rust_analysis);
    assert!(rust_runtimes.contains(&"sql"));
    assert!(rust_runtimes.contains(&"regex_rust"));
}

#[test]
fn typescript_and_tsx_hosts_reuse_comment_led_template_injection_runtimes() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let ts_path = fixture_path("typescript/injections.ts");
    let ts_source = read_file(&ts_path);
    let ts_analysis = analysis_snapshot_for_path(ts_path.as_path(), &ts_source, &theme);
    let ts_runtimes = nested_runtime_names(&ts_analysis);
    assert!(ts_runtimes.contains(&"sql"));
    assert!(ts_runtimes.contains(&"graphql"));
    assert!(ts_runtimes.contains(&"html"));
    assert!(ts_runtimes.contains(&"css"));

    let tsx_path = fixture_path("tsx/injections.tsx");
    let tsx_source = read_file(&tsx_path);
    let tsx_analysis = analysis_snapshot_for_path(tsx_path.as_path(), &tsx_source, &theme);
    let tsx_runtimes = nested_runtime_names(&tsx_analysis);
    assert!(tsx_runtimes.contains(&"sql"));
    assert!(tsx_runtimes.contains(&"graphql"));
    assert!(tsx_runtimes.contains(&"html"));
    assert!(tsx_runtimes.contains(&"css"));
}

#[test]
fn regex_highlights_lookarounds_posix_unicode_and_inline_flags() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let rendered = highlight_named_language(
        "regex",
        r"(?im-s:(?<=\b)(?<word>[[:alpha:]]+)\p{Script=Latin}(?P=word)(?!\d)){2,4}?",
        &theme,
    )
    .expect("failed to highlight regex source");

    assert!(
        rendered.contains("\x1b[38;2;255;121;198mim-s"),
        "expected regex inline flags styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;139;233;253malpha"),
        "expected regex POSIX class name styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m\\p"),
        "expected regex unicode property escape styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;139;233;253mScript"),
        "expected regex unicode property name styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;139;233;253mLatin"),
        "expected regex unicode property value styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m="),
        "expected regex unicode property separator styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108m2"),
        "expected regex quantifier lower bound styling"
    );
}

#[test]
fn regex_semantic_overlay_reports_quantifiers_unicode_and_class_ranges() {
    let output = debug_semantics("regex", r"(?im-s:\p{Script=Latin}[a-z0-9-]{2,4})")
        .expect("failed to render regex semantic overlay");

    assert!(
        output.contains("keyword.operator.regex") && output.contains("im"),
        "expected regex semantic overlay to include inline flag spans: {output}"
    );
    assert!(
        output.contains("operator.regex") && output.contains("\\p"),
        "expected regex semantic overlay to include unicode property escape spans: {output}"
    );
    assert!(
        output.contains("type.builtin") && output.contains("Script"),
        "expected regex semantic overlay to include unicode property name spans: {output}"
    );
    assert!(
        output.contains("type.builtin") && output.contains("Latin"),
        "expected regex semantic overlay to include unicode property value spans: {output}"
    );
    assert!(
        output.contains("number.quantifier.regex") && output.contains("2"),
        "expected regex semantic overlay to include quantifier number spans: {output}"
    );
    assert!(
        output.contains("punctuation.delimiter") && output.contains(","),
        "expected regex semantic overlay to include quantifier delimiter spans: {output}"
    );
    assert!(
        output.contains("operator.regex") && output.contains("-"),
        "expected regex semantic overlay to include class range operator spans: {output}"
    );
}

#[test]
fn regex_host_runtimes_flag_unsupported_constructs() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let python_rendered =
        highlight_named_language("regex_python", r"(?<name>\w+)\k<name>\p{L}", &theme)
            .expect("failed to highlight python regex source");
    assert!(
        python_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?<name>"),
        "expected python regex runtime to flag JS-style named groups as invalid"
    );
    let rust_rendered =
        highlight_named_language("regex_rust", r"(?=foo)(?P<name>\w+)(?P=name)", &theme)
            .expect("failed to highlight rust regex source");
    assert!(
        rust_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?=foo)"),
        "expected rust regex runtime to flag lookahead as invalid"
    );

    let go_rendered = highlight_named_language("regex_go", r"(?=foo)\1", &theme)
        .expect("failed to highlight go regex source");
    assert!(
        go_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?=foo)"),
        "expected go regex runtime to flag lookahead as invalid"
    );

    let javascript_rendered = highlight_named_language("regex_javascript", r"(?i:theme)", &theme)
        .expect("failed to highlight javascript regex source");
    assert!(
        javascript_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?i:theme)"),
        "expected javascript regex runtime to flag inline flag groups as invalid"
    );

    let posix_rendered = highlight_named_language("regex_posix", r"(?:theme)+?", &theme)
        .expect("failed to highlight posix regex source");
    assert!(
        posix_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?:theme)"),
        "expected posix regex runtime to flag non-capturing groups as invalid"
    );
}

#[test]
fn sql_like_api_calls_in_hosts_reuse_sql_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let js_rendered = render_with_theme(
            Some(Path::new("inline.js")),
            "export async function load(db) { return db.query(\"SELECT id FROM themes WHERE slug = $1 RETURNING id\"); }",
            &theme,
        )
        .expect("failed to render inline javascript SQL source");
    assert!(
        js_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected JavaScript db.query string to reuse SQL runtime"
    );
    assert!(
        js_rendered.contains("\x1b[3m\x1b[38;2;255;184;108m$1"),
        "expected JavaScript db.query string to preserve SQL parameter styling"
    );
    let js_sqlite_rendered = render_with_theme(
            Some(Path::new("inline.js")),
            "export function load(db) { db.get(\"SELECT id FROM themes WHERE slug = ?\"); db.run(\"INSERT INTO theme_cache (slug) VALUES (?)\"); }",
            &theme,
        )
        .expect("failed to render inline javascript sqlite-style SQL source");
    assert!(
        js_sqlite_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected JavaScript db.get string to reuse SQL runtime"
    );
    assert!(
        js_sqlite_rendered.contains("\x1b[38;2;255;121;198mINSERT"),
        "expected JavaScript db.run string to reuse SQL runtime"
    );

    let python_rendered = render_with_theme(
        Some(Path::new("inline.py")),
        "def load(cursor):\n    return cursor.execute(\"SELECT id FROM themes WHERE slug = ?\")\n",
        &theme,
    )
    .expect("failed to render inline python SQL source");
    assert!(
        python_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected Python cursor.execute string to reuse SQL runtime"
    );

    let go_rendered = render_with_theme(
        Some(Path::new("inline.go")),
        "package main\nfunc load(db DB) { db.Query(\"SELECT id FROM themes WHERE slug = $1\") }\n",
        &theme,
    )
    .expect("failed to render inline go SQL source");
    assert!(
        go_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected Go db.Query string to reuse SQL runtime"
    );

    let rust_rendered = render_with_theme(
            Some(Path::new("inline.rs")),
            "fn load(client: Client) { let _ = client.query(\"SELECT id FROM themes WHERE slug = $1\", &[]); }\n",
            &theme,
        )
        .expect("failed to render inline rust SQL source");
    assert!(
        rust_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected Rust client.query string to reuse SQL runtime"
    );
    let rust_prepare_rendered = render_with_theme(
            Some(Path::new("inline.rs")),
            "fn load(client: Client) { let _ = client.prepare_typed(\"SELECT id FROM themes WHERE slug = $1\", &[]); let _ = client.simple_query(\"SELECT id FROM themes\"); }\n",
            &theme,
        )
        .expect("failed to render inline rust extended SQL source");
    assert!(
        rust_prepare_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected Rust prepare_typed/simple_query strings to reuse SQL runtime"
    );
}

#[test]
fn detect_language_uses_extensionless_sql_and_graphql_heuristics() {
    assert!(matches!(
        detect_language(None, "SELECT id FROM themes WHERE enabled = true;"),
        Some(SupportedLanguage::Sql)
    ));
    assert!(matches!(
        detect_language(
            None,
            "query ThemeBySlug($slug: ID!) { theme(slug: $slug) { id } }"
        ),
        Some(SupportedLanguage::Graphql)
    ));
}

#[test]
fn bash_and_just_sql_heredocs_reuse_sql_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let bash_path = fixture_path("bash/sql_heredoc.sh");
    let bash_source = read_file(&bash_path);
    let bash_rendered = render_with_theme(Some(bash_path.as_path()), &bash_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", bash_path.display()));
    assert!(
        bash_rendered.contains("\x1b[38;2;255;121;198mRETURNING"),
        "expected psql heredoc to route through postgres SQL runtime"
    );
    assert!(
        bash_rendered.contains("\x1b[38;2;255;121;198mAUTO_INCREMENT"),
        "expected mysql heredoc to route through mysql SQL runtime"
    );
    assert!(
        bash_rendered.contains("\x1b[38;2;255;121;198mWITHOUT"),
        "expected sqlite heredoc to route through sqlite SQL runtime"
    );
    assert!(
        bash_rendered.contains("\x1b[38;2;80;250;123mpsql"),
        "expected Bash command styling to remain visible around SQL injection"
    );

    let just_path = fixture_path("just/heredoc_recipes.just");
    let just_source = read_file(&just_path);
    let just_rendered = render_with_theme(Some(just_path.as_path()), &just_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", just_path.display()));
    assert!(
        just_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected Justfile Bash heredoc to reuse SQL runtime"
    );
}

#[test]
fn javascript_highlights_classes_private_fields_regex_and_jsx() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let rich_path = fixture_path("javascript/rich.js");
    let rich_source = read_file(&rich_path);
    let rich_rendered = render_with_theme(Some(rich_path.as_path()), &rich_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", rich_path.display()));

    assert!(
        rich_rendered.contains("\x1b[38;2;139;233;253mThemePreview"),
        "expected class definition styling for ThemePreview"
    );
    assert!(
        strip_ansi(&rich_rendered).contains("#theme"),
        "expected private field property styling for #theme"
    );
    assert!(
        rich_rendered.contains("\x1b[38;2;80;250;123mrender"),
        "expected method definition styling for render"
    );
    assert!(
        rich_rendered.contains("\x1b[38;2;255;85;85m/dracula/"),
        "expected regex body styling for /dracula/"
    );
    assert!(
        rich_rendered.contains("\x1b[38;2;255;121;198mgi"),
        "expected regex flag styling for gi"
    );
    assert!(
        rich_rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected builtin console styling"
    );

    let jsx_path = fixture_path("javascript/component.jsx");
    let jsx_source = read_file(&jsx_path);
    let jsx_rendered = render_with_theme(Some(jsx_path.as_path()), &jsx_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", jsx_path.display()));

    assert!(
        jsx_rendered.contains("\x1b[38;2;255;121;198msection"),
        "expected intrinsic JSX tag styling for section"
    );
    assert!(
        jsx_rendered.contains("\x1b[38;2;139;233;253mPreviewCard"),
        "expected component JSX tag styling for PreviewCard"
    );
    assert!(
        jsx_rendered.contains("\x1b[3m\x1b[38;2;80;250;123mclassName"),
        "expected JSX attribute styling for className"
    );
}

#[test]
fn css_highlights_selectors_namespace_custom_properties_and_units() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("css/rich.css");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198m@supports"),
        "expected at-rule styling for @supports"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m#"),
        "expected id selector punctuation styling for #"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mapp"),
        "expected id selector name styling for app"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m."),
        "expected class selector punctuation styling for ."
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mpanel"),
        "expected class selector name styling for panel"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m:"),
        "expected pseudo selector punctuation styling for :"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mhover"),
        "expected pseudo selector name styling for hover"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mrem"),
        "expected unit styling for rem"
    );
}

#[test]
fn html_highlights_entities_custom_elements_and_nested_language_regions() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("html/rich.html");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198msection"),
        "expected html element styling for section"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m&amp;"),
        "expected entity styling for &amp;"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198mtheme-card"),
        "expected custom element tag styling for theme-card"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123maccent"),
        "expected html attribute styling for accent"
    );
}

#[test]
fn jsdoc_highlights_tags_types_and_identifiers() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let rendered = highlight_named_language(
        "jsdoc",
        "/**\n * @param {ThemePreview} preview Render the preview state.\n */",
        &theme,
    )
    .expect("failed to highlight jsdoc source");

    assert!(
        rendered.contains("\x1b[38;2;255;121;198m@param"),
        "expected jsdoc tag styling for @param"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mThemePreview"),
        "expected jsdoc type styling for ThemePreview"
    );
}

#[test]
fn jsdoc_highlights_optionals_defaults_inline_tags_and_code_fences() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let rendered = highlight_named_language(
            "jsdoc",
            "/**\n * Render {@link ThemePreview} for docs.\n * @param {ThemePreview} [preview=ThemeStore.default] Preview value.\n * @param {number} [retryCount=3] Retries.\n * @see theme/preview\n * ```js\n * console.log(preview)\n * ```\n */",
            &theme,
        )
        .expect("failed to highlight jsdoc source");

    assert!(
        rendered.contains("\x1b[38;2;255;121;198m@param"),
        "expected jsdoc tag styling for @param"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m="),
        "expected jsdoc default-value operator styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108m3"),
        "expected jsdoc numeric default styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m."),
        "expected jsdoc member-expression delimiter styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140mjs"),
        "expected jsdoc code-block language styling"
    );
}

#[test]
fn sql_semantic_overlay_reports_dialect_specific_identifiers() {
    let postgres_output = debug_semantics(
            "sql_postgres",
            "CREATE FUNCTION refresh_theme_cache() RETURNS void LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE AS $$ BEGIN RETURN; END; $$;\nCREATE INDEX theme_payload_gin ON theme_snapshots USING GIN (payload jsonb_path_ops);",
        )
        .expect("failed to render postgres semantic overlay");
    assert!(
        postgres_output.contains("type.builtin") && postgres_output.contains("plpgsql"),
        "expected postgres semantic overlay to include function language spans: {postgres_output}"
    );
    assert!(
        postgres_output.contains("type.builtin") && postgres_output.contains("jsonb_path_ops"),
        "expected postgres semantic overlay to include opclass spans: {postgres_output}"
    );

    let mysql_output = debug_semantics(
        "sql_mysql",
        "CREATE TABLE theme_snapshots (name VARCHAR(255)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;",
    )
    .expect("failed to render mysql semantic overlay");
    assert!(
        mysql_output.contains("type.builtin") && mysql_output.contains("InnoDB"),
        "expected mysql semantic overlay to include engine value spans: {mysql_output}"
    );
    assert!(
        mysql_output.contains("type.builtin") && mysql_output.contains("utf8mb4"),
        "expected mysql semantic overlay to include charset value spans: {mysql_output}"
    );
}

#[test]
fn jsdoc_semantic_overlay_reports_inline_reference_targets() {
    let output = debug_semantics(
        "jsdoc",
        "/**\n * Render {@link ThemePreview#render} and {@link module:theme/preview}.\n */",
    )
    .expect("failed to render jsdoc semantic overlay");

    assert!(
        output.contains("variable.jsdoc") && output.contains("ThemePreview"),
        "expected jsdoc semantic overlay to include member reference identifiers: {output}"
    );
    assert!(
        output.contains("punctuation.delimiter") && output.contains("#"),
        "expected jsdoc semantic overlay to include member delimiter spans: {output}"
    );
    assert!(
        output.contains("text.uri") && output.contains("theme"),
        "expected jsdoc semantic overlay to include path-like inline reference spans: {output}"
    );
    assert!(
        output.contains("punctuation.delimiter") && output.contains("/"),
        "expected jsdoc semantic overlay to include path delimiter spans: {output}"
    );
}

#[test]
fn graphql_runtime_is_reused_by_javascript_and_markdown_hosts() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let graphql_path = fixture_path("graphql/schema.graphql");
    let graphql_source = read_file(&graphql_path);
    let graphql_analysis =
        analysis_snapshot_for_path(graphql_path.as_path(), &graphql_source, &theme);
    assert_eq!(
        graphql_analysis
            .resolved_document_kind
            .as_ref()
            .map(|kind| kind.runtime_name),
        Some("graphql")
    );

    let js_path = fixture_path("javascript/graphql.js");
    let js_source = read_file(&js_path);
    let js_analysis = analysis_snapshot_for_path(js_path.as_path(), &js_source, &theme);
    assert!(nested_runtime_names(&js_analysis).contains(&"graphql"));

    let markdown_path = fixture_path("markdown/graphql_fence.md");
    let markdown_source = read_file(&markdown_path);
    let markdown_analysis =
        analysis_snapshot_for_path(markdown_path.as_path(), &markdown_source, &theme);
    assert!(nested_runtime_names(&markdown_analysis).contains(&"graphql"));
}

#[test]
fn injected_python_reuses_the_same_highlight_rules() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let just_path = fixture_path("just/injections.just");
    let just_source = read_file(&just_path);
    let just_analysis = analysis_snapshot_for_path(just_path.as_path(), &just_source, &theme);
    assert!(nested_runtime_names(&just_analysis).contains(&"python"));

    let markdown_path = Path::new("testdata/showcase/markdown/mixed-code-blocks.md").to_path_buf();
    let markdown_source = read_file(&markdown_path);
    let markdown_analysis =
        analysis_snapshot_for_path(markdown_path.as_path(), &markdown_source, &theme);
    assert!(nested_runtime_names(&markdown_analysis).contains(&"python"));
}

#[test]
fn userscript_metadata_block_highlights_keys_urls_patterns_and_special_values() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("javascript/userscript.user.js");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m==UserScript=="),
        "expected userscript start marker styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mname"),
        "expected userscript metadata key styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m:zh-CN"),
        "expected userscript localized key suffix styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mhttps://example.com/kat"),
        "expected userscript URL styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;85;85m*://example.com/*"),
        "expected userscript match-pattern styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mGM_addStyle"),
        "expected userscript grant API styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mnone"),
        "expected userscript special grant value styling"
    );
    assert!(
        rendered.contains("icon"),
        "expected userscript resource alias styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mdocument-start"),
        "expected userscript run-at enum styling"
    );
}

#[test]
fn userscript_metadata_highlighting_reuses_javascript_runtime_in_markdown_fences() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("markdown/userscript_fence.md");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m==UserScript=="),
        "expected fenced JavaScript userscript marker styling in Markdown"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mgrant"),
        "expected fenced JavaScript userscript key styling in Markdown"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mGM_addStyle"),
        "expected fenced JavaScript userscript grant styling in Markdown"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mdocument-end"),
        "expected fenced JavaScript userscript enum styling in Markdown"
    );
}

#[test]
fn json_highlights_keys_booleans_escapes_and_punctuation() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("json/rich.json");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253m\"theme\""),
        "expected JSON object key styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108mtrue"),
        "expected JSON boolean styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mnull"),
        "expected JSON null styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m\\\\"),
        "expected JSON escape sequence styling"
    );
    assert!(
        strip_ansi(&rendered).contains("{\n"),
        "expected JSON bracket punctuation styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m:"),
        "expected JSON delimiter punctuation styling"
    );
}

#[test]
fn bash_highlights_shebang_special_variables_regex_and_parameters() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("bash/rich.sh");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m#!/usr/bin/env bash"),
        "expected Bash shebang directive styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m=~"),
        "expected Bash regex operator styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;85;85m^kat.*"),
        "expected Bash regex literal styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249m#"),
        "expected Bash special variable styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123mpipefail"),
        "expected Bash set option styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m$'line\\nsecond'"),
        "expected Bash ANSI-C string styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mread"),
        "expected Bash read builtin styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mdeclare"),
        "expected Bash declare builtin styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253munset"),
        "expected Bash unset builtin styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;255;184;108mtheme_line"),
        "expected Bash read target variable styling"
    );
    assert!(
        strip_ansi(&rendered).contains("palettes[0]"),
        "expected Bash subscript bracket styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108m0"),
        "expected Bash subscript index number styling"
    );
}

#[test]
fn toml_highlights_quoted_keys_datetimes_inline_tables_and_cargo_lock() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let rich_path = fixture_path("toml/rich.toml");
    let rich_source = read_file(&rich_path);
    let rich_rendered = render_with_theme(Some(rich_path.as_path()), &rich_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", rich_path.display()));

    assert!(
        rich_rendered.contains("\x1b[38;2;139;233;253m\"display-name\""),
        "expected TOML quoted-key property styling"
    );
    assert!(
        rich_rendered.contains("\x1b[38;2;255;121;198m\\n"),
        "expected TOML escape sequence styling"
    );
    assert!(
        rich_rendered.contains("\x1b[38;2;255;184;108m2026-03-26T18:00:00Z"),
        "expected TOML datetime styling"
    );
    assert!(
        strip_ansi(&rich_rendered).contains("{ enabled = true"),
        "expected TOML inline-table bracket styling"
    );
    assert!(
        rich_rendered.contains("\x1b[38;2;255;184;108mtrue"),
        "expected TOML boolean styling inside inline table"
    );

    let cargo_lock_path = fixture_path("toml/Cargo.lock");
    let cargo_lock_source = read_file(&cargo_lock_path);
    let cargo_lock_rendered =
        render_with_theme(Some(cargo_lock_path.as_path()), &cargo_lock_source, &theme)
            .unwrap_or_else(|error| {
                panic!("failed to render {}: {error}", cargo_lock_path.display())
            });

    assert!(
        cargo_lock_rendered.contains("\x1b["),
        "expected Cargo.lock to be detected as TOML"
    );
    assert!(
        cargo_lock_rendered.contains("\x1b[38;2;139;233;253mversion"),
        "expected Cargo.lock keys to reuse TOML property styling"
    );

    let uv_lock_path = fixture_path("toml/uv.lock");
    let uv_lock_source = read_file(&uv_lock_path);
    let uv_lock_rendered = render_with_theme(Some(uv_lock_path.as_path()), &uv_lock_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", uv_lock_path.display()));

    assert!(
        uv_lock_rendered.contains("\x1b["),
        "expected uv.lock to be detected as TOML"
    );
    assert!(
        uv_lock_rendered.contains("\x1b[38;2;139;233;253mversion"),
        "expected uv.lock keys to reuse TOML property styling"
    );
}

#[test]
fn yaml_highlights_anchors_tags_block_scalars_and_github_script_injection() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("yaml/actions.yaml");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[4m\x1b[38;2;80;250;123m&defaults"),
        "expected YAML anchor styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[4m\x1b[38;2;80;250;123m*defaults"),
        "expected YAML alias styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m!color"),
        "expected YAML tag styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m|"),
        "expected YAML block-scalar indicator styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected YAML github-script block to inject JavaScript"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198mconst"),
        "expected YAML github-script block to reuse JavaScript keyword styling"
    );
}

#[test]
fn github_actions_yaml_profiles_inject_run_steps_and_highlight_expressions() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let workflow_path = fixture_path("yaml/github-actions-workflow.yaml");
    let workflow_source = read_file(&workflow_path);
    let workflow_rendered = render_with_theme(
        Some(Path::new(".github/workflows/build.yml")),
        &workflow_source,
        &theme,
    )
    .unwrap_or_else(|error| panic!("failed to render {}: {error}", workflow_path.display()));
    let workflow_regions = collect_top_level_injection_regions(
        yaml_document_kind(Some(Path::new(".github/workflows/build.yml"))),
        &workflow_source,
        &theme,
        1,
        None,
    )
    .expect("expected GitHub Actions workflow injections to resolve");
    assert!(
        workflow_regions.iter().any(|region| {
            region.overlays.iter().any(|span| {
                &workflow_source[span.range.clone()] == "printf"
                    && span.style == theme.token_style_for("function.builtin", "printf")
            })
        }),
        "expected GitHub Actions host resolver to produce Bash overlays for run blocks"
    );

    assert!(
        workflow_rendered.contains("\x1b[38;2;139;233;253mprintf"),
        "expected GitHub Actions run block to reuse Bash runtime styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;139;233;253mprint"),
        "expected shell: python run block to reuse Python builtin styling"
    );
    assert!(
        strip_ansi(&workflow_rendered).contains("github.ref"),
        "expected GitHub Actions expressions to highlight context identifiers"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;255;121;198m&&"),
        "expected GitHub Actions expressions to highlight logical operators"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;139;233;253mactions"),
        "expected uses owner to receive dedicated GitHub Actions styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;80;250;123mcheckout"),
        "expected uses repository to receive dedicated GitHub Actions styling"
    );

    let action_path = fixture_path("yaml/action.yaml");
    let action_source = read_file(&action_path);
    let action_rendered = render_with_theme(Some(Path::new("action.yml")), &action_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", action_path.display()));
    assert!(
        action_rendered.contains("\x1b[38;2;80;250;123mecho"),
        "expected composite action run step to reuse Bash runtime styling"
    );
    assert!(
        action_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mcomposite"),
        "expected runs.using values to receive GitHub Actions schema-aware styling"
    );
}

#[test]
fn github_actions_advanced_workflow_inherits_default_shell_and_highlights_nested_expressions() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let workflow_path = fixture_path("yaml/github-actions-workflow-advanced.yaml");
    let workflow_source = read_file(&workflow_path);
    let workflow_rendered = render_with_theme(
        Some(Path::new(".github/workflows/example.yml")),
        &workflow_source,
        &theme,
    )
    .unwrap_or_else(|error| panic!("failed to render {}: {error}", workflow_path.display()));
    let workflow_regions = collect_top_level_injection_regions(
        yaml_document_kind(Some(Path::new(".github/workflows/example.yml"))),
        &workflow_source,
        &theme,
        1,
        None,
    )
    .expect("expected advanced GitHub Actions workflow injections to resolve");

    assert!(
        workflow_regions.iter().any(|region| {
            region.overlays.iter().any(|span| {
                &workflow_source[span.range.clone()] == "set"
                    && span.style == theme.token_style_for("function.builtin", "set")
            })
        }),
        "expected defaults.run.shell to drive Bash highlighting for run blocks"
    );
    assert!(
        workflow_regions.iter().any(|region| {
            region.overlays.iter().any(|span| {
                &workflow_source[span.range.clone()] == "print"
                    && span.style == theme.token_style_for("function.builtin", "print")
            })
        }),
        "expected shell templates like python {{0}} to resolve to the Python runtime"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;255;121;198m&&"),
        "expected bare if expressions to reuse GitHub Actions operator styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;255;121;198m||"),
        "expected bare if expressions to reuse GitHub Actions operator styling"
    );
    assert!(
        workflow_regions.iter().any(|region| {
            region.overlays.iter().any(|span| {
                &workflow_source[span.range.clone()] == "${{"
                    && span.style == theme.token_style_for("punctuation.special", "${{")
            })
        }),
        "expected GitHub Actions expressions inside run blocks to keep workflow punctuation styling"
    );
    assert!(
        workflow_regions.iter().any(|region| {
            region.overlays.iter().any(|span| {
                &workflow_source[span.range.clone()] == "target"
                    && span.style == theme.token_style_for("property", "target")
            })
        }),
        "expected nested run-block expressions to highlight GitHub Actions property segments"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;255;121;198mdocker://"),
        "expected docker uses refs to highlight the docker:// prefix"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;80;250;123msetup-node"),
        "expected uses refs with subpaths to keep repository highlighting"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;241;250;140mbin"),
        "expected uses refs with subpaths to highlight the nested action path"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;139;233;253mWrite-Host"),
        "expected pwsh run blocks to reuse PowerShell builtin highlighting"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;255;121;198m@echo off"),
        "expected cmd run blocks to reuse batch highlighting"
    );
    assert!(
        workflow_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mpwsh"),
        "expected shell values to receive schema-aware runtime styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mcmd"),
        "expected cmd shell values to receive schema-aware runtime styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mread"),
        "expected permissions values to receive schema-aware styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mpnpm"),
        "expected with.cache values to receive schema-aware styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[3m\x1b[38;2;139;233;253merror"),
        "expected with.if-no-files-found values to receive schema-aware styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;139;233;253mubuntu-latest"),
        "expected runs-on labels to receive GitHub Actions runner styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;139;233;253mself-hosted"),
        "expected runs-on array labels to receive GitHub Actions runner styling"
    );
}

#[test]
fn prek_toml_profile_injects_system_hook_entries_into_bash_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let source = r#"minimum_prek_version = "0.3.8"

[[repos]]
repo = "local"

[[repos.hooks]]
id = "clippy"
language = "system"
entry = "cargo clippy --workspace --locked --all-targets --all-features -- -D warnings"

[[repos.hooks]]
id = "pytest"
language = "python"
entry = "python -m pytest -q"
"#;
    let document_kind = detect_document_kind(Some(Path::new("prek.toml")), source)
        .expect("expected prek.toml to resolve as TOML");
    let regions = collect_top_level_injection_regions(document_kind, source, &theme, 1, None)
        .expect("expected prek.toml injections to resolve");

    assert_eq!(
        regions
            .iter()
            .map(|region| region.resolved_document_kind.runtime_name())
            .collect::<Vec<_>>(),
        vec!["bash"],
        "expected only system hook entries to inject a shell runtime"
    );
    assert!(
        regions.iter().any(|region| {
            region.overlays.iter().any(|span| {
                &source[span.range.clone()] == "cargo"
                    && span.style == theme.token_style_for("function", "cargo")
            })
        }),
        "expected prek.toml system entry to reuse Bash command-name styling"
    );
}

#[test]
fn prek_toml_system_entry_stays_inline_without_block_tint() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let source = r#"minimum_prek_version = "0.3.8"

[[repos]]
repo = "local"

[[repos.hooks]]
id = "clippy"
language = "system"
entry = "cargo clippy --workspace --locked --all-targets --all-features -- -D warnings"
"#;
    let analysis = analysis_snapshot_for_path(Path::new("prek.toml"), source, &theme);
    let region = find_nested_region(&analysis, "bash", "cargo clippy --workspace", source);
    assert_eq!(
        region.visual_kind, "transparent",
        "expected single-line prek hook entries to stay inline instead of becoming block regions"
    );
    assert!(
        region_covers_line(region, source, "cargo clippy --workspace"),
        "expected injected shell region to cover the entry line"
    );
}

#[test]
fn proto_highlights_keywords_types_and_literals() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let rendered = render_with_theme(
        Some(Path::new("theme.proto")),
        r#"syntax = "proto3";

message ThemePreview {
  string name = 1;
  bool enabled = 2;
}
"#,
        &theme,
    )
    .expect("expected proto highlight to succeed");

    assert!(
        rendered.contains("\x1b["),
        "expected proto output to contain ANSI styling"
    );
    assert!(
        rendered.contains("syntax"),
        "expected proto keyword to be present in rendered output"
    );
    assert!(
        rendered.contains("\"proto3\""),
        "expected proto syntax literal to be present in rendered output"
    );
    assert!(
        rendered.contains("ThemePreview"),
        "expected proto message name to be present in rendered output"
    );
    assert!(
        rendered.contains("string"),
        "expected proto scalar type to be present in rendered output"
    );
    assert!(
        rendered.contains("1"),
        "expected proto field number to be present in rendered output"
    );
}

#[test]
fn textproto_highlights_fields_strings_booleans_and_numbers() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let rendered = render_with_theme(
        Some(Path::new("theme.textproto")),
        r#"name: "Dracula"
enabled: true
priority: 7
"#,
        &theme,
    )
    .expect("expected textproto highlight to succeed");

    assert!(
        rendered.contains("\x1b["),
        "expected textproto output to contain ANSI styling"
    );
    assert!(
        rendered.contains("name"),
        "expected textproto field name to be present in rendered output"
    );
    assert!(
        rendered.contains(":"),
        "expected textproto delimiter to be present in rendered output"
    );
    assert!(
        rendered.contains("\"Dracula\""),
        "expected textproto string to be present in rendered output"
    );
    assert!(
        rendered.contains("true"),
        "expected textproto boolean to be present in rendered output"
    );
    assert!(
        rendered.contains("7"),
        "expected textproto number to be present in rendered output"
    );
}

#[test]
fn rust_highlights_traits_attributes_macros_and_special_variables() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("rust/rich.rs");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123mderive"),
        "expected Rust attribute name styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mRenderable"),
        "expected Rust trait/interface styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mrender"),
        "expected Rust method call styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mthemed"),
        "expected Rust macro styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249mself"),
        "expected Rust self styling"
    );
    assert!(
        strip_ansi(&rendered).contains(" preview "),
        "expected Rust local variable bindings to keep foreground styling"
    );
}

#[test]
fn just_recipe_body_uses_block_region_tint_without_tinting_header_line() {
    let source = "install:\n    cargo install --path .\n";
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let analysis = analysis_snapshot_for_path(Path::new("Justfile"), source, &theme);
    let region = find_nested_region(&analysis, "bash", "cargo install --path .", source);

    assert!(
        !region_covers_line(region, source, "install:"),
        "recipe header line should not be part of the nested block"
    );
    assert!(
        region_covers_line(region, source, "cargo install --path ."),
        "first recipe command line should receive nested block coverage"
    );
}

#[test]
fn just_recipe_body_uses_rectangular_visual_kind() {
    let source = "install:\n    cargo install --path .\n    cargo fmt --check\n";
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let analysis = analysis_snapshot_for_path(Path::new("Justfile"), source, &theme);
    let region = find_nested_region(&analysis, "bash", "cargo install --path .", source);
    let first_segment = segment_for_line(region, source, "cargo install --path .");

    assert_eq!(
        region.visual_kind, "rect_block",
        "expected Justfile recipe bodies to use rectangular visuals"
    );
    assert_eq!(
        segment_left_column(source, first_segment),
        4,
        "expected recipe background to align with the shared recipe indent"
    );
    let layout = layout_snapshot_for_path(Path::new("Justfile"), source, &theme, 80);
    let first_row = layout_row_containing(&layout.rows, "cargo install --path .");
    let second_row = layout_row_containing(&layout.rows, "cargo fmt --check");
    assert_eq!(
        first_background_bounds(first_row),
        first_background_bounds(second_row),
        "expected recipe rows to share one rectangle"
    );
}

#[test]
fn just_recipe_trailing_separator_blank_line_stays_outside_block_region() {
    let source = "install:\n    cargo install --path .\n\nnext:\n    echo hi\n";
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let analysis = analysis_snapshot_for_path(Path::new("Justfile"), source, &theme);
    let first_region = find_nested_region(&analysis, "bash", "cargo install --path .", source);
    let second_region = find_nested_region(&analysis, "bash", "echo hi", source);

    assert!(
        region_covers_line(first_region, source, "cargo install --path ."),
        "first recipe body line should receive nested block coverage"
    );
    assert!(
        !first_region
            .layout_segments
            .iter()
            .any(|segment| segment.line_start == line_start_for_line_number(source, 2)),
        "separator blank line between recipes should stay outside the previous block tint"
    );
    assert!(
        !first_region
            .layout_segments
            .iter()
            .any(|segment| segment.line_start == line_start_for_line_number(source, 3)),
        "next recipe header line should stay outside the previous block tint"
    );
    assert!(
        region_covers_line(second_region, source, "echo hi"),
        "next recipe body line should still receive its own nested block coverage"
    );
}

#[test]
fn markdown_fenced_code_uses_block_region_tint_only_inside_fence_body() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("markdown/go_fence.md");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let first_region = find_nested_region(&analysis, "go", "package preview", &source);
    let second_region = find_nested_region(
        &analysis,
        "go",
        "func NewRenderer(name string) string {",
        &source,
    );
    assert!(
        region_covers_line(first_region, &source, "package preview"),
        "fenced code content should receive nested block coverage"
    );
    assert!(
        first_region
            .layout_segments
            .iter()
            .any(|segment| segment.line_start == line_start_for_line_number(&source, 4)),
        "empty line inside first fenced block should still be part of the block region"
    );
    assert!(
        second_region
            .layout_segments
            .iter()
            .any(|segment| segment.line_start == line_start_for_line_number(&source, 14)),
        "empty line inside second fenced block should still be part of the block region"
    );
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 120);
    for line_number in [4, 14] {
        let row = layout
            .rows
            .iter()
            .find(|row| row.source_line_start == line_start_for_line_number(&source, line_number))
            .unwrap_or_else(|| panic!("expected layout row for blank line {line_number}"));
        assert!(
            first_background_bounds(row).is_some(),
            "expected blank fenced-code line {line_number} to keep block background"
        );
    }
}

#[test]
fn markdown_fenced_code_wraps_into_one_exact_block_bbox() {
    let source = "# Demo\n\n```bash\necho short\n\necho 这里是一段足够长的中文内容用于验证围栏代码换行后的统一背景边界\n```\n";
    let theme =
        Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(RgbColor(1, 2, 3)));
    let layout =
        crate::api::layout_with_theme(Some(Path::new("demo.md")), source, &theme, Some(48))
            .expect("expected fenced Markdown layout")
            .snapshot();
    let fence_rows = layout
        .rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| row.text.starts_with("```").then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(fence_rows.len(), 2);
    let block_rows = &layout.rows[fence_rows[0] + 1..fence_rows[1]];

    assert!(block_rows.len() > 3, "expected the CJK code line to wrap");
    assert!(
        block_rows
            .iter()
            .all(|row| background_bounds_at_level(row, 1) == Some((0, 47))),
        "expected content, blank, and continuation rows to share the exact non-viewport-wide bbox: {block_rows:?}"
    );
    assert!(
        fence_rows
            .iter()
            .all(|index| { background_bounds_at_level(&layout.rows[*index], 1).is_none() }),
        "expected fence rows to stay outside the fenced-code Block"
    );
}

#[test]
fn github_actions_run_block_uses_block_region_tint_without_tinting_run_header() {
    let source = "jobs:\n  build:\n    steps:\n      - run: |\n          echo hi\n          printf '%s\\n' \"$GITHUB_REF\"\n";
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let analysis =
        analysis_snapshot_for_path(Path::new(".github/workflows/demo.yml"), source, &theme);
    let region = find_nested_region(&analysis, "bash", "echo hi", source);
    assert!(
        !region_covers_line(region, source, "- run: |"),
        "run: | header line should stay outside the nested block"
    );
    assert!(
        region_covers_line(region, source, "echo hi"),
        "first run body line should receive nested block coverage"
    );
    assert!(
        region_covers_line(region, source, "printf '%s\\n' \"$GITHUB_REF\""),
        "second run body line should receive nested block coverage"
    );

    let layout =
        layout_snapshot_for_path(Path::new(".github/workflows/demo.yml"), source, &theme, 120);
    assert_eq!(
        first_background_bounds(layout_row_containing(&layout.rows, "echo hi")),
        first_background_bounds(layout_row_containing(&layout.rows, "printf '%s\\n'")),
        "expected run body rows to share one layout-time block bbox"
    );
}

#[test]
fn github_actions_run_block_keeps_consistent_rendered_right_edge_with_cjk_content() {
    let source = "jobs:\n  build:\n    steps:\n      - run: |\n          echo \"短描述\"\n          echo \"这里放一段更长的中文描述用来验证共享几何也会补齐右边界\"\n";
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let layout = layout_snapshot_for_path(
        Path::new(".github/workflows/demo-cjk.yml"),
        source,
        &theme,
        49,
    );
    let member_line_starts = [
        line_start_containing(source, "echo \"短描述\""),
        line_start_containing(source, "这里放一段更长的中文描述"),
    ];
    let rows = layout
        .rows
        .iter()
        .filter(|row| member_line_starts.contains(&row.source_line_start))
        .collect::<Vec<_>>();
    assert!(rows.len() > 2, "expected the CJK run body to wrap");
    assert!(
        rows.iter()
            .all(|row| background_bounds_at_level(row, 1) == Some((0, 48))),
        "expected every GitHub Actions run content and continuation row to share the exact non-viewport-wide bbox: {rows:?}"
    );
}

#[test]
fn nested_region_tint_levels_scale_monotonically_for_five_layers() {
    let tint = RgbColor(16, 24, 32);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let mut previous_luminance = relative_luminance(tint);

    for level in 1..=5 {
        let background = theme
            .nested_region_background(level)
            .unwrap_or_else(|| panic!("expected nested region tint for level {level}"));
        let luminance = relative_luminance(background);
        assert!(
            luminance > previous_luminance,
            "nested region level {level} should be brighter than the previous level"
        );
        previous_luminance = luminance;
    }
}

fn analysis_snapshot_for_path(path: &Path, source: &str, theme: &Theme) -> AnalysisSnapshot {
    analyze_with_theme(Some(path), source, theme)
        .unwrap_or_else(|error| panic!("failed to analyze {}: {error}", path.display()))
        .snapshot()
}

fn render_plan_snapshot_for_path(path: &Path, source: &str, theme: &Theme) -> RenderPlanSnapshot {
    render_plan_with_theme(Some(path), source, theme)
        .unwrap_or_else(|error| panic!("failed to build render plan {}: {error}", path.display()))
        .snapshot(*theme)
}

fn layout_snapshot_for_path(
    path: &Path,
    source: &str,
    theme: &Theme,
    terminal_width: usize,
) -> LayoutSnapshot {
    crate::api::layout_with_theme(Some(path), source, theme, Some(terminal_width))
        .unwrap_or_else(|error| panic!("failed to build layout {}: {error}", path.display()))
        .snapshot()
}

fn nested_regions(regions: &[NestedRegionSnapshot]) -> Vec<&NestedRegionSnapshot> {
    let mut flattened = Vec::new();
    for region in regions {
        flattened.push(region);
        flattened.extend(nested_regions(&region.child_nested_regions));
    }
    flattened
}

fn nested_runtime_names(snapshot: &AnalysisSnapshot) -> Vec<&'static str> {
    nested_regions(&snapshot.nested_regions)
        .into_iter()
        .map(|region| region.resolved_document_kind.runtime_name)
        .collect()
}

fn find_nested_region<'a>(
    snapshot: &'a AnalysisSnapshot,
    runtime_name: &str,
    line_fragment: &str,
    source: &str,
) -> &'a NestedRegionSnapshot {
    let line_start = line_start_containing(source, line_fragment);
    nested_regions(&snapshot.nested_regions)
            .into_iter()
            .find(|region| {
                region.resolved_document_kind.runtime_name == runtime_name
                    && region
                        .layout_segments
                        .iter()
                        .any(|segment| segment.line_start == line_start)
            })
            .unwrap_or_else(|| {
                panic!(
                    "expected nested region runtime {runtime_name:?} to cover line fragment {line_fragment:?}"
                )
            })
}

fn region_covers_line(region: &NestedRegionSnapshot, source: &str, line_fragment: &str) -> bool {
    let line_start = line_start_containing(source, line_fragment);
    region
        .layout_segments
        .iter()
        .any(|segment| segment.line_start == line_start)
}

fn line_start_containing(source: &str, needle: &str) -> usize {
    let start = source
        .find(needle)
        .unwrap_or_else(|| panic!("expected source to contain line fragment {needle:?}"));
    crate::injection::line_start_offset(source, start)
}

fn line_start_for_line_number(source: &str, line_number: usize) -> usize {
    if line_number == 0 {
        return 0;
    }

    source
        .match_indices('\n')
        .nth(line_number - 1)
        .map(|(index, _)| index + 1)
        .unwrap_or_else(|| panic!("expected source to have line number {line_number}"))
}

fn line_text_at(source: &str, line_start: usize) -> &str {
    let line_end = source[line_start..]
        .find('\n')
        .map(|offset| line_start + offset)
        .unwrap_or(source.len());
    &source[line_start..line_end]
}

fn segment_for_line<'a>(
    region: &'a NestedRegionSnapshot,
    source: &str,
    line_fragment: &str,
) -> &'a RegionSegmentSnapshot {
    let line_start = line_start_containing(source, line_fragment);
    region
        .layout_segments
        .iter()
        .find(|segment| segment.line_start == line_start)
        .unwrap_or_else(|| panic!("expected region to cover line fragment {line_fragment:?}"))
}

fn segment_left_column(source: &str, segment: &RegionSegmentSnapshot) -> usize {
    segment
        .left_column_override
        .unwrap_or_else(|| display_width(&source[segment.line_start..segment.left]).as_usize())
}

fn render_plan_text(snapshot: &RenderPlanSnapshot) -> String {
    let mut text = String::new();
    for op in &snapshot.ops {
        match op {
            RenderOpSnapshot::Text { text: chunk } => text.push_str(chunk),
            RenderOpSnapshot::Newline => text.push('\n'),
            RenderOpSnapshot::StartHyperlink { .. }
            | RenderOpSnapshot::EndHyperlink
            | RenderOpSnapshot::SetStyle { .. }
            | RenderOpSnapshot::ResetStyle => {}
        }
    }
    text
}

fn render_plan_lines(snapshot: &RenderPlanSnapshot) -> Vec<Vec<RenderOpSnapshot>> {
    let mut lines = vec![Vec::new()];
    for op in &snapshot.ops {
        match op {
            RenderOpSnapshot::Newline => lines.push(Vec::new()),
            other => lines
                .last_mut()
                .expect("render plan should always have a current line")
                .push(other.clone()),
        }
    }
    lines
}

fn layout_row_containing<'a>(rows: &'a [LayoutRowSnapshot], needle: &str) -> &'a LayoutRowSnapshot {
    rows.iter()
        .find(|row| row.text.contains(needle))
        .unwrap_or_else(|| {
            panic!(
                "expected layout rows to contain text fragment {needle:?}, got rows {:?}",
                rows.iter().map(|row| row.text.as_str()).collect::<Vec<_>>()
            )
        })
}

fn first_background_bounds(row: &LayoutRowSnapshot) -> Option<(usize, usize)> {
    row.background_runs
        .first()
        .map(|run| (run.start_column, run.end_column))
}

fn background_bounds_at_level(
    row: &LayoutRowSnapshot,
    visual_level: usize,
) -> Option<(usize, usize)> {
    row.background_runs
        .iter()
        .find(|run| run.visual_level == visual_level)
        .map(|run| (run.start_column, run.end_column))
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

fn relative_luminance(color: RgbColor) -> f32 {
    0.2126 * color.0 as f32 + 0.7152 * color.1 as f32 + 0.0722 * color.2 as f32
}

#[test]
fn overlay_style_spans_only_splits_overlapping_ranges() {
    let theme = Theme::for_mode(ColorMode::Ansi);
    let parent_style = theme.token_style_for("string", "value");
    let overlay_style = theme.token_style_for("comment", "//");
    let parent_spans = vec![StyledSpan {
        range: 0..10,
        style: parent_style,
        hyperlink: None,
    }];
    let overlay_spans = vec![
        StyledSpan {
            range: 2..4,
            style: overlay_style,
            hyperlink: None,
        },
        StyledSpan {
            range: 6..8,
            style: overlay_style,
            hyperlink: None,
        },
    ];

    let result = overlay_style_spans(parent_spans, overlay_spans);

    assert_eq!(
        result,
        vec![
            StyledSpan {
                range: 0..2,
                style: parent_style,
                hyperlink: None,
            },
            StyledSpan {
                range: 2..4,
                style: overlay_style,
                hyperlink: None,
            },
            StyledSpan {
                range: 4..6,
                style: parent_style,
                hyperlink: None,
            },
            StyledSpan {
                range: 6..8,
                style: overlay_style,
                hyperlink: None,
            },
            StyledSpan {
                range: 8..10,
                style: parent_style,
                hyperlink: None,
            },
        ]
    );
}

#[test]
fn overlay_nested_region_merges_parent_style_only_inside_child_overlays() {
    let theme = Theme::for_mode(ColorMode::Ansi);
    let parent_style = None;
    let child_style = theme.token_style_for("comment", "//");
    let parent_spans = vec![StyledSpan {
        range: 0..10,
        style: parent_style,
        hyperlink: None,
    }];
    let region = NestedRegion {
        visual_level: 0,
        resolved_document_kind: plain_document_kind("sql"),
        visual_kind: InjectionVisualKind::Transparent,
        layout_segments: Vec::<RegionSegment>::new(),
        child_regions: Vec::new(),
        child_nested_regions: Vec::new(),
        overlays: vec![StyledSpan {
            range: 2..4,
            style: child_style,
            hyperlink: None,
        }],
        merge_parent_styles: true,
    };

    let result = overlay_nested_region(parent_spans, &region);

    assert_eq!(
        result,
        vec![
            StyledSpan {
                range: 0..2,
                style: parent_style,
                hyperlink: None,
            },
            StyledSpan {
                range: 2..4,
                style: child_style,
                hyperlink: None,
            },
            StyledSpan {
                range: 4..10,
                style: parent_style,
                hyperlink: None,
            },
        ]
    );
}

#[test]
fn typescript_multiline_block_comments_keep_comment_style_on_following_lines() {
    let theme = Theme::for_mode(ColorMode::Ansi);
    let rendered = highlight_named_language("typescript", "/* hi\nworld\n*/\n", &theme)
        .expect("failed to highlight multiline TypeScript block comment");
    let lines = rendered.lines().collect::<Vec<_>>();

    assert!(
        lines
            .get(1)
            .is_some_and(|line| line.starts_with("\x1b[3m\x1b[90mworld")),
        "expected second block-comment line to keep comment style: {rendered:?}"
    );
    assert!(
        lines
            .get(2)
            .is_some_and(|line| line.starts_with("\x1b[3m\x1b[90m*/")),
        "expected closing block-comment line to keep comment style: {rendered:?}"
    );
}

#[test]
fn typescript_jsdoc_uses_block_region_tint() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let source = "/**\n * docs\n */\nconst answer = 42;\n";
    let analysis = analysis_snapshot_for_path(Path::new("demo.ts"), source, &theme);
    let region = find_nested_region(&analysis, "jsdoc", "docs", source);

    assert!(
        region_covers_line(region, source, "/**"),
        "expected JSDoc opening line to receive nested block coverage"
    );
    assert!(
        region_covers_line(region, source, "docs"),
        "expected JSDoc body line to receive nested block coverage"
    );
    assert!(
        region_covers_line(region, source, "*/"),
        "expected JSDoc closing line to receive nested block coverage"
    );
    assert!(
        !region_covers_line(region, source, "const answer = 42;"),
        "expected following code line to stay outside the JSDoc block"
    );
}

#[test]
fn typescript_jsdoc_prose_does_not_use_rectangular_trailing_padding() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let source = "/**\n * this is a much longer JSDoc line\n * short\n */\nconst answer = 42;\n";
    let analysis = analysis_snapshot_for_path(Path::new("demo.ts"), source, &theme);
    let region = find_nested_region(&analysis, "jsdoc", "short", source);
    assert!(
        region_covers_line(region, source, "short"),
        "expected shorter JSDoc prose line to keep its nested coverage"
    );
    assert_ne!(
        region.visual_kind, "rect_block",
        "expected JSDoc prose to avoid rectangular block layout"
    );
    let layout = layout_snapshot_for_path(Path::new("demo.ts"), source, &theme, 20);
    let long_row = layout_row_containing(&layout.rows, "this is a much");
    let short_row = layout_row_containing(&layout.rows, "* short");
    assert_eq!(background_bounds_at_level(long_row, 1), Some((0, 20)));
    assert_eq!(
        background_bounds_at_level(short_row, 1),
        Some((0, 8)),
        "expected TightBlock background to stop at the short row's own content edge"
    );
}

#[test]
fn typescript_comment_hint_template_injection_stays_inline() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let source = "const query = /* sql */ `\n  select 1\n`;\n";
    let analysis = analysis_snapshot_for_path(Path::new("demo.ts"), source, &theme);
    let region = find_nested_region(&analysis, "sql", "select 1", source);

    assert!(
        region.visual_kind == "transparent",
        "expected inline SQL template injection to avoid block tint"
    );
}

#[test]
fn markdown_truecolor_cjk_repro_avoids_excessive_reset_churn() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("markdown/markdown_truecolor_cjk_repro.md");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    let reset_count = count_occurrences(&rendered, "\x1b[0m");
    let default_fg_count = count_occurrences(&rendered, "\x1b[38;2;248;248;242m");

    assert!(
        reset_count <= 7,
        "expected compact truecolor output for the markdown CJK repro fixture, got {reset_count} resets: {rendered:?}"
    );
    assert!(
        default_fg_count <= 6,
        "expected markdown CJK repro fixture to avoid repeatedly re-entering the default foreground, got {default_fg_count}: {rendered:?}"
    );
}

#[test]
fn markdown_top_level_prose_does_not_pick_up_block_tint_from_inline_injections() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("markdown/markdown_truecolor_cjk_repro.md");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let prose_lines = [
        line_start_for_line_number(&source, 0),
        line_start_for_line_number(&source, 1),
    ];

    assert!(
        !nested_regions(&analysis.nested_regions)
            .into_iter()
            .any(|region| {
                region.visual_kind == "rect_block"
                    && region
                        .layout_segments
                        .iter()
                        .any(|segment| segment.line_start == prose_lines[0])
            }),
        "expected top-level markdown prose line to avoid block tint"
    );
    assert!(
        !nested_regions(&analysis.nested_regions)
            .into_iter()
            .any(|region| {
                region.visual_kind == "rect_block"
                    && region
                        .layout_segments
                        .iter()
                        .any(|segment| segment.line_start == prose_lines[1])
            }),
        "expected second top-level markdown prose line to avoid block tint"
    );
}

#[test]
fn markdown_html_block_table_keeps_consistent_rendered_right_edge_with_cjk_content() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = Path::new("demo.md");
    let source = "<div>\n<div>这里是一段足够长的中文内容用于验证HTML块换行后的所有视觉行共享同一个精确背景边界</div>\n</div>\n";
    let layout = layout_snapshot_for_path(path, source, &theme, 80);
    let table_start = layout
        .rows
        .iter()
        .position(|row| row.text == "<div>")
        .expect("expected opening HTML row");
    let table_end = layout
        .rows
        .iter()
        .position(|row| row.text == "</div>")
        .expect("expected closing HTML row");
    let rows = &layout.rows[table_start..=table_end];
    let expected_bounds = Some((0, 79));
    assert!(
        rows.len() > 3,
        "expected the long CJK table row to produce a continuation row"
    );
    assert!(
        rows.iter()
            .all(|row| first_background_bounds(row) == expected_bounds),
        "expected every HTML content and continuation row to share the exact non-viewport-wide bbox: {rows:?}"
    );
}

#[test]
fn markdown_html_block_tabs_keep_consistent_rendered_right_edge() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("markdown/html_block_tabs.md");
    let source = read_file(&path);
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 120);
    let rows = [
        layout_row_containing(&layout.rows, "<table>"),
        layout_row_containing(&layout.rows, "<td>值"),
        layout_row_containing(&layout.rows, "这里放一段更长的中文内容"),
        layout_row_containing(&layout.rows, "</table>"),
    ];
    let expected_bounds = first_background_bounds(rows[0]);
    assert!(expected_bounds.is_some());
    assert!(
        rows.iter()
            .all(|row| first_background_bounds(row) == expected_bounds),
        "expected tabbed HTML block rows to share one layout-time block bbox"
    );
}

#[test]
fn markdown_html_block_tabs_layout_expands_tabs_into_display_spaces() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("markdown/html_block_tabs.md");
    let source = read_file(&path);
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 120);
    let short_row = layout_row_containing(&layout.rows, "<td>值");
    let long_row = layout_row_containing(&layout.rows, "这里放一段更长的中文内容");

    assert!(
        !short_row.text.contains('\t'),
        "expected layout row text to expand tabs before rendering: {:?}",
        short_row.text
    );
    assert!(
        !long_row.text.contains('\t'),
        "expected layout row text to expand tabs before rendering: {:?}",
        long_row.text
    );
}

#[test]
fn rust_multiline_raw_string_sql_uses_block_region_tint() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("rust/injections.rs");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let region = find_nested_region(&analysis, "sql", "SELECT id, name, enabled", &source);

    assert!(
        region_covers_line(region, &source, "SELECT id, name, enabled"),
        "expected multiline Rust SQL raw string to receive nested block coverage"
    );
    let select_segment = segment_for_line(region, &source, "SELECT id, name, enabled");
    let background_column = segment_left_column(&source, select_segment);
    assert!(
        background_column == 8,
        "expected Rust SQL block to preserve host indentation before the block tint, got column {background_column}: {:?}",
        line_text_at(&source, select_segment.line_start)
    );
    assert!(
        region_covers_line(region, &source, "ORDER BY name DESC"),
        "expected trailing Rust SQL raw-string line to stay inside the block coverage"
    );
    assert!(
        !region_covers_line(region, &source, "\"#"),
        "expected Rust raw-string closing delimiter line to stay outside the block tint"
    );
}

#[test]
fn debug_analysis_json_reports_markdown_document_kind() {
    let path = fixture_path("markdown/markdown_truecolor_cjk_repro.md");
    let source = read_file(&path);
    let json = debug_analysis_json(Some(path.as_path()), &source)
        .expect("analysis debug json should render");
    let value: Value = serde_json::from_str(&json).expect("analysis json should parse");

    assert_eq!(value["detected_document_kind"]["runtime_name"], "markdown");
    assert!(
        value["spans"]
            .as_array()
            .is_some_and(|spans| !spans.is_empty())
    );
}

#[test]
fn debug_analysis_json_reports_nested_region_runtime_identity() {
    let path = fixture_path("just/injections.just");
    let source = read_file(&path);
    let json = debug_analysis_json(Some(path.as_path()), &source)
        .expect("analysis debug json should render");
    let value: Value = serde_json::from_str(&json).expect("analysis json should parse");
    let nested_regions = value["nested_regions"]
        .as_array()
        .expect("expected nested regions array");

    assert!(
        nested_regions.iter().any(|region| {
            region["resolved_document_kind"]["runtime_name"] == Value::String("python".into())
        }),
        "expected analysis nested regions to expose injected python runtime identity: {json}"
    );
    assert!(
        nested_regions.iter().any(|region| {
            region["resolved_document_kind"]["runtime_name"] == Value::String("javascript".into())
        }),
        "expected analysis nested regions to expose injected javascript runtime identity: {json}"
    );
}

#[test]
fn debug_analysis_json_preserves_recursive_child_nested_regions() {
    let path = fixture_path("rust/doc_comments_nested.rs");
    let source = read_file(&path);
    let json = debug_analysis_json(Some(path.as_path()), &source)
        .expect("analysis debug json should render");
    let value: Value = serde_json::from_str(&json).expect("analysis json should parse");
    let nested_regions = value["nested_regions"]
        .as_array()
        .expect("expected nested regions array");
    let markdown_region = nested_regions
        .iter()
        .find(|region| {
            region["resolved_document_kind"]["runtime_name"] == Value::String("markdown".into())
        })
        .expect("expected outer rustdoc markdown region");
    let child_nested_regions = markdown_region["child_nested_regions"]
        .as_array()
        .expect("expected recursive child nested regions");

    assert!(
        child_nested_regions.iter().any(|region| {
            region["resolved_document_kind"]["runtime_name"] == Value::String("rust".into())
        }),
        "expected nested markdown region to preserve rust fenced block identity: {json}"
    );
    assert!(
        child_nested_regions.iter().any(|region| {
            region["resolved_document_kind"]["runtime_name"] == Value::String("python".into())
        }),
        "expected nested markdown region to preserve python fenced block identity: {json}"
    );
}

#[test]
fn debug_visual_json_reports_regions_array() {
    let path = fixture_path("markdown/markdown_truecolor_cjk_repro.md");
    let source = read_file(&path);
    let json =
        debug_visual_json(Some(path.as_path()), &source).expect("visual debug json should render");
    let value: Value = serde_json::from_str(&json).expect("visual json should parse");

    assert!(value["regions"].is_array());
}

#[test]
fn debug_layout_json_reports_rows_array() {
    let source = "install:\n    cargo install --path .\n";
    let json = debug_layout_json(Some(Path::new("Justfile")), source)
        .expect("layout debug json should render");
    let value: Value = serde_json::from_str(&json).expect("layout json should parse");

    assert!(value["rows"].is_array());
}

#[test]
fn markdown_frontmatter_block_bbox_follows_wrapped_rows() {
    let path = fixture_path("markdown/frontmatter-narrow.md");
    let source = read_file(&path);
    let theme =
        Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(RgbColor(1, 2, 3)));
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 48);

    assert!(
        layout.rows.iter().all(|row| row.display_width <= 48),
        "expected frontmatter background padding to stay within the 48-column viewport, got rows {:?}",
        layout
            .rows
            .iter()
            .map(|row| (&row.text, row.display_width))
            .collect::<Vec<_>>()
    );

    let delimiter_rows = layout
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| row.text == "---")
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    assert_eq!(delimiter_rows.len(), 2);
    let block_rows = &layout.rows[delimiter_rows[0]..=delimiter_rows[1]];
    assert_eq!(block_rows.len(), 5);
    assert!(
        block_rows.iter().all(|row| {
            row.display_width == 47 && first_background_bounds(row) == Some((0, 47))
        }),
        "expected every wrapped frontmatter row to share the 47-column block bbox, got {:?}",
        block_rows
            .iter()
            .map(|row| (&row.text, row.display_width, &row.background_runs))
            .collect::<Vec<_>>()
    );
}

#[test]
fn debug_layout_json_preserves_python_docstring_blank_line_background_runs() {
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let json = debug_layout_json(Some(path.as_path()), &source)
        .expect("layout debug json for python docstrings should render");
    let value: Value = serde_json::from_str(&json).expect("layout json should parse");
    let blank_line_start = line_start_for_line_number(&source, 42);
    let rows = value["rows"]
        .as_array()
        .unwrap_or_else(|| panic!("expected layout rows array: {json}"));
    let blank_row = rows
        .iter()
        .find(|row| row["source_line_start"].as_u64() == Some(blank_line_start as u64))
        .unwrap_or_else(|| panic!("expected layout row for blank init docstring line: {json}"));

    assert!(
        blank_row["background_runs"]
            .as_array()
            .is_some_and(|runs| !runs.is_empty()),
        "expected debug-layout json to preserve block background runs for blank python docstring lines: {json}"
    );
}

#[test]
fn just_recipe_rectangle_keeps_wrapped_rows_without_fake_blank_rows() {
    let theme =
        Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(RgbColor(1, 2, 3)));
    let path = PathBuf::from("testdata/showcase/just/recipe-block.just");
    let source = read_file(&path);
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 80);
    let first = layout_row_containing(&layout.rows, "@cd mdsre");
    let second = layout_row_containing(&layout.rows, "uv run mdsre mongo query");
    let continuation = layout_row_containing(&layout.rows, "ws.forEach(r =>");

    assert_eq!(
        first
            .background_runs
            .first()
            .map(|run| run.start_column)
            .unwrap_or_default(),
        0,
        "expected the recipe rectangle to include wrapped continuation rows"
    );
    assert_eq!(
        second
            .background_runs
            .first()
            .map(|run| run.start_column)
            .unwrap_or_default(),
        0,
        "expected all recipe rows to share the same left edge"
    );
    assert_eq!(
        continuation
            .background_runs
            .first()
            .map(|run| run.start_column)
            .unwrap_or(usize::MAX),
        0,
        "expected wrapped recipe rows to share the rectangle's left edge"
    );

    let first_index = layout
        .rows
        .iter()
        .position(|row| row.text.contains("@cd mdsre"))
        .expect("expected first recipe row");
    let second_index = layout
        .rows
        .iter()
        .position(|row| row.text.contains("uv run mdsre mongo query"))
        .expect("expected second recipe row");

    assert_eq!(
        second_index,
        first_index + 1,
        "expected no fake blank layout rows between recipe command rows: {:?}",
        layout
            .rows
            .iter()
            .map(|row| (&row.text, &row.background_runs))
            .collect::<Vec<_>>()
    );
}

#[test]
fn just_recipe_rectangle_tracks_cjk_wrapped_width_in_display_cells() {
    let theme =
        Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(RgbColor(1, 2, 3)));
    let source = "demo:\n    printf '短描述短描述短描述短描述短描述'\n";
    let layout =
        crate::api::layout_with_theme(Some(Path::new("Justfile")), source, &theme, Some(16))
            .expect("expected layout to render for CJK recipe")
            .snapshot();
    let first = layout_row_containing(&layout.rows, "    printf '短");
    let continuation = layout_row_containing(&layout.rows, "描述短描述");

    assert_eq!(
        first
            .background_runs
            .first()
            .map(|run| run.start_column)
            .unwrap_or_default(),
        0,
        "expected the CJK recipe rectangle to include wrapped rows"
    );
    assert_eq!(
        continuation
            .background_runs
            .first()
            .map(|run| run.start_column)
            .unwrap_or(usize::MAX),
        0,
        "expected wrapped CJK continuation rows to be measured in display cells"
    );
}

#[test]
fn just_recipe_nested_indentation_uses_one_rectangular_background() {
    let theme =
        Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(RgbColor(1, 2, 3)));
    let source = "demo:\n    echo outer\n        echo nested\n";
    let layout =
        crate::api::layout_with_theme(Some(Path::new("Justfile")), source, &theme, Some(80))
            .expect("expected layout to render for nested recipe indentation")
            .snapshot();

    assert_eq!(
        first_background_bounds(layout_row_containing(&layout.rows, "echo outer")),
        Some((4, 19)),
    );
    assert_eq!(
        first_background_bounds(layout_row_containing(&layout.rows, "echo nested")),
        Some((4, 19)),
        "expected every recipe row to share one rectangular left edge"
    );
}

#[test]
fn debug_visual_json_reports_rectangular_kind_for_just_recipe() {
    let source = "install:\n    cargo install --path .\n";
    let json = debug_visual_json(Some(Path::new("Justfile")), source)
        .expect("visual debug json for Justfile should render");
    let value: Value = serde_json::from_str(&json).expect("visual json should parse");
    let regions = value["regions"]
        .as_array()
        .unwrap_or_else(|| panic!("expected visual regions array: {json}"));

    assert!(
        regions
            .iter()
            .any(|region| region["visual_kind"] == Value::String("rect_block".into())),
        "expected visual debug json to preserve rectangular regions: {json}"
    );
}

#[test]
fn debug_render_ops_json_reports_text_ops() {
    let path = fixture_path("markdown/markdown_truecolor_cjk_repro.md");
    let source = read_file(&path);
    let json = debug_render_ops_json(Some(path.as_path()), &source)
        .expect("render-op debug json should render");
    let value: Value = serde_json::from_str(&json).expect("render-op json should parse");

    assert!(
        value["ops"]
            .as_array()
            .is_some_and(|ops| ops.iter().any(|op| op["op"] == "text"))
    );
}

#[test]
fn debug_terminal_json_reports_encoded_output() {
    let path = fixture_path("markdown/markdown_truecolor_cjk_repro.md");
    let source = read_file(&path);
    let json = debug_terminal_json(Some(path.as_path()), &source)
        .expect("terminal debug json should render");
    let value: Value = serde_json::from_str(&json).expect("terminal json should parse");

    assert!(
        value["encoded_output"]
            .as_str()
            .is_some_and(|output| !output.is_empty())
    );
    assert_eq!(
        value["capabilities"]["background_queries_enabled"],
        Value::Bool(false)
    );
}

#[test]
fn coffeescript_query_captures_class_and_method_names() {
    let path = fixture_path("coffeescript/theme.coffee");
    let source = read_file(&path);
    let language = runtime("coffeescript")
        .expect("expected coffeescript runtime")
        .language
        .clone();
    let query = Query::new(
        &language,
        include_str!("../grammars/coffeescript/queries/highlights.scm"),
    )
    .expect("expected coffeescript highlights query to compile");
    let capture_names = query.capture_names();
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .expect("expected coffeescript parser to load");
    let tree = parser
        .parse(&source, None)
        .expect("expected coffeescript tree");
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    let mut captures = Vec::new();

    while {
        matches.advance();
        matches.get().is_some()
    } {
        let query_match = matches
            .get()
            .expect("query match should exist immediately after advance");
        for capture in query_match.captures() {
            captures.push((
                capture_names[capture.index as usize].to_owned(),
                source[capture.node.byte_range()].to_owned(),
            ));
        }
    }

    assert!(
        captures
            .iter()
            .any(|(capture, text)| capture == "type" && text == "ThemePreview"),
        "expected CoffeeScript query to capture the class name, got {captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|(capture, text)| capture == "function.method" && text == "constructor"),
        "expected CoffeeScript query to capture constructor, got {captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|(capture, text)| capture == "function.method" && text == "render"),
        "expected CoffeeScript query to capture render, got {captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|(capture, text)| capture == "function.method" && text == "log"),
        "expected CoffeeScript query to capture member-call fallback names, got {captures:?}"
    );
}

#[test]
fn coffeescript_analysis_styles_class_and_method_names() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("coffeescript/theme.coffee");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let type_style = theme
        .token_style_for("type", "ThemePreview")
        .expect("expected type style");
    let method_style = theme
        .token_style_for("function.method", "constructor")
        .expect("expected function method style");
    let render_offset = source.find("render").expect("expected render token");
    let log_offset = source.find("log").expect("expected log token");
    let constructor_offset = source
        .find("constructor")
        .expect("expected constructor token");
    let class_offset = source.find("ThemePreview").expect("expected class token");

    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= class_offset
                && class_offset < span.end
                && span.style == Some(type_style.snapshot(ColorMode::TrueColor))
        }),
        "expected ThemePreview to use type styling, got {:?}",
        analysis.spans
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= constructor_offset
                && constructor_offset < span.end
                && span.style == Some(method_style.snapshot(ColorMode::TrueColor))
        }),
        "expected constructor to use method styling, got {:?}",
        analysis.spans
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= render_offset
                && render_offset < span.end
                && span.style == Some(method_style.snapshot(ColorMode::TrueColor))
        }),
        "expected render to use method styling, got {:?}",
        analysis.spans
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= log_offset
                && log_offset < span.end
                && span.style == Some(method_style.snapshot(ColorMode::TrueColor))
        }),
        "expected log to use method styling, got {:?}",
        analysis.spans
    );
}

fn fixture_path(relative_path: &str) -> PathBuf {
    Path::new("testdata/fixtures").join(relative_path)
}

fn fixture_family(relative_path: &str) -> &str {
    relative_path
        .split('/')
        .next()
        .expect("fixture path should have a top-level family directory")
}

fn showcase_family(path: &Path) -> &str {
    path.strip_prefix("testdata/showcase")
        .expect("showcase path should live under testdata/showcase")
        .components()
        .next()
        .and_then(|component| component.as_os_str().to_str())
        .expect("showcase path should have a top-level family directory")
}

fn read_file(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn collect_files(root: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files_recursive(Path::new(root), &mut files);
    files.sort();
    files
}

fn collect_files_recursive(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read directory {}: {error}", root.display()))
    {
        let entry = entry
            .unwrap_or_else(|error| panic!("failed to read entry in {}: {error}", root.display()));
        let path = entry.path();

        if path.is_dir() {
            collect_files_recursive(&path, files);
        } else {
            files.push(path);
        }
    }
}

fn is_supported_highlight_path(path: &Path) -> bool {
    detect_language(Some(path), "").is_some()
}
