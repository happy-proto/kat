use super::*;

#[test]
fn debug_semantics_is_empty_for_languages_without_overlays() {
    for language in ["pem", "openpgp_armor", "python_docstring", "json"] {
        assert_eq!(
            crate::api::debug_semantics(language, "plain text").unwrap(),
            "",
            "{language} has no semantic overlays"
        );
    }
}

#[test]
fn proto_annotations_highlight_options_http_templates_and_cel() {
    let path = fixture_path("proto/annotations.proto");
    let source = read_file(&path);
    let json = debug_analysis_json(Some(path.as_path()), &source)
        .expect("proto annotations should be analyzed");
    let analysis: Value = serde_json::from_str(&json).expect("analysis JSON should parse");
    let color_at = |needle: &str| {
        let offset = source.find(needle).expect("fixture fragment should exist");
        analysis["spans"]
            .as_array()
            .expect("analysis should have spans")
            .iter()
            .find(|span| {
                span["start"]
                    .as_u64()
                    .is_some_and(|start| start <= offset as u64)
                    && span["end"].as_u64().is_some_and(|end| end > offset as u64)
            })
            .and_then(|span| span["style"]["color_name"].as_str())
            .unwrap_or("none")
            .to_owned()
    };

    for option in ["buf.validate.field", "validate.rules", "google.api.http"] {
        assert_ne!(
            color_at(option),
            "foreground",
            "option {option} should be styled"
        );
    }
    for key in ["min_len", "additional_bindings", "expression:"] {
        assert_ne!(
            color_at(key),
            "foreground",
            "option key {key} should be styled"
        );
    }
    assert_ne!(color_at("parent=publishers/*"), "yellow");
    assert_eq!(color_at("**"), "pink");
    assert_ne!(color_at("create\""), "yellow");
    assert_ne!(color_at("has(this.title)"), "yellow");
    assert_eq!(color_at("parent != ''"), "cyan");
    assert_ne!(color_at("matches(\\\""), "yellow");
    assert_eq!(color_at("body: \"*\""), "cyan");
    assert_eq!(color_at("\"*\""), "yellow");
    let cel_regions = analysis["nested_regions"]
        .as_array()
        .expect("expected nested regions")
        .iter()
        .filter(|region| region["resolved_document_kind"]["runtime_name"] == "cel")
        .count();
    assert_eq!(
        cel_regions, 2,
        "field and message CEL should both be injected"
    );
}

#[test]
fn swift_nil_literal_is_highlighted_as_a_builtin_constant() {
    let path = fixture_path("swift/ThemePreview.swift");
    let source = read_file(&path);
    let json =
        debug_analysis_json(Some(path.as_path()), &source).expect("Swift should be analyzed");
    let analysis: Value = serde_json::from_str(&json).expect("analysis JSON should parse");
    let start = source.find("nil").expect("fixture should contain nil") as u64;
    assert!(analysis["spans"].as_array().unwrap().iter().any(|span| {
        span["start"].as_u64().is_some_and(|offset| offset <= start)
            && span["end"]
                .as_u64()
                .is_some_and(|offset| offset >= start + 3)
            && span["style"]["color_name"] == "purple"
    }));
}

#[test]
fn go_highlights_generics_methods_builtins_and_directives() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("go/rich.go");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m//go:build linux"),
        "expected go build directive styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;139;233;253mshowcase"),
        "expected package namespace styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mRenderer"),
        "expected go type styling for Renderer"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mNewRenderer"),
        "expected go function definition styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mRender"),
        "expected go method definition styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mmake"),
        "expected go builtin function styling for make"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;139;233;253mstring"),
        "expected go builtin type styling for string"
    );
}

#[test]
fn dockerfile_highlights_keys_ports_and_paths_with_more_specific_semantics() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.semantics");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253mAPP_HOME"),
        "expected ENV / ARG keys to use key-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253morg.opencontainers.image.source"),
        "expected LABEL keys to use key-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108m8080"),
        "expected EXPOSE ports to use number styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m/app"),
        "expected Dockerfile paths to use string styling"
    );
}

