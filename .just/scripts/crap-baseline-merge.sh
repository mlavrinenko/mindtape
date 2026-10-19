#!/usr/bin/env bash
# .just/scripts/crap-baseline-merge.sh — the git merge driver for
# .config/crap-baseline.json (truth/tasks/merge-the-crap-baseline-and-same-key-
# outdatty-conflicts.typ). `.gitattributes` assigns it as `merge=crap-baseline`,
# and `just install-hooks` registers it as
#
#   bash .just/scripts/crap-baseline-merge.sh %O %A %B %P
#
# Git runs it with the base, ours and theirs blobs in three temp files and
# expects the result in %A. Exit 0 is a clean merge. During a rebase "ours" is
# the upstream being rebased onto.
#
# Why a driver. Every entry carries the `line` of its function, so two
# branches that each edit one file shift the lines of every function below the
# edit, and git's text merge conflicts on entries neither branch re-scored.
#
# The key is the ratchet's. _crap-ratchet-lib.sh pairs a baseline entry with a
# function by file, name and position among the same-named functions of that
# file, in line order, and never by line. So the driver keys entries the same
# way and merges each key three ways, its scores (`cyclomatic`, `coverage`,
# `crap`, `crate`) as one value and its `line` as another. A value only one
# side changed takes that side.
#
# A key whose scores both sides changed keeps the stricter: the lower `crap`,
# then the lower `cyclomatic`, then the higher `coverage`. The merged code
# scores a third value neither side measured, and the ratchet fails a function
# that scores above its entry. The stricter entry can only make that fail on
# the merged tree, where a human re-mints. The looser one could hide a rise the
# other branch's mint would have caught. The driver names each such key on
# stderr. A key one side dropped stays if the other side re-scored it, since a
# stale entry is harmless and a missing one fails as unlisted. A key the other
# side only moved goes.
#
# Lines cannot be right for a merged file, and nothing reads them but the
# ratchet's ordering. A line both sides moved takes ours. Within one file's
# same-named functions each line is kept above the one before, so the ratchet
# still pairs them in the same order. The output is sorted by file, function
# and line, as `just crap::baseline` writes it, and `just crap::check` on the merged
# tree decides the rest.
#
# Without `jq` on PATH, or with a side that does not parse as a baseline, it
# falls back to git's text merge, which is what the repo had before the driver.
set -eo pipefail

base=$1 ours=$2 theirs=$3 path=${4:-.config/crap-baseline.json}

text_merge() {
    echo "crap-baseline-merge: $1; falling back to git's text merge of $path" >&2
    local rc=0
    git merge-file "$ours" "$base" "$theirs" || rc=$?
    exit "$rc"
}

command -v jq >/dev/null || text_merge "jq is not on PATH (the dev shell provides it)"

# baseline_json <side> <out>: the side as one JSON value. An empty side is a
# baseline that does not exist there, as in an add/add merge. The baseline is
# passed through files, since a thousand entries overflow an argument list.
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
baseline_json() {
    if [ -s "$1" ]; then
        jq -c 'if (.entries | type) == "array" then . else error("no entries") end' "$1" >"$2" 2>/dev/null
    else
        echo '{"entries": []}' >"$2"
    fi
}
if ! { baseline_json "$base" "$tmp/o" && baseline_json "$ours" "$tmp/a" && baseline_json "$theirs" "$tmp/b"; }; then
    text_merge "a side of $path does not parse as a baseline"
fi

# merged: `{doc, contested}`, the merged baseline and the keys whose scores
# both sides changed.
# shellcheck disable=SC2016 # a jq program, whose $names are jq's
merge='
def keyed: .entries | group_by([.file, .function])
    | map(sort_by(.line) | to_entries
        | map({key: ([.value.file, .value.function, .key] | tojson), value: .value}))
    | add // [] | from_entries;
def scores: if . == null then null else {cyclomatic, coverage, crap, crate} end;
def three($o; $a; $b): if $a == $b or $o == $b then $a elif $o == $a then $b else null end;
def stricter($x; $y): [$x, $y] | sort_by(.crap, .cyclomatic, -.coverage) | first;
($o | keyed) as $O | ($a | keyed) as $A | ($b | keyed) as $B
| [ [$O, $A, $B] | map(keys) | add | unique | .[] | . as $k
    | ($O[$k] | scores) as $so | ($A[$k] | scores) as $sa | ($B[$k] | scores) as $sb
    | (if $sa == $sb or $so == $sb then {s: $sa}
       elif $so == $sa then {s: $sb}
       elif $sa == null then {s: $sb}
       elif $sb == null then {s: $sa}
       else {s: stricter($sa; $sb), contested: true} end) as $m
    | select($m.s != null)
    | ($k | fromjson) as [$file, $function, $ordinal]
    | {file: $file, function: $function, ordinal: $ordinal,
       line: (three($O[$k].line; $A[$k].line; $B[$k].line) // $A[$k].line // $B[$k].line),
       scores: $m.s, contested: ($m.contested // false)} ] as $merged
| def top($f): three($o[$f]; $a[$f]; $b[$f]) // $a[$f] // $b[$f];
  {doc: ({"$schema": top("$schema"), version: top("version")} | with_entries(select(.value != null)))
     + {entries: ($merged | group_by([.file, .function])
         | map(sort_by(.ordinal) | reduce .[] as $e ([];
             . + [$e + {line: ([$e.line, ((last.line // 0) + 1)] | max)}]))
         | add // [] | sort_by(.file, .function, .line)
         | map({file, function, line} + .scores))},
   contested: [$merged[] | select(.contested) | "\(.file) \(.function)"]}
'
jq -n --slurpfile o "$tmp/o" --slurpfile a "$tmp/a" --slurpfile b "$tmp/b" \
    '$o[0] as $o | $a[0] as $a | $b[0] as $b | '"$merge" >"$tmp/merged"
jq '.doc' "$tmp/merged" >"$ours"
jq -r '.contested[]' "$tmp/merged" | while IFS= read -r where; do
    echo "crap-baseline-merge: $where: both sides re-scored it; kept the stricter score, and \`just crap::check\` on the merged tree decides" >&2
done
