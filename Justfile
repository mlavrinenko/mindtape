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
check: (mmz "fmt-check") (mmz "clippy") (mmz "test") (mmz "machete") (mmz "check-file-size") (mmz "check-dry") (mmz "check-secrets") (mmz "outdatty-check") (mmz "check-examples") (mmz "check-doc-coverage") (mmz "check-doc-facts") (mmz "check-doc-recipes") (mmz "check-doc-spans") (mmz "check-doc-prose") (mmz "check-doc-blocks") (mmz "docs::check") (mmz "docs::md-check") (mmz "fmt-fixture-check") (mmz "arg-pattern-check") (mmz "positional-args-check") (mmz "script-attr-check") (mmz "size-fixture-check") (mmz "script-scope-check") (mmz "site-pages-fixture-check") (mmz "check-crap") (mmz "crap-baseline-check") (mmz "check-gate-drift")

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
    tombi format --check --quiet {{ TOML_FMT_PATHS }}
    typstyle --check {{ TYPST_FMT_PATHS }}
    just --fmt --check
    for f in {{ JUST_FMT_PATHS }}; do just --justfile "$f" --fmt --check; done

[doc("Run clippy only")]
[group("dev")]
[group("gate")]
clippy:
    cargo clippy --workspace --all-targets -q -- -D warnings

# The suite runs through cargo-nextest — one process per test, so a panicking
# or hanging test is isolated and in-process global state cannot leak between
# tests — with `--status-level fail`: a green run prints only the run header
# and the summary line, a red one prints every failure's full output. It is
# deliberately the only runner: `just cover` measures the same one
# (.just/scripts/cover.sh), and CI's Windows job installs it too (ci.yml), so
# a test passing under one execution model while failing under another cannot
# go unnoticed.
#
# nextest runs no doctests. The extra `cargo test --doc` pass cratemplate
# carries for them is absent here on purpose: `cargo test --doc` reports 0
# tests in both crates, and no ```rust fence exists in the tree. A doctest
#
# `--cargo-quiet` twice drops cargo's own `Compiling …`/`Finished …` build
# noise (specify-twice is nextest's own spelling for "no Cargo output at
# all"); nextest's run banner and Summary line have no such flag — verified
# against `--status-level none` and `--final-status-level none`, neither
# touches them — so they still print even on an all-pass run. Kept as a
# single body line (bare comments above, not inside, the recipe) so
# www/gates.jq's `gate_command` keeps quoting it verbatim in CONTRIBUTING.md
# instead of falling back to the `[doc(...)]` string.

[doc("Run tests only")]
[group("dev")]
[group("gate")]
[positional-arguments]
test *ARGS:
    cargo nextest run --workspace --status-level fail --cargo-quiet --cargo-quiet "$@"

# cargo-machete has no quiet flag (verified: `cargo machete --help` lists
# none) — "Analyzing dependencies…" and "…Good job!" print on every clean run
# with no way to silence just the success case. Left as a bare one-liner so
# www/gates.jq's `gate_command` still quotes it verbatim in CONTRIBUTING.md.

[doc("Check for unused dependencies")]
[group("gate")]
machete:
    cargo machete

[doc("Fail if any file exceeds its linecop cap")]
[group("gate")]
check-file-size:
    linecop --format paths

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
    #
    # `-r xcode` swaps the default console reporter (a table plus tips even
    # on a clean run) for one compiler-warning line per clone, naming the
    # file and span — jscpd has no flag that stays silent on a clean run
    # only: `-s`/`--silent` replaces ALL of this with one unconditional
    # summary line on every run, success or failure, and drops the location
    # a real finding needs. `-r xcode` still prints "Found 0 clones." and a
    # timing line on a clean run (verified: no reporter or flag combination
    # removes those two, jscpd 1.x), so the output is captured and replayed
    # only on failure: a clean run prints nothing, a finding keeps its
    # file:line intact.
    out=$(jscpd --no-tips -k 70 -f rust --exit-code 1 -r xcode crates 2>&1) || { printf '%s\n' "$out"; exit 1; }

