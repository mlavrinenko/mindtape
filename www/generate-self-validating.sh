#!/usr/bin/env bash
# www/generate-self-validating.sh — capture `mt check` on
# examples/self-validating/ for /self-validating-tasks/. The page quotes the
# fixture's code as slices (www/generate-examples.sh) and this one run as a
# record. www/generate.sh calls it inside the lock it holds, so this script
# takes no lock of its own. It is a separate file to keep generate.sh under
# the 300-line Shell cap.
#
# One task in the fixture skips a goal on purpose, so `mt check` exits 1.
# The capture is a record (www/capture.sh) that expects that code, so the
# build fails if the check ever passes and the page would show no refusal.
set -eo pipefail

REPO="$1"
OUT="$2"
MT="$REPO/target/debug/mt"

# shellcheck source=www/capture.sh
source "$REPO/www/capture.sh"
capture_init
capture --exit 1 --expect 'rules\.goals' self-validating-check "$REPO/examples/self-validating" 'mt check'
