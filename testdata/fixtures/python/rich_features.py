from dataclasses import dataclass


@dataclass
class ThemePreview:
    name: str
    enabled: bool = True

    def render(self) -> str:
        return f"{self.name}:{self.enabled}"


    preview = ThemePreview("kat")
print(preview.render())
