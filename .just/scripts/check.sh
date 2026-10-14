#!/usr/bin/env bash
# .just/scripts/check.sh — the body of the `check` recipe
# (truth/tasks/print-nothing-from-a-passing-just-check-but-its-verdict.typ).
# Runs from the repo root, like the recipe that invokes it.
#
# Mostly an agent reads this output, after the run ends, and each line costs
# its context. So a pass prints one verdict line and a failure prints the full
# log of each failed gate and nothing from the gates that passed. Each arm
# writes its log and its outcome under target/check/ (mmz-arm.sh), and this
# script reads them back once the stages are done.
#
# The stages are the dependencies of `_check-stages`, run one `just` call each,
# in order, stopping at the first red stage. Stage names as arguments (`fast`,
# `build`, `cover`) keep only those stages, still in that order: CI runs
# `cover` in a job of its own. The stage calls' own output is just's, not a
# gate's: on a red arm it is only just's "Recipe failed" line, so it is printed
# only when no arm recorded a failure, which means just itself failed (a parse
# error, a missing recipe).
set -eo pipefail

dir=target/check
rm -rf "$dir"
mkdir -p "$dir"
export MINDTAPE_CHECK_DIR="$PWD/$dir"

# Each stage is called by its literal name, so script-scope-check can read the
# call graph. That spells the list twice, so a stage `_check-stages` gains and
# this script misses fails here rather than going unrun.
dump=$(just --dump --dump-format json)
listed=$(jq -r '[.recipes["_check-stages"].dependencies[].recipe] | join(" ")' <<<"$dump")
if [ "$listed" != "_check-fast _check-build _check-cover" ]; then
    echo "check: _check-stages lists '$listed', but check.sh runs '_check-fast _check-build _check-cover'" >&2
    exit 1
fi
# Whether stage $1 runs: every stage when no names follow it, else the named.
want() { [ "$#" -eq 1 ] || [[ " ${*:2} " == *" $1 "* ]]; }
selected=()
for stage in fast build cover; do
    if want "$stage" "$@"; then selected+=("_check-$stage"); fi
done

stages=$dir/stages.log
red=0
if want fast "$@"; then just _check-fast >>"$stages" 2>&1 || red=1; fi
if [ "$red" -eq 0 ] && want build "$@"; then just _check-build >>"$stages" 2>&1 || red=1; fi
if [ "$red" -eq 0 ] && want cover "$@"; then just _check-cover >>"$stages" 2>&1 || red=1; fi

total=$(jq --args '.recipes as $r | [$ARGS.positional[] | $r[.].dependencies[]] | length' \
    "${selected[@]}" <<<"$dump")

shopt -s nullglob
passed=0 run=0 cached=0 failed=()
for status in "$dir"/*.status; do
    IFS=$'\t' read -r outcome how name log <"$status"
    if [ "$outcome" = pass ]; then
        passed=$((passed + 1))
        if [ "$how" = cached ]; then cached=$((cached + 1)); else run=$((run + 1)); fi
    else
        failed+=("$name")
        echo "==> $name failed (log: $log)"
        cat "$log"
        echo
    fi
done

if [ "$red" -eq 0 ]; then
    echo "check: $passed gates passed ($run run, $cached cached)"
    exit 0
fi

if [ ${#failed[@]} -eq 0 ]; then
    cat "$stages"
    echo "check: failed outside any gate (log: $stages)"
    exit 1
fi

skipped=$((total - passed - ${#failed[@]}))
verdict="check: ${#failed[@]} of $total gates failed: $(printf '%s, ' "${failed[@]}")"
verdict=${verdict%, }
if [ "$skipped" -gt 0 ]; then verdict="$verdict ($skipped not run)"; fi
echo "$verdict"
exit 1
