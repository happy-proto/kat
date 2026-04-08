<?php

namespace Kat;

final class ThemePreview
{
    public const DEFAULT_THEME = "Dracula";

    public function render(string $name): string
    {
        return "<section>{$name}</section>";
    }
}

echo (new ThemePreview())->render("Preview");
