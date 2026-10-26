#!/usr/bin/env bash
# www/generate-packs.sh — capture every built-in pack, this repo's
# own `mt config show --origin`, a pack's priority merged below the project's,
# a directory pack, a pack's parameter set from `with`, and ten configs
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

# Ten projects whose config `mt` refuses. Each record shows the
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
refused packs-not-installed 'not installed' 'head .mindtape/config.toml .mindtape/packs.lock' \
  $'.mindtape/config.toml=extends = ["acme:commits"]\n\n[pack-sources.acme]\ngit = "https://github.com/acme/mindtape-packs"\nref = "v1"\n' \
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

# A pack source the build can fetch offline: a bare repository beside the
# project, whose `v1` and `v2` tags hold two commits of one pack, and `v2`
# a second pack. `mt packs update` locks `v1`, `mt packs install` fetches it
# into an empty cache, and the bump to `v2` prints what the new commit
# changes. git reads no config of the builder's, and the fixed identity and
# dates give each commit the same hash on every build. capture.sh points
# MINDTAPE_CACHE_HOME at the capture area, which prints as `~/.cache`.
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=Ada GIT_AUTHOR_EMAIL=ada@example.com
export GIT_COMMITTER_NAME=Ada GIT_COMMITTER_EMAIL=ada@example.com
export GIT_AUTHOR_DATE=2026-10-01T12:00:00Z GIT_COMMITTER_DATE=2026-10-01T12:00:00Z
CAPTURE_PATHS+=("$CAPTURE_TMP/cache=~/.cache")
source=
capture_scratch source
work="$(dirname "$source")/work"
mkdir -p "$work" "$source/.mindtape"
cat >"$work/commits.toml" <<'TOML'
[[commits.rules]]
name = "short-subject"
assert = 'matches(subject, "^.{1,72}$")'
message = "keep the subject under 73 characters"
TOML
git -C "$work" init -q -b main
git -C "$work" add -A
git -C "$work" commit -qm "Ship the commits pack"
git -C "$work" tag -a -m v1 v1
sed -i 's/{1,72}/{1,50}/; s/under 73/under 51/' "$work/commits.toml"
cat >>"$work/commits.toml" <<'TOML'

[[on.flip]]
name = "close-gate"
when = 'into("done")'
actions = [{ run = "just check", gate = true, message = "the checks must pass" }]
TOML
printf '[[tasks.rules]]\nname = "has-owner"\nassert = "has(assignee)"\n' >"$work/ops.toml"
git -C "$work" add -A
git -C "$work" commit -qm "Gate a close on the checks, add the ops pack"
git -C "$work" tag -a -m v2 v2
git clone -q --bare "$work" "$(dirname "$source")/acme-packs.git"
printf 'extends = ["acme:commits"]\n\n[pack-sources.acme]\ngit = "../acme-packs.git"\nref = "v1"\n' \
  >"$source/.mindtape/config.toml"
capture packs-lock "$source" 'cat .mindtape/config.toml'
capture packs-lock "$source" 'mt packs update'
rm -rf "$CAPTURE_TMP/cache"
capture packs-install "$source" 'mt packs install'
capture packs-install "$source" 'mt packs install'
sed -i 's/ref = "v1"/ref = "v2"/' "$source/.mindtape/config.toml"
capture packs-update "$source" 'mt packs update'
