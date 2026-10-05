#!/usr/bin/env bash
# www/generate-guides.sh — capture the mt runs that /templates/, /agents/ and
# /derived-tags/ show, against examples/api-service/ and
# examples/derived-tags/. www/generate.sh calls it inside the lock it holds,
# so this script takes no lock of its own. It is a separate file to keep
# generate.sh under the 300-line Shell cap.
#
# The terminal transcripts are records (www/capture.sh), so each page prints
# the command from the run. Every command must succeed, except the one
# refusal /agents/ shows, which must exit 1. The JSON and Typst outputs the
# pages quote as code are plain files that `run` writes.
set -eo pipefail

REPO="$1"
OUT="$2"
MT="$REPO/target/debug/mt"
API="$REPO/examples/api-service"
TAGS="$REPO/examples/derived-tags"

# The api-service tasks are dated in the last week of September 2026, so
# "today" is just after them.
export MINDTAPE_NOW=2026-10-01

# shellcheck source=www/capture.sh
source "$REPO/www/capture.sh"
capture_init

# run <file> <project> <mt args...>: write one run's stdout to OUT/<file>,
# from the project's own directory, so every path mt prints is relative.
run() {
  local file="$1" dir="$2"
  shift 2
  (cd "$dir" && "$MT" --color never "$@") >"$OUT/$file"
}

# /templates/: the template, the slots mt finds in it, and the task that one
# values file fills. `--print` checks the task and prints it without writing
# it, so the run leaves the fixture untouched.
cp "$API/.mindtape/templates/task.typ" "$OUT/api-service-template.typ"
capture api-service-describe "$API" 'mt add --describe'
cat >"$OUT/api-service-values.yaml" <<'YAML'
title: Log slow database queries
src-files: 2
problem: |
  + Run `curl localhost:8080/reports/monthly`.
  + 🔴 It takes 9 seconds, and the log does not say which query was slow.
to-be: |
  - 🟢 A query slower than 500 ms logs its SQL and duration at `warn`.
changes: |
  - `src/db.rs`: time each query and log the slow ones.
  - `src/config.rs`: read the threshold from `SLOW_QUERY_MS`.
out-of-scope: |
  - Tracing every query.
YAML
run api-service-add-print.typ "$API" add -f "$OUT/api-service-values.yaml" --print

# /agents/: a listing cut to one task, and an add that names no slot values.
# Stdin is not a terminal, so mt opens no editor and refuses instead.
run api-service-ls-ready.json "$API" ls ready -n 1 --json
capture --exit 1 --expect 'missing required slot' api-service-add-refused "$API" 'mt add "Log slow database queries" --print </dev/null'

# /derived-tags/: every task with the tags mt derives for it, and a filter on
# one derived tag.
run derived-tags-ls.json "$TAGS" ls '' --json
capture derived-tags-ls-auth "$TAGS" "mt ls 'has_tag(\"auth\")'"