# Scan the working tree for a committed credential with gitleaks, under
# `.gitleaks.toml` (the default ruleset plus this repo's allowlist, argued in
# that file), bounded by what git ignores. Current tree only — history is deliberately out of
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
    outdatty check --format paths

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
# One `--tests` build, then the binary itself: `cargo run` cannot take
# `--tests`, and a plain `cargo run` relinks `target/debug/mt` to the build
# without the dev-dependency features the test arm links it with — the race
# docs/contributing/gates.md ("One mt binary, one feature set") argues.

[doc("Validate each example project against itself")]
[group("gate")]
check-examples:
    cargo build -p mindtape-cli --tests -q
    for ex in learn-piano minimal links estimates dsl; do MINDTAPE_DATA_HOME= ./target/debug/mt -w "examples/$ex" check -q || exit 1; done

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

[doc("Fail if a derived crate, DSL field, framework, gate, or sweep flag fact has no doc entry")]
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

# Fails when a doc source tells history or shouts in all-caps words. The word
# lists, and what the scan skips, are in .just/scripts/check-doc-prose.sh.

[doc("Fail on history words or all-caps emphasis in the doc sources")]
[group("gate")]
check-doc-prose:
    bash .just/scripts/check-doc-prose.sh

# A code block in a doc source comes from a checked fixture, a capture or a
# derived fact. What counts as typed by hand, the marker for a block no
# fixture can hold, and the list of sources not yet converted are in
# .just/scripts/check-doc-blocks.sh.

[doc("Fail on a code block typed by hand into a doc source")]
[group("gate")]
check-doc-blocks:
    bash .just/scripts/check-doc-blocks.sh

# An inline code span naming an `mt` command, flag or config key must name a
# real one. The shapes it reads and the control it runs first are argued in
# .just/scripts/check-doc-spans.sh.

[doc("Fail if a doc code span names an unknown mt command, flag or config key")]
[group("gate")]
check-doc-spans:
    bash .just/scripts/check-doc-spans.sh

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

# Two hand-kept facts, read from their source: TOML_FMT_PATHS against the TOML
# files git knows (a new untracked one counts), and every gate-reachable
# `cargo build`/`cargo run` of mindtape-cli against `--tests`, the flag that
# keeps target/debug/mt one build for the test arm. Why `check-rrule` is out
# of scope, how the reach is derived rather than listed, and the control that
# runs first are argued in place in .just/scripts/check-gate-drift.sh.

[doc("Fail if TOML_FMT_PATHS drifts from git, or a gate builds mt without --tests")]
[group("gate")]
check-gate-drift:
    bash .just/scripts/check-gate-drift.sh

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
    cargo build --release -p mindtape-cli
    examples/large/generate.sh {{ N }}
    export MINDTAPE_NOW=2026-07-16 && hyperfine \
        --warmup 1 --runs 3 \
        --command-name "mt ls ({{ N }} tasks)" "./target/release/mt -w examples/large ls" \
        --command-name "mt check ({{ N }} tasks)" "./target/release/mt -w examples/large check"

# Check the `rrule(...)` escape hatch's expansion engine: `just check` builds
# with default features, so it never compiles or runs this code (see
# crates/mindtape/src/recur_rrule.rs). Reproduces what CI runs for it. Runs
# through nextest, the same runner `just test` gates on.
#
# Both crates, not only `mindtape`: `mt init`'s recurrence axis hides its
# `rrule` choice from a build without the engine, so mindtape-cli carries
# `cfg(feature = "rrule")` arms of its own. Lint and run one crate here and the
# other's feature-on arms compile nowhere — the same rot this recipe exists to
# stop, one crate over.
#
# examples/recurring/'s `rrule(...)` task is checked here, never in
# check-examples: that recipe runs in parallel with `just test`, and a feature
# build and the default-feature test arm both write target/debug/mt — the
# feature-on binary made `mt init --recurrence rrule` succeed in a test that
# expects the refusal (measured; the test is init_e2e's
# init_refuses_the_rrule_answer_without_the_engine). The engine's example
# shares the engine's recipe.

