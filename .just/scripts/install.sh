#!/usr/bin/env bash
# Body of `just install`, run after its install-bin and install-lib deps: the
# generated JSON Schema, then a copy of every skill in the harness dirs. Runs
# from the repo root, where `skills/` is.
#
# Each copied skill gets a `.mindtape-source-hash` stamp of two lines. The
# first is the hash of the source tree it copied, and the second is the
# absolute path of the checkout that copied it. The path lets `just check`
# tell a stale copy of this checkout from a sibling worktree's install. A
# sibling's copy is that worktree's to verify
# (.just/scripts/check-skill-drift.sh).
#
# `defaults.toml` is deliberately not refreshed. The install seeds it once, and the file then
# takes hand edits as the global config layer (crates/mindtape-cli/src/cmd/install/mod.rs).
# `--force` there would erase local settings. The generator builds the schema from the config
# structs, so forcing it is always safe.
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

# A SKILL.md is what makes a directory a skill, and the discriminator
# matters. git ignores `skills/*-workspace` as eval scratch, so a bare
# `skills/*/` glob would link a sandbox tree into the harness as a real skill.
#
# `pwd -P` resolves a symlinked path, so the checkout a stamp names is the one
# the drift check resolves to from the same directory. Both run from the repo
# root, so this is the root.
checkout=$(pwd -P)
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
        {
            hash_tree "skills/$name"
            printf '%s\n' "$checkout"
        } >"$target/.mindtape-source-hash"
        echo "copied skill -> $target"
    done
done
