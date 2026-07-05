set quiet := true

# List available recipes
default:
    @just --list

# Run fixes, then other checks
fix-check: eject fmt clippy-fix check

# Run all checks in parallel (fmt + clippy + tests + unused deps + file size + drift)
check:
    parallel -j 0 -- \
        "chronic just fmt-check" \
        "chronic cargo clippy --workspace --all-targets -q -- -D warnings" \
        "chronic cargo test --workspace -q" \
        "chronic just machete" \
        "chronic just check-file-size" \
        "chronic just check-dry" \
        "chronic just outdatty-check"

# Fail if a source changed without its dependents being re-confirmed
outdatty-check:
    outdatty check --format quiet

# Re-confirm dependency groups: record current hashes into outdatty.lock
outdatty-update:
    outdatty update

# Prove `nix build` succeeds in a clean sandbox (no dev shell, no sccache).
# Catches a committed .cargo/config.toml — or any dev-only setting — leaking
# into the package build and breaking downstream flake consumers. Stage first:
# flakes filter source to git-tracked files, and naersk needs the current tree.
nix-build:
    git add -A
    nix build .#default

# Run tests only
test *ARGS:
    cargo test --workspace {{ ARGS }}

# Run clippy only
clippy:
    cargo clippy --workspace --all-targets -q -- -D warnings

# Auto-fix clippy warnings
clippy-fix:
    cargo clippy --fix --workspace --all-targets --allow-dirty --allow-staged -- -D warnings

# Build the project
build *args:
    cargo build --workspace -q {{ args }}

# Run coverage with tarpaulin (also writes target/coverage/lcov.info)
cover:
    cargo tarpaulin --workspace --skip-clean

# Gate complex, undertested functions via CRAP metric. Needs lcov from `just cover`.
crap:
    cargo crap --lcov target/coverage/lcov.info --workspace --exclude 'crates/mindtape-cli/src/main.rs' --threshold 30 --fail-above

# Format code
fmt:
    cargo fmt --all

# Format check (CI-friendly)
fmt-check:
    cargo fmt --all -- --check

# Check for unused dependencies
machete:
    cargo machete

# Count tests across the workspace
count-tests:
    #!/usr/bin/env bash
    cargo test --workspace 2>&1 | grep "test result:" | awk '{sum += $4} END {print sum " tests"}'

# Show the top 20 files by line count
file-sizes:
    #!/usr/bin/env bash
    find . -type f \( -name '*.rs' -o -name '*.md' \) ! -path './target/*' -exec wc -l {} + | sort -rn | head -20

# Eject inline tests from Rust files nearing the linecop limit
eject PCT='90':
    linecop --baseline {{ PCT }} --format paths | ejectest apply . --files-from - --lenient

# Check for oversized files (fails if any exceed limits)
check-file-size:
    linecop

# Copy-paste gate: fail on any duplicated Rust block across the crates
check-dry:
    #!/usr/bin/env bash
    set -eo pipefail
    # jscpd (the Rust port, packaged in flake.nix) fails on any duplicated
    # block of >=70 tokens, naming both spans. Scans all Rust under crates/.
    # Fix a finding by extracting a shared helper — never by shuffling tokens
    # until the detector loses the scent.
    #
    # `--exit-code 1` is spelled with its value: bare `--exit-code` would
    # swallow the following path argument as its optional code.
    jscpd --no-tips -k 70 -f rust --exit-code 1 crates

# Install the mindtape Typst library for local #import (dev symlink)
install-lib:
    #!/usr/bin/env bash
    set -eo pipefail
    VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' lib/typst.toml | head -1)
    TARGET="$HOME/.local/share/typst/packages/local/mindtape"
    mkdir -p "$TARGET"
    # Drop any prior install first: `ln -sfn` into an existing *directory*
    # places the link inside it, leaving a stale copy that tinymist reads.
    rm -rf "$TARGET/$VERSION"
    ln -sfn "$(pwd)/lib" "$TARGET/$VERSION"
    echo "linked lib -> $TARGET/$VERSION (use @local/mindtape:$VERSION)"

# Remove the dev symlink installed by install-lib
uninstall-lib:
    #!/usr/bin/env bash
    VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' lib/typst.toml | head -1)
    TARGET="$HOME/.local/share/typst/packages/local/mindtape/$VERSION"
    [ -L "$TARGET" ] && rm "$TARGET" && echo "removed $TARGET" || echo "no symlink at $TARGET"

# Install git hooks (pre-commit runs `mt check`)
install-hooks:
    #!/usr/bin/env bash
    set -eo pipefail
    hook=.git/hooks/pre-commit
    mkdir -p .git/hooks
    cat > "$hook" <<'HOOK'
    #!/usr/bin/env bash
    set -eo pipefail
    exec direnv exec "$(git rev-parse --show-toplevel)" cargo run -q -- check
    HOOK
    chmod +x "$hook"
    echo "installed $hook"

# Tag a release and push (usage: just release 0.2.0)
release VERSION:
    #!/usr/bin/env bash
    set -eo pipefail
    cargo_version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
    if [ "{{ VERSION }}" != "$cargo_version" ]; then
        echo "error: requested v{{ VERSION }} but Cargo.toml is $cargo_version; bump Cargo.toml first" >&2
        exit 1
    fi
    just check
    just nix-build
    git tag -a "v{{ VERSION }}" -m "v{{ VERSION }}"
    git push origin "v{{ VERSION }}"
