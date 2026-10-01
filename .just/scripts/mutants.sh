#!/usr/bin/env bash
# .just/scripts/mutants.sh — the body of the `mutants` recipe, extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it. The TIMEOUT caveat,
# how to bound a loop a mutant can hang, and why there is deliberately no CI
# mutation job are argued in docs/contributing/mutation-testing.md. The file
# set, the excludes and the `mutants` profile come from .cargo/mutants.toml;
# extra args pass through ("$@").
#
# CARGO_MUTANTS_JOBS/RUSTFLAGS come from _mutants-env.sh, shared with
# mutants-diff.sh — argued there for why both stay script-local rather than a
# Justfile-level `export`.
set -eo pipefail
. "$(dirname "${BASH_SOURCE[0]}")/_mutants-env.sh"

# A run that selects a subset rewrites its output directory's mutant list and
# outcomes, which would destroy the `--iterate` cache a full sweep keeps in
# `mutants.out`: the 2026-09-30 full sweep found only 46 of 4240 mutants to
# skip because earlier scoped runs had overwritten it, and re-tested the rest
# cold (15h). Route scoped runs to the scratch dir under `target/` unless the
# caller passed an output of their own; only a no-subset full sweep — the one
# `--iterate` reads — touches `mutants.out`.
has_output=""
has_subset=""
for arg in "$@"; do
    case "$arg" in
        -o|-o?*|--output|--output=*) has_output=1 ;;
        -f|-f?*|--file|--file=*|--in-diff|--in-diff=*|--diff|--diff=*) has_subset=1 ;;
    esac
done
if [ -n "$has_subset" ] && [ -z "$has_output" ]; then
    set -- "$@" --output target/mutants-scoped
fi

cargo mutants "$@"
