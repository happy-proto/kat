#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bin_path="${KAT_PERF_BIN:-$repo_root/target/release/kat}"
iterations="${KAT_PERF_ITERATIONS:-3}"
manifest_path="$repo_root/testdata/perf/baseline.txt"

if [[ ! -x "$bin_path" ]]; then
  echo "missing benchmark binary: $bin_path" >&2
  exit 1
fi

resolve_paths() {
  if [[ "$#" -gt 0 ]]; then
    printf '%s\n' "$@"
    return
  fi

  grep -v '^[[:space:]]*#' "$manifest_path" | sed '/^[[:space:]]*$/d'
}

run_case() {
  local relative_path="$1"
  local absolute_path="$repo_root/$relative_path"

  if [[ ! -f "$absolute_path" ]]; then
    echo "missing benchmark input: $relative_path" >&2
    exit 1
  fi

  echo "==> $relative_path <=="
  local totals=()

  local run
  for run in $(seq 1 "$iterations"); do
    local timing_line
    timing_line="$("$bin_path" --paging=never --debug-timing "$absolute_path" 2>&1 >/dev/null)"
    echo "run=$run $timing_line"

    local total_ms
    total_ms="$(printf '%s\n' "$timing_line" | sed -n 's/.* total=\([0-9.]*\)ms/\1/p')"
    if [[ -n "$total_ms" ]]; then
      totals+=("$total_ms")
    fi
  done

  if [[ "${#totals[@]}" -gt 0 ]]; then
    local average_total
    average_total="$(printf '%s\n' "${totals[@]}" | awk '{sum += $1} END {printf "%.3f", sum / NR}')"
    echo "avg_total=${average_total}ms"
  fi
}

while IFS= read -r relative_path; do
  run_case "$relative_path"
done < <(resolve_paths "$@")
