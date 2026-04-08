interface ThemePreview {
  title: string;
  enabled?: boolean;
}

export class Renderer {
  constructor(private readonly preview: ThemePreview) {}

  render(slug: string): string {
    const tags = ["dracula", slug].filter(Boolean);
    return `${this.preview.title}:${tags.join(",")}`;
  }
}
