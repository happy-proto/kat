# GraphQL Fence

```graphql
query ThemeDashboard($slug: ID!, $withPalette: Boolean = true) {
  theme(slug: $slug) {
    id
    name
    palette @include(if: $withPalette)
  }
}
```

```gql
fragment ThemeFields on Theme {
  id
  name
  tags
}
```
