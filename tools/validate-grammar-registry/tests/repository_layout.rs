use std::{collections::BTreeSet, fs};

use validate_grammar_registry::{
    ParserSource, default_repository_root, load_registry, scanner_sources_with_raw_gnu_attributes,
    validate_repository_layout,
};

#[test]
fn vendored_grammar_exceptions_doc_matches_registry() {
    let repository_root = default_repository_root();
    let registry = load_registry(&repository_root).expect("workspace grammar registry must load");
    let documented_markdown =
        fs::read_to_string(repository_root.join("docs/vendor-grammar-exceptions.md"))
            .expect("vendor grammar exceptions doc must load");
    let documented = documented_grammar_names(&documented_markdown);
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

#[test]
fn scanner_sources_avoid_raw_gnu_attributes() {
    let manifest_dir = default_repository_root();
    let grammars_dir = manifest_dir.join("grammars");
    let mut violations = Vec::new();

    for entry in fs::read_dir(&grammars_dir).expect("workspace grammars directory must exist") {
        let entry = entry.expect("grammar directory entry must load");
        let file_type = entry
            .file_type()
            .expect("grammar directory file type must load");
        if !file_type.is_dir() {
            continue;
        }

        let mut scanner_violations = scanner_sources_with_raw_gnu_attributes(&entry.path())
            .expect("scanner source portability check must be readable for every grammar");
        violations.append(&mut scanner_violations);
    }

    assert!(
        violations.is_empty(),
        "scanner sources must not use raw GNU __attribute__ annotations outside preprocessor guards: {violations:?}"
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
