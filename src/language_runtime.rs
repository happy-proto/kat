use std::{
    collections::BTreeMap,
    sync::{LazyLock, OnceLock},
    time::Instant,
};

use anyhow::{Context, Result};
use tree_sitter::{Language, Query};
use tree_sitter_highlight::HighlightConfiguration;
use tree_sitter_language::LanguageFn;

use crate::debug_progress::log as progress_log;
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
const QUERY_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/query/queries/highlights.scm");
const IGNORE_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/ignore/queries/highlights.scm");
const GIT_CONFIG_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/git_config/queries/highlights.scm");
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
const C_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/c/queries/highlights.scm");
const CPP_HIGHLIGHTS_QUERY: &str = concat!(
    include_str!("../grammars/c/queries/highlights.scm"),
    "\n",
    include_str!("../grammars/cpp/queries/highlights.scm")
);
const CPP_INJECTIONS_QUERY: &str = include_str!("../grammars/cpp/queries/injections.scm");
const JAVA_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/java/queries/highlights.scm");
const KOTLIN_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/kotlin/queries/highlights.scm");
const CSHARP_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/csharp/queries/highlights.scm");
const GROOVY_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/groovy/queries/highlights.scm");
const DIFF_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/diff/queries/highlights.scm");
const PROPERTIES_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/properties/queries/highlights.scm");
const PHP_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/php/queries/highlights.scm");
const PHP_INJECTIONS_QUERY: &str = concat!(
    include_str!("../grammars/php/queries/injections.scm"),
    "\n",
    include_str!("../grammars/php/queries/injections-text.scm")
);
const SCALA_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/scala/queries/highlights.scm");
const SCALA_LOCALS_QUERY: &str = include_str!("../grammars/scala/queries/locals.scm");
const SWIFT_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/swift/queries/highlights.scm");
const SWIFT_INJECTIONS_QUERY: &str = include_str!("../grammars/swift/queries/injections.scm");
const SWIFT_LOCALS_QUERY: &str = include_str!("../grammars/swift/queries/locals.scm");
const DART_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/dart/queries/highlights.scm");
const DART_LOCALS_QUERY: &str = include_str!("../grammars/dart/queries/locals.scm");
const ELIXIR_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/elixir/queries/highlights.scm");
const ELIXIR_INJECTIONS_QUERY: &str = include_str!("../grammars/elixir/queries/injections.scm");
const ZIG_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/zig/queries/highlights.scm");
const ZIG_INJECTIONS_QUERY: &str = include_str!("../grammars/zig/queries/injections.scm");
const ZIG_LOCALS_QUERY: &str = include_str!("../grammars/zig/queries/locals.scm");
const SSH_CONFIG_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/ssh_config/queries/highlights.scm");
const SSH_CONFIG_INJECTIONS_QUERY: &str =
    include_str!("../grammars/ssh_config/queries/injections.scm");
const GITATTRIBUTES_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/gitattributes/queries/highlights.scm");
const GIT_COMMIT_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/git_commit/queries/highlights.scm");
const GIT_REBASE_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/git_rebase/queries/highlights.scm");
const REQUIREMENTS_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/requirements/queries/highlights.scm");
const APACHE_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/apache/queries/highlights.scm");
const SCSS_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/scss/queries/highlights.scm");
const SASS_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/sass/queries/highlights.scm");
const JQ_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/jq/queries/highlights.scm");
const LESS_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/less/queries/highlights.scm");
const DOT_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/dot/queries/highlights.scm");
const DOT_INJECTIONS_QUERY: &str = include_str!("../grammars/dot/queries/injections.scm");
const NGINX_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/nginx/queries/highlights.scm");
const TYPESCRIPT_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/typescript/queries/highlights.scm");
const TYPESCRIPT_INJECTIONS_QUERY: &str =
    include_str!("../grammars/typescript/queries/injections.scm");
