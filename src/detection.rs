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
use crate::language_runtime::RuntimeId;
use crate::python_docstrings::resolve_python_docstring_document_kind;
use crate::sql_dialect::resolve_sql_runtime;
use std::path::Path;

pub(crate) fn detect_document_kind(
    source_path: Option<&Path>,
    source: &str,
) -> Option<DocumentKind> {
    // Ghostty theme names may have arbitrary suffixes, including other
    // languages' extensions. Their explicit directory wins over those hints.
    if matches_path(grammar("ghostty"), source_path) || is_ghostty_config_path(source_path) {
        return Some(DocumentKind::plain(RuntimeId::Ghostty));
    }

    if is_tree_sitter_query_path(source_path) {
        return Some(DocumentKind::plain(RuntimeId::Query));
    }

    if let Some(kind) = detect_registry_languages(&[RuntimeId::Just], source_path, source) {
        return Some(kind);
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
    if let Some(kind) = detect_registry_languages(
        &[
            RuntimeId::Hcl,
            RuntimeId::ActionScript,
            RuntimeId::Ada,
            RuntimeId::AppleScript,
            RuntimeId::Asm,
            RuntimeId::Nasm,
            RuntimeId::AsciiDoc,
        ],
        source_path,
        source,
    ) {
        return Some(kind);
    }

    let authorized_keys = grammar("authorized_keys");
    if matches_path(authorized_keys, source_path)
        || is_authorized_keys_pub_path(source_path, source)
    {
        return Some(DocumentKind::plain(RuntimeId::AuthorizedKeys));
    }

    if let Some(kind) = detect_registry_languages(
        &[RuntimeId::Pgpass, RuntimeId::KnownHosts],
        source_path,
        source,
    ) {
        return Some(kind);
    }

    if is_pem_path(source_path, source) || looks_like_pem(source) {
        return Some(DocumentKind::plain(RuntimeId::Pem));
    }

    if is_openpgp_armor_path(source_path, source) || looks_like_openpgp_armor(source) {
        return Some(DocumentKind::plain(RuntimeId::OpenPgpArmor));
    }

    if let Some(kind) = detect_registry_languages(
        &[
            RuntimeId::Awk,
            RuntimeId::Bibtex,
            RuntimeId::Cabal,
            RuntimeId::Caddyfile,
            RuntimeId::Cfml,
            RuntimeId::Clojure,
            RuntimeId::CmakeCache,
            RuntimeId::CoffeeScript,
            RuntimeId::CommandHelp,
            RuntimeId::CpuInfo,
        ],
        source_path,
        source,
    ) {
        return Some(kind);
    }

    let crontab = grammar("crontab");
    if matches_path(crontab, source_path) || is_cron_d_path(source_path) {
        return Some(DocumentKind::plain(RuntimeId::Crontab));
    }

    if let Some(kind) = detect_registry_languages(
        &[
            RuntimeId::Crystal,
            RuntimeId::D,
            RuntimeId::Debsources,
            RuntimeId::Elm,
        ],
        source_path,
        source,
    ) {
        return Some(kind);
    }

    let email = grammar("email");
    if matches_path(email, source_path) || is_mail_spool_path(source_path) {
        return Some(DocumentKind::plain(RuntimeId::Email));
    }

    if let Some(kind) = detect_registry_languages(
        &[
            RuntimeId::Erlang,
            RuntimeId::Fortran,
            RuntimeId::FortranNamelist,
            RuntimeId::Fsharp,
            RuntimeId::FsharpSignature,
            RuntimeId::Fstab,
        ],
        source_path,
        source,
    ) {
        return Some(kind);
    }

    if let Some(document_kind) = cmake_template_document_kind(source_path) {
        return Some(document_kind);
    }

    if let Some(kind) = detect_registry_languages(
        &[
            RuntimeId::Rust,
            RuntimeId::Python,
            RuntimeId::C,
            RuntimeId::Cpp,
            RuntimeId::Java,
            RuntimeId::Kotlin,
            RuntimeId::Csharp,
            RuntimeId::Groovy,
            RuntimeId::Diff,
            RuntimeId::Properties,
            RuntimeId::Php,
            RuntimeId::Scala,
            RuntimeId::Swift,
            RuntimeId::Dart,
            RuntimeId::Elixir,
            RuntimeId::Zig,
            RuntimeId::Vhdl,
            RuntimeId::Verilog,
            RuntimeId::SystemVerilog,
            RuntimeId::Vim,
            RuntimeId::VimHelp,
        ],
        source_path,
        source,
    ) {
        return Some(kind);
    }

    let ssh_config = grammar("ssh_config");
    if matches_path(ssh_config, source_path) || is_ssh_config_path(source_path) {
        return Some(DocumentKind::plain(RuntimeId::SshConfig));
    }

    let gitattributes = grammar("gitattributes");
    if matches_path(gitattributes, source_path) || is_gitattributes_path(source_path) {
        return Some(DocumentKind::plain(RuntimeId::GitAttributes));
    }

    if let Some(kind) = detect_registry_languages(
        &[
            RuntimeId::GitCommit,
            RuntimeId::GitRebase,
            RuntimeId::Requirements,
            RuntimeId::TodoTxt,
        ],
        source_path,
        source,
    ) {
        return Some(kind);
    }

    let apache = grammar("apache");
    if matches_path(apache, source_path) || is_apache_path(source_path) {
        return Some(DocumentKind::plain(RuntimeId::Apache));
    }

    if let Some(kind) = detect_registry_languages(
        &[
            RuntimeId::Scss,
            RuntimeId::Sass,
            RuntimeId::Jq,
            RuntimeId::Less,
            RuntimeId::GraphEasy,
            RuntimeId::Dot,
            RuntimeId::Mermaid,
        ],
        source_path,
        source,
    ) {
        return Some(kind);
    }

    let nginx = grammar("nginx");
    if matches_path(nginx, source_path) || is_nginx_path(source_path) {
        return Some(DocumentKind::plain(RuntimeId::Nginx));
    }

    if let Some(kind) = detect_registry_languages(
        &[
            RuntimeId::TypeScript,
            RuntimeId::Tsx,
            RuntimeId::Go,
            RuntimeId::GoMod,
            RuntimeId::GoWork,
            RuntimeId::GoSum,
            RuntimeId::Sql,
            RuntimeId::Html,
            RuntimeId::Vue,
            RuntimeId::Svelte,
            RuntimeId::Css,
            RuntimeId::Graphql,
            RuntimeId::Proto,
            RuntimeId::Cel,
            RuntimeId::Textproto,
            RuntimeId::Latex,
            RuntimeId::Sml,
            RuntimeId::Solidity,
            RuntimeId::Strace,
            RuntimeId::Stylus,
            RuntimeId::Syslog,
            RuntimeId::Tcl,
            RuntimeId::Textile,
            RuntimeId::Csv,
            RuntimeId::Tsv,
            RuntimeId::Typst,
            RuntimeId::Varlink,
            RuntimeId::Vyper,
            RuntimeId::Wgsl,
            RuntimeId::JavaScript,
        ],
        source_path,
        source,
    ) {
        return Some(kind);
    }

    let fish = grammar("fish");
    if matches_path(fish, source_path) || matches_shebang(fish, source) {
        return Some(fish_document_kind(source_path));
    }

    if let Some(kind) = detect_registry_languages(
        &[
            RuntimeId::Csh,
            RuntimeId::Zsh,
            RuntimeId::Nushell,
            RuntimeId::Powershell,
            RuntimeId::Batch,
            RuntimeId::Markdown,
            RuntimeId::Ruby,
            RuntimeId::Lua,
            RuntimeId::Nix,
            RuntimeId::Dotenv,
            RuntimeId::Ini,
            RuntimeId::Xml,
            RuntimeId::Jam,
            RuntimeId::Make,
            RuntimeId::Cmake,
            RuntimeId::Ninja,
        ],
        source_path,
        source,
    ) {
        return Some(kind);
    }

    let jinja = grammar("jinja");
    if matches_path(jinja, source_path) {
        let default_profile = if is_sls_path(source_path) {
            DocumentProfile::TemplateYaml
        } else {
            DocumentProfile::TemplateHtml
        };
        return Some(template_document_kind(
            RuntimeId::Jinja,
            source_path,
            default_profile,
        ));
    }

    let eex = grammar("eex");
    if matches_path(eex, source_path) {
        return Some(template_document_kind(
            RuntimeId::Eex,
            source_path,
            DocumentProfile::TemplateHtml,
        ));
    }

    let twig = grammar("twig");
    if matches_path(twig, source_path) {
        return Some(template_document_kind(
            RuntimeId::Twig,
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
        return Some(template_document_kind(
            RuntimeId::Erb,
            source_path,
            default_profile,
        ));
    }

    let jsp = grammar("jsp");
    if matches_path(jsp, source_path) {
        return Some(template_document_kind(
            RuntimeId::Jsp,
            source_path,
            DocumentProfile::TemplateHtml,
        ));
    }

    let asp = grammar("asp");
    if matches_path(asp, source_path) {
        return Some(template_document_kind(
            RuntimeId::Asp,
            source_path,
            DocumentProfile::TemplateHtml,
        ));
    }

    let adp = grammar("adp");
    if matches_path(adp, source_path) {
        return Some(template_document_kind(
            RuntimeId::Adp,
            source_path,
            DocumentProfile::TemplateHtml,
        ));
    }

    let json = grammar("json");
    if matches_path(json, source_path) || looks_like_json(source) {
        return Some(DocumentKind::plain(RuntimeId::Json));
    }

    let ignore = grammar("ignore");
    if matches_path(ignore, source_path) || is_ignore_path(source_path) {
        return Some(DocumentKind::plain(RuntimeId::Ignore));
    }

    let git_link = grammar("git_link");
    if matches_path(git_link, source_path) || looks_like_git_link(source) {
        return Some(DocumentKind::plain(RuntimeId::GitLink));
    }

    if let Some(kind) = detect_registry_languages(&[RuntimeId::GitMailmap], source_path, source) {
        return Some(kind);
    }

    let git_log = grammar("git_log");
    if matches_path(git_log, source_path) || looks_like_git_log(source) {
        return Some(DocumentKind::plain(RuntimeId::GitLog));
    }

    if let Some(kind) = detect_registry_languages(&[RuntimeId::Dockerfile], source_path, source) {
        return Some(kind);
    }

    if source_path.is_none() && looks_like_graphql(source) {
        return Some(DocumentKind::plain(RuntimeId::Graphql));
    }

    if source_path.is_none() && looks_like_sql(source) {
        return Some(DocumentKind::plain(RuntimeId::Sql));
    }

    let bash = grammar("bash");
    if matches_path(bash, source_path)
        || matches_shebang(bash, source)
        || is_bat_config_path(source_path)
        || is_shell_profile_path(source_path)
        || is_os_release_path(source_path)
    {
        return Some(DocumentKind::plain(RuntimeId::Bash));
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
        return Some(DocumentKind::plain(RuntimeId::C));
    }

    [".hh.in", ".hpp.in", ".hxx.in", ".h++.in"]
        .iter()
        .any(|suffix| file_name.ends_with(suffix))
        .then_some(DocumentKind::plain(RuntimeId::Cpp))
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
        components.as_slice(),
        [.., ".ssh", "conf.d", _] if path.extension().is_some_and(|extension| extension == "conf")
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
            crate::document_kind::plain_document_kind(resolve_sql_runtime(
                source_path,
                document_kind.runtime_name(),
                source,
            ))
            .runtime_id(),
            document_kind.profile(),
        ),
        _ => document_kind,
    }
}

fn detect_registry_languages(
    ids: &[RuntimeId],
    path: Option<&Path>,
    source: &str,
) -> Option<DocumentKind> {
    ids.iter()
        .copied()
        .find(|id| {
            let spec = grammar(id.name());
            matches_path(spec, path) || matches_shebang(spec, source)
        })
        .map(DocumentKind::plain)
}
