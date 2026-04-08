using System;

namespace Kat.Rendering;

public sealed class ThemePreview
{
    private const string DefaultTheme = "Dracula";

    public ThemePreview(string name)
    {
        Name = string.IsNullOrWhiteSpace(name) ? DefaultTheme : name;
    }

    public string Name { get; }

    [Obsolete("Use Render instead.")]
    public string Render()
    {
        var title = $"{Name} Preview";
        return $"{title}: enabled={true}";
    }
}
