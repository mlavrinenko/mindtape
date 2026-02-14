# MindTape development recipes

# Run all checks (clippy + tests)
check:
    cargo clippy --all-targets -q
    cargo test -q

# Run tests only
test *ARGS:
    cargo test -q {{ARGS}}

# Run clippy only
clippy:
    cargo clippy --all-targets -q

# Build the project
build:
    cargo build -q

# Run coverage with tarpaulin
cover:
    cargo tarpaulin

# Run shell integration tests
itest: build
    cd itest && PATH="../target/debug:$$PATH" bash basic.sh

# Run everything (clippy + tests + itest)
all: check itest

# Format code
fmt:
    cargo fmt

# Format check (CI-friendly)
fmt-check:
    cargo fmt -- --check
