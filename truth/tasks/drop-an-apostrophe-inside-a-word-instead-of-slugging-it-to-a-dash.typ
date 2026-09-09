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
  status: proposed(2026, 9, 9),
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

= Watch for

The third function is a judgment call rather than a consequence: a derived
tag is a controlled vocabulary a project filters on, and changing how one is
spelled changes what `has_tag(...)` matches for anyone whose folder carries
an apostrophe. Should `derive.rs` move with the other two, or is a tag
vocabulary a compatibility surface that should stay put?

