#!/usr/bin/env bash
# Body of `just install`, run after its install-bin and install-lib deps: the
# generated JSON Schema, then a copy of every skill in the harness dirs. Runs
# from the repo root, where `skills/` is.
#
# Each copied skill gets a `.mindtape-source-hash` stamp holding the hash of
# the source tree it copied, so `just check` can tell a stale copy from a
# current one (.just/scripts/check-skill-drift.sh).
#
# `defaults.toml` is deliberately not refreshed. It is seeded once and then
# hand-edited as the global config layer (crates/mindtape-cli/src/cmd/install/mod.rs),
# so --force there would erase local settings; the schema is generated from the
# config structs, so forcing it is always safe.
set -eo pipefail
"${CARGO_HOME:-$HOME/.cargo}/bin/mt" install --no-lib --no-defaults --force

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

# A SKILL.md is what makes a directory a skill, and the discriminator is
# load-bearing: `skills/*-workspace` is gitignored eval scratch, so a bare
# `skills/*/` glob would link a sandbox tree into the harness as a real skill.
shopt -s nullglob
for dir in "$HOME/.claude/skills" "$HOME/.config/opencode/skills"; do
    [ -d "$dir" ] || continue
    for manifest in skills/*/SKILL.md; do
        name=$(basename "$(dirname "$manifest")")
        target="$dir/$name"
        # A copy, not a link: the skill stays matched to the installed `mt`
        # while this checkout moves on. `rm -rf` on a link from an older
        # install removes the link alone.
        rm -rf "$target"
        cp -R "skills/$name" "$target"
        hash_tree "skills/$name" >"$target/.mindtape-source-hash"
        echo "copied skill -> $target"
    done
done
