#!/usr/bin/env bash
# www/generate-packs.sh — capture every built-in pack, this repo's
# own `mt config show --origin`, a pack's priority merged below the project's,
# a directory pack, a pack's parameter set from `with`, and nine configs
# `mt` refuses, for /packs/.
#
# Split out of www/generate.sh (which calls this, still inside the lock it
# already holds) under the same 300-line Shell cap that split off
# generate-self-validating.sh — see that script's header for the reasoning
# this one does not repeat.
#
# Unlike every other capture in generate.sh, this one runs `mt` against the
# real repo, not a fixture: the packs' dogfood use — one pack replacing
# another's rule by name (`section-allowlist`, in `mindtape:dogfood`), and a
# project-only rule no pack has an opinion on (`goals`) — is the worked example
# the page quotes, and only this repo's own config shows it. `config show --origin` is read-only (see
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

# A directory pack: its `pack.toml` is the pack, and the template beside it
# reaches `mt add --list`.
dirpack=
capture_scratch dirpack
mkdir -p "$dirpack/.mindtape" "$dirpack/team/templates"
printf 'extends = ["./team"]\n' >"$dirpack/.mindtape/config.toml"
: >"$dirpack/team/pack.toml"
cp "$REPO/examples/api-service/.mindtape/templates/task.typ" "$dirpack/team/templates/feat.typ"
capture packs-dir "$dirpack" 'find team -type f'
capture packs-dir "$dirpack" 'mt add --list'

# A pack's param() call, written in with the project's `with` value.
params=
capture_scratch params
mkdir -p "$params/.mindtape"
printf 'extends = [{ pack = "./team.toml", with = { max-files = 8 } }]\n' \
  >"$params/.mindtape/config.toml"
printf '[params]\nmax-files = 5\n\n[[tasks.rules]]\nname = "small"\nassert = %s\n' \
  "'estimate.src-files <= param(\"max-files\")'" >"$params/team.toml"
capture packs-params "$params" 'head .mindtape/config.toml team.toml'
capture packs-params "$params" \
  "mt config show --json | jq -r '.tasks.rules[] | select(.name == \"small\") | .assert'"

# Nine projects whose config `mt` refuses. Each record shows the
# config, then the refusal, which must match <expect>.
refused() { # <slug> <expect> <show-command> <file=content>...
  local slug=$1 expect=$2 show=$3 dir pair
  shift 3
  capture_scratch dir
  mkdir -p "$dir/.mindtape"
  for pair in "$@"; do
    mkdir -p "$(dirname "$dir/${pair%%=*}")"
    printf '%s' "${pair#*=}" >"$dir/${pair%%=*}"
  done
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
refused packs-typo 'unknown field' 'head .mindtape/config.toml team.toml' \
  $'.mindtape/config.toml=extends = ["./team.toml"]\n' \
  $'team.toml=[views.board]\nfilter = "has(title)"\n\n[projet]\nname = "app"\n'
refused packs-bad-value 'views.board.filter' 'head .mindtape/config.toml team.toml' \
  $'.mindtape/config.toml=extends = ["./team.toml"]\n\n[views.mine]\nfilter = "has(title)"\n' \
  $'team.toml=[views.board]\nfilter = "(("\n'
refused packs-stale 'lock is stale' 'head .mindtape/config.toml .mindtape/packs.lock' \
  $'.mindtape/config.toml=[pack-sources.acme]\ngit = "https://github.com/acme/mindtape-packs"\nref = "v2"\n' \
  $'.mindtape/packs.lock=[acme]\ngit = "https://github.com/acme/mindtape-packs"\nref = "v1"\ncommit = "3f2c1a9e0b7d4c55e1a2b3c4d5e6f708192a3b4c"\nsha256 = "9c41e0d2b7a35f8c6e1d4a9b0c2f7e3d5a8b1c4e7f0a3d6b9c2e5f8a1b4d7c0e"\n'
refused packs-disable 'no rule named' 'cat .mindtape/config.toml' \
  $'.mindtape/config.toml=extends = ["mindtape:tasks"]\ndisable = ["imperative-titles"]\n'
refused packs-ambiguous 'ambiguous pack' 'find team team.toml' \
  $'.mindtape/config.toml=extends = ["./team"]\n' \
  $'team/pack.toml=' \
  $'team.toml='
refused packs-params-unknown 'has no parameter' 'head .mindtape/config.toml team.toml' \
  $'.mindtape/config.toml=extends = [{ pack = "./team.toml", with = { max-file = 8 } }]\n' \
  $'team.toml=[params]\nmax-files = 5\n'