#[test]
fn dockerfile_highlights_mount_param_pairs_with_key_value_semantics() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.params");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253mtype"),
        "expected mount param keys to use key-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mtarget"),
        "expected mount param keys to use key-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mcache"),
        "expected mount param enum-like values to use type styling"
    );
    assert!(
        rendered.contains("CACHE_DIR") && rendered.contains("CACHE_STAGE"),
        "expected mount param expansion-based values to render"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mreadonly"),
        "expected bare mount flags to use constant styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108m1000"),
        "expected mount param numeric values to use number styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mlocked")
            && rendered.contains("\x1b[38;2;139;233;253mcache"),
        "expected mount param enum-like values to use type-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140mcache-") && rendered.contains("TARGETARCH"),
        "expected mount id values to keep string fragments while splitting embedded expansions"
    );
    assert!(
        rendered.contains("APP_ENV"),
        "expected env mount values to render as an independent identifier-like token"
    );
}

#[test]
fn dockerfile_highlights_json_form_commands_and_paths_with_host_semantics() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.exec");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253m\"python\""),
        "expected JSON-form CMD executable to use command-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m\"sh\""),
        "expected JSON-form ENTRYPOINT executable to use command-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m\"zsh\""),
        "expected SHELL executable to use command-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"app\"")
            && rendered.contains("\x1b[38;2;241;250;140m\"echo hi\""),
        "expected non-option JSON-form argv entries to remain string styled"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"/data\""),
        "expected JSON-form VOLUME paths to keep string styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m\"-m\"")
            && rendered.contains("\x1b[3m\x1b[38;2;80;250;123m\"-lc\"")
            && rendered.contains("\x1b[3m\x1b[38;2;80;250;123m\"--config\""),
        "expected JSON-form option argv entries to use attribute styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"./bin/server\""),
        "expected path-like JSON-form executables to keep path/string styling"
    );
}

#[test]
fn dockerfile_highlights_mount_type_families_with_key_aware_semantics() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.mount-kinds");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253msecret")
            && rendered.contains("\x1b[38;2;139;233;253mssh")
            && rendered.contains("\x1b[38;2;139;233;253mbind"),
        "expected mount type values to keep family/type styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140mapi-key")
            && rendered.contains("\x1b[38;2;241;250;140mdefault"),
        "expected secret/ssh ids to use string-like styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mrequired")
            && rendered.contains("\x1b[38;2;189;147;249mreadonly"),
        "expected bare flags in mount families to keep constant styling"
    );
    assert!(
        rendered.contains("API_KEY"),
        "expected env mount values to remain an independently rendered identifier-like token"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m./hack")
            && rendered.contains("\x1b[38;2;241;250;140m/work"),
        "expected bind mount source and target paths to keep string/path styling"
    );
}

#[test]
fn dockerfile_highlights_json_form_with_unified_command_option_path_env_and_expansion_semantics() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.exec-advanced");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"./bin/server\""),
        "expected path-like JSON-form executable to stay string styled"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m\"--config\"")
            && rendered.contains("\x1b[3m\x1b[38;2;80;250;123m\"--root\""),
        "expected JSON-form options to keep attribute styling across commands"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"/etc/app.toml\"")
            && rendered.contains("\x1b[38;2;241;250;140m\"/workspace\""),
        "expected path-like JSON-form argv values to use string/path styling"
    );
    assert!(
        rendered.contains("APP_ENV=prod"),
        "expected env-style JSON-form argv to render as its own structured token"
    );
    assert!(
        rendered.contains("PORT"),
        "expected JSON-form expansion argv to keep expansion semantics visible"
    );
}

#[test]
fn python_highlights_advanced_semantics() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/advanced.py");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;80;250;123mclassmethod"),
        "expected decorator builtin styling for classmethod"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mbuild_default"),
        "expected function definition styling for build_default"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123m__init__"),
        "expected constructor method styling for __init__"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;139;233;253mstr"),
        "expected builtin type styling for str annotations"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253misinstance"),
        "expected builtin predicate styling for isinstance"
    );
}

