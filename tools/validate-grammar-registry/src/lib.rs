use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GrammarRegistry {
    pub grammar: Vec<GrammarSpec>,
}

#[derive(Debug, Deserialize)]
pub struct GrammarSpec {
    pub name: String,
}

pub fn default_repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("validator crate must live under tools/<name>")
        .to_path_buf()
}

pub fn validate_repository_layout(manifest_dir: &Path) -> Result<(), String> {
    let registry = load_registry(manifest_dir)?;
    let mut errors = Vec::new();
    let grammars_dir = manifest_dir.join("grammars");

    for grammar in &registry.grammar {
        let grammar_dir = grammars_dir.join(&grammar.name);

        if !grammar_dir.is_dir() {
            errors.push(format!(
                "registry entry `{}` is missing directory `{}`",
                grammar.name,
                grammar_dir.display()
            ));
            continue;
        }

        let highlights = grammar_dir.join("queries/highlights.scm");
        if !highlights.is_file() {
            errors.push(format!(
                "grammar `{}` is missing highlights query `{}`",
                grammar.name,
                highlights.display()
            ));
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

pub fn load_registry(manifest_dir: &Path) -> Result<GrammarRegistry, String> {
    let registry_path = manifest_dir.join("grammars/registry.toml");
    let registry = fs::read_to_string(&registry_path)
        .map_err(|error| format!("failed to read `{}`: {error}", registry_path.display()))?;
    toml::from_str(&registry)
        .map_err(|error| format!("failed to parse `{}`: {error}", registry_path.display()))
}
