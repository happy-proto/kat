pub(crate) fn normalize_language_name(name: &str) -> Option<&str> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }

    let head = trimmed
        .trim_start_matches('{')
        .split(|ch: char| ch.is_whitespace() || matches!(ch, ',' | '{' | '}'))
        .next()
        .unwrap_or(trimmed)
        .rsplit('/')
        .next()
        .unwrap_or(trimmed)
        .trim_matches(|ch| matches!(ch, '{' | '}' | '"' | '\''))
        .trim_end_matches(".exe");

    if head.is_empty() {
        return None;
    }

    Some(match head {
        "c" => "c",
        "cs" | "c#" | "csharp" => "csharp",
        "cpp" | "cxx" | "cc" | "c++" | "cplusplus" => "cpp",
        "diff" | "patch" => "diff",
        "dot" | "graphviz" | "gv" => "dot",
        "dart" => "dart",
        "elixir" | "iex" => "elixir",
        "git-commit" | "gitcommit" => "git_commit",
        "git-rebase" | "git-rebase-todo" | "git_rebase" | "gitrebase" => "git_rebase",
        "gitattributes" | "git-attributes" => "gitattributes",
        "gradle" | "groovy" | "gvy" => "groovy",
        "apache" | "apacheconf" | "httpd" => "apache",
        "java" | "bsh" => "java",
        "js" | "node" | "nodejs" | "bun" => "javascript",
        "jq" => "jq",
        "less" => "less",
        "kt" | "kts" | "kotlin" => "kotlin",
        "nginx" => "nginx",
        "php" => "php",
        "properties" => "properties",
        "pip-requirements" | "requirements" | "requirements.txt" => "requirements",
        "sass" => "sass",
        "scala" | "sbt" => "scala",
        "scss" => "scss",
        "ssh" | "ssh-config" | "ssh_config" => "ssh_config",
        "swift" => "swift",
        "ts" | "mts" | "cts" => "typescript",
        "tsx" | "typescriptreact" => "tsx",
        "golang" => "go",
        "rb" | "ruby" => "ruby",
        "lua" => "lua",
        "nix" => "nix",
        "zig" => "zig",
        "powershell" | "pwsh" => "powershell",
        "cmd" | "bat" | "batch" => "batch",
        "sql:postgres" | "sql-postgres" | "sql_postgres" => "sql_postgres",
        "postgres" | "postgresql" | "pgsql" | "psql" => "sql_postgres",
        "sql:mysql" | "sql-mysql" | "sql_mysql" => "sql_mysql",
        "mysql" | "mariadb" => "sql_mysql",
        "sql:sqlite" | "sql:sqlite3" | "sql-sqlite" | "sql_sqlite" => "sql_sqlite",
        "sqlite" | "sqlite3" => "sql_sqlite",
        "py" | "python3" | "py3" | "uv" => "python",
        "regex:javascript" | "regex-javascript" | "regex_javascript" => "regex_javascript",
        "regex:python" | "regex-python" | "regex_python" => "regex_python",
        "regex:rust" | "regex-rust" | "regex_rust" => "regex_rust",
        "regex:go" | "regex-go" | "regex_go" => "regex_go",
        "regex:posix" | "regex-posix" | "regex_posix" | "ere" | "extended-regexp" => "regex_posix",
        "rs" => "rust",
        "sh" | "shell" => "bash",
        "zsh" => "zsh",
        "fish" => "fish",
        "gql" | "graphqls" => "graphql",
        "md" => "markdown",
        "yml" => "yaml",
        "dotenv" | "env" => "dotenv",
        "ini" => "ini",
        "xml" => "xml",
        "makefile" | "make" => "make",
        "cmake" => "cmake",
        "ninja" => "ninja",
        "jinja" | "jinja2" | "j2" => "jinja",
        "twig" => "twig",
        "erb" | "rhtml" => "erb",
        "vue" => "vue",
        "svelte" | "svlt" => "svelte",
        other => other,
    })
}

pub(crate) fn shebang_interpreter_name(line: &str) -> Option<&str> {
    let mut parts = line
        .strip_prefix("#!")?
        .split_whitespace()
        .map(|part| part.rsplit('/').next().unwrap_or(part));
    let first = parts.next()?;

    if matches!(first, "env" | "/usr/bin/env") {
        for part in parts {
            if part.starts_with('-') {
                continue;
            }
            return Some(part);
        }
        None
    } else {
        Some(first)
    }
}