#[test]
fn sql_highlights_keywords_types_parameters_and_dollar_strings() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("sql/rich.sql");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;98;114;164m-- Dashboard query"),
        "expected SQL comment styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198mWITH"),
        "expected SQL keyword styling for WITH"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mCOUNT"),
        "expected SQL function-call styling for COUNT"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;255;184;108m$1"),
        "expected SQL parameter styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m$kat$dracula$kat$"),
        "expected SQL dollar-quoted string styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108m10"),
        "expected SQL numeric literal styling"
    );
}

#[test]
fn sql_dialect_runtimes_highlight_postgres_mysql_and_sqlite_specific_tokens() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let postgres_path = fixture_path("sql/schema.postgres.sql");
    let postgres_source = read_file(&postgres_path);
    let postgres_rendered =
        render_with_theme(Some(postgres_path.as_path()), &postgres_source, &theme).unwrap_or_else(
            |error| panic!("failed to render {}: {error}", postgres_path.display()),
        );
    assert!(
        postgres_rendered.contains("\x1b[38;2;139;233;253mjsonb"),
        "expected postgres runtime to highlight jsonb as builtin type"
    );
    assert!(
        postgres_rendered.contains("\x1b[38;2;255;121;198mUNLOGGED"),
        "expected postgres runtime to highlight UNLOGGED"
    );
    assert!(
        postgres_rendered.contains("\x1b[38;2;255;121;198mALWAYS"),
        "expected postgres runtime to highlight ALWAYS"
    );
    assert!(
        postgres_rendered.contains("\x1b[38;2;255;121;198mRETURNING"),
        "expected postgres runtime to highlight RETURNING"
    );
    assert!(
        postgres_rendered.contains("\x1b[38;2;241;250;140m$tag$dracula$tag$"),
        "expected postgres runtime to highlight dollar-quoted strings"
    );

    let mysql_path = fixture_path("sql/schema.mysql.sql");
    let mysql_source = read_file(&mysql_path);
    let mysql_rendered = render_with_theme(Some(mysql_path.as_path()), &mysql_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", mysql_path.display()));
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mAUTO_INCREMENT"),
        "expected mysql runtime to highlight AUTO_INCREMENT"
    );
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mENGINE"),
        "expected mysql runtime to highlight ENGINE"
    );
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mZEROFILL"),
        "expected mysql runtime to highlight ZEROFILL"
    );
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mREPLACE"),
        "expected mysql runtime to highlight REPLACE"
    );

    let sqlite_path = fixture_path("sql/schema.sqlite.sql");
    let sqlite_source = read_file(&sqlite_path);
    let sqlite_rendered = render_with_theme(Some(sqlite_path.as_path()), &sqlite_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", sqlite_path.display()));
    assert!(
        sqlite_rendered.contains("\x1b[38;2;255;121;198mWITHOUT"),
        "expected sqlite runtime to highlight WITHOUT"
    );
}

