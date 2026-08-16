#!/usr/bin/env bash
# .just/scripts/site-pages-fixture-check.sh — the body of the
# `site-pages-fixture-check` recipe
# (truth/tasks/pin-the-docs-manifest-gate-failures-with-a-selftest.typ). Runs
# from the repo root, like the recipe that invokes it.
#
# Pin the three failures www/generate-site-pages.sh and check-doc-facts.sh's
# pages arm exist to catch — two pages declaring the same route, a page with
# no `<page-meta>` block, and a page missing from www/utils/site.typ's NAV —
# each demonstrated by hand once, when they landed
# (truth/tasks/derive-site-pages-json-from-the-pages-page-meta-blocks.typ),
# and never proven again since. The evidence lived only in a commit message;
# a gate that never fires reads identical to one that cannot.
#
# WHY A REPLICA. Mutating a page in the working tree to trip the producer
# cannot run here: `just check` runs its arms `[parallel]`, so a concurrent
# fmt-check/docs::check arm would read the mutation as real, and a failed
# assertion would leave the repo half-mutated. `cp -r www <tmp>` is proven
# to work instead: www/generate-site-pages.sh derives its own REPO from
# `${BASH_SOURCE[0]}`, so running the COPY (never the original) gives it a
# copied REPO, and it globs the copy's own content/*.typ. site.typ resolves
# tola.toml and www/generated/ relative to ITSELF rather than to the root, so
# the copy needs no re-pointing, and www/generated/ rides along in the same
# `cp -r` — it is already on disk by the time any `just check` arm reaches
# this gate, so the producer needs no captures regenerated.
#
# Cases 1 and 2 drive the replica's own copy of the real producer end to
# end. Case 3 does not: catching a page missing from NAV lives in check-doc-
# facts.sh, which needs a release `mt` build and a full regenerate before it
# can query a single route — far too heavy for a selftest. Its set-
# difference was extracted to .just/scripts/_nav-diff.sh for exactly this
# reason (callable with two lists), so this pins it directly against a
# synthetic pair instead of paying for the regenerate.
set -eo pipefail
fail=0

note() { echo "site-pages-fixture-check: $*"; }
problem() { echo "site-pages-fixture-check: $*" >&2; fail=1; }

# Asserts BOTH a non-zero exit and that the combined stdout+stderr NAMES the
# offending page or route — an exit-code-only assertion would pass for a
# script that failed for an unrelated reason, which is the exact hole this
# whole gate exists to close.
assert_catches() {
    local desc="$1" needle="$2"; shift 2
    local out
    if out=$("$@" 2>&1); then
        problem "$desc: expected a non-zero exit, got 0"
        printf '%s\n' "$out" >&2
        return
    fi
    if ! grep -qF "$needle" <<<"$out"; then
        problem "$desc: failed, but never named \"$needle\""
        printf '%s\n' "$out" >&2
        return
    fi
    note "$desc: caught, naming \"$needle\""
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# Each variant needs its OWN parent dir, with the copy's leaf directory still
# named "www": generate-site-pages.sh's REPO is the PARENT of the directory
# it lives in (see its own header), so `$variant/www/generate-site-pages.sh`
# derives REPO=$variant and globs `$variant/www/content/*.typ` correctly —
# renaming the leaf itself (e.g. to `$tmp/clean`) breaks that derivation and
# every page query fails "input file not found" before a single mutation is
# even in play.
mkdir -p "$tmp/clean"
cp -r www "$tmp/clean/www"

# Positive control: the UNMUTATED replica must pass. Without this, a probe
# that fails for an environmental reason (a missing tool, a broken `--root`)
# would look identical to three successful catches.
if out=$(bash "$tmp/clean/www/generate-site-pages.sh" 2>&1); then
    note "positive control: unmutated replica passes"
else
    problem "positive control: unmutated replica failed"
    printf '%s\n' "$out" >&2
fi

# Case 1: two pages declaring the same route. A verbatim copy of an existing
# page under a new filename declares the same route by construction.
mkdir -p "$tmp/dup-route"
cp -r "$tmp/clean/www" "$tmp/dup-route/www"
cp "$tmp/dup-route/www/content/concepts.typ" "$tmp/dup-route/www/content/concepts-dup.typ"
assert_catches "case 1 (duplicate route)" "concepts-dup" \
    bash "$tmp/dup-route/www/generate-site-pages.sh"

# Case 2: a page with no <page-meta> block. Plain markup, no imports needed —
# the point is the missing marker, not anything about the page's content.
mkdir -p "$tmp/no-meta"
cp -r "$tmp/clean/www" "$tmp/no-meta/www"
printf '= No meta page\n\nThis page intentionally has no page-meta block.\n' \
    > "$tmp/no-meta/www/content/no-meta-probe.typ"
assert_catches "case 2 (missing page-meta)" "no-meta-probe.typ" \
    bash "$tmp/no-meta/www/generate-site-pages.sh"

# Case 3: a page missing from site.typ's NAV. Cheap on purpose (see header
# above): a synthetic pair of route lists, not a replica.
assert_catches "case 3 (missing from NAV)" "/orphan/" \
    bash .just/scripts/_nav-diff.sh $'/a/\n/b/\n/orphan/' $'/a/\n/b/'

if [ "$fail" -ne 0 ]; then
    echo "site-pages-fixture-check: the docs manifest gate no longer catches everything it claims to" >&2
    exit 1
fi
echo "site-pages-fixture-check: all three manifest failures are still caught, naming the offender"
