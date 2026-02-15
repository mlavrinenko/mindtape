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

# Install mindtape library globally for Typst (via symlink)
install-lib:
    #!/usr/bin/env bash
    set -euo pipefail
    TARGET_DIR="$HOME/.local/share/typst/packages/local/mindtape"
    mkdir -p "$TARGET_DIR"
    ln -sfn "$(pwd)/lib" "$TARGET_DIR/0.1.0"
    echo "✓ Installed mindtape library to $TARGET_DIR/0.1.0"
    echo "  Use in Typst files: #import \"@local/mindtape:0.1.0\": due, id, tag"

# Uninstall global mindtape library
uninstall-lib:
    #!/usr/bin/env bash
    TARGET="$HOME/.local/share/typst/packages/local/mindtape/0.1.0"
    if [ -L "$TARGET" ]; then
        rm "$TARGET"
        echo "✓ Removed $TARGET"
    else
        echo "! No symlink found at $TARGET"
    fi
