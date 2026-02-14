# MindTape development recipes

# Run all checks (clippy + tests)
check:
    cargo clippy --workspace --all-targets -q
    cargo test --workspace -q

# Run tests only
test *ARGS:
    cargo test --workspace {{ARGS}}

# Run clippy only
clippy:
    cargo clippy --workspace --all-targets -q

# Build the project
build:
    cargo build --workspace -q

# Run coverage with tarpaulin
cover:
    cargo tarpaulin --workspace

# Run shell integration tests
itest: build
    cd itest && PATH="../target/debug:$$PATH" bash basic.sh

# Run everything (clippy + tests + itest)
all: check itest

# Format code
fmt:
    cargo fmt --all

# Format check (CI-friendly)
fmt-check:
    cargo fmt --all -- --check
