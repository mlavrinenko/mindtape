#!/usr/bin/env bash
# .just/scripts/outdatty-review-trailer.sh <msg-file> — the fourth check of
# .githooks/commit-msg (truth/tasks/require-a-docs-unaffected-trailer-when-an-
# outdatty-re-record-edits-no-dependent.typ). Runs from the repo root, as git
# runs a hook. Reads the index, HEAD and the message, and writes nothing.
# .just/scripts/outdatty-review-trailer-fixture-check.sh holds its cases.
#
# A re-record (`just outdatty::update <group>`) moves the group's source hashes
# in .config/outdatty.lock and says someone read the group's dependents. When the
# commit edits one of them, the diff shows the review. When it edits none, the
# lock alone can't tell a review that found nothing from one nobody did, and
# the dependents' hashes the re-record carried along don't help either. So such
# a commit names each group it read and left as is:
#
#   Docs-Unaffected: cli-ls-docs
#
# A group "moved" when a source path in the staged lock is new or has another
# hash than at the base. A source dropped from a group owes no review. A group
# the base lock lacks is skipped: the commit that declares a group is the one
# that chose its dependents. So is a group with no dependents. A dependent
# counts only when the commit changes the file itself.
#
# Why a script and not a `[[commits.rules]]` entry. A commit rule reads the
# commit's `files`, `subject` and `trailers`, never a file's content, so it
# can't see which group a lock hunk belongs to or pair that group with its
# dependents. Teaching the filter language to parse outdatty's YAML would put
# one dev tool's format into Mindtape. A coarse rule ("the lock changed, no doc
# did, so want any Docs-Unaffected") would refuse a dependent-only record and
# accept a trailer naming the wrong group.
#
# The base is HEAD, or HEAD^ under `git commit --amend`, found the way
# crates/mindtape/src/commit/amend.rs finds it: git tells a hook neither, so
# walk up to the `git commit` process and look for the flag. A merge commit
# passes: its diff against the first parent is the other branch's work, whose
# commits carried their own reviews. A rebase pick runs no commit-msg hook, so
# `just land` passes through, and the review a zeroed hash asks for after it
# (.just/scripts/outdatty-lock-merge.sh) is a new commit that names the group.
set -eo pipefail

msg_file=$1
lock=.config/outdatty.lock

git rev-parse -q --verify MERGE_HEAD >/dev/null && exit 0

# Whether the nearest `git commit` above this process carries `--amend`. No
# `ps -o` (Windows) means no.
amending() {
    local pid=$$ depth ppid args word is_commit
    local -a argv
    for depth in 1 2 3 4 5 6 7 8 9 10 11 12; do
        read -r ppid args < <(ps -o ppid= -o args= -p "$pid" 2>/dev/null) || return 1
        read -r -a argv <<<"$args"
        is_commit=
        case "${argv[0]##*[/\\]}" in
            git | git.exe)
                for word in "${argv[@]:1}"; do [ "$word" = commit ] && is_commit=1; done
                ;;
        esac
        if [ -n "$is_commit" ]; then
            for word in "${argv[@]:1}"; do [ "$word" = --amend ] && return 0; done
            return 1
        fi
        [ "${ppid:-0}" -gt 1 ] || return 1
        pid=$ppid
        : "$depth"
    done
    return 1
}

empty_tree=$(git hash-object -t tree /dev/null)
if ! git rev-parse -q --verify HEAD >/dev/null; then
    base=$empty_tree
elif amending; then
    base=$(git rev-parse -q --verify 'HEAD^') || base=$empty_tree
else
    base=HEAD
fi

git diff --cached --quiet "$base" -- "$lock" && exit 0

# A lock's `groups` map as JSON, `{}` where the revision has no lock.
groups() {
    local blob
    blob=$(git show "$1:$lock" 2>/dev/null) || blob=
    if [ -z "$blob" ]; then
        echo '{}'
    else
        printf '%s\n' "$blob" | yq -c '.groups // {}'
    fi
}
old=$(groups "$base")
new=$(groups "")

changed=$(git diff --cached --name-only --no-renames "$base" | jq -R -s -c 'split("\n") | map(select(. != ""))')
named=$(git interpret-trailers --parse "$msg_file" \
    | sed -n 's/^[Dd][Oo][Cc][Ss]-[Uu][Nn][Aa][Ff][Ff][Ee][Cc][Tt][Ee][Dd]: *//p' \
    | jq -R -s -c 'split("\n") | map(sub("\\s+$"; "")) | map(select(. != ""))')

# One line per group the commit owes a trailer: the name, then its dependents.
owed=$(jq -r -n --argjson old "$old" --argjson new "$new" \
    --argjson changed "$changed" --argjson named "$named" '
    $new | to_entries[]
    | .key as $g | (.value.source // {}) as $src | ($old[$g].source // {}) as $was
    | select($old | has($g))
    | select([$src | to_entries[] | select($was[.key] != .value)] | length > 0)
    | [(.value.dependents // {}) | keys[]] as $deps
    | select($deps | length > 0)
    | select(any($deps[]; . as $d | $changed | index($d)) | not)
    | select($named | index($g) | not)
    | ([$g] + $deps) | join(" ")')

[ -z "$owed" ] && exit 0

{
    echo "commit-msg: .config/outdatty.lock records a review of these groups, but this commit"
    echo "changes none of their dependents:"
    echo
    while read -r group deps; do
        echo "  $group"
        for dep in $deps; do echo "    $dep"; done
    done <<<"$owed"
    echo
    echo "Read each dependent and edit the one that drifted. If they all still hold,"
    echo "add a footer line per group reading exactly:"
    echo
    while read -r group _; do echo "    Docs-Unaffected: $group"; done <<<"$owed"
    echo
    echo "(see CONTRIBUTING.md, Dependency Drift) and commit again."
} >&2
exit 1
