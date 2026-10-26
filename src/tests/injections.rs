use super::*;

#[test]
fn caddyfile_injects_cel_and_tagged_heredocs() {
    let source = include_str!("../../testdata/fixtures/caddyfile/Caddyfile");
    let analysis = analysis_snapshot_for_path(
        Path::new("Caddyfile"),
        source,
        &Theme::for_mode(ColorMode::TrueColor),
    );
    find_nested_region(&analysis, "cel", "@allowed expression", source);
    find_nested_region(&analysis, "html", "<html>", source);
}

#[test]
fn markdown_caddyfile_fences_reuse_configuration_and_nested_runtimes() {
    for alias in ["caddyfile", "caddy", "Caddyfile"] {
        let source = format!(
            "```{alias}\n{}\n```\n",
            include_str!("../../testdata/fixtures/caddyfile/Caddyfile")
        );
        let analysis = analysis_snapshot_for_path(
            Path::new("README.md"),
            &source,
            &Theme::for_mode(ColorMode::TrueColor),
        );
        find_nested_region(&analysis, "caddyfile", "reverse_proxy", &source);
        find_nested_region(&analysis, "cel", "@allowed expression", &source);
        find_nested_region(&analysis, "html", "<html>", &source);
    }
}

#[test]
fn just_injections_highlight_nested_languages() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("just/injections.just");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198mimport"),
        "expected injected Python keyword styling for import"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108mTrue"),
        "expected injected Python boolean styling for True"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected injected Bash builtin styling for source"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected bun shebang to reuse injected JavaScript builtin styling"
    );
}

#[test]
fn html_injections_highlight_script_and_style_content() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("html/index.html");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198mstyle"),
        "expected html tag-name styling for style tag"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m#8be9fd"),
        "expected injected css color styling for custom property value"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected injected javascript builtin styling for console"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m@param"),
        "expected jsdoc tag styling for @param"
    );
}

#[test]
fn bash_heredoc_interpreter_blocks_highlight_nested_languages() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("bash/heredoc_injections.sh");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253mPreview"),
        "expected python heredoc to reuse python class styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mrender"),
        "expected python heredoc to reuse python method styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected node heredoc to reuse javascript builtin styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected bash heredoc to reuse bash builtin styling"
    );
}

#[test]
fn html_attribute_injections_highlight_css_and_javascript() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("html/attribute_injections.html");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;80;250;123mvar"),
        "expected injected CSS function styling inside style attribute"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected injected JavaScript builtin styling inside onclick attribute"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198mreturn"),
        "expected injected JavaScript keyword styling inside onclick attribute"
    );
}

#[test]
fn jinja_yaml_projection_preserves_mapping_structure_across_expressions() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("jinja/config.yaml.j2");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(&path, &source, &theme);
    let yaml_region = find_nested_region(&analysis, "yaml", "sources:", &source);
    let property_style = theme
        .token_style_for("property.yaml", "key")
        .map(|style| style.snapshot(ColorMode::TrueColor));

    for key in ["scrape_interval_secs", "sinks", "healthcheck", "enabled"] {
        let offset = source
            .find(key)
            .unwrap_or_else(|| panic!("expected source to contain key {key:?}"));
        assert!(
            yaml_region.overlays.iter().any(|span| {
                span.start <= offset && offset < span.end && span.style == property_style
            }),
            "expected YAML key {key:?} after Jinja expressions to retain property styling"
        );
    }
}

#[test]
fn jinja_template_projection_preserves_each_registered_host_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let cases = [
        ("jinja/page.html.j2", "html", "section", "tag"),
        ("jinja/feed.xml.j2", "xml", "feed", "tag"),
        ("jinja/query.sql.j2", "sql", "FROM", "keyword"),
        ("jinja/theme.css.j2", "css", "color", "property"),
        ("jinja/app.js.j2", "javascript", "export", "keyword.import"),
        (
            "jinja/app.coffee.j2",
            "coffeescript",
            "class",
            "keyword.control",
        ),
    ];

    for (relative_path, runtime_name, needle, capture) in cases {
        let path = fixture_path(relative_path);
        let source = read_file(&path);
        let analysis = analysis_snapshot_for_path(&path, &source, &theme);
        let region = find_nested_region(&analysis, runtime_name, needle, &source);
        let offset = source
            .find(needle)
            .unwrap_or_else(|| panic!("expected source to contain {needle:?}"));
        let expected_style = theme
            .token_style_for(capture, needle)
            .map(|style| style.snapshot(ColorMode::TrueColor));

        assert!(
            region.overlays.iter().any(|span| {
                span.start <= offset && offset < span.end && span.style == expected_style
            }),
            "expected {runtime_name} token {needle:?} after a Jinja expression to retain {capture:?} styling"
        );
    }
}

#[test]
fn jinja_html_projection_preserves_script_and_style_injections() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("jinja/page.html.j2");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(&path, &source, &theme);

    for (runtime_name, needle, capture) in [
        ("javascript", "export", "keyword.import"),
        ("css", "color", "property"),
    ] {
        let region = find_nested_region(&analysis, runtime_name, needle, &source);
        let offset = source
            .find(needle)
            .unwrap_or_else(|| panic!("expected source to contain {needle:?}"));
        let expected_style = theme
            .token_style_for(capture, needle)
            .map(|style| style.snapshot(ColorMode::TrueColor));

        assert!(
            region.overlays.iter().any(|span| {
                span.start <= offset && offset < span.end && span.style == expected_style
            }),
            "expected nested {runtime_name} token {needle:?} to retain {capture:?} styling"
        );
    }
}

