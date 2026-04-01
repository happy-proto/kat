set shell := ["zsh", "-cu"]

cwd := `source .env && printf '%s' "$PWD"`

default:
    source .env
    printf '%s\n' "$cwd"

fish-like:
    #!/usr/bin/env fish
    source .env
    printf '%s\n' "$HOME"
