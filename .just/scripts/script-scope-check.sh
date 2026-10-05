#!/usr/bin/env bash
# .just/scripts/script-scope-check.sh — the body of the `script-scope-check`
# recipe (truth/tasks/split-gates-meta-into-per-script-scopes.typ). Runs from
# the repo root, like the recipe that invokes it.
#
# What it costs. A shared `.just/scripts/**` glob covers a body extracted out
# of the Justfile by arriving, never by someone remembering to wire it up, but
# it makes editing one script re-run every rule in the manifest — clippy, the
# full test suite and the docs build included — for a change to one gate's
# body. One scope per script avoids that and gives up the arriving-is-enough
# property, so the split is a net loss unless something closes it. This script
# is that something: both halves of what the glob guarantees, asserted instead
# of assumed.
#
# The three arms answer three different questions, and none implies another:
#
#   1. Is every script named at all? A set difference between the files on disk
#      and the literal paths the scopes list. A script no scope names is
#      invisible to mmz — the rule that runs it serves a stale cache hit
#      against a body it never hashed, which is the false-fresh every comment
#      in .mmz/config.yaml argues against.
#   2. Is it named by the right rule? Arm one passes for a script listed in a
#      scope no rule reads, or read by some unrelated rule, and the stale hit
#      survives both. So the call graph from each rule's recipe body is walked
#      (.just/scripts/_script-graph.sh) and the rule's declared inputs must
#      cover every script that body reaches — through another script and
#      through a nested `just`, not only the ones it names itself.
#   3. Is it coupled to the docs it can rot, or placed on purpose where none
#      does? Same enumeration problem, one file over: `outdatty.yaml` lists
#      these scripts one by one in groups, and outdatty reacts only to files it
#      lists, so a script never added is coupled to no dependent and reported
#      by nothing (truth/tasks/fail-a-half-wired-gate-arm-at-the-wiring-it-is-
#      missing.typ). Membership is all this arm claims: which dependents a
#      script owes is outdatty's own subject, and it answers that well once the
#      file is listed at all.
#
# Arm three demands a decision, not a particular group, and the difference is
# the whole design. A script belongs to the group of the doc that restates it,
# so editing it flags that doc and no other. A script no doc restates belongs
# to the source-only group, which has no dependents, so listing it flags
# nothing. Either way somebody chose, and a new script is in neither, so the
# file nobody decided about is the file that fails.
#
# Only wildcard-free paths count as naming a script in arm one, and that is
# load-bearing rather than tidy. The obvious way to make a failure here go
# away is to put `.just/scripts/**` back into some scope; if a glob counted,
# that would satisfy this gate while restoring exactly the shared-scope churn
# the split removed. It does not count, so the only way to green is to name the
# file. The one glob that remains over this directory (`scripts-all`) therefore
# names nothing here — it exists for the three rules whose subject is the whole
# directory, this one included: a check that has to notice a script arriving
# cannot take an enumeration as its input, because the file it must react to is
# by definition the one nobody listed.
#
# Arm two is transitive. A body reached one hop further than the scripts a
# recipe body names itself is still checked, so no caller's rule has to name
# that scope by hand. The walk lives in .just/scripts/_script-graph.sh,
# which argues in place what it reads as a call and what it refuses to guess at;
# this file only asks it which scripts a rule reaches and compares that to the
# scopes the rule declares. .just/scripts/_script-graph-control.sh runs first,
# for the reason check-secrets.sh gives for its own control pair: a derivation
# that quietly stopped following an edge would report every rule covered, which
# is exactly what a correct manifest reports.
#
# Static, like check-doc-recipes.sh and positional-args-check.sh: it reads the
# manifest, the directory listing and the just dump. No recipe body runs.
set -eo pipefail

manifest=".mmz/config.yaml"
fail=0

