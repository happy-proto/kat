package kat.preview;

public final class ThemePreview {
  private static final String DEFAULT_THEME = "Dracula";

  public static void main(String[] args) {
    System.out.println(render(DEFAULT_THEME));
  }

  static String render(String name) {
    return "theme:" + name;
  }
}
