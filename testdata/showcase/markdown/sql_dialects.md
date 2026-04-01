# SQL Dialects Showcase

```postgresql
INSERT INTO theme_snapshots (payload)
VALUES ('{"name":"Dracula"}')
ON CONFLICT (id) DO UPDATE
SET payload = EXCLUDED.payload
RETURNING id;
```

```mariadb
CREATE TABLE `theme_snapshots` (
  `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

```sqlite3
CREATE TABLE theme_cache (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  slug TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
```
