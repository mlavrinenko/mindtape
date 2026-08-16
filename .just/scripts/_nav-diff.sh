#!/usr/bin/env bash
# .just/scripts/_nav-diff.sh — the set-difference that used to sit inline as
# check-doc-facts.sh's pages arm, part two, extracted under
# truth/tasks/pin-the-docs-manifest-gate-failures-with-a-selftest.typ so
# site-pages-fixture-check.sh can pin it against a cheap, synthetic pair of
# route lists instead of driving check-doc-facts.sh's whole regenerate (a
# release `mt` build plus a full page-compile pass) just to reach two lines
# of `comm`.
#
# Takes two newline-separated route lists as its only two arguments — the
# pages manifest's routes ($1, www/generated/site-pages.json's keys) and
# www/utils/site.typ's NAV routes ($2) — and knows nothing about how either
# was produced, so a caller can hand it real ones or made-up ones. Leading
# underscore: no `just` recipe wraps it directly, the same reason
# `_mutants-env.sh` carries one — every caller here invokes it as a plain
# script, never through `just`.
#
# Two failure directions, neither implying the other: a page with no NAV
# entry is invisible in the sidebar, and a NAV entry naming no real page's
# route 404s the moment it's clicked.
set -eo pipefail
routes="$1"
nav_routes="$2"

unlisted=$(comm -23 <(sort <<<"$routes") <(sort <<<"$nav_routes"))
if [ -n "$unlisted" ]; then
    echo "nav-diff: pages missing from www/utils/site.typ's NAV (invisible in the sidebar):" >&2
    while IFS= read -r route; do echo "  - $route" >&2; done <<<"$unlisted"
    exit 1
fi

orphaned=$(comm -13 <(sort <<<"$routes") <(sort <<<"$nav_routes"))
if [ -n "$orphaned" ]; then
    echo "nav-diff: www/utils/site.typ's NAV routes with no page declaring them (404 in the sidebar):" >&2
    while IFS= read -r route; do echo "  - $route" >&2; done <<<"$orphaned"
    exit 1
fi

echo "nav-diff: $(wc -l <<<"$routes") routes checked against NAV, no mismatch"
