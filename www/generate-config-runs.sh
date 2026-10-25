#!/usr/bin/env bash
# www/generate-config-runs.sh — capture the writes /statuses/ and /rules/
# show, each in a copy of its checked fixture: examples/statuses/ for the
# transition, note and unblocked guards, examples/rules/ for a rule breach, a
# required field and a warning. It also draws the moves of the
# examples/statuses/ legend, from its `mt config show --json`.
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
capture --exit 1 --expect 'illegal transition' statuses-transition "$S" 'mt flip done import-xlsx-files'
capture --exit 1 --expect 'requires a note' statuses-note "$S" 'mt flip failed add-a-dry-run-flag'
capture --exit 1 --expect 'requires an unblocked task' statuses-unblocked "$S" 'mt flip wip import-xlsx-files'

# The moves the fixture's legend allows, drawn as a graph.
"$MT" -w "$REPO/examples/statuses" config show --json \
  | jq -r -f "$REPO/www/status-moves.jq" | render_dot statuses-moves

capture_copy R examples/rules
capture --exit 1 --expect 'rules\.one-task-in-flight' rules-wip-cap "$R" 'mt flip wip add-csv-export'
capture --exit 1 --expect 'rules\.bugs-outrank-the-low-band' rules-assert "$R" 'mt set fix-the-session-timeout priority=1'
capture --exit 1 --expect 'required field is missing' rules-required "$R" 'mt set add-csv-export kind=bug'
# A paragraph with two em dashes breaks one-dash-per-paragraph, whose message
# takes the line while the check's own words follow on a note. The file goes
# back after, so the captures below see the fixture as it is.
cp "$R/tasks/add-csv-export.typ" "$R/add-csv-export.typ.orig"
printf '\nThe export runs on the server — not in the browser — so it scales.\n' >>"$R/tasks/add-csv-export.typ"
capture --exit 1 --expect 'note: section .Notes. paragraph 2' rules-content-detail "$R" 'mt check'
mv "$R/add-csv-export.typ.orig" "$R/tasks/add-csv-export.typ"
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
