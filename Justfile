set quiet := true

# List available recipes
default:
    @just --list

# Run fixes, then other checks
fix-check: eject fmt clippy-fix check

# Run all checks in parallel (fmt + clippy + tests + unused deps + file size +
# drift). Each arm is `just mmz <subgate>` — mmz wraps it, so a no-op `just
# check` re-run skips every arm whose declared inputs are unchanged, and `mt
# flip` into a terminal status asserts them all fresh via
# `mmz --is-fresh --tag gate` without running any. Per-rule scopes and the
# `gate` tag live in .mmz/config.yaml. There is no `chronic` here: a cache hit
# prints mmz's own `on_hit` note to stderr (quiet by construction), a miss
# streams the underlying recipe's stdout/stderr live, and a failure propagates
# its non-zero exit out of `parallel` unchanged.
check:
    parallel -j 0 -- \
        "just mmz fmt-check" \
        "just mmz clippy" \
        "just mmz test" \
        "just mmz machete" \
        "just mmz check-file-size" \
        "just mmz check-dry" \
        "just mmz outdatty-check" \
        "just mmz check-examples" \
        "just mmz docs-check" \
        "just mmz fmt-fixture-check"

# Wrap a just recipe in mmz: `just mmz fmt-check` runs `mmz just fmt-check`. The
# double hop makes mmz the cache identity (rule name `just <recipe>` in
# .mmz/config.yaml), uniform across every sub-gate of `just check`. `just test
# <filter>` (a developer query, no `mmz`) bypasses this recipe and stays plain
# `cargo test` with ARGS — so the `just test` cache record reflects only the
# bare, full-suite invocation that `just check` runs, never a filtered subset.
mmz *ARGS:
    mmz just {{ ARGS }}

# Validate every example project against itself, so a fixture some other tool
# runs `mt` against can never rot: learn-piano feeds the docs generator and the
# skill's triage evals, minimal feeds the skill's trap evals. Gating them is
# what earns the right to depend on them — the eval fixture that preceded these
# sat in untracked scratch and silently outlived two config migrations (the
# retired `[ls] default-filter`, the `depends-on:` → `links:` move), so no eval
# could have run against it and nothing said so.
check-examples:
    cargo run -p mindtape-cli -q -- -w examples/learn-piano check
    cargo run -p mindtape-cli -q -- -w examples/minimal check

# (Re)generate the examples/large/ benchmark fixture: N deterministic task
# files under examples/large/tasks/ (gitignored). Default N=2000.
gen-bench N='2000':
    examples/large/generate.sh {{ N }}

# Benchmark `mt ls` and `mt check` against a large generated task set (N
# files, default 2000): build the release binary, regenerate the fixture,
# then time both commands with hyperfine at a pinned clock (MINDTAPE_NOW).
bench N='2000':
    direnv exec . cargo build --release -p mindtape-cli
    examples/large/generate.sh {{ N }}
    export MINDTAPE_NOW=2026-07-16 && direnv exec . hyperfine \
        --warmup 1 --runs 3 \
        --command-name "mt ls ({{ N }} tasks)" "./target/release/mt -w examples/large ls" \
        --command-name "mt check ({{ N }} tasks)" "./target/release/mt -w examples/large check"

# Check the `rrule(...)` escape hatch's expansion engine: `just check` builds
# `mindtape` with default features, so it never compiles or runs this code
# (see crates/mindtape/src/recur_rrule.rs). Reproduces what CI runs for it.
check-rrule:
    cargo fmt --all -- --check
    cargo clippy -p mindtape --features rrule --all-targets -q -- -D warnings
    cargo test -p mindtape --features rrule -q

# Cross-check the workspace, tests included, against Windows without leaving
# Linux — the compile half of what the CI windows job proves at runtime, minutes
# earlier and without pushing. Runs in its own dev shell (flake.nix
# `devShells.windows`) because the default shell's rustc has no Windows std;
# see that shell's comment for why the target is -gnu and the release is -msvc.
#
# Deliberately not a `just check` arm: it pulls a second toolchain closure and
# a separate target/ tree, and CI already covers the regression. Reach for it
# when touching a `cfg(windows)` arm, a process spawn, a path join, or anything
# else with a platform seam.
#
# `just --list` shows only the LAST comment line of a recipe, so the summary is
# repeated as an explicit [doc] rather than left as a dangling fragment.
[doc("Cross-check the workspace against Windows (x86_64-pc-windows-gnu)")]
check-windows:
    nix develop .#windows --command \
        cargo check --workspace --all-targets --locked \
        --target x86_64-pc-windows-gnu

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

