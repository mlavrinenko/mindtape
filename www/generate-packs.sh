#!/usr/bin/env bash
# www/generate-packs.sh — capture every built-in pack, this repo's
# own `mt config show --origin`, a pack's priority merged below the project's,
# and four configs `mt` refuses, for /packs/.
#
# Split out of www/generate.sh (which calls this, still inside the lock it
# already holds) under the same 300-line Shell cap that split off
# generate-self-validating.sh — see that script's header for the reasoning
# this one does not repeat.
#
# Unlike every other capture in generate.sh, this one runs `mt` against the
# real repo, not a fixture: the pack's dogfood use — a project
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

# Every pack file, copied verbatim to packs/<pack>.toml so a page can read()
# it under www's --root (same reason generate.sh copies hero-source.typ and
# template.typ out of examples/api-service instead of reading them in place).
mkdir -p "$OUT/packs"
cp "$REPO"/crates/mindtape/src/packs/*.toml "$OUT/packs/"

# shellcheck source=www/capture.sh
source "$REPO/www/capture.sh"
capture_init
capture packs-origin "$REPO" 'mt config show --origin'

# A pack's [tasks.priority] under the project's own: the project's key wins,
# the pack's other key stays, and the floor fills the rest.
merge=
capture_scratch merge
mkdir -p "$merge/.mindtape"
printf 'extends = ["./team.toml"]\n\n[tasks.priority]\nallow-manual = true\n' \
  >"$merge/.mindtape/config.toml"
printf '[tasks.priority]\nframework = "ice"\nallow-manual = false\n' >"$merge/team.toml"
capture packs-merge "$merge" 'head .mindtape/config.toml team.toml'
capture packs-merge "$merge" \
  "mt config show --json | jq -c '.tasks.priority | {framework, \"allow-manual\", min, max}'"

# Four projects whose config names a pack `mt` refuses. Each record shows the
# config, then the refusal, which must match <expect>.
refused() { # <slug> <expect> <show-command> <file=content>...
  local slug=$1 expect=$2 show=$3 dir pair
  shift 3
  capture_scratch dir
  mkdir -p "$dir/.mindtape"
  for pair in "$@"; do printf '%s' "${pair#*=}" >"$dir/${pair%%=*}"; done
  capture "$slug" "$dir" "$show"
  capture --exit 1 --expect "$expect" "$slug" "$dir" 'mt check'
}
refused packs-unknown 'unknown pack' 'cat .mindtape/config.toml' \
  $'.mindtape/config.toml=extends = ["mindtape:task"]\n'
refused packs-shape 'may not declare' 'head .mindtape/config.toml team.toml' \
  $'.mindtape/config.toml=extends = ["./team.toml"]\n' \
  $'team.toml=extends = ["mindtape:tasks"]\n\n[[tasks.rules]]\nname = "has-owner"\nassert = "has(assignee)"\nmessage = "name an owner"\n'
refused packs-conflict 'set different values' 'head .mindtape/config.toml team.toml ops.toml' \
  $'.mindtape/config.toml=extends = ["./team.toml", "./ops.toml"]\n' \
  $'team.toml=[tasks.priority]\nframework = "ice"\n' \
  $'ops.toml=[tasks.priority]\nframework = "rice"\n'
refused packs-disable 'no rule named' 'cat .mindtape/config.toml' \
  $'.mindtape/config.toml=extends = ["mindtape:tasks"]\ndisable = ["imperative-titles"]\n'
