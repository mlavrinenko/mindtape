#!/usr/bin/env bash
# Body of `just install::lib`, which runs after `install::bin`. The installed
# `mt` writes the prelude it embeds as plain files, so every other project on
# the machine reads a prelude that matches the binary until the next install.
# `--force` replaces the dev symlink an older install left there. `mt` in a
# checkout of this repo reads that checkout's `lib/` whatever is installed
# (`package_overrides` in crates/mindtape/src/loader/mod.rs).
set -eo pipefail
"${CARGO_HOME:-$HOME/.cargo}/bin/mt" install --no-defaults --no-schema --no-completions --force
