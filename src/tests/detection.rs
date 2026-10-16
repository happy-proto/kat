use super::*;

#[test]
fn cel_file_detects_and_highlights_standalone_expression() {
    let path = fixture_path("cel/policy.cel");
    let source = read_file(&path);
    assert_eq!(
        detected_runtime(Some(path.as_path()), &source),
        Some(RuntimeId::Cel)
    );
    let json = debug_analysis_json(Some(path.as_path()), &source)
        .expect("standalone CEL should be analyzed");
    let analysis: Value = serde_json::from_str(&json).expect("analysis JSON should parse");
    assert_eq!(analysis["detected_document_kind"]["runtime_name"], "cel");
    let exists = source
        .find("exists")
        .expect("fixture should contain exists");
    assert!(
        analysis["spans"]
            .as_array()
            .is_some_and(|spans| spans.iter().any(|span| {
                span["start"]
                    .as_u64()
                    .is_some_and(|start| start <= exists as u64)
                    && span["end"].as_u64().is_some_and(|end| end > exists as u64)
                    && span["style"]["color_name"] == "green"
            }))
    );
}

#[test]
fn fish_and_zsh_detect_by_extension_filename_and_shebang() {
    for (path, source, expected) in [
        (
            Some(Path::new(".envrc")),
            "export THEME=Dracula\n",
            RuntimeId::Bash,
        ),
        (
            Some(Path::new(".bash_aliases")),
            "alias k='kat'\n",
            RuntimeId::Bash,
        ),
        (
            Some(Path::new(".bash_completions")),
            "complete -W 'render' kat\n",
            RuntimeId::Bash,
        ),
        (
            Some(Path::new("bashrc")),
            "export KAT_THEME=dracula\n",
            RuntimeId::Bash,
        ),
        (
            Some(Path::new("theme.bashrc")),
            "export KAT_THEME=dracula\n",
            RuntimeId::Bash,
        ),
        (
            Some(Path::new("PKGBUILD")),
            "pkgname=kat\n",
            RuntimeId::Bash,
        ),
        (
            Some(Path::new("pkg/kat.ebuild")),
            "src_install() { :; }\n",
            RuntimeId::Bash,
        ),
        (
            Some(Path::new("pkg/eclass/kat.eclass")),
            "EXPORT_FUNCTIONS src_install\n",
            RuntimeId::Bash,
        ),
        (
            Some(Path::new("/etc/profile")),
            "export PATH=$PATH:/opt/kat\n",
            RuntimeId::Bash,
        ),
        (
            Some(Path::new("/etc/os-release")),
            "NAME=\"katOS\"\n",
            RuntimeId::Bash,
        ),
        (
            Some(Path::new("/var/run/os-release")),
            "ID=kat\n",
            RuntimeId::Bash,
        ),
        (
            Some(Path::new(".config/bat/config")),
            "--theme=Dracula\n",
            RuntimeId::Bash,
        ),
        (
            Some(Path::new("config.fish")),
            "set theme Dracula\n",
            RuntimeId::Fish,
        ),
        (
            Some(Path::new("fish_variables")),
            "SETUVAR __fish_initialized:4300\n",
            RuntimeId::Fish,
        ),
        (
            Some(Path::new(".zshrc")),
            "emulate -L zsh\n",
            RuntimeId::Zsh,
        ),
        (Some(Path::new("zshrc")), "emulate -L zsh\n", RuntimeId::Zsh),
        (
            Some(Path::new("theme.zshrc")),
            "emulate -L zsh\n",
            RuntimeId::Zsh,
        ),
        (
            Some(Path::new(".zsh_aliases")),
            "setopt EXTENDED_GLOB\n",
            RuntimeId::Zsh,
        ),
        (
            Some(Path::new("dracula.zsh-theme")),
            "autoload -Uz colors\n",
            RuntimeId::Zsh,
        ),
        (
            None,
            "#!/usr/bin/env fish\nset theme Dracula\n",
            RuntimeId::Fish,
        ),
    ] {
        assert_eq!(detected_runtime(path, source), Some(expected), "{path:?}");
    }
    assert_eq!(
        detect_document_kind(
            Some(Path::new("fish_variables")),
            "SETUVAR fish_user_paths:/opt/homebrew/bin\\x1e/Users/example/bin\n"
        ),
        Some(DocumentKind::with_profile(
            RuntimeId::Fish,
            DocumentProfile::FishVariables
        ))
    );
    for (path, source, expected) in [
        (None, "#!/usr/bin/env zsh\nemulate -L zsh\n", RuntimeId::Zsh),
        (
            Some(Path::new(".gitignore")),
            "target/\n",
            RuntimeId::Ignore,
        ),
        (
            Some(Path::new(".dockerignore")),
            "node_modules/\n",
            RuntimeId::Ignore,
        ),
        (
            Some(Path::new("Dockerfile.dockerignore")),
            ".git\n",
            RuntimeId::Ignore,
        ),
        (Some(Path::new(".npmignore")), "dist/\n", RuntimeId::Ignore),
    ] {
        assert_eq!(detected_runtime(path, source), Some(expected), "{path:?}");
    }
    assert_eq!(
        detect_document_kind(Some(Path::new(".git/info/exclude")), ".direnv/\n")
            .map(|kind| kind.runtime_name()),
        Some("ignore")
    );
    assert_eq!(
        detect_document_kind(Some(Path::new(".config/git/ignore")), "*.local\n")
            .map(|kind| kind.runtime_name()),
        Some("ignore")
    );
    for (path, source, expected) in [
        (
            Some(Path::new(".gitconfig")),
            "[core]\n  editor = nvim\n",
            RuntimeId::GitConfig,
        ),
        (
            Some(Path::new(".gitmodules")),
            "[submodule \"kat\"]\n  path = vendor/kat\n",
            RuntimeId::GitConfig,
        ),
        (
            Some(Path::new(".git/config")),
            "[core]\n  bare = false\n",
            RuntimeId::GitConfig,
        ),
        (
            Some(Path::new(".config/git/config")),
            "[include]\n  path = ~/.gitconfig.local\n",
            RuntimeId::GitConfig,
        ),
        (
            Some(Path::new(".git/worktrees/demo/config.worktree")),
            "[core]\n  sparseCheckout = true\n",
            RuntimeId::GitConfig,
        ),
        (
            Some(Path::new("Dockerfile")),
            "FROM alpine:3.21\n",
            RuntimeId::Dockerfile,
        ),
        (
            Some(Path::new("Dockerfile.dev")),
            "RUN printf '%s\\n' \"$HOME\"\n",
            RuntimeId::Dockerfile,
        ),
        (
            Some(Path::new("example.hcl")),
            "job \"api\" { datacenters = [\"dc1\"] }\n",
            RuntimeId::Hcl,
        ),
        (
            Some(Path::new("example.nomad")),
            "job \"api\" { datacenters = [\"dc1\"] }\n",
            RuntimeId::Hcl,
        ),
        (
            Some(Path::new("main.tf")),
            "resource \"null_resource\" \"preview\" {}\n",
            RuntimeId::Hcl,
        ),
        (
            Some(Path::new("terraform.tfvars")),
            "region = \"us-east-1\"\n",
            RuntimeId::Hcl,
        ),
        (
            Some(Path::new("scripts/profile.ps1")),
            "Write-Host \"kat\"\n",
            RuntimeId::Powershell,
        ),
        (
            None,
            "#!/usr/bin/env pwsh\nWrite-Host \"kat\"\n",
            RuntimeId::Powershell,
        ),
        (
            Some(Path::new("scripts/build.cmd")),
            "@echo off\r\n",
            RuntimeId::Batch,
        ),
    ] {
        assert_eq!(detected_runtime(path, source), Some(expected), "{path:?}");
    }

    let workflow_kind = detect_document_kind(
        Some(Path::new(".github/workflows/build.yml")),
        "name: Build\njobs: {}\n",
    )
    .expect("expected GitHub workflow path to detect as YAML");
    assert_eq!(workflow_kind.runtime_name(), "yaml");
    assert_eq!(
        workflow_kind.profile(),
        DocumentProfile::GitHubActionsWorkflow
    );

    let action_kind = detect_document_kind(
        Some(Path::new("action.yml")),
        "name: Example\nruns:\n  using: composite\n",
    )
    .expect("expected action metadata path to detect as YAML");
    assert_eq!(action_kind.runtime_name(), "yaml");
    assert_eq!(action_kind.profile(), DocumentProfile::GitHubActionMetadata);

    let prek_kind = detect_document_kind(
        Some(Path::new("prek.toml")),
        "minimum_prek_version = \"0.3.8\"\n",
    )
    .expect("expected prek.toml path to detect as TOML");
    assert_eq!(prek_kind.runtime_name(), "toml");
    assert_eq!(prek_kind.profile(), DocumentProfile::PrekConfig);

    for (path, source, expected) in [
        (
            Some(Path::new("proto/theme.proto")),
            "syntax = \"proto3\";\nmessage Theme {}\n",
            RuntimeId::Proto,
        ),
        (
            Some(Path::new("proto/theme.textproto")),
            "name: \"Dracula\"\nenabled: true\n",
            RuntimeId::Textproto,
        ),
        (
            Some(Path::new("docs/theme.tex")),
            "\\section{Theme Preview}\\label{sec:theme}\n",
            RuntimeId::Latex,
        ),
        (
            Some(Path::new("theme.sty")),
            "\\ProvidesPackage{kat-theme}\n",
            RuntimeId::Latex,
        ),
        (
            Some(Path::new("theme.textile")),
            "h1. Kat Textile Preview\n",
            RuntimeId::Textile,
        ),
        (
            Some(Path::new("themes.csv")),
            "name,active\nDracula,true\n",
            RuntimeId::Csv,
        ),
        (
            Some(Path::new("themes.tsv")),
            "name\tactive\nDracula\ttrue\n",
            RuntimeId::Tsv,
        ),
        (
            Some(Path::new("theme.typ")),
            "#let theme = \"Dracula\"\n= Theme Preview\n",
            RuntimeId::Typst,
        ),
        (
            Some(Path::new("theme.tcl")),
            "#!/usr/bin/env tclsh\nputs \"kat\"\n",
            RuntimeId::Tcl,
        ),
        (
            Some(Path::new("src/theme.c")),
            "typedef struct Theme { int enabled; } Theme;\n",
            RuntimeId::C,
        ),
        (
            Some(Path::new("include/theme.hpp")),
            "class ThemePreview {};\n",
            RuntimeId::Cpp,
        ),
        (
            Some(Path::new("src/ThemePreview.java")),
            "public final class ThemePreview {}\n",
            RuntimeId::Java,
        ),
        (
            Some(Path::new("build.gradle.kts")),
            "plugins { kotlin(\"jvm\") }\n",
            RuntimeId::Kotlin,
        ),
        (
            Some(Path::new("src/theme.ts")),
            "export interface Theme { title: string }\n",
            RuntimeId::TypeScript,
        ),
        (
            Some(Path::new("src/ThemePreview.cs")),
            "public sealed class ThemePreview {}\n",
            RuntimeId::Csharp,
        ),
        (
            Some(Path::new("build.gradle")),
            "plugins { id 'java' }\n",
            RuntimeId::Groovy,
        ),
        (
            Some(Path::new("Jenkinsfile")),
            "pipeline { agent any }\n",
            RuntimeId::Groovy,
        ),
        (
            Some(Path::new("theme.patch")),
            "diff --git a/a b/b\n",
            RuntimeId::Diff,
        ),
        (
            Some(Path::new("gradle.properties")),
            "theme.name=Dracula\n",
            RuntimeId::Properties,
        ),
        (
            Some(Path::new("filters/theme.jq")),
            ".themes[] | .name\n",
            RuntimeId::Jq,
        ),
        (
            Some(Path::new("styles/theme.less")),
            "@color: #6272a4;\n",
            RuntimeId::Less,
        ),
        (
            Some(Path::new("graph/theme.dot")),
            "digraph ThemeGraph { a -> b }\n",
            RuntimeId::Dot,
        ),
        (
            Some(Path::new("/etc/nginx/sites-enabled/default.conf")),
            "server { listen 8080; }\n",
            RuntimeId::Nginx,
        ),
        (
            Some(Path::new("src/component.tsx")),
            "export const View = () => <section />;\n",
            RuntimeId::Tsx,
        ),
        (
            Some(Path::new("src/App.vue")),
            "<template><section /></template>\n",
            RuntimeId::Vue,
        ),
        (
            Some(Path::new("src/App.svelte")),
            "<script>let title = 'kat';</script>\n",
            RuntimeId::Svelte,
        ),
        (Some(Path::new(".env")), "APP=kat\n", RuntimeId::Dotenv),
        (
            Some(Path::new(".env.local")),
            "APP=kat\n",
            RuntimeId::Dotenv,
        ),
        (
            Some(Path::new("env.example")),
            "APP=kat\n",
            RuntimeId::Dotenv,
        ),
        (
            Some(Path::new(".editorconfig")),
            "root = true\n",
            RuntimeId::Ini,
        ),
        (
            Some(Path::new("layout.xml")),
            "<layout />\n",
            RuntimeId::Xml,
        ),
        (
            Some(Path::new("page.xhtml")),
            "<section />\n",
            RuntimeId::Html,
        ),
        (
            Some(Path::new("page.shtml")),
            "<section />\n",
            RuntimeId::Html,
        ),
        (
            Some(Path::new("legacy/component.htc")),
            "<PUBLIC:COMPONENT></PUBLIC:COMPONENT>\n",
            RuntimeId::Html,
        ),
        (
            Some(Path::new("templates/page.yaws")),
            "<section></section>\n",
            RuntimeId::Html,
        ),
        (
            Some(Path::new("Makefile")),
            "build:\n\tcargo test\n",
            RuntimeId::Make,
        ),
        (
            Some(Path::new("CMakeLists.txt")),
            "cmake_minimum_required(VERSION 3.28)\n",
            RuntimeId::Cmake,
        ),
        (
            Some(Path::new("build.ninja")),
            "build a: phony\n",
            RuntimeId::Ninja,
        ),
        (
            Some(Path::new("templates/page.html.j2")),
            "<section>{{ title }}</section>\n",
            RuntimeId::Jinja,
        ),
        (
            Some(Path::new("templates/feed.xml.j2")),
            "<feed>{{ title }}</feed>\n",
            RuntimeId::Jinja,
        ),
        (
            Some(Path::new("states/webserver.sls")),
            "webserver:\n  test.configurable_test_state:\n",
            RuntimeId::Jinja,
        ),
        (
            Some(Path::new("templates/page.html.twig")),
            "<section>{{ title }}</section>\n",
            RuntimeId::Twig,
        ),
        (
            Some(Path::new("templates/page.html.eex")),
            "<section><%= @title %></section>\n",
            RuntimeId::Eex,
        ),
        (
            Some(Path::new("templates/page.jsp")),
            "<section><%= title %></section>\n",
            RuntimeId::Jsp,
        ),
        (
            Some(Path::new("templates/page.asp")),
            "<section><%= title %></section>\n",
            RuntimeId::Asp,
        ),
        (
            Some(Path::new("templates/global.asa")),
            "<OBJECT RUNAT=Server SCOPE=Session ID=Theme></OBJECT>\n",
            RuntimeId::Asp,
        ),
        (
            Some(Path::new("templates/page.adp")),
            "<section><%= title %></section>\n",
            RuntimeId::Adp,
        ),
        (
            Some(Path::new("templates/page.html.erb")),
            "<section><%= title %></section>\n",
            RuntimeId::Erb,
        ),
        (
            Some(Path::new("templates/app.js.erb")),
            "const theme = \"<%= theme_name %>\";\n",
            RuntimeId::Erb,
        ),
        (
            Some(Path::new("templates/theme.css.erb")),
            ":root { --accent: <%= accent %>; }\n",
            RuntimeId::Erb,
        ),
        (
            Some(Path::new("db/schema.sql.erb")),
            "CREATE TABLE themes (accent TEXT DEFAULT '<%= accent %>');\n",
            RuntimeId::Erb,
        ),
        (
            Some(Path::new("db/schema.erbsql")),
            "SELECT '<%= accent %>';\n",
            RuntimeId::Erb,
        ),
        (
            Some(Path::new("notebook.ipynb")),
            "{\n  \"cells\": []\n}\n",
            RuntimeId::Json,
        ),
        (
            Some(Path::new("events.jsonl")),
            "{\"kind\":\"theme\"}\n",
            RuntimeId::Json,
        ),
        (
            Some(Path::new("flake.lock")),
            "{\n  \"nodes\": {}\n}\n",
            RuntimeId::Json,
        ),
        (
            Some(Path::new("CITATION.cff")),
            "cff-version: 1.2.0\n",
            RuntimeId::Yaml,
        ),
        (
            Some(Path::new(".clang-format")),
            "BasedOnStyle: LLVM\n",
            RuntimeId::Yaml,
        ),
        (
            Some(Path::new("README.mkd")),
            "# kat\n",
            RuntimeId::Markdown,
        ),
        (
            Some(Path::new("Gemfile")),
            "source \"https://rubygems.org\"\n",
            RuntimeId::Ruby,
        ),
        (
            None,
            "#!/usr/bin/env ruby\nputs ThemePreview.new.render\n",
            RuntimeId::Ruby,
        ),
        (
            Some(Path::new("scripts/theme.nse")),
            "return { action = function() end }\n",
            RuntimeId::Lua,
        ),
        (None, "#!/usr/bin/env lua\nprint('kat')\n", RuntimeId::Lua),
        (
            Some(Path::new("flake.nix")),
            "{ outputs = { self }: {}; }\n",
            RuntimeId::Nix,
        ),
        (
            Some(Path::new("schema.ddl")),
            "CREATE TABLE themes ();",
            RuntimeId::Sql,
        ),
        (
            Some(Path::new("seed.dml")),
            "INSERT INTO themes VALUES (1);",
            RuntimeId::Sql,
        ),
    ] {
        assert_eq!(detected_runtime(path, source), Some(expected), "{path:?}");
    }
}