const TYPESCRIPT_LOCALS_QUERY: &str = include_str!("../grammars/typescript/queries/locals.scm");
const TSX_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/tsx/queries/highlights.scm");
const TSX_INJECTIONS_QUERY: &str = include_str!("../grammars/tsx/queries/injections.scm");
const TSX_LOCALS_QUERY: &str = include_str!("../grammars/tsx/queries/locals.scm");
const GO_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/go/queries/highlights.scm");
const GO_INJECTIONS_QUERY: &str = include_str!("../grammars/go/queries/injections.scm");
const GOMOD_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/gomod/queries/highlights.scm");
const GOWORK_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/gowork/queries/highlights.scm");
const GOSUM_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/gosum/queries/highlights.scm");
const SQL_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/sql/queries/highlights.scm");
const HTML_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/html/queries/highlights.scm");
const HTML_INJECTIONS_QUERY: &str = include_str!("../grammars/html/queries/injections.scm");
const VUE_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/vue/queries/highlights.scm");
const VUE_INJECTIONS_QUERY: &str = include_str!("../grammars/vue/queries/injections.scm");
const SVELTE_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/svelte/queries/highlights.scm");
const SVELTE_INJECTIONS_QUERY: &str = include_str!("../grammars/svelte/queries/injections.scm");
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
const RUBY_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/ruby/queries/highlights.scm");
const RUBY_LOCALS_QUERY: &str = include_str!("../grammars/ruby/queries/locals.scm");
const LUA_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/lua/queries/highlights.scm");
const LUA_INJECTIONS_QUERY: &str = include_str!("../grammars/lua/queries/injections.scm");
const LUA_LOCALS_QUERY: &str = include_str!("../grammars/lua/queries/locals.scm");
const NIX_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/nix/queries/highlights.scm");
const NIX_INJECTIONS_QUERY: &str = include_str!("../grammars/nix/queries/injections.scm");
const DOTENV_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/dotenv/queries/highlights.scm");
const INI_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/ini/queries/highlights.scm");
const XML_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/xml/queries/highlights.scm");
const MAKE_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/make/queries/highlights.scm");
const CMAKE_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/cmake/queries/highlights.scm");
const NINJA_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/ninja/queries/highlights.scm");
const JINJA_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/jinja/queries/highlights.scm");
const JINJA_INJECTIONS_QUERY: &str = include_str!("../grammars/jinja/queries/injections.scm");
const TWIG_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/twig/queries/highlights.scm");
const TWIG_INJECTIONS_QUERY: &str = include_str!("../grammars/twig/queries/injections.scm");
const ERB_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/erb/queries/highlights.scm");
const ERB_INJECTIONS_QUERY: &str = include_str!("../grammars/erb/queries/injections.scm");

unsafe extern "C" {
    fn tree_sitter_ignore() -> *const ();
    fn tree_sitter_query() -> *const ();
    fn tree_sitter_git_config() -> *const ();
    fn tree_sitter_dockerfile() -> *const ();
    fn tree_sitter_fish() -> *const ();
    fn tree_sitter_hcl() -> *const ();
    fn tree_sitter_gomod() -> *const ();
    fn tree_sitter_gowork() -> *const ();
    fn tree_sitter_gosum() -> *const ();
    fn tree_sitter_vue() -> *const ();
    fn tree_sitter_svelte() -> *const ();
    fn tree_sitter_scss() -> *const ();
    fn tree_sitter_graphql() -> *const ();
    fn tree_sitter_proto() -> *const ();
    fn tree_sitter_textproto() -> *const ();
    fn tree_sitter_userscript_metadata() -> *const ();
    fn tree_sitter_markdown() -> *const ();
    fn tree_sitter_markdown_inline() -> *const ();
    fn tree_sitter_just() -> *const ();
    fn tree_sitter_dotenv() -> *const ();
    fn tree_sitter_ninja() -> *const ();
    fn tree_sitter_jinja() -> *const ();
    fn tree_sitter_twig() -> *const ();
    fn tree_sitter_jq() -> *const ();
    fn tree_sitter_less() -> *const ();
    fn tree_sitter_dot() -> *const ();
    fn tree_sitter_ssh_config() -> *const ();
    fn tree_sitter_gitattributes() -> *const ();
    fn tree_sitter_git_commit() -> *const ();
    fn tree_sitter_git_rebase() -> *const ();
    fn tree_sitter_requirements() -> *const ();
    fn tree_sitter_apache() -> *const ();
    fn tree_sitter_sass() -> *const ();
}

