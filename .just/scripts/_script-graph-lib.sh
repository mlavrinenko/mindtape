# .just/scripts/_script-graph-lib.sh — the walk both front-ends share: reads the
# `just` dump and the scripts on disk and leaves the call graph in `adjacent`
# (node -> newline-separated nodes it names), the recipe names in `recipes` and
# every code line in `lines` (owner<TAB>where<TAB>line). Sourced by
# _script-graph.sh (which prints the recipe-to-script pairs script-scope-check
# judges) and by _script-reach.sh (which prints the code lines a recipe
# reaches); what counts as a call is argued below, once, for both.
#
# The caller sets `dump`, and optionally `extra` (globs naming scripts outside
# .just/scripts/ to add as nodes) and `reach=1` (recipe dependencies become
# edges too; the default leaves them, since no rule's scopes turn on one).
#
# A script is named by its path in any position, and a `$VAR/name.sh` also
# resolves beside the script naming it — how www/ and .just/scripts/ call their
# own siblings, and how the front-ends source this file.

shopt -s nullglob
scripts=(.just/scripts/*.sh)
if [ ${#scripts[@]} -eq 0 ]; then
    echo "script-graph: no .just/scripts/*.sh under $PWD — running outside a tree root?" >&2
    exit 1
fi

# The node set: .just/scripts/*.sh, plus whatever `extra` globs name.
allscripts=("${scripts[@]}")
for glob in "${extra[@]}"; do
    # shellcheck disable=SC2206 # the glob is the point
    allscripts+=($glob)
done

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
--evaluate	prints a variable of the justfile, runs no recipe
EOF
)

# One pass over the dump for both things it holds: the recipe names (`N`), which
# are what a nested `just` has to resolve to, and the body lines (`L`), which are
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
       | "L\tr:\($recipe)\t\($recipe) body line \($n + 1)\t\(.)"),
      ((.dependencies // [])[]
       | ([.recipe] + (.arguments // []) | map(select(type == "string")))[]
       | "D\tr:\($recipe)\tr:\(if contains("::") then . else ($recipe | sub("[^:]*$"; "")) + . end)")
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
    # A heredoc body is data, and the control fixtures write scripts carrying
    # `just` calls and script paths with one — so reading a heredoc as code
    # would make the control's own fixture an edge of the gate that runs it.
    # The opening line is still code (it is where a redirection or a `cat` is).
    # `<<<` is a herestring, not a heredoc, and the delimiter pattern requires a
    # word character where a herestring has its third `<`.
    for script in "${allscripts[@]}"; do
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
        # `p` as a whole path: not the tail of a longer name, nor the head of one.
        function mention(text, p,   pos, before, after) {
            while ((pos = index(text, p)) > 0) {
                before = (pos > 1) ? substr(text, pos - 1, 1) : ""
                after = substr(text, pos + length(p), 1)
                if (before !~ /[A-Za-z0-9_-]/ && after !~ /[A-Za-z0-9_-]/) return 1
                text = substr(text, pos + 1)
            }
            return 0
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
                    for (path in script)
                        if (mention(f, path)) edge[owner "\ts:" path "\t" where] = 1
                    dir = (owner ~ /^s:/) ? substr(owner, 3) : ""
                    sub(/\/[^\/]*$/, "", dir)
                    rest = f
                    while (dir != "" && match(rest, /\$\{?[A-Za-z_][A-Za-z0-9_]*\}?\/[A-Za-z0-9._-]+\.sh/)) {
                        path = substr(rest, RSTART, RLENGTH)
                        rest = substr(rest, RSTART + RLENGTH)
                        sub(/^.*\//, "", path)
                        if ((dir "/" path) in script) edge[owner "\ts:" dir "/" path "\t" where] = 1
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
        <(printf '%s\n' "${allscripts[@]}") \
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

if [ -n "${reach:-}" ]; then
    declare -A is_recipe=()
    for recipe in $recipes; do is_recipe["r:$recipe"]=1; done
    while IFS=$'\t' read -r from to; do
        if [ -n "${is_recipe[$to]}" ]; then adjacent["$from"]+="$to"$'\n'; fi
    done < <(grep '^D	' <<<"$dumped" | cut -f2- || true)
fi

