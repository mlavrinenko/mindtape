# MindTape's root justfile. `just` with no arguments lists every recipe.
#
# Where a comment about a recipe goes, now that this file no longer holds all
# three kinds (truth/tasks/relocate-justfile-commentary-to-scripts-and-
# docs.typ):
#
#   - HERE: what a caller needs before running it — what it does, when to
#     reach for it, what it refuses to do. The `[doc(...)]` string plus at
#     most a few lines.
#   - IN THE SCRIPT (.just/scripts/*.sh): why the code inside is written the
#     way it is, next to the line it explains.
#   - IN docs/src/contributing/*.typ: the arguments — what was measured, what
#     was tried, why a band or a job deliberately does not exist. `just
#     docs::md` renders those to docs/contributing/*.md.
#
# `just --list` shows only the LAST comment line of a recipe, so a summary
# worth reading there is repeated as an explicit `[doc(...)]` rather than left
# as a dangling fragment.

set quiet := true
set unstable := true

# Six recipes live in .just/docs.just rather than here
# (truth/tasks/split-the-justfile-into-modules.typ): docs::gen, docs::build,
# docs::serve and docs::md (render docs/src/*.typ to the repo-root Markdown),
# plus the two gates — docs::check (build, index with Pagefind, then validate
# internal links + assets) and docs::md-check (the docs-md freshness gate).
# `just docs::<name>` and `just docs <name>` both work.
#
# Why the path is spelled out, why `docs::md` reaches doc prose as `docs-md`,
# and why the module file carries `set working-directory := '..'` of its own
# are argued in .just/docs.just's header.

# Docs site generation, build, serve, and the docs-md/docs-check gates
mod docs '.just/docs.just'

[doc("List available recipes")]
[group("workflow")]
default:
    @just --list

[doc("Run fixes, then run the full check gate")]
[group("workflow")]
fix-check: eject fmt clippy-fix check

# Every arm is `just mmz <subgate>`, so a no-op re-run skips each one whose
# declared inputs are unchanged. What the memoization buys, why there is no
# `chronic` wrapper, and how `mt flip` asserts the whole set fresh without
# running any of it are in docs/contributing/gates.md.

[doc("Run every check-gate sub-recipe in parallel")]
[group("workflow")]
[parallel]
check: (mmz "fmt-check") (mmz "clippy") (mmz "test") (mmz "machete") (mmz "check-file-size") (mmz "check-dry") (mmz "outdatty-check") (mmz "check-examples") (mmz "check-doc-coverage") (mmz "check-doc-facts") (mmz "check-doc-recipes") (mmz "docs::check") (mmz "docs::md-check") (mmz "fmt-fixture-check") (mmz "arg-pattern-check") (mmz "positional-args-check") check-crap

# Wrap a just recipe in mmz: `just mmz fmt-check` runs `mmz just fmt-check`.
# Why the double hop, and why `just test <filter>` deliberately bypasses it,
# are in docs/contributing/gates.md.

[doc("Run a just recipe through mmz memoization")]
[group("workflow")]
mmz *ARGS:
    mmz just {{ ARGS }}

[doc("Check formatting without writing changes (CI-friendly)")]
[group("gate")]
fmt-check:
    cargo fmt --all -- --check
    tombi format --check {{ TOML_FMT_PATHS }}
    typstyle --check {{ TYPST_FMT_PATHS }}
    just --fmt --check
    for f in {{ JUST_FMT_PATHS }}; do just --justfile "$f" --fmt --check; done

[doc("Run clippy only")]
[group("dev")]
[group("gate")]
clippy:
    cargo clippy --workspace --all-targets -q -- -D warnings

[doc("Run tests only")]
[group("dev")]
[group("gate")]
[positional-arguments]
test *ARGS:
    cargo test --workspace "$@"

[doc("Check for unused dependencies")]
[group("gate")]
machete:
    cargo machete

[doc("Fail if any file exceeds its linecop cap")]
[group("gate")]
check-file-size:
    linecop

[doc("Copy-paste gate: fail on any duplicated Rust block")]
[group("gate")]
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

[doc("Fail if a source changed without its dependents reconfirmed")]
[group("gate")]
outdatty-check:
    outdatty check

# So a fixture some other tool runs `mt` against can never rot: learn-piano
# feeds the docs generator and the skill's triage evals, minimal feeds the
# skill's trap evals. Why gating them is what earns the right to depend on
# them is in docs/contributing/gates.md.

