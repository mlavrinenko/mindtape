#!/usr/bin/env bash
# www/generate-import-export.sh — run the /import-export/ recipes and record
# them. www/generate.sh calls it inside the lock it holds, so this script takes
# no lock of its own.
#
# The imports run the contrib/jq adapters over their test inputs, in a project
# `mt init --no-input` made. The round trip exports examples/round-trip/ with
# `mt ls '' --json`, imports the file into a new project and lists it there.
# The export recipes run against the same fixture.
#
# `mt import` exits 1 when it skips a record, so a skip fails the build, and the
# round trip fails it when the new project's tasks differ from the ones
# exported.
set -eo pipefail

REPO="$1"
OUT="$2"
MT="$REPO/target/debug/mt"
JQ="$REPO/contrib/jq"

# shellcheck source=www/capture.sh
source "$REPO/www/capture.sh"
# The fixture's dates are in late September 2026, so "today" is just after.
export MINDTAPE_NOW=$API_SERVICE_NOW
capture_init

# capture_one <slug> <dir>: capture all of stdin as one command. A quoted
# heredoc keeps a multi-line command exactly as the reader sees it.
capture_one() { capture "$1" "$2" "$(cat)"; }

# --- imports from other trackers ----------------------------------------------
capture_scratch P
capture_prep "$P" 'mt init --no-input'
mkdir -p "$P/contrib/jq"
cp "$JQ"/*.jq "$P/contrib/jq/"
cp "$JQ/fixtures/taskwarrior.in.json" "$P/taskwarrior.json"
cp "$JQ/fixtures/github.in.json" "$P/issues.json"
jq -r '.[]' "$JQ/fixtures/todo.txt.in.json" >"$P/todo.txt"

capture import-taskwarrior "$P" 'jq -f contrib/jq/taskwarrior.jq taskwarrior.json | mt import'
capture import-github "$P" 'jq -f contrib/jq/github.jq issues.json | mt import --to github'
capture_one import-todotxt "$P" <<'EOF'
jq -R -s 'split("\n") | map(select(length > 0))' todo.txt \
  | jq -f contrib/jq/todo.txt.jq | mt import
EOF

# --- the round trip -----------------------------------------------------------
capture_copy SRC examples/round-trip
NEW="$(dirname "$SRC")/new-project"
mkdir "$NEW"
capture_lines round-trip-export "$SRC" <<'EOF'
mt ls ''
mt ls '' --json > tasks.json
EOF
capture_lines round-trip-import "$NEW" <<'EOF'
mt init --no-input
mt import ../round-trip/tasks.json
mt ls ''
EOF
capture_prep "$NEW" "diff <(jq -S '.tasks | sort_by(.id)' ../round-trip/tasks.json) <(mt ls '' --json | jq -S '.tasks | sort_by(.id)')"

# --- exports to other formats -------------------------------------------------
capture_one export-todotxt "$SRC" <<'EOF'
mt ls '' --json | jq -r '.tasks[] | select(.status.name != "cancelled")
  | (.priority | if type == "object" then .score else . end) as $p
  | [ (if .status.name == "done" then "x" else empty end),
      (if ($p | type) == "number" and $p >= 7 then "(A)" else empty end),
      .title,
      (.tags[] | "+" + .),
      (.due // empty | "due:" + .) ]
  | join(" ")'
EOF
capture export-csv "$SRC" "mt ls '' --format csv"
capture_one export-csv-jq "$SRC" <<'EOF'
mt ls '' --json | jq -r '["title", "status", "priority", "due"],
  (.tasks[] | [.title, .status.name, (.priority | if type == "object" then .score else . end), .due])
  | @csv'
EOF
capture_one export-taskwarrior "$SRC" <<'EOF'
mt ls '' --json | jq '[.tasks[] | {
    description: .title,
    status: ({done: "completed", cancelled: "deleted"}[.status.name] // "pending"),
    tags: .tags,
    due: (if .due then (.due | gsub("-"; "")) + "T000000Z" else null end)
  } | with_entries(select(.value != null))]'
EOF

# --- /schema/ -----------------------------------------------------------------
# One full record, the two shapes of `priority` from examples/dsl/, which
# configures the ICE framework, and the envelope with its records counted.
capture schema-record "$SRC" "mt ls 'id == \"expire-idle-sessions\"' --json | jq '.tasks[0]'"
capture schema-priority "$REPO/examples/dsl" "mt ls '' --json | jq -c '.tasks[] | {id, priority}'"
capture schema-envelope "$SRC" "mt ls -n 1 --json | jq '.tasks |= length'"
