#!/usr/bin/env bash
# .just/scripts/outdatty-lock-merge.sh — the git merge driver for outdatty.lock
# (truth/tasks/merge-outdatty-lock-by-group-so-parallel-branches-rebase-
# without-a-conflict.typ). `.gitattributes` assigns it as `merge=outdatty-lock`,
# and `just install-hooks`, which the dev shell runs on entry, registers it as
#
#   bash .just/scripts/outdatty-lock-merge.sh %O %A %B %P %S %X %Y
#
# Git runs it from the top of the worktree with the base, ours and theirs blobs
# in three temp files, the path being merged, and the three conflict labels. It
# writes the result over %A and exits 0 for a clean merge, non-zero to leave a
# conflict. During a rebase "ours" is the upstream being rebased onto.
#
# Why a driver. Two branches that re-record different groups change hash lines
# that sit a line or two apart, so git's text merge conflicts on lines neither
# branch's review touched, and `git checkout --ours` drops the other branch's
# recorded reviews. The lock is a map, so it merges as one: each leaf
# (`version`, `algorithm`, and every `groups.<g>.<source|dependents>.<path>`
# hash, an empty `{}` map counting as a leaf) is a three-way merge of its own,
# and a leaf only one side changed takes that side.
#
# A leaf both sides changed to different hashes takes the side whose hash is
# the hash of that path in the merged tree, since that is the side whose review
# the merged file still matches. Neither matching (both edited the file, so the
# merged file is a third content) is a conflict naming the group and path, with
# conflict markers around that one line, and the fix is the review the
# conflict asks for: `just outdatty-update <group>`.
#
# How the merged file is read, and why not from the working tree. merge-ort
# merges every path in memory and writes the worktree only after the last one,
# so while this driver runs the working tree still holds ours (measured on git
# 2.54: a rebase sees HEAD's file, not the merged one). So the merged blob is
# derived from the three commits instead, and they come from the labels: %Y is
# `<sha> (<subject>)` in a rebase or cherry-pick and the branch name in a merge,
# %S is `parent of <sha> …` in a pick and the base sha in a merge, and ours is
# HEAD in all three. The labels are text, so they are trusted only after each
# commit's lock blob is checked against the blob git handed over. A path
# unchanged on one side takes the other side's blob, which is what git's own
# merge does, and a path changed on both sides has no single blob to hash. When
# the commits cannot be resolved (a stash, a criss-cross merge's virtual base)
# every key both sides changed conflicts. A wrong guess is worse than a stop.
#
# Without `yq`, `jq` or `b3sum` on PATH (a merge run outside the dev shell), or
# with a side that does not parse as a lock, it falls back to git's text merge,
# which is what the repo had before the driver.
set -eo pipefail

base=$1 ours=$2 theirs=$3 path=${4:-outdatty.lock}
base_label=${5:-base} ours_label=${6:-ours} theirs_label=${7:-theirs}

text_merge() {
    echo "outdatty-lock-merge: $1; falling back to git's text merge of $path" >&2
    exec git merge-file -L "$ours_label" -L "$base_label" -L "$theirs_label" \
        "$ours" "$base" "$theirs"
}

for tool in yq jq b3sum; do
    command -v "$tool" >/dev/null || text_merge "$tool is not on PATH (the dev shell provides it)"
done

# An empty side is a lock that does not exist there, as in an add/add merge.
lock_json() {
    if [ -s "$1" ]; then
        yq -c 'if type == "object" then . else error("not a map") end' "$1" 2>/dev/null
    else
        echo '{}'
    fi
}
o=$(lock_json "$base") && a=$(lock_json "$ours") && b=$(lock_json "$theirs") \
    || text_merge "a side of $path does not parse as a lock"

# merge($tree): one entry per leaf of the result, `{path, value}` for a merged
# leaf and `{path, conflict, file?}` for a conflicting one. `$tree` maps a
# tracked path to its blake3 hash in the merged tree.
lib='
def leaves: reduce (paths(type != "object" or . == {}) as $p | [$p, getpath($p)]) as [$p, $v]
    ({}; .[$p | tojson] = $v);
def merge($tree):
    ($o | leaves) as $O | ($a | leaves) as $A | ($b | leaves) as $B
    | [ [$O, $A, $B] | map(keys) | add | unique | .[]
        | . as $k | ($k | fromjson) as $p
        | def same($x; $y): ($x | has($k)) == ($y | has($k)) and $x[$k] == $y[$k];
          def keep($x): if $x | has($k) then {path: $p, value: $x[$k]} else empty end;
          if same($A; $B) or same($O; $B) then keep($A)
          elif same($O; $A) then keep($B)
          else ((if $p[0] == "groups" and ($p | length) == 4 and ($A | has($k)) and ($B | has($k))
                 then $p[3] else null end) as $file
                | if $file != null and $tree[$file] == $A[$k] then keep($A)
                  elif $file != null and $tree[$file] == $B[$k] then keep($B)
                  else {path: $p, file: $file, conflict: {ours: $A[$k], theirs: $B[$k],
                        inA: ($A | has($k)), inB: ($B | has($k))}}
                  end)
          end ];
'

