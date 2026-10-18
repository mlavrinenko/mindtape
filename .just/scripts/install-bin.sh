#!/usr/bin/env bash
# Body of `just install::bin`. Lands in cargo's bin directory, as `cargo install` does.
set -eo pipefail
BIN="${CARGO_HOME:-$HOME/.cargo}/bin"
cargo build --release -p mindtape-cli
mkdir -p "$BIN"
# `install` opens-and-truncates, which is ETXTBSY while any `mt` is still
# executing; rename(2) swaps the directory entry without touching the running
# inode.
install -m 755 target/release/mt "$BIN/mt.new"
mv "$BIN/mt.new" "$BIN/mt"
echo "installed $("$BIN/mt" --version) -> $BIN/mt"