#[test]
fn newer_languages_and_special_files_detect_by_path() {
    for (path, source, expected) in [
        (
            Some(Path::new("ThemePreview.as")),
            "package { public class ThemePreview { public function render():String { return \"kat\"; } } }\n",
            RuntimeId::ActionScript,
        ),
        (
            Some(Path::new("theme_preview.adb")),
            "procedure Theme_Preview is begin null; end Theme_Preview;\n",
            RuntimeId::Ada,
        ),
        (
            Some(Path::new("theme.ads")),
            "package Theme is end Theme;\n",
            RuntimeId::Ada,
        ),
        (
            Some(Path::new("kat.gpr")),
            "project Kat is end Kat;\n",
            RuntimeId::Ada,
        ),
        (
            Some(Path::new("theme.applescript")),
            "on run\n  display dialog \"kat\"\nend run\n",
            RuntimeId::AppleScript,
        ),
        (
            Some(Path::new("Script Editor")),
            "tell application \"Finder\" to activate\n",
            RuntimeId::AppleScript,
        ),
        (
            Some(Path::new("theme.s")),
            "theme_preview:\n  mov x0, #42\n",
            RuntimeId::Asm,
        ),
        (
            Some(Path::new("theme.asm")),
            "section .text\nglobal theme_preview\n",
            RuntimeId::Nasm,
        ),
        (
            Some(Path::new("theme.yasm")),
            "section .text\nglobal theme_preview\n",
            RuntimeId::Nasm,
        ),
        (
            Some(Path::new("macros.mac")),
            "%macro save_regs 0\n%endmacro\n",
            RuntimeId::Nasm,
        ),
        (
            Some(Path::new("macros.inc")),
            "%define THEME_NAME \"Dracula\"\n",
            RuntimeId::Nasm,
        ),
        (
            Some(Path::new("guide.adoc")),
            "= Kat\n\nNOTE: render all the things.\n",
            RuntimeId::AsciiDoc,
        ),
        (
            Some(Path::new(".ssh/authorized_keys")),
            "restrict ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIKb0pQ1wTq8mC9oG0qD1nVwXnQ7lYQd+O4Wn6H2YfA kat@example\n",
            RuntimeId::AuthorizedKeys,
        ),
        (
            Some(Path::new("id_ed25519.pub")),
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIKb0pQ1wTq8mC9oG0qD1nVwXnQ7lYQd+O4Wn6H2YfA kat@example\n",
            RuntimeId::AuthorizedKeys,
        ),
        (
            Some(Path::new("server.pem")),
            "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n",
            RuntimeId::Pem,
        ),
        (
            Some(Path::new("private.key")),
            concat!(
                "-----BEGIN ENCRYPTED PRIVATE ",
                "KEY-----\nMIIB\n-----END ENCRYPTED PRIVATE ",
                "KEY-----\n",
            ),
            RuntimeId::Pem,
        ),
        (
            Some(Path::new("public.asc")),
            "-----BEGIN PGP PUBLIC KEY BLOCK-----\n\nmQEN\n=abcd\n-----END PGP PUBLIC KEY BLOCK-----\n",
            RuntimeId::OpenPgpArmor,
        ),
    ] {
        assert_eq!(detected_runtime(path, source), Some(expected), "{path:?}");
    }
    assert_eq!(
        detected_runtime(Some(Path::new("archive.gpg")), "not ascii armor\n"),
        None
    );
    for (path, source, expected) in [
        (
            Some(Path::new("theme.awk")),
            "BEGIN { print \"kat\" }\n",
            RuntimeId::Awk,
        ),
        (
            None,
            "#!/usr/bin/env awk -f\nBEGIN { print \"kat\" }\n",
            RuntimeId::Awk,
        ),
        (
            Some(Path::new("references.bib")),
            "@article{kat, title = {Dracula}}\n",
            RuntimeId::Bibtex,
        ),
        (
            Some(Path::new("kat.cabal")),
            "name: kat\n",
            RuntimeId::Cabal,
        ),
        (
            Some(Path::new("theme.cfm")),
            "<cfset theme = \"kat\">\n",
            RuntimeId::Cfml,
        ),
        (
            Some(Path::new("theme.coffee")),
            "class ThemePreview\n",
            RuntimeId::CoffeeScript,
        ),
        (
            Some(Path::new("Cakefile")),
            "task 'build', -> console.log 'kat'\n",
            RuntimeId::CoffeeScript,
        ),
        (
            Some(Path::new("theme.clj")),
            "(defn render-theme [] (println \"kat\"))\n",
            RuntimeId::Clojure,
        ),
        (
            Some(Path::new("ThemeCache/CMakeCache.txt")),
            "KAT_THEME:STRING=kat\n",
            RuntimeId::CmakeCache,
        ),
        (
            Some(Path::new("kat.help")),
            "NAME\n  kat render\n",
            RuntimeId::CommandHelp,
        ),
        (
            Some(Path::new("theme.cpuinfo")),
            "processor : 0\n",
            RuntimeId::CpuInfo,
        ),
        (
            Some(Path::new("root.tab")),
            "*/5 * * * * kat render\n",
            RuntimeId::Crontab,
        ),
        (
            Some(Path::new("theme.cr")),
            "class ThemePreview\nend\n",
            RuntimeId::Crystal,
        ),
        (
            Some(Path::new("/etc/cron.d/kat")),
            "*/5 * * * * root kat render\n",
            RuntimeId::Crontab,
        ),
        (Some(Path::new("theme.d")), "module theme;\n", RuntimeId::D),
        (
            Some(Path::new("sources.list")),
            "deb https://deb.debian.org/debian bookworm main\n",
            RuntimeId::Debsources,
        ),
        (
            Some(Path::new("Main.elm")),
            "module Main exposing (main)\n",
            RuntimeId::Elm,
        ),
        (
            Some(Path::new("theme.eml")),
            "From: theme@example.com\n",
            RuntimeId::Email,
        ),
        (
            Some(Path::new("/var/mail/dcjanus")),
            "From: theme@example.com\n",
            RuntimeId::Email,
        ),
        (
            Some(Path::new("theme.erl")),
            "-module(theme).\n",
            RuntimeId::Erlang,
        ),
        (
            Some(Path::new("Emakefile")),
            "{deps, []}.\n",
            RuntimeId::Erlang,
        ),
        (
            None,
            "#!/usr/bin/env escript\nmain(_) -> ok.\n",
            RuntimeId::Erlang,
        ),
        (
            Some(Path::new("Theme.fs")),
            "module Theme\nlet render = printfn \"kat\"\n",
            RuntimeId::Fsharp,
        ),
        (
            Some(Path::new("Theme.fsi")),
            "module Theme\nval render: unit -> unit\n",
            RuntimeId::FsharpSignature,
        ),
        (
            Some(Path::new("theme.f90")),
            "program theme_preview\nend program theme_preview\n",
            RuntimeId::Fortran,
        ),
        (
            Some(Path::new("theme.f")),
            "      program theme_preview\n      end\n",
            RuntimeId::Fortran,
        ),
        (
            Some(Path::new("theme.namelist")),
            "&theme\n/\n",
            RuntimeId::FortranNamelist,
        ),
        (
            Some(Path::new("Theme.sml")),
            "structure ThemePreview = struct end\n",
            RuntimeId::Sml,
        ),
        (
            Some(Path::new("fstab")),
            "UUID=0123-ABCD / ext4 defaults 0 1\n",
            RuntimeId::Fstab,
        ),
        (
            Some(Path::new("theme_config.h.in")),
            "#define KAT_THEME \"kat\"\n",
            RuntimeId::C,
        ),
        (
            Some(Path::new("theme_config.hpp.in")),
            "namespace kat {}\n",
            RuntimeId::Cpp,
        ),
        (
            Some(Path::new("ThemePreview.php")),
            "<?php echo 'kat';\n",
            RuntimeId::Php,
        ),
        (
            None,
            "#!/usr/bin/env php\n<?php echo 'kat';\n",
            RuntimeId::Php,
        ),
        (
            Some(Path::new("build.sbt")),
            "name := \"kat\"\n",
            RuntimeId::Scala,
        ),
        (
            Some(Path::new("ThemePreview.sol")),
            "pragma solidity ^0.8.28;\ncontract ThemePreview {}\n",
            RuntimeId::Solidity,
        ),
        (
            Some(Path::new("ThemePreview.swift")),
            "print(\"kat\")\n",
            RuntimeId::Swift,
        ),
        (
            None,
            "#!/usr/bin/env swift\nprint(\"kat\")\n",
            RuntimeId::Swift,
        ),
        (
            Some(Path::new("theme.dart")),
            "void main() {}\n",
            RuntimeId::Dart,
        ),
        (
            Some(Path::new("theme.exs")),
            "IO.puts(\"kat\")\n",
            RuntimeId::Elixir,
        ),
        (
            None,
            "#!/usr/bin/env elixir\nIO.puts(\"kat\")\n",
            RuntimeId::Elixir,
        ),
        (
            Some(Path::new("build.zig.zon")),
            ".{ .name = \"kat\" }\n",
            RuntimeId::Zig,
        ),
        (
            Some(Path::new("theme.vy")),
            "@external\ndef render() -> String[32]:\n    return \"kat\"\n",
            RuntimeId::Vyper,
        ),
        (
            Some(Path::new(".ssh/config")),
            "Host github.com\n",
            RuntimeId::SshConfig,
        ),
        (
            Some(Path::new("ssh_config")),
            "Host *\n",
            RuntimeId::SshConfig,
        ),
        (
            Some(Path::new("sshd_config")),
            "Port 22\n",
            RuntimeId::SshConfig,
        ),
        (
            Some(Path::new(".config/git/attributes")),
            "*.rs text eol=lf\n",
            RuntimeId::GitAttributes,
        ),
    ] {
        assert_eq!(detected_runtime(path, source), Some(expected), "{path:?}");
    }
    assert_eq!(
        detect_document_kind(
            Some(Path::new(".git")),
            "gitdir: /Users/dcjanus/Code/kat/.git/worktrees/git-language-support\n"
        )
        .map(|kind| kind.runtime_name()),
        Some("git_link")
    );
    assert_eq!(
        detect_document_kind(
            Some(Path::new(".mailmap")),
            "DCjanus <DCjanus@dcjanus.com>\n"
        )
        .map(|kind| kind.runtime_name()),
        Some("git_mailmap")
    );
    assert_eq!(
        detect_document_kind(
            Some(Path::new("history.gitlog")),
            "commit deadbeef\nAuthor: DCjanus <DCjanus@dcjanus.com>\n"
        )
        .map(|kind| kind.runtime_name()),
        Some("git_log")
    );
    for (path, source, expected) in [
        (
            Some(Path::new("COMMIT_EDITMSG")),
            "Add support\n",
            RuntimeId::GitCommit,
        ),
        (
            Some(Path::new("git-rebase-todo")),
            "pick abc1234 add support\n",
            RuntimeId::GitRebase,
        ),
        (
            Some(Path::new("requirements.txt")),
            "httpx>=0.27\n",
            RuntimeId::Requirements,
        ),
        (
            Some(Path::new("strace")),
            "read(3, \"theme\", 5) = 5\n",
            RuntimeId::Strace,
        ),
        (
            Some(Path::new("theme.styl")),
            ".panel\n  color #fff\n",
            RuntimeId::Stylus,
        ),
        (
            Some(Path::new("syslog")),
            "Oct 15 10:42:00 kat-host sshd[1234]: Accepted publickey\n",
            RuntimeId::Syslog,
        ),
        (
            Some(Path::new("todo.txt")),
            "(A) 2026-04-12 Finish kat +syntax @terminal due:2026-04-13\n",
            RuntimeId::TodoTxt,
        ),
        (
            Some(Path::new("theme_preview.vhd")),
            "entity theme_preview is\nend entity theme_preview;\n",
            RuntimeId::Vhdl,
        ),
        (
            Some(Path::new("theme.v")),
            "module theme_preview; endmodule\n",
            RuntimeId::Verilog,
        ),
        (
            Some(Path::new("theme.sv")),
            "module theme_preview; logic active; endmodule\n",
            RuntimeId::SystemVerilog,
        ),
        (
            Some(Path::new("theme.vh")),
            "`define THEME_ACCENT 8'hbd\n",
            RuntimeId::SystemVerilog,
        ),
        (
            Some(Path::new("theme.wgsl")),
            "@fragment\nfn main() -> @location(0) vec4<f32> {\n  return vec4<f32>(1.0);\n}\n",
            RuntimeId::Wgsl,
        ),
        (
            Some(Path::new(".vimrc")),
            "let g:kat_theme = 'dracula'\n",
            RuntimeId::Vim,
        ),
        (
            Some(Path::new("theme.vimhelp")),
            "= Theme Preview =\n",
            RuntimeId::VimHelp,
        ),
        (
            Some(Path::new("theme.varlink")),
            "interface io.kat.theme\n",
            RuntimeId::Varlink,
        ),
        (
            Some(Path::new("/etc/apache2/sites-enabled/kat.conf")),
            "ServerName kat.example.com\n",
            RuntimeId::Apache,
        ),
        (
            Some(Path::new("theme.scss")),
            "$color: #fff;\n",
            RuntimeId::Scss,
        ),
        (
            Some(Path::new("theme.sass")),
            "$color: #fff\n",
            RuntimeId::Sass,
        ),
    ] {
        assert_eq!(detected_runtime(path, source), Some(expected), "{path:?}");
    }
}