[doc("Check the rrule feature build, lint, and tests")]
[group("dev")]
check-rrule:
    cargo fmt --all -- --check
    cargo clippy --workspace --features mindtape/rrule,mindtape-cli/rrule --all-targets -q -- -D warnings
    cargo nextest run --workspace --features mindtape/rrule,mindtape-cli/rrule --status-level fail
    MINDTAPE_DATA_HOME= cargo run -p mindtape-cli -q --features rrule -- -w examples/recurring check -q

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
#
# After a dependent-only edit, name the file too — `just outdatty-update
# dev-docs docs/src/contributing/windows.typ` records that dependent's hash
# alone and leaves every other pending dependent in the group pending
# (truth/tasks/record-one-dependents-watermark-without-claiming-the-groups.typ).

[arg('GROUP', pattern='[a-z][a-z0-9-]*', help='outdatty group to re-confirm — the names outdatty.yaml declares')]
[arg('DEPENDENTS', pattern='[A-Za-z0-9._/-]+', help='record only these dependents of GROUP, leaving the rest pending')]
[doc("Re-confirm one dependency group, or only the named dependents in it, into outdatty.lock")]
[group("dev")]
outdatty-update GROUP *DEPENDENTS:
    outdatty update --group {{ GROUP }} {{ prepend('--dependent ', DEPENDENTS) }}

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

# The dev shell's `mt`, at the path the shell hook puts on PATH. Its own target
# dir keeps the build off the lock `just check` holds on `target/` for minutes,
# and off `target/debug/mt`, which the rrule arm builds with a feature this
# default build would silently relink away. Called at the events that
# invalidate the binary — this recipe, `just fork`, `just integrate` — never
# per `mt` call.

[doc("Build the dev shell's mt")]
[group("dev")]
build-mt:
    cargo build -q -p mindtape-cli --bin mt --target-dir target/mt-dev

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

CRAP_EXCLUDES := "--exclude **/main.rs --exclude **/recur_rrule.rs --exclude **/ls_interactive/tui.rs --exclude **/*_tests.rs --exclude **/tests.rs --exclude **/*_tests_*.rs"

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
    bash .just/scripts/crap.sh all {{ CRAP_EXCLUDES }}

# Re-mint crap-baseline.json after a DELIBERATE complexity increase. Run it in
# the same commit as the change that justifies it, so the diff shows the new
# number next to the reason for it — a ratchet with no documented release valve
# gets bypassed instead of refreshed.
#
# The pre-mint guard is the THRESHOLD arm, not the whole gate: nothing above 30
# may be minted in as acceptable, but the ratchet arm is the file being
# rewritten and a deliberate increase makes it red by definition. Depending on
# the whole gate made this recipe refuse in its own documented use case; the
# split is argued in .just/scripts/crap.sh, and crap-baseline.sh runs the arm.

[doc("Re-mint crap-baseline.json after a deliberate complexity increase")]
[group("advisory")]
crap-baseline:
    bash .just/scripts/crap-baseline.sh {{ CRAP_EXCLUDES }}

# Pre-PR mutation check — the daily driver (see CONTRIBUTING.md "Testing").
# Sweeps only mutants touched by this branch's diff against main, so it needs
# no state file (nothing to invalidate) and covers new files for free (every
# line of a new file is "touched"). Extra args pass through to cargo-mutants;
# results land in `target/mutants-scoped`, never in `mutants.out`, whose
# `--iterate` cache only a full sweep reads. What a surviving mutant means,
# and when a verdict is worth believing, are in
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
# from .cargo/mutants.toml. Same speed knobs as `mutants-diff`. Results go to
# `mutants.out` for a full sweep (the `--iterate` cache) and to
# `target/mutants-scoped` for a subset run, so a scoped sweep cannot clobber
# the cache; `-o` overrides either default.
#
# The TIMEOUT caveat, what a REPRODUCIBLE timeout means, what to do with a
# survivor inside `#[cfg(not(feature = …))]` code, and why there is
# deliberately no CI mutation job are in docs/contributing/mutation-testing.md.

