use super::*;

#[test]
fn just_shell_aliases_reuse_supported_runtimes() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("just/runtime_matrix.just");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    let zsh_source_count = rendered.matches("\x1b[38;2;139;233;253msource").count();
    assert!(
        zsh_source_count >= 2,
        "expected global zsh shell setting to route both recipe bodies and external commands into zsh runtime"
    );
    assert!(
        rendered.matches("\x1b[38;2;139;233;253mprintf").count() >= 2,
        "expected zsh global shell setting to keep zsh builtin styling across recipe bodies and external commands"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected fish shebang recipe to route recipe body into fish runtime"
    );
}

#[test]
fn tree_sitter_query_files_highlight_captures_predicates_and_comments() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("query/grammars/lua/queries/highlights.scm");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;98;114;164m; Tree-sitter query fixture for kat"),
        "expected query fixture comment styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mfunction_item"),
        "expected query node names to use type styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123m@function"),
        "expected query capture names to use function-like styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253meq?"),
        "expected query predicates to use builtin-function styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m\"lua\""),
        "expected query string parameters to keep string styling"
    );
}

#[test]
fn fish_and_zsh_highlight_shell_specific_constructs() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let fish_path = fixture_path("fish/rich.fish");
    let fish_source = read_file(&fish_path);
    let fish_rendered = render_with_theme(Some(fish_path.as_path()), &fish_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", fish_path.display()));
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m#!/usr/bin/env fish"),
        "expected fish shebang directive styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;255;121;198mfunction"),
        "expected fish function keyword styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;139;233;253mset"),
        "expected fish builtin command styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;139;233;253mcontains"),
        "expected fish contains builtin styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;189;147;249m--argument-names"),
        "expected fish option styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;139;233;253memit"),
        "expected fish emit builtin styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;139;233;253mfunctions"),
        "expected fish functions builtin styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;139;233;253mtype"),
        "expected fish type builtin styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;80;250;123mcurrent-filename"),
        "expected fish status subcommand directive styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;255;85;85mDra*"),
        "expected fish case glob pattern styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;255;184;108mnew_theme"),
        "expected fish function argument names styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;255;184;108mTHEME_NAME"),
        "expected fish function handler target styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;80;250;123mreplace"),
        "expected fish string subcommand directive styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;189;147;249m$argv"),
        "expected fish argv special variable styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;189;147;249m$status"),
        "expected fish status special variable styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;189;147;249m$fish_pid"),
        "expected fish fish_pid special variable styling"
    );
    assert!(
        fish_rendered.contains("\x1b[3m\x1b[38;2;189;147;249m$last_pid"),
        "expected fish last_pid special variable styling"
    );
    assert!(
        fish_rendered.contains("\x1b[38;2;255;121;198m.."),
        "expected fish list range operator styling"
    );

    let zsh_path = fixture_path("zsh/rich.zsh");
    let zsh_source = read_file(&zsh_path);
    let zsh_rendered = render_with_theme(Some(zsh_path.as_path()), &zsh_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", zsh_path.display()));
    assert!(
        zsh_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m#!/usr/bin/env zsh"),
        "expected zsh shebang directive styling"
    );
    assert!(
        zsh_rendered.contains("\x1b[38;2;139;233;253memulate"),
        "expected zsh builtin command styling"
    );
    assert!(
        strip_ansi(&zsh_rendered).contains("[[ $theme_name"),
        "expected zsh test command bracket styling"
    );
    assert!(
        zsh_rendered.contains("\x1b[3m\x1b[38;2;80;250;123mEXTENDED_GLOB"),
        "expected zsh setopt option-name styling"
    );
    assert!(
        zsh_rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected zsh source builtin styling"
    );
    assert!(
        zsh_rendered.contains("\x1b[38;2;139;233;253mread"),
        "expected zsh read builtin styling"
    );
    assert!(
        zsh_rendered.contains("\x1b[3m\x1b[38;2;255;184;108mtheme_line"),
        "expected zsh read target variable styling"
    );
    assert!(
        strip_ansi(&zsh_rendered).contains("$palette[1]"),
        "expected zsh subscript bracket styling"
    );
    assert!(
        zsh_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m:l"),
        "expected zsh expansion modifier styling"
    );
    assert!(
        zsh_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m(I)"),
        "expected zsh subscript flag styling"
    );
}

#[test]
fn powershell_and_batch_highlight_shell_specific_constructs() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let powershell_path = fixture_path("powershell/rich.ps1");
    let powershell_source = read_file(&powershell_path);
    let powershell_rendered =
        render_with_theme(Some(powershell_path.as_path()), &powershell_source, &theme)
            .unwrap_or_else(|error| {
                panic!("failed to render {}: {error}", powershell_path.display())
            });
    assert!(
        powershell_rendered.contains("\x1b[38;2;255;121;198mparam"),
        "expected PowerShell param keyword styling"
    );
    assert!(
        powershell_rendered.contains("\x1b[38;2;139;233;253mstring"),
        "expected PowerShell type styling"
    );
    assert!(
        powershell_rendered.contains("\x1b[38;2;139;233;253mWrite-Host"),
        "expected PowerShell builtin command styling"
    );
    assert!(
        powershell_rendered.contains("\x1b[3m\x1b[38;2;189;147;249m$env:GITHUB_REF"),
        "expected PowerShell environment variables to receive special-variable styling"
    );

    let batch_path = fixture_path("batch/rich.cmd");
    let batch_source = read_file(&batch_path);
    let batch_rendered = render_with_theme(Some(batch_path.as_path()), &batch_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", batch_path.display()));
    assert!(
        batch_rendered.contains("\x1b[38;2;255;121;198m@echo off"),
        "expected batch echo-off keyword styling"
    );
    assert!(
        batch_rendered.contains("\x1b[38;2;255;121;198mset"),
        "expected batch set keyword styling"
    );
    assert!(
        batch_rendered.contains("\x1b[38;2;139;233;253mecho"),
        "expected batch builtin command styling"
    );
    assert!(
        batch_rendered.contains("\x1b[38;2;80;250;123m:build"),
        "expected batch labels to receive dedicated label styling"
    );
    assert!(
        batch_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m:eof"),
        "expected batch :eof targets to receive directive styling"
    );
}

