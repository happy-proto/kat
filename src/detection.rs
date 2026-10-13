use crate::armor_formats::is_openpgp_armor_extension;
use crate::armor_formats::is_pem_extension;
use crate::armor_formats::looks_like_openpgp_armor;
use crate::armor_formats::looks_like_pem;
use crate::document_kind::DocumentKind;
use crate::document_kind::DocumentProfile;
use crate::document_kind::fish_document_kind;
use crate::document_kind::git_config_document_kind;
use crate::document_kind::template_document_kind;
use crate::document_kind::toml_document_kind;
use crate::document_kind::yaml_document_kind;
use crate::grammar_registry;
use crate::grammar_registry::grammar;
use crate::language_aliases::normalize_language_name;
use crate::language_aliases::shebang_interpreter_name;
use crate::python_docstrings::resolve_python_docstring_document_kind;
use crate::sql_dialect::resolve_sql_runtime;
use std::path::Path;

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SupportedLanguage {
    ActionScript,
    Ada,
    Adp,
    AppleScript,
    AsciiDoc,
    Asp,
    Asm,
    AuthorizedKeys,
    KnownHosts,
    Ghostty,
    OpenPgpArmor,
    Awk,
    Bash,
    Batch,
    Bibtex,
    Cabal,
    C,
    Cel,
    Cfml,
    Csharp,
    Clojure,
    CmakeCache,
    Cmake,
    CoffeeScript,
    CommandHelp,
    CpuInfo,
    Crystal,
    Crontab,
    Csv,
    Css,
    Cpp,
    Apache,
    D,
    Debsources,
    Diff,
    Dockerfile,
    Dot,
    GraphEasy,
    Dotenv,
    Dart,
    Eex,
    Erb,
    Elixir,
    Fish,
    Fortran,
    FortranNamelist,
    Fsharp,
    Fstab,
    GitAttributes,
    GitCommit,
    GitConfig,
    GitLink,
    GitLog,
    GitMailmap,
    GitRebase,
    Go,
    GoMod,
    GoSum,
    GoWork,
    Graphql,
    Pkl,
    Groovy,
    Hcl,
    Html,
    Ignore,
    Ini,
    Java,
    Jinja,
    JavaScript,
    Jq,
    Just,
    Json,
    Jsp,
    Kotlin,
    Latex,
    Less,
    Lua,
    Make,
    Markdown,
    Mermaid,
    Nasm,
    Nginx,
    Ninja,
    Nix,
    Pem,
    Php,
    Properties,
    Proto,
    Powershell,
    Python,
    Query,
    Requirements,
    Ruby,
    Sass,
    Scala,
    Scss,
    Sml,
    Solidity,
    SshConfig,
    Sql,
    Rust,
    Strace,
    Stylus,
    Svelte,
    Swift,
    Syslog,
    SystemVerilog,
    Tcl,
    Textile,
    Textproto,
    TodoTxt,
    Toml,
    Tsv,
    Tsx,
    TypeScript,
    Twig,
    Typst,
    Varlink,
    Verilog,
    Vue,
    Vhdl,
    Vim,
    VimHelp,
    Vyper,
    Wgsl,
    Xml,
    Yaml,
    Zig,
    Zsh,
    Elm,
    Email,
    Erlang,
}

