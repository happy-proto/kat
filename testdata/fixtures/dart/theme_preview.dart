class ThemePreview {
  static const defaultTheme = 'Dracula';

  String render(String name) => 'Preview $name';
}

void main() {
  print(ThemePreview().render('kat'));
}
