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
    name: &'static str,
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

const STATIC_LANGUAGE_ASSETS: &[StaticLanguageAsset] = &[
    StaticLanguageAsset {
        name: "pkl",
        language: || Language::from(tree_sitter_pkl::LANGUAGE),
        highlights_query: PKL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: PKL_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "json",
        language: || Language::from(JSON_LANGUAGE),
        highlights_query: JSON_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "query",
        language: || Language::from(QUERY_LANGUAGE),
        highlights_query: QUERY_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "ignore",
        language: || Language::from(IGNORE_LANGUAGE),
        highlights_query: IGNORE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "git_link",
        language: || Language::from(GIT_LINK_LANGUAGE),
        highlights_query: GIT_LINK_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "git_mailmap",
        language: || Language::from(GIT_MAILMAP_LANGUAGE),
        highlights_query: GIT_MAILMAP_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "git_log",
        language: || Language::from(GIT_LOG_LANGUAGE),
        highlights_query: GIT_LOG_HIGHLIGHTS_QUERY,
        injections_query: GIT_LOG_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "git_config",
        language: || Language::from(GIT_CONFIG_LANGUAGE),
        highlights_query: GIT_CONFIG_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "dockerfile",
        language: || Language::from(DOCKERFILE_LANGUAGE),
        highlights_query: DOCKERFILE_HIGHLIGHTS_QUERY,
        injections_query: DOCKERFILE_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "actionscript",
        language: || Language::from(ACTIONSCRIPT_LANGUAGE),
        highlights_query: ACTIONSCRIPT_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "ada",
        language: || Language::from(ADA_LANGUAGE),
        highlights_query: ADA_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: ADA_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "applescript",
        language: || Language::from(APPLESCRIPT_LANGUAGE),
        highlights_query: APPLESCRIPT_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "asm",
        language: || Language::from(ASM_LANGUAGE),
        highlights_query: ASM_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "nasm",
        language: || Language::from(NASM_LANGUAGE),
        highlights_query: NASM_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "asciidoc",
        language: || Language::from(ASCIIDOC_LANGUAGE),
        highlights_query: ASCIIDOC_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "authorized_keys",
        language: || Language::from(AUTHORIZED_KEYS_LANGUAGE),
        highlights_query: AUTHORIZED_KEYS_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "ghostty",
        language: || Language::from(tree_sitter_kat_parsers::GHOSTTY_LANGUAGE),
        highlights_query: include_str!("../grammars/ghostty/queries/highlights.scm"),
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "known_hosts",
        language: || Language::from(tree_sitter_kat_parsers::KNOWN_HOSTS_LANGUAGE),
        highlights_query: KNOWN_HOSTS_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "awk",
        language: || Language::from(AWK_LANGUAGE),
        highlights_query: AWK_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "bash",
        language: || Language::from(BASH_LANGUAGE),
        highlights_query: BASH_HIGHLIGHTS_QUERY,
        injections_query: BASH_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "bibtex",
        language: || Language::from(BIBTEX_LANGUAGE),
        highlights_query: BIBTEX_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: BIBTEX_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "cabal",
        language: || Language::from(CABAL_LANGUAGE),
        highlights_query: CABAL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "cfml",
        language: || Language::from(CFML_LANGUAGE),
        highlights_query: CFML_HIGHLIGHTS_QUERY,
        injections_query: CFML_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "clojure",
        language: || Language::from(CLOJURE_LANGUAGE),
        highlights_query: CLOJURE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "cmakecache",
        language: || Language::from(CMAKECACHE_LANGUAGE),
        highlights_query: CMAKECACHE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "coffeescript",
        language: || Language::from(COFFEESCRIPT_LANGUAGE),
        highlights_query: COFFEESCRIPT_HIGHLIGHTS_QUERY,
        injections_query: COFFEESCRIPT_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "command_help",
        language: || Language::from(COMMAND_HELP_LANGUAGE),
        highlights_query: COMMAND_HELP_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "cpuinfo",
        language: || Language::from(CPUINFO_LANGUAGE),
        highlights_query: CPUINFO_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "crontab",
        language: || Language::from(CRONTAB_LANGUAGE),
        highlights_query: CRONTAB_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "crystal",
        language: || Language::from(CRYSTAL_LANGUAGE),
        highlights_query: CRYSTAL_HIGHLIGHTS_QUERY,
        injections_query: CRYSTAL_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "d",
        language: || Language::from(D_LANGUAGE),
        highlights_query: D_HIGHLIGHTS_QUERY,
        injections_query: D_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "debsources",
        language: || Language::from(DEBSOURCES_LANGUAGE),
        highlights_query: DEBSOURCES_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "elm",
        language: || Language::from(ELM_LANGUAGE),
        highlights_query: ELM_HIGHLIGHTS_QUERY,
        injections_query: ELM_INJECTIONS_QUERY,
        locals_query: ELM_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "email",
        language: || Language::from(EMAIL_LANGUAGE),
        highlights_query: EMAIL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "erlang",
        language: || Language::from(ERLANG_LANGUAGE),
        highlights_query: ERLANG_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "fish",
        language: || Language::from(FISH_LANGUAGE),
        highlights_query: FISH_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "fortran",
        language: || Language::from(FORTRAN_LANGUAGE),
        highlights_query: FORTRAN_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "fortran_namelist",
        language: || Language::from(FORTRAN_NAMELIST_LANGUAGE),
        highlights_query: FORTRAN_NAMELIST_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "fsharp",
        language: || Language::from(FSHARP_LANGUAGE),
        highlights_query: FSHARP_HIGHLIGHTS_QUERY,
        injections_query: FSHARP_INJECTIONS_QUERY,
        locals_query: FSHARP_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "fsharp_signature",
        language: || Language::from(FSHARP_SIGNATURE_LANGUAGE),
        highlights_query: FSHARP_SIGNATURE_HIGHLIGHTS_QUERY,
        injections_query: FSHARP_SIGNATURE_INJECTIONS_QUERY,
        locals_query: FSHARP_SIGNATURE_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "fstab",
        language: || Language::from(FSTAB_LANGUAGE),
        highlights_query: FSTAB_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "zsh",
        language: || Language::from(ZSH_LANGUAGE),
        highlights_query: ZSH_HIGHLIGHTS_QUERY,
        injections_query: ZSH_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "powershell",
        language: || Language::from(POWERSHELL_LANGUAGE),
        highlights_query: POWERSHELL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "batch",
        language: || Language::from(BATCH_LANGUAGE),
        highlights_query: BATCH_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "toml",
        language: || Language::from(TOML_LANGUAGE),
        highlights_query: TOML_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "yaml",
        language: || Language::from(YAML_LANGUAGE),
        highlights_query: YAML_HIGHLIGHTS_QUERY,
        injections_query: YAML_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "hcl",
        language: || Language::from(HCL_LANGUAGE),
        highlights_query: HCL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "rust",
        language: || Language::from(RUST_LANGUAGE),
        highlights_query: RUST_HIGHLIGHTS_QUERY,
        injections_query: RUST_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "python",
        language: || Language::from(PYTHON_LANGUAGE),
        highlights_query: PYTHON_HIGHLIGHTS_QUERY,
        injections_query: PYTHON_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "c",
        language: || Language::from(C_LANGUAGE),
        highlights_query: C_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "cpp",
        language: || Language::from(CPP_LANGUAGE),
        highlights_query: CPP_HIGHLIGHTS_QUERY,
        injections_query: CPP_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "java",
        language: || Language::from(JAVA_LANGUAGE),
        highlights_query: JAVA_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "kotlin",
        language: || Language::from(KOTLIN_LANGUAGE),
        highlights_query: KOTLIN_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "csharp",
        language: || Language::from(CSHARP_LANGUAGE),
        highlights_query: CSHARP_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "groovy",
        language: || Language::from(GROOVY_LANGUAGE),
        highlights_query: GROOVY_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "diff",
        language: || Language::from(DIFF_LANGUAGE),
        highlights_query: DIFF_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "properties",
        language: || Language::from(PROPERTIES_LANGUAGE),
        highlights_query: PROPERTIES_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "php",
        language: || Language::from(PHP_LANGUAGE),
        highlights_query: PHP_HIGHLIGHTS_QUERY,
        injections_query: PHP_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "scala",
        language: || Language::from(SCALA_LANGUAGE),
        highlights_query: SCALA_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: SCALA_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "swift",
        language: || Language::from(SWIFT_LANGUAGE),
        highlights_query: SWIFT_HIGHLIGHTS_QUERY,
        injections_query: SWIFT_INJECTIONS_QUERY,
        locals_query: SWIFT_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "dart",
        language: || Language::from(DART_LANGUAGE),
        highlights_query: DART_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: DART_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "elixir",
        language: || Language::from(ELIXIR_LANGUAGE),
        highlights_query: ELIXIR_HIGHLIGHTS_QUERY,
        injections_query: ELIXIR_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "zig",
        language: || Language::from(ZIG_LANGUAGE),
        highlights_query: ZIG_HIGHLIGHTS_QUERY,
        injections_query: ZIG_INJECTIONS_QUERY,
        locals_query: ZIG_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "ssh_config",
        language: || Language::from(SSH_CONFIG_LANGUAGE),
        highlights_query: SSH_CONFIG_HIGHLIGHTS_QUERY,
        injections_query: SSH_CONFIG_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "gitattributes",
        language: || Language::from(GITATTRIBUTES_LANGUAGE),
        highlights_query: GITATTRIBUTES_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "git_commit",
        language: || Language::from(GIT_COMMIT_LANGUAGE),
        highlights_query: GIT_COMMIT_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "git_rebase",
        language: || Language::from(GIT_REBASE_LANGUAGE),
        highlights_query: GIT_REBASE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "requirements",
        language: || Language::from(REQUIREMENTS_LANGUAGE),
        highlights_query: REQUIREMENTS_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "apache",
        language: || Language::from(APACHE_LANGUAGE),
        highlights_query: APACHE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "scss",
        language: || Language::from(SCSS_LANGUAGE),
        highlights_query: SCSS_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "sass",
        language: || Language::from(SASS_LANGUAGE),
        highlights_query: SASS_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "todotxt",
        language: || Language::from(TODOTXT_LANGUAGE),
        highlights_query: TODOTXT_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "vhdl",
        language: || Language::from(VHDL_LANGUAGE),
        highlights_query: VHDL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "verilog",
        language: || Language::from(VERILOG_LANGUAGE),
        highlights_query: VERILOG_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "systemverilog",
        language: || Language::from(SYSTEMVERILOG_LANGUAGE),
        highlights_query: SYSTEMVERILOG_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "vim",
        language: tree_sitter_vim::language,
        highlights_query: VIM_HIGHLIGHTS_QUERY,
        injections_query: VIM_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "vimhelp",
        language: || Language::from(VIMHELP_LANGUAGE),
        highlights_query: VIMHELP_HIGHLIGHTS_QUERY,
        injections_query: VIMHELP_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "jq",
        language: || Language::from(JQ_LANGUAGE),
        highlights_query: JQ_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "less",
        language: || Language::from(LESS_LANGUAGE),
        highlights_query: LESS_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "dot",
        language: || Language::from(DOT_LANGUAGE),
        highlights_query: DOT_HIGHLIGHTS_QUERY,
        injections_query: DOT_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "mermaid",
        language: || Language::from(tree_sitter_kat_parsers::MERMAID_LANGUAGE),
        highlights_query: MERMAID_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "nginx",
        language: || Language::from(NGINX_LANGUAGE),
        highlights_query: NGINX_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "typescript",
        language: || Language::from(TYPESCRIPT_LANGUAGE),
        highlights_query: TYPESCRIPT_HIGHLIGHTS_QUERY,
        injections_query: TYPESCRIPT_INJECTIONS_QUERY,
        locals_query: TYPESCRIPT_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "tsx",
        language: || Language::from(TSX_LANGUAGE),
        highlights_query: TSX_HIGHLIGHTS_QUERY,
        injections_query: TSX_INJECTIONS_QUERY,
        locals_query: TSX_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "go",
        language: || Language::from(GO_LANGUAGE),
        highlights_query: GO_HIGHLIGHTS_QUERY,
        injections_query: GO_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "gomod",
        language: || Language::from(GOMOD_LANGUAGE),
        highlights_query: GOMOD_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "gowork",
        language: || Language::from(GOWORK_LANGUAGE),
        highlights_query: GOWORK_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "gosum",
        language: || Language::from(GOSUM_LANGUAGE),
        highlights_query: GOSUM_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "sql",
        language: || Language::from(SQL_LANGUAGE),
        highlights_query: SQL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "sql_postgres",
        language: || Language::from(SQL_LANGUAGE),
        highlights_query: SQL_POSTGRES_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "sql_mysql",
        language: || Language::from(SQL_LANGUAGE),
        highlights_query: SQL_MYSQL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "sql_sqlite",
        language: || Language::from(SQL_LANGUAGE),
        highlights_query: SQL_SQLITE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "html",
        language: || Language::from(HTML_LANGUAGE),
        highlights_query: HTML_HIGHLIGHTS_QUERY,
        injections_query: HTML_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "vue",
        language: || Language::from(VUE_LANGUAGE),
        highlights_query: VUE_HIGHLIGHTS_QUERY,
        injections_query: VUE_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "svelte",
        language: || Language::from(SVELTE_LANGUAGE),
        highlights_query: SVELTE_HIGHLIGHTS_QUERY,
        injections_query: SVELTE_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "css",
        language: || Language::from(CSS_LANGUAGE),
        highlights_query: CSS_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "javascript",
        language: || Language::from(JAVASCRIPT_LANGUAGE),
        highlights_query: JAVASCRIPT_HIGHLIGHTS_QUERY,
        injections_query: JAVASCRIPT_INJECTIONS_QUERY,
        locals_query: JAVASCRIPT_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "graphql",
        language: || Language::from(GRAPHQL_LANGUAGE),
        highlights_query: GRAPHQL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "cel",
        language: || Language::from(CEL_LANGUAGE),
        highlights_query: CEL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "proto",
        language: || Language::from(PROTO_LANGUAGE),
        highlights_query: PROTO_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "textproto",
        language: || Language::from(TEXTPROTO_LANGUAGE),
        highlights_query: TEXTPROTO_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "latex",
        language: || Language::from(LATEX_LANGUAGE),
        highlights_query: LATEX_HIGHLIGHTS_QUERY,
        injections_query: LATEX_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "tcl",
        language: || Language::from(TCL_LANGUAGE),
        highlights_query: TCL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "textile",
        language: || Language::from(TEXTILE_LANGUAGE),
        highlights_query: TEXTILE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "csv",
        language: || Language::from(CSV_LANGUAGE),
        highlights_query: CSV_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "tsv",
        language: || Language::from(TSV_LANGUAGE),
        highlights_query: TSV_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "typst",
        language: || Language::from(TYPST_LANGUAGE),
        highlights_query: TYPST_HIGHLIGHTS_QUERY,
        injections_query: TYPST_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "strace",
        language: || Language::from(STRACE_LANGUAGE),
        highlights_query: STRACE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "stylus",
        language: || Language::from(STYLUS_LANGUAGE),
        highlights_query: STYLUS_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "syslog",
        language: || Language::from(SYSLOG_LANGUAGE),
        highlights_query: SYSLOG_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "sml",
        language: || Language::from(SML_LANGUAGE),
        highlights_query: SML_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "solidity",
        language: || Language::from(SOLIDITY_LANGUAGE),
        highlights_query: SOLIDITY_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "varlink",
        language: || Language::from(VARLINK_LANGUAGE),
        highlights_query: VARLINK_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "vyper",
        language: || Language::from(VYPER_LANGUAGE),
        highlights_query: VYPER_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "wgsl",
        language: || Language::from(WGSL_LANGUAGE),
        highlights_query: WGSL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "regex",
        language: || Language::from(REGEX_LANGUAGE),
        highlights_query: REGEX_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "regex_javascript",
        language: || Language::from(REGEX_LANGUAGE),
        highlights_query: REGEX_JAVASCRIPT_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "regex_python",
        language: || Language::from(REGEX_LANGUAGE),
        highlights_query: REGEX_PYTHON_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "regex_rust",
        language: || Language::from(REGEX_LANGUAGE),
        highlights_query: REGEX_RUST_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "regex_go",
        language: || Language::from(REGEX_LANGUAGE),
        highlights_query: REGEX_GO_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "regex_posix",
        language: || Language::from(REGEX_LANGUAGE),
        highlights_query: REGEX_POSIX_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "jsdoc",
        language: || Language::from(JSDOC_LANGUAGE),
        highlights_query: JSDOC_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "userscript_metadata",
        language: || Language::from(USERSCRIPT_METADATA_LANGUAGE),
        highlights_query: USERSCRIPT_METADATA_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "markdown",
        language: || Language::from(MARKDOWN_LANGUAGE),
        highlights_query: MARKDOWN_HIGHLIGHTS_QUERY,
        injections_query: MARKDOWN_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "markdown_inline",
        language: || Language::from(MARKDOWN_INLINE_LANGUAGE),
        highlights_query: MARKDOWN_INLINE_HIGHLIGHTS_QUERY,
        injections_query: MARKDOWN_INLINE_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "just",
        language: || Language::from(JUST_LANGUAGE),
        highlights_query: JUST_HIGHLIGHTS_QUERY,
        injections_query: JUST_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "ruby",
        language: || Language::from(RUBY_LANGUAGE),
        highlights_query: RUBY_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: RUBY_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "lua",
        language: || Language::from(LUA_LANGUAGE),
        highlights_query: LUA_HIGHLIGHTS_QUERY,
        injections_query: LUA_INJECTIONS_QUERY,
        locals_query: LUA_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "nix",
        language: || Language::from(NIX_LANGUAGE),
        highlights_query: NIX_HIGHLIGHTS_QUERY,
        injections_query: NIX_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "dotenv",
        language: || Language::from(DOTENV_LANGUAGE),
        highlights_query: DOTENV_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "ini",
        language: || Language::from(INI_LANGUAGE),
        highlights_query: INI_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "xml",
        language: || Language::from(XML_LANGUAGE),
        highlights_query: XML_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "make",
        language: || Language::from(MAKE_LANGUAGE),
        highlights_query: MAKE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "cmake",
        language: || Language::from(CMAKE_LANGUAGE),
        highlights_query: CMAKE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "ninja",
        language: || Language::from(NINJA_LANGUAGE),
        highlights_query: NINJA_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "jinja",
        language: || Language::from(JINJA_LANGUAGE),
        highlights_query: JINJA_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "eex",
        language: || Language::from(ERB_LANGUAGE),
        highlights_query: EEX_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "twig",
        language: || Language::from(TWIG_LANGUAGE),
        highlights_query: TWIG_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "erb",
        language: || Language::from(ERB_LANGUAGE),
        highlights_query: ERB_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "jsp",
        language: || Language::from(ERB_LANGUAGE),
        highlights_query: JSP_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "asp",
        language: || Language::from(ERB_LANGUAGE),
        highlights_query: ASP_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "adp",
        language: || Language::from(ERB_LANGUAGE),
        highlights_query: ADP_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
];

struct RuntimeSlot {
    asset: &'static StaticLanguageAsset,
    runtime: OnceLock<LanguageRuntime>,
}

static RUNTIME_SLOTS: LazyLock<BTreeMap<&'static str, RuntimeSlot>> = LazyLock::new(|| {
    STATIC_LANGUAGE_ASSETS
        .iter()
        .map(|asset| {
            (
                asset.name,
                RuntimeSlot {
                    asset,
                    runtime: OnceLock::new(),
                },
            )
        })
        .collect()
});

