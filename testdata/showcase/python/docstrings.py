"""Theme preview helpers.

This module exercises several common docstring styles.

The public entrypoint is :class:`ThemePreview`.
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
        :returns: A new :class:`ThemePreview` instance.
        :raises ValueError: If the theme name is empty.
        """
        return cls(name)

    @property
    def state_label(self) -> str:
        """Return the state label."""
        return "enabled" if self.enabled else "disabled"

    def render(self) -> str:
        """Render the preview value.

        Parameters
        ----------
        uppercase : bool
            Whether to uppercase the rendered label.

        Returns
        -------
        str
            The preview label.

        See Also
        --------
        from_name
        """
        return f"{self.name}:{self.state_label}"
