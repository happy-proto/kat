#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/bin"
export KAT_RETRY_TEST_DIR="$tmp"
cat > "$tmp/bin/zig" <<'MOCK'
#!/usr/bin/env bash
set -eu
printf '%s\n' "$*" >> "$KAT_RETRY_TEST_DIR/calls"
printf 'zig stdout\n'
count=$(wc -l < "$KAT_RETRY_TEST_DIR/calls")
if [[ "$count" -le "$FAILURES" ]]; then
  printf '%s\n' "$DIAGNOSTIC" >&2
  exit 42
fi
MOCK
cat > "$tmp/bin/sleep" <<'MOCK'
#!/usr/bin/env bash
printf '%s\n' "$1" >> "$KAT_RETRY_TEST_DIR/sleeps"
MOCK
chmod +x "$tmp/bin/zig" "$tmp/bin/sleep"
export PATH="$tmp/bin:$PATH"

run_case() {
  local failures=$1 diagnostic=$2 expected_status=$3 expected_calls=$4 expected_sleeps=$5
  shift 5
  : > "$tmp/calls"
  : > "$tmp/sleeps"
  local status=0
  FAILURES="$failures" DIAGNOSTIC="$diagnostic" \
    "$root/scripts/with-zig-download-retry.sh" zig "$@" \
    > "$tmp/stdout" 2> "$tmp/stderr" || status=$?
  [[ "$status" == "$expected_status" ]] || { cat "$tmp/stderr"; echo "unexpected status: $status"; exit 1; }
  [[ $(wc -l < "$tmp/calls") -eq "$expected_calls" ]]
  [[ $(cat "$tmp/sleeps") == "$expected_sleeps" ]]
  [[ $(wc -l < "$tmp/stdout") -eq "$expected_calls" ]]
  [[ $(head -n 1 "$tmp/calls") == "$*" ]]
  if [[ "$failures" -gt 0 ]]; then
    grep -Fq -- "$diagnostic" "$tmp/stderr"
  fi
}

network='pkg/translate-c/build.zig.zon:15:20: error: unable to connect to server: NameServerFailure'
run_case 2 "$network" 0 3 $'2\n4' build -Doptimize=Debug
run_case 99 "$network" 42 4 $'2\n4\n8' build
run_case 0 '' 0 1 '' build
run_case 99 'src/main.zig:1:1: error: expected type u32, found bool' 42 1 '' build
run_case 99 'pkg/foo/build.zig.zon:1:1: error: hash mismatch' 42 1 '' build
run_case 99 "$network" 42 1 '' version
run_case 99 'src/main.zig:1:1: error: NameServerFailure' 42 1 '' build
run_case 1 'pkg/foo/build.zig.zon:1:1: error: unable to download file: ConnectionTimedOut' 0 2 '2' build 'argument with spaces'
http_closing='pkg/translate_c/build.zig.zon:12:20: error: invalid HTTP response: HttpConnectionClosing'
run_case 2 "$http_closing" 0 3 $'2\n4' build
run_case 99 "$http_closing" 42 4 $'2\n4\n8' build
run_case 99 "$http_closing" 42 1 '' version
run_case 99 'src/main.zig:1:1: error: invalid HTTP response: HttpConnectionClosing' 42 1 '' build
run_case 99 'src/main.zig:1:1: error: invalid HTTP response: InvalidContentLength' 42 1 '' build
echo 'Zig download retry tests passed'
