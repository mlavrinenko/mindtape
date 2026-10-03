#!/usr/bin/env bash
# www/generate-cli.sh — capture the CLI reference's transcripts and examples,
# for www/content/cli.typ and the notes in www/utils/cli-notes.typ. Called by
# www/generate.sh with the repo root and the generated/ directory, inside the
# lock that script holds.
#
# Read commands run in examples/api-service in place. Commands that write run
# in a copy of a fixture, or in a project `mt init --no-input` makes. Each
# session gets its own project, so no session depends on what another wrote.
# Files a command reads (notes, an import, a link map) are written before the
# session; the reader brings their own.
set -eo pipefail

REPO="$1"
OUT="$2"
MT="$REPO/target/debug/mt"
FIX="$REPO/examples/api-service"

# The api-service tasks are dated in the last week of September 2026, so
# "today" is just after them.
export MINDTAPE_NOW=2026-10-01

# shellcheck source=www/capture.sh
source "$REPO/www/capture.sh"
capture_init

# A project `mt init --no-input` wrote, stored in <var>.
init_scratch() {
    capture_scratch "$1"
    capture_prep "${!1}" 'mt init --no-input'
}

# --- read commands, in place ------------------------------------------------
capture version "$FIX" 'mt --version'
capture ls "$FIX" 'mt ls'
capture views "$FIX" 'mt views'
capture check "$FIX" 'mt check'
capture flip-list "$FIX" 'mt flip --list'
capture eval "$FIX" 'mt eval tasks/throttle-failed-logins-per-ip.typ'
capture config-origin "$FIX" 'mt config show --origin'
capture ls-default "$FIX" "mt ls 'has_tag(\"billing\")' --columns @default --sort @default"
capture ls-cut "$FIX" "mt ls '' -n 5"
capture ls-file-when "$FIX" "mt config show --defaults | grep -A1 '^\[ls.columns.file\]'"

capture_lines ls-examples "$FIX" <<'EOF'
mt ls ''                        # empty filter: lists everything
mt ls 'has_tag("auth")'         # filter expression
mt ls ready -n 3                # predicate and limit; a table says what it cut
mt ls '' -n 5 --skip 5          # page two; the footer names the next --skip
mt ls @stuck                    # named view: filter, sort and columns
mt ls --json                    # shorthand for --format json
mt ls --format csv              # table (default), json, csv, path or path0
mt ls --columns slug#,title,file --hyperlinks always   # clickable slug# and file
mt ls ready --format path       # bare paths, one per line, for another program
mt ls 'has(due)' --format path0 | xargs -0 wc -l   # NUL-delimited
EOF
capture_lines ls-sort-examples "$FIX" <<'EOF'
mt ls --sort due:asc --sort priority:desc   # composed, repeatable
mt ls --sort @this-week                      # splice a view's sort chain
EOF
capture_lines ls-columns-examples "$FIX" <<'EOF'
mt ls --columns status,title    # one-off override of [ls] columns-order
EOF
capture_lines find-examples "$FIX" <<'EOF'
mt find redis                   # one term
mt find login token             # several terms conjoin (AND)
mt find redis --sort priority:desc -n 3   # every ls flag forwards
mt find email --format path | xargs wc -l
EOF
capture_lines config-examples "$FIX" <<'EOF'
mt config show              # effective config as re-parseable TOML
mt config show --defaults   # the embedded floor alone
mt config show --json       # effective config as JSON
EOF
capture_lines graph-examples "$REPO/examples/links" <<'EOF'
mt graph                                # every task, Graphviz DOT
mt graph '!terminal' --format mermaid   # the open tasks as a Mermaid flowchart
EOF

# --- commands that write, each session in its own project -------------------
init_scratch P
capture_lines add "$P" <<'EOF'
mt add fix the login bug
mt add --describe
mt add ship-it --format json
mt add ship-it --dedup skip
mt add ship-it --dedup add-suffix
EOF

