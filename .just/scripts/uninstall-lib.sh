#!/usr/bin/env bash
# Body of `just install::uninstall-lib`. Runs from the repo root, where the
# preamble reads lib/typst.toml for the version.
set -eo pipefail
. .just/scripts/_typst-package.sh
if [ -L "$LINK" ]; then
    rm "$LINK"
    echo "removed the link $LINK"
elif [ -d "$LINK" ]; then
    rm -rf "$LINK"
    echo "removed $LINK"
else
    echo "nothing installed at $LINK"
fi