[doc("Validate each example project against itself")]
[group("gate")]
check-examples:
    cargo run -p mindtape-cli -q -- -w examples/learn-piano check
    cargo run -p mindtape-cli -q -- -w examples/minimal check

# Every visible top-level `mt` subcommand must have a www/utils/cli-notes.typ
# entry. How the set-difference is taken, and what it deliberately never
# excludes by hand, are argued in .just/scripts/check-doc-coverage.sh.

[doc("Fail if a visible mt subcommand has no CLI doc entry")]
[group("gate")]
check-doc-coverage:
    bash .just/scripts/check-doc-coverage.sh

# Every derived fact www/generate-facts.sh writes must have a hand-written
# www/utils/*-notes.typ entry. The arms, and why the gate regenerates its own
# inputs, are argued in .just/scripts/check-doc-facts.sh.

[doc("Fail if a derived crate, DSL field, gate, or sweep flag fact has no doc entry")]
[group("gate")]
check-doc-facts:
    bash .just/scripts/check-doc-facts.sh

# A doc source names a recipe through www/generated/just.typ; this catches the
# hand-typed mention that never went through it. The three literal shapes it
# matches, which files it scans, and why ordinary English "just" is never
# touched are argued in .just/scripts/check-doc-recipes.sh.

[doc("Fail on a hand-typed just recipe mention outside the generated module")]
[group("gate")]
check-doc-recipes:
    bash .just/scripts/check-doc-recipes.sh

# Synthesize a deliberately-mis-formatted TOML, Typst and just-module fixture
# in a temp dir, then assert every formatter `fmt`/`fmt-check` run rejects it —
# and accepts a re-formatted one. Why the just arm is spelled the way it is,
# and why the SUCCESS case is asserted too, are argued in place in
# .just/scripts/fmt-fixture-check.sh.

[doc("Lock the formatter gates against a mis-formatted fixture")]
[group("gate")]
fmt-fixture-check:
    bash .just/scripts/fmt-fixture-check.sh

# Lock the `[arg(...)]` patterns on release/bench/gen-bench/eject and the five
# `ask-refactoring` thresholds: each must reject a known-bad argument and
# accept a known-good one at DISPATCH, before the recipe body runs
# (truth/tasks/validate-recipe-arguments-with-the-arg-attribute.typ). No recipe
# body ever actually runs, `release` included — argued in place in
# .just/scripts/arg-pattern-check.sh.

[doc("Lock the arg-attribute patterns against known-bad and known-good input")]
[group("gate")]
arg-pattern-check:
    bash .just/scripts/arg-pattern-check.sh

# Lock argument forwarding on the passthrough recipes (`test`, `build`,
# `mutants`, `mutants-diff`): each must carry `[positional-arguments]` and
# forward "$@" rather than interpolate `{{ ARGS }}`/`{{ args }}` into its body
# — a bare interpolation re-splits an argument containing a space and re-globs
# one containing a wildcard
# (truth/tasks/preserve-argument-quoting-with-positional-arguments.typ). Why it
# is static, why it is separate from arg-pattern-check, and why
# `ask-refactoring` left it by construction rather than by exemption are argued
# in place in .just/scripts/positional-args-check.sh.

[doc("Lock positional-argument forwarding on the four passthrough recipes")]
[group("gate")]
positional-args-check:
    bash .just/scripts/positional-args-check.sh

# The CRAP gate as a `just check` arm. Runs the real gate when the lcov is
# fresh; SKIPS LOUDLY otherwise, naming what would refresh it, and exits 0. It
# is the cheap partial, not the correct one: `just check-full` is the arm that
# is always right. Why it exists, what it does and does not buy, and why it is
# the one arm not wrapped in mmz are in docs/contributing/coverage-crap.md and
# docs/contributing/gates.md.

[doc("Run the CRAP gate when the lcov is fresh, else skip loudly")]
[group("gate")]
check-crap:
    bash .just/scripts/check-crap.sh

# (Re)generate the examples/large/ benchmark fixture: N deterministic task
# files under examples/large/tasks/ (gitignored). Default N=2000.

[arg('N', pattern='\d+', help='number of deterministic task files to generate')]
[doc("Regenerate the examples/large benchmark fixture")]
[group("dev")]
gen-bench N='2000':
    examples/large/generate.sh {{ N }}

# Benchmark `mt ls` and `mt check` against a large generated task set (N
# files, default 2000): build the release binary, regenerate the fixture,
# then time both commands with hyperfine at a pinned clock (MINDTAPE_NOW).

[arg('N', pattern='\d+', help='number of generated task files to benchmark against')]
[doc("Benchmark mt ls and mt check against a large task set")]
[group("dev")]
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

