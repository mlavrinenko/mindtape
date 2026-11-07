#!/usr/bin/env bash
# .just/scripts/check-skill-drift.sh — the body of the `skill-drift` recipe.
# Runs from the repo root, like the recipe that invokes it.
#
# `just install` writes a `.mindtape-source-hash` stamp into each skill it
# copies. This compares that stamp with a fresh hash of `skills/<name>`. A
# harness dir with no copy for a skill stays quiet. A copy with no stamp fails
# as unverified, because it came from an install this check cannot vouch for.
# The check reads the harness dirs and writes nothing there.
#
# A mismatch or a missing stamp warns and fails the gate: `just check` prints
# a failed gate's log and drops a passing one, so a warning on a green gate
# would never reach the reader.
set -eo pipefail

# The one hash install and check share. `sort -z` over the paths makes it
# independent of the order the filesystem returns them in, and both callers
# hash from the repo root, so the values are comparable.
hash_tree() {
    find "$1" -type f -print0 |
        LC_ALL=C sort -z |
        xargs -0 -r sha256sum |
        sha256sum |
        cut -d' ' -f1
}

stamp=.mindtape-source-hash
status=0

shopt -s nullglob
for manifest in skills/*/SKILL.md; do
    name=$(basename "$(dirname "$manifest")")
    source="skills/$name"
    source_hash=$(hash_tree "$source")
    for rel in .claude/skills .config/opencode/skills; do
        installed="$HOME/$rel/$name"
        [ -f "$installed/SKILL.md" ] || continue
        if [ ! -f "$installed/$stamp" ]; then
            # shellcheck disable=SC2016 # the backticks are message text
            printf 'warning: ~/%s/%s has no %s; run `just install` or delete the copy\n' \
                "$rel" "$name" "$stamp" >&2
            status=1
            continue
        fi
        if [ "$(cat "$installed/$stamp")" != "$source_hash" ]; then
            # shellcheck disable=SC2016 # the backticks are message text
            printf 'warning: ~/%s/%s differs from %s; run `just install`\n' \
                "$rel" "$name" "$source" >&2
            status=1
        fi
    done
done

exit "$status"