#[test]
fn tree_sitter_query_files_detect_by_grammar_queries_path() {
    let source = read_file(&fixture_path("query/grammars/lua/queries/highlights.scm"));

    assert_eq!(
        crate::api::detected_language_name(
            Some(Path::new("grammars/lua/queries/highlights.scm")),
            &source
        ),
        Some("query")
    );
    assert_eq!(
        crate::api::detected_language_name(
            Some(Path::new("grammars/lua/queries/injections.scm")),
            &source
        ),
        Some("query")
    );
    assert_eq!(
        crate::api::detected_language_name(
            Some(Path::new("grammars/lua/queries/locals.scm")),
            &source
        ),
        Some("query")
    );
    assert_eq!(
        crate::api::detected_language_name(Some(Path::new("queries/highlights.scm")), &source),
        None
    );
}

#[test]
fn fish_variables_profile_detects_and_highlights_persistent_record_syntax() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("fish/fish_variables");
    let source = read_file(&path);
    let document_kind = detect_document_kind(Some(path.as_path()), &source)
        .expect("expected fish_variables fixture to detect as fish");
    let overlays = semantic_capture_spans(document_kind, &source)
        .expect("expected fish_variables semantic overlays to resolve");
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert_eq!(document_kind.runtime_name(), "fish");
    assert_eq!(document_kind.profile(), DocumentProfile::FishVariables);
    assert!(
        overlays.iter().any(|span| {
            span.capture == "keyword.directive" && &source[span.range.clone()] == "SETUVAR"
        }),
        "expected fish_variables profile to capture record directives"
    );
    assert!(
        overlays.iter().any(|span| {
            span.capture == "string.special.key" && &source[span.range.clone()] == "fish_user_paths"
        }),
        "expected fish_variables profile to capture variable names as keys"
    );
    assert!(
        overlays.iter().any(|span| {
            span.capture == "punctuation.special" && &source[span.range.clone()] == "\\x1e"
        }),
        "expected fish_variables profile to treat list separators specially"
    );
    assert!(
        overlays.iter().any(|span| {
            span.capture == "string.escape" && &source[span.range.clone()] == "\\x20"
        }),
        "expected fish_variables profile to capture hex escapes inside values"
    );
    assert!(
        overlays.iter().any(|span| {
            span.capture == "string.special.path"
                && &source[span.range.clone()] == "/opt/homebrew/bin"
        }),
        "expected fish_variables profile to classify path-like list entries"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;80;250;123mSETUVAR"),
        "expected fish_variables directives to receive directive styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253mfish_user_paths"),
        "expected fish_variables keys to reuse config-key styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;184;108m4300"),
        "expected numeric persistent values to receive number styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m/opt/homebrew/bin"),
        "expected path-like values to remain visible and string-highlighted"
    );
}