#[test]
fn vhdl_vim_and_todotxt_highlight_language_specific_constructs() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let vhdl_path = fixture_path("vhdl/theme_preview.vhd");
    let vhdl_source = read_file(&vhdl_path);
    let vhdl_rendered = render_with_theme(Some(vhdl_path.as_path()), &vhdl_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", vhdl_path.display()));
    assert!(
        vhdl_rendered.contains("\x1b[38;2;255;121;198mentity"),
        "expected VHDL entity keyword styling"
    );
    assert!(
        vhdl_rendered.contains("\x1b[38;2;139;233;253mstd_logic_vector"),
        "expected VHDL builtin type styling"
    );
    assert!(
        vhdl_rendered.contains("\x1b[38;2;139;233;253mrising_edge"),
        "expected VHDL builtin function styling"
    );

    let vim_path = fixture_path("vim/theme_preview.vim");
    let vim_source = read_file(&vim_path);
    let vim_rendered = render_with_theme(Some(vim_path.as_path()), &vim_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", vim_path.display()));
    assert!(
        vim_rendered.contains("\x1b[38;2;255;121;198mlet"),
        "expected Vim let command styling"
    );
    assert!(
        vim_rendered.contains("\x1b[38;2;80;250;123mRenderTheme"),
        "expected Vim function declaration styling"
    );
    assert!(
        vim_rendered.contains("embedded lua from vim"),
        "expected Vim embedded Lua chunk to render"
    );

    let todotxt_path = fixture_path("todotxt/todo.txt");
    let todotxt_source = read_file(&todotxt_path);
    let todotxt_rendered = render_with_theme(Some(todotxt_path.as_path()), &todotxt_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", todotxt_path.display()));
    assert!(
        todotxt_rendered.contains("\x1b[38;2;255;121;198m(A)"),
        "expected todo.txt priority styling"
    );
    assert!(
        todotxt_rendered.contains("\x1b[38;2;241;250;140m+syntax"),
        "expected todo.txt project styling"
    );
    assert!(
        todotxt_rendered.contains("\x1b[38;2;139;233;253m@terminal"),
        "expected todo.txt context styling"
    );
}

#[test]
fn justfile_windows_shells_reuse_batch_and_powershell_runtimes() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("just/windows_shells.just");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198m@echo off"),
        "expected cmd-backed Justfile recipes to reuse batch highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mWrite-Host"),
        "expected pwsh shebang recipes to reuse PowerShell builtin highlighting"
    );
}

#[test]
fn embedded_template_profiles_route_host_content_into_matching_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let js_path = fixture_path("erb/app.js.erb");
    let js_source = read_file(&js_path);
    let js_analysis = analyze_with_theme(Some(js_path.as_path()), &js_source, &theme)
        .expect("failed to analyze js.erb fixture")
        .snapshot();
    assert!(
        nested_runtime_names(&js_analysis).contains(&"javascript"),
        "expected js.erb content to inject into javascript runtime"
    );
    assert!(
        !nested_runtime_names(&js_analysis).contains(&"html"),
        "expected js.erb content to avoid falling back to html runtime"
    );

    let css_path = fixture_path("erb/theme.css.erb");
    let css_source = read_file(&css_path);
    let css_analysis = analyze_with_theme(Some(css_path.as_path()), &css_source, &theme)
        .expect("failed to analyze css.erb fixture")
        .snapshot();
    assert!(
        nested_runtime_names(&css_analysis).contains(&"css"),
        "expected css.erb content to inject into css runtime"
    );

    let xml_path = fixture_path("jinja/feed.xml.j2");
    let xml_source = read_file(&xml_path);
    let xml_analysis = analyze_with_theme(Some(xml_path.as_path()), &xml_source, &theme)
        .expect("failed to analyze xml.j2 fixture")
        .snapshot();
    assert!(
        nested_runtime_names(&xml_analysis).contains(&"xml"),
        "expected xml.j2 content to inject into xml runtime"
    );
    assert!(
        !nested_runtime_names(&xml_analysis).contains(&"html"),
        "expected xml.j2 content to avoid falling back to html runtime"
    );

    let sls_path = fixture_path("jinja/webserver.sls");
    let sls_source = read_file(&sls_path);
    let sls_analysis = analyze_with_theme(Some(sls_path.as_path()), &sls_source, &theme)
        .expect("failed to analyze sls fixture")
        .snapshot();
    assert!(
        nested_runtime_names(&sls_analysis).contains(&"yaml"),
        "expected .sls content to inject into yaml runtime"
    );

    let sql_path = fixture_path("erb/schema.sql.erb");
    let sql_source = read_file(&sql_path);
    let sql_analysis = analyze_with_theme(Some(sql_path.as_path()), &sql_source, &theme)
        .expect("failed to analyze sql.erb fixture")
        .snapshot();
    assert!(
        nested_runtime_names(&sql_analysis).contains(&"sql"),
        "expected sql.erb content to inject into sql runtime"
    );
    assert!(
        !nested_runtime_names(&sql_analysis).contains(&"html"),
        "expected sql.erb content to avoid falling back to html runtime"
    );
}

#[test]
fn embedded_template_html_family_reuses_html_host_and_runtime_specific_code() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let eex_path = fixture_path("eex/page.html.eex");
    let eex_source = read_file(&eex_path);
    let eex_analysis = analyze_with_theme(Some(eex_path.as_path()), &eex_source, &theme)
        .expect("failed to analyze eex fixture")
        .snapshot();
    assert!(
        nested_runtime_names(&eex_analysis).contains(&"html"),
        "expected html.eex content to inject into html runtime"
    );
    assert!(
        nested_runtime_names(&eex_analysis).contains(&"elixir"),
        "expected html.eex code to inject into elixir runtime"
    );

    let jsp_path = fixture_path("jsp/page.jsp");
    let jsp_source = read_file(&jsp_path);
    let jsp_analysis = analyze_with_theme(Some(jsp_path.as_path()), &jsp_source, &theme)
        .expect("failed to analyze jsp fixture")
        .snapshot();
    assert!(
        nested_runtime_names(&jsp_analysis).contains(&"html"),
        "expected jsp content to inject into html runtime"
    );
    assert!(
        nested_runtime_names(&jsp_analysis).contains(&"java"),
        "expected jsp code to inject into java runtime"
    );
}

#[test]
fn asp_and_adp_templates_still_reuse_html_host_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    for relative_path in ["asp/page.asp", "adp/page.adp"] {
        let path = fixture_path(relative_path);
        let source = read_file(&path);
        let analysis = analyze_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to analyze {}: {error}", path.display()))
            .snapshot();
        assert!(
            nested_runtime_names(&analysis).contains(&"html"),
            "expected {relative_path} content to inject into html runtime"
        );
    }
}

#[test]
fn coffee_erb_templates_reuse_coffeescript_host_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("coffeescript/theme.coffee.erb");
    let source = read_file(&path);
    let analysis = analyze_with_theme(Some(path.as_path()), &source, &theme)
        .expect("failed to analyze coffee.erb fixture")
        .snapshot();

    assert!(
        nested_runtime_names(&analysis).contains(&"coffeescript"),
        "expected coffee.erb content to inject into coffeescript runtime"
    );
    assert!(
        nested_runtime_names(&analysis).contains(&"ruby"),
        "expected coffee.erb code to inject into ruby runtime"
    );
}

#[test]
fn nomad_template_destination_routes_data_into_target_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("hcl/nomad-template-python.nomad");
    let source = read_file(&path);
    let analysis = analyze_with_theme(Some(path.as_path()), &source, &theme)
        .expect("failed to analyze nomad python template fixture")
        .snapshot();

    assert!(
        nested_runtime_names(&analysis).contains(&"python"),
        "expected destination-driven nomad template content to inject into python runtime"
    );
}

