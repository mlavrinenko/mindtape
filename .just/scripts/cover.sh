#!/usr/bin/env bash
# .just/scripts/cover.sh — the body of the `cover` recipe, extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it. Why cargo-llvm-cov
# replaced tarpaulin, and why the `clean --workspace` below is the line that
# makes the number mean anything, are argued in
# docs/contributing/coverage-crap.md.
set -eo pipefail
# main.rs is excluded — keep it thin (`just crap` excludes it too); so is any
# registry checkout, which llvm-cov's own dep filter misses when CARGO_HOME is
# relocated (a local kache), so local and CI report the same set.
ignore='crates/mindtape-cli/src/main\.rs|/registry/src/'
mkdir -p target/coverage
# Instrument once (--no-report), then render both reports from that profdata.
# The clean is cargo-llvm-cov's own documented precondition for that split
# ("should be called before the first run"): --no-report deliberately does NOT
# clear the profraw pile, because the split exists so several runs CAN be
# merged. Dropping it unions every profraw and every object file left in
# target/llvm-cov-target since the last `cargo clean` into the lcov — measured,
# and argued in the docs page above.
cargo llvm-cov clean --workspace
cargo llvm-cov --workspace --no-report
cargo llvm-cov report --ignore-filename-regex "$ignore" \
    --lcov --output-path target/coverage/lcov.info
cargo llvm-cov report --ignore-filename-regex "$ignore" --fail-under-lines 90
