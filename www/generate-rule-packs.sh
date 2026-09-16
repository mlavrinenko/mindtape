#!/usr/bin/env bash
# www/generate-rule-packs.sh — capture the `mindtape-tasks` pack and this
# repo's own `mt config show --origin`, for /rule-packs/.
#
# Split out of www/generate.sh (which calls this, still inside the lock it
# already holds) under the same 300-line Shell cap that split off
# generate-self-validating.sh — see that script's header for the reasoning
# this one does not repeat.
#
# Unlike every other capture in generate.sh, this one runs `mt` against THE
# REAL REPO, not the learn-piano fixture: the pack's dogfood use — a project
# override replacing one of the pack's own rules by name
# (`section-allowlist`), and a project-only rule the pack has no opinion on
# (`goals`) — is the worked example the page quotes, and only this repo's
# own config shows it. `config show --origin` is read-only (see
# CONTRIBUTING.md's command-class table), so running it here mutates
# nothing.
set -eo pipefail

REPO="$1"
OUT="$2"
MT="$REPO/target/debug/mt"

# The pack file itself, copied verbatim so the page can read() it under
# www's --root (same reason generate.sh copies hero-source.typ/template.typ
# out of examples/learn-piano instead of reading them in place).
cp "$REPO/crates/mindtape/src/packs/mindtape-tasks.toml" "$OUT/mindtape-tasks-pack.toml"

# config show --origin prints the project's config path absolute
# (canonicalized root); normalize it to a project-relative path the same way
# generate.sh's own config-origin.txt capture does for the fixture.
"$MT" -w "$REPO" --color never config show --origin \
  | sed "s#$REPO/##g" >"$OUT/rule-packs-origin.txt"
