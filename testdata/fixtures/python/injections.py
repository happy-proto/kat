import re


# sql:sqlite
query = """
CREATE TABLE theme_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE
) STRICT, WITHOUT ROWID
"""

executed = cursor.execute("SELECT id FROM themes WHERE slug = ?")

slug_pattern = re.compile(r"^(?P<kind>kat)-(?P<slug>[a-z0-9_-]+)\b$")
unicode_pattern = re.compile(r"^(?P<section>theme|preview):(?P<value>\p{L}+)$")
class_pattern = re.compile(r"(?P<word>[[:alpha:]]+)-(?P=word)")
escaped_pattern = re.compile("^(?P<escaped_section>theme|preview):(?P<escaped_value>\\w+)$")
match = re.search(
    r"(?P<section>theme|preview):(?P<value>\w+)",
    "theme:dracula",
)
