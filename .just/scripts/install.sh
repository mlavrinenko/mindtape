#!/usr/bin/env bash
# Body of `just install`, run after its install-bin and install-lib deps: the
# generated JSON Schema, then every skill linked into the harness dirs. Runs
# from the repo root — `$(pwd)/skills` is the link target.
#
# `defaults.toml` is deliberately not refreshed. It is seeded once and then
# hand-edited as the global config layer (crates/mindtape-cli/src/install.rs),
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
        # Same trap as install-lib.sh: `ln -sfn` onto a real directory drops
        # the link inside it instead of replacing it.
        [ -L "$target" ] || rm -rf "$target"
        ln -sfn "$(pwd)/skills/$name" "$target"
        echo "linked skill -> $target"
    done
done