[doc("Check the rrule feature build, lint, and tests")]
[group("dev")]
check-rrule:
    cargo fmt --all -- --check
    cargo clippy -p mindtape --features rrule --all-targets -q -- -D warnings
    cargo test -p mindtape --features rrule -q

# Cross-check the workspace, tests included, against Windows without leaving
# Linux — the compile half of what the CI windows job proves at runtime, minutes
# earlier and without pushing. Reach for it when touching a `cfg(windows)` arm,
# a process spawn, a path join, or anything else with a platform seam. Runs in
# its own dev shell (flake.nix `devShells.windows`); see that shell's comment
# for why the target is -gnu and the release is -msvc, and
# docs/contributing/windows.md for why this is deliberately not a `just check`
# arm.

[doc("Cross-check the workspace against Windows (x86_64-pc-windows-gnu)")]
[group("dev")]
check-windows:
    nix develop .#windows --command \
        cargo check --workspace --all-targets --locked \
        --target x86_64-pc-windows-gnu

[doc("Re-confirm dependency groups: record hashes into outdatty.lock")]
[group("dev")]
outdatty-update:
    outdatty update

# Prove `nix build` succeeds in a clean sandbox (no dev shell, no sccache).
# Catches a committed .cargo/config.toml — or any dev-only setting — leaking
# into the package build and breaking downstream flake consumers. Stage first:
# flakes filter source to git-tracked files, and naersk needs the current tree.

[doc("Verify nix build succeeds in a clean sandbox")]
[group("release")]
nix-build:
    git add -A
    nix build .#default

[doc("Auto-fix clippy warnings")]
[group("dev")]
clippy-fix:
    cargo clippy --fix --workspace --all-targets --allow-dirty --allow-staged -- -D warnings

[doc("Build the project")]
[group("dev")]
[positional-arguments]
build *args:
    cargo build --workspace -q "$@"

# Run coverage with cargo-llvm-cov (also writes target/coverage/lcov.info, the
# lcov `just crap` scores against). It always pays a full instrumented rebuild:
# why that is not optional, and what an accumulated profraw pile did to the
# numbers when it was, are in docs/contributing/coverage-crap.md.

[doc("Run coverage with cargo-llvm-cov")]
[group("advisory")]
cover:
    bash .just/scripts/cover.sh

# Why target/coverage/lcov.info is unusable, or empty output when it is fine.
# One implementation for two callers with opposite policies — argued in place
# in .just/scripts/_lcov-stale-reason.sh.

[doc("Print why the lcov is stale, or nothing if fresh")]
[private]
_lcov-stale-reason:
    bash .just/scripts/_lcov-stale-reason.sh

# `just check` plus the two gates it cannot afford to run every time. This is
# the pre-PR run: it is always correct about coverage and CRAP, at the price of
# an instrumented rebuild and a full suite pass. Measured at ~21s with a hot
# `target/` on the maintainer's machine; a cold run and CI are both larger.

[doc("Run check plus coverage and the CRAP gate")]
[group("workflow")]
check-full: check cover crap

# What `crap` and `crap-baseline` both exclude. One list, two consumers, for the
# reason TYPST_FMT_PATHS exists: two hand-maintained copies drift, and a baseline
# minted over a different file set than the gate scores would report every
# difference between the two lists as a regression.

CRAP_EXCLUDES := "--exclude **/main.rs --exclude **/recur_rrule.rs --exclude **/ls_interactive/tui.rs --exclude **/*_tests.rs --exclude **/tests.rs"

# Same list for a third consumer that is not a `just` recipe: the sweep's
# undertested arm (contrib/ask-refactoring/undertested.sh) runs `cargo crap` for
# its per-function complexity and coverage, and must score the same file set the
# gate does or it files findings about `main.rs` and a feature-gated stub. A
# recipe rather than a copy, for the reason above.

[doc("Print the CRAP exclude flags shared across consumers")]
[private]
_crap-excludes:
    @echo '{{ CRAP_EXCLUDES }}'

# Gate complex, undertested functions via the CRAP metric, over the lcov `just
# cover` writes. Refuses a stale lcov instead of scoring against it, and names
# the recipe that refreshes it. The threshold, the exclusions, and why there is
# deliberately no advisory band below the gate are in
# docs/contributing/coverage-crap.md; how each flag is spelled and why is
# argued in place in .just/scripts/crap.sh.

[doc("Gate complex, undertested functions via the CRAP metric")]
[group("advisory")]
crap:
    bash .just/scripts/crap.sh {{ CRAP_EXCLUDES }}

