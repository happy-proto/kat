#!/usr/bin/env bash
set -euo pipefail

psql <<'SQL'
INSERT INTO theme_snapshots (payload)
VALUES ('{"name":"Dracula"}')
ON CONFLICT (id) DO UPDATE
SET payload = EXCLUDED.payload
RETURNING id;
SQL

sqlite3 <<'SQLITE'
CREATE TABLE theme_snapshots (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  slug TEXT NOT NULL UNIQUE
) STRICT, WITHOUT ROWID;
SQLITE

mysql <<'MYSQL'
CREATE TABLE `theme_snapshots` (
  `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB;
MYSQL