#[test]
fn nomad_template_keeps_hcl_template_syntax_while_highlighting_python_content() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("hcl/nomad-template-python.nomad");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let python_region = find_nested_region(&analysis, "python", "def render_message():", &source);
    let keyword_style = theme
        .token_style_for("keyword", "def")
        .expect("expected python keyword style")
        .snapshot(ColorMode::TrueColor);
    let return_style = theme
        .token_style_for("keyword", "return")
        .expect("expected python return style")
        .snapshot(ColorMode::TrueColor);
    let template_punctuation_style = theme
        .token_style_for("punctuation.special", "${")
        .expect("expected hcl template punctuation style")
        .snapshot(ColorMode::TrueColor);
    let template_keyword_style = theme
        .token_style_for("keyword.control.conditional", "if")
        .expect("expected hcl template conditional style")
        .snapshot(ColorMode::TrueColor);
    let def_offset = source
        .find("def render_message")
        .expect("expected python def");
    let return_offset = source
        .find("return os.environ.get")
        .expect("expected python return statement");
    let interpolation_offset = source
        .find("${NOMAD_ALLOC_ID}")
        .expect("expected interpolation");
    let if_offset = source
        .find("%{ if")
        .expect("expected template if directive")
        + 3;

    assert!(
        region_covers_line(python_region, &source, "def render_message():"),
        "expected python region to cover the function definition line"
    );
    assert!(
        region_covers_line(python_region, &source, "print(\"hello ${NOMAD_ALLOC_ID}\")"),
        "expected python region to keep covering lines that contain hcl interpolations"
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= def_offset
                && def_offset < span.end
                && span.style.as_ref() == Some(&keyword_style)
        }),
        "expected def to keep python keyword styling, got {:?}",
        analysis.spans
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= return_offset
                && return_offset < span.end
                && span.style.as_ref() == Some(&return_style)
        }),
        "expected return after template directives to keep python keyword styling, got {:?}",
        analysis.spans
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= interpolation_offset
                && interpolation_offset < span.end
                && span.style.as_ref() == Some(&template_punctuation_style)
        }),
        "expected ${{...}} to keep hcl template punctuation styling, got {:?}",
        analysis.spans
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= if_offset
                && if_offset < span.end
                && span.style.as_ref() == Some(&template_keyword_style)
        }),
        "expected %{{ if ... }} to keep hcl conditional styling, got {:?}",
        analysis.spans
    );
}

#[test]
fn go_module_workspace_and_sum_files_use_dedicated_runtimes() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let gomod_path = fixture_path("go/go.mod");
    let gomod_source = read_file(&gomod_path);
    let gomod_rendered = render_with_theme(Some(gomod_path.as_path()), &gomod_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", gomod_path.display()));
    assert!(
        gomod_rendered.contains("\x1b[38;2;255;121;198mmodule"),
        "expected go.mod directive keyword styling"
    );
    assert!(
        gomod_rendered.contains("\x1b[38;2;139;233;253mgithub.com/example/kat"),
        "expected go.mod module path styling"
    );
    assert!(
        gomod_rendered.contains("\x1b[38;2;241;250;140mgo1.24.1"),
        "expected go.mod toolchain value styling"
    );
    assert!(
        gomod_rendered.contains("\x1b[38;2;139;233;253m../forks/lipgloss"),
        "expected go.mod replacement file path styling"
    );

    let gowork_path = fixture_path("go/go.work");
    let gowork_source = read_file(&gowork_path);
    let gowork_rendered = render_with_theme(Some(gowork_path.as_path()), &gowork_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", gowork_path.display()));
    assert!(
        gowork_rendered.contains("\x1b[38;2;255;121;198muse"),
        "expected go.work use directive styling"
    );
    assert!(
        gowork_rendered.contains("\x1b[38;2;139;233;253m../shared/theme-kit"),
        "expected go.work workspace path styling"
    );

    let gosum_path = fixture_path("go/go.sum");
    let gosum_source = read_file(&gosum_path);
    let gosum_rendered = render_with_theme(Some(gosum_path.as_path()), &gosum_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", gosum_path.display()));
    assert!(
        gosum_rendered.contains("\x1b[38;2;139;233;253mgithub.com/charmbracelet/lipgloss"),
        "expected go.sum module path styling"
    );
    assert!(
        gosum_rendered.contains("\x1b[3m\x1b[38;2;80;250;123mh1"),
        "expected go.sum hash version styling"
    );
    assert!(
        gosum_rendered.contains("\x1b[38;2;255;121;198m+incompatible"),
        "expected go.sum compatibility suffix styling"
    );
}

#[test]
fn git_ignore_extra_paths_reuse_ignore_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("ignore/git-info-exclude");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(Path::new(".git/info/exclude")), &source, &theme)
        .unwrap_or_else(|error| {
            panic!(
                "failed to render {} as .git/info/exclude: {error}",
                path.display()
            )
        });

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;98;114;164m# local-only ignore rules"),
        "expected exclude comments to reuse ignore comment styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m.direnv"),
        "expected local exclude patterns to reuse ignore string styling"
    );
}

#[test]
fn git_modules_profile_preserves_git_config_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("git_config/.gitmodules");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));
    let document_kind = detect_document_kind(Some(path.as_path()), &source)
        .expect("expected .gitmodules to detect as git_config");

    assert_eq!(document_kind.runtime_name(), "git_config");
    assert_eq!(document_kind.profile(), DocumentProfile::GitModules);
    assert!(
        rendered.contains("\x1b[38;2;241;250;140mvendor/kat"),
        "expected submodule paths to remain visible and highlighted"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140mhttps://github.com/happy-proto/kat.git"),
        "expected submodule URLs to be string-highlighted"
    );
}

#[test]
fn dockerfile_highlights_directives_and_injects_shell_forms() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198mFROM"),
        "expected Dockerfile instruction keyword styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mprintf"),
        "expected shell-form RUN/CMD/ENTRYPOINT content to reuse bash builtin styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249m--version"),
        "expected shell-form command options to reuse bash parameter styling"
    );
}

#[test]
fn dockerfile_healthcheck_and_heredoc_reuse_bash_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.advanced");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198mHEALTHCHECK"),
        "expected HEALTHCHECK instruction keyword styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mprintf"),
        "expected HEALTHCHECK CMD shell form to reuse bash builtin styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected Dockerfile heredoc lines to reuse bash builtin styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m$"),
        "expected Dockerfile heredoc bash expansions to be highlighted"
    );
}

#[test]
fn dockerfile_shell_instruction_routes_shell_regions_to_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.shells");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253msource"),
        "expected SHELL [\"zsh\", ...] to route shell-form RUN into zsh runtime"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mset"),
        "expected SHELL [\"fish\", ...] to route shell-form RUN into fish runtime"
    );
}

