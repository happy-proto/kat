#!/usr/bin/env zsh
emulate -L zsh
autoload -Uz colors
setopt extended_glob
unsetopt nomatch

local theme_name=${1:-Dracula}
typeset -g theme_slug=${theme_name:l}
typeset -a palette=(Dracula Nord Gruvbox)
source ~/.zshrc

for candidate in $palette; do
  if [[ $candidate == (#i)(dracula|nord) ]]; then
    print -r -- "$candidate:$theme_slug"
  fi
done
