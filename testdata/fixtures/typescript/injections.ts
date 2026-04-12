const htmlSnippet =
  /* html */
  `<section accent="warm"><span>preview</span></section>`;

const sqlQuery =
  /* sql */
  `SELECT id, slug FROM themes WHERE enabled = true`;

const graphqlQuery =
  /* graphql */
  `query ThemeBySlug($slug: ID!) { theme(slug: $slug) { id } }`;

const cssSnippet =
  /* css */
  `.card { color: red; }`;

export function renderPreview(slug: string): string {
  return `${htmlSnippet}:${sqlQuery}:${graphqlQuery}:${cssSnippet}:${slug}`;
}
