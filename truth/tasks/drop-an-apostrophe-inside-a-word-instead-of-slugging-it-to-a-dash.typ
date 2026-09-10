#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "drop an apostrophe inside a word instead of slugging it to a dash",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (files: 3),
  assignee: "agent",
  status: done(
    2026,
    9,
    10,
  )[All three functions fixed and tested: `add_values::slug`, `import_map::stem::slug_from_title`, `derive::slugify`. See the Decision below on the fourth question.],
)

== Summary

The slug rule keeps Unicode alphanumerics and collapses every other run to a
single `-`, so an apostrophe inside a word splits it: `make a page's active
argument an error` mints `make-a-page-s-active-argument-an-error`. The `-s-`
is not a word boundary and reads as noise in a filename that is also the
task's identity.

The backlog already shows both ways around it. `make-a-page-s-active-argument-an-error.typ`
quoted the title and took the stem. `format-the-projects-own-templates.typ`
carries `title: "format the project's own templates"` over a stem with no
apostrophe at all — a title typed without one at the shell, then repaired by
hand afterwards, which is a hand edit the tool should not have asked for.


== Scope

Drop `'` (and the typographic `’`) before the run-collapsing step rather than
treating it as a separator, so `page's` mints `pages` and quoting a possessive
title stops costing a stem.

Three functions carry the rule and all three must move together:

- `crates/mindtape-cli/src/add_values.rs`, `slug` — `mt add`'s stem.
- `crates/mindtape/src/import_map/stem.rs`, `slug_from_title` — `mt import`'s.
  Duplicated from the first on purpose (the CLI crate depends on this one, not
  the reverse) so both mint identical stems from an identical title; that
  module's header says so, and a fix to one alone silently breaks the
  guarantee.
- `crates/mindtape/src/derive.rs`, `slugify` — folder name to derived tag, so
  a folder named `ada's-drills` currently derives `ada-s-drills`.

A regression test per function, since the duplication means one test cannot
cover the others.

No existing filename moves: a stem is minted once at `mt add` time and the
file is the identity thereafter. Only stems minted after the change differ,
so there is nothing to migrate and no link to rewrite.

= Decision

`derive.rs`'s `slugify` moved with the other two. MindTape has not released
yet, so a changed derived-tag spelling is not a compatibility break anyone
is exposed to; consistency across all three slug sites won out over treating
the tag vocabulary as a surface to protect early.

Fixing all three identically grew `add_values::slug` and
`import_map::stem::slug_from_title` into a clone past `just check-dry`'s
jscpd floor, since they were already byte-for-byte duplicates before this
task. Filed and closed as
[stop-duplicating-mt-adds-slug-in-mindtapes-slug-from-title](stop-duplicating-mt-adds-slug-in-mindtapes-slug-from-title.typ):
`add_values::slug` now calls `mindtape::slug_from_title` instead of
re-implementing it.