#[test]
fn dockerfile_highlights_param_names_separately_from_values() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("dockerfile/Dockerfile.params");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m--from"),
        "expected Dockerfile param names to keep attribute styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140mbuilder"),
        "expected Dockerfile param values to use string-style highlighting"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mBUILDPLATFORM"),
        "expected generic param expansions to split out the variable name semantics"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m5s"),
        "expected HEALTHCHECK param values to stop inheriting attribute styling"
    );
    assert!(
        (rendered.contains("\x1b[38;2;189;147;249mCACHE_DIR")
            || rendered.contains("\x1b[38;2;189;147;249mCACHE_STAGE"))
            && rendered.contains("\x1b[38;2;255;184;108m1000"),
        "expected mount param payload to keep expansion and numeric semantics"
    );
}

#[test]
fn rustdoc_comments_highlight_markdown_content() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("rust/doc_comments.rs");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[1m\x1b[38;2;189;147;249mPreview"),
        "expected rustdoc heading styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;241;250;140mmarkdown"),
        "expected rustdoc emphasis styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123minline code"),
        "expected rustdoc code-span styling"
    );
}

#[test]
fn rustdoc_inner_fenced_block_keeps_its_own_block_width() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("rust/doc_comments_nested.rs");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let markdown_region = find_nested_region(&analysis, "markdown", "class Nested:", &source);
    let python_region = find_nested_region(&analysis, "python", "class Nested:", &source);
    assert!(
        python_region.visual_level == 2,
        "inner fenced block line should use its own stronger block level"
    );
    assert!(
        region_covers_line(markdown_region, &source, "class Nested:"),
        "the same line should still preserve the weaker outer rustdoc container coverage"
    );
    assert_eq!(python_region.visual_kind, "rect_block");
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 120);
    let rows = [
        layout_row_containing(&layout.rows, "class Nested:"),
        layout_row_containing(&layout.rows, "def render(self)"),
        layout_row_containing(&layout.rows, "return 42"),
    ];
    let expected_bounds = background_bounds_at_level(rows[0], 2);
    assert!(
        expected_bounds.is_some()
            && rows
                .iter()
                .all(|row| background_bounds_at_level(row, 2) == expected_bounds),
        "inner fenced rows should share their own level-2 layout-time block bbox"
    );
}

#[test]
fn rustdoc_fenced_rust_does_not_inherit_markdown_literal_color_for_plain_identifiers() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = Path::new("testdata/showcase/rust/macros.rs").to_path_buf();
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        strip_ansi(&rendered).contains(" rendered "),
        "expected rustdoc fenced rust local binding to keep foreground styling"
    );
    assert!(
        !rendered.contains("\x1b[38;2;241;250;140m rendered = "),
        "plain rust identifier in rustdoc fence should not inherit markdown literal yellow"
    );
}

#[test]
fn python_highlights_class_and_method_names() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/main.py");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253mGreeter"),
        "expected python class name styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mhello"),
        "expected python method name styling"
    );
}

#[test]
fn python_highlights_decorators_parameters_and_calls() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/rich_features.py");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198m@"),
        "expected decorator marker styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mdataclass"),
        "expected decorator name styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249mself"),
        "expected self parameter styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mprint"),
        "expected builtin call styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mThemePreview"),
        "expected class call styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mrender"),
        "expected method call styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mpreview"),
        "expected local binding styling for preview"
    );
}

#[test]
fn css_highlights_selectors_namespace_custom_properties_and_units() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("css/rich.css");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198m@supports"),
        "expected at-rule styling for @supports"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m#"),
        "expected id selector punctuation styling for #"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mapp"),
        "expected id selector name styling for app"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m."),
        "expected class selector punctuation styling for ."
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mpanel"),
        "expected class selector name styling for panel"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m:"),
        "expected pseudo selector punctuation styling for :"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mhover"),
        "expected pseudo selector name styling for hover"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mrem"),
        "expected unit styling for rem"
    );
}

#[test]
fn jsdoc_highlights_tags_types_and_identifiers() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let rendered = highlight_named_language(
        "jsdoc",
        "/**\n * @param {ThemePreview} preview Render the preview state.\n */",
        &theme,
    )
    .expect("failed to highlight jsdoc source");

    assert!(
        rendered.contains("\x1b[38;2;255;121;198m@param"),
        "expected jsdoc tag styling for @param"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mThemePreview"),
        "expected jsdoc type styling for ThemePreview"
    );
}

#[test]
fn graphql_runtime_is_reused_by_javascript_and_markdown_hosts() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let graphql_path = fixture_path("graphql/schema.graphql");
    let graphql_source = read_file(&graphql_path);
    let graphql_analysis =
        analysis_snapshot_for_path(graphql_path.as_path(), &graphql_source, &theme);
    assert_eq!(
        graphql_analysis
            .resolved_document_kind
            .as_ref()
            .map(|kind| kind.runtime_name),
        Some("graphql")
    );

    let js_path = fixture_path("javascript/graphql.js");
    let js_source = read_file(&js_path);
    let js_analysis = analysis_snapshot_for_path(js_path.as_path(), &js_source, &theme);
    assert!(nested_runtime_names(&js_analysis).contains(&"graphql"));

    let markdown_path = fixture_path("markdown/graphql_fence.md");
    let markdown_source = read_file(&markdown_path);
    let markdown_analysis =
        analysis_snapshot_for_path(markdown_path.as_path(), &markdown_source, &theme);
    assert!(nested_runtime_names(&markdown_analysis).contains(&"graphql"));
}

#[test]
fn injected_python_reuses_the_same_highlight_rules() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let just_path = fixture_path("just/injections.just");
    let just_source = read_file(&just_path);
    let just_analysis = analysis_snapshot_for_path(just_path.as_path(), &just_source, &theme);
    assert!(nested_runtime_names(&just_analysis).contains(&"python"));

    let markdown_path = Path::new("testdata/showcase/markdown/mixed-code-blocks.md").to_path_buf();
    let markdown_source = read_file(&markdown_path);
    let markdown_analysis =
        analysis_snapshot_for_path(markdown_path.as_path(), &markdown_source, &theme);
    assert!(nested_runtime_names(&markdown_analysis).contains(&"python"));
}

#[test]
fn userscript_metadata_block_highlights_keys_urls_patterns_and_special_values() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("javascript/userscript.user.js");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m==UserScript=="),
        "expected userscript start marker styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mname"),
        "expected userscript metadata key styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m:zh-CN"),
        "expected userscript localized key suffix styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mhttps://example.com/kat"),
        "expected userscript URL styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;85;85m*://example.com/*"),
        "expected userscript match-pattern styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mGM_addStyle"),
        "expected userscript grant API styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mnone"),
        "expected userscript special grant value styling"
    );
    assert!(
        rendered.contains("icon"),
        "expected userscript resource alias styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mdocument-start"),
        "expected userscript run-at enum styling"
    );
}

#[test]
fn userscript_metadata_highlighting_reuses_javascript_runtime_in_markdown_fences() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("markdown/userscript_fence.md");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123m==UserScript=="),
        "expected fenced JavaScript userscript marker styling in Markdown"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mgrant"),
        "expected fenced JavaScript userscript key styling in Markdown"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mGM_addStyle"),
        "expected fenced JavaScript userscript grant styling in Markdown"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mdocument-end"),
        "expected fenced JavaScript userscript enum styling in Markdown"
    );
}