#[test]
fn actions_lock_is_detected_as_plain_yaml_by_exact_filename() {
    let fixture = fixture_path("yaml/actions.lock");
    let source = read_file(&fixture);

    for path in [
        Path::new("actions.lock"),
        Path::new(".github/workflows/actions.lock"),
        fixture.as_path(),
        Path::new("/example/project/.github/workflows/actions.lock"),
    ] {
        assert_eq!(
            detect_document_kind(Some(path), &source),
            Some(DocumentKind::plain(RuntimeId::Yaml)),
            "{path:?}"
        );
    }

    for path in [
        Some(Path::new(".github/workflows/other.lock")),
        Some(Path::new("actions.lock.bak")),
        Some(Path::new("my-actions.lock")),
        None,
    ] {
        assert_eq!(detect_document_kind(path, &source), None, "{path:?}");
    }
}

#[test]
fn cargo_lock_is_detected_as_toml() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("toml/Cargo.lock");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b["),
        "expected ANSI highlighting for {}",
        path.display()
    );
    assert!(
        rendered.contains("\"kat\""),
        "expected TOML string content to remain visible for {}",
        path.display()
    );
}

#[test]
fn uv_lock_is_detected_as_toml() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("toml/uv.lock");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b["),
        "expected ANSI highlighting for {}",
        path.display()
    );
    assert!(
        rendered.contains("\"kat\""),
        "expected TOML string content to remain visible for {}",
        path.display()
    );
}

