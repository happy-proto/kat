#!/usr/bin/env bash
set -euo pipefail

# Resolve the mise-selected compiler before adding the wrapper to PATH.
export KAT_REAL_ZIG
KAT_REAL_ZIG=$(command -v zig)
wrapper_dir=$(cd "$(dirname "$0")/zig-download-retry" && pwd)
export PATH="$wrapper_dir:$PATH"
exec "$@"
