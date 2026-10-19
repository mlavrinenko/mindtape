<!-- Generated from docs/src/contributing/release.typ by `just docs md`. Do not edit; edit the source. -->

# Releasing

What `just release VERSION` runs, and how to finish a release that failed. The tag push is the only publish path, so don’t run `cargo publish` or push `main` by hand.

## What the recipe runs

1. It runs [cargo-release](https://github.com/crate-ci/cargo-release) (`.config/release.toml`), which refuses a dirty tree or a branch other than `main`, bumps the version, dates the entries under `[Unreleased]` and commits.
2. Between the bump and the commit, cargo-release runs `.just/scripts/release-hook.sh`. The hook refuses every other state a release has failed from.
3. The recipe pushes `main`, so CI runs on the release tree.
4. It tags only once that commit’s `ci.yml` run is green.

`--dry-run` checks and pushes nothing.

## After a failure

After a red CI, fix `main` and re-run the recipe: the bump and the dating stay done.

A release whose workflow failed is finished on the same tag with `gh workflow run release.yml -f tag=vVERSION`, which skips each crate crates.io already has. A pushed tag never moves.
