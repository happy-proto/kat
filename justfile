test:
    @extra_args=""; \
    if [ "${KAT_AGENT_TEST_LOG_MODE:-}" = "quiet" ]; then \
      extra_args="--status-level fail --final-status-level fail --success-output never --show-progress none"; \
    fi; \
    cargo nextest run --workspace --config-file .config/nextest.toml --cargo-quiet --failure-output final --no-tests pass $extra_args

ghostty-e2e:
    mise exec -- cargo nextest run --workspace --features ghostty-e2e --test ghostty_terminal_e2e --config-file .config/nextest.toml

compact-release:
    @compact_cargo_home=$(mktemp -d); \
    trap 'rm -rf "$compact_cargo_home"' EXIT; \
    export CARGO_HOME="$compact_cargo_home"; \
    export SYSTEM_DEPS_DAV1D_LINK=static; \
    export SYSTEM_DEPS_DAV1D_BUILD_INTERNAL=always; \
    cargo fetch --locked; \
    uv run --script scripts/compact_parser_tables.py --manifest-path Cargo.toml --cargo-home "$CARGO_HOME"; \
    cargo build --release --locked --features compact-parser-tables

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
