#!/usr/bin/env bash
set -euo pipefail

name="kat"
escaped=$'line\nsecond'

if [[ "$name" =~ ^kat.* ]]; then
  printf '%s %s %s\n' "$name" "$#" "$@"
fi
