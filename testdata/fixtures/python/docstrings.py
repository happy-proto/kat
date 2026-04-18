"""Theme preview helpers.

# Preview

This module exercises several common docstring shapes:

- module-level prose
- class-level documentation
- method docstrings with sections

1. Parse configuration
2. Build the preview
3. Render the result

`inline code`

```python
preview = ThemePreview.from_name("kat")
print(preview.render())
```

:mod:`theme_preview`
:class:`ThemePreview`
"""


class ThemePreview:
    """Represent one theme preview entry.

    Attributes:
        name: Human-readable theme name.
        enabled: Whether the preview is active.

    Example:
        >>> preview = ThemePreview("Dracula")
        >>> preview.render()
        'Dracula:enabled'
    """

    def __init__(self, name: str, enabled: bool = True) -> None:
        """Initialize the preview instance.

        Args:
            name: Theme display name.
            enabled: Whether rendering should use the enabled state.

        Raises:
            ValueError: If ``name`` is empty.
        """
        if not name:
            raise ValueError("name must not be empty")

        self.name = name
        self.enabled = enabled

    @classmethod
    def from_name(cls, name: str) -> "ThemePreview":
        """Create a preview from a plain theme name.

        :param name: Source theme name.
        :returns: A new ``ThemePreview`` instance.
        """
        return cls(name)

    @property
    def state_label(self) -> str:
        """Return the state label.

        Returns:
            ``"enabled"`` when the preview is active, otherwise ``"disabled"``.
        """
        return "enabled" if self.enabled else "disabled"

    def render(self) -> str:
        """Render the preview value.

        Notes:
            The return shape is ``"<name>:<state>"``.

        See Also:
            :meth:`from_name`
        """
        return f"{self.name}:{self.state_label}"
