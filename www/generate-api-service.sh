#!/usr/bin/env bash
# www/generate-api-service.sh — capture, for each problem examples/api-service
# solves, a short session in which mt refuses the work that problem allows.
# Called by www/generate.sh with the repo root and the generated/ directory,
# inside the lock that script holds.
#
# Each `# docs:slice api-service/<id>` block in the project's config solves one
# problem, and the session for it is the record <id>. The script fails when a
# slice has no record, so a problem added to the config cannot ship without
# its demo.
#
# Each session runs in its own copy of the project, so no session sees what
# another wrote. Each copy is a git repository with the project at its root,
# because touches() reads the history of the repository the project roots.
set -eo pipefail

REPO="$1"
OUT="$2"
MT="$REPO/target/debug/mt"
API="$REPO/examples/api-service"

# git reads no config of the builder's, so a hooks path, a signing key or a
# default branch on this machine changes no capture. The fixed identity and
# dates give each commit the same hash on every build.
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=Ada GIT_AUTHOR_EMAIL=ada@example.com
export GIT_COMMITTER_NAME=Ada GIT_COMMITTER_EMAIL=ada@example.com

# shellcheck source=www/capture.sh
source "$REPO/www/capture.sh"
# The api-service tasks are dated in the last week of September 2026, so
# "today" is just after them, and a flip writes the same date on every build.
# www/capture.sh declares that clock for every generator that runs the fixture.
export MINDTAPE_NOW=$API_SERVICE_NOW
export GIT_AUTHOR_DATE="${API_SERVICE_NOW}T12:00:00Z" GIT_COMMITTER_DATE="${API_SERVICE_NOW}T12:00:00Z"
capture_init

# api_copy <var>: a copy of the project, committed as the first commit of a
# new repository, stored in <var>.
api_copy() {
    capture_copy "$1" examples/api-service
    capture_prep "${!1}" 'git init -q -b main && git add -A && git commit -qm "Start the backlog"'
}

# Split oversized tasks: a task estimated at 7 source files is refused at
# `mt add`, and nothing is written.
api_copy P
capture --exit 1 --expect 'rules\.split-big-work' split-oversized-tasks "$P" "$(
    cat <<'CMD'
mt add -f - <<'EOF'
title: Sign in with Google and GitHub
src-files: 7
problem: |
  + Open `/login`.
  + 🔴 It offers only a password form. A user who signed up with Google has to make a password first.
to-be: |
  - 🟢 `/login` offers Google and GitHub buttons, and the callback logs the user in.
changes: |
  - `Cargo.toml`: add the `oauth2` crate.
  - `src/config.rs`: read each provider's client ID and secret.
  - `src/auth/oauth.rs`, new: the authorization-code flow.
  - `src/auth/providers.rs`, new: the Google and GitHub endpoints and scopes.
  - `src/auth/login.rs`: log in the user the callback names.
  - `src/routes.rs`: mount `/auth/{provider}/callback`.
  - `migrations/0007_identities.sql`, new: link each provider account to a user.
out-of-scope: |
  - Sign in with Apple.
EOF
CMD
)"

# Require a task shape: the template refuses a task with sections left out,
# and the rule pack refuses a Problem written as prose.
api_copy P
capture --exit 1 --expect 'missing required slot' require-a-task-shape "$P" "$(
    cat <<'CMD'
mt add -f - <<'EOF'
title: Paginate the audit log endpoint
src-files: 2
problem: GET /audit returns every row, and on production the response is 40 MB.
EOF
CMD
)"
capture --exit 1 --expect 'problem-is-numbered-steps' require-a-task-shape "$P" "$(
    cat <<'CMD'
mt add -f - <<'EOF'
title: Paginate the audit log endpoint
src-files: 2
problem: GET /audit returns every row, and on production the response is 40 MB.
to-be: |
  - 🟢 `GET /audit` returns 100 rows and a `next` cursor for the page after them.
changes: |
  - `src/audit/routes.rs`: read `cursor` and `limit` from the query string.
  - `src/audit/store.rs`: select one page, ordered by `id`.
out-of-scope: |
  - Filtering the audit log by user.
EOF
CMD
)"

# Gate done on your checks: `mt done` runs scripts/test and refuses while it
# fails. `--force` closes the task anyway and writes the waiver into the
# status note.
api_copy P
capture --exit 1 --expect 'gate failed' gate-done-on-checks "$P" 'mt done reject-expired-sessions-on-token-refresh'
capture_lines gate-done-on-checks "$P" <<'EOF'
mt done reject-expired-sessions-on-token-refresh --force
grep status: tasks/reject-expired-sessions-on-token-refresh.typ
EOF

# One task at a time: a second task cannot start while one is in progress,
# and a task cannot start before its dependencies are done.
api_copy P
capture --exit 1 --expect 'rules\.one-wip' one-task-at-a-time "$P" 'mt wip add-redis-client'
capture --exit 1 --expect 'requires an unblocked task' one-task-at-a-time "$P" 'mt wip throttle-failed-logins-per-ip'
capture one-task-at-a-time "$P" 'mt ls blocked --columns title,why'

# Trace commits to tasks: a commit-msg hook runs `mt check --commit-msg`, which
# refuses a commit without a `Refs:` trailer. touches() then finds the task
# behind a file. The project's first commit holds the file, and the reader's
# edit to it is made before the session.
api_copy P
mkdir -p "$P/src/auth"
cat >"$P/src/auth/session.rs" <<'EOF'
pub fn refresh(session: &Session) -> Result<Token, AuthError> {
    Ok(Token::issue(session.user_id))
}
EOF
capture_prep "$P" 'git add src && git commit -qm "Add token refresh"'
cat >"$P/src/auth/session.rs" <<'EOF'
pub fn refresh(session: &Session) -> Result<Token, AuthError> {
    if session.expires_at <= Utc::now() {
        return Err(AuthError::Expired);
    }
    Ok(Token::issue(session.user_id))
}
EOF
capture trace-commits-to-tasks "$P" "$(
    cat <<'CMD'
cat >.git/hooks/commit-msg <<'EOF'
#!/bin/sh
exec mt check --commit-msg "$1"
EOF
CMD
)"
capture trace-commits-to-tasks "$P" 'chmod +x .git/hooks/commit-msg'
capture --exit 1 --expect 'work-names-a-task' trace-commits-to-tasks "$P" \
    'git commit -am "fix(auth): reject expired sessions on token refresh"'
capture trace-commits-to-tasks "$P" \
    'git commit -am "fix(auth): reject expired sessions on token refresh" -m "Refs: reject-expired-sessions-on-token-refresh"'
capture trace-commits-to-tasks "$P" "mt ls 'touches(\"src/auth/session.rs\")'"

problems=$(sed -n 's|^# docs:slice api-service/||p' "$API/.mindtape/config.toml")
if [ -z "$problems" ]; then
    echo "generate-api-service.sh: $API/.mindtape/config.toml has no api-service slice" >&2
    exit 1
fi
for id in $problems; do
    if [ ! -f "$OUT/$id.json" ]; then
        echo "generate-api-service.sh: the slice api-service/$id has no session; capture one as $id" >&2
        exit 1
    fi
done
