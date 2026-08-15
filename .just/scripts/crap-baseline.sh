#!/usr/bin/env bash
# .just/scripts/crap-baseline.sh — the body of the `crap-baseline` recipe,
# extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it. When to re-mint and
# why `just crap` runs first are stated above that recipe; how to recover when
# the BASELINE itself is the thing that is wrong is argued in
# docs/contributing/coverage-crap.md.
#
# The exclude flags arrive as ARGUMENTS ("$@"), from the same CRAP_EXCLUDES the
# gate is passed — see .just/scripts/crap.sh. Minting over a different file set
# than the gate scores is the exact failure the one-variable discipline exists
# to prevent.
set -eo pipefail
# `--sort file` is the documented stable ordering for a committed baseline;
# `--sort crap` reshuffles the whole file whenever any score moves, which turns
# every diff into noise.
cargo crap --lcov target/coverage/lcov.info --workspace "$@" \
    --format json --sort file --output crap-baseline.json
echo "crap-baseline.json re-minted — commit it with the change that justifies it"
