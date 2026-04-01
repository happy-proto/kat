const detailQuery = sql`
  WITH ranked AS (
    SELECT id, name, enabled
    FROM themes
    WHERE enabled = true
  )
  SELECT *
  FROM ranked
  ORDER BY name ASC
`;

const fallbackQuery =
  /* sql */
  `
    SELECT slug
    FROM themes
    WHERE kind = 'preview'
  `;

const postgresQuery =
  /* sql:postgres */
  `
    INSERT INTO theme_snapshots (payload)
    VALUES ('{"name":"Dracula"}')
    ON CONFLICT (id) DO UPDATE
    SET payload = EXCLUDED.payload
    RETURNING id
  `;

const mysqlQuery =
  /* sql:mysql */
  "CREATE TABLE `theme_snapshots` (`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT, PRIMARY KEY (`id`)) ENGINE=InnoDB";

const apiQuery = db.query("SELECT id FROM themes WHERE slug = $1 RETURNING id");
const sqliteOne = db.get("SELECT id FROM themes WHERE slug = ?");
const sqliteMany = db.all(`SELECT id FROM themes WHERE enabled = true`);
const sqliteWrite = db.run("INSERT INTO theme_cache (slug) VALUES (?)");
const gqlMemberQuery = client.gql`
  fragment ThemeCard on Theme {
    id
    name
  }
`;

const slugPattern = /^(?<kind>theme)-(?<slug>[a-z0-9_-]+)(?:\/(?<tab>preview|settings))?$/gi;
const routePattern = new RegExp(
  String.raw`^(?<section>theme|preview):(?<value>\p{L}[\w-]+)$`,
  "iu"
);
const escapedRoutePattern = new RegExp(
  "^(?<escapedSection>theme|preview):(?<escapedValue>\\w+)$",
  "iu"
);
const unsupportedPythonNamedGroup = new RegExp(`(?P<legacy>theme)`, "u");
const unsupportedInlineFlags = new RegExp(`(?i:theme)`, "u");

export function buildThemeRoute(value) {
  return {
    detailQuery,
    fallbackQuery,
    postgresQuery,
    mysqlQuery,
    apiQuery,
    sqliteOne,
    sqliteMany,
    sqliteWrite,
    gqlMemberQuery,
    match: slugPattern.exec(value),
    route: routePattern.exec(value),
    escapedRoute: escapedRoutePattern.exec(value),
    unsupported: unsupportedPythonNamedGroup.exec(value),
    unsupportedInlineFlags: unsupportedInlineFlags.exec(value),
  };
}
