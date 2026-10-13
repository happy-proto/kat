use std::fs;
use validate_grammar_registry::{default_repository_root, validate_repository_layout};

#[test]
fn repository_layout_matches_registry_requirements() {
    validate_repository_layout(&default_repository_root()).unwrap();
}

#[test]
fn registry_requires_each_grammars_highlights_asset() {
    let root = std::env::temp_dir().join(format!("kat-registry-contract-{}", std::process::id()));
    let grammar = root.join("grammars/example/queries");
    fs::create_dir_all(&grammar).unwrap();
    fs::write(
        root.join("grammars/registry.toml"),
        "[[grammar]]\nname = \"example\"\n",
    )
    .unwrap();
    let result = validate_repository_layout(&root).unwrap_err();
    assert!(result.contains("highlights.scm"));
    fs::write(grammar.join("highlights.scm"), "(identifier) @variable").unwrap();
    validate_repository_layout(&root).unwrap();
    fs::remove_dir_all(root).unwrap();
}
