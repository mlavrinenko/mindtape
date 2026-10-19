#!/usr/bin/env bash
# Sourced preamble for install-lib.sh and uninstall-lib.sh — the one place the
# @local/mindtape version and its install path are derived. Deriving them in
# each script with the same sed would give a version-path change two edit
# sites and no gate pairing them.
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' lib/typst.toml | head -1)
# The package directory `mt` installs into and loads from, `package_root()` in
# crates/mindtape/src/loader/app_dirs.rs: `$MINDTAPE_DATA_HOME/typst/packages`
# when that is set, else Typst's own, `$TYPST_PACKAGE_PATH` when set, else the
# Linux data directory's `typst/packages`. A set but empty variable means none.
if [ -n "${MINDTAPE_DATA_HOME+set}" ]; then
    VAR=MINDTAPE_DATA_HOME
    PACKAGES="${MINDTAPE_DATA_HOME:+$MINDTAPE_DATA_HOME/typst/packages}"
elif [ -n "${TYPST_PACKAGE_PATH+set}" ]; then
    VAR=TYPST_PACKAGE_PATH
    PACKAGES="$TYPST_PACKAGE_PATH"
else
    PACKAGES="${XDG_DATA_HOME:-$HOME/.local/share}/typst/packages"
fi
if [ -z "$PACKAGES" ]; then
    echo "$VAR is empty, so mt loads no installed package" >&2
    exit 1
fi
TARGET="$PACKAGES/local/mindtape"
# shellcheck disable=SC2034 # install-lib.sh and uninstall-lib.sh source this file and read it
LINK="$TARGET/$VERSION"
