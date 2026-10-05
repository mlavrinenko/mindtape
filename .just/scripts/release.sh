#!/usr/bin/env bash
# .just/scripts/release.sh — the body of the `release` recipe, extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it; the prose naming
# what it refuses to release over stays above that recipe in the Justfile.
# Takes the version as $1.
#
# The `[confirm(...)]` guard stays on the recipe, where `just` enforces it —
# this script cannot re-implement it, and must not let anyone step around it.
# So it refuses to run unless MINDTAPE_RELEASE_CONFIRMED is set, which only the
# recipe sets, right after the prompt is answered. Same handshake shape as
# MINDTAPE_GENERATED_LOCK. Without it, extracting the body would have turned a
# confirmed, tag-pushing recipe into a script anyone can run straight, and the
# `git push origin` at the bottom is not undoable.
set -eo pipefail
if [ -z "$MINDTAPE_RELEASE_CONFIRMED" ]; then
    echo "release.sh: run \`just release <version>\` instead — the tag-and-push confirmation lives on that recipe" >&2
    exit 1
fi
version="$1"
cargo_version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
if [ "$version" != "$cargo_version" ]; then
    echo "error: requested v$version but Cargo.toml is $cargo_version; bump Cargo.toml first" >&2
    exit 1
fi
if [ -n "$(git status --porcelain)" ]; then
    echo "error: working tree is dirty; commit or stash before releasing" >&2
    exit 1
fi
unreleased=$(sed -n '/^## \[Unreleased\]/,/^## \[/p' CHANGELOG.md | sed '1d;$d' | sed '/^[[:space:]]*$/d')
if [ -n "$unreleased" ]; then
    echo "error: CHANGELOG.md has a non-empty [Unreleased] section; move it into a dated [$version] section before releasing" >&2
    exit 1
fi
just check
just nix-build
git push origin main
git tag -a "v$version" -m "v$version"
git push origin "v$version"
