---
layout: showcase
theme: "solarized"
published: true
tags:
  - markdown
  - yaml
---

# Preview

This is *emphasis*, **strong**, ~~strikethrough~~, and [docs](https://example.com).
Reference links should work too: [guide][docs-ref], [shortcut], and ![diagram][img-ref].
Autolinks should stay distinct: <https://example.com/autolink> and <hello@example.com>.
Inline HTML such as <kbd>Cmd</kbd> and <mark>note</mark> should still route cleanly.

Secondary Heading
-----------------

- [x] done item
- [ ] pending item
1. ordered item
2. second ordered item

> quoted text with `inline code`
>
> - nested quoted list item

| Column | Value |
| :----- | ----: |
| Link | [site](https://example.com) |
| Code | `let value = 42` |

```rust
fn answer() -> i32 {
    return 42;
}
```

```json
{
  "mode": "showcase"
}
```

```
plain fence without syntax
```

[docs-ref]: https://example.com/docs "Docs Title"
[shortcut]: https://example.com/shortcut
[img-ref]: https://example.com/assets/diagram.png "Diagram Title"