#[test]
fn template_projection_preserves_html_across_supported_template_engines() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let cases = [
        (
            "templates/page.html.j2",
            "<main>{{ title }}<section>after</section></main>\n",
        ),
        (
            "templates/page.html.twig",
            "<main>{{ title }}<section>after</section></main>\n",
        ),
        (
            "templates/page.html.erb",
            "<main><%= title %><section>after</section></main>\n",
        ),
        (
            "templates/page.html.eex",
            "<main><%= @title %><section>after</section></main>\n",
        ),
        (
            "templates/page.jsp",
            "<main><%= title %><section>after</section></main>\n",
        ),
        (
            "templates/page.asp",
            "<main><%= title %><section>after</section></main>\n",
        ),
        (
            "templates/page.adp",
            "<main><%= $title %><section>after</section></main>\n",
        ),
    ];
    let tag_style = theme
        .token_style_for("tag", "section")
        .map(|style| style.snapshot(ColorMode::TrueColor));

    for (path, source) in cases {
        let analysis = analysis_snapshot_for_path(Path::new(path), source, &theme);
        let region = find_nested_region(&analysis, "html", "section", source);
        let offset = source
            .find("section")
            .expect("expected source to contain section tag");
        assert!(
            region.overlays.iter().any(|span| {
                span.start <= offset && offset < span.end && span.style == tag_style
            }),
            "expected HTML tag after template output to retain styling for {path}"
        );
    }
}

#[test]
fn nomad_template_python_projection_uses_rectangular_block_tint() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("hcl/nomad-template-python.nomad");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let region = find_nested_region(&analysis, "python", "def render_message():", &source);
    assert_eq!(region.visual_kind, "rect_block");

    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 120);
    let import_row = layout_row_containing(&layout.rows, "import os");
    let blank_row = layout
        .rows
        .iter()
        .find(|row| row.source_line_start == line_start_for_line_number(&source, 7))
        .expect("expected layout to retain the blank line inside the Nomad template body");
    let endif_row = layout_row_containing(&layout.rows, "%{ endif }");
    let expected_bounds = first_background_bounds(import_row)
        .expect("expected projected Python block to have a background");

    assert_eq!(
        expected_bounds.0, 0,
        "expected projected Python block to start at the source left edge"
    );
    assert_eq!(
        first_background_bounds(blank_row),
        Some(expected_bounds),
        "expected blank lines inside the projected template body to keep the same block tint"
    );
    assert_eq!(
        first_background_bounds(endif_row),
        Some(expected_bounds),
        "expected HCL template directive lines inside the projection to keep the same block tint"
    );
}

#[test]
fn go_injections_and_markdown_fences_reuse_same_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let go_path = fixture_path("go/rich.go");
    let go_source = read_file(&go_path);
    let go_rendered = render_with_theme(Some(go_path.as_path()), &go_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", go_path.display()));

    assert!(
        go_rendered.contains("\x1b[38;2;255;184;108mtrue"),
        "expected injected JSON boolean styling in Go"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected injected SQL keyword styling in Go"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;255;121;198mRETURNING"),
        "expected Go sql:postgres hint to route through postgres SQL runtime"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;255;121;198mAUTO_INCREMENT"),
        "expected Go sql:mysql hint to route through mysql SQL runtime"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;255;121;198mWITHOUT"),
        "expected Go sql:sqlite hint to route through sqlite SQL runtime"
    );
    assert!(
        go_rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected injected JavaScript builtin styling in Go"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;255;121;198msection"),
        "expected injected HTML tag styling in Go"
    );
    assert!(
        go_rendered.contains("--accent"),
        "expected injected CSS custom property content in Go"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;139;233;253mprintf"),
        "expected injected Bash builtin styling in Go"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;139;233;253msection"),
        "expected injected regex named group styling in Go"
    );
    assert!(
        go_rendered.contains("\x1b[3m\x1b[38;2;139;233;253malpha"),
        "expected Go regexp.MustCompilePOSIX case to reuse posix regex styling"
    );
    assert!(
        go_rendered.contains("\x1b[38;2;139;233;253mescapedSection"),
        "expected Go escaped regex string case to reuse regex named group styling"
    );

    let markdown_path = fixture_path("markdown/go_fence.md");
    let markdown_source = read_file(&markdown_path);
    let markdown_rendered =
        render_with_theme(Some(markdown_path.as_path()), &markdown_source, &theme).unwrap_or_else(
            |error| panic!("failed to render {}: {error}", markdown_path.display()),
        );

    assert!(
        markdown_rendered.contains("\x1b[38;2;139;233;253mRenderer"),
        "expected fenced go block to reuse go type styling"
    );
    assert!(
        markdown_rendered.contains("\x1b[38;2;80;250;123mRender"),
        "expected fenced go block to reuse go method styling"
    );
    assert!(
        markdown_rendered.contains("\x1b[38;2;80;250;123mNewRenderer"),
        "expected fenced golang block alias to reuse go function styling"
    );

    let sql_markdown_path = fixture_path("markdown/sql_dialects.md");
    let sql_markdown_source = read_file(&sql_markdown_path);
    let sql_markdown_rendered = render_with_theme(
        Some(sql_markdown_path.as_path()),
        &sql_markdown_source,
        &theme,
    )
    .unwrap_or_else(|error| panic!("failed to render {}: {error}", sql_markdown_path.display()));
    assert!(
        sql_markdown_rendered.contains("\x1b[38;2;255;121;198mRETURNING"),
        "expected fenced postgres block alias to route through postgres SQL runtime"
    );
    assert!(
        sql_markdown_rendered.contains("\x1b[38;2;255;121;198mAUTO_INCREMENT"),
        "expected fenced mysql block alias to route through mysql SQL runtime"
    );
    assert!(
        sql_markdown_rendered.contains("\x1b[38;2;255;121;198mWITHOUT"),
        "expected fenced sqlite block alias to route through sqlite SQL runtime"
    );
}