#[cfg(test)]
pub(crate) fn detect_language(
    source_path: Option<&Path>,
    source: &str,
) -> Option<SupportedLanguage> {
    let document_kind = detect_document_kind(source_path, source)?;
    Some(match document_kind.runtime_name() {
        "json" => SupportedLanguage::Json,
        "query" => SupportedLanguage::Query,
        "actionscript" => SupportedLanguage::ActionScript,
        "ada" => SupportedLanguage::Ada,
        "applescript" => SupportedLanguage::AppleScript,
        "asm" => SupportedLanguage::Asm,
        "nasm" => SupportedLanguage::Nasm,
        "asciidoc" => SupportedLanguage::AsciiDoc,
        "authorized_keys" => SupportedLanguage::AuthorizedKeys,
        "known_hosts" => SupportedLanguage::KnownHosts,
        "ghostty" => SupportedLanguage::Ghostty,
        "pem" => SupportedLanguage::Pem,
        "openpgp_armor" => SupportedLanguage::OpenPgpArmor,
        "awk" => SupportedLanguage::Awk,
        "bash" => SupportedLanguage::Bash,
        "batch" => SupportedLanguage::Batch,
        "bibtex" => SupportedLanguage::Bibtex,
        "cabal" => SupportedLanguage::Cabal,
        "cfml" => SupportedLanguage::Cfml,
        "clojure" => SupportedLanguage::Clojure,
        "cmakecache" => SupportedLanguage::CmakeCache,
        "coffeescript" => SupportedLanguage::CoffeeScript,
        "command_help" => SupportedLanguage::CommandHelp,
        "cpuinfo" => SupportedLanguage::CpuInfo,
        "crontab" => SupportedLanguage::Crontab,
        "crystal" => SupportedLanguage::Crystal,
        "d" => SupportedLanguage::D,
        "debsources" => SupportedLanguage::Debsources,
        "elm" => SupportedLanguage::Elm,
        "email" => SupportedLanguage::Email,
        "erlang" => SupportedLanguage::Erlang,
        "dockerfile" => SupportedLanguage::Dockerfile,
        "fish" => SupportedLanguage::Fish,
        "fortran" => SupportedLanguage::Fortran,
        "fortran_namelist" => SupportedLanguage::FortranNamelist,
        "fsharp" | "fsharp_signature" => SupportedLanguage::Fsharp,
        "fstab" => SupportedLanguage::Fstab,
        "git_link" => SupportedLanguage::GitLink,
        "git_log" => SupportedLanguage::GitLog,
        "git_mailmap" => SupportedLanguage::GitMailmap,
        "git_config" => SupportedLanguage::GitConfig,
        "zsh" => SupportedLanguage::Zsh,
        "toml" => SupportedLanguage::Toml,
        "yaml" => SupportedLanguage::Yaml,
        "hcl" => SupportedLanguage::Hcl,
        "pkl" => SupportedLanguage::Pkl,
        "rust" => SupportedLanguage::Rust,
        "python" => SupportedLanguage::Python,
        "c" => SupportedLanguage::C,
        "cel" => SupportedLanguage::Cel,
        "cpp" => SupportedLanguage::Cpp,
        "java" => SupportedLanguage::Java,
        "kotlin" => SupportedLanguage::Kotlin,
        "csharp" => SupportedLanguage::Csharp,
        "groovy" => SupportedLanguage::Groovy,
        "diff" => SupportedLanguage::Diff,
        "properties" => SupportedLanguage::Properties,
        "php" => SupportedLanguage::Php,
        "scala" => SupportedLanguage::Scala,
        "swift" => SupportedLanguage::Swift,
        "dart" => SupportedLanguage::Dart,
        "elixir" => SupportedLanguage::Elixir,
        "zig" => SupportedLanguage::Zig,
        "ssh_config" => SupportedLanguage::SshConfig,
        "gitattributes" => SupportedLanguage::GitAttributes,
        "git_commit" => SupportedLanguage::GitCommit,
        "git_rebase" => SupportedLanguage::GitRebase,
        "requirements" => SupportedLanguage::Requirements,
        "apache" => SupportedLanguage::Apache,
        "adp" => SupportedLanguage::Adp,
        "asp" => SupportedLanguage::Asp,
        "scss" => SupportedLanguage::Scss,
        "sass" => SupportedLanguage::Sass,
        "jq" => SupportedLanguage::Jq,
        "less" => SupportedLanguage::Less,
        "dot" => SupportedLanguage::Dot,
        "graph_easy" => SupportedLanguage::GraphEasy,
        "mermaid" => SupportedLanguage::Mermaid,
        "nginx" => SupportedLanguage::Nginx,
        "typescript" => SupportedLanguage::TypeScript,
        "tsx" => SupportedLanguage::Tsx,
        "go" => SupportedLanguage::Go,
        "gomod" => SupportedLanguage::GoMod,
        "gowork" => SupportedLanguage::GoWork,
        "gosum" => SupportedLanguage::GoSum,
        "graphql" => SupportedLanguage::Graphql,
        "sql" => SupportedLanguage::Sql,
        "textproto" => SupportedLanguage::Textproto,
        "todotxt" => SupportedLanguage::TodoTxt,
        "html" => SupportedLanguage::Html,
        "vue" => SupportedLanguage::Vue,
        "svelte" => SupportedLanguage::Svelte,
        "csv" => SupportedLanguage::Csv,
        "ignore" => SupportedLanguage::Ignore,
        "css" => SupportedLanguage::Css,
        "javascript" => SupportedLanguage::JavaScript,
        "markdown" => SupportedLanguage::Markdown,
        "proto" => SupportedLanguage::Proto,
        "powershell" => SupportedLanguage::Powershell,
        "just" => SupportedLanguage::Just,
        "ruby" => SupportedLanguage::Ruby,
        "lua" => SupportedLanguage::Lua,
        "nix" => SupportedLanguage::Nix,
        "dotenv" => SupportedLanguage::Dotenv,
        "ini" => SupportedLanguage::Ini,
        "xml" => SupportedLanguage::Xml,
        "make" => SupportedLanguage::Make,
        "cmake" => SupportedLanguage::Cmake,
        "ninja" => SupportedLanguage::Ninja,
        "jinja" => SupportedLanguage::Jinja,
        "twig" => SupportedLanguage::Twig,
        "eex" => SupportedLanguage::Eex,
        "erb" => SupportedLanguage::Erb,
        "jsp" => SupportedLanguage::Jsp,
        "latex" => SupportedLanguage::Latex,
        "sml" => SupportedLanguage::Sml,
        "solidity" => SupportedLanguage::Solidity,
        "strace" => SupportedLanguage::Strace,
        "stylus" => SupportedLanguage::Stylus,
        "syslog" => SupportedLanguage::Syslog,
        "systemverilog" => SupportedLanguage::SystemVerilog,
        "vhdl" => SupportedLanguage::Vhdl,
        "varlink" => SupportedLanguage::Varlink,
        "verilog" => SupportedLanguage::Verilog,
        "vim" => SupportedLanguage::Vim,
        "vimhelp" => SupportedLanguage::VimHelp,
        "vyper" => SupportedLanguage::Vyper,
        "wgsl" => SupportedLanguage::Wgsl,
        "tcl" => SupportedLanguage::Tcl,
        "textile" => SupportedLanguage::Textile,
        "tsv" => SupportedLanguage::Tsv,
        "typst" => SupportedLanguage::Typst,
        other => panic!("unsupported test language mapping for {other}"),
    })
}

