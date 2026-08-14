#!/usr/bin/env bash
# .just/scripts/mutants-diff.sh — the body of the `mutants-diff` recipe,
# extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it; the prose arguing
# what a surviving MISSED mutant means and where the speed knobs came from
# stays above that recipe in the Justfile. Extra args pass through ("$@").
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
cargo mutants --in-diff "$diff_file" "$@"