contested=$(jq -nr --argjson o "$o" --argjson a "$a" --argjson b "$b" \
    "$lib"' merge({}) | .[] | select(.conflict and .file) | .file')

# label_commit <label>: the commit a conflict label names, or nothing.
label_commit() {
    local label=$1 suffix=
    case $label in "parent of "*) label=${label#parent of }; suffix='^' ;; esac
    label=${label%% *}
    label=${label%...}
    git rev-parse -q --verify "$label$suffix^{commit}" 2>/dev/null
}

# lock_blob_is <commit> <file>: the commit holds exactly that side's lock.
lock_blob_is() {
    [ "$(git rev-parse -q --verify "$1:$path" 2>/dev/null)" = "$(git hash-object --no-filters "$2")" ]
}

blob() { git rev-parse -q --verify "$1:$2" 2>/dev/null || true; }

# merged_hash <path>: the blake3 of <path> in the merged tree, or nothing when
# both sides changed it or the merge deletes it.
merged_hash() {
    local ob oh ot merged
    ob=$(blob "$base_commit" "$1") oh=$(blob "$ours_commit" "$1") ot=$(blob "$theirs_commit" "$1")
    if [ "$ot" = "$ob" ]; then
        merged=$oh
    elif [ "$oh" = "$ob" ] || [ "$oh" = "$ot" ]; then
        merged=$ot
    else
        return 0
    fi
    [ -z "$merged" ] || git cat-file blob "$merged" | b3sum --no-names
}

tree='{}'
if [ -n "$contested" ]; then
    base_commit=$(label_commit "$base_label") || true
    ours_commit=$(git rev-parse -q --verify HEAD) || true
    theirs_commit=$(label_commit "$theirs_label") || true
    if [ -n "$base_commit" ] && [ -n "$ours_commit" ] && [ -n "$theirs_commit" ] \
        && lock_blob_is "$base_commit" "$base" && lock_blob_is "$ours_commit" "$ours" \
        && lock_blob_is "$theirs_commit" "$theirs"; then
        # Lock paths are relative to the lock's directory; git's are to the top.
        dir=$(dirname "$path")
        prefix=${dir#.}
        prefix=${prefix:+$prefix/}
        while IFS= read -r file; do
            hash=$(merged_hash "$prefix$file")
            [ -z "$hash" ] || tree=$(jq -c --arg f "$file" --arg h "$hash" '.[$f] = $h' <<<"$tree")
        done <<<"$contested"
    else
        echo "outdatty-lock-merge: cannot tell which commits '$ours_label' and '$theirs_label' are," \
            "so a hash both sides changed in $path conflicts" >&2
    fi
fi

# Emit the lock as outdatty writes it: `version`, `algorithm`, `groups` first,
# `source` before `dependents`, every other map sorted, a plain scalar unquoted.
# A conflicting leaf goes between markers, ours above theirs.
emit='
def indent($d): [range($d)] | map("  ") | join("");
def scalar: if type == "string" and test("^[A-Za-z0-9_./-]+$")
    and (test("^[-+0-9.]+$|^(true|false|null|yes|no|on|off|y|n)$"; "i") | not)
    then . else tojson end;
def order($d): keys_unsorted as $k
    | (if $d == 0 then ["version", "algorithm", "groups"] elif $d == 2 then ["source", "dependents"] else [] end) as $first
    | ($first | map(select(. as $x | $k | index($x)))) + ($k - $first | sort);
def emit($d): . as $node | order($d)[] as $key | $node[$key] as $v
    | "\(indent($d))\($key | scalar):" as $head
    | if ($v | type) == "object" and ($v | has("\u0000conflict")) then $v["\u0000conflict"] as $c
        | "<<<<<<< \($ours)", (if $c.inA then "\($head) \($c.ours | scalar)" else empty end),
          "=======", (if $c.inB then "\($head) \($c.theirs | scalar)" else empty end), ">>>>>>> \($theirs)"
      elif $v == {} then "\($head) {}"
      elif ($v | type) == "object" then $head, ($v | emit($d + 1))
      else "\($head) \($v | scalar)" end;
def build: sort_by(.path | length)
    | reduce .[] as $e ({}; setpath($e.path;
        if $e.conflict then {"\u0000conflict": $e.conflict} else $e.value end));
'
args=(--argjson o "$o" --argjson a "$a" --argjson b "$b" --argjson tree "$tree")
merged=$(jq -nr "${args[@]}" --arg ours "$ours_label" --arg theirs "$theirs_label" \
    "$lib$emit"' merge($tree) | build | emit(0)')
printf '%s\n' "$merged" >"$ours"

conflicts=$(jq -nr "${args[@]}" "$lib"' merge($tree) | .[] | select(.conflict)
    | if .file then "group \(.path[1]), \(.path[2]) \(.file)" else "key \(.path | join("."))" end')
if [ -n "$conflicts" ]; then
    while IFS= read -r where; do
        echo "outdatty-lock-merge: $path: $where: both sides changed it and neither matches the merged tree" >&2
    done <<<"$conflicts"
    echo "  review that group's dependents against the merged tree, then run \`just outdatty-update <group>\`" >&2
    exit 1
fi
