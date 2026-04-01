CREATE TABLE theme_snapshots (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  slug TEXT NOT NULL UNIQUE
) STRICT, WITHOUT ROWID;

BEGIN IMMEDIATE;
ATTACH DATABASE 'theme-cache.db' AS cache;

INSERT OR REPLACE INTO theme_snapshots (id, slug)
VALUES (1, 'dracula');

VACUUM;
REINDEX;

INSERT OR IGNORE INTO theme_snapshots (id, slug)
VALUES (2, 'kat');
