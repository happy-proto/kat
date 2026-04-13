use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use serde::Deserialize;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
struct GrammarRegistry {
    grammar: Vec<GrammarSpec>,
}

#[derive(Debug, Deserialize)]
struct GrammarSpec {
    name: String,
    #[serde(default)]
    parser_source: ParserSource,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ParserSource {
    #[default]
    Vendored,
    Crate,
}

fn main() -> ExitCode {
    let manifest_dir = repository_root_from_args(env::args().skip(1));
    match validate_repository_layout(&manifest_dir) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn repository_root_from_args(mut args: impl Iterator<Item = String>) -> PathBuf {
    if let Some(path) = args.next() {
        if args.next().is_some() {
            panic!("expected at most one optional repository root argument");
        }
        return PathBuf::from(path);
    }

    default_repository_root()
}

fn default_repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("validator crate must live under tools/<name>")
        .to_path_buf()
}

fn validate_repository_layout(manifest_dir: &Path) -> Result<(), String> {
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
    }

    if errors.is_empty() {
        return Ok(());
    }

    Err(format!(
        "grammar registry repository layout validation failed:\n- {}",
        errors.join("\n- ")
    ))
}

fn load_registry(manifest_dir: &Path) -> Result<GrammarRegistry, String> {
    let registry_path = manifest_dir.join("grammars/registry.toml");
    let registry = fs::read_to_string(&registry_path)
        .map_err(|error| format!("failed to read `{}`: {error}", registry_path.display()))?;
    toml::from_str(&registry)
        .map_err(|error| format!("failed to parse `{}`: {error}", registry_path.display()))
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{ParserSource, default_repository_root, load_registry, validate_repository_layout};

    const VENDORED_GRAMMAR_EXCEPTIONS_MD: &str =
        include_str!("../../../docs/vendor-grammar-exceptions.md");

    #[test]
    fn vendored_grammar_exceptions_doc_matches_registry() {
        let registry = load_registry(&default_repository_root())
            .expect("workspace grammar registry must load");
        let documented = documented_grammar_names(VENDORED_GRAMMAR_EXCEPTIONS_MD);
        let vendored_from_registry = registry
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
        let manifest_dir = default_repository_root();
        validate_repository_layout(&manifest_dir).unwrap_or_else(|error| panic!("{error}"));
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
