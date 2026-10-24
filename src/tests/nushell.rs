use super::*;

#[test]
fn nushell_detects_extensions_and_shebangs_without_content_guessing() {
    for (path, source) in [
        (Some(Path::new("activate.nu")), "export-env { let x = 1 }\n"),
        (None, "#!/usr/bin/env nu\nprint 'hello'\n"),
        (None, "#!/opt/tools/nu\nprint 'hello'\n"),
        (
            None,
            "#!/usr/bin/env -S nu --no-config-file\nprint 'hello'\n",
        ),
    ] {
        assert_eq!(detected_runtime(path, source), Some(RuntimeId::Nushell));
    }
    assert_eq!(
        detected_runtime(Some(Path::new("activate")), "export-env {}\n"),
        None
    );
}

#[test]
fn nushell_parses_and_highlights_environment_overlays_and_typed_pipelines() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    for path in [
        fixture_path("nushell/activation.nu"),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/showcase/nushell/services.nu"),
    ] {
        let source = read_file(&path);
        let mut parser = Parser::new();
        parser
            .set_language(&runtime("nushell").unwrap().language)
            .unwrap();
        let tree = parser.parse(&source, None).unwrap();
        assert!(
            !tree.root_node().has_error(),
            "{}: {}",
            path.display(),
            tree.root_node().to_sexp()
        );
        let rendered = render_with_theme(Some(&path), &source, &theme).unwrap();
        assert_eq!(strip_ansi(&rendered), source);
    }

    let source = "def greet [name: string] {\n let enabled = true\n print $\"hello ($name)\"\n}\n";
    let rendered = highlight_named_language("nushell", source, &theme).unwrap();
    assert_eq!(
        rendered,
        highlight_named_language("nu", source, &theme).unwrap()
    );
    for fragment in [
        "\x1b[38;2;255;121;198mdef",
        "\x1b[38;2;80;250;123mgreet",
        "\x1b[38;2;139;233;253mstring",
        "\x1b[38;2;255;184;108mtrue",
        "\x1b[38;2;139;233;253mprint",
        "\x1b[38;2;241;250;140m$\"hello ",
    ] {
        assert!(
            rendered.contains(fragment),
            "missing {fragment:?}: {rendered:?}"
        );
    }
}

#[test]
fn nushell_markdown_fences_resolve_both_aliases() {
    let source = "```nu\nlet answer = 42\n```\n\n```nushell\nprint $answer\n```\n";
    let json = debug_analysis_json(Some(Path::new("example.md")), source).unwrap();
    let snapshot: Value = serde_json::from_str(&json).unwrap();
    let regions = snapshot["nested_regions"].as_array().unwrap();
    assert_eq!(regions.len(), 2);
    for region in regions {
        assert_eq!(region["resolved_document_kind"]["runtime_name"], "nushell");
    }
}
