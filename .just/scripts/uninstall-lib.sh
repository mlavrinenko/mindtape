#!/usr/bin/env bash
# Body of `just uninstall-lib`. Runs from the repo root, like install-lib.sh,
# whose preamble it shares.
set -eo pipefail
. .just/scripts/_typst-package.sh
if [ -L "$LINK" ]; then
    rm "$LINK"
    echo "removed $LINK"
else
    echo "no symlink at $LINK"
fi