#[test]
fn nested_fixtures_lock_supported_injection_scenarios() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let markdown_path = fixture_path("markdown/nested_runtime.md");
    let markdown_source = read_file(&markdown_path);
    let markdown_rendered =
        render_with_theme(Some(markdown_path.as_path()), &markdown_source, &theme).unwrap_or_else(
            |error| panic!("failed to render {}: {error}", markdown_path.display()),
        );
    assert!(
        markdown_rendered.contains("title"),
        "expected TOML frontmatter property styling in nested markdown fixture"
    );
    assert!(
        markdown_rendered.contains("\x1b[38;2;139;233;253mPreview"),
        "expected injected Python class styling in nested markdown fixture"
    );
    assert!(
        markdown_rendered.contains("\x1b[38;2;255;184;108mtrue"),
        "expected injected JSON boolean styling in nested markdown fixture"
    );

    let html_path = fixture_path("html/nested_regions.html");
    let html_source = read_file(&html_path);
    let html_rendered = render_with_theme(Some(html_path.as_path()), &html_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", html_path.display()));
    assert!(
        html_rendered.contains("ff79c6"),
        "expected injected CSS styling in nested HTML fixture"
    );
    assert!(
        html_rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected injected JavaScript builtin styling in nested HTML fixture"
    );
    assert!(
        html_rendered.contains("\x1b[38;2;255;121;198m@param"),
        "expected injected JSDoc styling in nested HTML fixture"
    );

    let html_attr_path = fixture_path("html/attribute_injections.html");
    let html_attr_source = read_file(&html_attr_path);
    let html_attr_rendered =
        render_with_theme(Some(html_attr_path.as_path()), &html_attr_source, &theme)
            .unwrap_or_else(|error| {
                panic!("failed to render {}: {error}", html_attr_path.display())
            });
    assert!(
        html_attr_rendered.contains("\x1b[38;2;80;250;123mvar"),
        "expected injected CSS styling in HTML attribute fixture"
    );
    assert!(
        html_attr_rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected injected JavaScript styling in HTML attribute fixture"
    );

    let just_path = fixture_path("just/nested_recipes.just");
    let just_source = read_file(&just_path);
    let just_rendered = render_with_theme(Some(just_path.as_path()), &just_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", just_path.display()));
    assert!(
        just_rendered.contains("\x1b[38;2;139;233;253mPreview"),
        "expected injected Python class styling in nested Just fixture"
    );
    assert!(
        just_rendered.contains("\x1b[38;2;80;250;123mrender"),
        "expected injected Python method styling in nested Just fixture"
    );
    assert!(
        just_rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected injected Bash builtin styling in nested Just fixture"
    );

    let just_heredoc_path = fixture_path("just/heredoc_recipes.just");
    let just_heredoc_source = read_file(&just_heredoc_path);
    let just_heredoc_rendered = render_with_theme(
        Some(just_heredoc_path.as_path()),
        &just_heredoc_source,
        &theme,
    )
    .unwrap_or_else(|error| panic!("failed to render {}: {error}", just_heredoc_path.display()));
    assert!(
        just_heredoc_rendered.contains("\x1b[38;2;139;233;253mPreview"),
        "expected bash heredoc injections inside Justfile to reuse python class styling"
    );
    assert!(
        just_heredoc_rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected bash heredoc injections inside Justfile to reuse javascript builtin styling"
    );
    assert!(
        just_heredoc_rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected bash heredoc injections inside Justfile to reuse bash builtin styling"
    );

    let js_path = fixture_path("javascript/tagged_templates.js");
    let js_source = read_file(&js_path);
    let js_rendered = render_with_theme(Some(js_path.as_path()), &js_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", js_path.display()));
    assert!(
        js_rendered.contains("--accent"),
        "expected injected CSS styling in JavaScript tagged template fixture"
    );
    assert!(
        js_rendered.contains("\x1b[38;2;255;184;108mtrue"),
        "expected injected JSON boolean styling in JavaScript tagged template fixture"
    );
    assert!(
        js_rendered.contains("\x1b[38;2;255;121;198msection"),
        "expected injected HTML tag styling in JavaScript tagged template fixture"
    );
}

#[test]
fn markdown_injections_highlight_inline_and_fenced_content() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("markdown/sample.md");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));
    let stripped = strip_ansi(&rendered);

    assert!(
        rendered.contains("layout"),
        "expected markdown YAML frontmatter property styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"solarized\""),
        "expected markdown YAML frontmatter string styling"
    );
    assert!(
        rendered.contains("\x1b[1m\x1b[38;2;189;147;249mPreview"),
        "expected markdown heading styling"
    );
    assert!(
        rendered.contains("\x1b[1m\x1b[38;2;189;147;249mSecondary Heading"),
        "expected markdown setext heading styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;241;250;140memphasis"),
        "expected markdown inline emphasis styling"
    );
    assert!(
        rendered.contains("\x1b[1m\x1b[38;2;255;184;108mstrong"),
        "expected markdown strong-emphasis styling"
    );
    assert!(
        rendered.contains("\x1b[9m\x1b[38;2;98;114;164mstrikethrough"),
        "expected markdown strikethrough styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m- [x]"),
        "expected markdown task-list marker styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m1."),
        "expected markdown ordered-list marker styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mhttps://example.com"),
        "expected markdown link uri styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m<https://example.com/autolink>"),
        "expected markdown autolink styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m<hello@example.com>"),
        "expected markdown email autolink styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m[docs-ref]"),
        "expected markdown reference link label styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"Docs Title\""),
        "expected markdown link title styling"
    );
    assert!(
        stripped.contains("| Column | Value |") && stripped.contains("| Code | `let value = 42` |"),
        "expected markdown pipe table delimiter styling"
    );
    assert!(
        stripped.contains("```rust"),
        "expected markdown fenced code delimiter line styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mlet value = 42"),
        "expected markdown inline code-span styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108mplain fence without syntax"),
        "expected markdown plain fenced code block styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;241;250;140m>"),
        "expected markdown blockquote marker styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198mkbd"),
        "expected markdown inline HTML to route through HTML highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198mreturn"),
        "expected fenced rust block injection styling"
    );
}

