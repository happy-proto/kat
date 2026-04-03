use std::{collections::BTreeMap, sync::LazyLock};

use anyhow::{Context, Result};
use tree_sitter::Language;
use tree_sitter_highlight::HighlightConfiguration;
use tree_sitter_language::LanguageFn;

use crate::grammar_registry::HIGHLIGHT_NAMES;

const SQL_POSTGRES_HIGHLIGHTS_QUERY: &str = concat!(
    include_str!("../grammars/sql/queries/highlights.scm"),
    "\n",
    include_str!("../grammars/sql/queries/highlights-postgres.scm")
);
const SQL_MYSQL_HIGHLIGHTS_QUERY: &str = concat!(
    include_str!("../grammars/sql/queries/highlights.scm"),
    "\n",
    include_str!("../grammars/sql/queries/highlights-mysql.scm")
);
const SQL_SQLITE_HIGHLIGHTS_QUERY: &str = concat!(
    include_str!("../grammars/sql/queries/highlights.scm"),
    "\n",
    include_str!("../grammars/sql/queries/highlights-sqlite.scm")
);
const JSON_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/json/queries/highlights.scm");
const IGNORE_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/ignore/queries/highlights.scm");
const DOCKERFILE_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/dockerfile/queries/highlights.scm");
const DOCKERFILE_INJECTIONS_QUERY: &str =
    include_str!("../grammars/dockerfile/queries/injections.scm");
const BASH_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/bash/queries/highlights.scm");
const BASH_INJECTIONS_QUERY: &str = include_str!("../grammars/bash/queries/injections.scm");
const FISH_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/fish/queries/highlights.scm");
const ZSH_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/zsh/queries/highlights.scm");
const ZSH_INJECTIONS_QUERY: &str = include_str!("../grammars/zsh/queries/injections.scm");
const POWERSHELL_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/powershell/queries/highlights.scm");
const BATCH_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/batch/queries/highlights.scm");
const TOML_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/toml/queries/highlights.scm");
const YAML_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/yaml/queries/highlights.scm");
const YAML_INJECTIONS_QUERY: &str = include_str!("../grammars/yaml/queries/injections.scm");
const HCL_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/hcl/queries/highlights.scm");
const RUST_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/rust/queries/highlights.scm");
const RUST_INJECTIONS_QUERY: &str = include_str!("../grammars/rust/queries/injections.scm");
const PYTHON_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/python/queries/highlights.scm");
const PYTHON_INJECTIONS_QUERY: &str = include_str!("../grammars/python/queries/injections.scm");
const GO_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/go/queries/highlights.scm");
const GO_INJECTIONS_QUERY: &str = include_str!("../grammars/go/queries/injections.scm");
const GOMOD_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/gomod/queries/highlights.scm");
const GOWORK_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/gowork/queries/highlights.scm");
const GOSUM_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/gosum/queries/highlights.scm");
const SQL_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/sql/queries/highlights.scm");
const HTML_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/html/queries/highlights.scm");
const HTML_INJECTIONS_QUERY: &str = include_str!("../grammars/html/queries/injections.scm");
const CSS_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/css/queries/highlights.scm");
const JAVASCRIPT_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/javascript/queries/highlights.scm");
const JAVASCRIPT_INJECTIONS_QUERY: &str =
    include_str!("../grammars/javascript/queries/injections.scm");
const JAVASCRIPT_LOCALS_QUERY: &str = include_str!("../grammars/javascript/queries/locals.scm");
const GRAPHQL_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/graphql/queries/highlights.scm");
const PROTO_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/proto/queries/highlights.scm");
const TEXTPROTO_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/textproto/queries/highlights.scm");
const REGEX_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/regex/queries/highlights.scm");
const REGEX_JAVASCRIPT_HIGHLIGHTS_QUERY: &str = concat!(
    include_str!("../grammars/regex/queries/highlights.scm"),
    "\n",
    include_str!("../grammars/regex/queries/highlights-javascript.scm")
);
const REGEX_PYTHON_HIGHLIGHTS_QUERY: &str = concat!(
    include_str!("../grammars/regex/queries/highlights.scm"),
    "\n",
    include_str!("../grammars/regex/queries/highlights-python.scm")
);
const REGEX_RUST_HIGHLIGHTS_QUERY: &str = concat!(
    include_str!("../grammars/regex/queries/highlights.scm"),
    "\n",
    include_str!("../grammars/regex/queries/highlights-rust.scm")
);
const REGEX_GO_HIGHLIGHTS_QUERY: &str = concat!(
    include_str!("../grammars/regex/queries/highlights.scm"),
    "\n",
    include_str!("../grammars/regex/queries/highlights-go.scm")
);
const REGEX_POSIX_HIGHLIGHTS_QUERY: &str = concat!(
    include_str!("../grammars/regex/queries/highlights.scm"),
    "\n",
    include_str!("../grammars/regex/queries/highlights-posix.scm")
);
const JSDOC_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/jsdoc/queries/highlights.scm");
const USERSCRIPT_METADATA_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/userscript_metadata/queries/highlights.scm");
const MARKDOWN_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/markdown/queries/highlights.scm");
const MARKDOWN_INJECTIONS_QUERY: &str = include_str!("../grammars/markdown/queries/injections.scm");
const MARKDOWN_INLINE_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/markdown_inline/queries/highlights.scm");
const MARKDOWN_INLINE_INJECTIONS_QUERY: &str =
    include_str!("../grammars/markdown_inline/queries/injections.scm");