const JSON_LANGUAGE: LanguageFn = tree_sitter_json::LANGUAGE;
const QUERY_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_query) };
const IGNORE_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_ignore) };
const GIT_CONFIG_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_git_config) };
const DOCKERFILE_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_dockerfile) };
const BASH_LANGUAGE: LanguageFn = tree_sitter_bash::LANGUAGE;
const FISH_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_fish) };
const ZSH_LANGUAGE: LanguageFn = tree_sitter_zsh::LANGUAGE;
const POWERSHELL_LANGUAGE: LanguageFn = tree_sitter_powershell::LANGUAGE;
const BATCH_LANGUAGE: LanguageFn = tree_sitter_batch::LANGUAGE;
const TOML_LANGUAGE: LanguageFn = tree_sitter_toml_ng::LANGUAGE;
const YAML_LANGUAGE: LanguageFn = tree_sitter_yaml::LANGUAGE;
const HCL_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_hcl) };
const RUST_LANGUAGE: LanguageFn = tree_sitter_rust::LANGUAGE;
const PYTHON_LANGUAGE: LanguageFn = tree_sitter_python::LANGUAGE;
const C_LANGUAGE: LanguageFn = tree_sitter_c::LANGUAGE;
const CPP_LANGUAGE: LanguageFn = tree_sitter_cpp::LANGUAGE;
const JAVA_LANGUAGE: LanguageFn = tree_sitter_java::LANGUAGE;
const KOTLIN_LANGUAGE: LanguageFn = tree_sitter_kotlin_ng::LANGUAGE;
const CSHARP_LANGUAGE: LanguageFn = tree_sitter_c_sharp::LANGUAGE;
const GROOVY_LANGUAGE: LanguageFn = tree_sitter_groovy::LANGUAGE;
const DIFF_LANGUAGE: LanguageFn = tree_sitter_diff::LANGUAGE;
const PROPERTIES_LANGUAGE: LanguageFn = tree_sitter_properties::LANGUAGE;
const PHP_LANGUAGE: LanguageFn = tree_sitter_php::LANGUAGE_PHP;
const SCALA_LANGUAGE: LanguageFn = tree_sitter_scala::LANGUAGE;
const SWIFT_LANGUAGE: LanguageFn = tree_sitter_swift::LANGUAGE;
const DART_LANGUAGE: LanguageFn = tree_sitter_dart::LANGUAGE;
const ELIXIR_LANGUAGE: LanguageFn = tree_sitter_elixir::LANGUAGE;
const ZIG_LANGUAGE: LanguageFn = tree_sitter_zig::LANGUAGE;
const SCSS_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_scss) };
const SSH_CONFIG_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_ssh_config) };
const GITATTRIBUTES_LANGUAGE: LanguageFn =
    unsafe { LanguageFn::from_raw(tree_sitter_gitattributes) };