#[test]
fn sql_dialect_runtimes_highlight_deeper_dialect_specific_constructs() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let postgres_rendered = highlight_named_language(
            "sql_postgres",
            "CREATE FUNCTION refresh_theme_cache() RETURNS void LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE AS $$ BEGIN RETURN; END; $$;\nCREATE INDEX theme_payload_gin ON theme_snapshots USING GIN (payload jsonb_path_ops);",
            &theme,
        )
        .expect("failed to highlight postgres SQL source");
    assert!(
        postgres_rendered.contains("\x1b[38;2;255;121;198mLANGUAGE"),
        "expected postgres runtime to highlight LANGUAGE"
    );
    assert!(
        postgres_rendered.contains("\x1b[38;2;255;121;198mIMMUTABLE"),
        "expected postgres runtime to highlight IMMUTABLE"
    );
    assert!(
        postgres_rendered.contains("\x1b[38;2;255;121;198mPARALLEL"),
        "expected postgres runtime to highlight PARALLEL"
    );
    assert!(
        postgres_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mGIN"),
        "expected postgres runtime to highlight GIN as a builtin index method"
    );

    let mysql_rendered = highlight_named_language(
            "sql_mysql",
            "CREATE TABLE theme_snapshots (id INT UNSIGNED NOT NULL AUTO_INCREMENT, name VARCHAR(255) CHARACTER SET utf8mb4, PRIMARY KEY (id)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;\nINSERT HIGH_PRIORITY IGNORE INTO theme_snapshots (id, name) VALUES (1, 'Dracula');\nREPLACE DELAYED LOW_PRIORITY INTO theme_snapshots (id, name) VALUES (2, 'Kat');",
            &theme,
        )
        .expect("failed to highlight mysql SQL source");
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mSET"),
        "expected mysql runtime to highlight SET inside CHARACTER SET"
    );
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mHIGH_PRIORITY"),
        "expected mysql runtime to highlight HIGH_PRIORITY"
    );
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mIGNORE"),
        "expected mysql runtime to highlight IGNORE"
    );
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mDELAYED"),
        "expected mysql runtime to highlight DELAYED"
    );
    assert!(
        mysql_rendered.contains("\x1b[38;2;255;121;198mUNSIGNED"),
        "expected mysql runtime to highlight UNSIGNED as a type qualifier"
    );

    let sqlite_rendered = highlight_named_language(
            "sql_sqlite",
            "PRAGMA foreign_keys = ON;\nVACUUM;\nINSERT OR IGNORE INTO theme_cache (slug) VALUES ('dracula');\nSELECT slug FROM theme_cache WHERE slug GLOB 'dr*';",
            &theme,
        )
        .expect("failed to highlight sqlite SQL source");
    assert!(
        sqlite_rendered.contains("\x1b[38;2;255;121;198mVACUUM"),
        "expected sqlite runtime to highlight VACUUM"
    );
    assert!(
        sqlite_rendered.contains("\x1b[38;2;255;121;198mIGNORE"),
        "expected sqlite runtime to highlight IGNORE"
    );
}

#[test]
fn regex_highlights_lookarounds_posix_unicode_and_inline_flags() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let rendered = highlight_named_language(
        "regex",
        r"(?im-s:(?<=\b)(?<word>[[:alpha:]]+)\p{Script=Latin}(?P=word)(?!\d)){2,4}?",
        &theme,
    )
    .expect("failed to highlight regex source");

    assert!(
        rendered.contains("\x1b[38;2;255;121;198mim-s"),
        "expected regex inline flags styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;139;233;253malpha"),
        "expected regex POSIX class name styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m\\p"),
        "expected regex unicode property escape styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;139;233;253mScript"),
        "expected regex unicode property name styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;139;233;253mLatin"),
        "expected regex unicode property value styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m="),
        "expected regex unicode property separator styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108m2"),
        "expected regex quantifier lower bound styling"
    );
}

#[test]
fn regex_semantic_overlay_reports_quantifiers_unicode_and_class_ranges() {
    let output = debug_semantics("regex", r"(?im-s:\p{Script=Latin}[a-z0-9-]{2,4})")
        .expect("failed to render regex semantic overlay");

    assert!(
        output.contains("keyword.operator.regex") && output.contains("im"),
        "expected regex semantic overlay to include inline flag spans: {output}"
    );
    assert!(
        output.contains("operator.regex") && output.contains("\\p"),
        "expected regex semantic overlay to include unicode property escape spans: {output}"
    );
    assert!(
        output.contains("type.builtin") && output.contains("Script"),
        "expected regex semantic overlay to include unicode property name spans: {output}"
    );
    assert!(
        output.contains("type.builtin") && output.contains("Latin"),
        "expected regex semantic overlay to include unicode property value spans: {output}"
    );
    assert!(
        output.contains("number.quantifier.regex") && output.contains("2"),
        "expected regex semantic overlay to include quantifier number spans: {output}"
    );
    assert!(
        output.contains("punctuation.delimiter") && output.contains(","),
        "expected regex semantic overlay to include quantifier delimiter spans: {output}"
    );
    assert!(
        output.contains("operator.regex") && output.contains("-"),
        "expected regex semantic overlay to include class range operator spans: {output}"
    );
}

