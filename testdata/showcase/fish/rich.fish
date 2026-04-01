#!/usr/bin/env fish
set -gx THEME_NAME "Dracula"
set palette Dracula Nord Gruvbox

function render_theme --argument-names theme_name
    if test -n "$theme_name"
        string match -rq '^Dra.*' -- $theme_name
        contains -- Dracula $theme_name
        math "1 + 1" >/dev/null
        status is-interactive >/dev/null
        printf '%s\n' $theme_name
    end
end

for candidate in $palette
    render_theme $candidate
end
