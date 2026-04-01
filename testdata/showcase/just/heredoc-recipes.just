default:
    python <<'PY'
    class Preview:
        def render(self) -> int:
            return 42

    print(Preview().render())
    PY

    node <<'JS'
    const payload = { ready: true };
    console.log(JSON.stringify(payload));
    JS

    bash <<'SH'
    source .env
    printf '%s\n' "$HOME"
    SH

    psql <<'SQL'
    SELECT name, slug
    FROM themes
    WHERE enabled = true
    ORDER BY name;
    SQL