const JUST_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/just/queries/highlights.scm");
const JUST_INJECTIONS_QUERY: &str = include_str!("../grammars/just/queries/injections.scm");

unsafe extern "C" {
    fn tree_sitter_json() -> *const ();
    fn tree_sitter_ignore() -> *const ();
    fn tree_sitter_dockerfile() -> *const ();
    fn tree_sitter_bash() -> *const ();
    fn tree_sitter_fish() -> *const ();
    fn tree_sitter_zsh() -> *const ();
    fn tree_sitter_powershell() -> *const ();
    fn tree_sitter_batch() -> *const ();
    fn tree_sitter_toml() -> *const ();
    fn tree_sitter_yaml() -> *const ();
    fn tree_sitter_hcl() -> *const ();
    fn tree_sitter_rust() -> *const ();
    fn tree_sitter_python() -> *const ();
    fn tree_sitter_go() -> *const ();
    fn tree_sitter_gomod() -> *const ();
    fn tree_sitter_gowork() -> *const ();
    fn tree_sitter_gosum() -> *const ();
    fn tree_sitter_html() -> *const ();
    fn tree_sitter_css() -> *const ();
    fn tree_sitter_javascript() -> *const ();
    fn tree_sitter_graphql() -> *const ();
    fn tree_sitter_proto() -> *const ();
    fn tree_sitter_textproto() -> *const ();
    fn tree_sitter_regex() -> *const ();
    fn tree_sitter_jsdoc() -> *const ();
    fn tree_sitter_userscript_metadata() -> *const ();
    fn tree_sitter_markdown() -> *const ();
    fn tree_sitter_markdown_inline() -> *const ();
    fn tree_sitter_just() -> *const ();
}

const JSON_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_json) };
const IGNORE_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_ignore) };
const DOCKERFILE_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_dockerfile) };
const BASH_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_bash) };
const FISH_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_fish) };
const ZSH_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_zsh) };
const POWERSHELL_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_powershell) };
const BATCH_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_batch) };
const TOML_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_toml) };
const YAML_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_yaml) };
const HCL_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_hcl) };
const RUST_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_rust) };
const PYTHON_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_python) };
const GO_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_go) };
const GOMOD_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_gomod) };
const GOWORK_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_gowork) };
const GOSUM_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_gosum) };
const SQL_LANGUAGE: LanguageFn = tree_sitter_sequel::LANGUAGE;
const HTML_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_html) };
const CSS_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_css) };
const JAVASCRIPT_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_javascript) };
const GRAPHQL_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_graphql) };
const PROTO_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_proto) };
const TEXTPROTO_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_textproto) };
const REGEX_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_regex) };
const JSDOC_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_jsdoc) };
const USERSCRIPT_METADATA_LANGUAGE: LanguageFn =
    unsafe { LanguageFn::from_raw(tree_sitter_userscript_metadata) };
const MARKDOWN_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_markdown) };
const MARKDOWN_INLINE_LANGUAGE: LanguageFn =
    unsafe { LanguageFn::from_raw(tree_sitter_markdown_inline) };
const JUST_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_just) };

struct StaticLanguageAsset {
    name: &'static str,
    language_fn: LanguageFn,
    highlights_query: &'static str,
    injections_query: &'static str,
    locals_query: &'static str,
}

pub struct LanguageRuntime {
    pub language: Language,
    pub flat_configuration: HighlightConfiguration,
    pub injections_query: &'static str,
}

pub static RUNTIMES: LazyLock<BTreeMap<&'static str, LanguageRuntime>> = LazyLock::new(|| {
    build_runtimes()
        .unwrap_or_else(|error| panic!("failed to initialize language runtimes: {error:#}"))
});

