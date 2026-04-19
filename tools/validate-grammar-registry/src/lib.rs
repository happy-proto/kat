use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct GrammarRegistry {
    pub grammar: Vec<GrammarSpec>,
}

#[derive(Debug, Deserialize)]
pub struct GrammarSpec {
    pub name: String,
    #[serde(default)]
    pub parser_source: ParserSource,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ParserSource {
    #[default]
    Vendored,
    Crate,
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

        match scanner_sources_with_raw_gnu_attributes(&grammar_dir) {
            Ok(scanner_paths) => {
                for (path, line_number) in scanner_paths {
                    errors.push(format!(
                        "scanner source `{}` uses raw `__attribute__` on line {}; wrap compiler-specific attributes in macros or remove them so MSVC builds keep working",
                        path.display(),
                        line_number
                    ));
                }
            }
            Err(error) => errors.push(format!(
                "failed to inspect scanner sources under `{}` for grammar `{}`: {error}",
                grammar_dir.display(),
                grammar.name
            )),
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

pub fn scanner_sources_with_raw_gnu_attributes(
    grammar_dir: &Path,
) -> std::io::Result<Vec<(PathBuf, usize)>> {
    let mut violations = Vec::new();

    for entry in WalkDir::new(grammar_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !matches!(file_name, "scanner.c" | "scanner.cc" | "scanner.cpp") {
            continue;
        }

        let contents = fs::read_to_string(path)?;
        for (index, line) in contents.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#')
                || trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed.starts_with('*')
            {
                continue;
            }

            if trimmed.contains("__attribute__") {
                violations.push((path.to_path_buf(), index + 1));
            }
        }
    }

    Ok(violations)
}

fn directory_contains_tracked_assets(dir: &Path) -> std::io::Result<bool> {
    for entry in WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() {
            return Ok(true);
        }
    }

    Ok(false)
}
