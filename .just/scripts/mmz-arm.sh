#!/usr/bin/env bash
# .just/scripts/mmz-arm.sh — the body of the `mmz` recipe, which every
# `just check` arm runs (truth/tasks/print-nothing-from-a-passing-just-check-
# but-its-verdict.typ). Runs from the repo root, like the recipe that invokes
# it.
#
# Outside `just check` it is `mmz just <recipe>`, streaming as before. Inside,
# check.sh sets MINDTAPE_CHECK_DIR, and the arm writes its whole output to
# `<dir>/<recipe>.log` and its outcome to `<dir>/<recipe>.status`, so check.sh
# can print a failed gate's log and drop a passing one's. The capture sits here,
# once, rather than as a quiet flag on each tool: most of the tools have none.
#
# Whether the run was a cache hit comes from the run itself: a hit prints
# nothing (`on_hit: ""`), so mmz writes `skipped` or `ran` to the file
# MMZ_REPORT names instead. No word means mmz stopped before it decided, which
# counts as a run, since the arm did not pass on a cached record.
set -eo pipefail

if [ -z "${MINDTAPE_CHECK_DIR:-}" ]; then
    exec mmz just "$@"
fi

name="just $*"
slug=$(printf '%s' "$*" | tr -c 'A-Za-z0-9_.-' '_')
log="$MINDTAPE_CHECK_DIR/$slug.log"
report="$MINDTAPE_CHECK_DIR/$slug.report"
rm -f "$report"

rc=0
MMZ_REPORT="$report" mmz just "$@" >"$log" 2>&1 || rc=$?

how=run
if [ "$(cat "$report" 2>/dev/null)" = skipped ]; then how=cached; fi

outcome=pass
if [ "$rc" -ne 0 ]; then outcome=fail; fi
printf '%s\t%s\t%s\t%s\n' "$outcome" "$how" "$name" "${log#"$PWD"/}" \
    >"$MINDTAPE_CHECK_DIR/$slug.status"
exit "$rc"