#[test]
fn json_highlights_keys_booleans_escapes_and_punctuation() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("json/rich.json");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253m\"theme\""),
        "expected JSON object key styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108mtrue"),
        "expected JSON boolean styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mnull"),
        "expected JSON null styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m\\\\"),
        "expected JSON escape sequence styling"
    );
    assert!(
        strip_ansi(&rendered).contains("{\n"),
        "expected JSON bracket punctuation styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m:"),
        "expected JSON delimiter punctuation styling"
    );
}

#[test]
fn toml_highlights_quoted_keys_datetimes_inline_tables_and_cargo_lock() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let rich_path = fixture_path("toml/rich.toml");
    let rich_source = read_file(&rich_path);
    let rich_rendered = render_with_theme(Some(rich_path.as_path()), &rich_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", rich_path.display()));

    assert!(
        rich_rendered.contains("\x1b[38;2;139;233;253m\"display-name\""),
        "expected TOML quoted-key property styling"
    );
    assert!(
        rich_rendered.contains("\x1b[38;2;255;121;198m\\n"),
        "expected TOML escape sequence styling"
    );
    assert!(
        rich_rendered.contains("\x1b[38;2;255;184;108m2026-03-26T18:00:00Z"),
        "expected TOML datetime styling"
    );
    assert!(
        strip_ansi(&rich_rendered).contains("{ enabled = true"),
        "expected TOML inline-table bracket styling"
    );
    assert!(
        rich_rendered.contains("\x1b[38;2;255;184;108mtrue"),
        "expected TOML boolean styling inside inline table"
    );

    let cargo_lock_path = fixture_path("toml/Cargo.lock");
    let cargo_lock_source = read_file(&cargo_lock_path);
    let cargo_lock_rendered =
        render_with_theme(Some(cargo_lock_path.as_path()), &cargo_lock_source, &theme)
            .unwrap_or_else(|error| {
                panic!("failed to render {}: {error}", cargo_lock_path.display())
            });

    assert!(
        cargo_lock_rendered.contains("\x1b["),
        "expected Cargo.lock to be detected as TOML"
    );
    assert!(
        cargo_lock_rendered.contains("\x1b[38;2;139;233;253mversion"),
        "expected Cargo.lock keys to reuse TOML property styling"
    );

    let uv_lock_path = fixture_path("toml/uv.lock");
    let uv_lock_source = read_file(&uv_lock_path);
    let uv_lock_rendered = render_with_theme(Some(uv_lock_path.as_path()), &uv_lock_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", uv_lock_path.display()));

    assert!(
        uv_lock_rendered.contains("\x1b["),
        "expected uv.lock to be detected as TOML"
    );
    assert!(
        uv_lock_rendered.contains("\x1b[38;2;139;233;253mversion"),
        "expected uv.lock keys to reuse TOML property styling"
    );
}

#[test]
fn github_actions_yaml_profiles_inject_run_steps_and_highlight_expressions() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let workflow_path = fixture_path("yaml/github-actions-workflow.yaml");
    let workflow_source = read_file(&workflow_path);
    let workflow_rendered = render_with_theme(
        Some(Path::new(".github/workflows/build.yml")),
        &workflow_source,
        &theme,
    )
    .unwrap_or_else(|error| panic!("failed to render {}: {error}", workflow_path.display()));
    let workflow_regions = collect_top_level_injection_regions(
        yaml_document_kind(Some(Path::new(".github/workflows/build.yml"))),
        &workflow_source,
        &theme,
        1,
        None,
    )
    .expect("expected GitHub Actions workflow injections to resolve");
    assert!(
        workflow_regions.iter().any(|region| {
            region.overlays.iter().any(|span| {
                &workflow_source[span.range.clone()] == "printf"
                    && span.style == theme.token_style_for("function.builtin", "printf")
            })
        }),
        "expected GitHub Actions host resolver to produce Bash overlays for run blocks"
    );

    assert!(
        workflow_rendered.contains("\x1b[38;2;139;233;253mprintf"),
        "expected GitHub Actions run block to reuse Bash runtime styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;139;233;253mprint"),
        "expected shell: python run block to reuse Python builtin styling"
    );
    assert!(
        strip_ansi(&workflow_rendered).contains("github.ref"),
        "expected GitHub Actions expressions to highlight context identifiers"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;255;121;198m&&"),
        "expected GitHub Actions expressions to highlight logical operators"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;139;233;253mactions"),
        "expected uses owner to receive dedicated GitHub Actions styling"
    );
    assert!(
        workflow_rendered.contains("\x1b[38;2;80;250;123mcheckout"),
        "expected uses repository to receive dedicated GitHub Actions styling"
    );

    let action_path = fixture_path("yaml/action.yaml");
    let action_source = read_file(&action_path);
    let action_rendered = render_with_theme(Some(Path::new("action.yml")), &action_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", action_path.display()));
    assert!(
        action_rendered.contains("\x1b[38;2;80;250;123mecho"),
        "expected composite action run step to reuse Bash runtime styling"
    );
    assert!(
        action_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mcomposite"),
        "expected runs.using values to receive GitHub Actions schema-aware styling"
    );
}

#[test]
fn prek_toml_profile_injects_system_hook_entries_into_bash_runtime() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let source = r#"minimum_prek_version = "0.3.8"

[[repos]]
repo = "local"

[[repos.hooks]]
id = "clippy"
language = "system"
entry = "cargo clippy --workspace --locked --all-targets --all-features -- -D warnings"

[[repos.hooks]]
id = "pytest"
language = "python"
entry = "python -m pytest -q"
"#;
    let document_kind = detect_document_kind(Some(Path::new("prek.toml")), source)
        .expect("expected prek.toml to resolve as TOML");
    let regions = collect_top_level_injection_regions(document_kind, source, &theme, 1, None)
        .expect("expected prek.toml injections to resolve");

    assert_eq!(
        regions
            .iter()
            .map(|region| region.resolved_document_kind.runtime_name())
            .collect::<Vec<_>>(),
        vec!["bash"],
        "expected only system hook entries to inject a shell runtime"
    );
    assert!(
        regions.iter().any(|region| {
            region.overlays.iter().any(|span| {
                &source[span.range.clone()] == "cargo"
                    && span.style == theme.token_style_for("function", "cargo")
            })
        }),
        "expected prek.toml system entry to reuse Bash command-name styling"
    );
}

#[test]
fn prek_toml_system_entry_stays_inline_without_block_tint() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let source = r#"minimum_prek_version = "0.3.8"

[[repos]]
repo = "local"

[[repos.hooks]]
id = "clippy"
language = "system"
entry = "cargo clippy --workspace --locked --all-targets --all-features -- -D warnings"
"#;
    let analysis = analysis_snapshot_for_path(Path::new("prek.toml"), source, &theme);
    let region = find_nested_region(&analysis, "bash", "cargo clippy --workspace", source);
    assert_eq!(
        region.visual_kind, "transparent",
        "expected single-line prek hook entries to stay inline instead of becoming block regions"
    );
    assert!(
        region_covers_line(region, source, "cargo clippy --workspace"),
        "expected injected shell region to cover the entry line"
    );
}

