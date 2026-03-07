# MindTape development recipes

# Run all checks (clippy + tests + file size)
check:
    cargo clippy --workspace --all-targets -q
    cargo test --workspace -q
    just check-file-size

# Run tests only
test *ARGS:
    cargo test --workspace {{ ARGS }}

# Run clippy only
clippy:
    cargo clippy --workspace --all-targets -q

# Auto-fix clippy warnings
clippy-fix:
    cargo clippy --fix --workspace --all-targets

# Build the project
build:
    cargo build --workspace -q

# Run coverage with tarpaulin
cover:
    cargo tarpaulin --workspace

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

# Check for oversized files (fails if any exceed limits)
check-file-size:
    #!/usr/bin/env bash
    set -euo pipefail

    # Thresholds
    RUST_LIMIT=500
    MARKDOWN_LIMIT=200

    # Exception list (relative to project root)
    # sqlite.rs is schema-heavy, hard to split meaningfully
    # watcher/tests.rs has many integration-style scenarios
    EXCEPTIONS=(
        "crates/mindtape-store/src/store/sqlite.rs"
        "src/watcher/tests.rs"
    )

    failed=0

    # Check Rust files
    while IFS= read -r file; do
        lines=$(wc -l < "$file")
        # Check if file is in exceptions
        skip=0
        for exception in "${EXCEPTIONS[@]}"; do
            if [[ "$file" == "./$exception" ]]; then
                skip=1
                break
            fi
        done

        if [[ $skip -eq 0 && $lines -gt $RUST_LIMIT ]]; then
            echo "❌ $file: $lines lines (limit: $RUST_LIMIT)"
            failed=1
        fi
    done < <(find . -type f -name '*.rs' ! -path './target/*')

    # Check Markdown files
    while IFS= read -r file; do
        lines=$(wc -l < "$file")
        if [[ $lines -gt $MARKDOWN_LIMIT ]]; then
            echo "❌ $file: $lines lines (limit: $MARKDOWN_LIMIT)"
            failed=1
        fi
    done < <(find . -type f -name '*.md' ! -path './target/*')

    if [[ $failed -eq 1 ]]; then
        echo ""
        echo "Some files exceed size limits. Consider refactoring."
        exit 1
    else
        echo "✓ All files within size limits"
    fi

# Validate that nix/install-example.nix parses correctly
check-nix-example:
    nix-instantiate --parse nix/install-example.nix > /dev/null

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
