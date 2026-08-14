#!/usr/bin/env bash
# .just/scripts/arg-pattern-check.sh — the body of the `arg-pattern-check`
# recipe (truth/tasks/validate-recipe-arguments-with-the-arg-attribute.typ).
# Runs from the repo root, like the recipe that invokes it.
#
# A separate script from fmt-fixture-check.sh on purpose: that one locks
# three FORMATTERS reject drift in a synthesized fixture file; this one locks
# that the `[arg(...)]` `pattern`s on `release`, `bench`, `gen-bench` and
# `eject` actually reject a bad argument and accept a good one. Different
# subject (recipe dispatch, not file formatting), so a separate fixture-style
# script — bundling them would make one script answer two questions.
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

# assert_rejected <recipe> <bad-value>
# Dispatch must fail with just's own pattern-mismatch message, not reach the
# recipe body.
assert_rejected() {
    local recipe="$1" value="$2" out
    if out=$(just --dry-run --yes "$recipe" "$value" </dev/null 2>&1); then
        echo "arg-pattern-check: $recipe accepted bad argument '$value' — pattern is not biting" >&2
        echo "  output: $out" >&2
        fail=1
        return
    fi
    if [[ "$out" != *"does not match pattern"* ]]; then
        echo "arg-pattern-check: $recipe rejected '$value' for a reason other than the pattern:" >&2
        echo "  $out" >&2
        fail=1
        return
    fi
    echo "arg-pattern-check: $recipe correctly rejects '$value'"
}

# assert_accepted <recipe> <good-value>
# Dispatch must succeed and print the recipe's command line (dry-run), never
# the pattern-mismatch error.
assert_accepted() {
    local recipe="$1" value="$2" out
    if ! out=$(just --dry-run --yes "$recipe" "$value" </dev/null 2>&1); then
        echo "arg-pattern-check: $recipe rejected known-good argument '$value'" >&2
        echo "  output: $out" >&2
        fail=1
        return
    fi
    if [[ "$out" == *"does not match pattern"* ]]; then
        echo "arg-pattern-check: $recipe rejected '$value' on the pattern despite exit 0 — inconsistent" >&2
        fail=1
        return
    fi
    echo "arg-pattern-check: $recipe correctly accepts '$value'"
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

if [ "$fail" -ne 0 ]; then
    echo "arg-pattern-check: one or more arg patterns did not behave as expected" >&2
    exit 1
fi
echo "arg-pattern-check: all recipe arg patterns reject bad input and accept good input"
