#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "pin the docs manifest gate failures with a selftest",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 5,
  tags: ("tests", "gating", "www", "tooling"),
  estimate: (files: 6),
  links: related("derive-site-pages-json-from-the-pages-page-meta-blocks.typ")[
    Added the three failures this pins, and demonstrated them by hand.
  ],
  status: done(2026, 8, 17),
)

== Summary

`www/generate-site-pages.sh` and `check-doc-facts.sh`'s pages arm exist to fail
on three things: two pages declaring the same route, a page with no
`<page-meta>` block, and a page missing from `site.typ`'s NAV. All three were
demonstrated by hand when they landed — each was seen to fail, naming the
offender — and the evidence lives in a commit message.

Nothing pins them. A refactor of either script can silently stop checking, and
the gate goes green either way, because a gate that never fires is
indistinguishable from a gate that cannot. That is the same shape as the hole
this whole change set closed: `check-doc-facts` compared filenames against a
manifest derived from the same hand-written source, so it could not have caught
what it was there to catch.

Rust has `cargo test`; every shell gate in `.just/scripts/` has nothing.

== Constraints

The obvious approach — mutate a page, run the producer, assert non-zero —
cannot run against the working tree: a failed assertion leaves the repo
half-mutated, and `just check` is `[parallel]`, so a concurrent arm would read
the mutation as real.

A temp copy works and is proven: `cp -r www <tmp>/www` then
`typst query --root <tmp>/www …` resolves, because `site.typ` reads its own
paths relative to ITSELF rather than to the root (verified during the umbrella's
feasibility probe). `generate-site-pages.sh` lives under `www/`, so a copied
tree gives it a copied `REPO` too, and it globs `$REPO/www/content/*.typ`. Copy
`www/generated/` along with it and the producer needs no captures regenerated.

`check-doc-facts.sh` is the harder half: it regenerates via `www/generate.sh`,
which builds `mt` release and runs it against a fixture. Pinning its NAV arm
probably means extracting that set-difference into something callable with two
lists, rather than driving the whole script.

== Scope

- A selftest script under `.just/scripts/`, and a recipe for it.
- Three cases minimum, each asserting BOTH a non-zero exit and that the message
  names the offending page or route — an assertion on the exit code alone
  passes for a script that fails for an unrelated reason.
- The gate-coupled dependents a new `[group("gate")]` recipe drags in: it lands
  in `gates.json`, so `check-doc-facts` demands a `www/utils/gate-notes.typ`
  entry for it, CONTRIBUTING.md's gate table regenerates, and `.mmz/config.yaml`
  needs a scope (one per script — see the note in that file).

== Open question

Whether this is one selftest for the docs manifest or the first case of a
general harness for `.just/scripts/**`. Every script there has the same
problem; this is just the one where three specific failures were promised in a
commit message and nothing holds them to it. Decide before naming the recipe,
since a name like `check-manifest-selftest` forecloses the general answer.