pub(crate) fn detect_document_kind(
    source_path: Option<&Path>,
    source: &str,
) -> Option<DocumentKind> {
    // Ghostty theme names may have arbitrary suffixes, including other
    // languages' extensions. Their explicit directory wins over those hints.
    if matches_path(grammar("ghostty"), source_path) || is_ghostty_config_path(source_path) {
        return Some(DocumentKind::plain("ghostty"));
    }

    if is_tree_sitter_query_path(source_path) {
        return Some(DocumentKind::plain("query"));
    }

    let just = grammar("just");
    if matches_path(just, source_path) {
        return Some(DocumentKind::plain("just"));
    }

    let toml = grammar("toml");
    if matches_path(toml, source_path) {
        return Some(toml_document_kind(source_path));
    }

    let pkl = grammar("pkl");
    if matches_path(pkl, source_path) || matches_shebang(pkl, source) {
        return Some(crate::pkl::document_kind(source_path, source));
    }

    let yaml = grammar("yaml");
    if matches_path(yaml, source_path) {
        return Some(yaml_document_kind(source_path));
    }

    let git_config = grammar("git_config");
    if matches_path(git_config, source_path) || is_git_config_path(source_path) {
        return Some(git_config_document_kind(source_path));
    }

    // TODO: Keep HCL as one generic runtime for now. If Terraform/tfvars or
    // Nomad later need different file detection or semantics, split that at the
    // detector/runtime-overlay layer instead of fragmenting the base grammar.
    let hcl = grammar("hcl");
    if matches_path(hcl, source_path) {
        return Some(DocumentKind::plain("hcl"));
    }

    let actionscript = grammar("actionscript");
    if matches_path(actionscript, source_path) {
        return Some(DocumentKind::plain("actionscript"));
    }

    let ada = grammar("ada");
    if matches_path(ada, source_path) {
        return Some(DocumentKind::plain("ada"));
    }

    let applescript = grammar("applescript");
    if matches_path(applescript, source_path) || matches_shebang(applescript, source) {
        return Some(DocumentKind::plain("applescript"));
    }

    let asm = grammar("asm");
    if matches_path(asm, source_path) {
        return Some(DocumentKind::plain("asm"));
    }

    let nasm = grammar("nasm");
    if matches_path(nasm, source_path) {
        return Some(DocumentKind::plain("nasm"));
    }

    let asciidoc = grammar("asciidoc");
    if matches_path(asciidoc, source_path) {
        return Some(DocumentKind::plain("asciidoc"));
    }

    let authorized_keys = grammar("authorized_keys");
    if matches_path(authorized_keys, source_path)
        || is_authorized_keys_pub_path(source_path, source)
    {
        return Some(DocumentKind::plain("authorized_keys"));
    }

    if matches_path(grammar("known_hosts"), source_path) {
        return Some(DocumentKind::plain("known_hosts"));
    }

    if is_pem_path(source_path, source) || looks_like_pem(source) {
        return Some(DocumentKind::plain("pem"));
    }

    if is_openpgp_armor_path(source_path, source) || looks_like_openpgp_armor(source) {
        return Some(DocumentKind::plain("openpgp_armor"));
    }

    let awk = grammar("awk");
    if matches_path(awk, source_path) || matches_shebang(awk, source) {
        return Some(DocumentKind::plain("awk"));
    }

    let bibtex = grammar("bibtex");
    if matches_path(bibtex, source_path) {
        return Some(DocumentKind::plain("bibtex"));
    }

    let cabal = grammar("cabal");
    if matches_path(cabal, source_path) {
        return Some(DocumentKind::plain("cabal"));
    }

    let cfml = grammar("cfml");
    if matches_path(cfml, source_path) {
        return Some(DocumentKind::plain("cfml"));
    }

    let clojure = grammar("clojure");
    if matches_path(clojure, source_path) {
        return Some(DocumentKind::plain("clojure"));
    }

    let cmakecache = grammar("cmakecache");
    if matches_path(cmakecache, source_path) {
        return Some(DocumentKind::plain("cmakecache"));
    }

    let coffeescript = grammar("coffeescript");
    if matches_path(coffeescript, source_path) {
        return Some(DocumentKind::plain("coffeescript"));
    }

    let command_help = grammar("command_help");
    if matches_path(command_help, source_path) {
        return Some(DocumentKind::plain("command_help"));
    }

    let cpuinfo = grammar("cpuinfo");
    if matches_path(cpuinfo, source_path) {
        return Some(DocumentKind::plain("cpuinfo"));
    }

    let crontab = grammar("crontab");
    if matches_path(crontab, source_path) || is_cron_d_path(source_path) {
        return Some(DocumentKind::plain("crontab"));
    }

    let crystal = grammar("crystal");
    if matches_path(crystal, source_path) {
        return Some(DocumentKind::plain("crystal"));
    }

    let d = grammar("d");
    if matches_path(d, source_path) {
        return Some(DocumentKind::plain("d"));
    }

    let debsources = grammar("debsources");
    if matches_path(debsources, source_path) {
        return Some(DocumentKind::plain("debsources"));
    }

    let elm = grammar("elm");
    if matches_path(elm, source_path) {
        return Some(DocumentKind::plain("elm"));
    }

    let email = grammar("email");
    if matches_path(email, source_path) || is_mail_spool_path(source_path) {
        return Some(DocumentKind::plain("email"));
    }

    let erlang = grammar("erlang");
    if matches_path(erlang, source_path) || matches_shebang(erlang, source) {
        return Some(DocumentKind::plain("erlang"));
    }

    let fortran = grammar("fortran");
    if matches_path(fortran, source_path) {
        return Some(DocumentKind::plain("fortran"));
    }

    let fortran_namelist = grammar("fortran_namelist");
    if matches_path(fortran_namelist, source_path) {
        return Some(DocumentKind::plain("fortran_namelist"));
    }

    let fsharp_signature = grammar("fsharp_signature");
    if matches_path(fsharp_signature, source_path) {
        return Some(DocumentKind::plain("fsharp_signature"));
    }

    let fsharp = grammar("fsharp");
    if matches_path(fsharp, source_path) {
        return Some(DocumentKind::plain("fsharp"));
    }

    let fstab = grammar("fstab");
    if matches_path(fstab, source_path) {
        return Some(DocumentKind::plain("fstab"));
    }

    if let Some(document_kind) = cmake_template_document_kind(source_path) {
        return Some(document_kind);
    }

    let rust = grammar("rust");
    if matches_path(rust, source_path) {
        return Some(DocumentKind::plain("rust"));
    }

    let python = grammar("python");
    if matches_path(python, source_path) {
        return Some(DocumentKind::plain("python"));
    }

    let c = grammar("c");
    if matches_path(c, source_path) {
        return Some(DocumentKind::plain("c"));
    }

    let cpp = grammar("cpp");
    if matches_path(cpp, source_path) {
        return Some(DocumentKind::plain("cpp"));
    }

    let java = grammar("java");
    if matches_path(java, source_path) {
        return Some(DocumentKind::plain("java"));
    }

    let kotlin = grammar("kotlin");
    if matches_path(kotlin, source_path) {
        return Some(DocumentKind::plain("kotlin"));
    }

    let csharp = grammar("csharp");
    if matches_path(csharp, source_path) {
        return Some(DocumentKind::plain("csharp"));
    }

    let groovy = grammar("groovy");
    if matches_path(groovy, source_path) || matches_shebang(groovy, source) {
        return Some(DocumentKind::plain("groovy"));
    }

    let diff = grammar("diff");
    if matches_path(diff, source_path) {
        return Some(DocumentKind::plain("diff"));
    }

    let properties = grammar("properties");
    if matches_path(properties, source_path) {
        return Some(DocumentKind::plain("properties"));
    }

    let php = grammar("php");
    if matches_path(php, source_path) || matches_shebang(php, source) {
        return Some(DocumentKind::plain("php"));
    }

    let scala = grammar("scala");
    if matches_path(scala, source_path) {
        return Some(DocumentKind::plain("scala"));
    }

    let swift = grammar("swift");
    if matches_path(swift, source_path) || matches_shebang(swift, source) {
        return Some(DocumentKind::plain("swift"));
    }

    let dart = grammar("dart");
    if matches_path(dart, source_path) {
        return Some(DocumentKind::plain("dart"));
    }

    let elixir = grammar("elixir");
    if matches_path(elixir, source_path) || matches_shebang(elixir, source) {
        return Some(DocumentKind::plain("elixir"));
    }

    let zig = grammar("zig");
    if matches_path(zig, source_path) {
        return Some(DocumentKind::plain("zig"));
    }

    let vhdl = grammar("vhdl");
    if matches_path(vhdl, source_path) {
        return Some(DocumentKind::plain("vhdl"));
    }

    let verilog = grammar("verilog");
    if matches_path(verilog, source_path) {
        return Some(DocumentKind::plain("verilog"));
    }

    let systemverilog = grammar("systemverilog");
    if matches_path(systemverilog, source_path) {
        return Some(DocumentKind::plain("systemverilog"));
    }

    let vim = grammar("vim");
    if matches_path(vim, source_path) {
        return Some(DocumentKind::plain("vim"));
    }

    let vimhelp = grammar("vimhelp");
    if matches_path(vimhelp, source_path) {
        return Some(DocumentKind::plain("vimhelp"));
    }

    let ssh_config = grammar("ssh_config");
    if matches_path(ssh_config, source_path) || is_ssh_config_path(source_path) {
        return Some(DocumentKind::plain("ssh_config"));
    }

    let gitattributes = grammar("gitattributes");
    if matches_path(gitattributes, source_path) || is_gitattributes_path(source_path) {
        return Some(DocumentKind::plain("gitattributes"));
    }

    let git_commit = grammar("git_commit");
    if matches_path(git_commit, source_path) {
        return Some(DocumentKind::plain("git_commit"));
    }

    let git_rebase = grammar("git_rebase");
    if matches_path(git_rebase, source_path) {
        return Some(DocumentKind::plain("git_rebase"));
    }

    let requirements = grammar("requirements");
    if matches_path(requirements, source_path) {
        return Some(DocumentKind::plain("requirements"));
    }

    let todotxt = grammar("todotxt");
    if matches_path(todotxt, source_path) {
        return Some(DocumentKind::plain("todotxt"));
    }

    let apache = grammar("apache");
    if matches_path(apache, source_path) || is_apache_path(source_path) {
        return Some(DocumentKind::plain("apache"));
    }

    let scss = grammar("scss");
    if matches_path(scss, source_path) {
        return Some(DocumentKind::plain("scss"));
    }

    let sass = grammar("sass");
    if matches_path(sass, source_path) {
        return Some(DocumentKind::plain("sass"));
    }

    let jq = grammar("jq");
    if matches_path(jq, source_path) {
        return Some(DocumentKind::plain("jq"));
    }

    let less = grammar("less");
    if matches_path(less, source_path) {
        return Some(DocumentKind::plain("less"));
    }

    if matches_path(grammar("graph_easy"), source_path) {
        return Some(DocumentKind::plain("graph_easy"));
    }

    let dot = grammar("dot");
    if matches_path(dot, source_path) {
        return Some(DocumentKind::plain("dot"));
    }

    if matches_path(grammar("mermaid"), source_path) {
        return Some(DocumentKind::plain("mermaid"));
    }

    let nginx = grammar("nginx");
    if matches_path(nginx, source_path) || is_nginx_path(source_path) {
        return Some(DocumentKind::plain("nginx"));
    }

    let typescript = grammar("typescript");
    if matches_path(typescript, source_path) {
        return Some(DocumentKind::plain("typescript"));
    }

    let tsx = grammar("tsx");
    if matches_path(tsx, source_path) {
        return Some(DocumentKind::plain("tsx"));
    }

    let go = grammar("go");
    if matches_path(go, source_path) {
        return Some(DocumentKind::plain("go"));
    }

    let gomod = grammar("gomod");
    if matches_path(gomod, source_path) {
        return Some(DocumentKind::plain("gomod"));
    }

    let gowork = grammar("gowork");
    if matches_path(gowork, source_path) {
        return Some(DocumentKind::plain("gowork"));
    }

    let gosum = grammar("gosum");
    if matches_path(gosum, source_path) {
        return Some(DocumentKind::plain("gosum"));
    }

    let sql = grammar("sql");
    if matches_path(sql, source_path) {
        return Some(DocumentKind::plain("sql"));
    }

    let html = grammar("html");
    if matches_path(html, source_path) {
        return Some(DocumentKind::plain("html"));
    }

    let vue = grammar("vue");
    if matches_path(vue, source_path) {
        return Some(DocumentKind::plain("vue"));
    }

    let svelte = grammar("svelte");
    if matches_path(svelte, source_path) {
        return Some(DocumentKind::plain("svelte"));
    }

    let css = grammar("css");
    if matches_path(css, source_path) {
        return Some(DocumentKind::plain("css"));
    }

    let graphql = grammar("graphql");
    if matches_path(graphql, source_path) {
        return Some(DocumentKind::plain("graphql"));
    }

    let proto = grammar("proto");
    if matches_path(proto, source_path) {
        return Some(DocumentKind::plain("proto"));
    }

    let cel = grammar("cel");
    if matches_path(cel, source_path) {
        return Some(DocumentKind::plain("cel"));
    }

    let textproto = grammar("textproto");
    if matches_path(textproto, source_path) {
        return Some(DocumentKind::plain("textproto"));
    }

    let latex = grammar("latex");
    if matches_path(latex, source_path) {
        return Some(DocumentKind::plain("latex"));
    }

    let sml = grammar("sml");
    if matches_path(sml, source_path) {
        return Some(DocumentKind::plain("sml"));
    }

    let solidity = grammar("solidity");
    if matches_path(solidity, source_path) {
        return Some(DocumentKind::plain("solidity"));
    }

    let strace = grammar("strace");
    if matches_path(strace, source_path) {
        return Some(DocumentKind::plain("strace"));
    }

    let stylus = grammar("stylus");
    if matches_path(stylus, source_path) {
        return Some(DocumentKind::plain("stylus"));
    }

    let syslog = grammar("syslog");
    if matches_path(syslog, source_path) {
        return Some(DocumentKind::plain("syslog"));
    }

    let tcl = grammar("tcl");
    if matches_path(tcl, source_path) || matches_shebang(tcl, source) {
        return Some(DocumentKind::plain("tcl"));
    }

    let textile = grammar("textile");
    if matches_path(textile, source_path) {
        return Some(DocumentKind::plain("textile"));
    }

    let csv = grammar("csv");
    if matches_path(csv, source_path) {
        return Some(DocumentKind::plain("csv"));
    }

    let tsv = grammar("tsv");
    if matches_path(tsv, source_path) {
        return Some(DocumentKind::plain("tsv"));
    }

    let typst = grammar("typst");
    if matches_path(typst, source_path) {
        return Some(DocumentKind::plain("typst"));
    }

    let varlink = grammar("varlink");
    if matches_path(varlink, source_path) {
        return Some(DocumentKind::plain("varlink"));
    }

    let vyper = grammar("vyper");
    if matches_path(vyper, source_path) {
        return Some(DocumentKind::plain("vyper"));
    }

    let wgsl = grammar("wgsl");
    if matches_path(wgsl, source_path) {
        return Some(DocumentKind::plain("wgsl"));
    }

    let javascript = grammar("javascript");
    if matches_path(javascript, source_path) || matches_shebang(javascript, source) {
        return Some(DocumentKind::plain("javascript"));
    }

    let fish = grammar("fish");
    if matches_path(fish, source_path) || matches_shebang(fish, source) {
        return Some(fish_document_kind(source_path));
    }

    let zsh = grammar("zsh");
    if matches_path(zsh, source_path) || matches_shebang(zsh, source) {
        return Some(DocumentKind::plain("zsh"));
    }

    let powershell = grammar("powershell");
    if matches_path(powershell, source_path) || matches_shebang(powershell, source) {
        return Some(DocumentKind::plain("powershell"));
    }

    let batch = grammar("batch");
    if matches_path(batch, source_path) {
        return Some(DocumentKind::plain("batch"));
    }

    let markdown = grammar("markdown");
    if matches_path(markdown, source_path) {
        return Some(DocumentKind::plain("markdown"));
    }

    let ruby = grammar("ruby");
    if matches_path(ruby, source_path) || matches_shebang(ruby, source) {
        return Some(DocumentKind::plain("ruby"));
    }

    let lua = grammar("lua");
    if matches_path(lua, source_path) || matches_shebang(lua, source) {
        return Some(DocumentKind::plain("lua"));
    }

    let nix = grammar("nix");
    if matches_path(nix, source_path) {
        return Some(DocumentKind::plain("nix"));
    }

    let dotenv = grammar("dotenv");
    if matches_path(dotenv, source_path) {
        return Some(DocumentKind::plain("dotenv"));
    }

    let ini = grammar("ini");
    if matches_path(ini, source_path) {
        return Some(DocumentKind::plain("ini"));
    }

    let xml = grammar("xml");
    if matches_path(xml, source_path) {
        return Some(DocumentKind::plain("xml"));
    }

    let make = grammar("make");
    if matches_path(make, source_path) {
        return Some(DocumentKind::plain("make"));
    }

    let cmake = grammar("cmake");
    if matches_path(cmake, source_path) {
        return Some(DocumentKind::plain("cmake"));
    }

    let ninja = grammar("ninja");
    if matches_path(ninja, source_path) {
        return Some(DocumentKind::plain("ninja"));
    }

    let jinja = grammar("jinja");
    if matches_path(jinja, source_path) {
        let default_profile = if is_sls_path(source_path) {
            DocumentProfile::TemplateYaml
        } else {
            DocumentProfile::TemplateHtml
        };
        return Some(template_document_kind(
            "jinja",
            source_path,
            default_profile,
        ));
    }

    let eex = grammar("eex");
    if matches_path(eex, source_path) {
        return Some(template_document_kind(
            "eex",
            source_path,
            DocumentProfile::TemplateHtml,
        ));
    }

    let twig = grammar("twig");
    if matches_path(twig, source_path) {
        return Some(template_document_kind(
            "twig",
            source_path,
            DocumentProfile::TemplateHtml,
        ));
    }

    let erb = grammar("erb");
    if matches_path(erb, source_path) {
        let default_profile = if is_sql_erb_path(source_path) {
            DocumentProfile::TemplateSql
        } else {
            DocumentProfile::TemplateHtml
        };
        return Some(template_document_kind("erb", source_path, default_profile));
    }

    let jsp = grammar("jsp");
    if matches_path(jsp, source_path) {
        return Some(template_document_kind(
            "jsp",
            source_path,
            DocumentProfile::TemplateHtml,
        ));
    }

    let asp = grammar("asp");
    if matches_path(asp, source_path) {
        return Some(template_document_kind(
            "asp",
            source_path,
            DocumentProfile::TemplateHtml,
        ));
    }

    let adp = grammar("adp");
    if matches_path(adp, source_path) {
        return Some(template_document_kind(
            "adp",
            source_path,
            DocumentProfile::TemplateHtml,
        ));
    }

    let json = grammar("json");
    if matches_path(json, source_path) || looks_like_json(source) {
        return Some(DocumentKind::plain("json"));
    }

    let ignore = grammar("ignore");
    if matches_path(ignore, source_path) || is_ignore_path(source_path) {
        return Some(DocumentKind::plain("ignore"));
    }

    let git_link = grammar("git_link");
    if matches_path(git_link, source_path) || looks_like_git_link(source) {
        return Some(DocumentKind::plain("git_link"));
    }

    let git_mailmap = grammar("git_mailmap");
    if matches_path(git_mailmap, source_path) {
        return Some(DocumentKind::plain("git_mailmap"));
    }

    let git_log = grammar("git_log");
    if matches_path(git_log, source_path) || looks_like_git_log(source) {
        return Some(DocumentKind::plain("git_log"));
    }

    let dockerfile = grammar("dockerfile");
    if matches_path(dockerfile, source_path) {
        return Some(DocumentKind::plain("dockerfile"));
    }

    if source_path.is_none() && looks_like_graphql(source) {
        return Some(DocumentKind::plain("graphql"));
    }

    if source_path.is_none() && looks_like_sql(source) {
        return Some(DocumentKind::plain("sql"));
    }

    let bash = grammar("bash");
    if matches_path(bash, source_path)
        || matches_shebang(bash, source)
        || is_bat_config_path(source_path)
        || is_shell_profile_path(source_path)
        || is_os_release_path(source_path)
    {
        return Some(DocumentKind::plain("bash"));
    }

    None
}