#[test]
fn regex_host_runtimes_flag_unsupported_constructs() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let python_rendered =
        highlight_named_language("regex_python", r"(?<name>\w+)\k<name>\p{L}", &theme)
            .expect("failed to highlight python regex source");
    assert!(
        python_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?<name>"),
        "expected python regex runtime to flag JS-style named groups as invalid"
    );
    let rust_rendered =
        highlight_named_language("regex_rust", r"(?=foo)(?P<name>\w+)(?P=name)", &theme)
            .expect("failed to highlight rust regex source");
    assert!(
        rust_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?=foo)"),
        "expected rust regex runtime to flag lookahead as invalid"
    );

    let go_rendered = highlight_named_language("regex_go", r"(?=foo)\1", &theme)
        .expect("failed to highlight go regex source");
    assert!(
        go_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?=foo)"),
        "expected go regex runtime to flag lookahead as invalid"
    );

    let javascript_rendered = highlight_named_language("regex_javascript", r"(?i:theme)", &theme)
        .expect("failed to highlight javascript regex source");
    assert!(
        javascript_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?i:theme)"),
        "expected javascript regex runtime to flag inline flag groups as invalid"
    );

    let posix_rendered = highlight_named_language("regex_posix", r"(?:theme)+?", &theme)
        .expect("failed to highlight posix regex source");
    assert!(
        posix_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?:theme)"),
        "expected posix regex runtime to flag non-capturing groups as invalid"
    );
}

#[test]
fn sql_like_api_calls_in_hosts_reuse_sql_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let js_rendered = render_with_theme(
            Some(Path::new("inline.js")),
            "export async function load(db) { return db.query(\"SELECT id FROM themes WHERE slug = $1 RETURNING id\"); }",
            &theme,
        )
        .expect("failed to render inline javascript SQL source");
    assert!(
        js_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected JavaScript db.query string to reuse SQL runtime"
    );
    assert!(
        js_rendered.contains("\x1b[3m\x1b[38;2;255;184;108m$1"),
        "expected JavaScript db.query string to preserve SQL parameter styling"
    );
    let js_sqlite_rendered = render_with_theme(
            Some(Path::new("inline.js")),
            "export function load(db) { db.get(\"SELECT id FROM themes WHERE slug = ?\"); db.run(\"INSERT INTO theme_cache (slug) VALUES (?)\"); }",
            &theme,
        )
        .expect("failed to render inline javascript sqlite-style SQL source");
    assert!(
        js_sqlite_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected JavaScript db.get string to reuse SQL runtime"
    );
    assert!(
        js_sqlite_rendered.contains("\x1b[38;2;255;121;198mINSERT"),
        "expected JavaScript db.run string to reuse SQL runtime"
    );

    let python_rendered = render_with_theme(
        Some(Path::new("inline.py")),
        "def load(cursor):\n    return cursor.execute(\"SELECT id FROM themes WHERE slug = ?\")\n",
        &theme,
    )
    .expect("failed to render inline python SQL source");
    assert!(
        python_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected Python cursor.execute string to reuse SQL runtime"
    );

    let go_rendered = render_with_theme(
        Some(Path::new("inline.go")),
        "package main\nfunc load(db DB) { db.Query(\"SELECT id FROM themes WHERE slug = $1\") }\n",
        &theme,
    )
    .expect("failed to render inline go SQL source");
    assert!(
        go_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected Go db.Query string to reuse SQL runtime"
    );

    let rust_rendered = render_with_theme(
            Some(Path::new("inline.rs")),
            "fn load(client: Client) { let _ = client.query(\"SELECT id FROM themes WHERE slug = $1\", &[]); }\n",
            &theme,
        )
        .expect("failed to render inline rust SQL source");
    assert!(
        rust_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected Rust client.query string to reuse SQL runtime"
    );
    let rust_prepare_rendered = render_with_theme(
            Some(Path::new("inline.rs")),
            "fn load(client: Client) { let _ = client.prepare_typed(\"SELECT id FROM themes WHERE slug = $1\", &[]); let _ = client.simple_query(\"SELECT id FROM themes\"); }\n",
            &theme,
        )
        .expect("failed to render inline rust extended SQL source");
    assert!(
        rust_prepare_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected Rust prepare_typed/simple_query strings to reuse SQL runtime"
    );
}

