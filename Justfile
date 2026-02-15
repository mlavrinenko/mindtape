# MindTape development recipes

# Run all checks (clippy + tests)
check:
    cargo clippy --workspace --all-targets -q
    cargo test --workspace -q

# Run tests only
test *ARGS:
    cargo test --workspace {{ ARGS }}

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
    cd itest && PATH="../target/debug:$PATH" sh basic.sh

# Run everything (clippy + tests + itest)
all: check itest

# Format code
fmt:
    cargo fmt --all

# Format check (CI-friendly)
fmt-check:
    cargo fmt --all -- --check

# Count tests across workspace
count-tests:
    #!/usr/bin/env bash
    cargo test --workspace 2>&1 | grep "test result:" | awk '{sum += $4} END {print sum " tests"}'

# Show top 20 files by line count
file-sizes:
    #!/usr/bin/env bash
    find . -type f \( -name '*.rs' -o -name '*.md' \) ! -path './target/*' -exec wc -l {} + | sort -rn | head -20
