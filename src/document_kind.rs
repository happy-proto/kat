use crate::api::canonical_language_name;
use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DocumentProfile {
    Plain,
    PrekConfig,
    MiseConfig,
    HkConfig,
    GitHubActionsWorkflow,
    GitHubActionMetadata,
    GitConfig,
    GitModules,
    FishVariables,
    PythonDocstringAuto,
    PythonDocstringPlain,
    PythonDocstringRest,
    PythonDocstringGoogle,
    PythonDocstringNumpy,
    TemplateHtml,
    TemplateXml,
    TemplateYaml,
    TemplateSql,
    TemplateCss,
    TemplateCoffeeScript,
    TemplateJavaScript,
}

impl DocumentProfile {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::PrekConfig => "prek_config",
            Self::MiseConfig => "mise_config",
            Self::HkConfig => "hk_config",
            Self::GitHubActionsWorkflow => "github_actions_workflow",
            Self::GitHubActionMetadata => "github_action_metadata",
            Self::GitConfig => "git_config",
            Self::GitModules => "git_modules",
            Self::FishVariables => "fish_variables",
            Self::PythonDocstringAuto => "python_docstring_auto",
            Self::PythonDocstringPlain => "python_docstring_plain",
            Self::PythonDocstringRest => "python_docstring_rest",
            Self::PythonDocstringGoogle => "python_docstring_google",
            Self::PythonDocstringNumpy => "python_docstring_numpy",
            Self::TemplateHtml => "template_html",
            Self::TemplateXml => "template_xml",
            Self::TemplateYaml => "template_yaml",
            Self::TemplateSql => "template_sql",
            Self::TemplateCss => "template_css",
            Self::TemplateCoffeeScript => "template_coffeescript",
            Self::TemplateJavaScript => "template_javascript",
        }
    }
}