#[test]
fn markdown_frontmatter_supports_yaml_and_toml_metadata() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let yaml_path = fixture_path("markdown/sample.md");
    let yaml_source = read_file(&yaml_path);
    let yaml_rendered = render_with_theme(Some(yaml_path.as_path()), &yaml_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", yaml_path.display()));

    assert!(
        yaml_rendered.contains("layout"),
        "expected YAML frontmatter property styling"
    );
    assert!(
        yaml_rendered.contains("\x1b[38;2;255;184;108mtrue"),
        "expected YAML frontmatter boolean styling"
    );

    let toml_path = Path::new("testdata/showcase/markdown/mixed-code-blocks.md").to_path_buf();
    let toml_source = read_file(&toml_path);
    let toml_rendered = render_with_theme(Some(toml_path.as_path()), &toml_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", toml_path.display()));

    assert!(
        toml_rendered.contains("title"),
        "expected TOML frontmatter property styling"
    );
    assert!(
        toml_rendered.contains("\x1b[38;2;255;184;108mtrue"),
        "expected TOML frontmatter boolean styling"
    );
}

#[test]
fn rustdoc_markdown_fenced_code_reuses_nested_language_runtimes() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("rust/doc_comments_nested.rs");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let markdown_region =
        find_nested_region(&analysis, "markdown", "Rustdoc should support", &source);

    assert!(
        markdown_region
            .child_nested_regions
            .iter()
            .any(|region| region.resolved_document_kind.runtime_name == "rust"),
        "expected rustdoc markdown region to preserve fenced rust child runtime"
    );
    assert!(
        markdown_region
            .child_nested_regions
            .iter()
            .any(|region| region.resolved_document_kind.runtime_name == "python"),
        "expected rustdoc markdown region to preserve fenced python child runtime"
    );
}

#[test]
fn rustdoc_nested_regions_use_different_tint_levels() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("rust/doc_comments_nested.rs");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let markdown_region =
        find_nested_region(&analysis, "markdown", "Rustdoc should support", &source);
    let rust_region = find_nested_region(&analysis, "rust", "fn nested() -> i32 {", &source);

    assert!(
        markdown_region.visual_level == 1,
        "outer rustdoc markdown prose should stay at the first nested visual level"
    );
    assert!(
        rust_region.visual_level == 2,
        "expected fenced code inside rustdoc to use a stronger nested visual level"
    );
}

#[test]
fn python_docstrings_highlight_rest_google_and_numpy_structures() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let keyword_style = theme
        .token_style_for("keyword.directive", "Args")
        .expect("expected keyword.directive style")
        .snapshot(ColorMode::TrueColor);
    let parameter_style = theme
        .token_style_for("variable.parameter", "name")
        .expect("expected variable.parameter style")
        .snapshot(ColorMode::TrueColor);
    let type_style = theme
        .token_style_for("type", "ThemePreview")
        .expect("expected type style")
        .snapshot(ColorMode::TrueColor);
    let punctuation_style = theme
        .token_style_for("punctuation.special", ":")
        .expect("expected punctuation.special style")
        .snapshot(ColorMode::TrueColor);

    let args_offset = source.find("Args:").expect("expected Args section");
    let param_field_offset = source
        .find(":param name:")
        .expect("expected reST param field");
    let role_offset = source
        .find(":class:`ThemePreview`")
        .expect("expected reST class role");
    let numpy_section_offset = source
        .find("Parameters\n        ----------")
        .expect("expected NumPy section header");
    let numpy_param_offset = source
        .find("uppercase : bool")
        .expect("expected NumPy parameter entry");

    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= args_offset
                && args_offset < span.end
                && span.style == Some(keyword_style.clone())
        }),
        "expected Google section header to receive directive styling"
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= param_field_offset + 1
                && param_field_offset + 1 < span.end
                && span.style == Some(keyword_style.clone())
        }),
        "expected reST field name to receive directive styling"
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= param_field_offset + 7
                && param_field_offset + 7 < span.end
                && span.style == Some(parameter_style.clone())
        }),
        "expected reST parameter name to receive parameter styling"
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= role_offset + 8
                && role_offset + 8 < span.end
                && span.style == Some(type_style.clone())
        }),
        "expected reST role target to receive type styling"
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= numpy_section_offset
                && numpy_section_offset < span.end
                && span.style == Some(keyword_style.clone())
        }),
        "expected NumPy section title to receive directive styling"
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= numpy_param_offset
                && numpy_param_offset < span.end
                && span.style == Some(parameter_style.clone())
        }),
        "expected NumPy parameter name to receive parameter styling"
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= numpy_param_offset + "uppercase ".len()
                && numpy_param_offset + "uppercase ".len() < span.end
                && span.style == Some(punctuation_style.clone())
        }),
        "expected NumPy parameter separator to receive punctuation styling"
    );
}

