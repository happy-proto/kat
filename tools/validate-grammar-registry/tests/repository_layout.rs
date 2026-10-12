use std::fs;

use validate_grammar_registry::{
    ParserSource, default_repository_root, load_registry, scanner_sources_with_raw_gnu_attributes,
    validate_repository_layout,
};

#[test]
fn all_grammars_use_crate_backed_parsers() {
    let repository_root = default_repository_root();
    let registry = load_registry(&repository_root).expect("workspace grammar registry must load");
    let vendored = registry
        .grammar
        .iter()
        .filter(|grammar| grammar.parser_source == ParserSource::Vendored)
        .map(|grammar| grammar.name.as_str())
        .collect::<Vec<_>>();

    assert!(
        vendored.is_empty(),
        "kat consumes pre-generated crate-backed parsers; vendored grammars: {vendored:?}"
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
