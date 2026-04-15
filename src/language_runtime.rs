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

unsafe extern "C" {
    fn tree_sitter_ignore() -> *const ();
    fn tree_sitter_query() -> *const ();
    fn tree_sitter_git_link() -> *const ();
    fn tree_sitter_git_mailmap() -> *const ();
    fn tree_sitter_git_log() -> *const ();
    fn tree_sitter_git_config() -> *const ();
    fn tree_sitter_dockerfile() -> *const ();
    fn tree_sitter_actionscript() -> *const ();
    fn tree_sitter_applescript() -> *const ();
    fn tree_sitter_asm() -> *const ();
    fn tree_sitter_nasm() -> *const ();
    fn tree_sitter_asciidoc() -> *const ();
    fn tree_sitter_authorized_keys() -> *const ();
    fn tree_sitter_awk() -> *const ();
    fn tree_sitter_bibtex() -> *const ();
    fn tree_sitter_cabal() -> *const ();
    fn tree_sitter_cmakecache() -> *const ();
    fn tree_sitter_command_help() -> *const ();
    fn tree_sitter_cpuinfo() -> *const ();
    fn tree_sitter_crontab() -> *const ();
    fn tree_sitter_debsources() -> *const ();
    fn tree_sitter_fish() -> *const ();
    fn tree_sitter_fstab() -> *const ();
    fn tree_sitter_fortran_namelist() -> *const ();
    fn tree_sitter_hcl() -> *const ();
    fn tree_sitter_gomod() -> *const ();
    fn tree_sitter_gowork() -> *const ();
    fn tree_sitter_gosum() -> *const ();
    fn tree_sitter_mail() -> *const ();
    fn tree_sitter_vue() -> *const ();
    fn tree_sitter_svelte() -> *const ();
    fn tree_sitter_scss() -> *const ();
    fn tree_sitter_graphql() -> *const ();
    fn tree_sitter_proto() -> *const ();
    fn tree_sitter_textproto() -> *const ();
    fn tree_sitter_latex() -> *const ();
    fn tree_sitter_tcl() -> *const ();
    fn tree_sitter_textile() -> *const ();
    fn tree_sitter_csv() -> *const ();
    fn tree_sitter_tsv() -> *const ();
    fn tree_sitter_typst() -> *const ();
    fn tree_sitter_strace() -> *const ();
    fn tree_sitter_stylus() -> *const ();
    fn tree_sitter_syslog() -> *const ();
    fn tree_sitter_sml() -> *const ();
    fn tree_sitter_vimdoc() -> *const ();
    fn tree_sitter_vyper() -> *const ();
    fn tree_sitter_wgsl() -> *const ();
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
    fn tree_sitter_todotxt() -> *const ();
}

const JSON_LANGUAGE: LanguageFn = tree_sitter_json::LANGUAGE;
const QUERY_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_query) };
const IGNORE_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_ignore) };
const GIT_LINK_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_git_link) };
const GIT_MAILMAP_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_git_mailmap) };
const GIT_LOG_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_git_log) };
const GIT_CONFIG_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_git_config) };
const DOCKERFILE_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_dockerfile) };
const ACTIONSCRIPT_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_actionscript) };
const ADA_LANGUAGE: LanguageFn = tree_sitter_ada::LANGUAGE;
const APPLESCRIPT_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_applescript) };
const ASM_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_asm) };
const NASM_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_nasm) };
const ASCIIDOC_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_asciidoc) };
const AUTHORIZED_KEYS_LANGUAGE: LanguageFn =
    unsafe { LanguageFn::from_raw(tree_sitter_authorized_keys) };
const AWK_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_awk) };
const BASH_LANGUAGE: LanguageFn = tree_sitter_bash::LANGUAGE;
const BIBTEX_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_bibtex) };
const CABAL_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_cabal) };
const CFML_LANGUAGE: LanguageFn = tree_sitter_cfml::LANGUAGE_CFML;
const CLOJURE_LANGUAGE: LanguageFn = tree_sitter_clojure_orchard::LANGUAGE;
const CMAKECACHE_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_cmakecache) };
const COFFEESCRIPT_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::COFFEESCRIPT_LANGUAGE;
const COMMAND_HELP_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_command_help) };
const CPUINFO_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_cpuinfo) };
const CRONTAB_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_crontab) };
const CRYSTAL_LANGUAGE: LanguageFn = tree_sitter_kat_parsers::CRYSTAL_LANGUAGE;
const D_LANGUAGE: LanguageFn = tree_sitter_d::LANGUAGE;
const DEBSOURCES_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_debsources) };
const ELM_LANGUAGE: LanguageFn = tree_sitter_elm::LANGUAGE;
const EMAIL_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_mail) };
const ERLANG_LANGUAGE: LanguageFn = tree_sitter_erlang::LANGUAGE;
const FISH_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_fish) };
const FORTRAN_LANGUAGE: LanguageFn = tree_sitter_fortran::LANGUAGE;
const FORTRAN_NAMELIST_LANGUAGE: LanguageFn =
    unsafe { LanguageFn::from_raw(tree_sitter_fortran_namelist) };
