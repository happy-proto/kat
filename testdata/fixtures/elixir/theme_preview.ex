defmodule ThemePreview do
  @default_theme "Dracula"

  def render(name) do
    "Preview #{name}"
  end
end

IO.puts(ThemePreview.render("kat"))