#[test]
fn ignore_files_are_detected_and_highlight_ignore_syntax() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("ignore/.gitignore");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;98;114;164m# ignore build output"),
        "expected ignore-file comments to reuse comment styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198m!"),
        "expected ignore-file negation to use operator styling"
    );
}

#[test]
fn git_config_files_are_detected_and_highlight_core_syntax() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("git_config/.gitconfig");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;139;233;253minclude"),
        "expected include section names to reuse builtin styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m~/.gitconfig.local"),
        "expected include paths to receive path-oriented string styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;241;250;140m!.git/hooks/post-checkout"),
        "expected shell-command style values to remain string-highlighted"
    );
    assert!(
        rendered.contains("\x1b[38;2;189;147;249mtrue"),
        "expected boolean values to receive builtin constant styling"
    );
}

#[test]
fn armor_files_detect_and_highlight_textual_boundaries() {
    let theme = Theme::for_mode(ColorMode::TrueColor);

    let cert_path = fixture_path("armor/server.pem");
    let cert_source = read_file(&cert_path);
    let cert_rendered = render_with_theme(Some(cert_path.as_path()), &cert_source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", cert_path.display()));
    assert!(
        cert_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m-----BEGIN "),
        "expected PEM boundaries to receive directive styling"
    );
    assert!(
        cert_rendered.contains("\x1b[38;2;139;233;253mCERTIFICATE"),
        "expected PEM labels to receive type styling"
    );

    let private_source = concat!(
        "-----BEGIN ENCRYPTED PRIVATE ",
        "KEY-----\n",
        "Proc-Type: 4,ENCRYPTED\n",
        "DEK-Info: AES-256-CBC,0123456789ABCDEF0123456789ABCDEF\n\n",
        "MIIBvTBXBgkqhkiG9w0BBQ0wSjApBgkqhkiG9w0BBQwwHAQIkatprivatefixture\n",
        "-----END ENCRYPTED PRIVATE ",
        "KEY-----\n",
    );
    let private_analysis =
        analysis_snapshot_for_path(Path::new("private.key"), private_source, &theme);
    assert_eq!(
        private_analysis
            .detected_document_kind
            .expect("kind")
            .runtime_name,
        "pem"
    );
    let private_label = private_source
        .find("ENCRYPTED PRIVATE KEY")
        .expect("private label should exist");
    let private_style = private_analysis
        .spans
        .iter()
        .find(|span| span.start <= private_label && private_label < span.end)
        .and_then(|span| span.style.as_ref())
        .expect("private label should be styled");
    assert_eq!(private_style.color_name, "orange");
    assert!(private_style.bold);

    let pgp_path = fixture_path("armor/public.asc");
    let pgp_source = read_file(&pgp_path);
    let pgp_analysis = analysis_snapshot_for_path(pgp_path.as_path(), &pgp_source, &theme);
    assert_eq!(
        pgp_analysis
            .detected_document_kind
            .expect("kind")
            .runtime_name,
        "openpgp_armor"
    );

    let version_start = pgp_source.find("Version").expect("header should exist");
    let version_style = pgp_analysis
        .spans
        .iter()
        .find(|span| span.start <= version_start && version_start < span.end)
        .and_then(|span| span.style.as_ref())
        .expect("header key should be styled");
    assert_eq!(version_style.color_name, "foreground");

    let checksum_start = pgp_source.find("=abcd").expect("checksum should exist");
    let checksum_style = pgp_analysis
        .spans
        .iter()
        .find(|span| span.start <= checksum_start && checksum_start < span.end)
        .and_then(|span| span.style.as_ref())
        .expect("checksum should be styled");
    assert_eq!(checksum_style.color_name, "yellow");
}

#[test]
fn git_link_files_are_detected_and_highlight_gitdir_paths() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("git_link/worktree.gitdir");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198mgitdir"),
        "expected gitdir keys to receive keyword styling"
    );
    assert!(
        rendered.contains(
            "\x1b[38;2;241;250;140m/Users/dcjanus/Code/kat/.git/worktrees/git-language-support"
        ),
        "expected gitdir targets to receive path-oriented string styling"
    );
}