const GIT_COMMIT_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_git_commit) };
const GIT_REBASE_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_git_rebase) };
const REQUIREMENTS_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_requirements) };
const APACHE_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_apache) };
const SASS_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_sass) };
const JQ_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_jq) };
const LESS_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_less) };
const DOT_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_dot) };
const NGINX_LANGUAGE: LanguageFn = tree_sitter_nginx::LANGUAGE;
const TYPESCRIPT_LANGUAGE: LanguageFn = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
const TSX_LANGUAGE: LanguageFn = tree_sitter_typescript::LANGUAGE_TSX;
const GO_LANGUAGE: LanguageFn = tree_sitter_go::LANGUAGE;
const GOMOD_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_gomod) };
const GOWORK_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_gowork) };
const GOSUM_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_gosum) };
const SQL_LANGUAGE: LanguageFn = tree_sitter_sequel::LANGUAGE;
const HTML_LANGUAGE: LanguageFn = tree_sitter_html::LANGUAGE;
const VUE_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_vue) };
const SVELTE_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_svelte) };
const CSS_LANGUAGE: LanguageFn = tree_sitter_css::LANGUAGE;
const JAVASCRIPT_LANGUAGE: LanguageFn = tree_sitter_javascript::LANGUAGE;
const GRAPHQL_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_graphql) };
const PROTO_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_proto) };
const TEXTPROTO_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_textproto) };
const REGEX_LANGUAGE: LanguageFn = tree_sitter_regex::LANGUAGE;
const JSDOC_LANGUAGE: LanguageFn = tree_sitter_jsdoc::LANGUAGE;
const USERSCRIPT_METADATA_LANGUAGE: LanguageFn =
    unsafe { LanguageFn::from_raw(tree_sitter_userscript_metadata) };
const MARKDOWN_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_markdown) };
const MARKDOWN_INLINE_LANGUAGE: LanguageFn =
    unsafe { LanguageFn::from_raw(tree_sitter_markdown_inline) };
const JUST_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_just) };
const RUBY_LANGUAGE: LanguageFn = tree_sitter_ruby::LANGUAGE;
const LUA_LANGUAGE: LanguageFn = tree_sitter_lua::LANGUAGE;
const NIX_LANGUAGE: LanguageFn = tree_sitter_nix::LANGUAGE;
const DOTENV_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_dotenv) };
const INI_LANGUAGE: LanguageFn = tree_sitter_ini::LANGUAGE;
const XML_LANGUAGE: LanguageFn = tree_sitter_xml::LANGUAGE_XML;
const MAKE_LANGUAGE: LanguageFn = tree_sitter_make::LANGUAGE;
const CMAKE_LANGUAGE: LanguageFn = tree_sitter_cmake::LANGUAGE;
const NINJA_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_ninja) };
const JINJA_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_jinja) };
const TWIG_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_twig) };
const ERB_LANGUAGE: LanguageFn = tree_sitter_embedded_template::LANGUAGE;

#[derive(Clone, Copy)]
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
    pub injections_query: Option<Query>,
}

