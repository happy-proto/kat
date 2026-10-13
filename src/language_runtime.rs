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
const GIT_LINK_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/git_link/queries/highlights.scm");
const GIT_MAILMAP_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/git_mailmap/queries/highlights.scm");
const GIT_LOG_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/git_log/queries/highlights.scm");
const GIT_LOG_INJECTIONS_QUERY: &str = include_str!("../grammars/git_log/queries/injections.scm");
const GIT_CONFIG_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/git_config/queries/highlights.scm");
const DOCKERFILE_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/dockerfile/queries/highlights.scm");
const DOCKERFILE_INJECTIONS_QUERY: &str =
    include_str!("../grammars/dockerfile/queries/injections.scm");
const ACTIONSCRIPT_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/actionscript/queries/highlights.scm");
const ADA_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/ada/queries/highlights.scm");
const ADA_LOCALS_QUERY: &str = include_str!("../grammars/ada/queries/locals.scm");
const APPLESCRIPT_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/applescript/queries/highlights.scm");
const ASM_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/asm/queries/highlights.scm");
const NASM_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/nasm/queries/highlights.scm");
const ASCIIDOC_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/asciidoc/queries/highlights.scm");
const AUTHORIZED_KEYS_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/authorized_keys/queries/highlights.scm");
const KNOWN_HOSTS_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/known_hosts/queries/highlights.scm");
const AWK_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/awk/queries/highlights.scm");
const BASH_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/bash/queries/highlights.scm");
const BASH_INJECTIONS_QUERY: &str = include_str!("../grammars/bash/queries/injections.scm");
const BIBTEX_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/bibtex/queries/highlights.scm");
const BIBTEX_LOCALS_QUERY: &str = include_str!("../grammars/bibtex/queries/locals.scm");
const CABAL_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/cabal/queries/highlights.scm");
const CFML_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/cfml/queries/highlights.scm");
const CFML_INJECTIONS_QUERY: &str = include_str!("../grammars/cfml/queries/injections.scm");
const CLOJURE_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/clojure/queries/highlights.scm");
const CMAKECACHE_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/cmakecache/queries/highlights.scm");
const COFFEESCRIPT_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/coffeescript/queries/highlights.scm");
const COFFEESCRIPT_INJECTIONS_QUERY: &str =
    include_str!("../grammars/coffeescript/queries/injections.scm");
const COMMAND_HELP_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/command_help/queries/highlights.scm");
const CPUINFO_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/cpuinfo/queries/highlights.scm");
const CRONTAB_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/crontab/queries/highlights.scm");
const CRYSTAL_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/crystal/queries/highlights.scm");
const CRYSTAL_INJECTIONS_QUERY: &str = include_str!("../grammars/crystal/queries/injections.scm");
const D_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/d/queries/highlights.scm");
const D_INJECTIONS_QUERY: &str = include_str!("../grammars/d/queries/injections.scm");
const DEBSOURCES_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/debsources/queries/highlights.scm");
const ELM_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/elm/queries/highlights.scm");
const ELM_INJECTIONS_QUERY: &str = include_str!("../grammars/elm/queries/injections.scm");
const ELM_LOCALS_QUERY: &str = include_str!("../grammars/elm/queries/locals.scm");
const EMAIL_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/email/queries/highlights.scm");
const ERLANG_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/erlang/queries/highlights.scm");
const FORTRAN_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/fortran/queries/highlights.scm");
const FORTRAN_NAMELIST_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/fortran_namelist/queries/highlights.scm");
const FSHARP_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/fsharp/queries/highlights.scm");
const FSHARP_INJECTIONS_QUERY: &str = include_str!("../grammars/fsharp/queries/injections.scm");
const FSHARP_LOCALS_QUERY: &str = include_str!("../grammars/fsharp/queries/locals.scm");
const FSHARP_SIGNATURE_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/fsharp_signature/queries/highlights.scm");
const FSHARP_SIGNATURE_INJECTIONS_QUERY: &str =
    include_str!("../grammars/fsharp_signature/queries/injections.scm");
const FSHARP_SIGNATURE_LOCALS_QUERY: &str =
    include_str!("../grammars/fsharp_signature/queries/locals.scm");
const FSTAB_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/fstab/queries/highlights.scm");
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
const PKL_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/pkl/queries/highlights.scm");
const PKL_LOCALS_QUERY: &str = include_str!("../grammars/pkl/queries/locals.scm");
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
const TODOTXT_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/todotxt/queries/highlights.scm");
const VHDL_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/vhdl/queries/highlights.scm");
const VIM_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/vim/queries/highlights.scm");
const VIM_INJECTIONS_QUERY: &str = include_str!("../grammars/vim/queries/injections.scm");
const JQ_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/jq/queries/highlights.scm");
const LESS_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/less/queries/highlights.scm");
const DOT_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/dot/queries/highlights.scm");
const GRAPH_EASY_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/graph_easy/queries/highlights.scm");
const DOT_INJECTIONS_QUERY: &str = include_str!("../grammars/dot/queries/injections.scm");
const MERMAID_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/mermaid/queries/highlights.scm");
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
const CEL_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/cel/queries/highlights.scm");
const TEXTPROTO_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/textproto/queries/highlights.scm");
const LATEX_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/latex/queries/highlights.scm");
const LATEX_INJECTIONS_QUERY: &str = include_str!("../grammars/latex/queries/injections.scm");
const TCL_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/tcl/queries/highlights.scm");
const TEXTILE_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/textile/queries/highlights.scm");
const CSV_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/csv/queries/highlights.scm");
const TSV_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/tsv/queries/highlights.scm");
const TYPST_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/typst/queries/highlights.scm");
const TYPST_INJECTIONS_QUERY: &str = include_str!("../grammars/typst/queries/injections.scm");
const STRACE_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/strace/queries/highlights.scm");
const STYLUS_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/stylus/queries/highlights.scm");
const SYSLOG_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/syslog/queries/highlights.scm");
const SML_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/sml/queries/highlights.scm");
const SOLIDITY_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/solidity/queries/highlights.scm");
const SYSTEMVERILOG_HIGHLIGHTS_QUERY: &str =
    include_str!("../grammars/systemverilog/queries/highlights.scm");
