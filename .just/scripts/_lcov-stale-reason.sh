#!/usr/bin/env bash
# .just/scripts/_lcov-stale-reason.sh — the body of the `_lcov-stale-reason`
# recipe, extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it; the prose arguing
# why one implementation serves two callers with opposite policies stays above
# that recipe in the Justfile.
#
# It reports through STDOUT and ALWAYS exits 0 — an empty stdout means fresh,
# any other line names why the lcov is unusable. The exit status is not the
# channel and must not become one: `crap` treats a reason as fatal while
# `check-crap` treats it as a reason to skip, and a non-zero exit here would
# make `just` print its own `error: Recipe ... failed` line into that skip
# path, so a deliberate skip would read on screen exactly like a broken gate.
set -eo pipefail
lcov=target/coverage/lcov.info
if [ ! -f "$lcov" ]; then
    echo "$lcov is missing"
    exit 0
fi
newer=$(find crates -name '*.rs' -newer "$lcov" -print -quit)
if [ -n "$newer" ]; then
    echo "$lcov predates $newer"
fi
