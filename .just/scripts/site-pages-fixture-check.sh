#!/usr/bin/env bash
# .just/scripts/site-pages-fixture-check.sh — the body of the
# `site-pages-fixture-check` recipe
# (truth/tasks/pin-the-docs-manifest-gate-failures-with-a-selftest.typ). Runs
# from the repo root, like the recipe that invokes it.
#
# Pin the three failures www/generate-site-pages.sh and check-doc-facts.sh's
# pages arm exist to catch — two pages declaring the same route, a page with
# no `<page-meta>` block, and a page missing from www/utils/site.typ's `NAV` —
# each demonstrated by hand once, when they landed
# (truth/tasks/derive-site-pages-json-from-the-pages-page-meta-blocks.typ),
# and never proven again since. The evidence lived only in a commit message;
# a gate that never fires reads identical to one that cannot.
#
# Why a replica. Mutating a page in the working tree to trip the producer
# cannot run here: `just check` runs its arms `[parallel]`, so a concurrent
# fmt-check/docs::check arm would read the mutation as real, and a failed
# assertion would leave the repo half-mutated. A copy of www/ under a temp
# parent is proven to work instead (what the replica leaves out, below, is the
# one thing it does not copy): www/generate-site-pages.sh derives its `REPO` from
# `${BASH_SOURCE[0]}`, so running the copy (never the original) gives it a
# copied `REPO`, and it globs the copy's own content/*.typ. site.typ resolves
# tola.toml and www/generated/ relative to itself rather than to the root, so
# the copy needs no re-pointing.
#
# Why it regenerates first. www/generated/ is gitignored (www/.gitignore) and
# written by the docs-gen arm, which `just check` runs in parallel with this
# one. On a warm checkout it is already on disk and rides along in the same
# `cp -r`. On a cold tree — a fresh clone, or a `wt switch --create` worktree, which
# warms no cache by design — the producer's very first page query dies on
# `file not found` for generated/crate-map.json (site.typ's PKG_VERSION), the
# positive control fails, and cases 1 and 2 report "failed, but never named"
# without ever reaching the mutation they exist to catch. That reads exactly
# like this gate catching a real break in whatever change is being gated, on
# the run that decides whether it is green.
#
# So the gate carries its own inputs, the way check-doc-facts.sh already
# carries the same ones for the same stated reason: it runs www/generate.sh
# itself rather than trusting whatever www/generated/ happens to hold. The
# alternative — stubbing the captures into the replica — is cheaper (no `mt`
# build at all) and was rejected on what it costs the fixture. Querying one
# `<page-meta>` block is a full compile, so a stub owes every artifact any
# page reads at the top level: crate-map.json, dsl-fields.json, cli.json,
# config-schema.json, release-targets.json, site-pages.json, the generated
# `just` module, and a dozen transcripts and SVGs — and every capture
# www/generate.sh grows later owes the stub one more, with nothing to fail
# when it does not get it. A stub that falls behind fails the positive
# control for an environmental reason, which is precisely the bug being fixed
# here; a stub that keeps up is www/generate.sh with a second implementation
# and a second place for the fixture to be wrong. Measured cost of the real
# producer: 4s warm, on top of a debug `mt` build every other arm pays anyway.
#
# Why not a `docs::gen` dependency, the other option on the table. It would
# put the captures on disk and then release target/www-generated.lock, and the
# `cp -r` below is itself a reader of a directory the concurrent docs::check
# arm `rm -rf`s on entry — the same trap that made check-doc-facts.sh hold the
# lock across its typst queries and not merely across its own regenerate
# (argued in www/utils/gate-notes.typ's header). The regenerate and the
# snapshot have to sit inside one lock hold or the copy can still catch the
# directory mid-rebuild, and a recipe dependency cannot express that. So the
# lock is taken here, with the same fd-held idiom and MINDTAPE_GENERATED_LOCK
# handshake www/generate.sh, docs/generate-md.sh and check-doc-facts.sh use,
# and dropped the moment the snapshot exists: the three replica runs that
# follow read nothing under the repo, and holding a repo-wide lock across ~45
# page compiles would serialize docs::check and check-doc-facts behind this
# gate for nothing.
#
# What the replica leaves out. The snapshot copies www/ entry by entry so it
# can skip two gitignored build outputs no page compile reads: public/ (6.1 MB
# of built site, from `tola build` plus its pagefind index) and .tola/ (1.9 MB,
# tola's own vdom cache). Both are owned by the docs::check arm `just check`
# runs in parallel with this one, and the copy is a reader of them — the same
# shape as the captures problem above, one directory further out (truth/tasks/
# the-site-pages-replica-copies-www-public-from-a-parallel-arm.typ).
#
# Measured, because the filed task assumed worse. Neither `tola build` nor
# `pagefind` wipes public/ on entry — a sentinel file dropped there survives
# both — so this is not a `cp` racing an `rm -rf`, and the paragraph above is
# the only place in this script where that is the mechanism. What is measured
# is that `tola build` rewrites every www/.tola/cache/*.vdom and index.json in
# place on each run: the copy walks a tree another process is replacing files
# inside, and whether that hands back a torn file or a vanished one is a
# property of tola's write discipline, not of anything this gate controls. The
# gate was paying 8 MB per variant, three variants, to stay exposed to a
# question with no bearing on what it pins.
#
# Excluded by name rather than by copying an allow-list of what the pages
# need. An allow-list owes a line for every directory www/ grows, and the day
# someone forgets one it fails the positive control for an environmental
# reason — the failure mode this whole gate exists to be free of. A named
# exclusion fails the other way: a new directory rides along and costs a copy.
# The positive control below is what proves nothing a page reads was dropped,
# and the assertion beside the copy is what stops a revert to a whole-tree
# `cp -r www` putting both directories back unnoticed.
#
# Cases 1 and 2 drive the replica's own copy of the real producer end to
# end. Case 3 does not: catching a page missing from `NAV` lives in check-doc-
# facts.sh, which needs a release `mt` build and a full regenerate before it
# can query a single route — far too heavy for a selftest. Its set-
# difference was extracted to .just/scripts/_nav-diff.sh for exactly this
# reason (callable with two lists), so this pins it directly against a
# synthetic pair instead of paying for the regenerate.
set -eo pipefail
fail=0