pub(crate) fn is_ghostty_config_path(source_path: Option<&Path>) -> bool {
    let Some(path) = source_path else {
        return false;
    };
    let Some(parent) = path.parent() else {
        return false;
    };
    let parent_name = parent.file_name().and_then(|name| name.to_str());
    (path.file_name().is_some_and(|name| name == "config")
        && matches!(parent_name, Some("ghostty" | "com.mitchellh.ghostty")))
        || (parent_name == Some("themes")
            && parent
                .parent()
                .and_then(Path::file_name)
                .is_some_and(|name| name == "ghostty"))
}

pub(crate) fn matches_path(
    grammar: &grammar_registry::GrammarSpec,
    source_path: Option<&Path>,
) -> bool {
    source_path.is_some_and(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                grammar.filenames.iter().any(|filename| filename == name)
                    || grammar
                        .filename_prefixes
                        .iter()
                        .any(|prefix| name.starts_with(prefix))
            })
            || path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    grammar
                        .extensions
                        .iter()
                        .any(|expected_extension| expected_extension == extension)
                })
    })
}

pub(crate) fn cmake_template_document_kind(source_path: Option<&Path>) -> Option<DocumentKind> {
    let file_name = source_path?.file_name()?.to_str()?.to_ascii_lowercase();

    if file_name.ends_with(".h.in") {
        return Some(DocumentKind::plain("c"));
    }

    [".hh.in", ".hpp.in", ".hxx.in", ".h++.in"]
        .iter()
        .any(|suffix| file_name.ends_with(suffix))
        .then_some(DocumentKind::plain("cpp"))
}

