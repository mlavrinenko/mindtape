#!/usr/bin/env bash
# .just/scripts/arg-pattern-check.sh: the body of the `meta::arg-pattern`
# recipe (truth/tasks/validate-recipe-arguments-with-the-arg-attribute.typ).
# Runs from the repo root, like the recipe that invokes it.
#
# A pattern nobody exercises is a comment, not a gate. So this is a separate
# script from fmt-fixture-check.sh on purpose. That one locks that three
# formatters reject drift in a synthesized fixture file. This one locks that
# the `[arg(...)]` `pattern`s on `release`, `bench::run`, `bench::gen`, `eject`,
# `advisory::ask-refactoring`, `outdatty::update` and `check` actually reject a
# bad argument and accept a good one.
# Same reasoning, different subject (recipe dispatch, not file formatting), so
# a sibling fixture-style script rather than an added arm in that one.
# Bundling them would make one script answer two questions.
#
# `ask-refactoring`'s six thresholds are here for a second reason on top of
# the first (truth/tasks/derive-the-ask-refactoring-flag-table-from-arg-
# attributes.typ). Its patterns are not decoration. They are what replaced
# `[positional-arguments]` on that recipe. A digits-only value cannot be
# word-split or re-globbed by the shell that runs the body. The pattern biting
# is the quoting guarantee, so positional-args-check.sh skips the recipe.
# This script checks a named flag the way it is actually typed
# (`--cc-floor abc`, not a bare positional). Dispatch through the flag is the
# only way anyone reaches these parameters: declaring `long=` on a parameter
# makes it flag-only.
#
# `just --dry-run --yes <recipe> <value>` is what makes this safe to run
# inside `just check`. Argument-pattern validation happens at dispatch,
# before any recipe line runs, so a rejected value never reaches the body.
# `--dry-run` means even an accepted value never executes a recipe line
# either (verified: it prints the command instead of running it). `--yes`
# clears `release`'s `[confirm(...)]` prompt, which would otherwise block on
# stdin between the pattern check and the (skipped) execution. Every
# invocation redirects stdin from /dev/null too, belt-and-suspenders against
# a hang if `--yes` alone were ever not enough.
set -eo pipefail

fail=0

# `assert_rejected` <recipe> <bad-argument>...
# Dispatch must fail with just's own pattern-mismatch message, not reach the
# recipe body. Variadic after the recipe name so this script can check a named
# flag the way it is actually typed (`--cc-floor abc`), not only as a positional.
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
}

# `assert_accepted` <recipe> <good-argument>...
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
}

# `assert_default_accepted` <recipe>
# The recipe's own declared default must satisfy its own pattern. A
# default that fails its own gate breaks every caller that omits the
# argument (such as `fix-check`, which depends on bare `eject`).
assert_default_accepted() {
    local recipe="$1" out
    if ! out=$(just --dry-run --yes "$recipe" </dev/null 2>&1); then
        echo "arg-pattern-check: $recipe rejects its own default argument" >&2
        echo "  output: $out" >&2
        fail=1
        return
    fi
}

assert_rejected release v0.3.0
assert_rejected release 0.3
assert_accepted release 0.3.0

assert_rejected bench::run abc
assert_accepted bench::run 100

assert_rejected bench::gen abc
assert_accepted bench::gen 100

# `outdatty::update` takes the one group whose source was just edited. The
# blanket form is the separate `outdatty::update-all` recipe, so the group is
# mandatory (no default to assert). The `'Dev Docs'` case is the word-splitting
# one, where the shell would split the value into two words. The body
# interpolates the value into `outdatty update --group {{ GROUP }}`, so
# dispatch must refuse a space. `'DEV_DOCS'` locks the shape that a group
# name can never take (.config/outdatty.yaml names are all lowercase).
assert_rejected outdatty::update 'Dev Docs'
assert_rejected outdatty::update 'DEV_DOCS'
assert_accepted outdatty::update dev-shell-docs

# `prepend` splices the optional dependents into the body as `--dependent <path>`
# words, unquoted, so dispatch must refuse each path that carries a space or a
# shell metacharacter. Every tracked path here is letters, digits, `.`, `_`,
# `/` and `-`.
assert_rejected outdatty::update dev-shell-docs 'docs/src/a b.typ'
assert_rejected outdatty::update dev-shell-docs 'docs/src/a.typ;true'
assert_accepted outdatty::update dev-shell-docs docs/src/contributing/windows.typ

# Bare `just outdatty::update` must fail at dispatch too. The whole point of
# requiring the group is that muscle memory cannot reach the blanket form.
# The error is just's own missing-argument one, not a pattern
# mismatch, so the `assert_rejected` message check does not apply.
if out=$(just --dry-run --yes outdatty::update </dev/null 2>&1); then
    echo "arg-pattern-check: bare outdatty::update succeeded — GROUP must be required" >&2
    echo "  output: $out" >&2
    fail=1
fi

assert_rejected eject 120
assert_rejected eject abc
assert_accepted eject 50
assert_default_accepted eject

# The sweep's five thresholds. One rejection per distinct pattern rather than
# per flag: `--baseline` and `--fn-coverage-floor` share the 0-100
# percentage pattern, `--cc-floor` accepts unbounded digits, `--write` takes
# a single bit. The out-of-range case (`101`) gets checked once for the shared
# percentage pattern. The non-numeric case (`abc`) gets checked once for each
# of the remaining three, since those are three different regexes' business.
#
# `a b` is the assertion that earns the paragraph at the top of this file.
# Every other rejection here is about a value being wrong. That one is about a
# value being unsplittable. The pattern refuses it at dispatch, so the recipe
# never word-splits it.
#
# `--write` has no bad-value case, and cannot have one. It carries
# `value='1'`, so the flag doesn't consume an argument of its own. A
# parameter declared with `long=` is flag-only (verified on just 1.46: `just
# advisory::ask-refactoring 2` reports `Justfile does not contain recipe 2`
# rather than assigning `2` to the first parameter). Its `[01]` pattern guards exactly
# two values, the declared default and that fixed `1`, and the
# `assert_default_accepted` call below is what exercises it.
#
# `assert_default_accepted` matters more here than anywhere else in this
# file. `ask-refactoring`'s defaults are the only copy of the sweep's
# thresholds (contrib/ask-refactoring.sh requires all four and defaults
# nothing). A default that failed its own pattern would break every plain
# `just advisory ask-refactoring` run, and there is no fallback to hide it.
assert_rejected advisory::ask-refactoring --baseline 101
assert_rejected advisory::ask-refactoring --baseline 'a b'
assert_rejected advisory::ask-refactoring --cc-floor abc
assert_rejected advisory::ask-refactoring --fn-coverage-floor 101
assert_accepted advisory::ask-refactoring --baseline 85
assert_accepted advisory::ask-refactoring --fn-coverage-floor 92 --cc-floor 12
assert_accepted advisory::ask-refactoring --write
# `check` takes stage names, which CI uses to run the cover stage in a job of
# its own. A name outside the stages must fail at dispatch, not run nothing.
assert_rejected check bogus
assert_rejected check fast bogus
assert_accepted check
assert_accepted check fast build
assert_accepted check cover
assert_default_accepted advisory::ask-refactoring

if [ "$fail" -ne 0 ]; then
    echo "arg-pattern-check: one or more arg patterns did not behave as expected" >&2
    exit 1
fi