# Capture live `mt` output for the docs (www/generated/*.txt), so the site's
# CLI reference, help, version, and transcripts cannot drift from the binary.
# Builds `mt` release, runs it against examples/learn-piano at a pinned clock.
docs-gen:
    bash www/generate.sh

# Build the documentation site (www/ -> www/public/mindtape) with tola, then
# index the built HTML with Pagefind (bundle -> public/mindtape/pagefind/).
# docs-gen runs first so the injected transcripts reflect the current binary.
docs-build: docs-gen
    cd www && tola build && pagefind --site public/mindtape --silent

# Serve the docs locally with hot reload (no search index — build for that)
docs-serve: docs-gen
    cd www && tola serve

# Build the docs, index with Pagefind, then validate internal links + assets.
# Pagefind runs before validate so the /pagefind/ bundle exists on disk.
docs-check: docs-gen
    cd www && tola build && pagefind --site public/mindtape --silent && tola validate

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

# Run coverage with cargo-llvm-cov (also writes target/coverage/lcov.info).
# llvm-cov, unlike tarpaulin, merges the profraw of child processes, so the
# subprocess-based CLI e2e tests actually attribute to the code they exercise.
# Instrument once (--no-report), then render both reports from that profdata.
# main.rs is excluded — keep it thin (`just crap` excludes it too); so is any
# registry checkout, which llvm-cov's own dep filter misses when CARGO_HOME is
# relocated (a local kache), so local and CI report the same set.
cover:
    #!/usr/bin/env bash
    set -eo pipefail
    ignore='crates/mindtape-cli/src/main\.rs|/registry/src/'
    mkdir -p target/coverage
    cargo llvm-cov --workspace --no-report
    cargo llvm-cov report --ignore-filename-regex "$ignore" \
        --lcov --output-path target/coverage/lcov.info
    cargo llvm-cov report --ignore-filename-regex "$ignore" --fail-under-lines 90

# Gate complex, undertested functions via CRAP metric, over the lcov `just
# cover` writes.
#
# `--min` matches `--threshold`, so the table lists only the functions that
# FAIL. Without it the report is every analyzed function — 976 rows to carry the
# one that matters. The cost is that the tail summary then counts printed rows
# ("1/1", not "1/976"); the numerator, which is the number that decides the
# gate, is unaffected.
#
# The `*_tests.rs` exclusions keep CRAP on production code. They are inline test
# modules (`#[path] mod tests`), which llvm-cov emits no records for, so scoring
# them meant 71 of the 73 "no matching entry in the LCOV report" warnings and a
# complexity budget spent on test helpers. Excluding them changes no verdict.
#
# The lcov is a build artifact nothing re-derives for you, and scoring against a
# stale one does not degrade gracefully: a function in a file written since the
# lcov has no coverage record at all, `--missing` defaults to `pessimistic`, and
# so it reads as 0% covered rather than as unknown. Observed on a 10-day-old
# lcov — 28 failures, 27 of them phantom (each showed coverage `—`, not a
# percentage). A gate that invents failures is worse than no gate, so refuse a
# stale lcov and name the recipe that refreshes it. CI is unaffected: its
# `coverage` job runs `just cover` immediately before this.
crap:
    #!/usr/bin/env bash
    set -eo pipefail
    lcov=target/coverage/lcov.info
    if [ ! -f "$lcov" ]; then
        echo "just crap: $lcov is missing — run \`just cover\` first" >&2
        exit 1
    fi
    newer=$(find crates -name '*.rs' -newer "$lcov" -print -quit)
    if [ -n "$newer" ]; then
        echo "just crap: $lcov predates $newer — run \`just cover\` first" >&2
        echo "  (a source the lcov never saw scores as 0% covered, not as unknown)" >&2
        exit 1
    fi
    cargo crap --lcov "$lcov" --workspace \
        --exclude 'crates/mindtape-cli/src/main.rs' \
        --exclude '**/*_tests.rs' --exclude '**/tests.rs' \
        --threshold 30 --min 30 --fail-above