#[test]
fn python_docstring_doctest_reuses_python_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let doctest_region = find_nested_region(
        &analysis,
        "python",
        ">>> preview = ThemePreview(\"Dracula\")",
        &source,
    );

    assert!(
        doctest_region.visual_level >= 2,
        "expected doctest code to render as a deeper nested python runtime"
    );
    assert!(
        doctest_region
            .layout_segments
            .iter()
            .all(|segment| segment.left > segment.line_start),
        "expected doctest runtime to begin after the doctest prompt"
    );
}

#[test]
fn python_docstring_doctest_highlights_inner_python_literals() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let string_style = theme
        .token_style_for("string", "\"Dracula\"")
        .expect("expected string style")
        .snapshot(ColorMode::TrueColor);
    let string_offset = source
        .find("\"Dracula\"")
        .expect("expected doctest string literal");

    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= string_offset
                && string_offset < span.end
                && span.style == Some(string_style.clone())
        }),
        "expected doctest string literal to reuse Python string styling"
    );
}

#[test]
fn python_docstring_runtime_uses_rectangular_block_visuals() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let docstring_region = find_nested_region(
        &analysis,
        "python_docstring",
        "Theme preview helpers.",
        &source,
    );
    assert_eq!(
        docstring_region.visual_kind, "rect_block",
        "expected python docstring runtime to render as a rectangular block"
    );
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 120);
    let rows = [
        layout_row_containing(&layout.rows, "Theme preview helpers."),
        layout_row_containing(
            &layout.rows,
            "This module exercises several common docstring styles.",
        ),
        layout_row_containing(
            &layout.rows,
            "The public entrypoint is :class:`ThemePreview`.",
        ),
    ];
    let expected_bounds = first_background_bounds(rows[0]);
    assert!(expected_bounds.is_some());
    assert!(
        rows.iter()
            .all(|row| first_background_bounds(row) == expected_bounds),
        "expected python docstring rows to share one layout-time block bbox"
    );
}

#[test]
fn python_docstring_wraps_blank_and_content_rows_into_one_exact_bbox() {
    let source = "def demo():\n    \"\"\"短说明。\n\n    这里是一段足够长的中文内容用于验证文档字符串换行后的统一背景边界\n    \"\"\"\n";
    let theme =
        Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(RgbColor(1, 2, 3)));
    let layout =
        crate::api::layout_with_theme(Some(Path::new("demo.py")), source, &theme, Some(49))
            .expect("expected Python docstring layout")
            .snapshot();
    let block_start = layout
        .rows
        .iter()
        .position(|row| row.text.contains("短说明"))
        .expect("expected opening docstring row");
    let block_end = layout
        .rows
        .iter()
        .rposition(|row| row.text.trim() == "\"\"\"")
        .expect("expected closing docstring row");
    let rows = &layout.rows[block_start..=block_end];

    assert!(rows.len() > 4, "expected the long docstring row to wrap");
    assert!(
        rows.iter()
            .all(|row| background_bounds_at_level(row, 1) == Some((0, 48))),
        "expected docstring content, blank, and continuation rows to share the exact non-viewport-wide bbox: {rows:?}"
    );
}

#[test]
fn python_docstring_block_left_edge_starts_at_triple_quotes() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let docstring_region = find_nested_region(
        &analysis,
        "python_docstring",
        "Initialize the preview instance.",
        &source,
    );
    let first_segment = segment_for_line(
        docstring_region,
        &source,
        "\"\"\"Initialize the preview instance.",
    );
    let line_start = line_start_containing(&source, "\"\"\"Initialize the preview instance.");
    let quote_column = display_width(
        &source[line_start
            ..source[line_start..]
                .find("\"\"\"Initialize the preview instance.")
                .map(|offset| line_start + offset)
                .expect("expected init docstring line")],
    )
    .as_usize();

    assert_eq!(
        segment_left_column(&source, first_segment),
        quote_column,
        "expected python docstring block left edge to align with opening triple quotes"
    );
}

