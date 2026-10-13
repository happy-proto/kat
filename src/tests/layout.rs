use super::*;

#[test]
fn nomad_template_python_projection_renders_without_layout_panics() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("hcl/nomad-template-python.nomad");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .expect("expected nomad template projection fixture to render");

    assert!(rendered.contains("render_message"));
    assert!(rendered.contains("NOMAD_ALLOC_ID"));
    assert!(rendered.contains("endif"));
}

#[test]
fn rustdoc_outer_markdown_prose_uses_tint_without_rectangular_padding() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("rust/doc_comments_nested.rs");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let markdown_region = find_nested_region(
        &analysis,
        "markdown",
        "same nested runtimes as top-level Markdown.",
        &source,
    );
    assert!(
        markdown_region.visual_level == 1,
        "outer rustdoc markdown prose should still receive the outer nested level"
    );
    assert_ne!(
        markdown_region.visual_kind, "rect_block",
        "outer rustdoc prose should not become a rectangular block"
    );
}

#[test]
fn python_docstring_blank_lines_render_background_width_as_spaces() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let render_plan = render_plan_snapshot_for_path(path.as_path(), &source, &theme);
    let rendered_text = render_plan_text(&render_plan);
    let rendered_lines: Vec<_> = rendered_text.lines().collect();
    let blank_after_intro = rendered_lines
        .get(23)
        .copied()
        .expect("expected render plan to retain blank init docstring line");
    let blank_before_raises = rendered_lines
        .get(27)
        .copied()
        .expect("expected render plan to retain blank Raises separator line");

    assert!(
        !blank_after_intro.is_empty()
            && blank_after_intro.chars().all(|ch| ch == ' ')
            && blank_after_intro.starts_with("        "),
        "expected blank init docstring line to render as background-bearing spaces, got {blank_after_intro:?}"
    );
    assert!(
        !blank_before_raises.is_empty()
            && blank_before_raises.chars().all(|ch| ch == ' ')
            && blank_before_raises.starts_with("        "),
        "expected blank Raises separator line to render as background-bearing spaces, got {blank_before_raises:?}"
    );
}

#[test]
fn python_docstring_blank_lines_switch_into_background_after_indent() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let render_plan = render_plan_snapshot_for_path(path.as_path(), &source, &theme);
    let blank_after_intro = render_plan_lines(&render_plan)
        .get(23)
        .cloned()
        .expect("expected init docstring blank line in render plan");

    assert!(
        matches!(
            blank_after_intro.as_slice(),
            [
                RenderOpSnapshot::Text { text: indent },
                RenderOpSnapshot::SetStyle { .. },
                RenderOpSnapshot::Text { text: tinted },
                RenderOpSnapshot::ResetStyle,
            ] if indent == "        " && !tinted.is_empty() && tinted.chars().all(|ch| ch == ' ')
        ),
        "expected blank init docstring line to switch into a tinted space run after the shared indent, got {blank_after_intro:?}"
    );
}

#[test]
fn markdown_fenced_code_wraps_into_one_exact_block_bbox() {
    let source = "# Demo\n\n```bash\necho short\n\necho 这里是一段足够长的中文内容用于验证围栏代码换行后的统一背景边界\n```\n";
    let theme =
        Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(RgbColor(1, 2, 3)));
    let layout =
        crate::api::layout_with_theme(Some(Path::new("demo.md")), source, &theme, Some(48))
            .expect("expected fenced Markdown layout")
            .snapshot();
    let fence_rows = layout
        .rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| row.text.starts_with("```").then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(fence_rows.len(), 2);
    let block_rows = &layout.rows[fence_rows[0] + 1..fence_rows[1]];

    assert!(block_rows.len() > 3, "expected the CJK code line to wrap");
    assert!(
        block_rows
            .iter()
            .all(|row| background_bounds_at_level(row, 1) == Some((0, 47))),
        "expected content, blank, and continuation rows to share the exact non-viewport-wide bbox: {block_rows:?}"
    );
    assert!(
        fence_rows
            .iter()
            .all(|index| { background_bounds_at_level(&layout.rows[*index], 1).is_none() }),
        "expected fence rows to stay outside the fenced-code Block"
    );
}

#[test]
fn typescript_jsdoc_prose_does_not_use_rectangular_trailing_padding() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let source = "/**\n * this is a much longer JSDoc line\n * short\n */\nconst answer = 42;\n";
    let analysis = analysis_snapshot_for_path(Path::new("demo.ts"), source, &theme);
    let region = find_nested_region(&analysis, "jsdoc", "short", source);
    assert!(
        region_covers_line(region, source, "short"),
        "expected shorter JSDoc prose line to keep its nested coverage"
    );
    assert_ne!(
        region.visual_kind, "rect_block",
        "expected JSDoc prose to avoid rectangular block layout"
    );
    let layout = layout_snapshot_for_path(Path::new("demo.ts"), source, &theme, 20);
    let long_row = layout_row_containing(&layout.rows, "this is a much");
    let short_row = layout_row_containing(&layout.rows, "* short");
    assert_eq!(background_bounds_at_level(long_row, 1), Some((0, 20)));
    assert_eq!(
        background_bounds_at_level(short_row, 1),
        Some((0, 8)),
        "expected TightBlock background to stop at the short row's own content edge"
    );
}

