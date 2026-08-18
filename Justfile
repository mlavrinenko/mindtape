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

# Every `[script]` recipe runs under this interpreter, so bash strict mode
# (`-e` plus `pipefail`) arrives by construction instead of by a per-recipe
# `set -eo pipefail` that each new recipe must remember to type
# (truth/tasks/adopt-the-script-attribute-for-the-shebang-recipes.typ).

set script-interpreter := ["bash", "-eo", "pipefail"]

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
check: (mmz "fmt-check") (mmz "clippy") (mmz "test") (mmz "machete") (mmz "check-file-size") (mmz "check-dry") (mmz "check-secrets") (mmz "outdatty-check") (mmz "check-examples") (mmz "check-doc-coverage") (mmz "check-doc-facts") (mmz "check-doc-recipes") (mmz "docs::check") (mmz "docs::md-check") (mmz "fmt-fixture-check") (mmz "arg-pattern-check") (mmz "positional-args-check") (mmz "script-attr-check") (mmz "size-fixture-check") (mmz "script-scope-check") (mmz "site-pages-fixture-check") (mmz "check-crap") (mmz "crap-baseline-check")

# Wrap a just recipe in mmz: `just mmz fmt-check` runs `mmz just fmt-check`.
# Why the double hop, and why `just test <filter>` deliberately bypasses it,
# are in docs/contributing/gates.md.

[doc("Run a just recipe through mmz memoization")]
[group("workflow")]
[no-exit-message]
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
[extension('.sh')]
[group("gate")]
[script]
check-dry:
    # jscpd (the Rust port, packaged in flake.nix) fails on any duplicated
    # block of >=70 tokens, naming both spans. Scans all Rust under crates/.
    # Fix a finding by extracting a shared helper — never by shuffling tokens
    # until the detector loses the scent.
    #
    # `--exit-code 1` is spelled with its value: bare `--exit-code` would
    # swallow the following path argument as its optional code.
    jscpd --no-tips -k 70 -f rust --exit-code 1 crates

# Scan the working tree for a committed credential with gitleaks, under
# `.gitleaks.toml` (the default ruleset plus this repo's two allowlists, each
# argued in that file). Current tree only — history is deliberately out of
# scope. A control fixture runs first, because "no leaks found" is what an
# over-broad allowlist prints too; that, and why the arm also checks its own
# mmz input list, are argued in .just/scripts/check-secrets.sh.

[doc("Fail if the working tree carries a committed credential")]
[group("gate")]
check-secrets:
    bash .just/scripts/check-secrets.sh

[doc("Fail if a source changed without its dependents reconfirmed")]
[group("gate")]
outdatty-check:
    outdatty check

# So a fixture some other tool runs `mt` against can never rot: learn-piano
# feeds the docs generator and the skill's triage evals, minimal feeds the
# skill's trap evals. Why gating them is what earns the right to depend on
# them is in docs/contributing/gates.md.
#
# "Against itself" is enforced, not trusted. Each fixture declares a
# `[[typst.local-packages]]` entry pointing `@local/mindtape` at this
# checkout's lib/; the machine-global local-package directory would serve the
# same import from whichever checkout last ran `just install-lib`, so a fixture
# that lost its entry would keep passing here against some other tree's
# prelude. An empty `MINDTAPE_DATA_HOME` deletes that fallback — the variable
# set-but-empty means "no such directory", which resolves no global package at
# all rather than falling back — so the entry is load-bearing, and a fork
# editing the DSL gates against the prelude it is editing. It is also what lets
# a contributor, and CI, which never runs `mt install`, run this gate on a
# machine with no global install at all.
#
# The variable rather than XDG_DATA_HOME because `dirs` reads the environment
# on Unix only: XDG_DATA_HOME would isolate this gate on Linux, leave macOS
# reading the machine's packages, and mean nothing on Windows. `mt` resolves
# the local-package directory through its own setting on every platform
# (crates/mindtape/src/loader.rs), so the gate says what it means everywhere.

[doc("Validate each example project against itself")]
[group("gate")]
check-examples:
    MINDTAPE_DATA_HOME= cargo run -p mindtape-cli -q -- -w examples/learn-piano check
    MINDTAPE_DATA_HOME= cargo run -p mindtape-cli -q -- -w examples/minimal check

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

