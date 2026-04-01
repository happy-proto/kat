from dataclasses import dataclass


@classmethod
def build_default(cls) -> "ThemePreview":
    return ThemePreview(name="Dracula")


@dataclass
class ThemePreview:
    name: str
    enabled: bool = True

    def __init__(self, name: str, enabled: bool = True) -> None:
        self.name = name
        self.enabled = enabled

    @property
    def slug(self) -> str:
        return self.name.lower()

    def render(self) -> str:
        if isinstance(self.name, str):
            return f"{self.name}:{self.enabled}"
        return "invalid"


preview = ThemePreview("kat")
print(preview.render())
