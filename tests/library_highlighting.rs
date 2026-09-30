use std::path::Path;

use kat::{HighlightStyle, PreparedDocument, RenderOptions};

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
