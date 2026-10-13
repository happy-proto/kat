use super::*;

#[test]
fn debug_analysis_json_reports_markdown_document_kind() {
    let path = fixture_path("markdown/markdown_truecolor_cjk_repro.md");
    let source = read_file(&path);
    let json = debug_analysis_json(Some(path.as_path()), &source)
        .expect("analysis debug json should render");
    let value: Value = serde_json::from_str(&json).expect("analysis json should parse");

    assert_eq!(value["detected_document_kind"]["runtime_name"], "markdown");
    assert!(
        value["spans"]
            .as_array()
            .is_some_and(|spans| !spans.is_empty())
    );
}

#[test]
fn debug_analysis_json_reports_nested_region_runtime_identity() {
    let path = fixture_path("just/injections.just");
    let source = read_file(&path);
    let json = debug_analysis_json(Some(path.as_path()), &source)
        .expect("analysis debug json should render");
    let value: Value = serde_json::from_str(&json).expect("analysis json should parse");
    let nested_regions = value["nested_regions"]
        .as_array()
        .expect("expected nested regions array");

    assert!(
        nested_regions.iter().any(|region| {
            region["resolved_document_kind"]["runtime_name"] == Value::String("python".into())
        }),
        "expected analysis nested regions to expose injected python runtime identity: {json}"
    );
    assert!(
        nested_regions.iter().any(|region| {
            region["resolved_document_kind"]["runtime_name"] == Value::String("javascript".into())
        }),
        "expected analysis nested regions to expose injected javascript runtime identity: {json}"
    );
}

#[test]
fn debug_analysis_json_preserves_recursive_child_nested_regions() {
    let path = fixture_path("rust/doc_comments_nested.rs");
    let source = read_file(&path);
    let json = debug_analysis_json(Some(path.as_path()), &source)
        .expect("analysis debug json should render");
    let value: Value = serde_json::from_str(&json).expect("analysis json should parse");
    let nested_regions = value["nested_regions"]
        .as_array()
        .expect("expected nested regions array");
    let markdown_region = nested_regions
        .iter()
        .find(|region| {
            region["resolved_document_kind"]["runtime_name"] == Value::String("markdown".into())
        })
        .expect("expected outer rustdoc markdown region");
    let child_nested_regions = markdown_region["child_nested_regions"]
        .as_array()
        .expect("expected recursive child nested regions");

    assert!(
        child_nested_regions.iter().any(|region| {
            region["resolved_document_kind"]["runtime_name"] == Value::String("rust".into())
        }),
        "expected nested markdown region to preserve rust fenced block identity: {json}"
    );
    assert!(
        child_nested_regions.iter().any(|region| {
            region["resolved_document_kind"]["runtime_name"] == Value::String("python".into())
        }),
        "expected nested markdown region to preserve python fenced block identity: {json}"
    );
}

#[test]
fn debug_visual_json_reports_regions_array() {
    let path = fixture_path("markdown/markdown_truecolor_cjk_repro.md");
    let source = read_file(&path);
    let json =
        debug_visual_json(Some(path.as_path()), &source).expect("visual debug json should render");
    let value: Value = serde_json::from_str(&json).expect("visual json should parse");

    assert!(value["regions"].is_array());
}

#[test]
fn debug_layout_json_reports_rows_array() {
    let source = "install:\n    cargo install --path .\n";
    let json = debug_layout_json(Some(Path::new("Justfile")), source)
        .expect("layout debug json should render");
    let value: Value = serde_json::from_str(&json).expect("layout json should parse");

    assert!(value["rows"].is_array());
}

#[test]
fn debug_layout_json_preserves_python_docstring_blank_line_background_runs() {
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let json = debug_layout_json(Some(path.as_path()), &source)
        .expect("layout debug json for python docstrings should render");
    let value: Value = serde_json::from_str(&json).expect("layout json should parse");
    let blank_line_start = line_start_for_line_number(&source, 42);
    let rows = value["rows"]
        .as_array()
        .unwrap_or_else(|| panic!("expected layout rows array: {json}"));
    let blank_row = rows
        .iter()
        .find(|row| row["source_line_start"].as_u64() == Some(blank_line_start as u64))
        .unwrap_or_else(|| panic!("expected layout row for blank init docstring line: {json}"));

    assert!(
        blank_row["background_runs"]
            .as_array()
            .is_some_and(|runs| !runs.is_empty()),
        "expected debug-layout json to preserve block background runs for blank python docstring lines: {json}"
    );
}

#[test]
fn debug_visual_json_reports_rectangular_kind_for_just_recipe() {
    let source = "install:\n    cargo install --path .\n";
    let json = debug_visual_json(Some(Path::new("Justfile")), source)
        .expect("visual debug json for Justfile should render");
    let value: Value = serde_json::from_str(&json).expect("visual json should parse");
    let regions = value["regions"]
        .as_array()
        .unwrap_or_else(|| panic!("expected visual regions array: {json}"));

    assert!(
        regions
            .iter()
            .any(|region| region["visual_kind"] == Value::String("rect_block".into())),
        "expected visual debug json to preserve rectangular regions: {json}"
    );
}

#[test]
fn debug_render_ops_json_reports_text_ops() {
    let path = fixture_path("markdown/markdown_truecolor_cjk_repro.md");
    let source = read_file(&path);
    let json = debug_render_ops_json(Some(path.as_path()), &source)
        .expect("render-op debug json should render");
    let value: Value = serde_json::from_str(&json).expect("render-op json should parse");

    assert!(
        value["ops"]
            .as_array()
            .is_some_and(|ops| ops.iter().any(|op| op["op"] == "text"))
    );
}

#[test]
fn debug_terminal_json_reports_encoded_output() {
    let path = fixture_path("markdown/markdown_truecolor_cjk_repro.md");
    let source = read_file(&path);
    let json = debug_terminal_json(Some(path.as_path()), &source)
        .expect("terminal debug json should render");
    let value: Value = serde_json::from_str(&json).expect("terminal json should parse");

    assert!(
        value["encoded_output"]
            .as_str()
            .is_some_and(|output| !output.is_empty())
    );
    assert_eq!(
        value["capabilities"]["background_queries_enabled"],
        Value::Bool(false)
    );
}