#[test]
fn proto_highlights_keywords_types_and_literals() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let rendered = render_with_theme(
        Some(Path::new("theme.proto")),
        r#"syntax = "proto3";

message ThemePreview {
  string name = 1;
  bool enabled = 2;
}
"#,
        &theme,
    )
    .expect("expected proto highlight to succeed");

    assert!(
        rendered.contains("\x1b["),
        "expected proto output to contain ANSI styling"
    );
    assert!(
        rendered.contains("syntax"),
        "expected proto keyword to be present in rendered output"
    );
    assert!(
        rendered.contains("\"proto3\""),
        "expected proto syntax literal to be present in rendered output"
    );
    assert!(
        rendered.contains("ThemePreview"),
        "expected proto message name to be present in rendered output"
    );
    assert!(
        rendered.contains("string"),
        "expected proto scalar type to be present in rendered output"
    );
    assert!(
        rendered.contains("1"),
        "expected proto field number to be present in rendered output"
    );
}

#[test]
fn textproto_highlights_fields_strings_booleans_and_numbers() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let rendered = render_with_theme(
        Some(Path::new("theme.textproto")),
        r#"name: "Dracula"
enabled: true
priority: 7
"#,
        &theme,
    )
    .expect("expected textproto highlight to succeed");

    assert!(
        rendered.contains("\x1b["),
        "expected textproto output to contain ANSI styling"
    );
    assert!(
        rendered.contains("name"),
        "expected textproto field name to be present in rendered output"
    );
    assert!(
        rendered.contains(":"),
        "expected textproto delimiter to be present in rendered output"
    );
    assert!(
        rendered.contains("\"Dracula\""),
        "expected textproto string to be present in rendered output"
    );
    assert!(
        rendered.contains("true"),
        "expected textproto boolean to be present in rendered output"
    );
    assert!(
        rendered.contains("7"),
        "expected textproto number to be present in rendered output"
    );
}

#[test]
fn rust_highlights_traits_attributes_macros_and_special_variables() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("rust/rich.rs");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123mderive"),
        "expected Rust attribute name styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mRenderable"),
        "expected Rust trait/interface styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mrender"),
        "expected Rust method call styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123mthemed"),
        "expected Rust macro styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;189;147;249mself"),
        "expected Rust self styling"
    );
    assert!(
        strip_ansi(&rendered).contains(" preview "),
        "expected Rust local variable bindings to keep foreground styling"
    );
}

#[test]
fn just_recipe_body_uses_block_region_tint_without_tinting_header_line() {
    let source = "install:\n    cargo install --path .\n";
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let analysis = analysis_snapshot_for_path(Path::new("Justfile"), source, &theme);
    let region = find_nested_region(&analysis, "bash", "cargo install --path .", source);

    assert!(
        !region_covers_line(region, source, "install:"),
        "recipe header line should not be part of the nested block"
    );
    assert!(
        region_covers_line(region, source, "cargo install --path ."),
        "first recipe command line should receive nested block coverage"
    );
}

#[test]
fn just_recipe_body_uses_rectangular_visual_kind() {
    let source = "install:\n    cargo install --path .\n    cargo fmt --check\n";
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let analysis = analysis_snapshot_for_path(Path::new("Justfile"), source, &theme);
    let region = find_nested_region(&analysis, "bash", "cargo install --path .", source);
    let first_segment = segment_for_line(region, source, "cargo install --path .");

    assert_eq!(
        region.visual_kind, "rect_block",
        "expected Justfile recipe bodies to use rectangular visuals"
    );
    assert_eq!(
        segment_left_column(source, first_segment),
        4,
        "expected recipe background to align with the shared recipe indent"
    );
    let layout = layout_snapshot_for_path(Path::new("Justfile"), source, &theme, 80);
    let first_row = layout_row_containing(&layout.rows, "cargo install --path .");
    let second_row = layout_row_containing(&layout.rows, "cargo fmt --check");
    assert_eq!(
        first_background_bounds(first_row),
        first_background_bounds(second_row),
        "expected recipe rows to share one rectangle"
    );
}

#[test]
fn just_recipe_trailing_separator_blank_line_stays_outside_block_region() {
    let source = "install:\n    cargo install --path .\n\nnext:\n    echo hi\n";
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let analysis = analysis_snapshot_for_path(Path::new("Justfile"), source, &theme);
    let first_region = find_nested_region(&analysis, "bash", "cargo install --path .", source);
    let second_region = find_nested_region(&analysis, "bash", "echo hi", source);

    assert!(
        region_covers_line(first_region, source, "cargo install --path ."),
        "first recipe body line should receive nested block coverage"
    );
    assert!(
        !first_region
            .layout_segments
            .iter()
            .any(|segment| segment.line_start == line_start_for_line_number(source, 2)),
        "separator blank line between recipes should stay outside the previous block tint"
    );
    assert!(
        !first_region
            .layout_segments
            .iter()
            .any(|segment| segment.line_start == line_start_for_line_number(source, 3)),
        "next recipe header line should stay outside the previous block tint"
    );
    assert!(
        region_covers_line(second_region, source, "echo hi"),
        "next recipe body line should still receive its own nested block coverage"
    );
}

#[test]
fn markdown_fenced_code_uses_block_region_tint_only_inside_fence_body() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("markdown/go_fence.md");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let first_region = find_nested_region(&analysis, "go", "package preview", &source);
    let second_region = find_nested_region(
        &analysis,
        "go",
        "func NewRenderer(name string) string {",
        &source,
    );
    assert!(
        region_covers_line(first_region, &source, "package preview"),
        "fenced code content should receive nested block coverage"
    );
    assert!(
        first_region
            .layout_segments
            .iter()
            .any(|segment| segment.line_start == line_start_for_line_number(&source, 4)),
        "empty line inside first fenced block should still be part of the block region"
    );
    assert!(
        second_region
            .layout_segments
            .iter()
            .any(|segment| segment.line_start == line_start_for_line_number(&source, 14)),
        "empty line inside second fenced block should still be part of the block region"
    );
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 120);
    for line_number in [4, 14] {
        let row = layout
            .rows
            .iter()
            .find(|row| row.source_line_start == line_start_for_line_number(&source, line_number))
            .unwrap_or_else(|| panic!("expected layout row for blank line {line_number}"));
        assert!(
            first_background_bounds(row).is_some(),
            "expected blank fenced-code line {line_number} to keep block background"
        );
    }
}