# Lock the file-size gate's SCAN SCOPE: an oversized shell script under
# `.just/scripts/` must be reported by `linecop` reading this repo's
# `.linecop.yaml`. The subject is the walker, not the caps — a directory the
# scan never enters reports the same "All files within size limits." a clean
# tree does, which is how every body extracted out of the Justfile sat
# uncapped while the extraction claimed the opposite
# (truth/tasks/restore-size-gate-coverage-over-the-dot-just-scripts.typ). Why
# it runs against a replica tree instead of the real directory, and why the
# unreported case is asserted too, are argued in place in
# .just/scripts/size-fixture-check.sh.

[doc("Lock the size gate's scan scope against an oversized hidden-dir fixture")]
[group("gate")]
size-fixture-check:
    bash .just/scripts/size-fixture-check.sh

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

# Lock bash strict mode in by construction: no recipe body may open with a `#!`
# shebang (every script body goes through `[script]`, so `set
# script-interpreter` reaches it), and every justfile must declare a setting
# that actually aborts a failed pipeline. The setting removed the unstrict
# bodies that existed, not the shape — `count-tests` printed a wrong test count
# and exited 0 in exactly the form a new `#!` recipe would still be written in
# (truth/tasks/lock-the-script-attribute-against-a-shebang-regression.typ). Why
# `.shebang` cannot be the discriminator, why the setting is PROVEN by a probe
# instead of matched as text, and why every justfile needs its own are argued in
# place in .just/scripts/script-attr-check.sh.

[doc("Lock the script attribute and each justfile's strict-mode interpreter")]
[group("gate")]
script-attr-check:
    bash .just/scripts/script-attr-check.sh

# Lock both manifests that enumerate `.just/scripts/` onto the directory they
# enumerate: every script must be named by an mmz scope as a literal path, every
# memoized rule whose body invokes a script must name that script's scope, and
# every script must be an `outdatty.yaml` `dev-docs` source. This is what
# `.mmz/config.yaml` bought the scope split with — the `.just/scripts/**` glob
# it replaced covered a new body by ARRIVING, and an enumeration only covers
# what somebody listed (truth/tasks/split-gates-meta-into-per-script-scopes.typ,
# and truth/tasks/fail-a-half-wired-gate-arm-at-the-wiring-it-is-missing.typ for
# the outdatty half). Why a glob can never satisfy the first arm, which case the
# second deliberately cannot see, and why the third reads the config rather than
# the lock, are argued in place in .just/scripts/script-scope-check.sh.

[doc("Lock every gate script into a named mmz scope and outdatty's dev-docs group")]
[group("gate")]
script-scope-check:
    bash .just/scripts/script-scope-check.sh

# Lock the docs manifest gate itself: two pages declaring the same route, a
# page with no `<page-meta>` block, and a page missing from
# www/utils/site.typ's NAV were each demonstrated by hand once, when they
# landed, and never proven again since — a gate that never fires reads
# identical to one that cannot
# (truth/tasks/pin-the-docs-manifest-gate-failures-with-a-selftest.typ). Why
# it runs against a replica tree rather than the working one, and why the
# third case drives `.just/scripts/_nav-diff.sh` directly instead of the
# whole check-doc-facts regenerate, are argued in place in
# .just/scripts/site-pages-fixture-check.sh.

[doc("Lock the docs manifest gate against three known-bad fixtures")]
[group("gate")]
site-pages-fixture-check:
    bash .just/scripts/site-pages-fixture-check.sh

# The CRAP gate as a `just check` arm. Runs the real gate when the lcov is
# fresh; SKIPS LOUDLY otherwise, naming what would refresh it, and exits 0. It
# is the cheap partial, not the correct one: `just check-full` is the arm that
# is always right. Why it exists and what it does and does not buy are in
# docs/contributing/coverage-crap.md; it is wrapped in mmz like every other arm
# and the `lcov` scope that lets a skip be memoized without sticking is argued
# in .mmz/config.yaml.

[doc("Run the CRAP gate when the lcov is fresh, else skip loudly")]
[group("gate")]
check-crap:
    bash .just/scripts/check-crap.sh

# The committed ratchet baseline itself: every `file` in it must be repo-
# relative, and it must score something. Reads one JSON file and needs no
# lcov, so unlike check-crap above it never skips. What a machine-local path
# leaks, and why gitleaks cannot be the thing that catches it, are argued in
# .just/scripts/crap-baseline-check.sh.

[doc("Fail if the CRAP baseline is empty or carries machine-local paths")]
[group("gate")]
crap-baseline-check:
    bash .just/scripts/crap-baseline-check.sh

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

