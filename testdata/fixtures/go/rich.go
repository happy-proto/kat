//go:build linux

package showcase

import (
    "fmt"
    "regexp"
)

type Renderer[T comparable] struct {
    Name    string
    Enabled bool
}

type Previewer interface {
    Render() string
}

const DefaultTheme = "Dracula"

func NewRenderer[T comparable](name string) *Renderer[T] {
    items := make([]T, 0)
    _ = append(items)

    return &Renderer[T]{
        Name:    name,
        Enabled: true,
    }
}

func (r *Renderer[T]) Render() string {
    payload := /* json */ `{"theme":"dracula","enabled":true}`
    query := /* sql */ `SELECT name, slug FROM themes WHERE enabled = true ORDER BY name`
    upsertQuery := /* sql:postgres */ `INSERT INTO theme_snapshots (payload) VALUES ('{"name":"Dracula"}') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload RETURNING id`
    ddl := /* sql:mysql */ "CREATE TABLE `theme_snapshots` (`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT, PRIMARY KEY (`id`)) ENGINE=InnoDB"
    cacheTable := /* sql:sqlite */ `CREATE TABLE theme_cache (id INTEGER PRIMARY KEY AUTOINCREMENT, slug TEXT NOT NULL UNIQUE) WITHOUT ROWID`
    workflow := /* yaml */ "name: build\non: push\n"
    markup := /* html */ `<section data-kind="preview">ok</section>`
    styles := /* css */ `:root { --accent: #ff79c6; }`
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
