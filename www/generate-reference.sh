#!/usr/bin/env bash
# www/generate-reference.sh — capture the mt runs that /filters/,
# /named-sets/, /priority/, /estimates/ and /read-many-projects/ show.
# www/generate.sh calls it with the repo root and the generated/ directory,
# inside the lock it holds, so this script takes no lock of its own.
#
# Read commands run in a fixture in place. Commands that write, or need git,
# run in a copy. Each capture is a record (www/capture.sh), so a page prints
# the command from the run. A refusal records the exit code it must return,
# so a refusal that starts passing fails the build.
set -eo pipefail

REPO="$1"
OUT="$2"
if [ -z "$REPO" ] || [ -z "$OUT" ]; then
  echo "usage: generate-reference.sh <repo-root> <generated-dir>" >&2
  exit 1
fi
MT="$REPO/target/debug/mt"
API="$REPO/examples/api-service"
SETS="$REPO/examples/named-sets"

# shellcheck source=www/capture.sh
source "$REPO/www/capture.sh"
# The fixtures' tasks are dated in the last week of September 2026, so
# "today" is just after them.
export MINDTAPE_NOW=$API_SERVICE_NOW
capture_init

# A commit in a capture copy, with a fixed author and no signing.
COMMIT='git -c user.name=docs -c user.email=docs@example.com -c commit.gpgsign=false commit -q'

# --- the home page hero and /filters/ -----------------------------------------
capture hero-ls "$API" 'mt ls'
capture filters-open "$API" "mt ls '!terminal'"
capture filters-operators "$API" "mt ls 'kind == \"fix\" || (kind == \"feat\" && estimate.src-files <= 2)'"
capture_lines filters-missing "$API" <<'EOF'
mt ls 'estimate.src-files > 2'
mt ls '!(estimate.src-files <= 2)'
EOF
capture_lines filters-functions "$API" <<'EOF'
mt ls 'has(estimate) && miss(assignee)'
mt ls 'contains("redis")'                 # title, id and body
mt ls 'contains("redis", title)'          # title only
mt ls 'links("depends-on")'
mt ls 'stale(7) && !terminal'
mt ls '@default && kind == "feat"'
EOF
capture_lines filters-strings "$API" <<'EOF'
mt ls 'in_dict("verbs", word(title, 1))'
mt ls '!matches(word(title, 1), "ing$")'
mt ls 'in_dict("mindtape-verbs", word(title, 1)) || matches(title, "^`")'
mt ls 'matches(body, "(?m)^- TODO:")'
EOF
capture --exit 2 --expect "unknown dict: 'verb'" filters-unknown-dict "$API" "mt ls 'in_dict(\"verb\", word(title, 1))'"
capture filters-blocked "$API" 'mt ls blocked --columns title,why'
capture_lines filters-graph "$API" <<'EOF'
mt ls ready
mt ls 'count(status == "wip") > 1 && status == "wip"'
mt ls 'blocks > 0'
mt ls 'topo == 0 && !terminal'
EOF
capture_lines filters-sort "$API" <<'EOF'
mt ls --sort status.date:desc
mt ls --sort topo --sort priority:desc
mt ls --sort blocks:desc -n 1
EOF

capture_copy P examples/api-service
capture_prep "$P" "git init -q && git add -A && $COMMIT -m fixture"
capture_prep "$P" "mkdir -p src/auth && echo 'fn refresh() {}' >src/auth/session.rs && git add src && $COMMIT -m 'fix(auth): check expires_at on refresh' -m 'Refs: reject-expired-sessions-on-token-refresh'"
capture_lines filters-touches "$P" <<'EOF'
git --no-pager log -1 --format=%B
mt ls 'touches("src/auth/session.rs")'
EOF

# --- /named-sets/ -------------------------------------------------------------
capture_lines named-sets-views "$SETS" <<'EOF'
mt views
mt views waiting
mt ls @waiting
EOF
capture_lines named-sets-dicts "$SETS" <<'EOF'
mt dicts
mt dicts areas
EOF
capture --exit 1 --expect 'is not an allowed tag' named-sets-allowed "$SETS" 'mt set fix-the-refund-rounding tags+=payments'

# --- /priority/ ---------------------------------------------------------------
capture priority-ls "$REPO/examples/priority" 'mt ls --columns title,priority --sort priority:desc'
capture --exit 1 --expect 'allow-manual = false' priority-strict "$REPO/examples/priority" 'mt set fix-the-signup-email-typo priority=8'

# --- /estimates/ --------------------------------------------------------------
# A rule reading changed() needs a git repository, so the copy gets one.
capture_copy P examples/estimates
capture_prep "$P" "git init -q && git add -A && $COMMIT -m fixture"
capture estimates-eval "$P" 'mt eval rename-the-harvest-cache'
capture --exit 1 --expect 'rules\.judgment-stays-small' estimates-cap "$P" 'mt set rename-the-status-glyphs difficulty=3'
# shellcheck disable=SC2016 # capture_prep runs this text in the fixture, where $f expands
capture_prep "$P" 'mt wip rewrite-loader && mkdir -p src && for f in a b c d e; do echo x >src/$f.rs; done'
capture estimates-budget "$P" 'mt check'

# --- /read-many-projects/ -----------------------------------------------------
# The copy is `$HOME` and holds the at.toml under .config/, so `~/projects` is the
# copy's projects/. shop gets a git repository and one linked worktree, the
# fork that the `forks` key keeps out or lets in. The runs start outside every
# project, so each path prints whole, from `~`.
(
    capture_copy P examples/across-projects
    CAPTURE_HOME=$P CAPTURE_CONFIG_HOME=$P/.config
    CAPTURE_PATHS+=("$P=~")
    capture_prep "$P/projects/shop" "git init -q && git add -A && $COMMIT -m fixture && git worktree add -q ../shop-wishlist -b wishlist"
    capture_scratch Q
    capture_lines across-at "$Q" <<'EOF'
mt at
mt at work
EOF
    capture across-board "$Q" 'mt @work'
    capture_lines across-ls "$Q" <<'EOF'
mt @work ls
mt @work ls ready -n 1
mt @work find email
EOF
    capture across-check "$Q" 'mt @work check'
    capture schema-set-ls "$Q" "mt @work ls --json | jq 'map(del(.applied) | .tasks |= map(.id))'"
    capture schema-set-board "$Q" 'mt @work board --json'
    capture --exit 1 --expect 'runs on one project' across-write "$Q" 'mt @work add fix the coupon field'
    capture_lines across-forks "$Q" <<'EOF'
mt at all
mt at forks
mt @shop-wishlist ls
EOF
)
