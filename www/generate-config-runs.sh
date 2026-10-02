#!/usr/bin/env bash
# www/generate-config-runs.sh — capture the writes /statuses/ and /rules/
# show, each run against a throwaway copy of its checked fixture:
# examples/statuses/ for the transition, note and unblocked guards,
# examples/rules/ for a rule breach, a required field and a warning.
#
# Called by www/generate.sh with the repo root and the generated/ directory,
# inside the lock that script already holds, so this one takes none of its
# own. generate.sh exports MINDTAPE_NOW and an empty MINDTAPE_CONFIG_HOME, so
# every run here sees the same date and no user config.
#
# Each capture starts with the command line, printed from the same argument
# list that runs, so the page cannot show one command and the output of
# another. Most of these commands are refused and exit non-zero, which is the
# point of the capture, so the exit code is ignored.
set -eo pipefail

REPO="$1"
OUT="$2"
if [ -z "$REPO" ] || [ -z "$OUT" ]; then
  echo "usage: generate-config-runs.sh <repo-root> <generated-dir>" >&2
  exit 1
fi
MT="$REPO/target/debug/mt"

# Each fixture resolves @local/mindtape at ../../lib, so a copy keeps that
# depth and links the real lib/ at the top, as generate.sh does for its copy.
COPY_ROOT="$(mktemp -d)"
trap 'rm -rf "$COPY_ROOT"' EXIT
mkdir -p "$COPY_ROOT/examples"
ln -s "$REPO/lib" "$COPY_ROOT/lib"

# One argument as a shell reader would type it: bare when it holds only
# characters no shell treats specially, else single-quoted.
shell_word() {
  case "$1" in
    *[!A-Za-z0-9_./=:@%+-]* | "") printf "'%s'" "${1//\'/\'\\\'\'}" ;;
    *) printf '%s' "$1" ;;
  esac
}

# run <slug> <fixture> <mt args...> -> OUT/<slug>.txt: the command, then its
# stdout and stderr, with the copy's absolute path cut to a relative one.
run() {
  local slug="$1" fixture="$2" copy="$COPY_ROOT/examples/$2" arg
  shift 2
  [ -d "$copy" ] || cp -r "$REPO/examples/$fixture" "$copy"
  {
    printf '$ mt'
    for arg in "$@"; do printf ' %s' "$(shell_word "$arg")"; done
    printf '\n'
    "$MT" -w "$copy" --color never "$@" 2>&1 || true
  } | sed "s#$copy/##g" >"$OUT/$slug.txt"
}

run statuses-transition statuses flip done import-xlsx-files
run statuses-note statuses flip failed add-a-dry-run-flag
run statuses-unblocked statuses flip wip import-xlsx-files

run rules-wip-cap rules flip wip add-csv-export
run rules-assert rules set fix-the-session-timeout priority=1
run rules-required rules set add-csv-export kind=bug
run rules-warn rules set add-csv-export "title=Add CSV export to the report page."

# The legend a project gets when it declares none: the embedded floor's, read
# back through the same merge every project goes through.
mkdir -p "$COPY_ROOT/floor/.mindtape"
: >"$COPY_ROOT/floor/.mindtape/config.toml"
"$MT" -w "$COPY_ROOT/floor" config show --json | jq '.tasks.status' >"$OUT/floor-legend.json"

# Copied so the page can read() it: the site builds under --root www, and
# Typst refuses a read outside the root.
cp "$REPO/examples/rules/lib.typ" "$OUT/rules-lib.typ"