[doc("Run a full-file mutation testing sweep")]
[group("advisory")]
[positional-arguments]
mutants *ARGS:
    bash .just/scripts/mutants.sh "$@"

# Progress readout for a sweep, running or finished — cargo-mutants prints no
# ETA, so it is derived from the two files the run writes as it goes: `mutants.json`
# (the fixed list this run was told to test) and `outcomes.json` (one entry per
# finished mutant, plus the opening `Baseline` entry). The single argument is the
# directory holding those two, `mutants.out` by default — pass
# `/tmp/ws-verify/mutants.out` for a `-o` run. Needs jq >= 1.7 for `localtime`;
# on older jq, swap it for `todate` and read the clock as UTC.

[doc("Report progress and ETA of a mutants sweep")]
[extension('.sh')]
[group("advisory")]
[positional-arguments]
[script]
mutants-status +DIR='mutants.out':
    dir="$1"
    if [ ! -f "$dir/outcomes.json" ]; then
        echo "no $dir/outcomes.json — no sweep has written there"
        exit 1
    fi
    # A finished run's elapsed is its own wall time, not the clock since it
    # started, or a `done` line reports hours the run did not take.
    jq -nr \
      --slurpfile o "$dir/outcomes.json" \
      --slurpfile m "$dir/mutants.json" '
      def epoch: sub("\\.[0-9]+Z$"; "Z") | fromdateiso8601;
      ($m[0] | length) as $total
    | ($o[0].outcomes | length - 1) as $done
    | (if $o[0].end_time != null then ($o[0].end_time | epoch) else now end) as $end
    | ($end - ($o[0].start_time | epoch)) as $el
    | (if $done > 0 and $el > 0 then $done / $el else 0 end) as $rate
    | (if $rate > 0 then ($total - $done) / $rate else null end) as $eta
    | "\($done)/\($total)  \($el/60|floor)min elapsed  \((($rate*60)*100|round)/100)/min  "
      + (if $o[0].end_time != null or $eta == null or $eta <= 0 then "done"
         else "ETA \($eta/60|floor)min -> \((now + $eta) | localtime | strftime("%H:%M"))"
         end)
    '

# What the non-Rust formatters cover. One list each, read by both `fmt` and
# `fmt-check`, because two hand-maintained copies drift: `.mindtape/templates`
# (the project's own `mt add` templates) sat outside both lists while the
# learn-piano example's templates sat inside, so the example project's
# templates were formatted and the real ones were not. A single list cannot
# omit a path from the check while applying it in the fix.

TOML_FMT_PATHS := ".cargo/mutants.toml .config/nextest.toml .gitleaks.toml .mindtape/config.toml Cargo.toml clippy.toml crates/mindtape-cli/Cargo.toml crates/mindtape-cli/src/axes/headers_spec.toml crates/mindtape-cli/src/axes/workflow_flow.toml crates/mindtape-cli/src/axes/workflow_gated.toml crates/mindtape-cli/src/axes/workflow_personal.toml crates/mindtape-cli/src/axes/workflow_todo_done.toml crates/mindtape-cli/src/presets/flow/config.toml crates/mindtape-cli/src/presets/gated/config.toml crates/mindtape-cli/src/presets/personal/config.toml crates/mindtape-cli/src/presets/todone/config.toml crates/mindtape/Cargo.toml crates/mindtape/src/defaults.toml crates/mindtape/src/frameworks.toml crates/mindtape/src/packs/mindtape-tasks.toml examples/api-service/.mindtape/config.toml examples/dsl/.mindtape/config.toml examples/estimates/.mindtape/config.toml examples/large/.mindtape/config.toml examples/learn-piano/.mindtape/config.toml examples/links/.mindtape/config.toml examples/minimal/.mindtape/config.toml examples/recurring/.mindtape/config.toml examples/self-validating/.mindtape/config.toml lib/typst.toml rust-toolchain.toml rustfmt.toml tombi.toml www/tola.toml"
TYPST_FMT_PATHS := "lib truth .mindtape/templates examples/api-service/tasks examples/api-service/.mindtape/templates examples/learn-piano/tasks examples/learn-piano/.mindtape/templates examples/links/tasks examples/recurring/tasks examples/dsl/tasks www/content www/templates www/utils docs/src"

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

