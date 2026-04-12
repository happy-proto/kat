test:
    @extra_args=""; \
    if [ "${KAT_AGENT_TEST_LOG_MODE:-}" = "quiet" ]; then \
      extra_args="--status-level fail --final-status-level fail --success-output never --show-progress none"; \
    fi; \
    cargo nextest run --config-file .config/nextest.toml --cargo-quiet --failure-output final --no-tests pass $extra_args

perf iterations="3":
    @cargo build --release --quiet
    @KAT_PERF_ITERATIONS="{{iterations}}" ./scripts/perf-baseline.sh

perf-file path iterations="3":
    @cargo build --release --quiet
    @KAT_PERF_ITERATIONS="{{iterations}}" ./scripts/perf-baseline.sh "{{path}}"

showcase path="":
    @cargo build --quiet
    @bin=target/debug/kat; \
    divider='================================================================'; \
    if [ -n "{{path}}" ]; then \
      printf '\n%s\nSHOWCASE: %s\n%s\n\n' "$divider" "{{path}}" "$divider"; \
      "$bin" "{{path}}"; \
      printf '\n'; \
    else \
      find testdata/showcase -type f | sort | while read -r file; do \
        printf '\n%s\nSHOWCASE: %s\n%s\n\n' "$divider" "$file" "$divider"; \
        "$bin" "$file"; \
        printf '\n'; \
      done; \
    fi

install:
    cargo install --path .
