#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "The site-pages replica copies www/public from a parallel arm",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("gating",),
  estimate: (files: 1),
  assignee: "agent",
  status: proposed(2026, 8, 21),
  links: related("site-pages-fixture-check-fails-on-a-cold-checkout.typ")[
    the same family, found while fixing that one: a gate reading a directory
    a parallel arm owns
  ],
)

== Summary

`site-pages-fixture-check` builds its replica with a faithful `cp -r www`.
That copies `www/public` too — 6.1 MB of built site, gitignored, and written
by `tola build` in the `docs-check` arm, which `just check` runs in PARALLEL
and which `rm -rf`s the directory on entry.

Two costs, one of them a race:

- the replica pays a multi-megabyte copy per variant for a directory no page
  compile reads
- `cp -r` walking a tree another process is deleting can exit non-zero on a
  file that vanished mid-walk, failing the gate for a reason that has nothing
  to do with what it pins

The captures half of exactly this bug was just closed
(`site-pages-fixture-check-fails-on-a-cold-checkout.typ`): the gate now
produces `www/generated/` itself inside the flock and snapshots before the
lock drops. `www/public` is the same shape and was deliberately left, because
narrowing the copy is a change to what "replica" means and did not belong in
a bug fix riding on a different claim.

== Why it has not bitten yet

The window is narrow — `docs-check` deletes `public/` once, at the start of
its own `tola build` — and the `cp -r` is fast. Nothing in the log says it has
ever fired. That is an argument about probability, not about correctness, and
the gate it would fail is the one that decides whether a change is green.

== Scope

- Exclude `www/public` from the replica copy, keeping everything a page
  compile actually reads
- Prefer a form that does not silently drift as `www/` grows: an explicit
  exclusion of the one gitignored build output reads better than an
  allow-list of what to copy, which owes a line per new directory
- Argue the choice in the script header beside the `cp -r` reasoning already
  there, which currently justifies copying the tree faithfully
- Confirm the three fixtures still fire by name and the positive control
  still passes — a replica missing something a page reads fails for an
  environmental reason, which is the failure mode this whole family is about
- Out of scope: the flock discipline, settled by the sibling task
