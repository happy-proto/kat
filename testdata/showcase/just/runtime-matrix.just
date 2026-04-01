set shell := ["zsh", "-cu"]

cwd := `source .env && printf '%s' "$PWD"`

default:
    source .env
    printf '%s\n' "$cwd"

bun-report:
    #!/usr/bin/env bun
    const report = { tool: "kat", ready: true };
    console.log(JSON.stringify(report));

fish-like:
    #!/usr/bin/env fish
    source .env
    printf '%s\n' "$HOME"
