#!/usr/bin/env bash
# .just/scripts/mutants.sh — the body of the `mutants` recipe, extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it; the prose arguing
# the TIMEOUT caveat, how to bound a loop a mutant can hang, and why there is
# deliberately no CI mutation job stays above that recipe in the Justfile. The
# file set, the excludes and the `mutants` profile come from .cargo/mutants.toml;
# extra args pass through ("$@").
#
# CARGO_MUTANTS_JOBS/RUSTFLAGS come from _mutants-env.sh, shared with
# mutants-diff.sh — argued there for why both stay script-local rather than a
# Justfile-level `export`.
set -eo pipefail
. "$(dirname "${BASH_SOURCE[0]}")/_mutants-env.sh"
cargo mutants "$@"