const STATIC_LANGUAGE_ASSETS: &[StaticLanguageAsset] = &[
    StaticLanguageAsset {
        name: "json",
        language_fn: JSON_LANGUAGE,
        highlights_query: JSON_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "query",
        language_fn: QUERY_LANGUAGE,
        highlights_query: QUERY_HIGHLIGHTS_QUERY,
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
        name: "git_config",
        language_fn: GIT_CONFIG_LANGUAGE,
        highlights_query: GIT_CONFIG_HIGHLIGHTS_QUERY,
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
        name: "c",
        language_fn: C_LANGUAGE,
        highlights_query: C_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "cpp",
        language_fn: CPP_LANGUAGE,
        highlights_query: CPP_HIGHLIGHTS_QUERY,
        injections_query: CPP_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "java",
        language_fn: JAVA_LANGUAGE,
        highlights_query: JAVA_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "kotlin",
        language_fn: KOTLIN_LANGUAGE,
        highlights_query: KOTLIN_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "csharp",
        language_fn: CSHARP_LANGUAGE,
        highlights_query: CSHARP_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "groovy",
        language_fn: GROOVY_LANGUAGE,
        highlights_query: GROOVY_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "diff",
        language_fn: DIFF_LANGUAGE,
        highlights_query: DIFF_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "properties",
        language_fn: PROPERTIES_LANGUAGE,
        highlights_query: PROPERTIES_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "php",
        language_fn: PHP_LANGUAGE,
        highlights_query: PHP_HIGHLIGHTS_QUERY,
        injections_query: PHP_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "scala",
        language_fn: SCALA_LANGUAGE,
        highlights_query: SCALA_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: SCALA_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "swift",
        language_fn: SWIFT_LANGUAGE,
        highlights_query: SWIFT_HIGHLIGHTS_QUERY,
        injections_query: SWIFT_INJECTIONS_QUERY,
        locals_query: SWIFT_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "dart",
        language_fn: DART_LANGUAGE,
        highlights_query: DART_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: DART_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "elixir",
        language_fn: ELIXIR_LANGUAGE,
        highlights_query: ELIXIR_HIGHLIGHTS_QUERY,
        injections_query: ELIXIR_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "zig",
        language_fn: ZIG_LANGUAGE,
        highlights_query: ZIG_HIGHLIGHTS_QUERY,
        injections_query: ZIG_INJECTIONS_QUERY,
        locals_query: ZIG_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "ssh_config",
        language_fn: SSH_CONFIG_LANGUAGE,
        highlights_query: SSH_CONFIG_HIGHLIGHTS_QUERY,
        injections_query: SSH_CONFIG_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "gitattributes",
        language_fn: GITATTRIBUTES_LANGUAGE,
        highlights_query: GITATTRIBUTES_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "git_commit",
        language_fn: GIT_COMMIT_LANGUAGE,
        highlights_query: GIT_COMMIT_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "git_rebase",
        language_fn: GIT_REBASE_LANGUAGE,
        highlights_query: GIT_REBASE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "requirements",
        language_fn: REQUIREMENTS_LANGUAGE,
        highlights_query: REQUIREMENTS_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "apache",
        language_fn: APACHE_LANGUAGE,
        highlights_query: APACHE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "scss",
        language_fn: SCSS_LANGUAGE,
        highlights_query: SCSS_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "sass",
        language_fn: SASS_LANGUAGE,
        highlights_query: SASS_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "jq",
        language_fn: JQ_LANGUAGE,
        highlights_query: JQ_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "less",
        language_fn: LESS_LANGUAGE,
        highlights_query: LESS_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "dot",
        language_fn: DOT_LANGUAGE,
        highlights_query: DOT_HIGHLIGHTS_QUERY,
        injections_query: DOT_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "nginx",
        language_fn: NGINX_LANGUAGE,
        highlights_query: NGINX_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "typescript",
        language_fn: TYPESCRIPT_LANGUAGE,
        highlights_query: TYPESCRIPT_HIGHLIGHTS_QUERY,
        injections_query: TYPESCRIPT_INJECTIONS_QUERY,
        locals_query: TYPESCRIPT_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "tsx",
        language_fn: TSX_LANGUAGE,
        highlights_query: TSX_HIGHLIGHTS_QUERY,
        injections_query: TSX_INJECTIONS_QUERY,
        locals_query: TSX_LOCALS_QUERY,
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
        name: "vue",
        language_fn: VUE_LANGUAGE,
        highlights_query: VUE_HIGHLIGHTS_QUERY,
        injections_query: VUE_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "svelte",
        language_fn: SVELTE_LANGUAGE,
        highlights_query: SVELTE_HIGHLIGHTS_QUERY,
        injections_query: SVELTE_INJECTIONS_QUERY,
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
    StaticLanguageAsset {
        name: "ruby",
        language_fn: RUBY_LANGUAGE,
        highlights_query: RUBY_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: RUBY_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "lua",
        language_fn: LUA_LANGUAGE,
        highlights_query: LUA_HIGHLIGHTS_QUERY,
        injections_query: LUA_INJECTIONS_QUERY,
        locals_query: LUA_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "nix",
        language_fn: NIX_LANGUAGE,
        highlights_query: NIX_HIGHLIGHTS_QUERY,
        injections_query: NIX_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "dotenv",
        language_fn: DOTENV_LANGUAGE,
        highlights_query: DOTENV_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "ini",
        language_fn: INI_LANGUAGE,
        highlights_query: INI_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "xml",
        language_fn: XML_LANGUAGE,
        highlights_query: XML_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "make",
        language_fn: MAKE_LANGUAGE,
        highlights_query: MAKE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "cmake",
        language_fn: CMAKE_LANGUAGE,
        highlights_query: CMAKE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "ninja",
        language_fn: NINJA_LANGUAGE,
        highlights_query: NINJA_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "jinja",
        language_fn: JINJA_LANGUAGE,
        highlights_query: JINJA_HIGHLIGHTS_QUERY,
        injections_query: JINJA_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "twig",
        language_fn: TWIG_LANGUAGE,
        highlights_query: TWIG_HIGHLIGHTS_QUERY,
        injections_query: TWIG_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "erb",
        language_fn: ERB_LANGUAGE,
        highlights_query: ERB_HIGHLIGHTS_QUERY,
        injections_query: ERB_INJECTIONS_QUERY,
        locals_query: "",
    },
];

static LANGUAGE_ASSETS_BY_NAME: LazyLock<BTreeMap<&'static str, &'static StaticLanguageAsset>> =
    LazyLock::new(|| {
        STATIC_LANGUAGE_ASSETS
            .iter()
            .map(|asset| (asset.name, asset))
            .collect()
    });

static RUNTIME_SLOTS: LazyLock<BTreeMap<&'static str, OnceLock<LanguageRuntime>>> =
    LazyLock::new(|| {
        STATIC_LANGUAGE_ASSETS
            .iter()
            .map(|asset| (asset.name, OnceLock::new()))
            .collect()
    });

pub fn runtime(name: &str) -> Option<&'static LanguageRuntime> {
    let asset = *LANGUAGE_ASSETS_BY_NAME.get(name)?;
    let slot = RUNTIME_SLOTS.get(name)?;

    Some(slot.get_or_init(|| {
        build_runtime(*asset).unwrap_or_else(|error| {
            panic!(
                "failed to initialize language runtime {}: {error:#}",
                asset.name
            )
        })
    }))
}