#[test]
fn git_mailmap_files_are_detected_and_highlight_names_and_emails() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("git_mailmap/.mailmap");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;80;250;123mDCjanus"),
        "expected canonical names to receive tag-like styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m<DCjanus@dcjanus.com>"),
        "expected email addresses to receive builtin-like styling"
    );
    assert!(
        rendered.contains("\x1b[3m\x1b[38;2;98;114;164m# canonical identities"),
        "expected mailmap comments to reuse comment styling"
    );
}

#[test]
fn git_log_files_are_detected_and_highlight_metadata_and_diff_blocks() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("git_log/history.gitlog");
    let source = read_file(&path);
    let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

    assert!(
        rendered.contains("\x1b[38;2;255;121;198mcommit"),
        "expected commit headers to receive keyword styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;255;121;198mAuthor:"),
        "expected metadata keys to receive keyword styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;139;233;253m<DCjanus@dcjanus.com>"),
        "expected metadata emails to receive builtin-like styling"
    );
    assert!(
        rendered.contains("\x1b[38;2;80;250;123muse std::ffi::OsStr;"),
        "expected embedded diff hunks to reuse diff addition styling"
    );
}

#[test]
fn python_docstrings_use_dedicated_runtime_and_detect_styles() {
    let theme = Theme::for_mode(ColorMode::TrueColor);
    let path = fixture_path("python/docstrings.py");
    let source = read_file(&path);
    let analysis = analysis_snapshot_for_path(path.as_path(), &source, &theme);
    let module_docstring = find_nested_region(
        &analysis,
        "python_docstring",
        "Theme preview helpers.",
        &source,
    );
    let init_docstring = find_nested_region(
        &analysis,
        "python_docstring",
        "Initialize the preview instance.",
        &source,
    );
    let classmethod_docstring = find_nested_region(
        &analysis,
        "python_docstring",
        "Create a preview from a plain theme name.",
        &source,
    );
    let render_docstring = find_nested_region(
        &analysis,
        "python_docstring",
        "Render the preview value.",
        &source,
    );

    assert!(
        !nested_runtime_names(&analysis).contains(&"markdown"),
        "expected python docstrings to stop defaulting to markdown runtime"
    );
    assert!(
        module_docstring.resolved_document_kind.profile == "python_docstring_rest",
        "expected module docstring to resolve to reST-style profile"
    );
    assert!(
        init_docstring.resolved_document_kind.profile == "python_docstring_google",
        "expected __init__ docstring to resolve to Google-style profile"
    );
    assert!(
        classmethod_docstring.resolved_document_kind.profile == "python_docstring_rest",
        "expected classmethod docstring to resolve to reST-style profile"
    );
    assert!(
        render_docstring.resolved_document_kind.profile == "python_docstring_numpy",
        "expected render docstring to resolve to NumPy-style profile"
    );
}

