const themeQuery = graphql`
  fragment ThemeCard on Theme {
    id
    name
    palette
  }

  query ThemeDashboard($slug: ID!, $withPalette: Boolean = true) {
    theme(slug: $slug) {
      ...ThemeCard
      palette @include(if: $withPalette)
    }
  }
`;

const inlineMutation =
  /* gql */
  "mutation RenameTheme($id: ID!, $name: String!) { renameTheme(id: $id, name: $name) { id name } }";

export function buildGraphqlPayload(slug) {
  return { slug, themeQuery, inlineMutation };
}
