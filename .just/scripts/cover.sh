#!/usr/bin/env bash
# .just/scripts/cover.sh — the body of the `cover` recipe, extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it; the prose arguing
# why cargo-llvm-cov replaced tarpaulin, and why the `clean --workspace` below
# is the line that makes the number mean anything, stays above that recipe in
# the Justfile.
set -eo pipefail
ignore='crates/mindtape-cli/src/main\.rs|/registry/src/'
mkdir -p target/coverage
cargo llvm-cov clean --workspace
cargo llvm-cov --workspace --no-report
cargo llvm-cov report --ignore-filename-regex "$ignore" \
    --lcov --output-path target/coverage/lcov.info
cargo llvm-cov report --ignore-filename-regex "$ignore" --fail-under-lines 90
