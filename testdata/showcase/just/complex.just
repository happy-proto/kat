set shell := ["bash", "-cu"]
set dotenv-load := true

[group('build')]
[confirm("Deploy the preview environment?")]
deploy profile="staging" target="x86_64-unknown-linux-gnu":
    @echo "Preparing {{profile}} deploy for {{target}}"
    cargo build --target {{target}}
    if [ -f .env ]; then
      source .env
    fi
    ./scripts/deploy.sh "{{profile}}" "{{target}}"

lint:
    cargo fmt --check
    cargo clippy --all-targets --all-features

test *args:
    cargo nextest run {{args}}

regex-demo:
    if "release-2026-03-26" =~ 'release-\d{4}-\d{2}-\d{2}' {
      echo "matched"
    } else {
      echo "no match"
    }
