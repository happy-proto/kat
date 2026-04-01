+++
title = "Kat Theme Preview"
theme = "Dracula"
enabled = true
mode = "showcase"
+++

# Kat Theme Preview

This Markdown file is intentionally kept in the showcase set so embedded code
blocks are easy to inspect once language injection is wired up.

Inline syntax should also be easy to inspect: `kat --language markdown`,
**bold text**, *italic text*, ~~strikethrough~~, and
[a link with `inline code`](https://example.com/kat?mode=showcase).

Setext headings should also stay readable
-----------------------------------------

- Bullet items can include `inline code`, **strong emphasis**, and links like
  [README](../../README.md).
- Mixed inline content such as `fn render() -> String` inside prose is useful
  for checking whether code spans are visually distinct from surrounding text.

1. Ordered lists should keep their markers distinct.
2. Nested inline syntax should still work inside list items.
3. Block-level structure should not flatten the inline styling.

- [x] Task-list items should show completed state clearly.
- [ ] Pending task items should still read cleanly.

> Block quotes should keep their marker distinct while still rendering
> `inline code`, **strong emphasis**, and links like
> [the docs](https://example.com/docs).

Inline HTML such as <kbd>Cmd</kbd>+<kbd>K</kbd> and <mark>highlighted text</mark>
should also be easy to inspect.

Autolinks should be differentiated too:
- <https://example.com/autolink>
- <hello@example.com>

Images and link labels should remain legible:
![Preview diagram](https://example.com/assets/preview.png "Image Title")

Reference-style links should also stay readable:
[guide][docs-ref], [shortcut], [full reference][docs-ref], and
![Reference image][img-ref]

Autolinks and email links should remain distinct too:
<https://example.com/auto>
<dracula@example.com>

Inline HTML should remain intentional: <kbd>Shift</kbd>+<kbd>Enter</kbd>,
<mark>notice</mark>, and <sub>subtext</sub>.

Nested structures should stay readable:
> Quote level one
>
> - nested bullet
> - `inline code`

Secondary Showcase Heading
--------------------------

| Feature | Example | Expected focus |
| :------ | :------ | -------------: |
| Link | [README](../../README.md) | URI + label |
| Code span | `let rendered = true` | literal contrast |
| Strong | **Dracula** | emphasis strength |
| Strike | ~~legacy~~ | removed content |

---

<section class="callout">
  <p>HTML blocks should route through the HTML grammar.</p>
  <script>
    const payload = { mode: "markdown-showcase", nested: true };
    console.log(payload);
  </script>
</section>

```rust
fn preview(theme: &str) -> String {
    format!("theme={theme}")
}
```

```bash
#!/usr/bin/env bash
set -euo pipefail
echo "preview"
```

```python
class Preview:
    def render(self) -> int:
        return 42

print(Preview().render())
```

```json
{
  "theme": "Dracula",
  "mode": "showcase"
}
```

```
plain fenced block without syntax should use markup-code-block styling
```

[docs-ref]: https://example.com/reference/docs "Reference Title"
[shortcut]: https://example.com/reference/shortcut
[img-ref]: https://example.com/assets/reference-image.png "Reference Image"
