#!/usr/bin/env bash
# .just/scripts/mutants.sh — the body of the `mutants` recipe, extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it; the prose arguing
# the TIMEOUT caveat, how to bound a loop a mutant can hang, and why there is
# deliberately no CI mutation job stays above that recipe in the Justfile. The
# file set, the excludes and the `mutants` profile come from .cargo/mutants.toml;
# extra args pass through ("$@").
#
# Both knobs stay script-local, for the reason spelled out in
# .just/scripts/mutants-diff.sh: a Justfile-level `export RUSTFLAGS` would land
# in every cargo invocation's fingerprint and invalidate the build cache.
set -eo pipefail
export CARGO_MUTANTS_JOBS="${CARGO_MUTANTS_JOBS:-3}"
export RUSTFLAGS="${RUSTFLAGS:-} -Clink-arg=-fuse-ld=mold"
cargo mutants "$@"
