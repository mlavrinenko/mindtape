#!/usr/bin/env bash
# .just/scripts/_script-reach.sh — the code lines everything a recipe reaches
# would run, for a caller that judges the lines itself (check-gate-drift.sh reads
# them for a cargo build). Same walk as _script-graph.sh (_script-graph-lib.sh),
# so a shell form one learns the other learns; unlike it, recipe DEPENDENCIES
# are edges here, so `check` lands on every arm it hands to `mmz`, and on a
# module recipe's own `gen`.
#
# Usage: _script-reach.sh [--also GLOB]... <just-dump.json> <recipe>..., from a
# tree root holding `.just/scripts/*.sh`. `--also` adds scripts outside that
# directory as nodes (www/*.sh). Prints `where<TAB>line` for each recipe body
# line and each non-comment, non-heredoc script line the recipes reach, a
# recipe named `<recipe> body line N` and a script `path:N`.
set -eo pipefail

extra=()
while [ "${1:-}" = --also ]; do
    extra+=("${2:?--also needs a glob}")
    shift 2
done
if [ $# -lt 2 ]; then
    echo "script-reach: usage: _script-reach.sh [--also GLOB]... <just-dump.json> <recipe>..." >&2
    exit 1
fi
dump=$1
shift
reach=1

here=$(dirname "${BASH_SOURCE[0]}")
# shellcheck source=.just/scripts/_script-graph-lib.sh
source "$here/_script-graph-lib.sh"

declare -A seen=()
queue=()
for recipe in "$@"; do
    if ! grep -qxF "$recipe" <<<"$recipes"; then
        echo "script-reach: $recipe names no recipe in the dump" >&2
        exit 1
    fi
    seen["r:$recipe"]=1
    queue+=("r:$recipe")
done
while [ ${#queue[@]} -gt 0 ]; do
    node=${queue[0]}
    queue=("${queue[@]:1}")
    while IFS= read -r next; do
        [ -n "$next" ] || continue
        [ -z "${seen[$next]}" ] || continue
        seen["$next"]=1
        queue+=("$next")
    done <<<"${adjacent[$node]}"
done

awk -F'\t' 'NR == FNR { keep[$0] = 1; next } $1 in keep { print $2 "\t" $3 }' \
    <(printf '%s\n' "${!seen[@]}") <(printf '%s\n' "$lines")
