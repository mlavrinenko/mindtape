#!/usr/bin/env bash
# Sourced preamble for install-lib.sh and uninstall-lib.sh — the one place the
# @local/mindtape version and its install path are derived. Deriving them in
# each script with the same sed would give a version-path change two edit
# sites and no gate pairing them.
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' lib/typst.toml | head -1)
TARGET="$HOME/.local/share/typst/packages/local/mindtape"
LINK="$TARGET/$VERSION"
