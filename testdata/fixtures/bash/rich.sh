#!/usr/bin/env bash
set -euo pipefail

name="kat"
escaped=$'line\nsecond'
declare -ga palettes=(dracula nord)
read -r theme_line <<< "$name"

if [[ "$name" =~ ^kat.* ]]; then
  printf '%s\n' "${palettes[0]}"
  unset theme_line
  printf '%s %s %s\n' "$name" "$#" "$@"
fi