const VARLINK_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/varlink/queries/highlights.scm");
const VERILOG_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/verilog/queries/highlights.scm");
const VIMHELP_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/vimhelp/queries/highlights.scm");
const VIMHELP_INJECTIONS_QUERY: &str = include_str!("../grammars/vimhelp/queries/injections.scm");
const VYPER_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/vyper/queries/highlights.scm");
const WGSL_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/wgsl/queries/highlights.scm");
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
const EEX_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/eex/queries/highlights.scm");
const TWIG_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/twig/queries/highlights.scm");
const ERB_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/erb/queries/highlights.scm");
const JSP_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/jsp/queries/highlights.scm");
const ASP_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/asp/queries/highlights.scm");
const ADP_HIGHLIGHTS_QUERY: &str = include_str!("../grammars/adp/queries/highlights.scm");

const JSON_LANGUAGE: LanguageFn = tree_sitter_json::LANGUAGE;
const QUERY_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::QUERY_LANGUAGE;
const IGNORE_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::IGNORE_LANGUAGE;
const GIT_LINK_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::GIT_LINK_LANGUAGE;
const GIT_MAILMAP_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::GIT_MAILMAP_LANGUAGE;
const GIT_LOG_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::GIT_LOG_LANGUAGE;
const GIT_CONFIG_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::GIT_CONFIG_LANGUAGE;
const DOCKERFILE_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::DOCKERFILE_LANGUAGE;
const ACTIONSCRIPT_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::ACTIONSCRIPT_LANGUAGE;
const ADA_LANGUAGE: LanguageFn = tree_sitter_ada::LANGUAGE;
const APPLESCRIPT_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::APPLESCRIPT_LANGUAGE;
const ASM_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::ASM_LANGUAGE;
const NASM_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::NASM_LANGUAGE;
const ASCIIDOC_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::ASCIIDOC_LANGUAGE;
const AUTHORIZED_KEYS_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::AUTHORIZED_KEYS_LANGUAGE;
const AWK_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::AWK_LANGUAGE;
const BASH_LANGUAGE: LanguageFn = tree_sitter_bash::LANGUAGE;
const BIBTEX_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::BIBTEX_LANGUAGE;
const CABAL_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::CABAL_LANGUAGE;
const CFML_LANGUAGE: LanguageFn = tree_sitter_cfml::LANGUAGE_CFML;
const CLOJURE_LANGUAGE: LanguageFn = tree_sitter_clojure_orchard::LANGUAGE;
const CMAKECACHE_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::CMAKECACHE_LANGUAGE;
const COFFEESCRIPT_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::COFFEESCRIPT_LANGUAGE;
const COMMAND_HELP_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::COMMAND_HELP_LANGUAGE;
const CPUINFO_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::CPUINFO_LANGUAGE;
const CRONTAB_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::CRONTAB_LANGUAGE;
const CRYSTAL_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::CRYSTAL_LANGUAGE;
const D_LANGUAGE: LanguageFn = tree_sitter_d::LANGUAGE;
const DEBSOURCES_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::DEBSOURCES_LANGUAGE;
const ELM_LANGUAGE: LanguageFn = tree_sitter_elm::LANGUAGE;
const EMAIL_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::EMAIL_LANGUAGE;
const ERLANG_LANGUAGE: LanguageFn = tree_sitter_erlang::LANGUAGE;
const FISH_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::FISH_LANGUAGE;
const FORTRAN_LANGUAGE: LanguageFn = tree_sitter_fortran::LANGUAGE;
const FORTRAN_NAMELIST_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::FORTRAN_NAMELIST_LANGUAGE;
const FSHARP_LANGUAGE: LanguageFn = tree_sitter_fsharp::LANGUAGE_FSHARP;
const FSHARP_SIGNATURE_LANGUAGE: LanguageFn = tree_sitter_fsharp::LANGUAGE_SIGNATURE;
const FSTAB_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::FSTAB_LANGUAGE;
const ZSH_LANGUAGE: LanguageFn = tree_sitter_zsh::LANGUAGE;
const POWERSHELL_LANGUAGE: LanguageFn = tree_sitter_powershell::LANGUAGE;
const BATCH_LANGUAGE: LanguageFn = tree_sitter_batch::LANGUAGE;
const TOML_LANGUAGE: LanguageFn = tree_sitter_toml_ng::LANGUAGE;
const YAML_LANGUAGE: LanguageFn = tree_sitter_yaml::LANGUAGE;
const HCL_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::HCL_LANGUAGE;
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
const SCSS_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::SCSS_LANGUAGE;
const SSH_CONFIG_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::SSH_CONFIG_LANGUAGE;
const GITATTRIBUTES_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::GITATTRIBUTES_LANGUAGE;
const GIT_COMMIT_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::GIT_COMMIT_LANGUAGE;
const GIT_REBASE_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::GIT_REBASE_LANGUAGE;
const REQUIREMENTS_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::REQUIREMENTS_LANGUAGE;
const APACHE_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::APACHE_LANGUAGE;
const SASS_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::SASS_LANGUAGE;
const TODOTXT_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::TODOTXT_LANGUAGE;
const VHDL_LANGUAGE: LanguageFn = tree_sitter_vhdl::LANGUAGE;
const JQ_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::JQ_LANGUAGE;
const LESS_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::LESS_LANGUAGE;
const DOT_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::DOT_LANGUAGE;
const NGINX_LANGUAGE: LanguageFn = tree_sitter_nginx::LANGUAGE;
const TYPESCRIPT_LANGUAGE: LanguageFn = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
const TSX_LANGUAGE: LanguageFn = tree_sitter_typescript::LANGUAGE_TSX;
const GO_LANGUAGE: LanguageFn = tree_sitter_go::LANGUAGE;
const GOMOD_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::GOMOD_LANGUAGE;
const GOWORK_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::GOWORK_LANGUAGE;
const GOSUM_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::GOSUM_LANGUAGE;
const SQL_LANGUAGE: LanguageFn = tree_sitter_sequel::LANGUAGE;
const HTML_LANGUAGE: LanguageFn = tree_sitter_html::LANGUAGE;
const VUE_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::VUE_LANGUAGE;
const SVELTE_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::SVELTE_LANGUAGE;
const CSS_LANGUAGE: LanguageFn = tree_sitter_css::LANGUAGE;
const JAVASCRIPT_LANGUAGE: LanguageFn = tree_sitter_javascript::LANGUAGE;
const GRAPHQL_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::GRAPHQL_LANGUAGE;
const PROTO_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::PROTO_LANGUAGE;
const CEL_LANGUAGE: LanguageFn = tree_sitter_cel::LANGUAGE;
const TEXTPROTO_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::TEXTPROTO_LANGUAGE;
const LATEX_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::LATEX_LANGUAGE;
const TCL_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::TCL_LANGUAGE;
const TEXTILE_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::TEXTILE_LANGUAGE;
const CSV_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::CSV_LANGUAGE;
const TSV_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::TSV_LANGUAGE;
const TYPST_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::TYPST_LANGUAGE;
const STRACE_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::STRACE_LANGUAGE;
const STYLUS_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::STYLUS_LANGUAGE;
const SYSLOG_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::SYSLOG_LANGUAGE;
const SML_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::SML_LANGUAGE;
const SOLIDITY_LANGUAGE: LanguageFn = tree_sitter_solidity::LANGUAGE;
const SYSTEMVERILOG_LANGUAGE: LanguageFn = tree_sitter_systemverilog::LANGUAGE;
const VARLINK_LANGUAGE: LanguageFn = tree_sitter_varlink::LANGUAGE;
const VERILOG_LANGUAGE: LanguageFn = tree_sitter_verilog::LANGUAGE;
const VIMHELP_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::VIMHELP_LANGUAGE;
const VYPER_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::VYPER_LANGUAGE;
const WGSL_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::WGSL_LANGUAGE;
const REGEX_LANGUAGE: LanguageFn = tree_sitter_regex::LANGUAGE;
const JSDOC_LANGUAGE: LanguageFn = tree_sitter_jsdoc::LANGUAGE;
const USERSCRIPT_METADATA_LANGUAGE: LanguageFn =
    tree_sitter_kat_parsers::USERSCRIPT_METADATA_LANGUAGE;
