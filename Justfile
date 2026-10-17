# Mindtape's root justfile. `just` with no arguments lists every recipe.
#
# A comment about a recipe goes in one of three places:
#
#   - Here: what a caller needs before running it — what it does, when to
#     reach for it, what it refuses to do. The `[doc(...)]` string plus at
#     most a few lines.
#   - In the script (.just/scripts/*.sh): why the code is written the way it
#     is, next to the line it explains.
#   - In docs/src/contributing/*.typ: the arguments — what was measured, what
#     was tried, why a band or a job does not exist. `just docs::md` renders
#     those to docs/contributing/*.md.
#
# `just --list` shows only the last comment line of a recipe, so a summary
# worth reading there is repeated as an explicit `[doc(...)]` rather than left
# as a dangling fragment. Leave a blank line between a comment block and a
# recipe's `[doc(...)]`: just 1.46's `--fmt` deletes the comment line right
# above it.

set quiet := true
set unstable := true

# Every `[script]` recipe runs under this interpreter, so each gets bash
# strict mode (`-e` plus `pipefail`) without a `set -eo pipefail` of its own.

set script-interpreter := ["bash", "-eo", "pipefail"]

# Six recipes live in .just/docs.just rather than here: docs::gen, docs::build,
# docs::serve and docs::md (render docs/src/*.typ to the repo-root Markdown),
# plus the two gates — docs::check (build, index with Pagefind, then validate
# internal links + assets) and docs::md-check (the docs-md freshness gate).
# `just docs::<name>` and `just docs <name>` both work.
#
# .just/docs.just's header argues why the module path is spelled out, why
# `docs::md` reaches doc prose as `docs-md`, and why the module file carries
# a `set working-directory := '..'` of its own.

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
# declared inputs are unchanged. What the memoization buys, why a pass prints
# only its verdict, and how `mt flip` asserts the whole set fresh without
# running any of it are in docs/contributing/gates.md.
#
# The arms run in three stages, in order, and the first red stage stops the
# run. The fast stage needs no `mt` build, so a lint or prose error shows in
# seconds. The build stage holds every arm that builds or runs `mt`; they
# share one `cargo build --tests`. The coverage stage pays an instrumented
# clean rebuild, so it runs only on a tree every other gate passed.
#
# A pass prints one verdict line. A failure prints each failed gate's full log
# and a verdict naming them. Every arm's log stays under target/check/.
# Stage names (`just check fast build`) run only those stages, in their
# order. CI runs `cover` in a job of its own beside `fast build`.

[arg('STAGES', pattern='fast|build|cover', help='stages to run (fast, build, cover); all when none')]
[doc("Run every check gate, fast stage first; print only failures and a verdict")]
[group("workflow")]
[no-exit-message]
check *STAGES:
    bash .just/scripts/check.sh {{ STAGES }}

# The stages in order, which check.sh runs with its output captured. A failing
# stage stops the rest. www/gates.jq reads the gate list from here.

[doc("The check stages in order")]
[private]
_check-stages: _check-fast _check-build _check-cover

[doc("The check gates that build nothing")]
[parallel]
[private]
_check-fast: (mmz "fmt-check") (mmz "clippy") (mmz "machete") (mmz "check-workflows") (mmz "shellcheck") (mmz "check-file-size") (mmz "check-dry") (mmz "check-secrets") (mmz "outdatty-check") (mmz "check-doc-recipes") (mmz "check-doc-blocks") (mmz "fmt-fixture-check") (mmz "outdatty-review-trailer-fixture-check") (mmz "arg-pattern-check") (mmz "positional-args-check") (mmz "script-attr-check") (mmz "path-attr-check") (mmz "plural-check") (mmz "size-fixture-check") (mmz "script-scope-check") (mmz "crap-baseline-check") (mmz "check-gate-drift")

[doc("The check gates that build or run mt")]
[parallel]
[private]
_check-build: (mmz "test") (mmz "check-examples") (mmz "check-doc-coverage") (mmz "check-doc-facts") (mmz "check-doc-spans") (mmz "check-doc-prose") (mmz "docs::check") (mmz "docs::md-check") (mmz "site-pages-fixture-check")

[doc("Coverage, then the CRAP gate over it")]
[private]
_check-cover: (mmz "cover") (mmz "crap")

# Wrap a just recipe in mmz: `just mmz fmt-check` runs `mmz just fmt-check`.
# Under `just check` it captures the output to target/check/ instead of
# streaming it. Why the double hop, and why `just test <filter>` deliberately
# bypasses it, are in docs/contributing/gates.md.

[doc("Run a just recipe through mmz memoization")]
[group("workflow")]
[no-exit-message]
mmz *ARGS:
    bash .just/scripts/mmz-arm.sh {{ ARGS }}

# Delegated work lives in a sibling worktree on its own branch. Project config
# cannot set worktrunk's merge defaults, so the flags that keep them sane live
# in `fork` and `land`, and a bare `wt switch` or `wt merge` is not how work
# starts or ends here. Extra arguments reach `wt`: `just fork x --base=@` branches
# from the current commit instead of the default branch. The hooks that make the
# worktree work (`mt` built, gate, wip check, record carry) are .config/wt.toml.
# `--no-cd` because a recipe cannot move its caller's shell anyway, and wt
# warns three lines about missing shell integration every time it tries.
# `wt merge` has no `--no-cd`, so `land` sends its cd directive to /dev/null.

