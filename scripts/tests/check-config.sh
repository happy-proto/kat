#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
checker="$root/scripts/check-config.py"

printf 'name = "kat"\n' > "$scratch/valid config.toml"
printf 'name: !color red\n' > "$scratch/custom-tag.yaml"
"$checker" "$scratch/valid config.toml" "$scratch/custom-tag.yaml"
uv run --script "$checker" "$scratch/valid config.toml" "$scratch/custom-tag.yaml"
"$checker" --help >/dev/null

printf 'name = [\n' > "$scratch/broken.toml"
printf 'name: [\n' > "$scratch/broken.yml"
if "$checker" "$scratch/broken.toml" "$scratch/broken.yml" 2> "$scratch/errors"; then
    echo 'Invalid TOML / YAML must fail' >&2
    exit 1
fi
grep -q -F "$scratch/broken.toml:" "$scratch/errors"
grep -q -F "$scratch/broken.yml:" "$scratch/errors"

printf '%s\n' '---' 'name: first' '---' 'name: second' > "$scratch/multiple.yaml"
if "$checker" "$scratch/multiple.yaml" 2>/dev/null; then
    echo 'Multiple YAML documents must fail, matching the previous hook' >&2
    exit 1
fi
echo 'Config syntax checks passed'
