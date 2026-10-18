#!/usr/bin/env bash
# .just/scripts/positional-args-check.sh — the body of the `positional-args-
# check` recipe (truth/tasks/preserve-argument-quoting-with-positional-
# arguments.typ). Runs from the repo root, like the recipe that invokes it.
#
# A separate script from arg-pattern-check.sh on purpose, for the reason that
# one is itself separate from fmt-fixture-check.sh: different subject. That
# one locks that `[arg(...)]` `pattern`s reject bad input and accept good
# input at dispatch, entirely via `--dry-run`, so no recipe body ever runs.
# This one locks that `test`, `build`, `mutants::sweep` and `mutants::diff` — the
# recipes that take a passthrough `*ARGS` — actually forward it as a real
# argument vector instead of a bare string.
# That property is invisible to `--dry-run`: word-splitting and globbing
# happen when the shell interpreting the recipe body evaluates it, which
# dry-run explicitly never does. So this is a two-part check instead:
#
#   1. Structural, over `just --dump --dump-format json` — the same source
#      check-doc-recipes.sh reads recipe names from. Each of the four must
#      carry the `positional-arguments` attribute, and its body must be
#      exactly one line with no `{{ VAR }}` interpolation fragment left in
#      it (gates.jq's own comment documents the shape: an interpolated
#      fragment serializes as `["variable", "NAME"]`, not a plain string) —
#      catching a regression where the attribute stays but a body reverts to
#      `{{ ARGS }}`, which would silently drop back to word-splitting even
#      though the attribute suggests otherwise.
#   2. Textual, over the two scripts the passthrough recipes forward into
#      (mutants.sh, mutants-diff.sh) — each must consume its own "$@", not
#      re-join or re-split it.
#
# `ask-refactoring` is not covered here, because it takes no passthrough
# `*ARGS`. Its five thresholds are declared parameters, each with an
# `[arg(...)]` attribute and a digits-only `pattern`, so a value that could be
# word-split or globbed cannot pass dispatch at all, and the body forwards
# exported parameters through quoted `"$VAR"` expansions instead of
# interpolating `{{ }}`. arg-pattern-check.sh locks the patterns, and the
# patterns lock the quoting.
# This script reads nothing outside .just/scripts/** and the two justfiles.
#
# Both are static: nothing here runs cargo test, cargo build or cargo
# mutants. A dynamic proof (an argument containing a space and a glob
# character surviving as one word, end to end) lives in this task's own
# commit message and is not re-run on every `just check` — mutants and
# ask-refactoring shell out to real cargo/mt builds, and this repo already
# keeps them out of `just check` for exactly that cost (see docs/contributing/
# mutation-testing.md for why the mutation sweep has no CI job either). A static gate is
# what stays cheap
# enough to run every time and still catches the regression that matters:
# the attribute or the "$@" disappearing from the Justfile or a script.
set -eo pipefail

fail=0
dump="$(just --dump --dump-format json)"

# assert_forwards <recipe>
# `[positional-arguments]` present, body is exactly one line, and that line
# has no leftover `{{ VAR }}` fragment — i.e. every element of the body's
# single line is a plain string, never a `{"variable": ...}` node.
assert_forwards() {
    local recipe="$1" ok
    ok=$(jq -e --arg name "$recipe" '
        ($name | split("::")) as $path
        | (reduce $path[:-1][] as $m (.; .modules[$m]) | .recipes[$path[-1]]) as $r
        | ($r.attributes // [] | any(. == "positional-arguments"))
          and (($r.body // []) | length == 1)
          and (($r.body[0] // []) | all(type == "string"))
    ' <<<"$dump" 2>/dev/null) || ok=false
    if [ "$ok" != "true" ]; then
        echo "positional-args-check: $recipe is missing [positional-arguments] or its body still interpolates a {{ VAR }} fragment" >&2
        fail=1
        return
    fi
}

assert_forwards test
assert_forwards build
assert_forwards mutants::sweep
assert_forwards mutants::diff

# assert_script_forwards <script> <pattern>
# The script a passthrough recipe forwards into must consume "$@" itself —
# grepped, not parsed, because these are the same files
# .just/scripts/crap.sh's own header points to as the convention to follow.
assert_script_forwards() {
    local script="$1" pattern="$2"
    if ! grep -qE "$pattern" "$script"; then
        echo "positional-args-check: $script does not forward \"\$@\" (expected to match: $pattern)" >&2
        fail=1
        return
    fi
}

assert_script_forwards .just/scripts/mutants.sh 'cargo mutants.*"\$@"'
assert_script_forwards .just/scripts/mutants-diff.sh 'cargo mutants.*"\$@"'

if [ "$fail" -ne 0 ]; then
    echo "positional-args-check: one or more passthrough recipes lost real argument forwarding" >&2
    exit 1
fi