# Pre-PR mutation check — the daily driver (see CONTRIBUTING.md "Testing").
# Sweeps only mutants touched by this branch's diff against main, so it needs
# no state file (nothing to invalidate) and covers new files for free (every
# line of a new file is "touched"). A surviving `MISSED` mutant is a real
# assertion gap: a line covered but not pinned. Speed knobs (benchmarked
# ~3.5x over the naive run): CARGO_MUTANTS_JOBS=3 parallel jobs and the mold
# linker via RUSTFLAGS — both overridable; drop jobs to 2 if parallel load
# provokes false TIMEOUTs. Extra args pass through to cargo-mutants.
mutants-diff *ARGS:
    #!/usr/bin/env bash
    set -eo pipefail
    export CARGO_MUTANTS_JOBS="${CARGO_MUTANTS_JOBS:-3}"
    export RUSTFLAGS="${RUSTFLAGS:-} -Clink-arg=-fuse-ld=mold"
    diff_file=$(mktemp)
    trap 'rm -f "$diff_file"' EXIT
    git diff "$(git merge-base main HEAD)" > "$diff_file"
    cargo mutants --in-diff "$diff_file" {{ ARGS }}

# Deliberate full-file sweep — a dev aid, never a `just check` gate (see
# CONTRIBUTING.md "Testing"). `-f <file>` forces one file; any other args pass
# through to cargo-mutants untouched. File set, excludes (main.rs + equivalent
# mutants), the rrule feature and the debuginfo-free `mutants` profile come
# from .cargo/mutants.toml. Same speed knobs as `mutants-diff`.
mutants *ARGS:
    #!/usr/bin/env bash
    set -eo pipefail
    export CARGO_MUTANTS_JOBS="${CARGO_MUTANTS_JOBS:-3}"
    export RUSTFLAGS="${RUSTFLAGS:-} -Clink-arg=-fuse-ld=mold"
    cargo mutants {{ ARGS }}

# Format code
fmt:
    cargo fmt --all
    tombi format \
        .mindtape/config.toml \
        examples/learn-piano/.mindtape/config.toml \
        examples/large/.mindtape/config.toml
    typstyle --inplace \
        lib \
        truth/tasks \
        examples/learn-piano/tasks \
        examples/learn-piano/.mindtape/templates \
        www/content \
        www/templates \
        www/utils

# Format check (CI-friendly)
fmt-check:
    cargo fmt --all -- --check
    tombi format --check \
        .mindtape/config.toml \
        examples/learn-piano/.mindtape/config.toml \
        examples/large/.mindtape/config.toml
    typstyle --check \
        lib \
        truth/tasks \
        examples/learn-piano/tasks \
        examples/learn-piano/.mindtape/templates \
        www/content \
        www/templates \
        www/utils

# Lock the formatter gates: synthesize a deliberately-mis-formatted TOML and
# Typst fixture in a temp dir, then assert `tombi format --check` and
# `typstyle --check` both fail. The fmt recipe is what `just fmt`/`fmt-check`
# apply to the tracked config + docs tree, so the same formatters must
# reject the mis-formatted fixture (and accept a re-formatted one). A
# formatter that silently accepts drift would let `just fmt-check` pass on
# an out-of-sync editor save — exactly the bug this task is fixing.
fmt-fixture-check:
    #!/usr/bin/env bash
    set -eo pipefail
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    bad_toml="$tmp/bad.toml"
    bad_typ="$tmp/bad.typ"
    printf '[a]\nb=1\nc  =   2\n' > "$bad_toml"
    printf '#let   x   =   1\n#let y=2\n' > "$bad_typ"
    if tombi format --check --offline "$bad_toml" >/dev/null 2>&1; then
        echo "fmt-fixture-check: tombi accepted the mis-formatted TOML — gate is broken" >&2
        exit 1
    fi
    if typstyle --check "$bad_typ" >/dev/null 2>&1; then
        echo "fmt-fixture-check: typstyle accepted the mis-formatted Typst — gate is broken" >&2
        exit 1
    fi
    echo "fmt-fixture-check: both formatters reject drift"

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
