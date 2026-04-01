# SQL Dialects

```postgres
INSERT INTO theme_snapshots (payload)
VALUES ('{"name":"Dracula"}')
ON CONFLICT (id) DO UPDATE
SET payload = EXCLUDED.payload
RETURNING id;
```

```mysql
CREATE TABLE `theme_snapshots` (
  `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB;
```

```sqlite
CREATE TABLE theme_cache (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  slug TEXT NOT NULL UNIQUE
) WITHOUT ROWID;
```