# Re-mint crap-baseline.json after a DELIBERATE complexity increase. Run it in
# the same commit as the change that justifies it, so the diff shows the new
# number next to the reason for it — a ratchet with no documented release valve
# gets bypassed instead of refreshed.
#
# Mint only against a green gate: the baseline records today as acceptable, so
# minting while something is red freezes the red. `just crap` runs first here
# for exactly that reason, and its failure aborts the mint. That guard has one
# blind spot, and the recovery when it fires is in
# docs/contributing/coverage-crap.md.

[doc("Re-mint crap-baseline.json after a deliberate complexity increase")]
[group("advisory")]
crap-baseline: crap
    bash .just/scripts/crap-baseline.sh {{ CRAP_EXCLUDES }}

# Pre-PR mutation check — the daily driver (see CONTRIBUTING.md "Testing").
# Sweeps only mutants touched by this branch's diff against main, so it needs
# no state file (nothing to invalidate) and covers new files for free (every
# line of a new file is "touched"). Extra args pass through to cargo-mutants.
# What a surviving mutant means, and when a verdict is worth believing, are in
# docs/contributing/mutation-testing.md; the speed knobs are argued in place in
# .just/scripts/_mutants-env.sh.

[doc("Run mutation testing over this branch's diff against main")]
[group("advisory")]
[positional-arguments]
mutants-diff *ARGS:
    bash .just/scripts/mutants-diff.sh "$@"

# Deliberate full-file sweep — a dev aid, never a `just check` gate (see
# CONTRIBUTING.md "Testing"). `-f <file>` forces one file; any other args pass
# through to cargo-mutants untouched. File set, excludes (main.rs + equivalent
# mutants), the rrule feature and the debuginfo-free `mutants` profile come
# from .cargo/mutants.toml. Same speed knobs as `mutants-diff`.
#
# The TIMEOUT caveat, what a REPRODUCIBLE timeout means, what to do with a
# survivor inside `#[cfg(not(feature = …))]` code, and why there is
# deliberately no CI mutation job are in docs/contributing/mutation-testing.md.

[doc("Run a full-file mutation testing sweep")]
[group("advisory")]
[positional-arguments]
mutants *ARGS:
    bash .just/scripts/mutants.sh "$@"

# What the non-Rust formatters cover. One list each, read by both `fmt` and
# `fmt-check`, because two hand-maintained copies drift: `.mindtape/templates`
# (the project's own `mt add` templates) sat outside both lists while the
# learn-piano example's templates sat inside, so the example project's
# templates were formatted and the real ones were not. A single list cannot
# omit a path from the check while applying it in the fix.

TOML_FMT_PATHS := ".mindtape/config.toml examples/learn-piano/.mindtape/config.toml examples/large/.mindtape/config.toml"
TYPST_FMT_PATHS := "lib truth/tasks .mindtape/templates examples/learn-piano/tasks examples/learn-piano/.mindtape/templates www/content www/templates www/utils docs/src"

# The same one-list discipline for the justfiles themselves. A bare `just --fmt`
# formats the ROOT justfile only and silently skips every `mod` file, so each
# module file needs its own `--justfile <file> --fmt` pass and the set of them
# lives here rather than being spelled twice: a third module file cannot land
# in the fix and miss the check. Every file listed needs `set unstable := true`
# of its own — `--fmt` is unstable and reads that setting from the file it is
# POINTED AT, never from the Justfile that declares the `mod`.
#
# The paths follow the module files, and must: this list is the whole coverage
# of the formatter gate over them, so a module file that moves without its
# entry moving silently drops back out of `just fmt` and `just fmt-check`. The
# verification behind that, and the drift it already cost, are in
# docs/contributing/gates.md.

JUST_FMT_PATHS := ".just/docs.just"

[doc("Format code")]
[group("dev")]
fmt:
    cargo fmt --all
    tombi format {{ TOML_FMT_PATHS }}
    typstyle --inplace {{ TYPST_FMT_PATHS }}
    just --fmt
    for f in {{ JUST_FMT_PATHS }}; do just --justfile "$f" --fmt; done

[doc("Count tests across the workspace")]
[group("advisory")]
count-tests:
    #!/usr/bin/env bash
    cargo test --workspace 2>&1 | grep "test result:" | awk '{sum += $4} END {print sum " tests"}'

[doc("Show the top 20 files by line count")]
[group("advisory")]
file-sizes:
    #!/usr/bin/env bash
    find . -type f \( -name '*.rs' -o -name '*.md' \) ! -path './target/*' -exec wc -l {} + | sort -rn | head -20