[doc("Fork a sibling worktree on a new branch, with its dev mt built")]
[group("workflow")]
[positional-arguments]
fork slug *args:
    wt switch --create --yes --no-cd "$@"

# Run in the worktree. `--no-squash --no-commit` keeps each commit's `Refs:`
# footer and refuses a dirty tree instead of committing it; the `pre-merge`
# hooks run the gate and the wip check before the fast-forward.

[doc("Gate, fast-forward the branch onto main and remove its worktree")]
[group("workflow")]
[positional-arguments]
land *args:
    WORKTRUNK_DIRECTIVE_CD_FILE=/dev/null wt merge --no-squash --no-commit --yes "$@"

# `just --fmt --check` prints its diff with the whole file as context, about 900
# lines for one misplaced blank line. `just --dump` prints what `--fmt` would
# write, so diffing it gives the same verdict with three lines of context.

[doc("Check formatting without writing changes (CI-friendly)")]
[group("gate")]
fmt-check:
    cargo fmt --all -- --check
    tombi format --check --quiet {{ TOML_FMT_PATHS }}
    typstyle --check {{ TYPST_FMT_PATHS }}
    just --dump | diff -u Justfile -
    for f in {{ JUST_FMT_PATHS }}; do just --justfile "$f" --dump | diff -u "$f" -; done

[doc("Run clippy only")]
[group("dev")]
[group("gate")]
[script]
clippy:
    # A ban whose path resolves to nothing only warns, and `-D warnings` leaves
    # that warning alone, so the ban would silently ban nothing. Fail on it.
    rc=0
    out=$(cargo clippy --workspace --all-targets -q -- -D warnings 2>&1) || rc=$?
    [ -z "$out" ] || printf '%s\n' "$out" >&2
    if grep -q 'does not refer to a reachable' <<<"$out"; then
        echo "clippy.toml names a disallowed path that resolves to nothing" >&2
        exit 1
    fi
    exit $rc

# The suite runs through cargo-nextest: one process per test, so a panicking
# or hanging test is isolated and in-process global state cannot leak between
# tests. With `--status-level fail` a green run prints only the run header and
# the summary line, and a red one prints every failure's full output. It is the
# only runner. `just cover` measures the same one (.just/scripts/cover.sh) and
# CI's Windows job installs it too (ci.yml), so a test cannot pass under one
# execution model while failing under another.
#
# nextest runs no doctests, and this recipe has no `cargo test --doc` pass,
# because `cargo test --doc` reports 0 tests in both crates and the tree has no
# `rust` fenced block. A doctest added later needs a `cargo test --doc` pass of
# its own.
#
# `--cargo-quiet` drops cargo's own `Compiling …`/`Finished …` build output
# and keeps rustc's diagnostics, so a red test that does not compile says why.
# Given twice it drops all Cargo output, errors included, and a build failure
# prints only the cargo command line.
# nextest's run banner and summary line have no such flag, so they print even
# on an all-pass run (verified: `--status-level none` and
# `--final-status-level none` do not touch them). `just check` captures them
# and prints them only when the arm fails.

[doc("Run tests only")]
[group("dev")]
[group("gate")]
[positional-arguments]
test *ARGS:
    cargo nextest run --workspace --status-level fail --cargo-quiet "$@"

# cargo-machete has no quiet flag (verified: `cargo machete --help` lists
# none), so "Analyzing dependencies…" and "…Good job!" print on every clean
# run. `just check` captures them and prints them only when the arm fails.

[doc("Check for unused dependencies")]
[group("gate")]
machete:
    cargo machete

# A workflow runs only once pushed, so this is the one check of it before
# then: expressions, job graph, action inputs, and shellcheck on each `run:`.

[doc("Lint the GitHub workflows")]
[group("gate")]
check-workflows:
    actionlint

# A syntax error or quoting bug in a script shows only when something runs it:
# git for a hook, a recipe for a script. shellcheck reads every tracked file
# whose first line is a shell shebang, so a script without a `.sh` extension
# (`.githooks/*`, `examples/*/scripts/*`) is linted too. The `.shellcheckrc`
# beside the Justfile holds the options. A clean run prints nothing.

[doc("Lint every tracked shell script with shellcheck")]
[group("gate")]
[script]
shellcheck:
    git ls-files -z | xargs -0 awk 'FNR == 1 && /^#!.*[\/ ](ba)?sh( |$)/ { print FILENAME } { nextfile }' | xargs -r -d '\n' shellcheck

[doc("Fail if any file exceeds its linecop cap")]
[group("gate")]
check-file-size:
    linecop

# What jscpd counts as a clone, for check-dry and dry-baseline alike: blocks of
# >=50 tokens that match with their names and literals blanked, plus functions
# whose normalized syntax trees share 80% of their subtrees. Tests included.

