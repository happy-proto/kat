object ThemePreview {
  val DefaultTheme = "Dracula"

  def render(name: String): String =
    s"Preview: $name"
}

println(ThemePreview.render("kat"))
