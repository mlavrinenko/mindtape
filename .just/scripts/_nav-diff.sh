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
# THAT USED TO COST A GATE. script-scope-check's arm two derived a rule's
# scopes from the recipe body it could see, so a script reached script-to-script
# rather than through `just` was invisible to it and every caller's mmz rule had
# to name `script-nav-diff` BY HAND or serve a stale hit after this file
# changes, with nothing failing to say so. Adding a caller was a thing to
# remember. The count this header named as the signal to fix that was reached,
# and the derivation now follows the hop
# (truth/tasks/follow-script-to-script-calls-in-the-scope-derivation.typ): a
# caller that does not name `script-nav-diff` fails the gate with the chain that
# reached this file. Both callers still carry the input in .mmz/config.yaml
# (`just check-doc-facts` and `just site-pages-fixture-check`) — what changed is
# that neither is trusted to.
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