#[test]
fn markdown_html_block_tabs_layout_expands_tabs_into_display_spaces() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("markdown/html_block_tabs.md");
    let source = read_file(&path);
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 120);
    let short_row = layout_row_containing(&layout.rows, "<td>值");
    let long_row = layout_row_containing(&layout.rows, "这里放一段更长的中文内容");

    assert!(
        !short_row.text.contains('\t'),
        "expected layout row text to expand tabs before rendering: {:?}",
        short_row.text
    );
    assert!(
        !long_row.text.contains('\t'),
        "expected layout row text to expand tabs before rendering: {:?}",
        long_row.text
    );
}

#[test]
fn markdown_frontmatter_block_bbox_follows_wrapped_rows() {
    let path = fixture_path("markdown/frontmatter-narrow.md");
    let source = read_file(&path);
    let theme =
        Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(RgbColor(1, 2, 3)));
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 48);

    assert!(
        layout.rows.iter().all(|row| row.display_width <= 48),
        "expected frontmatter background padding to stay within the 48-column viewport, got rows {:?}",
        layout
            .rows
            .iter()
            .map(|row| (&row.text, row.display_width))
            .collect::<Vec<_>>()
    );

    let delimiter_rows = layout
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| row.text == "---")
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    assert_eq!(delimiter_rows.len(), 2);
    let block_rows = &layout.rows[delimiter_rows[0]..=delimiter_rows[1]];
    assert_eq!(block_rows.len(), 5);
    assert!(
        block_rows.iter().all(|row| {
            row.display_width == 47 && first_background_bounds(row) == Some((0, 47))
        }),
        "expected every wrapped frontmatter row to share the 47-column block bbox, got {:?}",
        block_rows
            .iter()
            .map(|row| (&row.text, row.display_width, &row.background_runs))
            .collect::<Vec<_>>()
    );
}

#[test]
fn just_recipe_rectangle_keeps_wrapped_rows_without_fake_blank_rows() {
    let theme =
        Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(RgbColor(1, 2, 3)));
    let path = PathBuf::from("testdata/showcase/just/recipe-block.just");
    let source = read_file(&path);
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 80);
    let first = layout_row_containing(&layout.rows, "@cd mdsre");
    let second = layout_row_containing(&layout.rows, "uv run mdsre mongo query");
    let continuation = layout_row_containing(&layout.rows, "ws.forEach(r =>");

    assert_eq!(
        first
            .background_runs
            .first()
            .map(|run| run.start_column)
            .unwrap_or_default(),
        0,
        "expected the recipe rectangle to include wrapped continuation rows"
    );
    assert_eq!(
        second
            .background_runs
            .first()
            .map(|run| run.start_column)
            .unwrap_or_default(),
        0,
        "expected all recipe rows to share the same left edge"
    );
    assert_eq!(
        continuation
            .background_runs
            .first()
            .map(|run| run.start_column)
            .unwrap_or(usize::MAX),
        0,
        "expected wrapped recipe rows to share the rectangle's left edge"
    );

    let first_index = layout
        .rows
        .iter()
        .position(|row| row.text.contains("@cd mdsre"))
        .expect("expected first recipe row");
    let second_index = layout
        .rows
        .iter()
        .position(|row| row.text.contains("uv run mdsre mongo query"))
        .expect("expected second recipe row");

    assert_eq!(
        second_index,
        first_index + 1,
        "expected no fake blank layout rows between recipe command rows: {:?}",
        layout
            .rows
            .iter()
            .map(|row| (&row.text, &row.background_runs))
            .collect::<Vec<_>>()
    );
}

#[test]
fn just_recipe_rectangle_tracks_cjk_wrapped_width_in_display_cells() {
    let theme =
        Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(RgbColor(1, 2, 3)));
    let source = "demo:\n    printf '短描述短描述短描述短描述短描述'\n";
    let layout =
        crate::api::layout_with_theme(Some(Path::new("Justfile")), source, &theme, Some(16))
            .expect("expected layout to render for CJK recipe")
            .snapshot();
    let first = layout_row_containing(&layout.rows, "    printf '短");
    let continuation = layout_row_containing(&layout.rows, "描述短描述");

    assert_eq!(
        first
            .background_runs
            .first()
            .map(|run| run.start_column)
            .unwrap_or_default(),
        0,
        "expected the CJK recipe rectangle to include wrapped rows"
    );
    assert_eq!(
        continuation
            .background_runs
            .first()
            .map(|run| run.start_column)
            .unwrap_or(usize::MAX),
        0,
        "expected wrapped CJK continuation rows to be measured in display cells"
    );
}

#[test]
fn just_recipe_nested_indentation_uses_one_rectangular_background() {
    let theme =
        Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(RgbColor(1, 2, 3)));
    let source = "demo:\n    echo outer\n        echo nested\n";
    let layout =
        crate::api::layout_with_theme(Some(Path::new("Justfile")), source, &theme, Some(80))
            .expect("expected layout to render for nested recipe indentation")
            .snapshot();

    assert_eq!(
        first_background_bounds(layout_row_containing(&layout.rows, "echo outer")),
        Some((4, 19)),
    );
    assert_eq!(
        first_background_bounds(layout_row_containing(&layout.rows, "echo nested")),
        Some((4, 19)),
        "expected every recipe row to share one rectangular left edge"
    );
}
