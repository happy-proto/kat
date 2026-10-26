use std::path::Path;

use kat::{HighlightStyle, PreparedDocument, RenderOptions};

#[test]
fn caddyfile_detection_and_explicit_aliases() {
    let source = include_str!("../testdata/fixtures/caddyfile/Caddyfile");
    for path in [
        "Caddyfile",
        "/etc/caddy/Caddyfile",
        "caddyfile",
        "Caddyfile.dev",
        "caddyfile.local",
        "site.caddyfile",
    ] {
        assert_eq!(
            kat::detected_language_name(Some(Path::new(path)), source),
            Some("caddyfile"),
            "{path}"
        );
    }
    for path in ["caddy.conf", "site.conf", "CaddyfileLike"] {
        assert_ne!(
            kat::detected_language_name(Some(Path::new(path)), source),
            Some("caddyfile"),
            "{path}"
        );
    }
    assert_ne!(kat::detected_language_name(None, source), Some("caddyfile"));
    assert_eq!(
        kat::detected_language_name(Some(Path::new("caddy.json")), "{}"),
        Some("json")
    );
    for alias in ["caddyfile", "caddy", "Caddyfile"] {
        let explicit = PreparedDocument::named_language(
            alias,
            source,
            RenderOptions::with_colors((248, 248, 242), (40, 42, 54)),
        )
        .unwrap();
        assert_eq!(
            explicit.spans(),
            document("Caddyfile", source).spans(),
            "{alias}"
        );
    }
}

#[test]
fn caddyfile_highlights_configuration_tokens() {
    let source = include_str!("../testdata/fixtures/caddyfile/Caddyfile");
    for (needle, expected) in [
        ("# Synthetic", (98, 114, 164)),
        ("reverse_proxy", (255, 121, 198)),
        ("example.com,", (241, 250, 140)),
        ("5s", (255, 184, 108)),
        ("\\n\" 200", (255, 121, 198)),
        ("200", (255, 184, 108)),
        ("{$LOG_FILE", (189, 147, 249)),
    ] {
        assert_eq!(
            style_at(source, needle, 0, "Caddyfile").foreground,
            expected,
            "{needle}"
        );
    }
}

#[test]
fn pgpass_detection_uses_exact_filenames_without_content_guessing() {
    let source = "localhost:5432:demo:reader:example-only\n";
    for path in [
        ".pgpass",
        "/home/example/.pgpass",
        "pgpass.conf",
        "postgresql/pgpass.conf",
    ] {
        assert_eq!(
            kat::detected_language_name(Some(Path::new(path)), source),
            Some("pgpass"),
            "{path}"
        );
    }
    for path in [
        ".pgpass.bak",
        "pgpass.conf.bak",
        "custom.passfile",
        "example.pgpass",
    ] {
        assert_ne!(
            kat::detected_language_name(Some(Path::new(path)), source),
            Some("pgpass"),
            "{path}"
        );
    }
    assert_ne!(kat::detected_language_name(None, source), Some("pgpass"));
}

#[test]
fn pgpass_highlights_fields_wildcards_escapes_and_literal_password_data() {
    let source = include_str!("../testdata/fixtures/pgpass/.pgpass");
    for (needle, expected) in [
        ("# Synthetic", (98, 114, 164)),
        ("localhost", (139, 233, 253)),
        ("5432", (255, 184, 108)),
        ("demo:", (139, 233, 253)),
        ("reader:", (189, 147, 249)),
        ("example-only", (241, 250, 140)),
        ("*:*:replication", (255, 121, 198)),
        ("# literal", (241, 250, 140)),
        ("虚构口令", (241, 250, 140)),
        ("\\:db8", (255, 121, 198)),
        ("\\\\word", (255, 121, 198)),
    ] {
        assert_eq!(
            style_at(source, needle, 0, ".pgpass").foreground,
            expected,
            "{needle}"
        );
    }
    let password_star = source.find("*:*:*:*:*").unwrap() + 8;
    let prepared = document(".pgpass", source);
    assert_eq!(
        prepared
            .spans()
            .iter()
            .find(|span| span.range.contains(&password_star))
            .unwrap()
            .style
            .foreground,
        (241, 250, 140)
    );
    let explicit = PreparedDocument::named_language(
        "pgpass",
        source,
        RenderOptions::with_colors((248, 248, 242), (40, 42, 54)),
    )
    .unwrap();
    assert_eq!(explicit.spans(), prepared.spans());
    assert_eq!(
        document("postgresql/pgpass.conf", source).spans(),
        prepared.spans()
    );
}