pub(crate) fn is_git_config_path(source_path: Option<&Path>) -> bool {
    let Some(path) = source_path else {
        return false;
    };

    let components = path
        .iter()
        .filter_map(|component| component.to_str())
        .collect::<Vec<_>>();

    matches!(
        components.as_slice(),
        [.., ".git", "config"] | [.., "git", "config"]
    )
}

pub(crate) fn is_tree_sitter_query_path(source_path: Option<&Path>) -> bool {
    let Some(path) = source_path else {
        return false;
    };

    let components = path
        .iter()
        .filter_map(|component| component.to_str())
        .collect::<Vec<_>>();

    components.windows(4).any(|window| {
        matches!(
            window,
            ["grammars", _, "queries", filename] if filename.ends_with(".scm")
        )
    })
}

pub(crate) fn is_cron_d_path(source_path: Option<&Path>) -> bool {
    let Some(path) = source_path else {
        return false;
    };

    let components = path
        .iter()
        .filter_map(|component| component.to_str())
        .collect::<Vec<_>>();

    matches!(components.as_slice(), [.., "cron.d", _])
}

pub(crate) fn is_mail_spool_path(source_path: Option<&Path>) -> bool {
    let Some(path) = source_path else {
        return false;
    };

    let components = path
        .iter()
        .filter_map(|component| component.to_str())
        .collect::<Vec<_>>();

    matches!(
        components.as_slice(),
        [.., "var", "spool", "mail", _] | [.., "var", "mail", _]
    )
}