# Re-record ONE group after editing its source — `just outdatty-update <group>`
# (usage: just outdatty-update dev-docs). A recorded hash is the review
# watermark, so the blanket form (`just outdatty-update-all`) claims a review
# of every group's dependents that never happened; it stays the explicit
# choice, never the default
# (truth/tasks/scope-the-outdatty-update-recipe-to-a-group.typ).

[arg('GROUP', pattern='[a-z][a-z0-9-]*', help='outdatty group to re-confirm — the names outdatty.yaml declares')]
[doc("Re-confirm one dependency group: record its hashes into outdatty.lock")]
[group("dev")]
outdatty-update GROUP:
    outdatty update --group {{ GROUP }}

[doc("Re-confirm every dependency group: record all hashes into outdatty.lock")]
[group("dev")]
outdatty-update-all:
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

TOML_FMT_PATHS := ".gitleaks.toml .mindtape/config.toml examples/learn-piano/.mindtape/config.toml examples/large/.mindtape/config.toml"
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

# Counts PASSING tests, and the count is only authoritative when the suite is
# green: `cargo test --workspace` stops at the first failing test binary
# (fail-fast), so a red suite's sum is partial by construction, and summing
# field 4 counts passes, not tests. The deliberate shape is therefore to let
# the script-interpreter's pipefail fail loudly: a red suite still prints the
# partial sum (awk runs before `-e` fires) but the recipe exits with cargo's
# non-zero status, so a wrong number is never presented as a success — no
# `--no-fail-fast` and no PIPESTATUS salvage, both of which would only print a
# different wrong number and pretend the count survived
# (truth/tasks/adopt-the-script-attribute-for-the-shebang-recipes.typ).

[doc("Count tests across the workspace")]
[extension('.sh')]
[group("advisory")]
[script]
count-tests:
    cargo test --workspace 2>&1 | grep "test result:" | awk '{sum += $4} END {print sum " tests"}'

# `awk 'NR <= 20'` instead of `head -20` is deliberate: `head` exits at line
# 20 and the upstream `sort` then takes SIGPIPE, which pipefail reports as
# exit 141 on a perfectly good run. awk consumes the whole stream and prints
# the first 20 lines, so nothing upstream dies and real failures stay visible
# (truth/tasks/adopt-the-script-attribute-for-the-shebang-recipes.typ).

[doc("Show the top 20 files by line count")]
[extension('.sh')]
[group("advisory")]
[script]
file-sizes:
    find . -type f \( -name '*.rs' -o -name '*.md' \) ! -path './target/*' -exec wc -l {} + | sort -rn | awk 'NR <= 20'

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

# Stand up a worktree for a delegated agent: a new branch off BASE (default
# HEAD) at `../<repo>-<slug>`, `direnv allow`-ed, with the base read back out of
# the created worktree and compared to the sha it was created from. Refuses a
# dirty HEAD (`git worktree add` copies commits, never the working tree), and
# reports — without refusing — a base behind or diverged from this checkout.
#
# Deliberately warms nothing: what was measured about `kache`, `target/` and the
# cold `.mmz/cache/`, what the assertion can and cannot prove, and why the
# worktree is a sibling rather than a child are argued in place in
# .just/scripts/fork.sh.

[arg('SLUG', pattern='[a-z0-9][a-z0-9-]*', help='branch name, and the `../<repo>-<slug>` directory suffix')]
[arg('BASE', pattern='[A-Za-z0-9._/~^{}-]+', help='commit-ish to fork from — a branch, tag, sha or HEAD~n (default HEAD)')]
[doc("Stand up a ready worktree on a new branch off BASE")]
[group("dev")]
fork SLUG BASE='HEAD':
    bash .just/scripts/fork.sh '{{ SLUG }}' '{{ BASE }}'

[doc("Install the mindtape Typst library for local #import (dev symlink)")]
[group("dev")]
install-lib:
    bash .just/scripts/install-lib.sh

[doc("Remove the dev symlink installed by install-lib")]
[extension('.sh')]
[group("dev")]
[script]
uninstall-lib:
    VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' lib/typst.toml | head -1)
    TARGET="$HOME/.local/share/typst/packages/local/mindtape/$VERSION"
    [ -L "$TARGET" ] && rm "$TARGET" && echo "removed $TARGET" || echo "no symlink at $TARGET"

# Point git at the tracked hook dir (pre-commit asserts the memoized gate via
# `mmz --is-fresh --tag gate`, then checks task-file formatting and structure)

[doc("Point git at the tracked hook directory")]
[extension('.sh')]
[group("dev")]
[script]
install-hooks:
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