#[test]
fn tmtheme_files_detect_and_highlight_as_xml() {
    let source = include_str!("../testdata/fixtures/xml/theme.tmTheme");
    for path in ["theme.tmTheme", ".codex/themes/custom.tmTheme"] {
        assert_eq!(
            kat::detected_language_name(Some(Path::new(path)), source),
            Some("xml"),
            "{path}"
        );
        assert_eq!(
            document(path, source).spans(),
            document("theme.xml", source).spans()
        );
    }
    for (needle, offset, expected) in [
        ("<plist", 1, (255, 121, 198)),
        ("<key>", 1, (255, 121, 198)),
        ("<plist version", 7, (248, 248, 242)),
        ("version=\"1.0\">", 9, (241, 250, 140)),
        ("Default colors", 0, (98, 114, 164)),
        ("&amp;", 0, (189, 147, 249)),
    ] {
        assert_eq!(
            style_at(source, needle, offset, "theme.tmTheme").foreground,
            expected,
            "{needle}"
        );
    }
    assert_ne!(
        kat::detected_language_name(Some(Path::new("theme.tmTheme.bak")), source),
        Some("xml")
    );
}

#[test]
fn public_key_trailing_comment_has_one_comment_style() {
    let source = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIExample dcjanusmacbook-pro tailnet\n";
    assert_eq!(
        kat::detected_language_name(Some(Path::new("id_ed25519.pub")), source),
        Some("authorized_keys")
    );
    for needle in ["dcjanusmacbook-pro", "tailnet"] {
        let style = style_at(source, needle, 0, "id_ed25519.pub");
        assert_eq!(style.foreground, (98, 114, 164), "{needle}");
        assert!(style.italic, "{needle}");
    }
    assert_eq!(
        style_at(source, "ssh-ed25519", 0, "id_ed25519.pub").foreground,
        (139, 233, 253)
    );
}

#[test]
fn public_key_base64_slash_suffix_keeps_the_key_blob_style() {
    let source = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIExample/8\n";
    for needle in ["AAAAC3", "Example", "/8"] {
        assert_eq!(
            style_at(source, needle, 0, "id_ed25519.pub").foreground,
            (189, 147, 249),
            "{needle}"
        );
    }
}

#[test]
fn known_hosts_detection_uses_exact_filenames_without_content_guessing() {
    let source = "example.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIExample\n";
    for path in [
        "known_hosts",
        ".ssh/known_hosts",
        "./.ssh/known_hosts",
        "/home/alice/.ssh/known_hosts",
        "ssh_known_hosts",
        "/etc/ssh/ssh_known_hosts",
    ] {
        assert_eq!(
            kat::detected_language_name(Some(Path::new(path)), source),
            Some("known_hosts"),
            "{path}"
        );
    }
    for path in [
        "config",
        "project/config",
        "known_hosts.bak",
        "my_known_hosts",
    ] {
        assert_ne!(
            kat::detected_language_name(Some(Path::new(path)), source),
            Some("known_hosts"),
            "{path}"
        );
    }
    assert_ne!(
        kat::detected_language_name(None, source),
        Some("known_hosts")
    );
}

#[test]
fn ssh_config_detection_remains_distinct_from_known_hosts() {
    for path in [
        "ssh_config",
        "sshd_config",
        "/etc/ssh/sshd_config",
        ".ssh/config",
        "/home/alice/.ssh/config",
    ] {
        assert_eq!(
            kat::detected_language_name(Some(Path::new(path)), "Host example\n"),
            Some("ssh_config"),
            "{path}"
        );
    }
    assert_eq!(
        kat::detected_language_name(
            Some(Path::new("authorized_keys")),
            "ssh-ed25519 AAAA comment\n"
        ),
        Some("authorized_keys")
    );
}

