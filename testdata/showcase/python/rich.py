from dataclasses import dataclass


@dataclass
class ThemePreview:
    name: str
    enabled: bool = True

    def render(self) -> str:
        return f"{self.name}:{self.enabled}"


if __name__ == "__main__":
    preview = ThemePreview("kat")
    print(preview.render())