const STATIC_LANGUAGE_ASSETS: &[StaticLanguageAsset] = &[
    StaticLanguageAsset {
        name: "json",
        language_fn: JSON_LANGUAGE,
        highlights_query: JSON_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "ignore",
        language_fn: IGNORE_LANGUAGE,
        highlights_query: IGNORE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "dockerfile",
        language_fn: DOCKERFILE_LANGUAGE,
        highlights_query: DOCKERFILE_HIGHLIGHTS_QUERY,
        injections_query: DOCKERFILE_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "bash",
        language_fn: BASH_LANGUAGE,
        highlights_query: BASH_HIGHLIGHTS_QUERY,
        injections_query: BASH_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "fish",
        language_fn: FISH_LANGUAGE,
        highlights_query: FISH_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "zsh",
        language_fn: ZSH_LANGUAGE,
        highlights_query: ZSH_HIGHLIGHTS_QUERY,
        injections_query: ZSH_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "powershell",
        language_fn: POWERSHELL_LANGUAGE,
        highlights_query: POWERSHELL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "batch",
        language_fn: BATCH_LANGUAGE,
        highlights_query: BATCH_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "toml",
        language_fn: TOML_LANGUAGE,
        highlights_query: TOML_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "yaml",
        language_fn: YAML_LANGUAGE,
        highlights_query: YAML_HIGHLIGHTS_QUERY,
        injections_query: YAML_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "hcl",
        language_fn: HCL_LANGUAGE,
        highlights_query: HCL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "rust",
        language_fn: RUST_LANGUAGE,
        highlights_query: RUST_HIGHLIGHTS_QUERY,
        injections_query: RUST_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "python",
        language_fn: PYTHON_LANGUAGE,
        highlights_query: PYTHON_HIGHLIGHTS_QUERY,
        injections_query: PYTHON_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "go",
        language_fn: GO_LANGUAGE,
        highlights_query: GO_HIGHLIGHTS_QUERY,
        injections_query: GO_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "gomod",
        language_fn: GOMOD_LANGUAGE,
        highlights_query: GOMOD_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "gowork",
        language_fn: GOWORK_LANGUAGE,
        highlights_query: GOWORK_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "gosum",
        language_fn: GOSUM_LANGUAGE,
        highlights_query: GOSUM_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "sql",
        language_fn: SQL_LANGUAGE,
        highlights_query: SQL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "sql_postgres",
        language_fn: SQL_LANGUAGE,
        highlights_query: SQL_POSTGRES_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "sql_mysql",
        language_fn: SQL_LANGUAGE,
        highlights_query: SQL_MYSQL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "sql_sqlite",
        language_fn: SQL_LANGUAGE,
        highlights_query: SQL_SQLITE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "html",
        language_fn: HTML_LANGUAGE,
        highlights_query: HTML_HIGHLIGHTS_QUERY,
        injections_query: HTML_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "css",
        language_fn: CSS_LANGUAGE,
        highlights_query: CSS_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "javascript",
        language_fn: JAVASCRIPT_LANGUAGE,
        highlights_query: JAVASCRIPT_HIGHLIGHTS_QUERY,
        injections_query: JAVASCRIPT_INJECTIONS_QUERY,
        locals_query: JAVASCRIPT_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "graphql",
        language_fn: GRAPHQL_LANGUAGE,
        highlights_query: GRAPHQL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "proto",
        language_fn: PROTO_LANGUAGE,
        highlights_query: PROTO_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "textproto",
        language_fn: TEXTPROTO_LANGUAGE,
        highlights_query: TEXTPROTO_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "regex",
        language_fn: REGEX_LANGUAGE,
        highlights_query: REGEX_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "regex_javascript",
        language_fn: REGEX_LANGUAGE,
        highlights_query: REGEX_JAVASCRIPT_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "regex_python",
        language_fn: REGEX_LANGUAGE,
        highlights_query: REGEX_PYTHON_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "regex_rust",
        language_fn: REGEX_LANGUAGE,
        highlights_query: REGEX_RUST_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "regex_go",
        language_fn: REGEX_LANGUAGE,
        highlights_query: REGEX_GO_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "regex_posix",
        language_fn: REGEX_LANGUAGE,
        highlights_query: REGEX_POSIX_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "jsdoc",
        language_fn: JSDOC_LANGUAGE,
        highlights_query: JSDOC_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "userscript_metadata",
        language_fn: USERSCRIPT_METADATA_LANGUAGE,
        highlights_query: USERSCRIPT_METADATA_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "markdown",
        language_fn: MARKDOWN_LANGUAGE,
        highlights_query: MARKDOWN_HIGHLIGHTS_QUERY,
        injections_query: MARKDOWN_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "markdown_inline",
        language_fn: MARKDOWN_INLINE_LANGUAGE,
        highlights_query: MARKDOWN_INLINE_HIGHLIGHTS_QUERY,
        injections_query: MARKDOWN_INLINE_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "just",
        language_fn: JUST_LANGUAGE,
        highlights_query: JUST_HIGHLIGHTS_QUERY,
        injections_query: JUST_INJECTIONS_QUERY,
        locals_query: "",
    },
];

pub fn runtime(name: &str) -> Option<&'static LanguageRuntime> {
    RUNTIMES.get(name)
}

pub fn global_highlight_name(highlight_index: usize) -> &'static str {
    HIGHLIGHT_NAMES[highlight_index]
}

fn build_runtimes() -> Result<BTreeMap<&'static str, LanguageRuntime>> {
    let mut runtimes = BTreeMap::new();

    for asset in STATIC_LANGUAGE_ASSETS {
        let language = Language::from(asset.language_fn);
        let mut flat_configuration = HighlightConfiguration::new(
            language.clone(),
            asset.name,
            asset.highlights_query,
            "",
            asset.locals_query,
        )
        .with_context(|| {
            format!(
                "failed to build flat highlight configuration for {}",
                asset.name
            )
        })?;
        flat_configuration.configure(&*HIGHLIGHT_NAMES);

        runtimes.insert(
            asset.name,
            LanguageRuntime {
                language,
                flat_configuration,
                injections_query: asset.injections_query,
            },
        );
    }

    Ok(runtimes)
}