impl DocumentProfile {
    pub(crate) fn from_query_value(value: &str) -> Option<Self> {
        match value {
            "python_docstring_auto" | "python-docstring-auto" => Some(Self::PythonDocstringAuto),
            "python_docstring_plain" | "python-docstring-plain" => Some(Self::PythonDocstringPlain),
            "python_docstring_rest" | "python-docstring-rest" => Some(Self::PythonDocstringRest),
            "python_docstring_google" | "python-docstring-google" => {
                Some(Self::PythonDocstringGoogle)
            }
            "python_docstring_numpy" | "python-docstring-numpy" => Some(Self::PythonDocstringNumpy),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DocumentKind {
    runtime_name: &'static str,
    profile: DocumentProfile,
}

impl DocumentKind {
    pub(crate) const fn plain(runtime_name: &'static str) -> Self {
        Self {
            runtime_name,
            profile: DocumentProfile::Plain,
        }
    }

    pub(crate) const fn with_profile(runtime_name: &'static str, profile: DocumentProfile) -> Self {
        Self {
            runtime_name,
            profile,
        }
    }

    pub(crate) const fn runtime_name(self) -> &'static str {
        self.runtime_name
    }

    pub(crate) const fn profile(self) -> DocumentProfile {
        self.profile
    }

    pub(crate) fn snapshot(self) -> DocumentKindSnapshot {
        DocumentKindSnapshot {
            runtime_name: self.runtime_name,
            profile: self.profile.name(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DocumentKindSnapshot {
    pub runtime_name: &'static str,
    pub profile: &'static str,
}

pub(crate) fn yaml_document_kind(source_path: Option<&Path>) -> DocumentKind {
    match github_actions_profile(source_path) {
        Some(profile) => DocumentKind::with_profile("yaml", profile),
        None => DocumentKind::plain("yaml"),
    }
}

pub(crate) fn toml_document_kind(source_path: Option<&Path>) -> DocumentKind {
    match toml_profile(source_path) {
        Some(profile) => DocumentKind::with_profile("toml", profile),
        None => DocumentKind::plain("toml"),
    }
}

pub(crate) fn git_config_document_kind(source_path: Option<&Path>) -> DocumentKind {
    match git_config_profile(source_path) {
        Some(profile) => DocumentKind::with_profile("git_config", profile),
        None => DocumentKind::plain("git_config"),
    }
}

pub(crate) fn fish_document_kind(source_path: Option<&Path>) -> DocumentKind {
    match fish_profile(source_path) {
        Some(profile) => DocumentKind::with_profile("fish", profile),
        None => DocumentKind::plain("fish"),
    }
}

pub(crate) fn template_document_kind(
    runtime_name: &'static str,
    source_path: Option<&Path>,
    default_profile: DocumentProfile,
) -> DocumentKind {
    let profile = template_profile(source_path).unwrap_or(default_profile);
    DocumentKind::with_profile(runtime_name, profile)
}

pub(crate) fn github_actions_profile(source_path: Option<&Path>) -> Option<DocumentProfile> {
    let path = source_path?;

    if is_github_actions_workflow_path(path) {
        return Some(DocumentProfile::GitHubActionsWorkflow);
    }

    if is_github_action_metadata_path(path) {
        return Some(DocumentProfile::GitHubActionMetadata);
    }

    None
}

pub(crate) fn toml_profile(source_path: Option<&Path>) -> Option<DocumentProfile> {
    let path = source_path?;
    if crate::mise::is_config_path(path) {
        Some(DocumentProfile::MiseConfig)
    } else {
        is_prek_config_path(path).then_some(DocumentProfile::PrekConfig)
    }
}

pub(crate) fn git_config_profile(source_path: Option<&Path>) -> Option<DocumentProfile> {
    let path = source_path?;

    if matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".gitmodules")
    ) {
        return Some(DocumentProfile::GitModules);
    }

    if is_git_config_path(path) {
        return Some(DocumentProfile::GitConfig);
    }

    None
}

pub(crate) fn template_profile(source_path: Option<&Path>) -> Option<DocumentProfile> {
    let path = source_path?;
    let file_name = path.file_name()?.to_str()?;
    let mut segments = file_name.rsplit('.');
    segments.next()?;
    let host_segment = segments.next()?.to_ascii_lowercase();

    Some(match host_segment.as_str() {
        "html" | "htm" | "shtml" | "xhtml" | "htc" => DocumentProfile::TemplateHtml,
        "xml" | "xsd" | "xslt" | "svg" | "rss" | "opml" | "rng" => DocumentProfile::TemplateXml,
        "yaml" | "yml" => DocumentProfile::TemplateYaml,
        "sql" => DocumentProfile::TemplateSql,
        "css" => DocumentProfile::TemplateCss,
        "coffee" => DocumentProfile::TemplateCoffeeScript,
        "js" | "mjs" | "cjs" | "jsx" | "es6" | "babel" | "pac" => {
            DocumentProfile::TemplateJavaScript
        }
        _ => return None,
    })
}

pub(crate) fn fish_profile(source_path: Option<&Path>) -> Option<DocumentProfile> {
    let path = source_path?;
    is_fish_variables_path(path).then_some(DocumentProfile::FishVariables)
}

pub(crate) fn is_github_actions_workflow_path(path: &Path) -> bool {
    if !matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("yml" | "yaml")
    ) {
        return false;
    }

    let Some(parent) = path.parent() else {
        return false;
    };
    let Some(workflows) = parent.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let Some(github_parent) = parent.parent() else {
        return false;
    };
    let Some(github) = github_parent.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    workflows == "workflows" && github == ".github"
}

pub(crate) fn is_github_action_metadata_path(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("action.yml" | "action.yaml")
    )
}

pub(crate) fn is_prek_config_path(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("prek.toml")
    )
}

pub(crate) fn is_git_config_path(path: &Path) -> bool {
    let components = path
        .iter()
        .filter_map(|component| component.to_str())
        .collect::<Vec<_>>();

    matches!(
        components.as_slice(),
        [.., ".git", "config"] | [.., "git", "config"]
    ) || matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".gitconfig" | "gitconfig" | "config.worktree")
    )
}

pub(crate) fn is_fish_variables_path(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("fish_variables")
    )
}

pub(crate) fn named_document_kind(language_name: &str) -> Result<DocumentKind> {
    Ok(plain_document_kind(canonical_language_name(language_name)?))
}