#[test]
fn sql_dialect_detector_uses_path_and_content_signals() {
    assert_eq!(
        detect_sql_dialect(Some(Path::new("db/schema.postgres.sql")), "SELECT 1;").runtime_name(),
        "sql_postgres"
    );
    assert_eq!(
        detect_sql_dialect(Some(Path::new("db/schema.psql")), "SELECT 1;").runtime_name(),
        "sql_postgres"
    );
    assert_eq!(
        detect_sql_dialect(
            Some(Path::new("db/schema.sql")),
            "INSERT INTO themes VALUES ($1) ON CONFLICT (id) DO NOTHING RETURNING id;"
        )
        .runtime_name(),
        "sql_postgres"
    );
    assert_eq!(
        detect_sql_dialect(Some(Path::new("db/schema.mysql")), "SELECT 1;").runtime_name(),
        "sql_mysql"
    );
    assert_eq!(
        detect_sql_dialect(
            Some(Path::new("db/schema.sql")),
            "CREATE TABLE `themes` (`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT) ENGINE=InnoDB;"
        )
        .runtime_name(),
        "sql_mysql"
    );
    assert_eq!(
        detect_sql_dialect(Some(Path::new("db/schema.sqlite3")), "SELECT 1;").runtime_name(),
        "sql_sqlite"
    );
    assert_eq!(
        detect_sql_dialect(
            Some(Path::new("db/schema.sql")),
            "CREATE TABLE themes (id INTEGER PRIMARY KEY AUTOINCREMENT) STRICT, WITHOUT ROWID;"
        )
        .runtime_name(),
        "sql_sqlite"
    );
    assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "CREATE FUNCTION refresh_theme_cache() RETURNS void LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE AS $$ BEGIN RETURN; END; $$; CREATE INDEX theme_payload_gin ON theme_snapshots USING GIN (payload);"
            )
            .runtime_name(),
            "sql_postgres"
        );
    assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "CREATE UNLOGGED TABLE theme_cache (id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY); CREATE EXTENSION IF NOT EXISTS pgcrypto;"
            )
            .runtime_name(),
            "sql_postgres"
        );
    assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "CREATE TABLE theme_snapshots (id INT UNSIGNED NOT NULL AUTO_INCREMENT, name VARCHAR(255) CHARACTER SET utf8mb4, PRIMARY KEY (id)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4; INSERT INTO theme_snapshots (id, name) VALUES (1, 'Dracula') ON DUPLICATE KEY UPDATE name = VALUES(name);"
            )
            .runtime_name(),
            "sql_mysql"
        );
    assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "CREATE TABLE `theme_snapshots` (`id` BIGINT UNSIGNED ZEROFILL NOT NULL AUTO_INCREMENT) ENGINE=InnoDB; INSERT IGNORE INTO `theme_snapshots` (`id`) VALUES (1); SHOW CREATE TABLE `theme_snapshots`;"
            )
            .runtime_name(),
            "sql_mysql"
        );
    assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "PRAGMA foreign_keys = ON; VACUUM; INSERT OR IGNORE INTO theme_cache (slug) VALUES ('dracula');"
            )
            .runtime_name(),
            "sql_sqlite"
        );
    assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "BEGIN IMMEDIATE; ATTACH DATABASE 'theme-cache.db' AS cache; REINDEX; INSERT OR FAIL INTO theme_cache (slug) VALUES ('kat');"
            )
            .runtime_name(),
            "sql_sqlite"
        );
    assert_eq!(
        detect_sql_dialect(
            Some(Path::new("db/schema.mysql.sql")),
            "INSERT INTO theme_snapshots (payload) VALUES ($1::jsonb) RETURNING id;"
        )
        .runtime_name(),
        "sql_postgres"
    );
    assert_eq!(
        detect_sql_dialect(
            Some(Path::new("db/schema.sql")),
            "SELECT id, name FROM themes;"
        )
        .runtime_name(),
        "sql"
    );
}

#[test]
fn detect_language_uses_extensionless_sql_and_graphql_heuristics() {
    assert!(matches!(
        detected_runtime(None, "SELECT id FROM themes WHERE enabled = true;"),
        Some(RuntimeId::Sql)
    ));
    assert!(matches!(
        detected_runtime(
            None,
            "query ThemeBySlug($slug: ID!) { theme(slug: $slug) { id } }"
        ),
        Some(RuntimeId::Graphql)
    ));
}
