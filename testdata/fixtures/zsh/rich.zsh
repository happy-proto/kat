#!/usr/bin/env zsh
emulate -L zsh
autoload -Uz colors
setopt EXTENDED_GLOB
unsetopt nomatch

local theme_name=${1:-Dracula}
typeset -g theme_slug=${theme_name:l}
source ~/.zshrc

if [[ $theme_name == (#i)(dracula|nord) ]]; then
  print -r -- "$theme_name:$theme_slug"
fi
