#!/usr/bin/env bash

set -euo pipefail

real_linker=${KAT_REAL_LINKER:?missing KAT_REAL_LINKER}
link_log=${KAT_LINK_LOG:?missing KAT_LINK_LOG}

mkdir -p "$(dirname "${link_log}")"

output_path="(unknown)"
next_is_output=0
for arg in "$@"; do
  if [[ "${next_is_output}" == "1" ]]; then
    output_path="${arg}"
    next_is_output=0
    continue
  fi

  if [[ "${arg}" == "-o" ]]; then
    next_is_output=1
  fi
done

start_ms=$(perl -MTime::HiRes=time -e 'printf "%.0f\n", time() * 1000')
set +e
"${real_linker}" "$@"
status=$?
set -e
end_ms=$(perl -MTime::HiRes=time -e 'printf "%.0f\n", time() * 1000')
duration_ms=$((end_ms - start_ms))

printf '%s\t%s\t%s\t%s\n' \
  "$(date -u +%FT%TZ)" \
  "${duration_ms}" \
  "${status}" \
  "${output_path}" >> "${link_log}"

exit "${status}"