pub fn runtime(name: &str) -> Option<&'static LanguageRuntime> {
    let slot = RUNTIME_SLOTS.get(name)?;
    Some(slot.runtime.get_or_init(|| {
        build_runtime(*slot.asset).unwrap_or_else(|error| {
            panic!(
                "failed to initialize language runtime {}: {error:#}",
                slot.asset.name
            )
        })
    }))
}

pub fn supports_runtime(name: &str) -> bool {
    matches!(name, "python_docstring" | "pem" | "openpgp_armor") || RUNTIME_SLOTS.contains_key(name)
}

pub fn global_highlight_name(highlight_index: usize) -> &'static str {
    HIGHLIGHT_NAMES[highlight_index]
}

fn build_runtime(asset: StaticLanguageAsset) -> Result<LanguageRuntime> {
    progress_log("runtime_init", format!("begin runtime={}", asset.name));
    let started_at = Instant::now();

    let language = (asset.language)();
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
    use super::{STATIC_LANGUAGE_ASSETS, build_runtime, runtime};

    #[test]
    fn registered_languages_load_with_the_current_runtime() {
        let mut failures = Vec::new();
        for asset in STATIC_LANGUAGE_ASSETS {
            let language_runtime = match build_runtime(*asset) {
                Ok(runtime) => runtime,
                Err(error) => {
                    failures.push(format!("{}: {error:#}", asset.name));
                    continue;
                }
            };
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&language_runtime.language).unwrap();
            let tree = parser.parse("", None).unwrap();
            assert_eq!(tree.root_node().end_byte(), 0, "{}", asset.name);
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