shopt -s nullglob
scripts=(.just/scripts/*.sh)
if [ ${#scripts[@]} -eq 0 ]; then
    echo "script-scope-check: no .just/scripts/*.sh found — running outside the repo root?" >&2
    exit 1
fi

# scope<TAB>glob, every entry of every scope. A scope's value is either a bare
# list of globs or a `{gitignore, globs}` dict (the `lcov` scope is the second
# shape), so both are unwrapped here rather than one being assumed.
entries=$(yq -r '
    .scopes // {} | to_entries[]
    | .key as $scope
    | (if (.value | type) == "object" then (.value.globs // []) else .value end)[]
    | "\($scope)\t\(.)"
' "$manifest")

# rule<TAB>input-scope, for every rule in `commands`.
rule_inputs=$(yq -r '
    .commands // [] | .[] | .name as $rule | (.inputs // [])[] | "\($rule)\t\(.)"
' "$manifest")

# recipe<TAB>script<TAB>chain, for every script a recipe body reaches. The
# control runs first and fails loudly on its own, so a derivation that stopped
# following an edge cannot reach arm two and report every rule covered.
bash .just/scripts/_script-graph-control.sh
dump=$(mktemp)
trap 'rm -f "$dump"' EXIT
just --dump --dump-format json >"$dump"
reachable=$(bash .just/scripts/_script-graph.sh "$dump")

# Arm one: every script on disk is named by some scope, as a literal path.
for script in "${scripts[@]}"; do
    if ! awk -F'\t' -v p="$script" '
        $2 == p && $2 !~ /[][*?]/ { found = 1 } END { exit !found }
    ' <<<"$entries"; then
        echo "script-scope-check: $script is named by no scope in $manifest" >&2
        echo "  add a \`script-<stem>\` scope listing it, and name that scope from the rule whose body it is" >&2
        fail=1
    fi
done

# The same difference the other way: a scope naming a script that does not
# exists. Harmless to mmz (a glob matching nothing is legal), and exactly the
# line a rename leaves behind pointing at the old name while the rule it feeds
# quietly stops hashing anything.
while IFS=$'\t' read -r scope path; do
    [ -n "$path" ] || continue
    case "$path" in
        .just/scripts/*.sh) ;;
        *) continue ;;
    esac
    [[ "$path" == *[][*?]* ]] && continue
    if [ ! -f "$path" ]; then
        echo "script-scope-check: scope \`$scope\` names $path, which does not exist" >&2
        fail=1
    fi
done <<<"$entries"

# Arm two: a rule whose recipe body reaches a script must declare a scope
# covering it. A glob input counts here — unlike arm one, the question is only
# whether the body is hashed at all, and `scripts-all` hashes it.
checked=0
while IFS=$'\t' read -r recipe script chain; do
    [ -n "$recipe" ] || continue
    rule="just $recipe"
    grep -qxF "$rule" <<<"$(cut -f1 <<<"$rule_inputs")" || continue
    checked=$((checked + 1))
    covered=0
    while IFS= read -r scope; do
        while IFS= read -r pattern; do
            # shellcheck disable=SC2053  # pattern match, not string equality
            if [[ "$script" == $pattern ]]; then
                covered=1
                break 2
            fi
        done < <(awk -F'\t' -v s="$scope" '$1 == s { print $2 }' <<<"$entries")
    done < <(awk -F'\t' -v r="$rule" '$1 == r { print $2 }' <<<"$rule_inputs")
    if [ "$covered" -eq 0 ]; then
        echo "script-scope-check: rule \`$rule\` reaches $script but no scope in its inputs names it" >&2
        echo "  $chain" >&2
        echo "  it would serve a stale hit against a body it never hashed" >&2
        fail=1
    fi
done <<<"$reachable"

# Arm three: every script is a source of some outdatty.yaml group. Read from
# outdatty.yaml rather than outdatty.lock, because the lock records what was
# confirmed, so a script listed in neither is missing from both and the lock
# agrees with itself.
group_sources=$(yq -r '.groups // [] | .[] | (.source // [])[]' outdatty.yaml)

if [ -z "$group_sources" ]; then
    echo "script-scope-check: outdatty.yaml declares no group sources" >&2
    echo "  the key this arm reads was renamed or removed; retarget the arm, do not delete it" >&2
    exit 1
fi

for script in "${scripts[@]}"; do
    if ! grep -qxF "$script" <<<"$group_sources"; then
        echo "script-scope-check: $script is a source of no outdatty.yaml group" >&2
        echo "  add \`      - $script\` to the group of the doc that restates it, so editing it asks for that doc" >&2
        echo "  or to the source-only group scripts-no-doc, with the reason no doc restates it" >&2
        fail=1
    fi
done

if [ "$fail" -ne 0 ]; then
    echo "script-scope-check: the manifests that enumerate .just/scripts/ no longer cover it" >&2
    exit 1
fi
