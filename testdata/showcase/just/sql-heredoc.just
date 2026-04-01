sql-heredoc:
    psql <<'SQL'
    WITH ranked AS (
      SELECT slug, name, enabled
      FROM themes
      WHERE enabled = true
    )
    SELECT slug, name
    FROM ranked
    ORDER BY name;
    SQL
