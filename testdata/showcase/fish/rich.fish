#!/usr/bin/env fish
set -gx THEME_NAME "Dracula"
set palette Dracula Nord Gruvbox

function render_theme --argument-names theme_name
    if test -n "$theme_name"
        string match -rq '^Dra.*' -- $theme_name
        contains -- Dracula $theme_name
        emit theme-changed
        functions --query render_theme
        math "1 + 1" >/dev/null
        status is-interactive >/dev/null
        status current-filename >/dev/null
        printf '%s\n' $theme_name

        switch $theme_name
        case Dra*
            type --query fish_prompt
        case '*'
            printf '%s\n' fallback
        end
    end
end

function watch_theme --on-variable THEME_NAME --argument-names new_theme
    string replace -ra 'a' 'o' $new_theme
    printf '%s %s %s %s\n' $argv[1] $status $fish_pid $last_pid
    printf '%s\n' $argv[1..-1]
end

for candidate in $palette
    render_theme $candidate
end