const MARKDOWN_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::MARKDOWN_LANGUAGE;
const MARKDOWN_INLINE_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::MARKDOWN_INLINE_LANGUAGE;
const JUST_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::JUST_LANGUAGE;
const RUBY_LANGUAGE: LanguageFn = tree_sitter_ruby::LANGUAGE;
const LUA_LANGUAGE: LanguageFn = tree_sitter_lua::LANGUAGE;
const NIX_LANGUAGE: LanguageFn = tree_sitter_nix::LANGUAGE;
const DOTENV_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::DOTENV_LANGUAGE;
const INI_LANGUAGE: LanguageFn = tree_sitter_ini::LANGUAGE;
const XML_LANGUAGE: LanguageFn = tree_sitter_xml::LANGUAGE_XML;
const MAKE_LANGUAGE: LanguageFn = tree_sitter_make::LANGUAGE;
const CMAKE_LANGUAGE: LanguageFn = tree_sitter_cmake::LANGUAGE;
const NINJA_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::NINJA_LANGUAGE;
const JINJA_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::JINJA_LANGUAGE;
const TWIG_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::TWIG_LANGUAGE;
const ERB_LANGUAGE: LanguageFn = tree_sitter_embedded_template::LANGUAGE;

#[derive(Clone, Copy)]
struct StaticLanguageAsset {
    id: RuntimeId,
    language: fn() -> Language,
    highlights_query: &'static str,
    injections_query: &'static str,
    locals_query: &'static str,
}

pub struct LanguageRuntime {
    pub language: Language,
    pub flat_configuration: HighlightConfiguration,
    pub injections_query: Option<Query>,
}

// One declaration owns runtime identity, parser assets and lazy initialization input.
macro_rules! language_catalog {
    ($($id:ident = $name:literal => ($language:expr, $highlights:expr, $injections:expr, $locals:expr)),* $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
        pub(crate) enum RuntimeId { $($id,)* Pem, OpenPgpArmor, PythonDocstring }

        impl RuntimeId {
            pub(crate) const fn name(self) -> &'static str { match self {
                $(Self::$id => $name,)* Self::Pem => "pem", Self::OpenPgpArmor => "openpgp_armor", Self::PythonDocstring => "python_docstring",
            }}
            pub(crate) fn from_name(name: &str) -> Option<Self> { Some(match name {
                $($name => Self::$id,)* "pem" => Self::Pem, "openpgp_armor" => Self::OpenPgpArmor, "python_docstring" => Self::PythonDocstring,
                _ => return None,
            })}
        }

        const STATIC_LANGUAGE_ASSETS: &[StaticLanguageAsset] = &[$(StaticLanguageAsset {
            id: RuntimeId::$id, language: $language, highlights_query: $highlights,
            injections_query: $injections, locals_query: $locals,
        },)*];
    };
}

