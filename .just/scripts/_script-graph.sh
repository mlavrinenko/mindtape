#!/usr/bin/env bash
# .just/scripts/_script-graph.sh — the recipe-to-script reachability that
# script-scope-check.sh's arm two judges a rule's declared mmz inputs against
# (truth/tasks/follow-script-to-script-calls-in-the-scope-derivation.typ).
#
# WHAT IT REPLACED. Arm two used to demand a scope for every `.just/scripts/*.sh`
# path a recipe body NAMED ITSELF. A script one hop further — script-to-script,
# or through a nested `just` — was invisible, so the caller's rule had to name
# that scope BY HAND or serve a stale hit against a body it never hashed, with
# nothing failing to say so. Four such scopes accumulated (_nav-diff.sh on two
# rules, the two since-retired gitleaks-bound scripts on `just check-secrets`) on top of
# check-crap's nested `just crap`, which is what a missing derivation looks like
# from the outside. Walking the closure instead makes a new caller covered by
# ARRIVING — the property the `.just/scripts/**` glob had and the enumeration
# gave up.
#
# ONE GRAPH FOR BOTH SHAPES, not two derivations: a recipe and a script are the
# same kind of node, a body that can name another body, and the hop kinds differ
# only in which edge reaches the next one. `just check-crap` reaches crap.sh
# through one recipe hop and _lcov-stale-reason.sh through two separate ones;
# nothing in the caller's manifest entry records which.
#
# A WRONG EDGE FAILS A CORRECT MANIFEST — the reason the old arm claimed direct
# invocation only, and the reason this reading is narrow where it could
# over-read and loud where it cannot read at all:
#
#   - A `.just/scripts/*.sh` path counts in ANY position, not only after a
#     `bash`: a script is as much a dependency read as run (positional-args-
#     check.sh greps mutants.sh; a script may assign another's path and run it
#     a line later). Three shapes are subtracted. A comment
#     line and a heredoc body are not code — the control fixtures WRITE scripts
#     with a heredoc, so reading one as code would make a fixture an edge of the
#     gate that runs it. The path must EXIST: size-fixture-check.sh names
#     `.just/scripts/_probe.sh`, which it synthesizes inside a replica, and no
#     scope can hash a file that is not there. And an `echo`/`printf` fragment
#     is skipped, that being how these scripts name a script they do not run.
#   - A nested `just` counts only at COMMAND POSITION — the start of a fragment
#     the line splits into at `; & | ( ) { } !`. A backquote is deliberately NOT
#     a command-substitution opener here: `$( )` is the invocation form these
#     scripts use, and reading a backquote as one would turn every "run
#     \`just cover\` first" hint into an edge demanding a scope for a body that
#     never runs.
#
# AND LOUD WHERE IT CANNOT READ. A command-position `just` resolves two ways: to
# a known recipe (an edge) or to a flag in the table below that runs no recipe of
# this justfile (no edge). Anything else — a variable, a flag nobody has
# classified — fails naming the file and line, because swallowing it would put
# back the silent miss this file exists to remove. One shape is still not
# walked, and is not a miss today: a recipe DEPENDENCY (`check-full: check cover
# crap`), which the dump reports separately from bodies and which no rule's
# scopes turn on.
#
# Static: it reads the dump it is handed and the script files under the tree it
# runs in. No recipe body and no script runs. Pinned by
# .just/scripts/_script-graph-control.sh, which drives it over a synthetic tree
# carrying each of these shapes at once.
#
# The walk itself (nodes, edges, what counts as a call) is in
# _script-graph-lib.sh, shared with _script-reach.sh.
#
# Usage: _script-graph.sh <just-dump.json>, from a tree root holding
# `.just/scripts/*.sh`. Prints `recipe<TAB>script<TAB>chain`, one line per
# (recipe, reachable script) pair, where the chain names the hops that reached
# it so a caller's failure message can say HOW rather than only THAT.
set -eo pipefail

if [ $# -ne 1 ]; then
    echo "script-graph: usage: _script-graph.sh <just-dump.json>" >&2
    exit 1
fi
dump=$1

here=$(dirname "${BASH_SOURCE[0]}")
# shellcheck source=.just/scripts/_script-graph-lib.sh
source "$here/_script-graph-lib.sh"

# A node's printable form: a recipe is named the way the manifest names its
# rule, a script by its path, so a chain reads as the sequence a reader can
# follow by opening files.
label() { case $1 in r:*) echo "just ${1#r:}" ;; *) echo "${1#s:}" ;; esac; }

# Breadth-first from each recipe, keeping the first parent that reached a node —
# so the chain reported is a shortest one, which is the one a reader has to
# check the fewest files to confirm.
for recipe in $recipes; do
    root="r:$recipe"
    [ -n "${adjacent[$root]}" ] || continue
    unset seen parent
    declare -A seen=() parent=()
    queue=("$root")
    seen["$root"]=1
    while [ ${#queue[@]} -gt 0 ]; do
        node=${queue[0]}
        queue=("${queue[@]:1}")
        while IFS= read -r next; do
            [ -n "$next" ] || continue
            [ -z "${seen[$next]}" ] || continue
            seen["$next"]=1
            parent["$next"]=$node
            queue+=("$next")
        done <<<"${adjacent[$node]}"
    done
    for node in "${!seen[@]}"; do
        case $node in s:*) ;; *) continue ;; esac
        chain=$(label "$node")
        walk=$node
        while [ -n "${parent[$walk]}" ]; do
            walk=${parent[$walk]}
            chain="$(label "$walk") -> $chain"
        done
        printf '%s\t%s\t%s\n' "$recipe" "${node#s:}" "$chain"
    done
done | sort -u
