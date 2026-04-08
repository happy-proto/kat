local ThemePreview = {}
ThemePreview.__index = ThemePreview

function ThemePreview.new(name)
  return setmetatable({ name = name or "Dracula" }, ThemePreview)
end

function ThemePreview:render()
  return "theme:" .. self.name
end

local preview = ThemePreview.new("Dracula")
print(preview:render())
