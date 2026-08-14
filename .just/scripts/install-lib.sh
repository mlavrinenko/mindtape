#!/usr/bin/env bash
# .just/scripts/install-lib.sh — the body of the `install-lib` recipe,
# extracted under
# truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ.
# Runs from the repo root, like the recipe that invokes it — `$(pwd)/lib` is
# the link target, so a caller in another directory would install a broken
# symlink. `uninstall-lib` stays inline in the Justfile: four lines.
set -eo pipefail
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' lib/typst.toml | head -1)
TARGET="$HOME/.local/share/typst/packages/local/mindtape"
mkdir -p "$TARGET"
# Drop any prior install first: `ln -sfn` into an existing *directory*
# places the link inside it, leaving a stale copy that tinymist reads.
rm -rf "$TARGET/$VERSION"
ln -sfn "$(pwd)/lib" "$TARGET/$VERSION"
echo "linked lib -> $TARGET/$VERSION (use @local/mindtape:$VERSION)"