[arg('PCT', pattern='(100|[1-9]?[0-9])', help='linecop baseline percentage (0-100) — files at or above it get ejected')]
[doc("Eject inline tests from Rust files nearing the linecop cap")]
[group("advisory")]
eject PCT='90':
    linecop --baseline {{ PCT }} --format paths | ejectest apply . --files-from - --lenient

# File backlog tasks from the ADVISORY quality signals — the ones that predict
# mechanical work instead of blocking on it. Four arms: linecop (file size),
# per-file line coverage — which nothing gates, since `just cover` fails on the
# workspace total only — per-function complexity against coverage, and surviving
# mutants. Each arm is a scoring function plus a `.mindtape/templates/sweep-*.typ`
# template, not a config change.
#
# Dry by default: it prints, via `mt add --print`, the exact tasks it would
# file; `--write` files them. It refuses to run while `mt ls @ask-refactoring`
# is non-empty, so one sprint of mechanical work is emptied before the next is
# filed. The five parameters below are the only copy of their defaults, and the
# reason each arm carries its own threshold is in
# docs/contributing/signal-sweep.md.
#
# Deliberately no longer a passthrough recipe: its five pattern-checked scalars
# replaced `[positional-arguments]`, which is why `just positional-args-check`
# no longer names it and `just arg-pattern-check` does — argued in place in
# .just/scripts/positional-args-check.sh.

[arg('COVERAGE_FLOOR', long='coverage-floor', pattern='(100|[1-9]?[0-9])', help='coverage arm: report files below PCT% line coverage')]
[arg('WRITE', long='write', pattern='[01]', value='1', help='file the tasks instead of printing what would be filed')]
[arg('BASELINE', long='baseline', pattern='(100|[1-9]?[0-9])', help='size arm: report files at or above PCT% of their linecop limit')]
[arg('CC_FLOOR', long='cc-floor', pattern='\d+', help='undertested arm: floor on cyclomatic complexity')]
[arg('FN_COVERAGE_FLOOR', long='fn-coverage-floor', pattern='(100|[1-9]?[0-9])', help='undertested arm: report functions below PCT% line coverage')]
[doc("File backlog tasks from advisory quality signals")]
[group("advisory")]
ask-refactoring $WRITE='0' $BASELINE='90' $COVERAGE_FLOOR='88' $CC_FLOOR='5' $FN_COVERAGE_FLOOR='80':
    bash contrib/ask-refactoring.sh --write "$WRITE" --baseline "$BASELINE" \
        --coverage-floor "$COVERAGE_FLOOR" --cc-floor "$CC_FLOOR" \
        --fn-coverage-floor "$FN_COVERAGE_FLOOR"

[doc("Install the mindtape Typst library for local #import (dev symlink)")]
[group("dev")]
install-lib:
    bash .just/scripts/install-lib.sh

[doc("Remove the dev symlink installed by install-lib")]
[group("dev")]
uninstall-lib:
    #!/usr/bin/env bash
    VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' lib/typst.toml | head -1)
    TARGET="$HOME/.local/share/typst/packages/local/mindtape/$VERSION"
    [ -L "$TARGET" ] && rm "$TARGET" && echo "removed $TARGET" || echo "no symlink at $TARGET"

# Point git at the tracked hook dir (pre-commit asserts the memoized gate via
# `mmz --is-fresh --tag gate`, then checks task-file formatting and structure)

[doc("Point git at the tracked hook directory")]
[group("dev")]
install-hooks:
    #!/usr/bin/env bash
    set -eo pipefail
    git config core.hooksPath .githooks
    rm -f .git/hooks/pre-commit
    echo "installed .githooks"

# Tag a release and push (usage: just release 0.3.0). Refuses to run in a
# dirty worktree, and while CHANGELOG.md has an [Unreleased] section with
# content — move it into a dated [VERSION] section first. Pushes main before
# the tag so the docs site (pages.yml) and CI run on the release tree.
#
# The MINDTAPE_RELEASE_CONFIRMED prefix is how the [confirm] guard survives the
# body living in a script — argued in full in .just/scripts/release.sh.

[arg('VERSION', pattern='\d+\.\d+\.\d+', help='semver, no leading v (e.g. 0.3.0)')]
[confirm("Tag and push a release to origin — proceed?")]
[doc("Tag a release and push")]
[group("release")]
release VERSION:
    MINDTAPE_RELEASE_CONFIRMED=1 bash .just/scripts/release.sh "{{ VERSION }}"