#[test]
fn github_actions_run_block_uses_block_region_tint_without_tinting_run_header() {
    let source = "jobs:\n  build:\n    steps:\n      - run: |\n          echo hi\n          printf '%s\\n' \"$GITHUB_REF\"\n";
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let analysis =
        analysis_snapshot_for_path(Path::new(".github/workflows/demo.yml"), source, &theme);
    let region = find_nested_region(&analysis, "bash", "echo hi", source);
    assert!(
        !region_covers_line(region, source, "- run: |"),
        "run: | header line should stay outside the nested block"
    );
    assert!(
        region_covers_line(region, source, "echo hi"),
        "first run body line should receive nested block coverage"
    );
    assert!(
        region_covers_line(region, source, "printf '%s\\n' \"$GITHUB_REF\""),
        "second run body line should receive nested block coverage"
    );

    let layout =
        layout_snapshot_for_path(Path::new(".github/workflows/demo.yml"), source, &theme, 120);
    assert_eq!(
        first_background_bounds(layout_row_containing(&layout.rows, "echo hi")),
        first_background_bounds(layout_row_containing(&layout.rows, "printf '%s\\n'")),
        "expected run body rows to share one layout-time block bbox"
    );
}

#[test]
fn github_actions_run_block_keeps_consistent_rendered_right_edge_with_cjk_content() {
    let source = "jobs:\n  build:\n    steps:\n      - run: |\n          echo \"短描述\"\n          echo \"这里放一段更长的中文描述用来验证共享几何也会补齐右边界\"\n";
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let layout = layout_snapshot_for_path(
        Path::new(".github/workflows/demo-cjk.yml"),
        source,
        &theme,
        49,
    );
    let member_line_starts = [
        line_start_containing(source, "echo \"短描述\""),
        line_start_containing(source, "这里放一段更长的中文描述"),
    ];
    let rows = layout
        .rows
        .iter()
        .filter(|row| member_line_starts.contains(&row.source_line_start))
        .collect::<Vec<_>>();
    assert!(rows.len() > 2, "expected the CJK run body to wrap");
    assert!(
        rows.iter()
            .all(|row| background_bounds_at_level(row, 1) == Some((0, 48))),
        "expected every GitHub Actions run content and continuation row to share the exact non-viewport-wide bbox: {rows:?}"
    );
}

#[test]
fn overlay_style_spans_only_splits_overlapping_ranges() {
    let theme = Theme::for_mode(ColorMode::Ansi);
    let parent_style = theme.token_style_for("string", "value");
    let overlay_style = theme.token_style_for("comment", "//");
    let parent_spans = vec![StyledSpan {
        range: 0..10,
        style: parent_style,
        hyperlink: None,
    }];
    let overlay_spans = vec![
        StyledSpan {
            range: 2..4,
            style: overlay_style,
            hyperlink: None,
        },
        StyledSpan {
            range: 6..8,
            style: overlay_style,
            hyperlink: None,
        },
    ];

    let result = overlay_style_spans(parent_spans, overlay_spans);

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
                style: overlay_style,
                hyperlink: None,
            },
            StyledSpan {
                range: 4..6,
                style: parent_style,
                hyperlink: None,
            },
            StyledSpan {
                range: 6..8,
                style: overlay_style,
                hyperlink: None,
            },
            StyledSpan {
                range: 8..10,
                style: parent_style,
                hyperlink: None,
            },
        ]
    );
}

#[test]
fn typescript_multiline_block_comments_keep_comment_style_on_following_lines() {
    let theme = Theme::for_mode(ColorMode::Ansi);
    let rendered = highlight_named_language("typescript", "/* hi\nworld\n*/\n", &theme)
        .expect("failed to highlight multiline TypeScript block comment");
    let lines = rendered.lines().collect::<Vec<_>>();

    assert!(
        lines
            .get(1)
            .is_some_and(|line| line.starts_with("\x1b[3m\x1b[90mworld")),
        "expected second block-comment line to keep comment style: {rendered:?}"
    );
    assert!(
        lines
            .get(2)
            .is_some_and(|line| line.starts_with("\x1b[3m\x1b[90m*/")),
        "expected closing block-comment line to keep comment style: {rendered:?}"
    );
}

#[test]
fn typescript_jsdoc_uses_block_region_tint() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let source = "/**\n * docs\n */\nconst answer = 42;\n";
    let analysis = analysis_snapshot_for_path(Path::new("demo.ts"), source, &theme);
    let region = find_nested_region(&analysis, "jsdoc", "docs", source);

    assert!(
        region_covers_line(region, source, "/**"),
        "expected JSDoc opening line to receive nested block coverage"
    );
    assert!(
        region_covers_line(region, source, "docs"),
        "expected JSDoc body line to receive nested block coverage"
    );
    assert!(
        region_covers_line(region, source, "*/"),
        "expected JSDoc closing line to receive nested block coverage"
    );
    assert!(
        !region_covers_line(region, source, "const answer = 42;"),
        "expected following code line to stay outside the JSDoc block"
    );
}

#[test]
fn markdown_truecolor_cjk_repro_avoids_excessive_reset_churn() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("markdown/markdown_truecolor_cjk_repro.md");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    let reset_count = count_occurrences(&rendered, "\x1b[0m");
    let default_fg_count = count_occurrences(&rendered, "\x1b[38;2;248;248;242m");

    assert!(
        reset_count <= 7,
        "expected compact truecolor output for the markdown CJK repro fixture, got {reset_count} resets: {rendered:?}"
    );
    assert!(
        default_fg_count <= 6,
        "expected markdown CJK repro fixture to avoid repeatedly re-entering the default foreground, got {default_fg_count}: {rendered:?}"
    );
}

#[test]
fn markdown_html_block_table_keeps_consistent_rendered_right_edge_with_cjk_content() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = Path::new("demo.md");
    let source = "<div>\n<div>这里是一段足够长的中文内容用于验证HTML块换行后的所有视觉行共享同一个精确背景边界</div>\n</div>\n";
    let layout = layout_snapshot_for_path(path, source, &theme, 80);
    let table_start = layout
        .rows
        .iter()
        .position(|row| row.text == "<div>")
        .expect("expected opening HTML row");
    let table_end = layout
        .rows
        .iter()
        .position(|row| row.text == "</div>")
        .expect("expected closing HTML row");
    let rows = &layout.rows[table_start..=table_end];
    let expected_bounds = Some((0, 79));
    assert!(
        rows.len() > 3,
        "expected the long CJK table row to produce a continuation row"
    );
    assert!(
        rows.iter()
            .all(|row| first_background_bounds(row) == expected_bounds),
        "expected every HTML content and continuation row to share the exact non-viewport-wide bbox: {rows:?}"
    );
}

#[test]
fn markdown_html_block_tabs_keep_consistent_rendered_right_edge() {
    let tint = RgbColor(1, 2, 3);
    let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
    let path = fixture_path("markdown/html_block_tabs.md");
    let source = read_file(&path);
    let layout = layout_snapshot_for_path(path.as_path(), &source, &theme, 120);
    let rows = [
        layout_row_containing(&layout.rows, "<table>"),
        layout_row_containing(&layout.rows, "<td>值"),
        layout_row_containing(&layout.rows, "这里放一段更长的中文内容"),
        layout_row_containing(&layout.rows, "</table>"),
    ];
    let expected_bounds = first_background_bounds(rows[0]);
    assert!(expected_bounds.is_some());
    assert!(
        rows.iter()
            .all(|row| first_background_bounds(row) == expected_bounds),
        "expected tabbed HTML block rows to share one layout-time block bbox"
    );
}

