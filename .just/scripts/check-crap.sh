#!/usr/bin/env bash
# .just/scripts/check-crap.sh — the body of the `check-crap` recipe, extracted
# under truth/tasks/extract-the-long-gate-recipe-bodies-into-scripts.typ. Runs
# from the repo root, like the recipe that invokes it. What this arm buys, and
# why it skips loudly instead of failing, are argued in
# docs/contributing/coverage-crap.md; it runs through mmz like every other
# `just check` arm, over an `lcov` input scope naming the artifact this reads,
# so a recorded skip goes stale the moment that artifact appears rather than
# caching "I could not measure" forever — see .mmz/config.yaml and
# docs/contributing/gates.md. `just _lcov-stale-reason`
# stays a recipe call: one staleness test, two callers with opposite policies
# (see its own comment).
set -eo pipefail
reason=$(just _lcov-stale-reason)
if [ -z "$reason" ]; then
    just crap
else
    echo "check-crap: SKIPPED — ${reason}" >&2
    echo "  the CRAP gate was NOT evaluated. \`just cover\` refreshes it;" >&2
    echo "  \`just check-full\` runs the whole thing. CI evaluates it either way." >&2
fi
