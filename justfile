test:
    cargo nextest run --config-file .config/nextest.toml --cargo-quiet --failure-output final --no-tests pass

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
    pnpm install
    cargo install --path .