problem() { echo "site-pages-fixture-check: $*" >&2; fail=1; }

# Asserts both a non-zero exit and that the combined stdout+stderr names the
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
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# The captures every page compile reads, produced inside the lock the
# snapshot below is taken under — see why it regenerates first above. The
# producer's own output is held back unless it fails: a green run has nothing
# to say here, and a red one has to name www/generate.sh rather than surface
# three lines later as an unexplained positive-control failure.
mkdir -p target
exec 9>target/www-generated.lock
flock 9
export MINDTAPE_GENERATED_LOCK=1
if ! gen_out=$(mmz just docs::gen 2>&1); then
    printf '%s\n' "$gen_out" >&2
    problem "could not regenerate www/generated/: the replica has no captures to compile against"
    exit 1
fi

# Each variant needs its own parent dir, with the copy's leaf directory still
# named "www": generate-site-pages.sh's `REPO` is the parent of the directory
# it lives in (see its own header), so `$variant/www/generate-site-pages.sh`
# derives REPO=$variant and globs `$variant/www/content/*.typ` correctly —
# renaming the leaf itself (e.g. to `$tmp/clean`) breaks that derivation and
# every page query fails "input file not found" before a single mutation is
# even in play.
#
# `dotglob` so www/.gitignore and www/.ignore ride along the way a whole-tree
# `cp -r www` carried them; bash never yields `.`/`..` from a glob, so the
# loop needs no guard against them. The two skipped names are argued in what
# The replica leaves out above. The variant copies below are made from this
# snapshot, not from www/, so each of them is short the same two directories.
mkdir -p "$tmp/clean/www"
shopt -s dotglob
for entry in www/*; do
    case "${entry#www/}" in
        public | .tola) continue ;;
    esac
    cp -r "$entry" "$tmp/clean/www/"
done
shopt -u dotglob

# Regression pin for that exclusion: a revert to `cp -r www` would restore
# both directories with nothing else in the gate reacting. Checked rather than
# assumed because the whole point of the loop is a property of the copy.
for skipped in public .tola; do
    if [ -e "$tmp/clean/www/$skipped" ]; then
        problem "replica copied www/$skipped: no page compile reads it and the parallel docs::check arm writes it"
    fi
done

# Snapshot taken: closing the fd releases the lock, and dropping the
# handshake stops this process claiming to hold one — so each replica
# producer takes its own lock over its own temp `REPO`, exactly as a standalone
# run does.
exec 9>&-
unset MINDTAPE_GENERATED_LOCK

# Positive control: the unmutated replica must pass. Without this, a probe
# that fails for an environmental reason (a missing tool, a broken `--root`)
# would look identical to three successful catches.
if ! out=$(bash "$tmp/clean/www/generate-site-pages.sh" 2>&1); then
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

# Case 3: a page missing from site.typ's `NAV`. Cheap on purpose (see header
# above): a synthetic pair of route lists, not a replica.
assert_catches "case 3 (missing from NAV)" "/orphan/" \
    bash .just/scripts/_nav-diff.sh $'/a/\n/b/\n/orphan/' $'/a/\n/b/'

if [ "$fail" -ne 0 ]; then
    echo "site-pages-fixture-check: the docs manifest gate no longer catches everything it claims to" >&2
    exit 1
fi
