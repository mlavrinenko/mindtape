#!/usr/bin/env bash
# www/generate-quickstart.sh — capture the session www/content/quickstart.typ
# walks through. Called by www/generate.sh with the repo root and the
# generated/ directory, inside the lock that script holds.
#
# The session starts in an empty directory with an empty config and data home,
# so it prints what a newcomer's first commands print: install, a project from
# the default preset, three tasks, a listing and a filter, a status change the
# legend refuses and the two it allows, and a check.
set -eo pipefail

REPO="$1"
OUT="$2"
MT="$REPO/target/debug/mt"

# shellcheck source=www/capture.sh
source "$REPO/www/capture.sh"
capture_init
capture_home
capture_scratch QS

capture qs-install "$QS" 'mt install'
capture qs-init "$QS" 'mt init --no-input'
capture_lines qs-add "$QS" <<'EOF'
mt add "Fix the login bug"
mt add "Write the onboarding guide"
mt add "Set up CI"
EOF
cp "$QS/tasks/fix-the-login-bug.typ" "$OUT/qs-written.typ"
capture qs-ls "$QS" 'mt ls'
capture qs-filter "$QS" "mt ls 'contains(\"login\")'"
capture --exit 1 --expect 'illegal transition' qs-refused "$QS" 'mt done fix-the-login-bug'
capture_lines qs-flip "$QS" <<'EOF'
mt wip fix-the-login-bug
mt done fix-the-login-bug
EOF
capture qs-check "$QS" 'mt check'
