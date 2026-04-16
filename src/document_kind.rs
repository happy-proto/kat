use std::path::Path;

use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DocumentProfile {
    Plain,
    PrekConfig,
    GitHubActionsWorkflow,
    GitHubActionMetadata,
    GitConfig,
    GitModules,
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
            Self::GitHubActionsWorkflow => "github_actions_workflow",
            Self::GitHubActionMetadata => "github_action_metadata",
            Self::GitConfig => "git_config",
            Self::GitModules => "git_modules",
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

pub(crate) fn template_document_kind(
    runtime_name: &'static str,
    source_path: Option<&Path>,
    default_profile: DocumentProfile,
) -> DocumentKind {
    let profile = template_profile(source_path).unwrap_or(default_profile);
    DocumentKind::with_profile(runtime_name, profile)
}

fn github_actions_profile(source_path: Option<&Path>) -> Option<DocumentProfile> {
    let path = source_path?;

    if is_github_actions_workflow_path(path) {
        return Some(DocumentProfile::GitHubActionsWorkflow);
    }

    if is_github_action_metadata_path(path) {
        return Some(DocumentProfile::GitHubActionMetadata);
    }

    None
}

fn toml_profile(source_path: Option<&Path>) -> Option<DocumentProfile> {
    let path = source_path?;
    is_prek_config_path(path).then_some(DocumentProfile::PrekConfig)
}

fn git_config_profile(source_path: Option<&Path>) -> Option<DocumentProfile> {
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

fn template_profile(source_path: Option<&Path>) -> Option<DocumentProfile> {
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

fn is_github_actions_workflow_path(path: &Path) -> bool {
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

fn is_github_action_metadata_path(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("action.yml" | "action.yaml")
    )
}

fn is_prek_config_path(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("prek.toml")
    )
}

fn is_git_config_path(path: &Path) -> bool {
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
