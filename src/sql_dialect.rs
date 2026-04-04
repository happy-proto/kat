use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SqlDialect {
    Generic,
    Postgres,
    Mysql,
    Sqlite,
}

impl SqlDialect {
    pub const fn runtime_name(self) -> &'static str {
        match self {
            Self::Generic => "sql",
            Self::Postgres => "sql_postgres",
            Self::Mysql => "sql_mysql",
            Self::Sqlite => "sql_sqlite",
        }
    }

    pub fn from_language_hint(name: &str) -> Option<Self> {
        match name {
            "sql" => Some(Self::Generic),
            "sql_postgres" => Some(Self::Postgres),
            "sql_mysql" => Some(Self::Mysql),
            "sql_sqlite" => Some(Self::Sqlite),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct SqlDialectScores {
    postgres: i32,
    mysql: i32,
    sqlite: i32,
}

pub fn resolve_sql_runtime(
    source_path: Option<&Path>,
    requested_name: &str,
    source: &str,
) -> &'static str {
    match SqlDialect::from_language_hint(requested_name) {
        Some(SqlDialect::Generic) => detect_sql_dialect(source_path, source).runtime_name(),
        Some(dialect) => dialect.runtime_name(),
        None => "sql",
    }
}

pub fn detect_sql_dialect(source_path: Option<&Path>, source: &str) -> SqlDialect {
    let mut scores = SqlDialectScores::default();
    if let Some(path) = source_path {
        apply_path_hints(path, &mut scores);
    }
    apply_content_hints(source, &mut scores);

    let (dialect, score, runner_up) = best_dialect(scores);
    if score < 6 || score - runner_up < 3 {
        SqlDialect::Generic
    } else {
        dialect
    }
}

fn apply_path_hints(path: &Path, scores: &mut SqlDialectScores) {
    let lower = path.to_string_lossy().to_ascii_lowercase();

    if contains_any(&lower, &[".postgres.", ".postgresql.", ".pgsql.", ".psql."])
        || lower.ends_with(".postgres.sql")
        || lower.ends_with(".pgsql.sql")
        || lower.ends_with(".psql.sql")
        || lower.ends_with(".postgres")
        || lower.ends_with(".postgresql")
        || lower.ends_with(".pgsql")
        || lower.ends_with(".psql")
    {
        scores.postgres += 12;
    }

    if contains_any(&lower, &[".mysql.", ".mariadb."])
        || lower.ends_with(".mysql.sql")
        || lower.ends_with(".mariadb.sql")
        || lower.ends_with(".mysql")
        || lower.ends_with(".mariadb")
    {
        scores.mysql += 12;
    }

    if contains_any(&lower, &[".sqlite.", ".sqlite3."])
        || lower.ends_with(".sqlite.sql")
        || lower.ends_with(".sqlite3.sql")
        || lower.ends_with(".sqlite")
        || lower.ends_with(".sqlite3")
    {
        scores.sqlite += 12;
    }
}

fn apply_content_hints(source: &str, scores: &mut SqlDialectScores) {
    let upper = source.to_ascii_uppercase();

    score_against_if(source.contains('`'), 2, &mut scores.postgres);
    score_against_if(source.contains('`'), 1, &mut scores.sqlite);
    score_exclusive(
        upper.contains("$$"),
        12,
        &mut scores.postgres,
        &mut [&mut scores.mysql, &mut scores.sqlite],
    );
    score_exclusive(
        upper.contains("::"),
        8,
        &mut scores.postgres,
        &mut [&mut scores.mysql, &mut scores.sqlite],
    );
    score_exclusive(
        upper.contains(" ILIKE "),
        8,
        &mut scores.postgres,
        &mut [&mut scores.mysql, &mut scores.sqlite],
    );
    score_exclusive(
        upper.contains(" JSONB"),
        8,
        &mut scores.postgres,
        &mut [&mut scores.mysql, &mut scores.sqlite],
    );
    score_if(upper.contains(" ARRAY["), 6, &mut scores.postgres);
    score_exclusive(
        upper.contains(" DISTINCT ON "),
        8,
        &mut scores.postgres,
        &mut [&mut scores.mysql, &mut scores.sqlite],
    );
    score_exclusive(
        upper.contains(" BIGSERIAL"),
        8,
        &mut scores.postgres,
        &mut [&mut scores.mysql, &mut scores.sqlite],
    );
    score_if(upper.contains(" SERIAL"), 6, &mut scores.postgres);
    score_exclusive(
        upper.contains(" GENERATED ALWAYS AS IDENTITY"),
        10,
        &mut scores.postgres,
        &mut [&mut scores.mysql, &mut scores.sqlite],
    );
    score_exclusive(
        upper.contains(" LANGUAGE PLPGSQL"),
        10,
        &mut scores.postgres,
        &mut [&mut scores.mysql, &mut scores.sqlite],
    );
    score_exclusive(
        upper.contains(" CREATE EXTENSION ") || upper.starts_with("CREATE EXTENSION "),
        10,
        &mut scores.postgres,
        &mut [&mut scores.mysql, &mut scores.sqlite],
    );
    score_exclusive(
        upper.contains(" UNLOGGED "),
        8,
        &mut scores.postgres,
        &mut [&mut scores.mysql, &mut scores.sqlite],
    );
    score_exclusive(
        upper.contains(" COPY ") && upper.contains(" FROM STDIN"),
        10,
        &mut scores.postgres,
        &mut [&mut scores.mysql, &mut scores.sqlite],
    );
    score_exclusive(
        upper.contains("->>") || upper.contains(" #>> "),
        8,
        &mut scores.postgres,
        &mut [&mut scores.mysql, &mut scores.sqlite],
    );
    score_if(
        upper.contains(" SET SEARCH_PATH ")
            || upper.contains(" PG_CATALOG.")
            || upper.contains(" PG_TEMP."),
        6,
        &mut scores.postgres,
    );
    score_if(upper.contains(" IMMUTABLE"), 6, &mut scores.postgres);
    score_if(upper.contains(" STABLE"), 6, &mut scores.postgres);
    score_if(upper.contains(" VOLATILE"), 6, &mut scores.postgres);
    score_if(upper.contains(" PARALLEL "), 6, &mut scores.postgres);
    score_if(upper.contains(" SECURITY DEFINER"), 6, &mut scores.postgres);
    score_if(upper.contains(" SECURITY INVOKER"), 6, &mut scores.postgres);
    score_if(
        upper.contains(" USING GIN ")
            || upper.contains(" USING GIST ")
            || upper.contains(" USING BRIN ")
            || upper.contains(" USING BTREE ")
            || upper.contains(" USING HASH "),
        8,
        &mut scores.postgres,
    );
    score_exclusive(
        upper.contains(" RETURN QUERY "),
        8,
        &mut scores.postgres,
        &mut [&mut scores.mysql, &mut scores.sqlite],
    );
    score_if(upper.contains(" ON CONFLICT "), 3, &mut scores.postgres);
    score_if(upper.contains(" RETURNING "), 2, &mut scores.postgres);
    score_if(
        upper.contains("$1") || upper.contains("$2"),
        4,
        &mut scores.postgres,
    );

    score_if(source.contains('`'), 8, &mut scores.mysql);
    score_exclusive(
        upper.contains(" AUTO_INCREMENT"),
        10,
        &mut scores.mysql,
        &mut [&mut scores.postgres, &mut scores.sqlite],
    );
    score_exclusive(
        upper.contains(" ENGINE="),
        10,
        &mut scores.mysql,
        &mut [&mut scores.postgres, &mut scores.sqlite],
    );
    score_exclusive(
        upper.contains(" DEFAULT CHARSET="),
        8,
        &mut scores.mysql,
        &mut [&mut scores.postgres, &mut scores.sqlite],
    );
    score_if(upper.contains(" CHARACTER SET "), 6, &mut scores.mysql);
    score_if(upper.contains(" UNSIGNED"), 6, &mut scores.mysql);
    score_if(upper.contains(" ZEROFILL"), 8, &mut scores.mysql);
    score_if(upper.contains(" INSERT IGNORE "), 8, &mut scores.mysql);
    score_if(upper.contains(" TINYINT(1)"), 4, &mut scores.mysql);
    score_exclusive(
        upper.contains(" LOCK IN SHARE MODE"),
        8,
        &mut scores.mysql,
        &mut [&mut scores.postgres, &mut scores.sqlite],
    );
    score_exclusive(
        upper.contains(" LOCK TABLES ") || upper.contains(" UNLOCK TABLES"),
        10,
        &mut scores.mysql,
        &mut [&mut scores.postgres, &mut scores.sqlite],
    );
    score_if(upper.contains(" REPLACE INTO "), 8, &mut scores.mysql);
    score_exclusive(
        upper.contains(" SHOW CREATE TABLE "),
        10,
        &mut scores.mysql,
        &mut [&mut scores.postgres, &mut scores.sqlite],
    );
    score_if(
        upper.contains(" ON UPDATE CURRENT_TIMESTAMP"),
        6,
        &mut scores.mysql,
    );
    score_exclusive(
        upper.contains(" ON DUPLICATE KEY UPDATE "),
        10,
        &mut scores.mysql,
        &mut [&mut scores.postgres, &mut scores.sqlite],
    );
    score_if(has_mysql_limit_syntax(&upper), 6, &mut scores.mysql);

    score_exclusive(
        upper.contains(" WITHOUT ROWID"),
        12,
        &mut scores.sqlite,
        &mut [&mut scores.postgres, &mut scores.mysql],
    );
    score_exclusive(
        upper.contains(" AUTOINCREMENT"),
        10,
        &mut scores.sqlite,
        &mut [&mut scores.postgres, &mut scores.mysql],
    );
    score_exclusive(
        upper.contains(" INSERT OR REPLACE "),
        8,
        &mut scores.sqlite,
        &mut [&mut scores.postgres, &mut scores.mysql],
    );
    score_exclusive(
        upper.contains(" INSERT OR IGNORE "),
        8,
        &mut scores.sqlite,
        &mut [&mut scores.postgres, &mut scores.mysql],
    );
    score_exclusive(
        upper.contains(" PRAGMA "),
        12,
        &mut scores.sqlite,
        &mut [&mut scores.postgres, &mut scores.mysql],
    );
    score_exclusive(
        upper.contains(" VACUUM"),
        8,
        &mut scores.sqlite,
        &mut [&mut scores.postgres, &mut scores.mysql],
    );
    score_exclusive(
        upper.contains(" BEGIN IMMEDIATE")
            || upper.contains("BEGIN EXCLUSIVE")
            || upper.starts_with("BEGIN IMMEDIATE")
            || upper.starts_with("BEGIN EXCLUSIVE"),
        10,
        &mut scores.sqlite,
        &mut [&mut scores.postgres, &mut scores.mysql],
    );
    score_exclusive(
        upper.contains(" ATTACH DATABASE "),
        10,
        &mut scores.sqlite,
        &mut [&mut scores.postgres, &mut scores.mysql],
    );
    score_exclusive(
        upper.contains(" REINDEX "),
        8,
        &mut scores.sqlite,
        &mut [&mut scores.postgres, &mut scores.mysql],
    );
    score_exclusive(
        upper.contains(" GLOB "),
        6,
        &mut scores.sqlite,
        &mut [&mut scores.postgres, &mut scores.mysql],
    );
    score_if(upper.contains(" STRICT"), 6, &mut scores.sqlite);
    score_if(
        upper.contains(" INSERT OR FAIL ")
            || upper.contains(" INSERT OR ABORT ")
            || upper.contains(" INSERT OR ROLLBACK "),
        8,
        &mut scores.sqlite,
    );
}

fn best_dialect(scores: SqlDialectScores) -> (SqlDialect, i32, i32) {
    let mut entries = [
        (SqlDialect::Postgres, scores.postgres),
        (SqlDialect::Mysql, scores.mysql),
        (SqlDialect::Sqlite, scores.sqlite),
    ];
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.1));
    (entries[0].0, entries[0].1, entries[1].1)
}