#[test]
fn python_docstring_blank_lines_keep_block_tint_geometry() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 120);
    let args_row = layout_row_containing(&layout.rows, "        Args:");
    let raises_row = layout_row_containing(&layout.rows, "        Raises:");
    let blank_after_intro = layout
        .rows
        .iter()
        .find(|row| row.source_line_start == line_start_for_line_number(&source, 24))
        .expect("expected blank line between init docstring intro and Args section");
    let blank_before_raises = layout
        .rows
        .iter()
        .find(|row| row.source_line_start == line_start_for_line_number(&source, 28))
        .expect("expected blank line between init docstring Args and Raises sections");

    assert_eq!(
        blank_after_intro
            .background_runs
            .first()
            .map(|run| (run.start_column, run.end_column)),
        args_row
            .background_runs
            .first()
            .map(|run| (run.start_column, run.end_column)),
        "expected blank line after the init summary to keep the same docstring block tint as Args"
    );
    assert_eq!(
        blank_before_raises
            .background_runs
            .first()
            .map(|run| (run.start_column, run.end_column)),
        raises_row
            .background_runs
            .first()
            .map(|run| (run.start_column, run.end_column)),
        "expected blank line before Raises to keep the same docstring block tint as Raises"
    );
}

#[test]
fn python_docstring_doctest_highlights_local_bindings_and_receivers() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let doctest_region = find_nested_region(
        &analysis,
        "python",
        ">>> preview = ThemePreview(\"Dracula\")",
        &source,
    );
    let local_style = theme
        .token_style_for("variable.local", "preview")
        .expect("expected local variable style")
        .snapshot(ColorMode::TrueColor);
    let assignment_preview = source
        .find("preview = ThemePreview(\"Dracula\")")
        .expect("expected doctest assignment preview");
    let receiver_preview = source
        .find("preview.render()")
        .expect("expected doctest receiver preview");

    for offset in [assignment_preview, receiver_preview] {
        assert!(
            doctest_region.overlays.iter().any(|span| {
                span.start <= offset && offset < span.end && span.style == Some(local_style.clone())
            }),
            "expected doctest preview at byte offset {offset} to reuse local variable styling"
        );
    }
}

#[test]
fn python_docstring_doctest_child_python_overlays_reach_final_analysis_spans() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let local_style = theme
        .token_style_for("variable.local", "preview")
        .expect("expected local variable style")
        .snapshot(ColorMode::TrueColor);
    let method_style = theme
        .token_style_for("function.method.call", "render")
        .expect("expected method call style")
        .snapshot(ColorMode::TrueColor);

    for (needle, style) in [
        ("preview = ThemePreview(\"Dracula\")", local_style.clone()),
        ("preview.render()", local_style.clone()),
        ("render()", method_style),
    ] {
        let offset = source.find(needle).expect("expected doctest fragment");
        assert!(
            analysis.spans.iter().any(|span| {
                span.start <= offset && offset < span.end && span.style == Some(style.clone())
            }),
            "expected final analysis spans to retain doctest child overlay for {needle}"
        );
    }
}

#[test]
fn javascript_python_rust_and_regex_hosts_reuse_nested_sql_and_regex_runtimes() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let js_path = fixture_path("javascript/injections.js");
    let js_source = read_file(&js_path);
    let js_analysis = analysis_snapshot_for_path(js_path.as_path(), &js_source, &theme);
    let js_runtimes = nested_runtime_names(&js_analysis);
    assert!(js_runtimes.contains(&"sql_postgres"));
    assert!(js_runtimes.contains(&"sql_mysql"));
    assert!(js_runtimes.contains(&"regex_javascript"));

    let python_path = fixture_path("python/injections.py");
    let python_source = read_file(&python_path);
    let python_analysis = analysis_snapshot_for_path(python_path.as_path(), &python_source, &theme);
    let python_runtimes = nested_runtime_names(&python_analysis);
    assert!(python_runtimes.contains(&"sql_sqlite"));
    assert!(python_runtimes.contains(&"regex_python"));

    let rust_path = fixture_path("rust/injections.rs");
    let rust_source = read_file(&rust_path);
    let rust_analysis = analysis_snapshot_for_path(rust_path.as_path(), &rust_source, &theme);
    let rust_runtimes = nested_runtime_names(&rust_analysis);
    assert!(rust_runtimes.contains(&"sql"));
    assert!(rust_runtimes.contains(&"regex_rust"));
}

#[test]
fn typescript_and_tsx_hosts_reuse_comment_led_template_injection_runtimes() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let ts_path = fixture_path("typescript/injections.ts");
    let ts_source = read_file(&ts_path);
    let ts_analysis = analysis_snapshot_for_path(ts_path.as_path(), &ts_source, &theme);
    let ts_runtimes = nested_runtime_names(&ts_analysis);
    assert!(ts_runtimes.contains(&"sql"));
    assert!(ts_runtimes.contains(&"graphql"));
    assert!(ts_runtimes.contains(&"html"));
    assert!(ts_runtimes.contains(&"css"));

    let tsx_path = fixture_path("tsx/injections.tsx");
    let tsx_source = read_file(&tsx_path);
    let tsx_analysis = analysis_snapshot_for_path(tsx_path.as_path(), &tsx_source, &theme);
    let tsx_runtimes = nested_runtime_names(&tsx_analysis);
    assert!(tsx_runtimes.contains(&"sql"));
    assert!(tsx_runtimes.contains(&"graphql"));
    assert!(tsx_runtimes.contains(&"html"));
    assert!(tsx_runtimes.contains(&"css"));
}

