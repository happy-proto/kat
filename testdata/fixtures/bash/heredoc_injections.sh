#!/usr/bin/env bash
set -euo pipefail

python <<'PY'
class Preview:
    def render(self) -> int:
        return 42

print(Preview().render())
PY

node <<'JS'
const payload = { ready: true };
console.log(JSON.stringify(payload));
JS

bash <<'SH'
source .env
printf '%s\n' "$HOME"
SH
