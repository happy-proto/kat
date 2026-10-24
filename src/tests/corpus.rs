use super::*;

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
    "csh",
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
    "jam",
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
        relative_path: "csh/activation.csh",
        expect_highlight: true,
        expected_fragments: &["setenv", "$VIRTUAL_ENV:t:q", "deactivate", "endif"],
    },
    FixtureCase {
        relative_path: "csh/fences.md",
        expect_highlight: true,
        expected_fragments: &["foreach", "$file:q", "setenv"],
    },
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
        relative_path: "yaml/actions.lock",
        expect_highlight: true,
        expected_fragments: &["workflows", "dependencies", "example/build@v1", "12345"],
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
        relative_path: "xml/theme.tmTheme",
        expect_highlight: true,
        expected_fragments: &["DOCTYPE", "plist", "Kat &amp; Code", "#F8F8F2", "fontStyle"],
    },
    FixtureCase {
        relative_path: "jam/Jamfile",
        expect_highlight: true,
        expected_fragments: &["rule", "linking", "$(properties)", "ws2_32", "generate"],
    },
    FixtureCase {
        relative_path: "jam/upstream-regressions.jam",
        expect_highlight: true,
        expected_fragments: &["[path]", "#include", "$(file:E=(none))", "*[a-z]*"],
    },
    FixtureCase {
        relative_path: "jam/fences.md",
        expect_highlight: true,
        expected_fragments: &["greet", "example", "import"],
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
fn fixture_suite_matches_shell_and_config_rendering_behavior() {
    run_fixture_suite(FIXTURE_SHELL_AND_CONFIG_FAMILIES);
}

#[test]
fn fixture_suite_matches_systems_programming_rendering_behavior() {
    run_fixture_suite(FIXTURE_SYSTEMS_PROGRAMMING_FAMILIES);
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
    "csh",
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
