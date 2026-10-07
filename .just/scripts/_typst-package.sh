#!/usr/bin/env bash
# Sourced preamble for install-lib.sh and uninstall-lib.sh — the one place the
# @local/mindtape version and its install path are derived. Deriving them in
# each script with the same sed would give a version-path change two edit
# sites and no gate pairing them.
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' lib/typst.toml | head -1)
# The data directory `mt` loads the package from (crates/mindtape/src/loader/app_dirs.rs):
# `$MINDTAPE_DATA_HOME` when set, where empty means none, else what `dirs`
# resolves on Linux.
if [ -n "${MINDTAPE_DATA_HOME+set}" ]; then
    DATA="$MINDTAPE_DATA_HOME"
else
    DATA="${XDG_DATA_HOME:-$HOME/.local/share}"
fi
if [ -z "$DATA" ]; then
    echo "MINDTAPE_DATA_HOME is empty, so mt loads no installed package" >&2
    exit 1
fi
TARGET="$DATA/typst/packages/local/mindtape"
LINK="$TARGET/$VERSION"