pub fn supports_runtime(name: &str) -> bool {
    LANGUAGE_ASSETS_BY_NAME.contains_key(name)
}

pub fn global_highlight_name(highlight_index: usize) -> &'static str {
    HIGHLIGHT_NAMES[highlight_index]
}

fn build_runtime(asset: StaticLanguageAsset) -> Result<LanguageRuntime> {
    progress_log("runtime_init", format!("begin runtime={}", asset.name));
    let started_at = Instant::now();

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
    flat_configuration.configure(&HIGHLIGHT_NAMES);

    let runtime = LanguageRuntime {
        language: language.clone(),
        flat_configuration,
        injections_query: if asset.injections_query.trim().is_empty() {
            None
        } else {
            Some(
                Query::new(&language, asset.injections_query).with_context(|| {
                    format!("failed to build injections query for {}", asset.name)
                })?,
            )
        },
    };

    progress_log(
        "runtime_init",
        format!(
            "done runtime={} elapsed={:.3}ms",
            asset.name,
            started_at.elapsed().as_secs_f64() * 1_000.0
        ),
    );

    Ok(runtime)
}

#[cfg(test)]
mod tests {
    use super::runtime;

    #[test]
    fn caches_typescript_injections_query_when_runtime_is_requested() {
        let typescript = runtime("typescript").expect("missing typescript runtime");
        assert!(
            typescript.injections_query.is_some(),
            "expected TypeScript runtime to cache its injections query"
        );
    }

    #[test]
    fn omits_cached_injections_query_for_plain_runtime() {
        let json = runtime("json").expect("missing json runtime");
        assert!(
            json.injections_query.is_none(),
            "expected JSON runtime to skip empty injections query compilation"
        );
    }
}
