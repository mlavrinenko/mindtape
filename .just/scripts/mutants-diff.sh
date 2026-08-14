#!/usr/bin/env bash
# .just/scripts/mutants-diff.sh — the body of the `mutants-diff` recipe,
# extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it; the prose arguing
# what a surviving MISSED mutant means and where the speed knobs came from
# stays above that recipe in the Justfile. Extra args pass through ("$@").
#
# Both knobs stay script-local. RUSTFLAGS in particular must never become a
# Justfile-level `export`: it lands in every cargo invocation's fingerprint and
# invalidates the workspace build cache for every other recipe.
set -eo pipefail
export CARGO_MUTANTS_JOBS="${CARGO_MUTANTS_JOBS:-3}"
export RUSTFLAGS="${RUSTFLAGS:-} -Clink-arg=-fuse-ld=mold"
diff_file=$(mktemp)
trap 'rm -f "$diff_file"' EXIT
git diff "$(git merge-base main HEAD)" > "$diff_file"
cargo mutants --in-diff "$diff_file" "$@"