#[test]
fn coffeescript_query_captures_class_and_method_names() {
    let path = fixture_path("coffeescript/theme.coffee");
    let source = read_file(&path);
    let language = runtime("coffeescript")
        .expect("expected coffeescript runtime")
        .language
        .clone();
    let query = Query::new(
        &language,
        include_str!("../../grammars/coffeescript/queries/highlights.scm"),
    )
    .expect("expected coffeescript highlights query to compile");
    let capture_names = query.capture_names();
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .expect("expected coffeescript parser to load");
    let tree = parser
        .parse(&source, None)
        .expect("expected coffeescript tree");
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    let mut captures = Vec::new();

    while {
        matches.advance();
        matches.get().is_some()
    } {
        let query_match = matches
            .get()
            .expect("query match should exist immediately after advance");
        for capture in query_match.captures() {
            captures.push((
                capture_names[capture.index as usize].to_owned(),
                source[capture.node.byte_range()].to_owned(),
            ));
        }
    }

    assert!(
        captures
            .iter()
            .any(|(capture, text)| capture == "type" && text == "ThemePreview"),
        "expected CoffeeScript query to capture the class name, got {captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|(capture, text)| capture == "function.method" && text == "constructor"),
        "expected CoffeeScript query to capture constructor, got {captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|(capture, text)| capture == "function.method" && text == "render"),
        "expected CoffeeScript query to capture render, got {captures:?}"
    );
    assert!(
        captures
            .iter()
            .any(|(capture, text)| capture == "function.method" && text == "log"),
        "expected CoffeeScript query to capture member-call fallback names, got {captures:?}"
    );
}

#[test]
fn coffeescript_analysis_styles_class_and_method_names() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("coffeescript/theme.coffee");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let type_style = theme
        .token_style_for("type", "ThemePreview")
        .expect("expected type style");
    let method_style = theme
        .token_style_for("function.method", "constructor")
        .expect("expected function method style");
    let render_offset = source.find("render").expect("expected render token");
    let log_offset = source.find("log").expect("expected log token");
    let constructor_offset = source
        .find("constructor")
        .expect("expected constructor token");
    let class_offset = source.find("ThemePreview").expect("expected class token");

    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= class_offset
                && class_offset < span.end
                && span.style == Some(type_style.snapshot(ColorMode::TrueColor))
        }),
        "expected ThemePreview to use type styling, got {:?}",
        analysis.spans
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= constructor_offset
                && constructor_offset < span.end
                && span.style == Some(method_style.snapshot(ColorMode::TrueColor))
        }),
        "expected constructor to use method styling, got {:?}",
        analysis.spans
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= render_offset
                && render_offset < span.end
                && span.style == Some(method_style.snapshot(ColorMode::TrueColor))
        }),
        "expected render to use method styling, got {:?}",
        analysis.spans
    );
    assert!(
        analysis.spans.iter().any(|span| {
            span.start <= log_offset
                && log_offset < span.end
                && span.style == Some(method_style.snapshot(ColorMode::TrueColor))
        }),
        "expected log to use method styling, got {:?}",
        analysis.spans
    );
}

#[test]
fn csh_aliases_highlight_activation_syntax_and_preserve_source() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let source = read_file(&fixture_path("csh/activation.csh"));
    let mut parser = Parser::new();
    parser
        .set_language(&runtime("csh").unwrap().language)
        .unwrap();
    let tree = parser.parse(&source, None).unwrap();
    assert!(
        !tree.root_node().has_error(),
        "{}",
        tree.root_node().to_sexp()
    );
    for alias in ["csh", "tcsh"] {
        let rendered = highlight_named_language(alias, &source, &theme).unwrap();
        assert_eq!(strip_ansi(&rendered), source);
        assert!(
            rendered.contains("\x1b[38;2;255;121;198mif"),
            "{alias}: {rendered:?}"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198mendif"),
            "{alias}: {rendered:?}"
        );
        assert!(
            rendered.contains("\x1b[38;2;80;250;123mdeactivate"),
            "{alias}: {rendered:?}"
        );
    }
}

#[test]
fn markdown_csh_and_tcsh_fences_share_csh_runtime() {
    let path = fixture_path("csh/fences.md");
    let source = read_file(&path);
    let snapshot =
        analysis_snapshot_for_path(&path, &source, &Theme::for_mode(ColorMode::TrueColor));
    assert_eq!(
        nested_runtime_names(&snapshot)
            .into_iter()
            .filter(|name| *name == "csh")
            .count(),
        2
    );
}

#[test]
fn jam_build_language_highlights_rules_variables_and_comments() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let source = read_file(&fixture_path("jam/Jamfile"));
    let mut parser = Parser::new();
    parser
        .set_language(&runtime("jam").unwrap().language)
        .unwrap();
    let tree = parser.parse(&source, None).unwrap();
    assert!(
        !tree.root_node().has_error(),
        "{}",
        tree.root_node().to_sexp()
    );
    for alias in ["jam", "bjam", "boost-build"] {
        let rendered = highlight_named_language(alias, &source, &theme).unwrap();
        assert_eq!(strip_ansi(&rendered), source);
        assert!(
            rendered.contains("\x1b[38;2;255;121;198mrule"),
            "missing rule keyword styling: {alias}"
        );
        assert!(
            rendered.contains("\x1b[38;2;80;250;123mlinking"),
            "missing rule name styling: {alias}"
        );
        assert!(rendered.contains("\x1b[3m\x1b[38;2;98;114;164m# Portable B2 library"));
    }
}

#[test]
fn markdown_jam_aliases_use_jam_runtime() {
    let path = fixture_path("jam/fences.md");
    let source = read_file(&path);
    let snapshot =
        analysis_snapshot_for_path(&path, &source, &Theme::for_mode(ColorMode::TrueColor));
    assert_eq!(
        nested_runtime_names(&snapshot)
            .into_iter()
            .filter(|name| *name == "jam")
            .count(),
        3
    );
}

#[test]
fn jam_upstream_regressions_preserve_syntax_text_and_styles() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("jam/upstream-regressions.jam");
    let source = read_file(&path);
    let mut parser = Parser::new();
    parser
        .set_language(&runtime("jam").unwrap().language)
        .unwrap();
    let parsed = parser.parse(&source, None).unwrap();
    assert!(
        !parsed.root_node().has_error(),
        "{}",
        parsed.root_node().to_sexp()
    );
    let rendered = render_with_theme(Some(&path), &source, &theme).unwrap();
    assert_eq!(strip_ansi(&rendered), source);
    let analysis = analysis_snapshot_for_path(&path, &source, &theme);
    for (text, capture) in [("#include <demo.h>", "string"), ("[path]", "type")] {
        let offset = source.find(text).unwrap();
        let expected = theme
            .token_style_for(capture, text)
            .unwrap()
            .snapshot(ColorMode::TrueColor);
        assert!(
            analysis.spans.iter().any(|span| span.start <= offset
                && offset < span.end
                && span.style.as_ref() == Some(&expected)),
            "wrong style for {text}: {:?}",
            analysis.spans
        );
    }
}