#[test]
fn bash_and_just_sql_heredocs_reuse_sql_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let bash_path = fixture_path("bash/sql_heredoc.sh");
    let bash_source = read_file(&bash_path);
    let bash_rendered = render_with_theme(Some(bash_path.as_path()), &bash_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", bash_path.display()));
    assert!(
        bash_rendered.contains("\x1b[38;2;255;121;198mRETURNING"),
        "expected psql heredoc to route through postgres SQL runtime"
    );
    assert!(
        bash_rendered.contains("\x1b[38;2;255;121;198mAUTO_INCREMENT"),
        "expected mysql heredoc to route through mysql SQL runtime"
    );
    assert!(
        bash_rendered.contains("\x1b[38;2;255;121;198mWITHOUT"),
        "expected sqlite heredoc to route through sqlite SQL runtime"
    );
    assert!(
        bash_rendered.contains("\x1b[38;2;80;250;123mpsql"),
        "expected Bash command styling to remain visible around SQL injection"
    );

    let just_path = fixture_path("just/heredoc_recipes.just");
    let just_source = read_file(&just_path);
    let just_rendered = render_with_theme(Some(just_path.as_path()), &just_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", just_path.display()));
    assert!(
        just_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
        "expected Justfile Bash heredoc to reuse SQL runtime"
    );
}

#[test]
fn javascript_highlights_classes_private_fields_regex_and_jsx() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let rich_path = fixture_path("javascript/rich.js");
    let rich_source = read_file(&rich_path);
    let rich_rendered = render_with_theme(Some(rich_path.as_path()), &rich_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", rich_path.display()));

    assert!(
        rich_rendered.contains("\x1b[38;2;139;233;253mThemePreview"),
        "expected class definition styling for ThemePreview"
    );
    assert!(
        strip_ansi(&rich_rendered).contains("#theme"),
        "expected private field property styling for #theme"
    );
    assert!(
        rich_rendered.contains("\x1b[38;2;80;250;123mrender"),
        "expected method definition styling for render"
    );
    assert!(
        rich_rendered.contains("\x1b[38;2;255;85;85m/dracula/"),
        "expected regex body styling for /dracula/"
    );
    assert!(
        rich_rendered.contains("\x1b[38;2;255;121;198mgi"),
        "expected regex flag styling for gi"
    );
    assert!(
        rich_rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
        "expected builtin console styling"
    );

    let jsx_path = fixture_path("javascript/component.jsx");
    let jsx_source = read_file(&jsx_path);
    let jsx_rendered = render_with_theme(Some(jsx_path.as_path()), &jsx_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", jsx_path.display()));

    assert!(
        jsx_rendered.contains("\x1b[38;2;255;121;198msection"),
        "expected intrinsic JSX tag styling for section"
    );
    assert!(
        jsx_rendered.contains("\x1b[38;2;139;233;253mPreviewCard"),
        "expected component JSX tag styling for PreviewCard"
    );
    assert!(
        jsx_rendered.contains("\x1b[3m\x1b[38;2;80;250;123mclassName"),
        "expected JSX attribute styling for className"
    );
}

#[test]
fn jsdoc_highlights_optionals_defaults_inline_tags_and_code_fences() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let rendered = highlight_named_language(
            "jsdoc",
            "/**\n * Render {@link ThemePreview} for docs.\n * @param {ThemePreview} [preview=ThemeStore.default] Preview value.\n * @param {number} [retryCount=3] Retries.\n * @see theme/preview\n * ```js\n * console.log(preview)\n * ```\n */",
            &theme,
        )
        .expect("failed to highlight jsdoc source");

    assert!(
        rendered.contains("\x1b[38;2;255;121;198m@param"),
        "expected jsdoc tag styling for @param"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m="),
        "expected jsdoc default-value operator styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108m3"),
        "expected jsdoc numeric default styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m."),
        "expected jsdoc member-expression delimiter styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140mjs"),
        "expected jsdoc code-block language styling"
    );
}

