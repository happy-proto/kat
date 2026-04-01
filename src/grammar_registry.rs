use std::sync::LazyLock;

use serde::Deserialize;

const REGISTRY_TOML: &str = include_str!("../grammars/registry.toml");

#[derive(Debug, Deserialize)]
pub struct GrammarRegistry {
    pub grammar: Vec<GrammarSpec>,
}

#[derive(Debug, Deserialize)]
pub struct GrammarSpec {
    pub name: String,
    #[allow(dead_code)]
    pub library_name: String,
    pub extensions: Vec<String>,
    pub filenames: Vec<String>,
    #[serde(default)]
    pub filename_prefixes: Vec<String>,
    pub shebang_substrings: Vec<String>,
    #[allow(dead_code)]
    pub extra_c_flags: Vec<String>,
    pub highlight_names: Vec<String>,
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