#[test]
fn html_highlights_entities_custom_elements_and_nested_language_regions() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("html/rich.html");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198msection"),
        "expected html element styling for section"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m&amp;"),
        "expected entity styling for &amp;"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198mtheme-card"),
        "expected custom element tag styling for theme-card"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123maccent"),
        "expected html attribute styling for accent"
    );
}

#[test]
fn yaml_highlights_anchors_tags_block_scalars_and_github_script_injection() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("yaml/actions.yaml");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[4m\x1b[38;2;80;250;123m&defaults"),
        "expected YAML anchor styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[4m\x1b[38;2;80;250;123m*defaults"),
        "expected YAML alias styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m!color"),
        "expected YAML tag styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m|"),
        "expected YAML block-scalar indicator styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected YAML github-script block to inject JavaScript"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198mconst"),
        "expected YAML github-script block to reuse JavaScript keyword styling"
    );
}

#[test]
fn github_actions_advanced_workflow_inherits_default_shell_and_highlights_nested_expressions() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let workflow_path = fixture_path("yaml/github-actions-workflow-advanced.yaml");
    let workflow_source = read_file(&workflow_path);
    let workflow_rendered = render_with_theme(
        Some(Path::new(".github/workflows/example.yml")),
        &workflow_source,
        &theme,
    )
    .unwrap_or_else(|error| panic!("failed to render {}: {error}", workflow_path.display()));
    let workflow_regions = collect_top_level_injection_regions(
        yaml_document_kind(Some(Path::new(".github/workflows/example.yml"))),
        &workflow_source,
        &theme,
        1,
        None,
    )
    .expect("expected advanced GitHub Actions workflow injections to resolve");

    assert!(
        workflow_regions.iter().any(|region| {
            region.overlays.iter().any(|span| {
                &workflow_source[span.range.clone()] == "set"
                    && span.style == theme.token_style_for("function.builtin", "set")
            })
        }),
        "expected defaults.run.shell to drive Bash highlighting for run blocks"
    );
    assert!(
        workflow_regions.iter().any(|region| {
            region.overlays.iter().any(|span| {
                &workflow_source[span.range.clone()] == "print"
                    && span.style == theme.token_style_for("function.builtin", "print")
            })
        }),
        "expected shell templates like python {{0}} to resolve to the Python runtime"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;255;121;198m&&"),
        "expected bare if expressions to reuse GitHub Actions operator styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;255;121;198m||"),
        "expected bare if expressions to reuse GitHub Actions operator styling"
    );
    assert!(
        workflow_regions.iter().any(|region| {
            region.overlays.iter().any(|span| {
                &workflow_source[span.range.clone()] == "${{"
                    && span.style == theme.token_style_for("punctuation.special", "${{")
            })
        }),
        "expected GitHub Actions expressions inside run blocks to keep workflow punctuation styling"
    );
    assert!(
        workflow_regions.iter().any(|region| {
            region.overlays.iter().any(|span| {
                &workflow_source[span.range.clone()] == "target"
                    && span.style == theme.token_style_for("property", "target")
            })
        }),
        "expected nested run-block expressions to highlight GitHub Actions property segments"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;255;121;198mdocker://"),
        "expected docker uses refs to highlight the docker:// prefix"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;80;250;123msetup-node"),
        "expected uses refs with subpaths to keep repository highlighting"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;241;250;140mbin"),
        "expected uses refs with subpaths to highlight the nested action path"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;139;233;253mWrite-Host"),
        "expected pwsh run blocks to reuse PowerShell builtin highlighting"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;255;121;198m@echo off"),
        "expected cmd run blocks to reuse batch highlighting"
    );
    assert!(
        workflow_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mpwsh"),
        "expected shell values to receive schema-aware runtime styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mcmd"),
        "expected cmd shell values to receive schema-aware runtime styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mread"),
        "expected permissions values to receive schema-aware styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mpnpm"),
        "expected with.cache values to receive schema-aware styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[3m\x1b[38;2;139;233;253merror"),
        "expected with.if-no-files-found values to receive schema-aware styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;139;233;253mubuntu-latest"),
        "expected runs-on labels to receive GitHub Actions runner styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;139;233;253mself-hosted"),
        "expected runs-on array labels to receive GitHub Actions runner styling"
    );
}

#[test]
fn nested_region_tint_levels_scale_monotonically_for_five_layers() {
    let tint = RgbColor(16, 24, 32);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let mut previous_luminance = relative_luminance(tint);

    for level in 1..=5 {
        let background = theme
            .nested_region_background(level)
            .unwrap_or_else(|| panic!("expected nested region tint for level {level}"));
        let luminance = relative_luminance(background);
        assert!(
            luminance > previous_luminance,
            "nested region level {level} should be brighter than the previous level"
        );
        previous_luminance = luminance;
    }
}

