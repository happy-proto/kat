use std::{fs, path::Path, sync::LazyLock};

use serde::Deserialize;

const REGISTRY_TOML: &str = include_str!("../grammars/registry.toml");
#[cfg(test)]
const VENDORED_GRAMMAR_EXCEPTIONS_MD: &str = include_str!("../docs/vendor-grammar-exceptions.md");

#[derive(Debug, Deserialize)]
pub struct GrammarRegistry {
    pub grammar: Vec<GrammarSpec>,
}

#[derive(Debug, Deserialize)]
pub struct GrammarSpec {
    pub name: String,
    #[serde(default)]
    pub parser_source: ParserSource,
    pub extensions: Vec<String>,
    pub filenames: Vec<String>,
    #[serde(default)]
    pub filename_prefixes: Vec<String>,
    pub shebang_substrings: Vec<String>,
    pub highlight_names: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ParserSource {
    #[default]
    Vendored,
    Crate,
}

pub static REGISTRY: LazyLock<GrammarRegistry> = LazyLock::new(|| {
    toml::from_str(REGISTRY_TOML).expect("failed to parse grammars/registry.toml")
});

pub static HIGHLIGHT_NAMES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    let mut names = Vec::new();

    for grammar in &REGISTRY.grammar {
        for highlight_name in &grammar.highlight_names {
            let highlight_name = highlight_name.as_str();
            if !names.contains(&highlight_name) {
                names.push(highlight_name);
            }
        }
    }

    names
});

pub fn grammar(name: &str) -> &'static GrammarSpec {
    REGISTRY
        .grammar
        .iter()
        .find(|grammar| grammar.name == name)
        .unwrap_or_else(|| panic!("missing grammar registry entry for {name}"))
}

pub fn validate_repository_layout_at_manifest_dir() -> Result<(), String> {
    validate_repository_layout(Path::new(env!("CARGO_MANIFEST_DIR")))
}

pub fn validate_repository_layout(manifest_dir: &Path) -> Result<(), String> {
    let mut errors = Vec::new();
    let grammars_dir = manifest_dir.join("grammars");

    for grammar in &REGISTRY.grammar {
        let grammar_dir = grammars_dir.join(&grammar.name);

        if !grammar_dir.is_dir() {
            errors.push(format!(
                "registry entry `{}` is missing directory `{}`",
                grammar.name,
                grammar_dir.display()
            ));
            continue;
        }

        match grammar.parser_source {
            ParserSource::Vendored => {
                let grammar_js = grammar_dir.join("grammar.js");
                if !grammar_js.is_file() {
                    errors.push(format!(
                        "vendored grammar `{}` is missing `{}`",
                        grammar.name,
                        grammar_js.display()
                    ));
                }
            }
            ParserSource::Crate => match directory_contains_tracked_assets(&grammar_dir) {
                Ok(true) => {}
                Ok(false) => errors.push(format!(
                    "crate-backed grammar `{}` must keep at least one local asset under `{}`",
                    grammar.name,
                    grammar_dir.display()
                )),
                Err(error) => errors.push(format!(
                    "failed to inspect `{}` for grammar `{}`: {error}",
                    grammar_dir.display(),
                    grammar.name
                )),
            },
        }
    }

    if errors.is_empty() {
        return Ok(());
    }

    Err(format!(
        "grammar registry repository layout validation failed:\n- {}",
        errors.join("\n- ")
    ))
}

fn directory_contains_tracked_assets(dir: &Path) -> std::io::Result<bool> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_file() {
            return Ok(true);
        }

        if file_type.is_dir() && directory_contains_tracked_assets(&path)? {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        ParserSource, REGISTRY, VENDORED_GRAMMAR_EXCEPTIONS_MD, validate_repository_layout,
    };

    #[test]
    fn vendored_grammar_exceptions_doc_matches_registry() {
        let documented = documented_grammar_names(VENDORED_GRAMMAR_EXCEPTIONS_MD);
        let vendored_from_registry = REGISTRY
            .grammar
            .iter()
            .filter(|grammar| grammar.parser_source == ParserSource::Vendored)
            .map(|grammar| grammar.name.as_str())
            .collect::<BTreeSet<_>>();

        let documented_vendored = documented
            .difference(&support_asset_exceptions())
            .copied()
            .collect::<BTreeSet<_>>();

        assert_eq!(
            documented_vendored, vendored_from_registry,
            "docs/vendor-grammar-exceptions.md must list every vendored grammar exactly once"
        );
    }

    #[test]
    fn repository_layout_matches_registry_requirements() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        validate_repository_layout(manifest_dir).unwrap_or_else(|error| panic!("{error}"));
    }

    fn documented_grammar_names(markdown: &str) -> BTreeSet<&str> {
        markdown
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if !line.starts_with("- `") {
                    return None;
                }

                let name = line.strip_prefix("- `")?.split('`').next()?;
                Some(name)
            })
            .collect()
    }

    fn support_asset_exceptions() -> BTreeSet<&'static str> {
        BTreeSet::from(["css"])
    }
}
