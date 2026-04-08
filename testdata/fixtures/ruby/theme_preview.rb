class ThemePreview
  DEFAULT_THEME = "Dracula"

  def initialize(name = DEFAULT_THEME)
    @name = name
  end

  def render
    "theme:#{@name}"
  end
end

puts ThemePreview.new.render
