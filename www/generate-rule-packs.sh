#!/usr/bin/env bash
# www/generate-rule-packs.sh — capture the `mindtape-tasks` pack, this repo's
# own `mt config show --origin`, and three configs `mt` refuses, for
# /rule-packs/.
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

# shellcheck source=www/capture.sh
source "$REPO/www/capture.sh"
capture_init
capture rule-packs-origin "$REPO" 'mt config show --origin'

# Three projects whose config names a pack `mt` refuses. Each record shows the
# config, then the refusal.
refused() { # <slug> <show-command> <file=content>...
  local slug=$1 show=$2 dir pair
  shift 2
  capture_scratch dir
  mkdir -p "$dir/.mindtape"
  for pair in "$@"; do printf '%s' "${pair#*=}" >"$dir/${pair%%=*}"; done
  capture "$slug" "$dir" "$show"
  capture --exit 1 "$slug" "$dir" 'mt check'
}
refused rule-packs-unknown 'cat .mindtape/config.toml' \
  $'.mindtape/config.toml=extends = ["mindtape-task"]\n'
refused rule-packs-shape 'head .mindtape/config.toml team.toml' \
  $'.mindtape/config.toml=extends = ["./team.toml"]\n' \
  $'team.toml=[[tasks.rules]]\nname = "has-owner"\nassert = "has(assignee)"\nmessage = "name an owner"\n\n[tasks.priority]\nmax = 5\n'
refused rule-packs-disable 'cat .mindtape/config.toml' \
  $'.mindtape/config.toml=extends = ["mindtape-tasks"]\ndisable = ["imperative-titles"]\n'
