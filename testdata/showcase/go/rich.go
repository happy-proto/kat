//go:build darwin

package showcase

import (
    "fmt"
    "regexp"
    "strings"
)

type Renderer[T comparable] struct {
    Name    string
    Enabled bool
    Values  []T
}

type Previewer interface {
    Render() string
}

const DefaultTheme = "Dracula"

func NewRenderer[T comparable](name string, values ...T) *Renderer[T] {
    cloned := make([]T, 0, len(values))
    cloned = append(cloned, values...)

    return &Renderer[T]{
        Name:    strings.TrimSpace(name),
        Enabled: true,
        Values:  cloned,
    }
}

func (r *Renderer[T]) Render() string {
    payload := /* json */ `{"theme":"dracula","enabled":true,"count":2}`
    query := /* sql */ `SELECT name, slug FROM themes WHERE enabled = true ORDER BY name`
    upsertQuery := /* sql:postgres */ `INSERT INTO theme_snapshots (payload) VALUES ('{"name":"Dracula"}') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload RETURNING id`
    ddl := /* sql:mysql */ "CREATE TABLE `theme_snapshots` (`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT, PRIMARY KEY (`id`)) ENGINE=InnoDB"
    cacheTable := /* sql:sqlite */ `CREATE TABLE theme_cache (id INTEGER PRIMARY KEY AUTOINCREMENT, slug TEXT NOT NULL UNIQUE) WITHOUT ROWID`
    workflow := /* yaml */ "name: build\non: push\njobs:\n  test:\n    runs-on: ubuntu-latest\n"
    markup := /* html */ `<section data-kind="preview"><strong>ok</strong></section>`
    styles := /* css */ `:root { --accent: #ff79c6; } .card:hover { color: var(--accent); }`
    script := /* js */ `const ready = true; console.log(ready);`
    shell := /* bash */ `printf '%s\n' "$USER"`
    pattern := regexp.MustCompile(`^(?P<section>theme|preview)-(?P<slug>\p{L}+)$`)
    escapedPattern := regexp.MustCompile("^(?P<escapedSection>theme|preview)-(?P<escapedSlug>\\w+)$")
    posixWord := regexp.MustCompilePOSIX(`[[:alpha:]]+`)

    if pattern.MatchString(r.Name) {
        fmt.Println(payload, query, upsertQuery, ddl, cacheTable, workflow, markup, styles, script, shell, escapedPattern, posixWord)
    }

    return DefaultTheme
}
