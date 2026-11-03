<!-- Generated from docs/src/contributing/release.typ by `just docs md`. Do not edit; edit the source. -->

# Releasing

What `just release VERSION` runs, and how to finish a release that failed. The tag push is the only publish path, so don’t run `cargo publish` or push `main` by hand.

## What the recipe runs

1. It runs [cargo-release](https://github.com/crate-ci/cargo-release) (`.config/release.toml`), which refuses a dirty tree or a branch other than `main`, bumps the version, dates the entries under `[Unreleased]` and commits.
2. Between the bump and the commit, cargo-release runs `.just/scripts/release-hook.sh`. The hook re-records the outdatty groups whose sources are the Cargo manifests, since the release commit reviews the bump, holds the release commit’s message to `mt check --commit-msg`, and refuses every other state a release has failed from.
3. The recipe pushes `main`, so CI runs on the release tree.
4. It tags only once that commit’s `ci.yml` run is green.

`--dry-run` checks and pushes nothing. Its publish check verifies a throwaway copy bumped to the new version, since the working tree still carries the old one.

## Prelude versions

Before a release bumps `lib/typst.toml`, archive the outgoing prelude. Copy `lib/prelude.typ` to `crates/mindtape/src/loader/preludes/<old-version>/prelude.typ`, add `<old-version>` to `PRELUDE_SUPPORTED` in `crates/mindtape/src/loader/embedded.rs`, and keep a fixture that loads it. The snapshot is immutable. The current version’s snapshot equals `lib/prelude.typ`, and a test holds the two together.

A release drops a version on purpose by removing it from `PRELUDE_SUPPORTED` and deleting its snapshot and fixture. That release stops reading the version everywhere, including a past commit in `mt check --commits`.

## After a failure

After a red CI, fix `main` and re-run the recipe: the bump and the dating stay done.

A release whose workflow failed is finished on the same tag with `gh workflow run release.yml -f tag=vVERSION`, which skips each crate crates.io already has. A pushed tag never moves.
