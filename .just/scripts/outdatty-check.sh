#!/usr/bin/env bash
# .just/scripts/outdatty-check.sh — the body of the `outdatty-check` recipe.
# Runs from the repo root, like the recipe that invokes it.
#
# `outdatty check` compares each recorded hash to the file's current bytes, so a
# dependent that a glob expands to but that the lock holds no hash for is never
# compared: a new page under `www/content/cli/*.typ` stayed unrecorded and
# unwatched with every gate green (21 pages did, until 22b1024e). This script
# runs the stock check, then expands each glob dependent and names every match
# the group's lock entry lacks, with the command that records it. Only presence
# is checked, not whether the recorded hash was reviewed.
#
# `OUTDATTY_LOCK` points both steps at another lock, which is how a test deletes
# an entry without touching outdatty.lock.
set -eo pipefail

lock="${OUTDATTY_LOCK:-outdatty.lock}"
missing=0
outdatty --lock "$lock" check --format paths || missing=1
while IFS=$'\t' read -r group pattern; do
    while IFS= read -r file; do
        [ -n "$file" ] || continue
        if ! yq -e --arg g "$group" --arg f "$file" '.groups[$g].dependents[$f]' "$lock" > /dev/null 2>&1; then
            echo "unrecorded: $file"
            echo "  record it: just outdatty-update $group $file"
            missing=1
        fi
    done < <(git ls-files --cached --others --exclude-standard -- ":(glob)$pattern")
done < <(outdatty --lock "$lock" status --format json \
    | jq -r '.groups[] | .id as $g | .dependents[] | select(test("[*?\\[]")) | [$g, .] | @tsv')
exit "$missing"