pub(crate) fn plain_document_kind(language_name: &str) -> DocumentKind {
    match language_name {
        "json" => DocumentKind::plain("json"),
        "query" => DocumentKind::plain("query"),
        "actionscript" => DocumentKind::plain("actionscript"),
        "ada" => DocumentKind::plain("ada"),
        "applescript" => DocumentKind::plain("applescript"),
        "asm" => DocumentKind::plain("asm"),
        "nasm" => DocumentKind::plain("nasm"),
        "asciidoc" => DocumentKind::plain("asciidoc"),
        "authorized_keys" => DocumentKind::plain("authorized_keys"),
        "known_hosts" => DocumentKind::plain("known_hosts"),
        "ghostty" => DocumentKind::plain("ghostty"),
        "pem" => DocumentKind::plain("pem"),
        "openpgp_armor" => DocumentKind::plain("openpgp_armor"),
        "awk" => DocumentKind::plain("awk"),
        "ignore" => DocumentKind::plain("ignore"),
        "git_link" => DocumentKind::plain("git_link"),
        "git_mailmap" => DocumentKind::plain("git_mailmap"),
        "git_log" => DocumentKind::plain("git_log"),
        "git_config" => DocumentKind::plain("git_config"),
        "dockerfile" => DocumentKind::plain("dockerfile"),
        "bash" => DocumentKind::plain("bash"),
        "batch" => DocumentKind::plain("batch"),
        "bibtex" => DocumentKind::plain("bibtex"),
        "cabal" => DocumentKind::plain("cabal"),
        "cfml" => DocumentKind::plain("cfml"),
        "clojure" => DocumentKind::plain("clojure"),
        "cmakecache" => DocumentKind::plain("cmakecache"),
        "coffeescript" => DocumentKind::plain("coffeescript"),
        "command_help" => DocumentKind::plain("command_help"),
        "cpuinfo" => DocumentKind::plain("cpuinfo"),
        "crontab" => DocumentKind::plain("crontab"),
        "crystal" => DocumentKind::plain("crystal"),
        "d" => DocumentKind::plain("d"),
        "debsources" => DocumentKind::plain("debsources"),
        "elm" => DocumentKind::plain("elm"),
        "email" => DocumentKind::plain("email"),
        "erlang" => DocumentKind::plain("erlang"),
        "fish" => DocumentKind::plain("fish"),
        "fortran" => DocumentKind::plain("fortran"),
        "fortran_namelist" => DocumentKind::plain("fortran_namelist"),
        "fsharp" => DocumentKind::plain("fsharp"),
        "fsharp_signature" => DocumentKind::plain("fsharp_signature"),
        "fstab" => DocumentKind::plain("fstab"),
        "powershell" => DocumentKind::plain("powershell"),
        "zsh" => DocumentKind::plain("zsh"),
        "toml" => DocumentKind::plain("toml"),
        "yaml" => DocumentKind::plain("yaml"),
        "hcl" => DocumentKind::plain("hcl"),
        "pkl" => DocumentKind::plain("pkl"),
        "rust" => DocumentKind::plain("rust"),
        "python" => DocumentKind::plain("python"),
        "python_docstring" => {
            DocumentKind::with_profile("python_docstring", DocumentProfile::PythonDocstringAuto)
        }
        "c" => DocumentKind::plain("c"),
        "cpp" => DocumentKind::plain("cpp"),
        "java" => DocumentKind::plain("java"),
        "kotlin" => DocumentKind::plain("kotlin"),
        "csharp" => DocumentKind::plain("csharp"),
        "groovy" => DocumentKind::plain("groovy"),
        "diff" => DocumentKind::plain("diff"),
        "properties" => DocumentKind::plain("properties"),
        "php" => DocumentKind::plain("php"),
        "scala" => DocumentKind::plain("scala"),
        "swift" => DocumentKind::plain("swift"),
        "dart" => DocumentKind::plain("dart"),
        "elixir" => DocumentKind::plain("elixir"),
        "zig" => DocumentKind::plain("zig"),
        "ssh_config" => DocumentKind::plain("ssh_config"),
        "gitattributes" => DocumentKind::plain("gitattributes"),
        "git_commit" => DocumentKind::plain("git_commit"),
        "git_rebase" => DocumentKind::plain("git_rebase"),
        "requirements" => DocumentKind::plain("requirements"),
        "apache" => DocumentKind::plain("apache"),
        "scss" => DocumentKind::plain("scss"),
        "sass" => DocumentKind::plain("sass"),
        "jq" => DocumentKind::plain("jq"),
        "less" => DocumentKind::plain("less"),
        "dot" => DocumentKind::plain("dot"),
        "graph_easy" => DocumentKind::plain("graph_easy"),
        "mermaid" => DocumentKind::plain("mermaid"),
        "nginx" => DocumentKind::plain("nginx"),
        "typescript" => DocumentKind::plain("typescript"),
        "tsx" => DocumentKind::plain("tsx"),
        "go" => DocumentKind::plain("go"),
        "gomod" => DocumentKind::plain("gomod"),
        "gowork" => DocumentKind::plain("gowork"),
        "gosum" => DocumentKind::plain("gosum"),
        "proto" => DocumentKind::plain("proto"),
        "cel" => DocumentKind::plain("cel"),
        "sql" => DocumentKind::plain("sql"),
        "sql_postgres" => DocumentKind::plain("sql_postgres"),
        "sql_mysql" => DocumentKind::plain("sql_mysql"),
        "sql_sqlite" => DocumentKind::plain("sql_sqlite"),
        "textproto" => DocumentKind::plain("textproto"),
        "latex" => DocumentKind::plain("latex"),
        "sml" => DocumentKind::plain("sml"),
        "solidity" => DocumentKind::plain("solidity"),
        "strace" => DocumentKind::plain("strace"),
        "stylus" => DocumentKind::plain("stylus"),
        "syslog" => DocumentKind::plain("syslog"),
        "tcl" => DocumentKind::plain("tcl"),
        "textile" => DocumentKind::plain("textile"),
        "csv" => DocumentKind::plain("csv"),
        "todotxt" => DocumentKind::plain("todotxt"),
        "tsv" => DocumentKind::plain("tsv"),
        "typst" => DocumentKind::plain("typst"),
        "varlink" => DocumentKind::plain("varlink"),
        "html" => DocumentKind::plain("html"),
        "vue" => DocumentKind::plain("vue"),
        "svelte" => DocumentKind::plain("svelte"),
        "css" => DocumentKind::plain("css"),
        "javascript" => DocumentKind::plain("javascript"),
        "graphql" => DocumentKind::plain("graphql"),
        "regex" => DocumentKind::plain("regex"),
        "regex_javascript" => DocumentKind::plain("regex_javascript"),
        "regex_python" => DocumentKind::plain("regex_python"),
        "regex_rust" => DocumentKind::plain("regex_rust"),
        "regex_go" => DocumentKind::plain("regex_go"),
        "regex_posix" => DocumentKind::plain("regex_posix"),
        "jsdoc" => DocumentKind::plain("jsdoc"),
        "userscript_metadata" => DocumentKind::plain("userscript_metadata"),
        "markdown" => DocumentKind::plain("markdown"),
        "markdown_inline" => DocumentKind::plain("markdown_inline"),
        "just" => DocumentKind::plain("just"),
        "ruby" => DocumentKind::plain("ruby"),
        "lua" => DocumentKind::plain("lua"),
        "nix" => DocumentKind::plain("nix"),
        "dotenv" => DocumentKind::plain("dotenv"),
        "ini" => DocumentKind::plain("ini"),
        "xml" => DocumentKind::plain("xml"),
        "make" => DocumentKind::plain("make"),
        "cmake" => DocumentKind::plain("cmake"),
        "ninja" => DocumentKind::plain("ninja"),
        "jinja" => DocumentKind::plain("jinja"),
        "eex" => DocumentKind::plain("eex"),
        "twig" => DocumentKind::plain("twig"),
        "erb" => DocumentKind::plain("erb"),
        "jsp" => DocumentKind::plain("jsp"),
        "asp" => DocumentKind::plain("asp"),
        "adp" => DocumentKind::plain("adp"),
        "vhdl" => DocumentKind::plain("vhdl"),
        "verilog" => DocumentKind::plain("verilog"),
        "systemverilog" => DocumentKind::plain("systemverilog"),
        "vim" => DocumentKind::plain("vim"),
        "vimhelp" => DocumentKind::plain("vimhelp"),
        "vyper" => DocumentKind::plain("vyper"),
        "wgsl" => DocumentKind::plain("wgsl"),
        other => panic!("unsupported runtime name {other}"),
    }
}
