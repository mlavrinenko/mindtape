#!/usr/bin/env bash
# .just/scripts/script-scope-check.sh — the body of the `script-scope-check`
# recipe (truth/tasks/split-gates-meta-into-per-script-scopes.typ). Runs from
# the repo root, like the recipe that invokes it.
#
# THE PRICE THIS PAYS. `.mmz/config.yaml`'s `gates-meta` scope used to carry
# `.just/scripts/**` as a glob, shared by every rule, and that glob was a
# deliberate choice with a real property: the directory is recipe bodies by
# construction, so a body extracted out of the Justfile was covered by
# ARRIVING, never by someone remembering to wire it up. It also meant editing
# one script re-ran every rule in the manifest — clippy, the full test suite
# and the docs build included — for a change to one gate's body. Splitting the
# glob into one scope per script buys that back and gives up the arriving-is-
# enough property, so the split is a net loss unless something closes it. This
# is that something: both halves of what the glob guaranteed, asserted instead
# of assumed.
#
# The three arms answer three different questions, and none implies another:
#
#   1. Is every script NAMED at all? A set difference between the files on disk
#      and the literal paths the scopes list. A script no scope names is
#      invisible to mmz — the rule that runs it serves a stale cache hit
#      against a body it never hashed, which is the false-fresh every comment
#      in .mmz/config.yaml argues against.
#   2. Is it named by the RIGHT rule? Arm one passes for a script listed in a
#      scope no rule reads, or read by some unrelated rule, and the stale hit
#      survives both. So the call graph from each rule's recipe body is walked
#      (.just/scripts/_script-graph.sh) and the rule's declared inputs must
#      cover every script that body REACHES — through another script and
#      through a nested `just`, not only the ones it names itself.
#   3. Is it coupled to the docs it can rot, or exempted on purpose? Same
#      enumeration problem, one file over: `outdatty.yaml`'s `dev-docs` group
#      lists these scripts one by one, and outdatty reacts only to files it
#      lists — so a script never added is coupled to no dependent and reported
#      by nothing (truth/tasks/fail-a-half-wired-gate-arm-at-the-wiring-it-is-
#      missing.typ). Membership is all this arm claims: WHICH dependents a
#      script owes is outdatty's own subject, and it answers that well once the
#      file is listed at all.
#
# ARM THREE DEMANDS A DECISION, NOT MEMBERSHIP, and the difference is the whole
# design. Not every script belongs in that group: `_nav-diff.sh` is deliberately
# outside it, with the reason recorded in outdatty.yaml — no doc restates its
# logic, so listing it would ask a reader to review seven dependents for a
# change that cannot rot any of them. Demanding membership outright would have
# forced exactly the coupling this repo declined on purpose, and demanding
# nothing lets the next script fall out silently. So a script must be a source
# OR carry an exemption below, and a NEW one is neither — which is the arriving-
# is-enough property restored: the file nobody decided about is the file that
# fails.
#
# ONLY WILDCARD-FREE PATHS COUNT as naming a script in arm one, and that is
# load-bearing rather than tidy. The obvious way to make a failure here go
# away is to put `.just/scripts/**` back into some scope; if a glob counted,
# that would satisfy this gate while restoring exactly the shared-scope churn
# the split removed. It does not count, so the only way to green is to name the
# file. The one glob that remains over this directory (`scripts-all`) therefore
# names nothing here — it exists for the three rules whose subject IS the whole
# directory, this one included: a check that has to notice a script ARRIVING
# cannot take an enumeration as its input, because the file it must react to is
# by definition the one nobody listed.
#
# ARM TWO IS TRANSITIVE, and did not start that way. It used to claim direct
# invocation only — the scripts a recipe body names itself — so a body reached
# one hop further was invisible and the caller's rule had to name that scope BY
# HAND: check-crap's nested `just crap`, then _nav-diff.sh on two rules, then
# two gitleaks-bound scripts on `just check-secrets` (since retired). That list growing was
# the signal to derive it instead (truth/tasks/follow-script-to-script-calls-in-
# the-scope-derivation.typ). The walk lives in .just/scripts/_script-graph.sh,
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

# recipe<TAB>script<TAB>chain, for every script a recipe body REACHES. The
# control runs first and fails loudly on its own, so a derivation that stopped
# following an edge cannot reach arm two and report every rule covered.
bash .just/scripts/_script-graph-control.sh
dump=$(mktemp)
trap 'rm -f "$dump"' EXIT
just --dump --dump-format json >"$dump"
reachable=$(bash .just/scripts/_script-graph.sh "$dump")

# ARM ONE: every script on disk is named by some scope, as a literal path.
for script in "${scripts[@]}"; do
    if ! awk -F'\t' -v p="$script" '
        $2 == p && $2 !~ /[][*?]/ { found = 1 } END { exit !found }
    ' <<<"$entries"; then
        echo "script-scope-check: $script is named by no scope in $manifest" >&2
        echo "  add a \`script-<stem>\` scope listing it, and name that scope from the rule whose body it is" >&2
        fail=1
    fi
