#!/usr/bin/env bash
# .just/scripts/mutants-diff.sh — the body of the `mutants-diff` recipe,
# extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it. What a
# missed mutant means is argued in docs/contributing/mutation-testing.md; where
# the speed knobs came from is argued in _mutants-env.sh, next to them. Extra
# args pass through ("$@").
#
# CARGO_MUTANTS_JOBS/RUSTFLAGS come from _mutants-env.sh, shared with
# mutants.sh. Both stay script-local: sourcing exports them into this
# script's own process, never a Justfile-level `export`, which would land
# RUSTFLAGS in every cargo invocation's fingerprint and invalidate the
# workspace build cache for every other recipe.
set -eo pipefail
. "$(dirname "${BASH_SOURCE[0]}")/_mutants-env.sh"
diff_file=$(mktemp)
trap 'rm -f "$diff_file"' EXIT
git diff "$(git merge-base main HEAD)" > "$diff_file"

# Always a subset run, so it writes to the same `target/mutants-scoped`
# scratch `mutants.sh` routes `-f` sweeps to — never to `mutants.out`, whose
# `--iterate` cache a full sweep reads (see mutants.sh).
has_output=""
for arg in "$@"; do
    case "$arg" in
        -o|-o?*|--output|--output=*) has_output=1 ;;
    esac
done
if [ -z "$has_output" ]; then
    set -- "$@" --output target/mutants-scoped
fi

mutants_budget_check --in-diff "$diff_file" "$@"
cargo mutants --in-diff "$diff_file" "$@"