# Counts the suite `just test` gates on, WITHOUT running it: `cargo nextest
# list` reports every test nextest sees, and dropping the `#[ignore]`d ones
# (which `just test` never runs) matches the count a green full run would
# pass. The old body summed `cargo test`'s "test result:" lines and only made
# sense when the suite was green, and nextest prints no such line — swapping
# the command alone would have grep'd nothing and piped a bogus "0 tests" past
# pipefail's silence. Listing is the shape that stays honest without paying a
# full run; under the global pipefail shell, a listing that fails to build
# takes the recipe down with it, so a count scraped from a broken run is never
# presented as a success.

[doc("Count tests across the workspace")]
[extension('.sh')]
[group("advisory")]
[script]
count-tests:
    cargo nextest list --workspace --message-format json \
        | jq -r '[.["rust-suites"][].testcases[] | select(.ignored == false)] | "\(length) tests"'

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

# The doc SOURCES the audit scans: site prose, the hand-written notes behind
# its generated refs, and the Markdown sources. Never the renders — README.md,
# AGENTS.md and skills/mindtape/SKILL.md are generated from docs/src/*.typ, so
# scanning both sides of a render would pair every sentence with itself.

DOC_PROSE_PATHS := "www/content/*.typ www/utils/*.typ docs/src/readme.typ docs/src/agents.typ docs/src/skill.typ skills/mindtape/references/*.md"

# Report near-duplicate prose sentences across the doc sources, worst pair
# first. Advisory, not a gate: the threshold is a judgement call and the report
# is an input to a human deciding where a fact lives. The algorithm, the knobs
# and why it is jq rather than a language of its own are in
# contrib/dup-prose.jq's header; run it before and after a doc refactor.

[doc("Report near-duplicate prose across the doc sources")]
[group("advisory")]
dup-prose:
    @for f in {{ DOC_PROSE_PATHS }}; do jq -n --arg f "$f" --rawfile t "$f" '{file:$f,text:$t}'; done | jq -s -r -f contrib/dup-prose.jq

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
[no-exit-message]
fork SLUG BASE='HEAD':
    bash .just/scripts/fork.sh '{{ SLUG }}' '{{ BASE }}'

# Take a delegated worktree back down, and `fork`'s inverse in more than name:
# it refuses unless BRANCH's own worktree reports `mmz --is-fresh --tag gate`,
# then fast-forwards this checkout onto it and removes the worktree. That
# assertion is the whole recipe — a fast-forward writes no commit, so the
# pre-commit hook never fires on the merge, and a branch that scoped its gate
# per-commit has nowhere else to pay the full aggregate.
#
# Also refuses a dirty worktree and a branch that is not a fast-forward; it
# never runs the gate, and there is deliberately no waiver flag. Why the
# assertion is read from the worktree rather than here, and what the missing
# flag buys, are argued in place in .just/scripts/integrate.sh.

[arg('BRANCH', pattern='[A-Za-z0-9._/-]+', help='branch to fast-forward onto — the one `just fork` created')]
[doc("Assert a branch's gate, fast-forward onto it, drop its worktree")]
[group("dev")]
[no-exit-message]
integrate BRANCH:
    bash .just/scripts/integrate.sh '{{ BRANCH }}'

[doc("Build mt in release mode and install it into ~/.cargo/bin")]
[group("dev")]
install-bin:
    bash .just/scripts/install-bin.sh

[doc("Install everything this checkout provides: mt, the Typst lib, schema, skill")]
[group("dev")]
install: install-bin install-lib
    bash .just/scripts/install.sh

[doc("Install the mindtape Typst library for local #import (dev symlink)")]
[group("dev")]
install-lib:
    bash .just/scripts/install-lib.sh

[doc("Remove the dev symlink installed by install-lib")]
[group("dev")]
uninstall-lib:
    bash .just/scripts/uninstall-lib.sh

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
