# GraphQL Fence

```graphql
query ThemeBySlug($slug: ID!) {
  theme(slug: $slug) {
    id
    name
  }
}
```

```gql
fragment ThemeFields on Theme {
  id
  name
}
```
