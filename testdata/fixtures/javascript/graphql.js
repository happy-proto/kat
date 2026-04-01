const themeQuery = gql`
  fragment ThemeFields on Theme {
    id
    name
  }

  query ThemeBySlug($slug: ID!) {
    theme(slug: $slug) {
      ...ThemeFields
    }
  }
`;

const dashboardQuery =
  /* graphql */
  "query ThemeDashboard($limit: Int = 20) { themes(limit: $limit) { id name } }";

export function loadTheme(slug) {
  return { slug, themeQuery, dashboardQuery };
}
