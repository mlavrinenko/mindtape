#!/usr/bin/env bash
# .just/scripts/_mutants-env.sh — the CARGO_MUTANTS_JOBS/RUSTFLAGS preamble
# shared by mutants.sh and mutants-diff.sh. Sourced, not executed on its own
# — an underscore-prefixed helper, same convention as _lcov-stale-reason.sh
# and the `_crap-excludes` recipe.
#
# Three parallel jobs and the mold linker, benchmarked at ~3.5x over the naive
# run. Both are overridable, and both are tuned for the pre-PR pass: drop jobs
# to 2 if parallel load provokes false TIMEOUTs, and see
# docs/contributing/mutation-testing.md for why a sweep whose verdict you
# intend to believe runs at 2 or lower anyway.
#
# Both knobs stay script-local: sourcing this into a script's own process
# exports them for that process and its children (cargo mutants) only, never
# Justfile-wide. RUSTFLAGS in particular must never become a Justfile-level
# `export` — it is part of the cargo fingerprint, so exporting it to every
# recipe would invalidate the build cache for the whole workspace.
export CARGO_MUTANTS_JOBS="${CARGO_MUTANTS_JOBS:-3}"
export RUSTFLAGS="${RUSTFLAGS:-} -Clink-arg=-fuse-ld=mold"
