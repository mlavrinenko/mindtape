#!/usr/bin/env bash
# .just/scripts/dry-reasons.sh: the reasons sidecar check for `lint::dry` and
# `lint::dry-baseline`. Runs from the repo root, like the recipes that invoke
# it.
#
# The dry gate tolerates every clone whose fingerprint is in
# .config/jscpd-baseline.json. The gate lets a tolerated clone through, so the
# baseline alone does not distinguish a coincidence from a clone nobody got
# to. This check makes .config/jscpd-baseline-reasons.json the audit trail: every
# fingerprint in the baseline needs a non-empty reason, and every reason needs
# a fingerprint in the baseline. A re-mint that adds a fingerprint without a
# reason fails here, so a new clone cannot enter the baseline silently. A fix
# that drops a fingerprint leaves its reason behind, and the second half fails
# until the reason goes too.
#
# The reasons file is a flat JSON object, fingerprint to sentence, sorted by
# fingerprint. The sentence states the clone's shape and says why no helper
# removes it, or why the match is a coincidence. The house style is
# .config/jscpd-repeats-reasons.json.
#
# The check reads the two files only: jscpd has already run, and its exit
# status is the recipe's. A missing baseline or reasons file fails with the
# command or the file that fixes it.
set -eo pipefail

BASELINE=.config/jscpd-baseline.json
REASONS=.config/jscpd-baseline-reasons.json

if [ ! -f "$BASELINE" ]; then
    echo "dry: $BASELINE is missing; run \`just lint dry-baseline\`" >&2
    exit 1
fi
if [ ! -f "$REASONS" ]; then
    echo "dry: $REASONS is missing; add one reason per tolerated clone" >&2
    exit 1
fi

# A reason is a non-empty string. A null, a number or an empty sentence is not
# a reason, and the gate refuses the file rather than counting the key.
if ! jq -e 'type == "object" and all(.[]; type == "string" and length > 0)' "$REASONS" >/dev/null 2>&1; then
    echo "dry: $REASONS must be an object of non-empty reason strings" >&2
    exit 1
fi

missing=$(jq -r --slurpfile reasons "$REASONS" \
    '.fingerprints | keys - ($reasons[0] | keys) | .[]' "$BASELINE")
extra=$(jq -r --slurpfile baseline "$BASELINE" \
    'keys - ($baseline[0].fingerprints | keys) | .[]' "$REASONS")

status=0
if [ -n "$missing" ]; then
    count=$(printf '%s\n' "$missing" | wc -l)
    if [ "$count" -eq 1 ]; then
        echo "dry: 1 baseline entry has no reason: $missing"
    else
        echo "dry: $count baseline entries have no reason:"
        printf '%s\n' "$missing" | sed 's/^/  /'
    fi
    echo "hint: fix the clone, or add its reason to $REASONS"
    status=1
fi
if [ -n "$extra" ]; then
    count=$(printf '%s\n' "$extra" | wc -l)
    if [ "$count" -eq 1 ]; then
        echo "dry: 1 reason has no baseline entry: $extra"
    else
        echo "dry: $count reasons have no baseline entry:"
        printf '%s\n' "$extra" | sed 's/^/  /'
    fi
    echo "hint: the clone left the baseline; drop its reason from $REASONS"
    status=1
fi
if [ "$status" -ne 0 ]; then
    exit 1
fi

count=$(jq '.fingerprints | length' "$BASELINE")
echo "dry: $count clones tolerated, every one explained"