pub(crate) fn is_ssh_config_path(source_path: Option<&Path>) -> bool {
    let Some(path) = source_path else {
        return false;
    };

    let components = path
        .iter()
        .filter_map(|component| component.to_str())
        .collect::<Vec<_>>();

    matches!(
        components.as_slice(),
        [.., ".ssh", "config"] | [.., "ssh", "sshd_config"]
    ) || matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("sshd_config")
    )
}

pub(crate) fn is_sls_path(source_path: Option<&Path>) -> bool {
    source_path.is_some_and(|path| {
        path.extension().and_then(|extension| extension.to_str()) == Some("sls")
    })
}

pub(crate) fn is_sql_erb_path(source_path: Option<&Path>) -> bool {
    source_path.is_some_and(|path| {
        matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("erbsql")
        ) || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".sql.erb"))
    })
}

pub(crate) fn is_authorized_keys_pub_path(source_path: Option<&Path>, source: &str) -> bool {
    source_path.is_some_and(|path| {
        path.extension().and_then(|extension| extension.to_str()) == Some("pub")
            && looks_like_authorized_keys(source)
    })
}

pub(crate) fn is_pem_path(source_path: Option<&Path>, source: &str) -> bool {
    source_path.is_some_and(|path| {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(is_pem_extension)
            && looks_like_pem(source)
    })
}

