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
# rules, the two gitleaks-bound scripts on `just check-secrets`) on top of
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
#     check.sh greps mutants.sh; _gitleaks-bound-control.sh assigns the check's
#     path and runs it a line later). Three shapes are subtracted. A comment
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

shopt -s nullglob
scripts=(.just/scripts/*.sh)
if [ ${#scripts[@]} -eq 0 ]; then
    echo "script-graph: no .just/scripts/*.sh under $PWD — running outside a tree root?" >&2
    exit 1
fi

# The `just` flags that reach no recipe of the justfile being read, each with
# the reason it reaches none. A flag absent from here is not assumed harmless:
# the walk fails on it and asks for this decision, because "it probably runs
# nothing" is how a real call stops being followed.
noop_flags=$(
    cat <<'EOF'
--justfile	reads a different justfile; no recipe of this one is reached
-f	reads a different justfile; no recipe of this one is reached
--dry-run	prints what a recipe would do without running its body
-n	prints what a recipe would do without running its body
--dump	prints the justfile itself, runs no recipe
--fmt	rewrites the justfile, runs no recipe
--list	prints the recipe list, runs no recipe
EOF
)

# One pass over the dump for both things it holds: the recipe NAMES (`N`), which
# are what a nested `just` has to resolve to, and the body LINES (`L`), which are
# where a call can be written. Modules are walked recursively (`docs::md-check`'s
# body is the reason) and a `namepath` is already fully qualified, so it maps
# straight onto the `just <namepath>` rule names the manifest uses. A body line
# is a list of string and `{{ VAR }}` fragments; the variable fragments are
# dropped and the rest joined, and just's own line prefixes (`@` quiet, `-`
# ignore-error) are stripped so `@just --list` reads as a call rather than as a
# word opening with `@`.
dumped=$(jq -r '
    def all_recipes:
      ((.recipes // {}) | to_entries | map(.value))
      + ((.modules // {}) | to_entries | map(.value) | map(all_recipes) | add // []);
    all_recipes[]
    | .namepath as $recipe
    | "N\t\($recipe)",
      ((.body // []) | to_entries[]
       | .key as $n
       | (.value | map(select(type == "string")) | join(" "))
       | sub("^[ \t]+"; "") | sub("^[@-]+"; "")
       | select(startswith("#") | not)
       | "L\tr:\($recipe)\t\($recipe) body line \($n + 1)\t\(.)")
' "$dump")

recipes=$(grep '^N	' <<<"$dumped" | cut -f2 | sort -u)
if [ -z "$recipes" ]; then
    echo "script-graph: $dump lists no recipes — not a \`just --dump-format json\` dump?" >&2
    exit 1
fi

# owner<TAB>where<TAB>line, for every line that could carry a call: the dump's
# body lines, and every non-comment line of every script on disk.
lines=$(
    grep '^L	' <<<"$dumped" | cut -f2-
    # A heredoc body is DATA, and the control fixtures write scripts carrying
    # `just` calls and script paths with one — so reading a heredoc as code
    # would make the control's own fixture an edge of the gate that runs it.
    # The opening line is still code (it is where a redirection or a `cat` is).
    # `<<<` is a herestring, not a heredoc, and the delimiter pattern requires a
    # word character where a herestring has its third `<`.
    for script in "${scripts[@]}"; do
        awk -v owner="s:$script" -v file="$script" '
            { sub(/^[ \t]+/, "") }
            heredoc != "" { if ($0 == heredoc) heredoc = ""; next }
            /^#/ { next }
            {
                print owner "\t" file ":" NR "\t" $0
                if (match($0, /<<-?[ \t]*['"'"'"]?[A-Za-z_][A-Za-z0-9_]*['"'"'"]?/)) {
                    delim = substr($0, RSTART, RLENGTH)
                    sub(/^<<-?[ \t]*/, "", delim)
                    gsub(/['"'"'"]/, "", delim)
                    heredoc = delim
                }
            }
        ' "$script"
    done
)

# Phase 1 is the recipe set, 2 the script set, 3 the flag table, 4 the lines.
# `FNR == 1` advances the phase on each new input, stdin included.
edges=$(
    awk -F'\t' '
        function trim(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }
        function unkeyword(s,   prev) {
            do {
                prev = s
                sub(/^(if|then|else|elif|do|while|until|time|exec|command|eval)[ \t]+/, "", s)
            } while (s != prev)
            return s
        }
        function complain(where, msg) {
            print "script-graph: " where ": " msg > "/dev/stderr"
            bad = 1
        }
        function resolve(owner, where, tok) {
            if (tok ~ /^-/) {
                if (tok in noop) return
                complain(where, "`just " tok "` carries a flag this walk has not been told about")
                print "  classify it in _script-graph.sh: either it still runs a recipe of this" > "/dev/stderr"
                print "  justfile (hoist the recipe name ahead of it) or it runs none (add it to" > "/dev/stderr"
                print "  noop_flags with the reason)" > "/dev/stderr"
                return
            }
            if (tok in recipe) { edge[owner "\tr:" tok "\t" where] = 1; return }
            if (tok == "") {
                complain(where, "`just` with no statically readable target")
                return
            }
            complain(where, "`just " tok "` names no recipe in the dump")
        }
        FNR == 1 { phase++ }
        phase == 1 { recipe[$0] = 1; next }
        phase == 2 { script[$0] = 1; next }
        phase == 3 { noop[$1] = $2; next }
        {
            owner = $1; where = $2; line = $3
            split_line = line
            gsub(/[;&|(){}!]/, "\n", split_line)
            n = split(split_line, frag, "\n")
            for (i = 1; i <= n; i++) {
                f = unkeyword(trim(frag[i]))
                word = f
                sub(/[ \t].*$/, "", word)
                if (word != "echo" && word != "printf") {
                    rest = f
                    while (match(rest, /\.just\/scripts\/[A-Za-z0-9._-]+\.sh/)) {
                        path = substr(rest, RSTART, RLENGTH)
                        rest = substr(rest, RSTART + RLENGTH)
                        if (path in script) edge[owner "\ts:" path "\t" where] = 1
                    }
                }
                if (f !~ /^just([ \t]|$)/) continue
                sub(/^just[ \t]*/, "", f)
                tok = f
                sub(/[ \t].*$/, "", tok)
                resolve(owner, where, tok)
            }
        }
        END {
            if (bad) exit 1
            for (e in edge) print e
        }
    ' <(printf '%s\n' "$recipes") \
        <(printf '%s\n' "${scripts[@]}") \
        <(printf '%s\n' "$noop_flags") \
        - <<<"$lines" | sort -u
)

declare -A adjacent=()
declare -A hop=()
while IFS=$'\t' read -r from to where; do
    [ -n "$from" ] || continue
    adjacent["$from"]+="$to"$'\n'
    [ -n "${hop["$from|$to"]}" ] || hop["$from|$to"]=$where
done <<<"$edges"

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
