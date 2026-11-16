#!/usr/bin/env bash
# .just/scripts/crap-baseline.sh: the body of the `crap::baseline` recipe,
# split out under
# `truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ`.
# Runs from the repo root, like the recipe that invokes it. The comment above
# that recipe states when to re-mint.
#
# The exclude flags arrive as arguments ("$@"), from the same CRAP_EXCLUDES the
# gate gets. See .just/scripts/crap.sh. Minting over a different file set than
# the gate scores is the exact failure the one-variable discipline exists to
# prevent.
set -eo pipefail

# Guard: the threshold arm only. Minting a >30 function in as acceptable is the
# freeze this exists to stop, and it also refuses a stale lcov. The commit
# then holds the numbers a clean run reproduces. It is deliberately not the
# whole gate. The ratchet arm scores against the file this script is about to
# rewrite, so a deliberate complexity increase makes it red by definition.
# That is the one moment CONTRIBUTING.md says to re-mint. The case for the
# arm split lives in .just/scripts/crap.sh.
bash .just/scripts/crap.sh absolute "$@"

tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT
# `--sort file` is the documented stable ordering for a committed baseline.
# `--sort crap` reshuffles the whole file whenever any score moves, which turns
# every diff into noise. Stripping a shared prefix below preserves that order,
# because every path loses the same leading components.
cargo crap --lcov target/coverage/lcov.info --workspace "$@" \
    --format json --sort file --output "$tmp"

# Relativize, because `cargo crap --workspace` walks the member roots
# `cargo metadata` reports, and cargo-crap 0.3.1 has no flag to shorten them.
# Every entry lands as `/home/<user>/…/mindtape/crates/…`, publishing the
# minter's own machine layout in a tracked file.
#
# The exact match this gives up was never worth anything. cargo-crap's delta
# matcher keys pass 1 on `(file, function, line)`. A baseline minted here
# already misses that pass entirely under CI's own checkout root and pairs
# through the suffix-aware pass instead (`pass_suffix_match`, upstream test
# `cross_root_baseline_matches_without_move_status`). That pass has carried
# the ratchet on every CI run since its introduction. A relative baseline
# routes local through the same pass CI already uses. The gate stops behaving
# differently on the one machine that minted the file.
#
# The prefix comes from `cargo metadata` rather than `$PWD`: it is the root
# cargo-crap itself derived those paths from, so it matches even where the
# you reach the checkout through a symlink. `ltrimstr` is a no-op on a path that
# does not carry it, so a future cargo-crap that emits relative paths passes
# through untouched.
root=$(cargo metadata --format-version 1 --no-deps | jq -r '.workspace_root')
jq --arg root "$root/" \
    '.entries |= map(.file |= ltrimstr($root))' "$tmp" > .config/crap-baseline.json

# Fail where the mistake happens, not one gate later. Same script `just check`
# runs, so the mint and the gate can never disagree about what counts as a
# machine-local path.
bash .just/scripts/crap-baseline-check.sh
echo ".config/crap-baseline.json re-minted — commit it with the change that justifies it"
