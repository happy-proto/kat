# Rainbow bracket readability

Each pair keeps its color across lines; each code block starts a new palette.

```rust
fn collect<T>(values: Vec<Option<T>>) {
    let groups = [Some((1, 2)), Some((3, 4))];
    // Text such as ([{}]) keeps its comment color.
    let literal = "([{}])";
}
```

```json
{
  "pipeline": [
    { "retry": { "delays": [1, 2, 4] } },
    { "deploy": { "regions": ["sg", "id"] } }
  ]
}
```

```python
result = aggregate([
    transform((key, value))
    for key, value in items
])
```