pub(crate) fn is_openpgp_armor_path(source_path: Option<&Path>, source: &str) -> bool {
    source_path.is_some_and(|path| {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(is_openpgp_armor_extension)
            && looks_like_openpgp_armor(source)
    })
}

pub(crate) fn is_bat_config_path(source_path: Option<&Path>) -> bool {
    let Some(path) = source_path else {
        return false;
    };

    let components = path
        .iter()
        .filter_map(|component| component.to_str())
        .collect::<Vec<_>>();

    matches!(components.as_slice(), [.., "bat", "config"])
}

pub(crate) fn is_shell_profile_path(source_path: Option<&Path>) -> bool {
    let Some(path) = source_path else {
        return false;
    };

    let components = path
        .iter()
        .filter_map(|component| component.to_str())
        .collect::<Vec<_>>();

    matches!(components.as_slice(), [.., "etc", "profile"])
}

pub(crate) fn is_os_release_path(source_path: Option<&Path>) -> bool {
    let Some(path) = source_path else {
        return false;
    };

    let components = path
        .iter()
        .filter_map(|component| component.to_str())
        .collect::<Vec<_>>();

    matches!(
        components.as_slice(),
        [.., "etc", "os-release"] | [.., "var", "run", "os-release"]
    )
}

pub(crate) fn is_ignore_path(source_path: Option<&Path>) -> bool {
    let Some(path) = source_path else {
        return false;
    };

    let components = path
        .iter()
        .filter_map(|component| component.to_str())
        .collect::<Vec<_>>();

    matches!(
        components.as_slice(),
        [.., ".git", "info", "exclude"] | [.., ".config", "git", "ignore"] | [.., "git", "ignore"]
    )
}

pub(crate) fn is_gitattributes_path(source_path: Option<&Path>) -> bool {
    let Some(path) = source_path else {
        return false;
    };

    let components = path
        .iter()
        .filter_map(|component| component.to_str())
        .collect::<Vec<_>>();

    matches!(
        components.as_slice(),
        [.., ".config", "git", "attributes"] | [.., "git", "attributes"]
    )
}

pub(crate) fn looks_like_git_link(source: &str) -> bool {
    source
        .lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| line.trim_start().starts_with("gitdir:"))
}

pub(crate) fn looks_like_git_log(source: &str) -> bool {
    let Some(first_line) = source.lines().find(|line| !line.trim().is_empty()) else {
        return false;
    };

    if !first_line.starts_with("commit ") {
        return false;
    }

    let mut lines = source.lines().skip_while(|line| line.trim().is_empty());
    let _ = lines.next();
    lines.any(|line| {
        matches!(
            line,
            line if line.starts_with("Author: ")
                || line.starts_with("Commit: ")
                || line.starts_with("Date: ")
                || line.starts_with("diff --git ")
        )
    })
}

pub(crate) fn looks_like_authorized_keys(source: &str) -> bool {
    let mut saw_key = false;

    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if !looks_like_authorized_keys_line(line) {
            return false;
        }

        saw_key = true;
    }

    saw_key
}

