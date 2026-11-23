#!/usr/bin/env bash
# www/generate-goals.sh: capture the mt session /tie-work-to-goals/ shows,
# against examples/goals/. The template rates every goal neutral unless the
# answers say otherwise. `mt add` first refuses a task that helps none of them,
# and files it once one goal says how it helps. Called by www/generate.sh with
# the repo root and the generated/ directory, inside the lock that script
# holds, so this one takes none of its own.
#
# The session runs in a copy of the project, committed as the first commit of
# a new repository. Its writes never reach the fixture.
set -eo pipefail

REPO="$1"
OUT="$2"
MT="$REPO/target/debug/mt"

# The docs clock, and a git identity no builder's config can change: a hooks
# path, a signing key or a default branch cannot alter the capture.
export MINDTAPE_NOW="${MINDTAPE_NOW:-2026-07-06}"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=Ada GIT_AUTHOR_EMAIL=ada@example.com
export GIT_COMMITTER_NAME=Ada GIT_COMMITTER_EMAIL=ada@example.com
export GIT_AUTHOR_DATE="${MINDTAPE_NOW}T12:00:00Z" GIT_COMMITTER_DATE="${MINDTAPE_NOW}T12:00:00Z"

# shellcheck source=www/capture.sh
source "$REPO/www/capture.sh"
capture_init

# A copy of the project as the first commit of a new repository.
capture_copy P examples/goals
capture_prep "$P" 'git init -q -b main && git add -A && git commit -qm "Start the backlog"'

capture_prep "$P" "$(
    cat <<'CMD'
cat >task.yaml <<'EOF'
title: Search the archive
summary: |
  The archive search box returns nothing after the first page.
EOF
CMD
)"
capture tie-work-to-goals "$P" 'cat task.yaml'
capture --exit 1 --expect 'rules\.goals' tie-work-to-goals "$P" 'mt add -f task.yaml'

capture_prep "$P" "$(
    cat <<'CMD'
cat >>task.yaml <<'EOF'
goals: |
  + #Helps(to: answer-fast)[matches appear as the query changes].
  + #Neutral(avoid-lock-in).
EOF
CMD
)"
capture tie-work-to-goals "$P" 'mt add -f task.yaml'
