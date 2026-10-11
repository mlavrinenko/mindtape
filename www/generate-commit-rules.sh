#!/usr/bin/env bash
# www/generate-commit-rules.sh — record the commit checks /commit-rules/ shows,
# in a git copy of examples/rules/, whose config holds three commit rules.
# www/generate.sh calls it inside the lock it holds, so this script takes no
# lock of its own.
#
# git runs the `commit-msg` hook, and the hook runs `mt check --commit-msg`, so
# each refusal on the page is git's own. git reads no global or system config
# here, because a builder's `core.hooksPath` would skip the hook and their
# `commit.gpgsign` would prompt. The author, committer and dates are fixed, so
# the commit hashes the range check prints are the same on every build.
set -eo pipefail

REPO="$1"
OUT="$2"
MT="$REPO/target/debug/mt"

export MINDTAPE_NOW=2026-07-06
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=Ada GIT_AUTHOR_EMAIL=ada@example.com
export GIT_COMMITTER_NAME=Ada GIT_COMMITTER_EMAIL=ada@example.com
export GIT_AUTHOR_DATE=2026-07-06T10:00:00Z GIT_COMMITTER_DATE=2026-07-06T10:00:00Z

# shellcheck source=www/capture.sh
source "$REPO/www/capture.sh"
capture_init

capture_copy P examples/rules
capture_prep "$P" 'git init -q -b main && mkdir migrations src && touch src/session.rs && git add -A && git commit -qm "chore: start the project"'
capture_prep "$P" 'printf "#!/bin/sh\nmt check --commit-msg \"\$1\"\n" >.git/hooks/commit-msg && chmod +x .git/hooks/commit-msg'

capture commit-hook "$P" 'cat .git/hooks/commit-msg'

# The trailer check: a path in place of a stem.
capture_prep "$P" 'echo "CREATE TABLE sessions (id text);" >migrations/001-sessions.sql && git add -A'
capture --exit 1 --expect 'is a path; write the stem' commit-refs-path "$P" 'git commit -qm "feat: add the sessions table" -m "Refs: tasks/fix-the-session-timeout.typ"'

# A commit-scope rule: a migration with no `Refs:` trailer.
capture --exit 1 --expect 'migration-names-a-task' commit-scope-breach "$P" 'git commit -qm "feat: add the sessions table"'
capture commit-scope-breach "$P" 'git commit -qm "feat: add the sessions table" -m "Refs: fix-the-session-timeout"'

# A task-scope rule: the commit names a task whose status change is unstaged.
capture_prep "$P" 'mt done fix-the-session-timeout >/dev/null && echo "// expire after 30 minutes" >>src/session.rs && git add src/session.rs'
capture commit-task-scope "$P" 'git status --short'
capture --exit 1 --expect 'stage-the-flip' commit-task-scope "$P" 'git commit -qm "fix: expire idle sessions" -m "Refs: fix-the-session-timeout"'
capture commit-task-scope "$P" 'git add -A && git commit -qm "fix: expire idle sessions" -m "Refs: fix-the-session-timeout"'

# The range check: a commit that skipped the hook.
capture_prep "$P" 'echo "DROP TABLE legacy;" >migrations/002-drop-legacy.sql && git add -A && git commit -q --no-verify -m "chore: drop the legacy table"'
capture --exit 1 --expect 'broke commit rules' commit-range "$P" 'mt check --commits main~3..main'

# A tip rule: a branch starts a task and lands without finishing it.
capture_prep "$P" 'mt wip add-csv-export >/dev/null && echo "pub fn export() {}" >src/export.rs && git add -A && git commit -qm "feat: start the CSV export" -m "Refs: add-csv-export"'
capture --exit 1 --expect 'finish-named-tasks' commit-tip "$P" 'mt check --commits main~1..main'
