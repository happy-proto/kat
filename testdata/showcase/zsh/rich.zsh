#!/usr/bin/env zsh
emulate -L zsh
autoload -Uz colors
setopt extended_glob
unsetopt nomatch

local theme_name=${1:-Dracula}
typeset -g theme_slug=${theme_name:l}
typeset -a palette=(Dracula Nord Gruvbox)
read -r theme_line <<< "$theme_name"
source ~/.zshrc

for candidate in $palette; do
  if [[ $candidate == (#i)(dracula|nord) ]]; then
    print -r -- $palette[1]
    print -r -- $palette[(I)Dr*]
    print -r -- "$candidate:$theme_slug"
  fi
done