pub(crate) fn looks_like_authorized_keys_line(line: &str) -> bool {
    let fields = split_ssh_fields(line);

    match fields.as_slice() {
        [key_type, key_blob, ..] if is_ssh_key_type(key_type) && is_ssh_key_blob(key_blob) => true,
        [options, key_type, key_blob, ..]
            if looks_like_authorized_keys_options(options)
                && is_ssh_key_type(key_type)
                && is_ssh_key_blob(key_blob) =>
        {
            true
        }
        _ => false,
    }
}

pub(crate) fn split_ssh_fields(line: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut start = None;
    let mut in_quotes = false;
    let mut escaped = false;

    for (index, ch) in line.char_indices() {
        if in_quotes {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quotes = false;
            }
            continue;
        }

        if ch == '"' {
            in_quotes = true;
            start.get_or_insert(index);
            continue;
        }

        if ch == ' ' || ch == '\t' {
            if let Some(field_start) = start.take() {
                fields.push(&line[field_start..index]);
            }
            continue;
        }

        start.get_or_insert(index);
    }

    if let Some(field_start) = start {
        fields.push(&line[field_start..]);
    }

    fields
}

pub(crate) fn looks_like_authorized_keys_options(field: &str) -> bool {
    field.contains('=')
        || field.contains(',')
        || matches!(
            field,
            "restrict"
                | "cert-authority"
                | "pty"
                | "port-forwarding"
                | "agent-forwarding"
                | "user-rc"
                | "x11-forwarding"
                | "tunnel"
                | "no-port-forwarding"
                | "no-agent-forwarding"
                | "no-x11-forwarding"
                | "no-pty"
                | "no-user-rc"
        )
}

pub(crate) fn is_ssh_key_type(field: &str) -> bool {
    matches!(
        field,
        "ssh-rsa"
            | "ssh-dss"
            | "ssh-ed25519"
            | "ssh-xmss"
            | "sk-ssh-ed25519@openssh.com"
            | "sk-ecdsa-sha2-nistp256@openssh.com"
    ) || field.starts_with("ecdsa-sha2-")
        || field.starts_with("ssh-ed25519-cert-")
        || field.starts_with("ssh-rsa-cert-")
}

pub(crate) fn is_ssh_key_blob(field: &str) -> bool {
    !field.is_empty()
        && field
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))
}

pub(crate) fn is_apache_path(source_path: Option<&Path>) -> bool {
    let Some(path) = source_path else {
        return false;
    };

    if matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("httpd.conf" | "envvars")
    ) {
        return true;
    }

    let components = path
        .iter()
        .filter_map(|component| component.to_str())
        .collect::<Vec<_>>();

    let has_apache_root = components
        .iter()
        .any(|component| *component == "apache2" || *component == "httpd");
    let has_apache_subdir = components
        .iter()
        .any(|component| *component == "conf" || component.starts_with("sites-"));

    has_apache_root
        && has_apache_subdir
        && matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("conf")
        )
}

pub(crate) fn is_nginx_path(source_path: Option<&Path>) -> bool {
    let Some(path) = source_path else {
        return false;
    };

    if matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("nginx.conf" | "mime.types" | "fastcgi_params" | "scgi_params" | "uwsgi_params")
    ) {
        return true;
    }

    let has_nginx_component = path
        .iter()
        .filter_map(|component| component.to_str())
        .any(|component| component == "nginx" || component.starts_with("sites-"));

    has_nginx_component
        && matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("conf")
        )
}

pub(crate) fn looks_like_json(source: &str) -> bool {
    let trimmed = source.trim_start();
    trimmed.starts_with('{')
}

pub(crate) fn looks_like_sql(source: &str) -> bool {
    let trimmed = source.trim_start();
    let upper = trimmed.to_ascii_uppercase();
    [
        "SELECT ",
        "WITH ",
        "INSERT ",
        "UPDATE ",
        "DELETE ",
        "CREATE ",
        "ALTER ",
        "DROP ",
        "TRUNCATE ",
        "MERGE ",
        "VACUUM",
        "PRAGMA ",
        "EXPLAIN ",
    ]
    .iter()
    .any(|prefix| upper.starts_with(prefix))
}

pub(crate) fn looks_like_graphql(source: &str) -> bool {
    let trimmed = source.trim_start();
    let lower = trimmed.to_ascii_lowercase();
    [
        "query ",
        "mutation ",
        "subscription ",
        "fragment ",
        "schema ",
        "type ",
        "interface ",
        "union ",
        "enum ",
        "input ",
        "directive ",
        "extend ",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}

pub(crate) fn matches_shebang(grammar: &grammar_registry::GrammarSpec, source: &str) -> bool {
    source.lines().next().is_some_and(|line| {
        if !line.starts_with("#!") {
            return false;
        }

        let Some(interpreter) = shebang_interpreter_name(line) else {
            return false;
        };
        let normalized = normalize_language_name(interpreter).unwrap_or(interpreter);

        normalized == grammar.name
            || grammar.shebang_substrings.iter().any(|alias| {
                let alias = alias.as_str();
                normalize_language_name(alias).unwrap_or(alias) == normalized
            })
    })
}

pub(crate) fn resolve_highlight_document_kind(
    document_kind: DocumentKind,
    source_path: Option<&Path>,
    source: &str,
) -> DocumentKind {
    match document_kind.runtime_name() {
        "python_docstring" => resolve_python_docstring_document_kind(document_kind, source),
        "sql" | "sql_postgres" | "sql_mysql" | "sql_sqlite" => DocumentKind::with_profile(
            resolve_sql_runtime(source_path, document_kind.runtime_name(), source),
            document_kind.profile(),
        ),
        _ => document_kind,
    }
}