fn contains_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| text.contains(pattern))
}

fn score_if(condition: bool, delta: i32, score: &mut i32) {
    if condition {
        *score += delta;
    }
}

fn score_against_if(condition: bool, delta: i32, score: &mut i32) {
    if condition {
        *score -= delta;
    }
}

fn score_exclusive(
    condition: bool,
    positive_delta: i32,
    winner: &mut i32,
    losers: &mut [&mut i32],
) {
    if !condition {
        return;
    }

    *winner += positive_delta;
    let penalty = exclusive_penalty(positive_delta);
    for loser in losers.iter_mut() {
        **loser -= penalty;
    }
}

fn exclusive_penalty(positive_delta: i32) -> i32 {
    positive_delta.max(2) / 2
}

fn has_mysql_limit_syntax(upper: &str) -> bool {
    upper.lines().any(|line| {
        line.find("LIMIT ").is_some_and(|start| {
            let tail = &line[start + "LIMIT ".len()..];
            let mut parts = tail.split(',');
            let first = parts.next().unwrap_or_default().trim();
            let second = parts.next().unwrap_or_default().trim();
            !first.is_empty()
                && !second.is_empty()
                && first.chars().all(|ch| ch.is_ascii_digit())
                && second.chars().take_while(|ch| ch.is_ascii_digit()).count() > 0
        })
    })
}
