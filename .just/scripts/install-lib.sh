#!/usr/bin/env bash
# Body of `just install::lib`. Runs from the repo root, like the recipe that
# invokes it — `$(pwd)/lib` is the link target, so a caller in another
# directory would install a broken symlink.
set -eo pipefail
. .just/scripts/_typst-package.sh
mkdir -p "$TARGET"
# Drop any prior install first: `ln -sfn` into an existing *directory* places
# the link inside it, leaving a stale copy that tinymist reads.
rm -rf "$LINK"
ln -sfn "$(pwd)/lib" "$LINK"
echo "linked lib -> $LINK (use @local/mindtape:$VERSION)"
