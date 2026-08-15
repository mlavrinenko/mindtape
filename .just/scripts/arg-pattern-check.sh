#!/usr/bin/env bash
# .just/scripts/arg-pattern-check.sh — the body of the `arg-pattern-check`
# recipe (truth/tasks/validate-recipe-arguments-with-the-arg-attribute.typ).
# Runs from the repo root, like the recipe that invokes it.
#
# A separate script from fmt-fixture-check.sh on purpose: that one locks
# three FORMATTERS reject drift in a synthesized fixture file; this one locks
# that the `[arg(...)]` `pattern`s on `release`, `bench`, `gen-bench`, `eject`
# and `ask-refactoring` actually reject a bad argument and accept a good one.
# Different subject (recipe dispatch, not file formatting), so a separate
# fixture-style script — bundling them would make one script answer two
# questions.
#
# `ask-refactoring`'s five thresholds are here for a second reason on top of
# the first (truth/tasks/derive-the-ask-refactoring-flag-table-from-arg-
# attributes.typ). Its patterns are not decoration: they are what replaced
# `[positional-arguments]` on that recipe. A digits-only value cannot be
# word-split or re-globbed by the shell that runs the body, so the pattern
# biting is the quoting guarantee, and positional-args-check.sh no longer
# names the recipe. A named flag is checked the way it is actually typed
# (`--cc-floor abc`, not a bare positional) because dispatch through the flag
# is the only way anyone reaches these parameters: declaring `long=` on a
# parameter makes it flag-only.
#
# `just --dry-run --yes <recipe> <value>` is what makes this safe to run
# inside `just check`: argument-pattern validation happens at DISPATCH, before
# any recipe line runs, so a rejected value never reaches the body — and
# `--dry-run` means even an ACCEPTED value never executes a recipe line
# either (verified: it prints the command instead of running it). `--yes`
# clears `release`'s `[confirm(...)]` prompt, which would otherwise block on
# stdin between the pattern check and the (skipped) execution. Every
# invocation redirects stdin from /dev/null too, belt-and-suspenders against
# a hang if `--yes` alone were ever not enough.
set -eo pipefail

fail=0

# assert_rejected <recipe> <bad-argument>...
# Dispatch must fail with just's own pattern-mismatch message, not reach the
# recipe body. Variadic after the recipe name so a named flag can be checked
# the way it is actually typed (`--cc-floor abc`), not only as a positional.
assert_rejected() {
    local recipe="$1" out
    shift
    if out=$(just --dry-run --yes "$recipe" "$@" </dev/null 2>&1); then
        echo "arg-pattern-check: $recipe accepted bad argument '$*' — pattern is not biting" >&2
        echo "  output: $out" >&2
        fail=1
        return
    fi
    if [[ "$out" != *"does not match pattern"* ]]; then
        echo "arg-pattern-check: $recipe rejected '$*' for a reason other than the pattern:" >&2
        echo "  $out" >&2
        fail=1
        return
    fi
    echo "arg-pattern-check: $recipe correctly rejects '$*'"
}

# assert_accepted <recipe> <good-argument>...
# Dispatch must succeed and print the recipe's command line (dry-run), never
# the pattern-mismatch error.
assert_accepted() {
    local recipe="$1" out
    shift
    if ! out=$(just --dry-run --yes "$recipe" "$@" </dev/null 2>&1); then
        echo "arg-pattern-check: $recipe rejected known-good argument '$*'" >&2
        echo "  output: $out" >&2
        fail=1
        return
    fi
    if [[ "$out" == *"does not match pattern"* ]]; then
        echo "arg-pattern-check: $recipe rejected '$*' on the pattern despite exit 0 — inconsistent" >&2
        fail=1
        return
    fi
    echo "arg-pattern-check: $recipe correctly accepts '$*'"
}

# assert_default_accepted <recipe>
# The recipe's own declared default must satisfy its own pattern — a
# default that fails its own gate breaks every caller that omits the
# argument (e.g. `fix-check` depends on bare `eject`).
assert_default_accepted() {
    local recipe="$1" out
    if ! out=$(just --dry-run --yes "$recipe" </dev/null 2>&1); then
        echo "arg-pattern-check: $recipe rejects its own default argument" >&2
        echo "  output: $out" >&2
        fail=1
        return
    fi
    echo "arg-pattern-check: $recipe's default satisfies its own pattern"
}

assert_rejected release v0.3.0
assert_rejected release 0.3
assert_accepted release 0.3.0

assert_rejected bench abc
assert_accepted bench 100

assert_rejected gen-bench abc
assert_accepted gen-bench 100

assert_rejected eject 120
assert_rejected eject abc
assert_accepted eject 50
assert_default_accepted eject

# The sweep's five thresholds. One rejection per DISTINCT pattern rather than
# per flag: `--baseline`, `--coverage-floor` and `--fn-coverage-floor` share
# the 0-100 percentage pattern, `--cc-floor` is unbounded digits, `--write` is
# a single bit. The out-of-range case (`101`) is checked once for the shared
# percentage pattern and the non-numeric case (`abc`) once for each of the
# three, since those are three different regexes' business.
#
# `a b` is the assertion that earns the paragraph at the top of this file.
# Every other rejection here is about a value being wrong; that one is about a
# value being UNSPLITTABLE — it is the case `[positional-arguments]` used to
# protect this recipe against, refused at dispatch instead.
#
# `--write` gets no bad-value case, and cannot have one. It carries
# `value='1'`, so the flag consumes no argument of its own, and a parameter
# declared with `long=` is flag-only — verified on just 1.46: `just
# ask-refactoring 2` reports `Justfile does not contain recipe 2` rather than
# assigning `2` to the first parameter. Its `[01]` pattern therefore guards
# exactly two values, the declared default and that fixed `1`, and the
# assert_default_accepted call below is what exercises it.
#
# assert_default_accepted matters more here than anywhere else in this file:
# `ask-refactoring`'s defaults are the ONLY copy of the sweep's thresholds
# (contrib/ask-refactoring.sh requires all five and defaults nothing), so a
# default that failed its own pattern would break every plain
# `just ask-refactoring` run and there is no fallback to hide it.
assert_rejected ask-refactoring --baseline 101
assert_rejected ask-refactoring --baseline 'a b'
assert_rejected ask-refactoring --coverage-floor abc
assert_rejected ask-refactoring --cc-floor abc
assert_rejected ask-refactoring --fn-coverage-floor 101
assert_accepted ask-refactoring --baseline 85
assert_accepted ask-refactoring --coverage-floor 92 --cc-floor 12
assert_accepted ask-refactoring --write
assert_default_accepted ask-refactoring

if [ "$fail" -ne 0 ]; then
    echo "arg-pattern-check: one or more arg patterns did not behave as expected" >&2
    exit 1
fi
echo "arg-pattern-check: all recipe arg patterns reject bad input and accept good input"
