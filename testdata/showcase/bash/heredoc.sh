#!/usr/bin/env bash
set -euo pipefail

name="kat"

cat <<EOF
Previewing $name output.
Use this file to check heredoc styling once shell injections become richer.
EOF

printf '%s\n' "$name"