done

# The same difference the other way: a scope naming a script that no longer
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

# ARM TWO: a rule whose recipe body reaches a script must declare a scope
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

# ARM THREE: every script is a `dev-docs` SOURCE in outdatty.yaml, or is
# exempted here with the reason. Read from outdatty.yaml rather than
# outdatty.lock — the lock records what was confirmed, so a script listed in
# neither is missing from both and the lock agrees with itself. The group name
# is hard-coded because exactly one group's subject is this directory; a second
# would make "which group does this script belong to" a real question, and this
# arm would have to be told the answer rather than assume it.
#
# One entry per exempt script, with the reason it owes no dependent. Each is a
# claim about the DOCS, not about the script: nothing anywhere restates its
# behaviour by hand, so no prose can go stale when it changes.
declare -A dev_docs_exempt=(
    # gate-notes.typ names it, but restates none of its own two-arm logic —
    # the reason outdatty.yaml already records for leaving it out.
    ["_nav-diff.sh"]="no doc restates its logic; gate-notes.typ only names it"
    # The machine-install scripts. install-lib.sh IS a dev-docs source, because
    # the contributing docs walk a contributor through `just install-lib`;
    # these four are not, because no doc describes what they put on a machine —
    # they are read off `just --list` and run.
    ["install.sh"]="no doc describes the install set; --list is the only entry point"
    ["install-bin.sh"]="no doc describes the binary install; --list is the only entry point"
    ["uninstall-lib.sh"]="no doc describes it; the inverse of install-lib, named only in its own [doc]"
    ["_typst-package.sh"]="a sourced preamble, no behaviour of its own for a doc to restate"
    # Same shape one gate over: the prose restates what the DERIVATION reads as
    # a call, which is _script-graph.sh's own subject and makes that file a
    # source. The control's four cases are what would change here, and no doc
    # names one.
    ["_script-graph-control.sh"]="no doc restates its cases; the derivation it drives is the documented part"
    # The same shape for check-gate-drift: the readers (_gate-drift-lib.sh) are
    # the source, gate-notes.typ restates what they read, and the control's
    # fixtures are the part no doc names.
    ["_gate-drift-control.sh"]="no doc restates its fixtures; the readers it drives are the documented part"
    # The same for check-doc-blocks: the scanner (_doc-blocks-lib.sh) is the
    # source, and no doc names the control's fixtures.
    ["_doc-blocks-control.sh"]="no doc restates its fixtures; the scanner it drives is the documented part"
)

dev_docs_sources=$(yq -r '
    .groups // [] | .[] | select(.name == "dev-docs") | (.source // [])[]
' outdatty.yaml)

if [ -z "$dev_docs_sources" ]; then
    echo "script-scope-check: outdatty.yaml declares no \`dev-docs\` sources" >&2
    echo "  the group this arm reads was renamed or removed — retarget the arm, do not delete it" >&2
    exit 1
fi

exempt=0
for script in "${scripts[@]}"; do
    stem=${script##*/}
    listed=0
    grep -qxF "$script" <<<"$dev_docs_sources" && listed=1
    # An exemption for a script that IS a source is a contradiction, and the
    # shape a half-finished change leaves behind: the file was listed, the
    # exemption never removed, and the next reader cannot tell which of the two
    # is the decision.
    if [ -n "${dev_docs_exempt[$stem]}" ]; then
        if [ "$listed" -eq 1 ]; then
            echo "script-scope-check: $script is a \`dev-docs\` source AND exempt here" >&2
            echo "  drop the exemption (\"${dev_docs_exempt[$stem]}\") or drop the source line" >&2
            fail=1
        else
            exempt=$((exempt + 1))
        fi
        continue
    fi
    if [ "$listed" -eq 0 ]; then
        echo "script-scope-check: $script is neither a \`dev-docs\` source nor exempt" >&2
        echo "  add \`      - $script\` to that group in outdatty.yaml, so editing it asks for the docs it can rot" >&2
        echo "  or add it to dev_docs_exempt in this script, with the reason no doc restates it" >&2
        fail=1
    fi
done

# The reverse difference, like arm one's: an exemption for a file that is gone
# outlives the rename that removed it and quietly exempts nothing.
for stem in "${!dev_docs_exempt[@]}"; do
    if [ ! -f ".just/scripts/$stem" ]; then
        echo "script-scope-check: dev_docs_exempt names $stem, which does not exist" >&2
        fail=1
    fi
done

if [ "$fail" -ne 0 ]; then
    echo "script-scope-check: the manifests that enumerate .just/scripts/ no longer cover it" >&2
    exit 1
fi