const FSHARP_LANGUAGE: LanguageFn = tree_sitter_fsharp::LANGUAGE_FSHARP;
const FSHARP_SIGNATURE_LANGUAGE: LanguageFn = tree_sitter_fsharp::LANGUAGE_SIGNATURE;
const FSTAB_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_fstab) };
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
const TODOTXT_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_todotxt) };
const VHDL_LANGUAGE: LanguageFn = tree_sitter_vhdl::LANGUAGE;
// `tree-sitter-vim` still exposes `fn language() -> Language` instead of `LanguageFn`.
// The runtime bridge below special-cases this asset name and ignores the placeholder.
const VIM_LANGUAGE_PLACEHOLDER: LanguageFn = JSON_LANGUAGE;
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
const LATEX_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_latex) };
const TCL_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_tcl) };
const TEXTILE_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_textile) };
const CSV_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_csv) };
const TSV_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_tsv) };
const TYPST_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_typst) };
const STRACE_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_strace) };
const STYLUS_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_stylus) };
const SYSLOG_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_syslog) };
const SML_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_sml) };
const SOLIDITY_LANGUAGE: LanguageFn = tree_sitter_solidity::LANGUAGE;
const SYSTEMVERILOG_LANGUAGE: LanguageFn = tree_sitter_systemverilog::LANGUAGE;
const VARLINK_LANGUAGE: LanguageFn = tree_sitter_varlink::LANGUAGE;
const VERILOG_LANGUAGE: LanguageFn = tree_sitter_verilog::LANGUAGE;
const VIMHELP_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_vimdoc) };
const VYPER_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_vyper) };
const WGSL_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_wgsl) };
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
        name: "git_link",
        language_fn: GIT_LINK_LANGUAGE,
        highlights_query: GIT_LINK_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "git_mailmap",
        language_fn: GIT_MAILMAP_LANGUAGE,
        highlights_query: GIT_MAILMAP_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "git_log",
        language_fn: GIT_LOG_LANGUAGE,
        highlights_query: GIT_LOG_HIGHLIGHTS_QUERY,
        injections_query: GIT_LOG_INJECTIONS_QUERY,
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
        name: "actionscript",
        language_fn: ACTIONSCRIPT_LANGUAGE,
        highlights_query: ACTIONSCRIPT_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "ada",
        language_fn: ADA_LANGUAGE,
        highlights_query: ADA_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: ADA_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "applescript",
        language_fn: APPLESCRIPT_LANGUAGE,
        highlights_query: APPLESCRIPT_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "asm",
        language_fn: ASM_LANGUAGE,
        highlights_query: ASM_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "nasm",
        language_fn: NASM_LANGUAGE,
        highlights_query: NASM_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "asciidoc",
        language_fn: ASCIIDOC_LANGUAGE,
        highlights_query: ASCIIDOC_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "authorized_keys",
        language_fn: AUTHORIZED_KEYS_LANGUAGE,
        highlights_query: AUTHORIZED_KEYS_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "awk",
        language_fn: AWK_LANGUAGE,
        highlights_query: AWK_HIGHLIGHTS_QUERY,
        injections_query: "",
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
        name: "bibtex",
        language_fn: BIBTEX_LANGUAGE,
        highlights_query: BIBTEX_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: BIBTEX_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "cabal",
        language_fn: CABAL_LANGUAGE,
        highlights_query: CABAL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "cfml",
        language_fn: CFML_LANGUAGE,
        highlights_query: CFML_HIGHLIGHTS_QUERY,
        injections_query: CFML_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "clojure",
        language_fn: CLOJURE_LANGUAGE,
        highlights_query: CLOJURE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "cmakecache",
        language_fn: CMAKECACHE_LANGUAGE,
        highlights_query: CMAKECACHE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "coffeescript",
        language_fn: COFFEESCRIPT_LANGUAGE,
        highlights_query: COFFEESCRIPT_HIGHLIGHTS_QUERY,
        injections_query: COFFEESCRIPT_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "command_help",
        language_fn: COMMAND_HELP_LANGUAGE,
        highlights_query: COMMAND_HELP_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "cpuinfo",
        language_fn: CPUINFO_LANGUAGE,
        highlights_query: CPUINFO_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "crontab",
        language_fn: CRONTAB_LANGUAGE,
        highlights_query: CRONTAB_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "crystal",
        language_fn: CRYSTAL_LANGUAGE,
        highlights_query: CRYSTAL_HIGHLIGHTS_QUERY,
        injections_query: CRYSTAL_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "d",
        language_fn: D_LANGUAGE,
        highlights_query: D_HIGHLIGHTS_QUERY,
        injections_query: D_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "debsources",
        language_fn: DEBSOURCES_LANGUAGE,
        highlights_query: DEBSOURCES_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "elm",
        language_fn: ELM_LANGUAGE,
        highlights_query: ELM_HIGHLIGHTS_QUERY,
        injections_query: ELM_INJECTIONS_QUERY,
        locals_query: ELM_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "email",
        language_fn: EMAIL_LANGUAGE,
        highlights_query: EMAIL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "erlang",
        language_fn: ERLANG_LANGUAGE,
        highlights_query: ERLANG_HIGHLIGHTS_QUERY,
        injections_query: "",
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
        name: "fortran",
        language_fn: FORTRAN_LANGUAGE,
        highlights_query: FORTRAN_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "fortran_namelist",
        language_fn: FORTRAN_NAMELIST_LANGUAGE,
        highlights_query: FORTRAN_NAMELIST_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "fsharp",
        language_fn: FSHARP_LANGUAGE,
        highlights_query: FSHARP_HIGHLIGHTS_QUERY,
        injections_query: FSHARP_INJECTIONS_QUERY,
        locals_query: FSHARP_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "fsharp_signature",
        language_fn: FSHARP_SIGNATURE_LANGUAGE,
        highlights_query: FSHARP_SIGNATURE_HIGHLIGHTS_QUERY,
        injections_query: FSHARP_SIGNATURE_INJECTIONS_QUERY,
        locals_query: FSHARP_SIGNATURE_LOCALS_QUERY,
    },
    StaticLanguageAsset {
        name: "fstab",
        language_fn: FSTAB_LANGUAGE,
        highlights_query: FSTAB_HIGHLIGHTS_QUERY,
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
        name: "todotxt",
        language_fn: TODOTXT_LANGUAGE,
        highlights_query: TODOTXT_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "vhdl",
        language_fn: VHDL_LANGUAGE,
        highlights_query: VHDL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "verilog",
        language_fn: VERILOG_LANGUAGE,
        highlights_query: VERILOG_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "systemverilog",
        language_fn: SYSTEMVERILOG_LANGUAGE,
        highlights_query: SYSTEMVERILOG_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "vim",
        language_fn: VIM_LANGUAGE_PLACEHOLDER,
        highlights_query: VIM_HIGHLIGHTS_QUERY,
        injections_query: VIM_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "vimhelp",
        language_fn: VIMHELP_LANGUAGE,
        highlights_query: VIMHELP_HIGHLIGHTS_QUERY,
        injections_query: VIMHELP_INJECTIONS_QUERY,
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
        name: "latex",
        language_fn: LATEX_LANGUAGE,
        highlights_query: LATEX_HIGHLIGHTS_QUERY,
        injections_query: LATEX_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "tcl",
        language_fn: TCL_LANGUAGE,
        highlights_query: TCL_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "textile",
        language_fn: TEXTILE_LANGUAGE,
        highlights_query: TEXTILE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "csv",
        language_fn: CSV_LANGUAGE,
        highlights_query: CSV_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "tsv",
        language_fn: TSV_LANGUAGE,
        highlights_query: TSV_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "typst",
        language_fn: TYPST_LANGUAGE,
        highlights_query: TYPST_HIGHLIGHTS_QUERY,
        injections_query: TYPST_INJECTIONS_QUERY,
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "strace",
        language_fn: STRACE_LANGUAGE,
        highlights_query: STRACE_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "stylus",
        language_fn: STYLUS_LANGUAGE,
        highlights_query: STYLUS_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "syslog",
        language_fn: SYSLOG_LANGUAGE,
        highlights_query: SYSLOG_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "sml",
        language_fn: SML_LANGUAGE,
        highlights_query: SML_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "solidity",
        language_fn: SOLIDITY_LANGUAGE,
        highlights_query: SOLIDITY_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "varlink",
        language_fn: VARLINK_LANGUAGE,
        highlights_query: VARLINK_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "vyper",
        language_fn: VYPER_LANGUAGE,
        highlights_query: VYPER_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "wgsl",
        language_fn: WGSL_LANGUAGE,
        highlights_query: WGSL_HIGHLIGHTS_QUERY,
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
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "eex",
        language_fn: ERB_LANGUAGE,
        highlights_query: EEX_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "twig",
        language_fn: TWIG_LANGUAGE,
        highlights_query: TWIG_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "erb",
        language_fn: ERB_LANGUAGE,
        highlights_query: ERB_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "jsp",
        language_fn: ERB_LANGUAGE,
        highlights_query: JSP_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "asp",
        language_fn: ERB_LANGUAGE,
        highlights_query: ASP_HIGHLIGHTS_QUERY,
        injections_query: "",
        locals_query: "",
    },
    StaticLanguageAsset {
        name: "adp",
        language_fn: ERB_LANGUAGE,
        highlights_query: ADP_HIGHLIGHTS_QUERY,
        injections_query: "",
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

    let language = match asset.name {
        "vim" => tree_sitter_vim::language(),
        _ => Language::from(asset.language_fn),
    };
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
