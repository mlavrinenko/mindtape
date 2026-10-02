#!/usr/bin/env bash
# www/generate-self-validating.sh — capture `mt check` on
# examples/self-validating/ for /self-validating-tasks/. The page quotes the
# fixture's code as slices (www/generate-examples.sh) and this one run as
# text. www/generate.sh calls it inside the lock it holds, so this script
# takes no lock of its own. It is a separate file to keep generate.sh under
# the 300-line Shell cap.
#
# One task in the fixture skips a goal on purpose, so `mt check` exits
# non-zero. The script records that code beside the transcript instead of
# letting `set -e` stop the build, and fails if the check ever passes,
# because then the page would show no refusal.
set -eo pipefail

REPO="$1"
OUT="$2"
MT="$REPO/target/debug/mt"

code=0
(cd "$REPO/examples/self-validating" && "$MT" --color never check) \
  >"$OUT/self-validating-check.txt" 2>&1 || code=$?
[ "$code" -ne 0 ] || {
  echo "generate-self-validating.sh: mt check passed on examples/self-validating" >&2
  exit 1
}
printf '%s' "$code" >"$OUT/self-validating-check-exit.txt"