#[test]
fn known_hosts_styles_distinguish_hosts_markers_and_opaque_keys() {
    let source = include_str!("../testdata/fixtures/known_hosts/known_hosts");
    for needle in [
        "example.com",
        "192.0.2.1",
        "[example.net]:2222",
        "*.example.org",
        "excluded.example.org",
        "|1|c2FsdA==|aGFzaA==",
        "ssh-ed25519",
    ] {
        assert_eq!(
            style_at(source, needle, 0, "known_hosts").foreground,
            (139, 233, 253),
            "{needle}"
        );
    }
    let revoked = style_at(source, "@revoked", 0, "known_hosts");
    assert_eq!(revoked.foreground, (255, 184, 108));
    assert!(revoked.bold);
    assert_eq!(
        style_at(source, "@cert-authority", 0, "known_hosts").foreground,
        (255, 121, 198)
    );
    assert_eq!(
        style_at(source, "AAAAIExample", 0, "known_hosts").foreground,
        (98, 114, 164)
    );
    assert_eq!(
        style_at(source, "example host", 0, "known_hosts").foreground,
        (98, 114, 164)
    );
    let explicit = PreparedDocument::named_language(
        "known-hosts",
        source,
        RenderOptions::with_colors((248, 248, 242), (40, 42, 54)),
    )
    .unwrap();
    assert_eq!(explicit.spans(), document("known_hosts", source).spans());
}

fn document(path: &str, source: &str) -> PreparedDocument {
    PreparedDocument::detect(
        Some(Path::new(path)),
        source,
        RenderOptions::with_colors((248, 248, 242), (40, 42, 54)),
    )
    .unwrap()
}

#[test]
fn library_exposes_backgrounds_for_every_nested_region_kind() {
    let cases = [
        ("justfile", "build:\n    echo hello\n", "echo"),
        ("README.md", "```rust\nfn main() {}\n```\n", "fn main"),
    ];
    for (path, source, body) in cases {
        let document = document(path, source);
        let line = source[..source.find(body).unwrap()]
            .bytes()
            .filter(|b| *b == b'\n')
            .count();
        let backgrounds = document.background_lines(4);
        assert!(
            backgrounds[line]
                .iter()
                .any(|run| run.end_column > run.start_column),
            "{path}"
        );
        assert!(
            backgrounds[0].is_empty(),
            "header should remain unshaded: {path}"
        );
    }
}

#[test]
fn just_recipe_nested_lines_share_a_rectangular_background() {
    let source = "demo:\n    echo outer\n        echo nested\n";
    let backgrounds = document("justfile", source).background_lines(4);
    assert_eq!(
        backgrounds[1][0].start_column,
        backgrounds[2][0].start_column
    );
    assert_eq!(backgrounds[1][0].end_column, backgrounds[2][0].end_column);
}

fn style_at(source: &str, needle: &str, offset: usize, path: &str) -> HighlightStyle {
    let position = source.find(needle).expect("needle in source") + offset;
    document(path, source)
        .spans()
        .into_iter()
        .find(|span| span.range.start <= position && position < span.range.end)
        .expect("styled needle")
        .style
}

#[test]
fn source_spans_use_original_utf8_offsets_and_dracula_colors() {
    let source = "fn main() { let café = \"hello\"; }\n";
    let spans = document("main.rs", source).spans();

    assert!(!spans.is_empty());
    for span in &spans {
        assert!(span.range.start < span.range.end);
        assert!(source.get(span.range.clone()).is_some());
    }
    assert_eq!(
        style_at(source, "hello", 0, "main.rs").foreground,
        (241, 250, 140)
    );
}

#[test]
fn nested_markdown_rust_uses_the_same_syntax_style() {
    let rust = "fn nested() {}\n";
    let markdown = "```rust\nfn nested() {}\n```\n";
    assert_eq!(
        style_at(rust, "fn", 0, "main.rs"),
        style_at(markdown, "fn", 0, "README.md"),
    );
}

