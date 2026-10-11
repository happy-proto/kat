use std::path::Path;

use kat::{PreparedDocument, RenderOptions};

#[test]
fn recognizes_ghostty_paths_without_claiming_generic_config_files() {
    for path in [
        "config.ghostty",
        "custom.ghostty",
        "/home/user/.config/ghostty/config",
        "ghostty/config",
        "/Users/user/Library/Application Support/com.mitchellh.ghostty/config",
        "/usr/share/ghostty/themes/Dracula",
        "ghostty/themes/My Theme",
        "ghostty/themes/Light.toml",
    ] {
        assert_eq!(
            kat::detected_language_name(Some(Path::new(path)), "font-size = 14\n"),
            Some("ghostty"),
            "{path}"
        );
    }
    for path in ["config", "other/config", "themes/Dracula", "ghostty/README"] {
        assert_ne!(
            kat::detected_language_name(Some(Path::new(path)), "font-size = 14\n"),
            Some("ghostty"),
            "{path}"
        );
    }
}

#[test]
fn highlights_literal_hashes_opaque_values_and_bare_keys() {
    let source = "# comment\ntitle = # literal = true\nfont-family = \"JetBrains Mono\"\ncursor-click-to-move\n";
    let document = PreparedDocument::detect(
        Some(Path::new("config.ghostty")),
        source,
        RenderOptions::with_colors((248, 248, 242), (40, 42, 54)),
    )
    .unwrap();
    let style = |needle: &str| {
        let offset = source.find(needle).unwrap();
        document
            .spans()
            .into_iter()
            .find(|span| span.range.contains(&offset))
            .unwrap()
            .style
    };
    assert_ne!(style("# comment"), style("# literal"));
    assert_eq!(style("# literal"), style("true"));
    assert_eq!(style("title"), style("cursor-click-to-move"));
    assert_eq!(style("# literal"), style("JetBrains"));
    assert_ne!(style("title"), style("# literal"));
}
