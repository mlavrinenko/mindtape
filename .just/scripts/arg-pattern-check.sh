#!/usr/bin/env bash
# .just/scripts/arg-pattern-check.sh — the body of the `arg-pattern-check`
# recipe (truth/tasks/validate-recipe-arguments-with-the-arg-attribute.typ).
# Runs from the repo root, like the recipe that invokes it.
#
# A pattern nobody exercises is a comment, not a gate. So this is a separate
# script from fmt-fixture-check.sh on purpose: that one locks that three
# formatters reject drift in a synthesized fixture file; this one locks that
# the `[arg(...)]` `pattern`s on `release`, `bench`, `gen-bench`, `eject`,
# `ask-refactoring` and `outdatty-update` actually reject a bad argument and
# accept a good one.
# Same reasoning, different subject (recipe dispatch, not file formatting), so
# a sibling fixture-style script rather than an added arm in that one —
# bundling them would make one script answer two questions.
#
# `ask-refactoring`'s five thresholds are here for a second reason on top of
# the first (truth/tasks/derive-the-ask-refactoring-flag-table-from-arg-
# attributes.typ). Its patterns are not decoration: they are what replaced
# `[positional-arguments]` on that recipe. A digits-only value cannot be
# word-split or re-globbed by the shell that runs the body, so the pattern
# biting is the quoting guarantee, so positional-args-check.sh does not
# names the recipe. A named flag is checked the way it is actually typed
# (`--cc-floor abc`, not a bare positional) because dispatch through the flag
# is the only way anyone reaches these parameters: declaring `long=` on a
# parameter makes it flag-only.
#
# `just --dry-run --yes <recipe> <value>` is what makes this safe to run
# inside `just check`: argument-pattern validation happens at dispatch, before
# any recipe line runs, so a rejected value never reaches the body — and
# `--dry-run` means even an accepted value never executes a recipe line
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
}

assert_rejected release v0.3.0
assert_rejected release 0.3
assert_accepted release 0.3.0

assert_rejected bench abc
assert_accepted bench 100

assert_rejected gen-bench abc
assert_accepted gen-bench 100

# `outdatty-update` takes the one group whose source was just edited — the
# blanket form is the separate `outdatty-update-all` recipe, so the group is
# required (no default to assert). The `'Dev Docs'` case is the word-splitting
# one, where the shell would split the value into two words:
# the body interpolates the value into `outdatty update --group {{ GROUP }}`,
# so a space must be refused at dispatch. `'DEV_DOCS'` locks the shape that a
# group name can never take (outdatty.yaml names are all lowercase).
assert_rejected outdatty-update 'Dev Docs'
assert_rejected outdatty-update 'DEV_DOCS'
assert_accepted outdatty-update dev-shell-docs

# The optional dependents are spliced into the body as `--dependent <path>`
# words by `prepend`, unquoted, so each path must be refused at dispatch when
# it carries a space or a shell metacharacter. Every tracked path here is
# letters, digits, `.`, `_`, `/` and `-`.
assert_rejected outdatty-update dev-shell-docs 'docs/src/a b.typ'
assert_rejected outdatty-update dev-shell-docs 'docs/src/a.typ;true'
assert_accepted outdatty-update dev-shell-docs docs/src/contributing/windows.typ

# Bare `just outdatty-update` must fail at dispatch too — the whole point of
# requiring the group is that muscle memory cannot reach the blanket form.
# The error is just's own missing-argument one, not a pattern
# mismatch, so assert_rejected's message check does not apply.
if out=$(just --dry-run --yes outdatty-update </dev/null 2>&1); then
    echo "arg-pattern-check: bare outdatty-update succeeded — GROUP must be required" >&2
    echo "  output: $out" >&2
    fail=1
fi

# `fork`'s two parameters are both interpolated into the body, and each pattern
# is guarding a different thing. `slug` becomes a branch name and a directory name,
# so `feat/x` has to be refused: git would take the branch happily and the
# directory would land a level down from where the printed path says it is.
# `base` is a commit-ish and therefore has to admit far more shape (`HEAD~2`,
# `origin/main`, `v0.3.0^{commit}`, a bare sha) — what it exists to refuse is a
# value that a shell could split or re-read, which is why the space case is the
# one asserted rather than a malformed revision. A revision that is merely wrong
# is git's to reject at `rev-parse`, by name, and the script leaves it there.
#
# `base` defaults to `HEAD` and `slug` is required, so assert_default_accepted would
# be asserting a bare `just fork` — which just refuses for the missing `slug`, not
# on a pattern. The default is exercised by the one-argument accept below.
assert_rejected fork 'feat/x'
assert_rejected fork 'Slug'
assert_rejected fork agent-one 'a b'
assert_accepted fork agent-one
assert_accepted fork agent-one 'HEAD~2'
assert_accepted fork agent-one origin/main

assert_rejected eject 120
assert_rejected eject abc
assert_accepted eject 50
assert_default_accepted eject

# The sweep's five thresholds. One rejection per distinct pattern rather than
# per flag: `--baseline`, `--coverage-floor` and `--fn-coverage-floor` share
# the 0-100 percentage pattern, `--cc-floor` is unbounded digits, `--write` is
# a single bit. The out-of-range case (`101`) is checked once for the shared
# percentage pattern and the non-numeric case (`abc`) once for each of the
# three, since those are three different regexes' business.
#
# `a b` is the assertion that earns the paragraph at the top of this file.
# Every other rejection here is about a value being wrong; that one is about a
# value being unsplittable. The pattern refuses it at dispatch, so the recipe
# never word-splits it.
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
# `ask-refactoring`'s defaults are the only copy of the sweep's thresholds
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
