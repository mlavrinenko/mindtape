#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "follow script to script calls in the scope derivation",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("gating",),
  estimate: (files: 2),
  assignee: "agent",
  status: done(2026, 9, 8),
)

== Summary

This derivation misses nested calls. `script-scope-check.sh`'s arm two derives a rule's mmz scopes from the recipe
body it can read. A script reached script-to-script rather than through
`just` is invisible to it. The caller's rule has to name that script's scope
by hand. Otherwise it serves a stale hit against a body it never hashed, and nothing
fails to say so. The header of `_nav-diff.sh` gives the count at which this stops
being an exemption and becomes a missing feature: "a third is the signal that
the derivation needs to follow nested calls instead of the list growing."
Four scripts stand outside the derivation: `_nav-diff.sh` on `just check-doc-facts` and `just
site-pages-fixture-check`, plus `_gitleaks-bound-check.sh` and
`_gitleaks-bound-control.sh` on `just check-secrets`.


== Scope

- make arm two's derivation transitive: a recipe body invoking
  `.just/scripts/x.sh` pulls in whatever `x.sh` itself invokes. The
  caller's rule then needs to name both scopes.
- the same read `check-crap`'s nested `just crap` needs, which is the other
  standing exemption in that script's header. Decide whether one derivation
  covers both shapes or they stay separate.
- a wrong answer here fails a correct manifest, so the current arm
  claims only direct invocation. The new arm needs its own control.