#[test]
fn sql_semantic_overlay_reports_dialect_specific_identifiers() {
    let postgres_output = debug_semantics(
            "sql_postgres",
            "CREATE FUNCTION refresh_theme_cache() RETURNS void LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE AS $$ BEGIN RETURN; END; $$;\nCREATE INDEX theme_payload_gin ON theme_snapshots USING GIN (payload jsonb_path_ops);",
        )
        .expect("failed to render postgres semantic overlay");
    assert!(
        postgres_output.contains("type.builtin") && postgres_output.contains("plpgsql"),
        "expected postgres semantic overlay to include function language spans: {postgres_output}"
    );
    assert!(
        postgres_output.contains("type.builtin") && postgres_output.contains("jsonb_path_ops"),
        "expected postgres semantic overlay to include opclass spans: {postgres_output}"
    );

    let mysql_output = debug_semantics(
        "sql_mysql",
        "CREATE TABLE theme_snapshots (name VARCHAR(255)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;",
    )
    .expect("failed to render mysql semantic overlay");
    assert!(
        mysql_output.contains("type.builtin") && mysql_output.contains("InnoDB"),
        "expected mysql semantic overlay to include engine value spans: {mysql_output}"
    );
    assert!(
        mysql_output.contains("type.builtin") && mysql_output.contains("utf8mb4"),
        "expected mysql semantic overlay to include charset value spans: {mysql_output}"
    );
}

#[test]
fn jsdoc_semantic_overlay_reports_inline_reference_targets() {
    let output = debug_semantics(
        "jsdoc",
        "/**\n * Render {@link ThemePreview#render} and {@link module:theme/preview}.\n */",
    )
    .expect("failed to render jsdoc semantic overlay");

    assert!(
        output.contains("variable.jsdoc") && output.contains("ThemePreview"),
        "expected jsdoc semantic overlay to include member reference identifiers: {output}"
    );
    assert!(
        output.contains("punctuation.delimiter") && output.contains("#"),
        "expected jsdoc semantic overlay to include member delimiter spans: {output}"
    );
    assert!(
        output.contains("text.uri") && output.contains("theme"),
        "expected jsdoc semantic overlay to include path-like inline reference spans: {output}"
    );
    assert!(
        output.contains("punctuation.delimiter") && output.contains("/"),
        "expected jsdoc semantic overlay to include path delimiter spans: {output}"
    );
}

#[test]
fn bash_highlights_shebang_special_variables_regex_and_parameters() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("bash/rich.sh");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m#!/usr/bin/env bash"),
        "expected Bash shebang directive styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m=~"),
        "expected Bash regex operator styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;85;85m^kat.*"),
        "expected Bash regex literal styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249m#"),
        "expected Bash special variable styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123mpipefail"),
        "expected Bash set option styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m$'line\\nsecond'"),
        "expected Bash ANSI-C string styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mread"),
        "expected Bash read builtin styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mdeclare"),
        "expected Bash declare builtin styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253munset"),
        "expected Bash unset builtin styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;255;184;108mtheme_line"),
        "expected Bash read target variable styling"
    );
    assert!(
        strip_ansi(&rendered).contains("palettes[0]"),
        "expected Bash subscript bracket styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108m0"),
        "expected Bash subscript index number styling"
    );
}

#[test]
fn rust_multiline_raw_string_sql_uses_block_region_tint() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("rust/injections.rs");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let region = find_nested_region(&analysis, "sql", "SELECT id, name, enabled", &source);

    assert!(
        region_covers_line(region, &source, "SELECT id, name, enabled"),
        "expected multiline Rust SQL raw string to receive nested block coverage"
    );
    let select_segment = segment_for_line(region, &source, "SELECT id, name, enabled");
    let background_column = segment_left_column(&source, select_segment);
    assert!(
        background_column == 8,
        "expected Rust SQL block to preserve host indentation before the block tint, got column {background_column}: {:?}",
        line_text_at(&source, select_segment.line_start)
    );
    assert!(
        region_covers_line(region, &source, "ORDER BY name DESC"),
        "expected trailing Rust SQL raw-string line to stay inside the block coverage"
    );
    assert!(
        !region_covers_line(region, &source, "\"#"),
        "expected Rust raw-string closing delimiter line to stay outside the block tint"
    );
}
