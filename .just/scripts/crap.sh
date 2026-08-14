#!/usr/bin/env bash
# .just/scripts/crap.sh — the body of the `crap` recipe, extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it; the prose arguing
# the threshold, each exclusion and why there is deliberately no advisory band
# below the gate stays above that recipe in the Justfile.
#
# The exclude flags arrive as ARGUMENTS ("$@"), never as a copy of the list.
# CRAP_EXCLUDES stays one Justfile variable because three consumers must score
# the same file set — this gate, `crap-baseline`, and
# contrib/ask-refactoring/undertested.sh via `_crap-excludes` — and a baseline
# minted over a different set than the gate scores reports every difference
# between the two lists as a regression.
set -eo pipefail
lcov=target/coverage/lcov.info
reason=$(just _lcov-stale-reason)
if [ -n "$reason" ]; then
    echo "just crap: $reason — run \`just cover\` first" >&2
    echo "  (a source the lcov never saw scores as 0% covered, not as unknown)" >&2
    exit 1
fi
# Two checks, run separately so each failure prints its own readable table.
# Folding them into one invocation does not work: `--min 30` is a PRINT
# filter, so a regression from 5 to 12 would fail the gate with no visible
# row, and baseline mode hides unchanged rows, so a function sitting above
# the threshold without moving would fail with no visible row either. Each
# mode needs the other's rows suppressed to stay legible.
#
# 1. Absolute: no function above 30, whatever its history.
cargo crap --lcov "$lcov" --workspace "$@" \
    --threshold 30 --min 30 --fail-above --format markdown
# 2. Ratchet: no function worse than the committed baseline, at any level.
# This is the half that sees the 12 complex-and-undertested functions scoring
# below 15 that no advisory band could separate from noise
# (truth/tasks/sweep-crap-scores-into-tasks.typ measured why). It never reads
# the absolute score, only the delta, so the degeneracy that killed the band
# — CRAP collapsing to cyclomatic complexity above ~90% coverage — does not
# apply: a rise still means complexity went up or coverage went down.
cargo crap --lcov "$lcov" --workspace "$@" \
    --baseline crap-baseline.json --fail-regression --format markdown