#[test]
fn overlay_nested_region_merges_parent_style_only_inside_child_overlays() {
    let theme = Theme::for_mode(ColorMode::Ansi);
    let parent_style = None;
    let child_style = theme.token_style_for("comment", "//");
    let parent_spans = vec![StyledSpan {
        range: 0..10,
        style: parent_style,
        hyperlink: None,
    }];
    let region = NestedRegion {
        visual_level: 0,
        resolved_document_kind: plain_document_kind("sql"),
        visual_kind: InjectionVisualKind::Transparent,
        layout_segments: Vec::<RegionSegment>::new(),
        child_regions: Vec::new(),
        child_nested_regions: Vec::new(),
        overlays: vec![StyledSpan {
            range: 2..4,
            style: child_style,
            hyperlink: None,
        }],
        merge_parent_styles: true,
    };

    let result = overlay_nested_region(parent_spans, &region);

    assert_eq!(
        result,
        vec![
            StyledSpan {
                range: 0..2,
                style: parent_style,
                hyperlink: None,
            },
            StyledSpan {
                range: 2..4,
                style: child_style,
                hyperlink: None,
            },
            StyledSpan {
                range: 4..10,
                style: parent_style,
                hyperlink: None,
            },
        ]
    );
}

#[test]
fn typescript_comment_hint_template_injection_stays_inline() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let source = "const query = /* sql */ `\n  select 1\n`;\n";
    let analysis = analysis_snapshot_for_path(Path::new("demo.ts"), source, &theme);
    let region = find_nested_region(&analysis, "sql", "select 1", source);

    assert!(
        region.visual_kind == "transparent",
        "expected inline SQL template injection to avoid block tint"
    );
}

#[test]
fn markdown_top_level_prose_does_not_pick_up_block_tint_from_inline_injections() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("markdown/markdown_truecolor_cjk_repro.md");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let prose_lines = [
        line_start_for_line_number(&source, 0),
        line_start_for_line_number(&source, 1),
    ];

    assert!(
        !nested_regions(&analysis.nested_regions)
            .into_iter()
            .any(|region| {
                region.visual_kind == "rect_block"
                    && region
                        .layout_segments
                        .iter()
                        .any(|segment| segment.line_start == prose_lines[0])
            }),
        "expected top-level markdown prose line to avoid block tint"
    );
    assert!(
        !nested_regions(&analysis.nested_regions)
            .into_iter()
            .any(|region| {
                region.visual_kind == "rect_block"
                    && region
                        .layout_segments
                        .iter()
                        .any(|segment| segment.line_start == prose_lines[1])
            }),
        "expected second top-level markdown prose line to avoid block tint"
    );
}

#[test]
fn decoded_unicode_after_raw_text_has_one_source_interval() {
    let source = r"ab\U0001F7E3";
    let (text, map) = crate::projection::build_virtual_source(
        source,
        &std::iter::once(0..source.len()).collect::<Vec<_>>(),
        false,
        crate::host_injections::InjectionDecode::PythonString,
    );
    assert_eq!(text, "ab🟣");
    let spans = [StyledSpan {
        range: 0..text.len(),
        style: None,
        hyperlink: None,
    }];
    let mapped = crate::projection::map_virtual_spans_to_source(&spans, &map);
    assert_eq!(mapped.len(), 1);
    assert_eq!(mapped[0].range, 0..source.len());
}

#[test]
fn unicode_escape_projection_consumes_delimiters_and_maps_the_full_source() {
    use crate::host_injections::InjectionDecode;
    for decode in [
        InjectionDecode::RustString,
        InjectionDecode::JavaScriptString,
    ] {
        for (source, expected) in [
            (r"before\u{41}after", "beforeAafter"),
            (r"before\u{1f7e3}after", "before🟣after"),
            (r"before\u{01f7e3}after", "before🟣after"),
            (r"before\u{110000}after", r"before\u{110000}after"),
            (r"before\u{}after", r"before\u{}after"),
            (r"before\u{xyz}after", r"before\u{xyz}after"),
            (r"before\u{1f7e3", r"before\u{1f7e3"),
        ] {
            let quoted = format!("\"{source}\"");
            let range = 1..quoted.len() - 1;
            let (text, map) = crate::projection::build_virtual_source(
                &quoted,
                &std::iter::once(range.clone()).collect::<Vec<_>>(),
                false,
                decode,
            );
            assert_eq!(text, expected, "{decode:?}: {source}");
            assert_eq!(
                map.ranges(0..text.len()),
                std::iter::once(range).collect::<Vec<_>>()
            );
        }
    }
    for (decode, source, expected) in [
        (
            InjectionDecode::JavaScriptString,
            r"before\u263aafter",
            "before☺after",
        ),
        (
            InjectionDecode::PythonString,
            r"before\U0001F7E3after",
            "before🟣after",
        ),
        (
            InjectionDecode::RustString,
            r"before\x41after",
            "beforeAafter",
        ),
        (
            InjectionDecode::JavaScriptString,
            "before\\\nafter",
            "beforeafter",
        ),
    ] {
        let quoted = format!("\"{source}\"");
        let (text, _) = crate::projection::build_virtual_source(
            &quoted,
            &std::iter::once(1..quoted.len() - 1).collect::<Vec<_>>(),
            false,
            decode,
        );
        assert_eq!(text, expected);
    }
}

#[test]
fn unrecognized_unicode_escape_preserves_utf8_boundaries() {
    use crate::host_injections::InjectionDecode;
    for decode in [
        InjectionDecode::RustString,
        InjectionDecode::JavaScriptString,
        InjectionDecode::PythonString,
    ] {
        let source = r"before\中after";
        let quoted = format!("\"{source}\"");
        let (text, _) = crate::projection::build_virtual_source(
            &quoted,
            &std::iter::once(1..quoted.len() - 1).collect::<Vec<_>>(),
            false,
            decode,
        );
        assert_eq!(text, source);
    }
}
