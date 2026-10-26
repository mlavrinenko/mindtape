#!/usr/bin/env bash
# Body of `just lint dry`; the jscpd flags arrive as arguments. jscpd fails on
# any clone .config/jscpd-baseline.json does not list. The baseline keys a clone
# by a fingerprint of its raw tokens, so an edit inside an old clone makes it
# new.
# .config/jscpd.json (read from the repo root, so every jscpd run here sees it)
# makes jscpd skip declaration lines: `mod` lines with their attributes, `use`
# items and `("name", include_str!("..."))` registry entries. A list of those
# has no helper to extract, and adding one more line must not change a
# fingerprint. Skipped tokens also leave a clone that straddles one, so a new
# pattern re-mints the baseline once.
# Fix a finding by extracting a shared helper, not by shuffling tokens until
# the detector stops matching.
#
# Under a baseline every jscpd reporter prints all clones, old ones included,
# so this reads the json report and prints only the new ones.
set -eo pipefail
out=$(mktemp -d)
trap 'rm -rf "$out"' EXIT
log=$(jscpd "$@" -r json -o "$out" crates 2>&1) || { printf '%s\n' "$log"; exit 1; }
new=$(jq -r '.duplicates[] | select(.isNew)
    | "crates/\(.firstFile.name):\(.firstFile.start): \(.kind) clone of crates/\(.secondFile.name):\(.secondFile.start)"' \
    "$out/jscpd-report.json")
[ -z "$new" ] || { printf '%s\n' "$new"; exit 1; }
