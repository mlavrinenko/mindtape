#!/usr/bin/env bash
# .just/scripts/_crap-ratchet-lib.sh — the ratchet arm of `just crap`, sourced
# by crap.sh and pinned by its control (_crap-ratchet-control.sh).
#
# Why the baseline is aligned before cargo-crap reads it
# (truth/tasks/stop-the-crap-ratchet-flagging-untouched-same-named-functions.typ).
# cargo-crap 0.3.1 pairs a current function with its baseline entry in three
# passes (src/delta.rs): exact `(file, function, line)`; then same name, by the
# longest unique common path suffix; then same name, when exactly one of each
# is left. The committed baseline holds root-relative paths
# (crap-baseline-check.sh says why) and the live scan reports absolute ones, so
# the exact pass never fires and the suffix pass carries every pairing. Two
# functions of one name in one file tie on their suffix, so the suffix pass
# skips them, and the name pass skips them too. Each such function then reports
# as new and its baseline entry as removed: it is never compared, a rise in it
# never fails, and its new row sorts into the regression table above the real
# regressions — where a reader takes it for the one that failed.
#
# So before the comparison the baseline gets the scan's own root and, within
# each `(file, function)` group taken in line order, the line of the current
# function at the same position. That makes the exact pass key on
# `(file, function, ordinal)`, which a line shift leaves alone. A baseline
# entry with no counterpart at its position gets line 0, which no function has,
# so it can only pair through the name passes, never with a stranger's line.
# Inserting a same-named function above another in the same file still
# re-pairs the ones below; that file was touched, and the failure names it.

# crap_ratchet <root> <baseline> <cargo-crap scan args>...
crap_ratchet() {
    local root=$1 baseline=$2 cur aligned rc=0
    shift 2
    cur=$(mktemp)
    aligned=$(mktemp)
    cargo crap "$@" --format json --output "$cur"
    jq --arg root "$root/" --slurpfile cur "$cur" '
        def key: [.file, .function] | tojson;
        ($cur[0].entries
            | map(.file |= ltrimstr($root))
            | group_by(key)
            | map({key: (.[0] | key), value: (map(.line) | sort)})
            | from_entries) as $lines
        | .entries |= (group_by(key)
            | map(sort_by(.line) | to_entries
                | map(.value + {
                    file: ($root + .value.file),
                    line: ($lines[.value | key][.key] // 0)
                }))
            | add // [])
    ' "$baseline" > "$aligned"
    cargo crap "$@" --baseline "$aligned" --fail-regression --format markdown || rc=$?
    rm -f "$cur" "$aligned"
    return "$rc"
}
