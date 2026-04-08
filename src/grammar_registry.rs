use std::sync::LazyLock;

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
    #[allow(dead_code)]
    pub library_name: String,
    #[allow(dead_code)]
    #[serde(default)]
    pub parser_source: ParserSource,
    pub extensions: Vec<String>,
    pub filenames: Vec<String>,
    #[serde(default)]
    pub filename_prefixes: Vec<String>,
    pub shebang_substrings: Vec<String>,
    #[allow(dead_code)]
    pub extra_c_flags: Vec<String>,
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{ParserSource, REGISTRY, VENDORED_GRAMMAR_EXCEPTIONS_MD};

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