DRY_FLAGS := "--no-tips -f rust -k 50 --ignore-identifiers --ignore-literals --similarity=0.8 --ignore '**/tests/it/main.rs' --baseline .jscpd-baseline.json"

[doc("Copy-paste gate: fail on a duplicated Rust block the baseline lacks")]
[extension('.sh')]
[group("gate")]
[script]
check-dry:
    # jscpd (from qahq) fails on any clone .jscpd-baseline.json does not list,
    # naming both spans. The baseline holds the clones that predate the gate,
    # by a fingerprint of their raw tokens, so an edit inside one makes it
    # new. Fix a finding by extracting a shared helper, not by shuffling
    # tokens until the detector stops matching. `just dry-baseline` re-mints
    # the baseline, in plain sight in the diff.
    #
    # Under a baseline every jscpd reporter prints all clones, old ones
    # included. So the gate reads the json report and prints only the new
    # ones, as file:line pairs. A clean run prints nothing.
    out=$(mktemp -d)
    trap 'rm -rf "$out"' EXIT
    log=$(jscpd {{ DRY_FLAGS }} -r json -o "$out" crates 2>&1) || { printf '%s\n' "$log"; exit 1; }
    new=$(jq -r '.duplicates[] | select(.isNew)
        | "crates/\(.firstFile.name):\(.firstFile.start): \(.kind) clone of crates/\(.secondFile.name):\(.secondFile.start)"' \
        "$out/jscpd-report.json")
    [ -z "$new" ] || { printf '%s\n' "$new"; exit 1; }

# Scan the working tree for a committed credential with gitleaks, under
# `.gitleaks.toml` (the default ruleset plus this repo's allowlist, argued in
# that file), bounded by what git ignores. It scans the current tree only;
# history is out of scope. A control fixture runs first, because "no leaks
# found" is what an
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

# Checks each fixture another tool runs `mt` against, so the fixture cannot go
# stale: api-service feeds the docs generator and the skill's triage evals,
# minimal feeds the skill's trap evals. Why a fixture must be gated before
# another tool depends on it is in docs/contributing/example-projects.md.
#
# Each fixture is checked against its own project. It declares a
# `[[typst.local-packages]]` entry pointing `@local/mindtape` at this
# checkout's lib/. Without the entry, the import resolves from the
# machine-global local-package directory, which holds whichever checkout last
# ran `just install-lib`, so a fixture that lost its entry would keep passing
# against another tree's prelude. An empty `MINDTAPE_DATA_HOME` removes that
# fallback: set but empty, the variable means "no such directory", so no
# global package resolves. A fork editing the DSL then gates against the
# prelude it is editing. A contributor, and CI, which never runs `mt install`,
# can run this gate on a machine with no global install at all.
#
# The gate uses that variable and not XDG_DATA_HOME because `dirs` reads the
# environment on Unix only. XDG_DATA_HOME would isolate this gate on Linux,
# leave macOS reading the machine's packages, and mean nothing on Windows.
# `mt` resolves the local-package directory through its own setting on every
# platform (crates/mindtape/src/loader/mod.rs).
#
# The recipe makes one `--tests` build, then runs the binary itself. `cargo
# run` cannot take `--tests`, and a plain `cargo run` relinks `target/debug/mt`
# to a build without the dev-dependency features the test arm links it with.
# docs/contributing/gates.md ("One mt binary, one feature set") describes that
# race.

[doc("Validate each example project against itself")]
[group("gate")]
check-examples:
    cargo build -p mindtape-cli --tests -q
    for ex in minimal links estimates dsl statuses rules api-service derived-tags round-trip named-sets priority recurring task-files across-projects/projects/shop across-projects/projects/blog; do MINDTAPE_DATA_HOME= ./target/debug/mt -w "examples/$ex" check -q || exit 1; done

# Every visible top-level `mt` subcommand must have a www/utils/cli-notes.typ
# entry and a www/content/cli/<command>.typ page. How the set-difference is
# taken, and why it excludes nothing by hand, are argued in
# .just/scripts/check-doc-coverage.sh.

[doc("Fail if a visible mt subcommand has no CLI doc entry")]
[group("gate")]
check-doc-coverage:
    bash .just/scripts/check-doc-coverage.sh

# Every derived fact www/generate-facts.sh writes must have a hand-written
# www/utils/*-notes.typ entry. The arms, and why the gate regenerates its own
# inputs, are argued in .just/scripts/check-doc-facts.sh.

[doc("Fail if a derived crate, DSL field, framework, or sweep flag fact has no doc entry")]
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

# Fails when a doc source, a Justfile or YAML config comment, or a description
# in a schema tells history or shouts in all-caps words, when a Typst doc source
# types the product's name instead of `#mindtape`, or when site text, a schema
# description or a `mt _cli --json` help string holds an em dash. It builds `mt`
# for that dump. The word lists are in .just/scripts/_doc-prose-lib.sh, and what
# the scan skips is in .just/scripts/check-doc-prose.sh.

[doc("Fail on history words, all-caps emphasis, an em dash in site text or a typed product name in doc prose, config comments and schema descriptions")]
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
# and why the success case is asserted too, are argued in place in
# .just/scripts/fmt-fixture-check.sh.

[doc("Lock the formatter gates against a mis-formatted fixture")]
[group("gate")]
fmt-fixture-check:
    bash .just/scripts/fmt-fixture-check.sh

# Commit real commits in a temp repo whose commit-msg hook runs
# .just/scripts/outdatty-review-trailer.sh, and assert which ones it refuses: a
# re-record that edits no dependent and names no group in a `Docs-Unaffected:`
# trailer, under `--amend` too. The cases, and why each is a real commit, are
# in .just/scripts/outdatty-review-trailer-fixture-check.sh.

[doc("Lock the Docs-Unaffected trailer check against a fixture repo")]
[group("gate")]
outdatty-review-trailer-fixture-check:
    bash .just/scripts/outdatty-review-trailer-fixture-check.sh

# Lock the `[arg(...)]` patterns on release/bench/gen-bench/eject and the five
# `ask-refactoring` thresholds: each must reject a known-bad argument and
# accept a known-good one at dispatch, before the recipe body runs. No recipe
# body runs, `release` included; .just/scripts/arg-pattern-check.sh argues why.

[doc("Lock the arg-attribute patterns against known-bad and known-good input")]
[group("gate")]
arg-pattern-check:
    bash .just/scripts/arg-pattern-check.sh

# Hold the folder-per-area layout: a `#[path = "..."]` may name only a test
# sibling (`*_tests*.rs`, `tests_*.rs`, `tests.rs`).
# `self_named_module_files` in the workspace lints holds the other half. The
# script scans a synthesized stray first, so a scan that reaches nothing fails;
# .just/scripts/path-attr-check.sh argues the rest.

[doc("Refuse a #[path] attribute that names anything but a test sibling")]
[group("gate")]
path-attr-check:
    bash .just/scripts/path-attr-check.sh

# A count reads `1 task` or `12 tasks`: refuse `(s)` in a string literal in
# non-test Rust code under `crates/`. The script scans a synthesized fixture
# first, so a scan that reaches nothing fails; .just/scripts/plural-check.sh
# says what it reads as a string and as test code.

[doc("Refuse task(s) and the like in a string outside tests; use mindtape::count")]
[group("gate")]
plural-check:
    bash .just/scripts/plural-check.sh

# Lock the file-size gate's scan scope: an oversized shell script under
# `.just/scripts/` must be reported by `linecop` reading this repo's
# `.linecop.yaml`. The recipe tests the directory walker. A directory the scan
# never enters reports the same "All files within size limits." a clean tree
# does, so a script there stays uncapped with no signal. Why it runs against a
# replica tree instead of the real directory, and why the unreported case is
# asserted too, are argued in place in .just/scripts/size-fixture-check.sh.

[doc("Lock the size gate's scan scope against an oversized hidden-dir fixture")]
[group("gate")]
size-fixture-check:
    bash .just/scripts/size-fixture-check.sh

# Lock argument forwarding on the passthrough recipes (`test`, `build`,
# `mutants`, `mutants-diff`): each must carry `[positional-arguments]` and
# forward "$@" rather than interpolate `{{ ARGS }}`/`{{ args }}` into its body,
# because a bare interpolation re-splits an argument containing a space and
# re-globs one containing a wildcard. Why it is static, why it is separate from
# arg-pattern-check, and why `ask-refactoring`, which takes pattern-checked
# scalars, is outside it are argued in place in
# .just/scripts/positional-args-check.sh.

[doc("Lock positional-argument forwarding on the four passthrough recipes")]
[group("gate")]
positional-args-check:
    bash .just/scripts/positional-args-check.sh

# Lock bash strict mode in: no recipe body may open with a `#!` shebang (every
# script body goes through `[script]`, so `set script-interpreter` reaches it),
# and every justfile must declare a setting that actually aborts a failed
# pipeline. A `#!` recipe skips that setting and runs without strict mode, so a
# failed pipeline in it can exit 0. Why `.shebang` cannot be the discriminator,
# why the setting is proven by a probe instead of matched as text, and why
# every justfile needs its own are argued in place in
# .just/scripts/script-attr-check.sh.

[doc("Lock the script attribute and each justfile's strict-mode interpreter")]
[group("gate")]
script-attr-check:
    bash .just/scripts/script-attr-check.sh

# Lock both manifests that enumerate `.just/scripts/` onto the directory they
# enumerate: every script must be named by an mmz scope as a literal path, every
# memoized rule whose body invokes a script must name that script's scope, and
# every script must be a source of some `outdatty.yaml` group. A glob such as
# `.just/scripts/**` covers a new script as soon as the file exists, and an
# enumeration covers only what somebody listed, so this recipe checks each
# enumeration against the directory. Why a glob can never satisfy the first
# arm, which case the second cannot see, and why the third reads the config
# rather than the lock, are argued in place in
# .just/scripts/script-scope-check.sh.

[doc("Lock every gate script into a named mmz scope and outdatty's script groups")]
[group("gate")]
script-scope-check:
    bash .just/scripts/script-scope-check.sh

# Lock the docs manifest gate itself against three failures: two pages
# declaring the same route, a page with no `<page-meta>` block, and a page
# missing from www/utils/site.typ's `NAV`. A gate that never fires reads
# identical to one that cannot fire, so this recipe proves each failure is
# reported. Why it runs against a replica tree rather than the working one, and
# why the third case drives `.just/scripts/_nav-diff.sh` directly instead of the
# whole check-doc-facts regenerate, are argued in place in
# .just/scripts/site-pages-fixture-check.sh.

[doc("Lock the docs manifest gate against three known-bad fixtures")]
[group("gate")]
site-pages-fixture-check:
    bash .just/scripts/site-pages-fixture-check.sh

# The committed ratchet baseline itself: every `file` in it must be repo-
# relative, and it must score something. Reads one JSON file and needs no
# lcov. What a machine-local path leaks, and why gitleaks cannot be the thing
# that catches it, are argued in
# .just/scripts/crap-baseline-check.sh.

[doc("Fail if the CRAP baseline is empty or carries machine-local paths")]
[group("gate")]
crap-baseline-check:
    bash .just/scripts/crap-baseline-check.sh

# Two hand-kept facts, read from their source: TOML_FMT_PATHS against the TOML
# files git lists, tracked or not yet added, and every gate-reachable
# `cargo build`/`cargo run` of mindtape-cli against `--tests`, the flag that
# keeps target/debug/mt one build for the test arm. How the reach is derived
# rather than listed, and the control that runs first, are argued in place in
# .just/scripts/check-gate-drift.sh.

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

# Cross-check the workspace, tests included, against Windows without leaving
# Linux — the compile half of what the CI windows job proves at runtime, minutes
# earlier and without pushing. Reach for it when touching a `cfg(windows)` arm,
# a process spawn, a path join, or anything else that differs by platform. Runs in
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

# Re-record one group after editing its source: `just outdatty-update <group>`
# (usage: just outdatty-update dev-shell-docs). A recorded hash is the review
# watermark, so the blanket form (`just outdatty-update-all`) claims a review
# of every group's dependents that never happened. Use it only when that review
# did happen.
#
# After a dependent-only edit, name the file too: `just outdatty-update
# dev-shell-docs docs/src/contributing/windows.typ` records that dependent's hash
# alone and leaves every other pending dependent in the group pending.

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
# into the package build and breaking downstream flake consumers. Untracked files
# are marked intent-to-add first: flakes filter source to git-known files, and
# this makes new ones visible without staging any content. Edits and deletions
# stay unstaged; nix reads the working tree.

[doc("Verify nix build succeeds in a clean sandbox")]
[group("release")]
nix-build:
    git ls-files -z --others --exclude-standard | xargs -0 -r git add --intent-to-add --
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

# The dev shell's `mt`, at the path the shell hook puts on `PATH`. Its own target
# dir keeps the build off the lock `just check` holds on `target/` for minutes,
# and off `target/debug/mt`, which the test arm links with test features this
# plain build would relink away. Called at the events that
# invalidate the binary (this recipe, and the worktrunk hooks in
# .config/wt.toml: worktree creation, a landed merge), never per `mt` call.

[doc("Build the dev shell's mt")]
[group("dev")]
build-mt:
    cargo build -q -p mindtape-cli --bin mt --target-dir target/mt-dev

# Run coverage with cargo-llvm-cov (also writes target/coverage/lcov.info, the
# lcov `just crap` scores against). It always pays a full instrumented rebuild.
# Why that is not optional, and what leftover profraw files do to the numbers,
# are in docs/contributing/coverage-crap.md.

[doc("Run coverage with cargo-llvm-cov")]
[group("gate")]
cover:
    bash .just/scripts/cover.sh

# Why target/coverage/lcov.info is unusable, or empty output when it is fine.
# `crap` refuses to score when it prints a reason.

[doc("Print why the lcov is stale, or nothing if fresh")]
[private]
_lcov-stale-reason:
    bash .just/scripts/_lcov-stale-reason.sh

# What `crap` and `crap-baseline` both exclude. One list, two consumers, for the
# reason TYPST_FMT_PATHS exists: two hand-maintained copies drift, and a baseline
# minted over a different file set than the gate scores would report every
# difference between the two lists as a regression.

CRAP_EXCLUDES := "--exclude **/main.rs --exclude **/ls/interactive/tui.rs --exclude **/*_tests.rs --exclude **/tests.rs --exclude **/*_tests_*.rs --exclude **/tests_*.rs"

# Same list for a third consumer that is not a `just` recipe: the sweep's
# undertested arm (contrib/ask-refactoring/undertested.sh) runs `cargo crap` for
# its per-function complexity and coverage, and must score the same file set the
# gate does or it files findings about `main.rs`. A recipe rather than a copy,
# for the reason above.

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
[group("gate")]
crap:
    bash .just/scripts/crap.sh all {{ CRAP_EXCLUDES }}

# Re-mint crap-baseline.json after adding a function or a deliberate complexity
# increase. Run it in the same commit as the change that justifies it, so the
# diff shows the new number next to the reason for it. A ratchet with no documented release valve
# gets bypassed instead of refreshed.
#
# The pre-mint guard is the threshold arm only. Nothing above 30 may be minted
# in as acceptable. The ratchet arm compares against the file being rewritten,
# so a deliberate increase makes it red by definition, and a guard on the whole
# gate would refuse this recipe's own use. .just/scripts/crap.sh argues the
# split, and crap-baseline.sh runs the arm.

[doc("Re-mint crap-baseline.json after a new function or complexity rise")]
[group("advisory")]
crap-baseline:
    bash .just/scripts/crap-baseline.sh {{ CRAP_EXCLUDES }}

# A non-zero exit is the answer (9: a workaround can go), not a failure, so it
# prints no error line. canibump.toml lists what each workaround waits on.

[doc("Say which upstream change in canibump.toml landed, so its workaround can go")]
[group("advisory")]
[no-exit-message]
canibump *ARGS:
    canibump --refresh {{ ARGS }}

[doc("Re-mint .jscpd-baseline.json after extracting a clone or accepting one")]
[group("advisory")]
dry-baseline:
    jscpd {{ DRY_FLAGS }} --update-baseline -r silent crates

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
# mutants) and the debuginfo-free `mutants` profile come from
# .cargo/mutants.toml. Same speed knobs as `mutants-diff`. Results go to
# `mutants.out` for a full sweep (the `--iterate` cache) and to
# `target/mutants-scoped` for a subset run, so a scoped sweep cannot clobber
# the cache; `-o` overrides either default.
#
# The timeout caveat, what a reproducible timeout means, and why there is
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
# `fmt-check`, because two hand-maintained copies drift and leave a path
# formatted by one recipe and skipped by the other. A single list cannot omit a
# path from the check while applying it in the fix.

TOML_FMT_PATHS := ".cargo/mutants.toml canibump.toml .config/nextest.toml .config/wt.toml .gitleaks.toml .mindtape/config.toml Cargo.toml clippy.toml crates/mindtape-cli/Cargo.toml crates/mindtape-cli/src/cmd/init/axes/headers_spec.toml crates/mindtape-cli/src/cmd/init/axes/workflow_flow.toml crates/mindtape-cli/src/cmd/init/axes/workflow_gated.toml crates/mindtape-cli/src/cmd/init/axes/workflow_personal.toml crates/mindtape-cli/src/cmd/init/axes/workflow_todo_done.toml crates/mindtape-cli/src/cmd/init/presets/flow/config.toml crates/mindtape-cli/src/cmd/init/presets/gated/config.toml crates/mindtape-cli/src/cmd/init/presets/personal/config.toml crates/mindtape-cli/src/cmd/init/presets/todo-done/config.toml crates/mindtape/Cargo.toml crates/mindtape/src/at/board.toml crates/mindtape/src/config/defaults.toml crates/mindtape/src/framework/frameworks.toml crates/mindtape/src/packs/mindtape-tasks.toml examples/across-projects/.config/mindtape/at.toml examples/across-projects/projects/blog/.mindtape/config.toml examples/across-projects/projects/shop/.mindtape/config.toml examples/api-service/.mindtape/config.toml examples/derived-tags/.mindtape/config.toml examples/dsl/.mindtape/config.toml examples/estimates/.mindtape/config.toml examples/large/.mindtape/config.toml examples/links/.mindtape/config.toml examples/minimal/.mindtape/config.toml examples/named-sets/.mindtape/config.toml examples/priority/.mindtape/config.toml examples/recurring/.mindtape/config.toml examples/round-trip/.mindtape/config.toml examples/rules/.mindtape/config.toml examples/self-validating/.mindtape/config.toml examples/statuses/.mindtape/config.toml examples/task-files/.mindtape/config.toml lib/typst.toml release.toml rust-toolchain.toml rustfmt.toml tombi.toml www/tola.toml"
TYPST_FMT_PATHS := "lib truth .mindtape/templates examples/api-service/tasks examples/api-service/.mindtape/templates examples/estimates/.mindtape/templates examples/self-validating/.mindtape/templates examples/named-sets/.mindtape/templates examples/derived-tags/api examples/derived-tags/web examples/links/tasks examples/named-sets/tasks examples/priority/tasks examples/recurring/tasks examples/round-trip/tasks examples/dsl/tasks examples/task-files/tasks examples/across-projects/projects/blog/tasks examples/across-projects/projects/shop/tasks examples/lsp examples/rules examples/statuses/tasks www/content www/templates www/utils docs/src"

# The same one-list rule for the justfiles themselves. A bare `just --fmt`
# formats the root justfile only and silently skips every `mod` file, so each
# module file needs its own `--justfile <file> --fmt` pass. The set of them
# lives here and is not spelled twice, so a third module file cannot land in
# the fix and miss the check. Every file listed needs `set unstable := true` of
# its own, because `--fmt` is unstable and reads that setting from the file it
# is pointed at, never from the Justfile that declares the `mod`.
#
# The paths must follow the module files. This list is the whole coverage of
# the formatter gate over them, so a module file that moves without its entry
# moving silently drops out of `just fmt` and `just fmt-check`. The
# verification behind that is in docs/contributing/gates.md.

JUST_FMT_PATHS := ".just/docs.just"

[doc("Format code")]
[group("dev")]
fmt:
    cargo fmt --all
    tombi format {{ TOML_FMT_PATHS }}
    typstyle --inplace {{ TYPST_FMT_PATHS }}
    just --fmt
    for f in {{ JUST_FMT_PATHS }}; do just --justfile "$f" --fmt; done

# Counts the suite `just test` gates on without running it: `cargo nextest
# list` reports every test nextest sees, and dropping the `#[ignore]`d ones
# (which `just test` never runs) matches the count a green full run would
# pass. Listing stays honest without paying a full run. Under the global
# pipefail shell, a listing that fails to build takes the recipe down with it,
# so a count scraped from a broken run is never presented as a success.

[doc("Count tests across the workspace")]
[extension('.sh')]
[group("advisory")]
[script]
count-tests:
    cargo nextest list --workspace --message-format json \
        | jq -r '[.["rust-suites"][].testcases[] | select(.ignored == false)] | "\(length) tests"'

# `awk 'NR <= 20'` replaces `head -20` because `head` exits at line 20 and the
# upstream `sort` then takes `SIGPIPE`, which pipefail reports as exit 141 on a
# perfectly good run. awk consumes the whole stream and prints the first 20
# lines, so nothing upstream dies and real failures stay visible.

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
    linecop --baseline {{ PCT }} --format paths | ejectest apply . --files-from - --lenient --mod-rs-tests tests

# File backlog tasks from the advisory quality signals, the ones that predict
# mechanical work instead of blocking on it. Each arm, listed in `ARMS` in
# contrib/ask-refactoring.sh, is a scoring script plus a
# `.mindtape/templates/sweep-*.typ` template, not a config change.
#
# Dry by default: it prints each arm's finding count and its top findings, and
# writes those findings' task bodies under target/ask-refactoring/; `--write`
# files every finding. It refuses to run while `mt ls @ask-refactoring` is
# non-empty, so one sprint of mechanical work is emptied before the next is
# filed. The parameters below are the only copy of their defaults, and the
# reason each arm carries its own threshold is in
# docs/contributing/signal-sweep.md.
#
# It is not a passthrough recipe: its six parameters are pattern-checked
# scalars, so `just arg-pattern-check` names it and `just positional-args-check`
# does not. .just/scripts/positional-args-check.sh argues why.

[arg('DRY_GAIN', long='dry-gain', pattern='\d+', help='dry arm: report baselined clones whose extraction saves N tokens or more')]
[arg('WRITE', long='write', pattern='[01]', value='1', help='file the tasks instead of printing what would be filed')]
[arg('REPEAT_GAIN', long='repeat-gain', pattern='\d+', help='repeats arm: report snippets whose extraction saves N tokens or more')]
[arg('BASELINE', long='baseline', pattern='(100|[1-9]?[0-9])', help='size arm: report files at or above PCT% of their linecop limit')]
[arg('CC_FLOOR', long='cc-floor', pattern='\d+', help='undertested arm: floor on cyclomatic complexity')]
[arg('FN_COVERAGE_FLOOR', long='fn-coverage-floor', pattern='(100|[1-9]?[0-9])', help='undertested arm: report functions below PCT% line coverage')]
[doc("File backlog tasks from advisory quality signals")]
[group("advisory")]
ask-refactoring $WRITE='0' $BASELINE='90' $CC_FLOOR='5' $FN_COVERAGE_FLOOR='80' $REPEAT_GAIN='300' $DRY_GAIN='300':
    bash contrib/ask-refactoring.sh --write "$WRITE" --baseline "$BASELINE" \
        --cc-floor "$CC_FLOOR" --fn-coverage-floor "$FN_COVERAGE_FLOOR" \
        --repeat-gain "$REPEAT_GAIN" --dry-gain "$DRY_GAIN"

# The doc sources the audit scans: site prose, the hand-written notes behind
# its generated refs, and the Markdown sources. Never the renders — README.md,
# AGENTS.md and skills/mindtape/SKILL.md are generated from docs/src/*.typ, so
# scanning both sides of a render would pair every sentence with itself.

DOC_PROSE_PATHS := "www/content/*.typ www/content/cli/*.typ www/utils/*.typ docs/src/readme.typ docs/src/agents.typ docs/src/skill.typ skills/mindtape/references/*.md"

# Report near-duplicate prose sentences across the doc sources, worst pair
# first. Advisory, not a gate: the threshold is a judgement call and the report
# is an input to a human deciding where a fact lives. The algorithm, the knobs
# and why it is jq rather than a language of its own are in
# contrib/dup-prose.jq's header; run it before and after a doc refactor.

[doc("Report near-duplicate prose across the doc sources")]
[group("advisory")]
dup-prose:
    @for f in {{ DOC_PROSE_PATHS }}; do jq -n --arg f "$f" --rawfile t "$f" '{file:$f,text:$t}'; done | jq -s -r -f contrib/dup-prose.jq

[doc("Build mt in release mode and install it into $CARGO_HOME/bin")]
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
# and register the `outdatty-lock`, `jscpd-baseline` and `crap-baseline` merge
# drivers `.gitattributes` assigns to outdatty.lock, .jscpd-baseline.json and
# crap-baseline.json. The dev
# shell runs this on entry, and git config is shared by every worktree of a
# clone, so neither needs a manual step.

[doc("Point git at the tracked hooks and register the merge drivers")]
[extension('.sh')]
[group("dev")]
[script]
install-hooks:
    git config core.hooksPath .githooks
    git config merge.outdatty-lock.name "outdatty.lock, merged key by key"
    git config merge.outdatty-lock.driver 'bash .just/scripts/outdatty-lock-merge.sh %O %A %B %P %S %X %Y'
    git config merge.jscpd-baseline.name ".jscpd-baseline.json, merged as a fingerprint union"
    git config merge.jscpd-baseline.driver 'bash .just/scripts/jscpd-baseline-merge.sh %O %A %B %P'
    git config merge.crap-baseline.name "crap-baseline.json, merged entry by entry"
    git config merge.crap-baseline.driver 'bash .just/scripts/crap-baseline-merge.sh %O %A %B %P'
    rm -f .git/hooks/pre-commit
    echo "installed .githooks and the outdatty.lock, .jscpd-baseline.json and crap-baseline.json merge drivers"

# Push main, wait for its CI, then tag (usage: just release 0.3.0). The tag
# push is the only publish path. cargo-release (release.toml) refuses a dirty
# tree and a branch other than main, then bumps the version, dates
# CHANGELOG.md's [Unreleased] section, runs .just/scripts/release-hook.sh and
# commits. The hook refuses an origin/main with commits main lacks, an
# existing tag, a version already on crates.io, a CHANGELOG.md with nothing to
# release, a failing check, nix-build or `cargo publish --dry-run`, and a gh
# that is not logged in. Then main is pushed, so the docs site (pages.yml) and
# CI run on the release tree, and the ci.yml run on that exact commit is
# awaited; the tag is made and pushed only if it is green. A red run leaves
# main pushed and no tag: fix main and re-run, which changes nothing the first
# run already did. A tag whose workflow failed is re-run, never re-tagged:
# `gh workflow run release.yml -f tag=v<version>`. `--dry-run` runs every
# check and pushes nothing; run it before the real release.

[arg('DRY_RUN', long='dry-run', value='true', help='run every check, then say what would be pushed')]
[arg('VERSION', pattern='\d+\.\d+\.\d+', help='semver, no leading v (e.g. 0.3.0)')]
[doc("Push main, wait for its CI, then tag and push the tag")]
[extension('.sh')]
[group("release")]
[no-exit-message]
[script]
release VERSION $DRY_RUN='':
    refuse() { echo "error: $1" >&2; shift; for h in "$@"; do echo "hint: $h" >&2; done; exit 1; }
    tag=v{{ VERSION }}
    args=(-p mindtape-cli --no-confirm)
    if [ -z "$DRY_RUN" ]; then
        read -r -p "Push main and $tag to origin — proceed? [y/N] " answer
        [ "$answer" = y ] || exit 1
        args+=(--execute)
    fi
    cargo release {{ VERSION }} "${args[@]}" --no-tag
    if [ -n "$DRY_RUN" ]; then
        echo "dry run: every check passed"
        echo "would push main, wait for ci.yml on it, then tag and push '$tag'"
        exit 0
    fi
    head=$(git rev-parse HEAD)
    short=$(git rev-parse --short HEAD)
    run=""
    deadline=$((SECONDS + 300))
    while :; do
        run=$(gh run list --commit "$head" --workflow ci.yml --limit 1 \
            --json databaseId --jq '.[0].databaseId // empty') \
            || refuse "can't list ci.yml runs for '$short'" "check gh auth status"
        [ -n "$run" ] && break
        [ "$SECONDS" -lt "$deadline" ] \
            || refuse "no ci.yml run showed up for '$short' in 5 minutes, so '$tag' was not made" "main is pushed; once ci.yml is green on it, re-run just release {{ VERSION }}"
        sleep 5
    done
    echo "waiting for ci.yml run '$run' on '$short'"
    if ! gh run watch "$run" --exit-status --compact --interval 15 >/dev/null; then
        conclusion=$(gh run view "$run" --json conclusion --jq .conclusion 2>/dev/null || true)
        refuse "ci.yml ended '${conclusion:-unknown}' on '$short', so '$tag' was not made" \
            "see why: gh run view $run --log-failed" \
            "fix main and re-run just release {{ VERSION }}, or retry a flake with: gh run rerun $run --failed"
    fi
    cargo release tag "${args[@]}"
    cargo release push "${args[@]}" \
        || { git tag -d "$tag" >/dev/null; refuse "origin rejected the push of '$tag'" "re-run just release {{ VERSION }}"; }
    echo "pushed '$tag'; follow it with: gh run list --workflow release.yml --limit 1"
