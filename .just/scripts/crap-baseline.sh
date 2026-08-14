#!/usr/bin/env bash
# .just/scripts/crap-baseline.sh — the body of the `crap-baseline` recipe,
# extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it; the prose arguing
# when to re-mint, why `just crap` runs first, why `--sort file` is the only
# stable ordering for a committed baseline, and how to recover when the
# BASELINE is the thing that is wrong, stays above that recipe in the Justfile.
#
# The exclude flags arrive as ARGUMENTS ("$@"), from the same CRAP_EXCLUDES the
# gate is passed — see .just/scripts/crap.sh. Minting over a different file set
# than the gate scores is the exact failure the one-variable discipline exists
# to prevent.
set -eo pipefail
cargo crap --lcov target/coverage/lcov.info --workspace "$@" \
    --format json --sort file --output crap-baseline.json
echo "crap-baseline.json re-minted — commit it with the change that justifies it"
