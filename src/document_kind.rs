use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DocumentProfile {
    Plain,
    GitHubActionsWorkflow,
    GitHubActionMetadata,
    GitConfig,
    GitModules,
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
}

pub(crate) fn yaml_document_kind(source_path: Option<&Path>) -> DocumentKind {
    match github_actions_profile(source_path) {
        Some(profile) => DocumentKind::with_profile("yaml", profile),
        None => DocumentKind::plain("yaml"),
    }
}

pub(crate) fn git_config_document_kind(source_path: Option<&Path>) -> DocumentKind {
    match git_config_profile(source_path) {
        Some(profile) => DocumentKind::with_profile("git_config", profile),
        None => DocumentKind::plain("git_config"),
    }
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
