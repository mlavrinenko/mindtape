#!/usr/bin/env bash
# .just/scripts/crap.sh — the body of the `crap` recipe, extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it. The threshold
# itself, why a stale lcov is refused rather than scored against, and why there
# is deliberately no advisory band below the gate are argued in
# docs/contributing/coverage-crap.md.
#
# The exclude flags arrive as ARGUMENTS ("$@"), never as a copy of the list.
# CRAP_EXCLUDES stays one Justfile variable because three consumers must score
# the same file set — this gate, `crap-baseline`, and
# contrib/ask-refactoring/undertested.sh via `_crap-excludes` — and a baseline
# minted over a different set than the gate scores reports every difference
# between the two lists as a regression.
#
# Each value in that list is spelled `**/…` because `--exclude` is matched
# twice, in two different roots: once per-crate during `cargo crap`'s own
# source walk (so `main.rs` must read `**/main.rs`, not
# `crates/mindtape-cli/src/main.rs` — the latter never matches a walk rooted at
# the crate dir, and so leaves `main.rs` in the "no matching entry in the LCOV
# report" warning even though it is correctly excluded from scoring), and once
# workspace-root-relative when filtering the final table. A `**/` prefix
# matches both.
#
# What each exclusion buys:
#
#   - `*_tests.rs`/`tests.rs` keep CRAP on production code. They are inline
#     test modules (`#[path] mod tests`), which llvm-cov emits no records for,
#     so scoring them meant 71 of the 73 "no matching entry in the LCOV report"
#     warnings and a complexity budget spent on test helpers. Excluding them
#     changes no verdict.
#   - `recur_rrule.rs` is real production code, but `just cover` builds with
#     default features only, so nothing ever compiles it and it would otherwise
#     score every function inside as phantom 0%-covered (see `just
#     check-rrule`, which covers it under the `rrule` feature — CI runs it as a
#     separate job, per truth/tasks/run-the-rrule-feature-gate-in-ci.typ).
#   - `ls_interactive/tui.rs` drives the real terminal in every function — raw
#     mode, the alternate screen, keyboard-enhancement negotiation — so an
#     in-process test would either hijack the developer's own screen or assert
#     nothing without a TTY. It is pinned end-to-end under a pseudo-terminal
#     instead (tests/interactive_e2e.rs: quitting_restores_the_terminal pins
#     the enter/leave byte sequences;
#     opening_a_task_suspends_the_tui_and_runs_the_editor pins the suspend
#     path). The one uncovered branch left (the `enhanced`
#     keyboard-enhancement path in enter_screen) depends on a terminal
#     negotiation no test can force.
#   - `main.rs` stays thin, and `cover.sh` excludes it from coverage for the
#     same reason.
set -eo pipefail
lcov=target/coverage/lcov.info
reason=$(just _lcov-stale-reason)
if [ -n "$reason" ]; then
    echo "just crap: $reason — run \`just cover\` first" >&2
    echo "  (a source the lcov never saw scores as 0% covered, not as unknown)" >&2
    exit 1
fi
# Two checks, run separately so each failure prints its own readable table.
# Folding them into one invocation does not work: `--min 30` is a PRINT
# filter, so a regression from 5 to 12 would fail the gate with no visible
# row, and baseline mode hides unchanged rows, so a function sitting above
# the threshold without moving would fail with no visible row either. Each
# mode needs the other's rows suppressed to stay legible.
#
# 1. Absolute: no function above 30, whatever its history. `--min` matches
# `--threshold`, so the table lists only the functions that FAIL; without it
# the report is every analyzed function — 976 rows to carry the one that
# matters. The cost is that the tail summary then counts printed rows ("1/1",
# not "1/976"); the numerator, which is the number that decides the gate, is
# unaffected.
cargo crap --lcov "$lcov" --workspace "$@" \
    --threshold 30 --min 30 --fail-above --format markdown
# 2. Ratchet: no function worse than the committed baseline, at any level.
# This is the half that sees the 12 complex-and-undertested functions scoring
# below 15 that no advisory band could separate from noise
# (truth/tasks/sweep-crap-scores-into-tasks.typ measured why). It never reads
# the absolute score, only the delta, so the degeneracy that killed the band
# — CRAP collapsing to cyclomatic complexity above ~90% coverage — does not
# apply: a rise still means complexity went up or coverage went down.
cargo crap --lcov "$lcov" --workspace "$@" \
    --baseline crap-baseline.json --fail-regression --format markdown
