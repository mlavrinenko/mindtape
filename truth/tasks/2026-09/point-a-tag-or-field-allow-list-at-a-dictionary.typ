#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Point a tag or field allow list at a dictionary",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("mt",),
  estimate: (files: 6),
  assignee: "agent",
  status: done(
    2026,
    9,
    11,
  )[landed: \@\<dict> refs in \[tasks.tags\] allowed and \[tasks.fields.\<name>\] values, expanded at config load],
  links: depends-on("add-named-dictionaries-to-the-configuration.typ")
    + depends-on("move-the-config-page-deep-dives-onto-pages-of-their-own.typ"),
)

== Summary

`[tasks.tags] allowed` and `[tasks.fields.<name>] values` each spell their
vocabulary out inline. Two projects sharing a taxonomy copy it, and a list
long enough to be worth naming must appear in the config either way.

With `[dicts]` in place the vocabulary already has a name. These two keys
should be able to point at it instead of restating it.


== Scope

+ Accept a `@<dict>` reference wherever `[tasks.tags] allowed` and
  `[tasks.fields.<name>] values` accept a list, expanded at config load so
  every downstream reader (validation, `mt add` slot completion, the schema)
  sees a plain set and works unchanged.
+ Allow mixing: a list may hold literal entries and `@<dict>` references
  together, unioned in order.
+ Refuse an unknown dict name at load with a nearest-match suggestion, the same
  way an unknown view name fails.
+ Document the reference form on the config page beside both keys, and cover a
  pure reference, a mixed list, and the unknown-name error.

