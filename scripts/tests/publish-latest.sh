#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/bin" "$tmp/assets"
touch "$tmp/assets/kat-test.tar.xz"
export KAT_RELEASE_TEST_DIR="$tmp"
cat > "$tmp/bin/git" <<'MOCK'
#!/usr/bin/env bash
set -eu
printf 'git %s\n' "$*" >> "$KAT_RELEASE_TEST_DIR/mutations"
MOCK
cat > "$tmp/bin/gh" <<'MOCK'
#!/usr/bin/env bash
set -eu
if [[ "$1" == api ]]; then
  [[ "$2" == repos/test/kat/git/ref/heads/master ]]
  printf '%s\n' "$TEST_DEFAULT_SHA"
  exit "${TEST_API_STATUS:-0}"
fi
if [[ "$1 $2" == 'release view' ]]; then
  if [[ "$*" == *'--json assets'* ]]; then
    printf 'kat-test.tar.xz\n'
  fi
else
  printf 'gh %s\n' "$*" >> "$KAT_RELEASE_TEST_DIR/mutations"
fi
MOCK
chmod +x "$tmp/bin/git" "$tmp/bin/gh"
export PATH="$tmp/bin:$PATH"
export RELEASE_SHA=1111111111111111111111111111111111111111
export RELEASE_ASSET_DIR="$tmp/assets" RUNNER_TEMP="$tmp"
export GITHUB_REPOSITORY=test/kat GITHUB_SERVER_URL=https://github.com

run_case() {
  local default_sha=$1 api_status=$2 expected_status=$3
  : > "$tmp/mutations"
  local status=0
  TEST_DEFAULT_SHA="$default_sha" TEST_API_STATUS="$api_status" \
    bash "$root/scripts/publish-latest.sh" > "$tmp/stdout" 2> "$tmp/stderr" || status=$?
  [[ "$status" == "$expected_status" ]] || { cat "$tmp/stderr"; echo "unexpected status: $status"; exit 1; }
}

run_case 2222222222222222222222222222222222222222 0 0
[[ ! -s "$tmp/mutations" ]] || { echo 'Outdated release changed tags or release assets'; exit 1; }

run_case "$RELEASE_SHA" 0 0
grep -Fq "git tag -f latest $RELEASE_SHA" "$tmp/mutations"
grep -Fq 'git push --force origin refs/tags/latest' "$tmp/mutations"
grep -Fq 'gh release upload latest' "$tmp/mutations"

run_case "$RELEASE_SHA" 42 42
[[ ! -s "$tmp/mutations" ]] || { echo 'Release changed tags or assets after a failed freshness lookup'; exit 1; }

echo 'Latest release freshness tests passed'
