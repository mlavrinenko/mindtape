#!/usr/bin/env bash
# Body of `just install`, run after its install-bin and install-lib deps: the
# generated JSON Schema, then a copy of every skill in the harness dirs. Runs
# from the repo root, where `skills/` is.
#
# `defaults.toml` is deliberately not refreshed. It is seeded once and then
# hand-edited as the global config layer (crates/mindtape-cli/src/cmd/install/mod.rs),
# so --force there would erase local settings; the schema is generated from the
# config structs, so forcing it is always safe.
set -eo pipefail
"${CARGO_HOME:-$HOME/.cargo}/bin/mt" install --no-lib --no-defaults --force

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
        echo "copied skill -> $target"
    done
done
