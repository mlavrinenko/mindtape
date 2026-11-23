# contrib/ask-refactoring/goals.sh: the `== Goals` block a swept task carries.
#
# `contrib/ask-refactoring.sh` sources this file before it reads an arm, and
# `crates/mindtape-cli/tests/it/sweep_goals_e2e.rs` gates the block. It owns
# two functions. `sweep_goal_names` prints the registry's goal names, one per
# line. `goals_with_one_helps` turns one of them into the block an arm splices
# into `mt add`'s `goals` slot.
#
# The names come from the registries themselves. `goal-names.typ` imports
# truth/goals.typ and truth/process-goals.typ and exports their `goals` keys,
# and `typst query` prints them here. This file once held all ten names in a
# bash array. Adding or renaming a goal then broke every arm's `mt add`, and no
# gate compared the two lists. A fallback list would restore that drift and
# hide it. The derivation refuses instead, and names the dev shell that has
# typst and jq.
#
# The names cache in `_sweep_goal_names`. One query then serves the five arms.
# The last line here primes the cache as the file loads. A missing tool, an
# empty registry or a repeated key then fails before the first arm runs, not at
# its first `mt add`.

_sweep_goal_names=""

# The registry's goal names, one per line, in registry order.
sweep_goal_names() {
    if [ -n "$_sweep_goal_names" ]; then
        printf '%s\n' "$_sweep_goal_names"
        return 0
    fi
    local here root lib pkgs names repeated tool
    here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    root="$(cd "$here/../.." && pwd)"
    for tool in typst jq; do
        if ! command -v "$tool" >/dev/null; then
            echo "ask-refactoring: $tool is not on PATH; run the sweep from the dev shell" >&2
            return 1
        fi
    done
    # The registries import `@local/mindtape`, which must resolve to this
    # checkout's `lib/`, not to whichever version `just install::lib` last
    # wrote. docs/generate-md.sh links the same throwaway package path.
    lib="$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/lib/typst.toml" | head -1)"
    pkgs="$(mktemp -d)"
    mkdir -p "$pkgs/local/mindtape"
    ln -s "$root/lib" "$pkgs/local/mindtape/$lib"
    if ! names="$(typst query --root "$root" --package-path "$pkgs" \
        "$here/goal-names.typ" '<sweep-goals>' --field value --one | jq -r '.[]')"; then
        rm -rf "$pkgs"
        echo "ask-refactoring: cannot read the goal names from $here/goal-names.typ" >&2
        return 1
    fi
    rm -rf "$pkgs"
    if [ -z "$names" ]; then
        echo "ask-refactoring: truth/goals.typ and truth/process-goals.typ declare no goal" >&2
        return 1
    fi
    repeated="$(printf '%s\n' "$names" | sort | uniq -d)"
    if [ -n "$repeated" ]; then
        echo "ask-refactoring: the goal registries repeat: $repeated" >&2
        return 1
    fi
    _sweep_goal_names="$names"
    printf '%s\n' "$_sweep_goal_names"
}

# One `== Goals` block rating every goal: `$1` is the audience the single help
# serves, `$2` the goal it helps and `$3` the reason. Each sweep arm passes its
# own goal and reason, and the helper leaves that goal out of the neutral list.
# A goal neither registry declares refuses here. The arm's `mt add` then gets
# an empty required slot, and the goal rule reports the gap again, before
# anything lands in the backlog.
goals_with_one_helps() {
    local audience="$1" goal="$2" reason="$3"
    local goals neutral=() candidate joined
    goals="$(sweep_goal_names)"
    if ! printf '%s\n' "$goals" | grep -qxF -- "$goal"; then
        echo "ask-refactoring: '$goal' is not a goal in truth/goals.typ or truth/process-goals.typ" >&2
        echo "  fix the arm's goal, or add it to a registry; the goals are:" >&2
        printf '%s\n' "$goals" | sed 's/^/    /' >&2
        return 1
    fi
    while IFS= read -r candidate; do
        [ "$candidate" = "$goal" ] || neutral+=("$candidate")
    done <<<"$goals"
    joined="$(printf '%s, ' "${neutral[@]}")"
    printf '== Goals\n\n+ #Helps(%s, to: %s)[%s].\n+ #Neutral(%s).\n' \
        "$audience" "$goal" "$reason" "${joined%, }"
}

# Prime the cache. The arm that runs first must fail here, before any
# `mt add`, rather than inside its own command substitution, which discards
# the status.
sweep_goal_names >/dev/null
