module theme;

import std.stdio;

struct ThemePreview {
    string name = "Dracula";
}

void main() {
    writeln(ThemePreview.init.name);
}