#[test]
fn unknown_file_has_no_syntax_spans() {
    assert!(document("mystery.unknown", "hello\n").spans().is_empty());
}

#[test]
fn display_rows_preserve_zero_width_text_and_real_blank_lines() {
    let rows = kat::PreparedDocument::plain("\u{200b}\n\n").display_rows(40);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].text(), "\u{200b}");
    assert_eq!(rows[0].display_width(), 0);
    assert_eq!(rows[1].text(), "");
    assert_eq!(rows[1].display_width(), 0);
}

#[test]
fn nested_analysis_timings_do_not_count_child_work_twice() {
    let payload = (0..200)
        .map(|i| format!("{{\"id\":{i},\"text\":\"synthetic\"}}"))
        .collect::<Vec<_>>()
        .join(",\n");
    let mut source = format!("```json\n[{payload}]\n```\n");
    for width in 4..9 {
        let fence = "`".repeat(width);
        source = format!("{fence}markdown\n{source}{fence}\n");
    }
    let options = kat::RenderOptions::with_colors((255, 255, 255), (0, 0, 0));
    kat::PreparedDocument::named_language("markdown", &source, options)
        .unwrap()
        .render(None);
    let started = std::time::Instant::now();
    let rendered = kat::PreparedDocument::named_language("markdown", &source, options)
        .unwrap()
        .render(None);
    let elapsed = started.elapsed();
    assert!(
        rendered.timings.total_render_pipeline() <= elapsed,
        "pipeline {:?} exceeded wall {:?}",
        rendered.timings.total_render_pipeline(),
        elapsed
    );
}

#[test]
fn json_without_analysis_capabilities_does_not_parse_an_extra_tree() {
    let output = kat::PreparedDocument::named_language(
        "json",
        "{\"name\":\"synthetic\"}",
        kat::RenderOptions::with_colors((255, 255, 255), (0, 0, 0)),
    )
    .unwrap()
    .render(Some(40));
    assert_eq!(output.timings.analysis_parse, std::time::Duration::ZERO);
    assert_eq!(output.timings.semantic_overlays, std::time::Duration::ZERO);
    assert!(output.timings.layout > std::time::Duration::ZERO);
}

#[test]
fn display_rows_distinguish_source_controls_from_zero_width_text() {
    let source = "a\0b\x1b[2J\rc\u{200b}";
    let rows = kat::PreparedDocument::plain(source).display_rows(40);
    assert_eq!(rows[0].text(), "a␀b␛[2J␍c\u{200b}");
    assert_eq!(rows[0].display_width(), 9);
    let encoded = rows[0].encode(&[], None);
    assert!(!encoded.contains('\x1b'));
    assert!(!encoded.contains('\r'));
}

#[test]
fn crlf_and_lf_have_the_same_terminal_geometry() {
    let source = "# title\n\n```bash\necho 这里是一段代码\n```\n";
    let options = kat::RenderOptions::with_colors((255, 255, 255), (0, 0, 0));
    let lf = kat::PreparedDocument::named_language("markdown", source, options)
        .unwrap()
        .display_rows(20);
    let crlf =
        kat::PreparedDocument::named_language("markdown", &source.replace('\n', "\r\n"), options)
            .unwrap()
            .display_rows(20);
    let rows = |rows: &[kat::DisplayRow]| {
        rows.iter()
            .map(|row| (row.text().to_owned(), row.display_width()))
            .collect::<Vec<_>>()
    };
    assert_eq!(rows(&lf), rows(&crlf));
}

#[test]
fn expanded_tabs_wrap_as_display_spaces_at_narrow_widths() {
    let rows = kat::PreparedDocument::plain("abcd\t012").display_rows(3);
    let text = rows.iter().map(|row| row.text()).collect::<Vec<_>>();
    assert_eq!(text, ["abc", "d  ", "  0", "12"]);
    assert!(rows.iter().all(|row| row.display_width() <= 3));
    assert_eq!(rows[2].anchor().wrapped_from_column, 6);
}