# The jq adapters emit the default legend's status names, so the imports run
# in a project `mt init --no-input` made.
init_scratch P
mkdir -p "$P/contrib/jq"
cp "$REPO/contrib/jq/taskwarrior.jq" "$P/contrib/jq/"
cp "$REPO/contrib/jq/fixtures/taskwarrior.in.json" "$P/task-export.json"
cp "$REPO/contrib/jq/fixtures/taskwarrior.out.json" "$P/export.json"
capture_lines import-examples "$P" <<'EOF'
mt import export.json --dry-run   # from a file, preview only
mt import < export.json           # canonical JSON from stdin
jq -f contrib/jq/taskwarrior.jq task-export.json | mt import   # a Taskwarrior export
EOF

capture_copy P examples/api-service
capture_lines mv "$P" <<'EOF'
mt mv add-redis-client add-redis --dry-run
mt mv add-redis-client add-redis
EOF

init_scratch P
capture_prep "$P" 'mt add fix the login bug'
printf 'Fix the login bug on empty passwords\n' >"$P/title.txt"
capture_lines set "$P" <<'EOF'
mt set fix-the-login-bug priority=8
mt set fix-the-login-bug due=2026-07-10
mt set fix-the-login-bug tags+=backend
mt set fix-the-login-bug tags+=backend
mt set fix-the-login-bug title@title.txt
EOF

capture_copy P examples/links
printf 'search-too-slow:\n  depends-on: [auth-epic]\n' >"$P/links.yaml"
capture_lines link "$P" <<'EOF'
mt link write-docs depends-on wire-oauth
mt link drop-legacy-cli depends-on auth-epic write-docs
mt link write-docs related search-too-slow -m "same release"
mt link --apply links.yaml --replace --dry-run
mt link --apply links.yaml
mt link --apply - < links.yaml
EOF

capture_copy P examples/api-service
capture_prep "$P" "printf '#!/bin/sh\\nexit 0\\n' >scripts/test"
capture done "$P" 'mt done reject-expired-sessions-on-token-refresh'

capture_copy P examples/api-service
capture_lines flip-examples "$P" <<'EOF'
mt flip done reject-expired-sessions-on-token-refresh document-the-webhook-retry-policy -m "shipped in v1.9.0" --force   # batch, with a note, waiving the failing test gate
mt flip wip add-redis-client --quiet   # no [[on.flip]] show or list output
EOF

capture_copy P examples/api-service
capture_lines review-examples "$P" <<'EOF'
mt review replace-the-deprecated-search-client   # stamp reviewed: with today's date
EOF

capture_copy P examples/api-service
capture_lines recur "$P" <<'EOF'
mt recur rotate-the-jwt-signing-key --dry-run    # preview without writing
mt recur rotate-the-jwt-signing-key              # spawn a new file, or refresh in place
mt recur rotate-the-jwt-signing-key --force      # spawn again even if a successor exists
mt recur rotate-the-jwt-signing-key --skip       # advance the anchor and count, spawn nothing
EOF

capture_copy P examples/api-service
capture_prep "$P" 'git init -q && git add -A && git -c user.name=docs -c user.email=docs@example.com -c commit.gpgsign=false commit -qm fixture'
capture_lines rm "$P" <<'EOF'
mt rm terminal --dry-run                                      # the plan, nothing written
mt rm fix-the-404-page-typo --yes                             # by reference
mt rm upgrade-the-axum-web-framework pin-the-rust-toolchain-in-ci --yes   # two references, one pass
mt rm 'status == "done"' --yes                                # by filter: every match
EOF

capture_scratch P
capture_lines init-examples "$P" <<'EOF'
mt init --list-presets            # the built-in preset names
mt init --preset flow --annotate  # a named preset, with the reasoning comments kept
EOF

(
    capture_home
    init_scratch P
    capture_lines preset-examples "$P" <<'EOF'
mt preset save my-shape   # this project's config and templates, as a preset
mt preset ls              # built-in and user presets together
EOF
)

(
    capture_home
    capture_copy P examples/api-service
    mkdir -p "$CAPTURE_CONFIG_HOME/mindtape"
    printf '[[worktrees]]\nname = "api"\npath = "%s"\n' "$P" >"$CAPTURE_CONFIG_HOME/mindtape/worktrees.toml"
    CAPTURE_PATHS+=("$P=~/api-service")
    capture_scratch Q
    capture_lines worktrees-examples "$Q" <<'EOF'
mt worktrees      # the configured @name aliases
mt @api ls        # @name is -w <path>, rewritten before parsing
EOF
)