language_catalog! {
    Pkl = "pkl" => (|| Language::from(tree_sitter_pkl::LANGUAGE), PKL_HIGHLIGHTS_QUERY, "", PKL_LOCALS_QUERY),
    Json = "json" => (|| Language::from(JSON_LANGUAGE), JSON_HIGHLIGHTS_QUERY, "", ""),
    Query = "query" => (|| Language::from(QUERY_LANGUAGE), QUERY_HIGHLIGHTS_QUERY, "", ""),
    Ignore = "ignore" => (|| Language::from(IGNORE_LANGUAGE), IGNORE_HIGHLIGHTS_QUERY, "", ""),
    GitLink = "git_link" => (|| Language::from(GIT_LINK_LANGUAGE), GIT_LINK_HIGHLIGHTS_QUERY, "", ""),
    GitMailmap = "git_mailmap" => (|| Language::from(GIT_MAILMAP_LANGUAGE), GIT_MAILMAP_HIGHLIGHTS_QUERY, "", ""),
    GitLog = "git_log" => (|| Language::from(GIT_LOG_LANGUAGE), GIT_LOG_HIGHLIGHTS_QUERY, GIT_LOG_INJECTIONS_QUERY, ""),
    GitConfig = "git_config" => (|| Language::from(GIT_CONFIG_LANGUAGE), GIT_CONFIG_HIGHLIGHTS_QUERY, "", ""),
    Dockerfile = "dockerfile" => (|| Language::from(DOCKERFILE_LANGUAGE), DOCKERFILE_HIGHLIGHTS_QUERY, DOCKERFILE_INJECTIONS_QUERY, ""),
    ActionScript = "actionscript" => (|| Language::from(ACTIONSCRIPT_LANGUAGE), ACTIONSCRIPT_HIGHLIGHTS_QUERY, "", ""),
    Ada = "ada" => (|| Language::from(ADA_LANGUAGE), ADA_HIGHLIGHTS_QUERY, "", ADA_LOCALS_QUERY),
    AppleScript = "applescript" => (|| Language::from(APPLESCRIPT_LANGUAGE), APPLESCRIPT_HIGHLIGHTS_QUERY, "", ""),
    Asm = "asm" => (|| Language::from(ASM_LANGUAGE), ASM_HIGHLIGHTS_QUERY, "", ""),
    Nasm = "nasm" => (|| Language::from(NASM_LANGUAGE), NASM_HIGHLIGHTS_QUERY, "", ""),
    AsciiDoc = "asciidoc" => (|| Language::from(ASCIIDOC_LANGUAGE), ASCIIDOC_HIGHLIGHTS_QUERY, "", ""),
    AuthorizedKeys = "authorized_keys" => (|| Language::from(AUTHORIZED_KEYS_LANGUAGE), AUTHORIZED_KEYS_HIGHLIGHTS_QUERY, "", ""),
    Ghostty = "ghostty" => (|| Language::from(tree_sitter_kat_parsers::GHOSTTY_LANGUAGE), include_str!("../grammars/ghostty/queries/highlights.scm"), "", ""),
    KnownHosts = "known_hosts" => (|| Language::from(tree_sitter_kat_parsers::KNOWN_HOSTS_LANGUAGE), KNOWN_HOSTS_HIGHLIGHTS_QUERY, "", ""),
    Awk = "awk" => (|| Language::from(AWK_LANGUAGE), AWK_HIGHLIGHTS_QUERY, "", ""),
    Bash = "bash" => (|| Language::from(BASH_LANGUAGE), BASH_HIGHLIGHTS_QUERY, BASH_INJECTIONS_QUERY, ""),
    Bibtex = "bibtex" => (|| Language::from(BIBTEX_LANGUAGE), BIBTEX_HIGHLIGHTS_QUERY, "", BIBTEX_LOCALS_QUERY),
    Cabal = "cabal" => (|| Language::from(CABAL_LANGUAGE), CABAL_HIGHLIGHTS_QUERY, "", ""),
    Cfml = "cfml" => (|| Language::from(CFML_LANGUAGE), CFML_HIGHLIGHTS_QUERY, CFML_INJECTIONS_QUERY, ""),
    Clojure = "clojure" => (|| Language::from(CLOJURE_LANGUAGE), CLOJURE_HIGHLIGHTS_QUERY, "", ""),
    CmakeCache = "cmakecache" => (|| Language::from(CMAKECACHE_LANGUAGE), CMAKECACHE_HIGHLIGHTS_QUERY, "", ""),
    CoffeeScript = "coffeescript" => (|| Language::from(COFFEESCRIPT_LANGUAGE), COFFEESCRIPT_HIGHLIGHTS_QUERY, COFFEESCRIPT_INJECTIONS_QUERY, ""),
    CommandHelp = "command_help" => (|| Language::from(COMMAND_HELP_LANGUAGE), COMMAND_HELP_HIGHLIGHTS_QUERY, "", ""),
    CpuInfo = "cpuinfo" => (|| Language::from(CPUINFO_LANGUAGE), CPUINFO_HIGHLIGHTS_QUERY, "", ""),
    Crontab = "crontab" => (|| Language::from(CRONTAB_LANGUAGE), CRONTAB_HIGHLIGHTS_QUERY, "", ""),
    Crystal = "crystal" => (|| Language::from(CRYSTAL_LANGUAGE), CRYSTAL_HIGHLIGHTS_QUERY, CRYSTAL_INJECTIONS_QUERY, ""),
    D = "d" => (|| Language::from(D_LANGUAGE), D_HIGHLIGHTS_QUERY, D_INJECTIONS_QUERY, ""),
    Debsources = "debsources" => (|| Language::from(DEBSOURCES_LANGUAGE), DEBSOURCES_HIGHLIGHTS_QUERY, "", ""),
    Elm = "elm" => (|| Language::from(ELM_LANGUAGE), ELM_HIGHLIGHTS_QUERY, ELM_INJECTIONS_QUERY, ELM_LOCALS_QUERY),
    Email = "email" => (|| Language::from(EMAIL_LANGUAGE), EMAIL_HIGHLIGHTS_QUERY, "", ""),
    Erlang = "erlang" => (|| Language::from(ERLANG_LANGUAGE), ERLANG_HIGHLIGHTS_QUERY, "", ""),
    Fish = "fish" => (|| Language::from(FISH_LANGUAGE), FISH_HIGHLIGHTS_QUERY, "", ""),
    Fortran = "fortran" => (|| Language::from(FORTRAN_LANGUAGE), FORTRAN_HIGHLIGHTS_QUERY, "", ""),
    FortranNamelist = "fortran_namelist" => (|| Language::from(FORTRAN_NAMELIST_LANGUAGE), FORTRAN_NAMELIST_HIGHLIGHTS_QUERY, "", ""),
    Fsharp = "fsharp" => (|| Language::from(FSHARP_LANGUAGE), FSHARP_HIGHLIGHTS_QUERY, FSHARP_INJECTIONS_QUERY, FSHARP_LOCALS_QUERY),
    FsharpSignature = "fsharp_signature" => (|| Language::from(FSHARP_SIGNATURE_LANGUAGE), FSHARP_SIGNATURE_HIGHLIGHTS_QUERY, FSHARP_SIGNATURE_INJECTIONS_QUERY, FSHARP_SIGNATURE_LOCALS_QUERY),
    Fstab = "fstab" => (|| Language::from(FSTAB_LANGUAGE), FSTAB_HIGHLIGHTS_QUERY, "", ""),
    Zsh = "zsh" => (|| Language::from(ZSH_LANGUAGE), ZSH_HIGHLIGHTS_QUERY, ZSH_INJECTIONS_QUERY, ""),
    Powershell = "powershell" => (|| Language::from(POWERSHELL_LANGUAGE), POWERSHELL_HIGHLIGHTS_QUERY, "", ""),
    Batch = "batch" => (|| Language::from(BATCH_LANGUAGE), BATCH_HIGHLIGHTS_QUERY, "", ""),
    Toml = "toml" => (|| Language::from(TOML_LANGUAGE), TOML_HIGHLIGHTS_QUERY, "", ""),
    Yaml = "yaml" => (|| Language::from(YAML_LANGUAGE), YAML_HIGHLIGHTS_QUERY, YAML_INJECTIONS_QUERY, ""),
    Hcl = "hcl" => (|| Language::from(HCL_LANGUAGE), HCL_HIGHLIGHTS_QUERY, "", ""),
    Rust = "rust" => (|| Language::from(RUST_LANGUAGE), RUST_HIGHLIGHTS_QUERY, RUST_INJECTIONS_QUERY, ""),
    Python = "python" => (|| Language::from(PYTHON_LANGUAGE), PYTHON_HIGHLIGHTS_QUERY, PYTHON_INJECTIONS_QUERY, ""),
    C = "c" => (|| Language::from(C_LANGUAGE), C_HIGHLIGHTS_QUERY, "", ""),
    Cpp = "cpp" => (|| Language::from(CPP_LANGUAGE), CPP_HIGHLIGHTS_QUERY, CPP_INJECTIONS_QUERY, ""),
    Java = "java" => (|| Language::from(JAVA_LANGUAGE), JAVA_HIGHLIGHTS_QUERY, "", ""),
    Kotlin = "kotlin" => (|| Language::from(KOTLIN_LANGUAGE), KOTLIN_HIGHLIGHTS_QUERY, "", ""),
    Csharp = "csharp" => (|| Language::from(CSHARP_LANGUAGE), CSHARP_HIGHLIGHTS_QUERY, "", ""),
    Groovy = "groovy" => (|| Language::from(GROOVY_LANGUAGE), GROOVY_HIGHLIGHTS_QUERY, "", ""),
    Diff = "diff" => (|| Language::from(DIFF_LANGUAGE), DIFF_HIGHLIGHTS_QUERY, "", ""),
    Properties = "properties" => (|| Language::from(PROPERTIES_LANGUAGE), PROPERTIES_HIGHLIGHTS_QUERY, "", ""),
    Php = "php" => (|| Language::from(PHP_LANGUAGE), PHP_HIGHLIGHTS_QUERY, PHP_INJECTIONS_QUERY, ""),
    Scala = "scala" => (|| Language::from(SCALA_LANGUAGE), SCALA_HIGHLIGHTS_QUERY, "", SCALA_LOCALS_QUERY),
    Swift = "swift" => (|| Language::from(SWIFT_LANGUAGE), SWIFT_HIGHLIGHTS_QUERY, SWIFT_INJECTIONS_QUERY, SWIFT_LOCALS_QUERY),
    Dart = "dart" => (|| Language::from(DART_LANGUAGE), DART_HIGHLIGHTS_QUERY, "", DART_LOCALS_QUERY),
    Elixir = "elixir" => (|| Language::from(ELIXIR_LANGUAGE), ELIXIR_HIGHLIGHTS_QUERY, ELIXIR_INJECTIONS_QUERY, ""),
    Zig = "zig" => (|| Language::from(ZIG_LANGUAGE), ZIG_HIGHLIGHTS_QUERY, ZIG_INJECTIONS_QUERY, ZIG_LOCALS_QUERY),
    SshConfig = "ssh_config" => (|| Language::from(SSH_CONFIG_LANGUAGE), SSH_CONFIG_HIGHLIGHTS_QUERY, SSH_CONFIG_INJECTIONS_QUERY, ""),
    GitAttributes = "gitattributes" => (|| Language::from(GITATTRIBUTES_LANGUAGE), GITATTRIBUTES_HIGHLIGHTS_QUERY, "", ""),
    GitCommit = "git_commit" => (|| Language::from(GIT_COMMIT_LANGUAGE), GIT_COMMIT_HIGHLIGHTS_QUERY, "", ""),
    GitRebase = "git_rebase" => (|| Language::from(GIT_REBASE_LANGUAGE), GIT_REBASE_HIGHLIGHTS_QUERY, "", ""),
    Requirements = "requirements" => (|| Language::from(REQUIREMENTS_LANGUAGE), REQUIREMENTS_HIGHLIGHTS_QUERY, "", ""),
    Apache = "apache" => (|| Language::from(APACHE_LANGUAGE), APACHE_HIGHLIGHTS_QUERY, "", ""),
    Scss = "scss" => (|| Language::from(SCSS_LANGUAGE), SCSS_HIGHLIGHTS_QUERY, "", ""),
    Sass = "sass" => (|| Language::from(SASS_LANGUAGE), SASS_HIGHLIGHTS_QUERY, "", ""),
    TodoTxt = "todotxt" => (|| Language::from(TODOTXT_LANGUAGE), TODOTXT_HIGHLIGHTS_QUERY, "", ""),
    Vhdl = "vhdl" => (|| Language::from(VHDL_LANGUAGE), VHDL_HIGHLIGHTS_QUERY, "", ""),
    Verilog = "verilog" => (|| Language::from(VERILOG_LANGUAGE), VERILOG_HIGHLIGHTS_QUERY, "", ""),
    SystemVerilog = "systemverilog" => (|| Language::from(SYSTEMVERILOG_LANGUAGE), SYSTEMVERILOG_HIGHLIGHTS_QUERY, "", ""),
    Vim = "vim" => (tree_sitter_vim::language, VIM_HIGHLIGHTS_QUERY, VIM_INJECTIONS_QUERY, ""),
    VimHelp = "vimhelp" => (|| Language::from(VIMHELP_LANGUAGE), VIMHELP_HIGHLIGHTS_QUERY, VIMHELP_INJECTIONS_QUERY, ""),
    Jq = "jq" => (|| Language::from(JQ_LANGUAGE), JQ_HIGHLIGHTS_QUERY, "", ""),
    Less = "less" => (|| Language::from(LESS_LANGUAGE), LESS_HIGHLIGHTS_QUERY, "", ""),
    GraphEasy = "graph_easy" => (|| Language::from(tree_sitter_kat_parsers::GRAPH_EASY_LANGUAGE), GRAPH_EASY_HIGHLIGHTS_QUERY, "", ""),
    Dot = "dot" => (|| Language::from(DOT_LANGUAGE), DOT_HIGHLIGHTS_QUERY, DOT_INJECTIONS_QUERY, ""),
    Mermaid = "mermaid" => (|| Language::from(tree_sitter_kat_parsers::MERMAID_LANGUAGE), MERMAID_HIGHLIGHTS_QUERY, "", ""),
    Nginx = "nginx" => (|| Language::from(NGINX_LANGUAGE), NGINX_HIGHLIGHTS_QUERY, "", ""),
    TypeScript = "typescript" => (|| Language::from(TYPESCRIPT_LANGUAGE), TYPESCRIPT_HIGHLIGHTS_QUERY, TYPESCRIPT_INJECTIONS_QUERY, TYPESCRIPT_LOCALS_QUERY),
    Tsx = "tsx" => (|| Language::from(TSX_LANGUAGE), TSX_HIGHLIGHTS_QUERY, TSX_INJECTIONS_QUERY, TSX_LOCALS_QUERY),
    Go = "go" => (|| Language::from(GO_LANGUAGE), GO_HIGHLIGHTS_QUERY, GO_INJECTIONS_QUERY, ""),
    GoMod = "gomod" => (|| Language::from(GOMOD_LANGUAGE), GOMOD_HIGHLIGHTS_QUERY, "", ""),
    GoWork = "gowork" => (|| Language::from(GOWORK_LANGUAGE), GOWORK_HIGHLIGHTS_QUERY, "", ""),
    GoSum = "gosum" => (|| Language::from(GOSUM_LANGUAGE), GOSUM_HIGHLIGHTS_QUERY, "", ""),
    Sql = "sql" => (|| Language::from(SQL_LANGUAGE), SQL_HIGHLIGHTS_QUERY, "", ""),
    SqlPostgres = "sql_postgres" => (|| Language::from(SQL_LANGUAGE), SQL_POSTGRES_HIGHLIGHTS_QUERY, "", ""),
    SqlMysql = "sql_mysql" => (|| Language::from(SQL_LANGUAGE), SQL_MYSQL_HIGHLIGHTS_QUERY, "", ""),
    SqlSqlite = "sql_sqlite" => (|| Language::from(SQL_LANGUAGE), SQL_SQLITE_HIGHLIGHTS_QUERY, "", ""),
    Html = "html" => (|| Language::from(HTML_LANGUAGE), HTML_HIGHLIGHTS_QUERY, HTML_INJECTIONS_QUERY, ""),
    Vue = "vue" => (|| Language::from(VUE_LANGUAGE), VUE_HIGHLIGHTS_QUERY, VUE_INJECTIONS_QUERY, ""),
    Svelte = "svelte" => (|| Language::from(SVELTE_LANGUAGE), SVELTE_HIGHLIGHTS_QUERY, SVELTE_INJECTIONS_QUERY, ""),
    Css = "css" => (|| Language::from(CSS_LANGUAGE), CSS_HIGHLIGHTS_QUERY, "", ""),
    JavaScript = "javascript" => (|| Language::from(JAVASCRIPT_LANGUAGE), JAVASCRIPT_HIGHLIGHTS_QUERY, JAVASCRIPT_INJECTIONS_QUERY, JAVASCRIPT_LOCALS_QUERY),
    Graphql = "graphql" => (|| Language::from(GRAPHQL_LANGUAGE), GRAPHQL_HIGHLIGHTS_QUERY, "", ""),
    Cel = "cel" => (|| Language::from(CEL_LANGUAGE), CEL_HIGHLIGHTS_QUERY, "", ""),
    Proto = "proto" => (|| Language::from(PROTO_LANGUAGE), PROTO_HIGHLIGHTS_QUERY, "", ""),
    Textproto = "textproto" => (|| Language::from(TEXTPROTO_LANGUAGE), TEXTPROTO_HIGHLIGHTS_QUERY, "", ""),
    Latex = "latex" => (|| Language::from(LATEX_LANGUAGE), LATEX_HIGHLIGHTS_QUERY, LATEX_INJECTIONS_QUERY, ""),
    Tcl = "tcl" => (|| Language::from(TCL_LANGUAGE), TCL_HIGHLIGHTS_QUERY, "", ""),
    Textile = "textile" => (|| Language::from(TEXTILE_LANGUAGE), TEXTILE_HIGHLIGHTS_QUERY, "", ""),
    Csv = "csv" => (|| Language::from(CSV_LANGUAGE), CSV_HIGHLIGHTS_QUERY, "", ""),
    Tsv = "tsv" => (|| Language::from(TSV_LANGUAGE), TSV_HIGHLIGHTS_QUERY, "", ""),
    Typst = "typst" => (|| Language::from(TYPST_LANGUAGE), TYPST_HIGHLIGHTS_QUERY, TYPST_INJECTIONS_QUERY, ""),
    Strace = "strace" => (|| Language::from(STRACE_LANGUAGE), STRACE_HIGHLIGHTS_QUERY, "", ""),
    Stylus = "stylus" => (|| Language::from(STYLUS_LANGUAGE), STYLUS_HIGHLIGHTS_QUERY, "", ""),
    Syslog = "syslog" => (|| Language::from(SYSLOG_LANGUAGE), SYSLOG_HIGHLIGHTS_QUERY, "", ""),
    Sml = "sml" => (|| Language::from(SML_LANGUAGE), SML_HIGHLIGHTS_QUERY, "", ""),
    Solidity = "solidity" => (|| Language::from(SOLIDITY_LANGUAGE), SOLIDITY_HIGHLIGHTS_QUERY, "", ""),
    Varlink = "varlink" => (|| Language::from(VARLINK_LANGUAGE), VARLINK_HIGHLIGHTS_QUERY, "", ""),
    Vyper = "vyper" => (|| Language::from(VYPER_LANGUAGE), VYPER_HIGHLIGHTS_QUERY, "", ""),
    Wgsl = "wgsl" => (|| Language::from(WGSL_LANGUAGE), WGSL_HIGHLIGHTS_QUERY, "", ""),
    Regex = "regex" => (|| Language::from(REGEX_LANGUAGE), REGEX_HIGHLIGHTS_QUERY, "", ""),
    RegexJavascript = "regex_javascript" => (|| Language::from(REGEX_LANGUAGE), REGEX_JAVASCRIPT_HIGHLIGHTS_QUERY, "", ""),
    RegexPython = "regex_python" => (|| Language::from(REGEX_LANGUAGE), REGEX_PYTHON_HIGHLIGHTS_QUERY, "", ""),
    RegexRust = "regex_rust" => (|| Language::from(REGEX_LANGUAGE), REGEX_RUST_HIGHLIGHTS_QUERY, "", ""),
    RegexGo = "regex_go" => (|| Language::from(REGEX_LANGUAGE), REGEX_GO_HIGHLIGHTS_QUERY, "", ""),
    RegexPosix = "regex_posix" => (|| Language::from(REGEX_LANGUAGE), REGEX_POSIX_HIGHLIGHTS_QUERY, "", ""),
    Jsdoc = "jsdoc" => (|| Language::from(JSDOC_LANGUAGE), JSDOC_HIGHLIGHTS_QUERY, "", ""),
    UserscriptMetadata = "userscript_metadata" => (|| Language::from(USERSCRIPT_METADATA_LANGUAGE), USERSCRIPT_METADATA_HIGHLIGHTS_QUERY, "", ""),
    Markdown = "markdown" => (|| Language::from(MARKDOWN_LANGUAGE), MARKDOWN_HIGHLIGHTS_QUERY, MARKDOWN_INJECTIONS_QUERY, ""),
    MarkdownInline = "markdown_inline" => (|| Language::from(MARKDOWN_INLINE_LANGUAGE), MARKDOWN_INLINE_HIGHLIGHTS_QUERY, MARKDOWN_INLINE_INJECTIONS_QUERY, ""),
    Just = "just" => (|| Language::from(JUST_LANGUAGE), JUST_HIGHLIGHTS_QUERY, JUST_INJECTIONS_QUERY, ""),
    Ruby = "ruby" => (|| Language::from(RUBY_LANGUAGE), RUBY_HIGHLIGHTS_QUERY, "", RUBY_LOCALS_QUERY),
    Lua = "lua" => (|| Language::from(LUA_LANGUAGE), LUA_HIGHLIGHTS_QUERY, LUA_INJECTIONS_QUERY, LUA_LOCALS_QUERY),
    Nix = "nix" => (|| Language::from(NIX_LANGUAGE), NIX_HIGHLIGHTS_QUERY, NIX_INJECTIONS_QUERY, ""),
    Dotenv = "dotenv" => (|| Language::from(DOTENV_LANGUAGE), DOTENV_HIGHLIGHTS_QUERY, "", ""),
    Ini = "ini" => (|| Language::from(INI_LANGUAGE), INI_HIGHLIGHTS_QUERY, "", ""),
    Xml = "xml" => (|| Language::from(XML_LANGUAGE), XML_HIGHLIGHTS_QUERY, "", ""),
    Make = "make" => (|| Language::from(MAKE_LANGUAGE), MAKE_HIGHLIGHTS_QUERY, "", ""),
    Cmake = "cmake" => (|| Language::from(CMAKE_LANGUAGE), CMAKE_HIGHLIGHTS_QUERY, "", ""),
    Ninja = "ninja" => (|| Language::from(NINJA_LANGUAGE), NINJA_HIGHLIGHTS_QUERY, "", ""),
    Jinja = "jinja" => (|| Language::from(JINJA_LANGUAGE), JINJA_HIGHLIGHTS_QUERY, "", ""),
    Eex = "eex" => (|| Language::from(ERB_LANGUAGE), EEX_HIGHLIGHTS_QUERY, "", ""),
    Twig = "twig" => (|| Language::from(TWIG_LANGUAGE), TWIG_HIGHLIGHTS_QUERY, "", ""),
    Erb = "erb" => (|| Language::from(ERB_LANGUAGE), ERB_HIGHLIGHTS_QUERY, "", ""),
    Jsp = "jsp" => (|| Language::from(ERB_LANGUAGE), JSP_HIGHLIGHTS_QUERY, "", ""),
    Asp = "asp" => (|| Language::from(ERB_LANGUAGE), ASP_HIGHLIGHTS_QUERY, "", ""),
    Adp = "adp" => (|| Language::from(ERB_LANGUAGE), ADP_HIGHLIGHTS_QUERY, "", ""),
}

