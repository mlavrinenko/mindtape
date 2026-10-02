#!/usr/bin/env bash
# www/generate-config-runs.sh — capture the writes /statuses/ and /rules/
# show, each in a copy of its checked fixture: examples/statuses/ for the
# transition, note and unblocked guards, examples/rules/ for a rule breach, a
# required field and a warning.
#
# Called by www/generate.sh with the repo root and the generated/ directory,
# inside the lock that script already holds, so this one takes none of its
# own. generate.sh exports MINDTAPE_NOW, so every run sees the same date.
#
# Each capture is a record (www/capture.sh), so the page prints the command
# from the run. Most of these commands are refused, which is the point of the
# capture, so each records the exit code it must return. A refusal that
# starts passing fails the build.
set -eo pipefail

REPO="$1"
OUT="$2"
if [ -z "$REPO" ] || [ -z "$OUT" ]; then
  echo "usage: generate-config-runs.sh <repo-root> <generated-dir>" >&2
  exit 1
fi
MT="$REPO/target/debug/mt"

# shellcheck source=www/capture.sh
source "$REPO/www/capture.sh"
capture_init

capture_copy S examples/statuses
capture --exit 1 statuses-transition "$S" 'mt flip done import-xlsx-files'
capture --exit 1 statuses-note "$S" 'mt flip failed add-a-dry-run-flag'
capture --exit 1 statuses-unblocked "$S" 'mt flip wip import-xlsx-files'

capture_copy R examples/rules
capture --exit 1 rules-wip-cap "$R" 'mt flip wip add-csv-export'
capture --exit 1 rules-assert "$R" 'mt set fix-the-session-timeout priority=1'
capture --exit 1 rules-required "$R" 'mt set add-csv-export kind=bug'
capture rules-warn "$R" "mt set add-csv-export 'title=Add CSV export to the report page.'"

# The legend a project gets when it declares none: the embedded floor's, read
# back through the same merge every project goes through.
capture_scratch F
mkdir -p "$F/.mindtape"
: >"$F/.mindtape/config.toml"
"$MT" -w "$F" config show --json | jq '.tasks.status' >"$OUT/floor-legend.json"

# Copied so the page can read() it: the site builds under --root www, and
# Typst refuses a read outside the root.
cp "$REPO/examples/rules/lib.typ" "$OUT/rules-lib.typ"
