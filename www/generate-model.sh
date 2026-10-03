#!/usr/bin/env bash
# www/generate-model.sh — capture the runs /concepts/, /links/ and
# /recurring/ show, against copies of examples/links/ and examples/recurring/.
# www/generate.sh calls it inside the lock it holds, so this script takes no
# lock of its own.
#
# Each capture is a record (www/capture.sh), so a page prints the command from
# the run. A step that exits other than its expected code fails the build.
set -eo pipefail

REPO="$1"
OUT="$2"
MT="$REPO/target/debug/mt"

# The fixtures' tasks are dated August and September 2026, so "today" falls
# just after the latest of them.
export MINDTAPE_NOW=2026-09-15

# shellcheck source=www/capture.sh
source "$REPO/www/capture.sh"
capture_init

# /concepts/: one whole task file, copied so the page can read() it under the
# site's --root www. `just check-examples` validates its project.
cp "$REPO/examples/api-service/tasks/add-redis-client.typ" "$OUT/concepts-task.typ"

# /concepts/: a rename by hand leaves an inbound link dangling, and `mt mv`
# renames the file and rewrites the link.
capture_copy C examples/links
capture concepts-mv-by-hand "$C" 'mv tasks/pick-a-provider.typ tasks/choose-a-provider.typ'
capture --exit 1 concepts-mv-by-hand "$C" 'mt check'
capture_prep "$C" 'mv tasks/choose-a-provider.typ tasks/pick-a-provider.typ'
capture concepts-mv "$C" 'mt mv pick-a-provider choose-a-provider'
capture concepts-mv "$C" 'mt check'

# /links/: a parent's progress, and `mt link` writing one edge twice.
capture links-progress "$REPO/examples/links" "mt ls 'children > 0' --columns title,status,progress"
capture_copy L examples/links
capture_lines links-write "$L" <<'EOF'
mt link write-docs depends-on pick-a-provider
mt link write-docs depends-on pick-a-provider.typ
EOF

# /recurring/: `mt recur` writes the next occurrence of a daily task. The
# default build lacks the rrule feature, so `mt check` on this fixture fails
# on its rrule(...) task, but `mt recur` on another task still writes.
capture_copy R examples/recurring
capture recurring-spawn "$R" 'mt recur nightly-dependency-audit'
