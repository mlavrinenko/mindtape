#!/usr/bin/env bash
# .just/scripts/_lcov-stale-reason.sh — the body of the `_lcov-stale-reason`
# recipe, extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it.
#
# `crap` treats a reason printed here as a hard error, so it never scores
# functions against an lcov that predates their source.
#
# It reports through STDOUT and always exits 0 — an empty stdout means fresh,
# any other line names why the lcov is unusable. The caller decides what a
# reason means, and prints it beside the recipe that refreshes the lcov.
set -eo pipefail
lcov=target/coverage/lcov.info
if [ ! -f "$lcov" ]; then
    echo "$lcov is missing"
    exit 0
fi
newer=$(find crates -name '*.rs' -newer "$lcov" -print -quit)
# An mtime proves nothing about content: a checkout or rebase touches files
# without changing their bytes, and mmz, which keys `just cover` by content,
# then hits and leaves the lcov alone. So a newer source is stale only when
# mmz does not call `just cover` fresh.
if [ -n "$newer" ] && ! mmz --is-fresh -- just cover >/dev/null 2>&1; then
    echo "$lcov predates $newer"
fi