struct RuntimeSlot {
    asset: &'static StaticLanguageAsset,
    runtime: OnceLock<LanguageRuntime>,
}

static RUNTIME_SLOTS: LazyLock<BTreeMap<RuntimeId, RuntimeSlot>> = LazyLock::new(|| {
    STATIC_LANGUAGE_ASSETS
        .iter()
        .map(|asset| {
            (
                asset.id,
                RuntimeSlot {
                    asset,
                    runtime: OnceLock::new(),
                },
            )
        })
        .collect()
});

pub fn runtime(name: &str) -> Option<&'static LanguageRuntime> {
    let slot = RUNTIME_SLOTS.get(&RuntimeId::from_name(name)?)?;
    Some(slot.runtime.get_or_init(|| {
        build_runtime(*slot.asset).unwrap_or_else(|error| {
            panic!(
                "failed to initialize language runtime {}: {error:#}",
                slot.asset.id.name()
            )
        })
    }))
}

pub fn supports_runtime(name: &str) -> bool {
    RuntimeId::from_name(name).is_some()
}

pub fn global_highlight_name(highlight_index: usize) -> &'static str {
    HIGHLIGHT_NAMES[highlight_index]
}

fn build_runtime(asset: StaticLanguageAsset) -> Result<LanguageRuntime> {
    progress_log("runtime_init", format!("begin runtime={}", asset.id.name()));
    let started_at = Instant::now();

    let language = (asset.language)();
    let mut flat_configuration = HighlightConfiguration::new(
        language.clone(),
        asset.id.name(),
        asset.highlights_query,
        "",
        asset.locals_query,
    )
    .with_context(|| {
        format!(
            "failed to build flat highlight configuration for {}",
            asset.id.name()
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
                    format!("failed to build injections query for {}", asset.id.name())
                })?,
            )
        },
    };

    progress_log(
        "runtime_init",
        format!(
            "done runtime={} elapsed={:.3}ms",
            asset.id.name(),
            started_at.elapsed().as_secs_f64() * 1_000.0
        ),
    );

    Ok(runtime)
}

#[cfg(test)]
mod tests {
    use super::{STATIC_LANGUAGE_ASSETS, build_runtime, runtime};

    #[test]
    fn registered_languages_load_with_the_current_runtime() {
        let mut failures = Vec::new();
        for asset in STATIC_LANGUAGE_ASSETS {
            let language_runtime = match build_runtime(*asset) {
                Ok(runtime) => runtime,
                Err(error) => {
                    failures.push(format!("{}: {error:#}", asset.id.name()));
                    continue;
                }
            };
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&language_runtime.language).unwrap();
            let tree = parser.parse("", None).unwrap();
            assert_eq!(tree.root_node().end_byte(), 0, "{}", asset.id.name());
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

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
