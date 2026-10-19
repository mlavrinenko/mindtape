#!/usr/bin/env bash
# .just/scripts/jscpd-baseline-merge.sh — the git merge driver for
# .config/jscpd-baseline.json (truth/tasks/merge-the-jscpd-baseline-by-fingerprint-
# union.typ). `.gitattributes` assigns it as `merge=jscpd-baseline`, and
# `just install-hooks` registers it as
#
#   bash .just/scripts/jscpd-baseline-merge.sh %O %A %B %P
#
# Git runs it with the base, ours and theirs blobs in three temp files and
# expects the result in %A. Exit 0 is a clean merge.
#
# Why a driver. The baseline is a sorted map of clone fingerprints, so two
# branches that each minted a clone add lines next to each other and git's text
# merge conflicts on every rebase. The merge is the union of both sides'
# `fingerprints` (a fingerprint on both sides keeps the larger count), with
# `version` from ours. A fingerprint a side removed comes back; it is a stale
# entry at worst, and `just lint::dry` on the merged tree decides the rest.
# The output is `version`, then `fingerprints` sorted by key, as jscpd writes it.
#
# Without `jq` on PATH, or with a side that does not parse as a baseline, it
# falls back to git's text merge, which is what the repo had before the driver.
set -eo pipefail

base=$1 ours=$2 theirs=$3

union='{version: .[0].version, fingerprints: ([.[].fingerprints | to_entries[]]
    | group_by(.key) | map({key: .[0].key, value: (map(.value) | max)}) | from_entries)}'

if command -v jq >/dev/null 2>&1 \
    && merged=$(jq -s "$union" "$ours" "$theirs" 2>/dev/null) && [ -n "$merged" ]; then
    printf '%s\n' "$merged" >"$ours"
else
    git merge-file "$ours" "$base" "$theirs"
fi
