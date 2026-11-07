#!/usr/bin/env bash
# .just/scripts/skill-evals-check.sh — the body of the `skill-evals-check`
# recipe. Runs from the repo root, like the recipe that invokes it.
#
# The evals have two arms. `assertions` grades the executor that loaded the
# skill, and `without.assertions` grades a consumer given only the prompt and
# `mt`. The second block is the bar the CLI must clear with no external
# documentation, so a missing or empty block would quietly drop that arm from a
# release run. This checks the shape and the fixture paths; it never calls a
# model. The manual loop is skill-creator's, and its results live in README.md.
set -eo pipefail

manifest=skills/mindtape/evals/evals.json

if ! jq -e . "$manifest" >/dev/null; then
    echo "skill-evals-check: $manifest is not valid JSON" >&2
    exit 1
fi

fail=0

# An eval with no with-skill assertions or no no-docs block cannot be graded on
# both arms.
while IFS= read -r id; do
    if ! jq -e --argjson id "$id" '
        .evals[] | select(.id == $id)
        | (.assertions | type == "array" and length > 0)
          and (.without | type == "object")
          and (.without.assertions | type == "array" and length > 0)
          and (all(.without.assertions[]; (.text | type == "string" and length > 0)))
    ' "$manifest" >/dev/null; then
        echo "skill-evals-check: eval $id has no non-empty assertions/without.assertions pair" >&2
        fail=1
    fi
done < <(jq -r '.evals[].id' "$manifest")

if [ "$(jq '.evals | length' "$manifest")" -eq 0 ]; then
    echo "skill-evals-check: $manifest lists no evals" >&2
    fail=1
fi

# A fixture may be renamed or retired, and a stale path fails every eval aimed
# at it at config load. Check both the shared map and each eval's own key.
while IFS=$'\t' read -r where path; do
    [ -n "$path" ] || continue
    if [ ! -d "$path" ]; then
        echo "skill-evals-check: $where names fixture $path, which is not a directory" >&2
        fail=1
    elif [ ! -f "$path/.mindtape/config.toml" ]; then
        echo "skill-evals-check: $where names $path, which has no .mindtape/config.toml" >&2
        fail=1
    fi
done < <(
    jq -r '.fixtures | to_entries[] | select(.key != "note") | "fixtures.\(.key)\t\(.value)"' "$manifest"
    jq -r '.evals[] | "eval \(.id) (\(.name))\t\(.fixture)"' "$manifest"
)

if [ "$fail" -ne 0 ]; then
    echo "skill-evals-check: $manifest does not carry a gradable no-docs arm" >&2
    exit 1
fi
