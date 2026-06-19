#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Priority as a numeric-field instance",
  status: wip(2026, 6, 19),
  priority: 5,
  estimate: (hours: 4),
  depends-on: (link("numeric-field.typ"),),
  tags: ("substrate", "cli"),
)

== Summary

Make the priority slot an instance of the numeric-field primitive: a number on a
configured scale, default 0..9, classified by bands. Drop the `fields.priority`
string enum. The generic numeric machinery (model, scale config, numeric filter
and sort — which fixes today's lexicographic mis-sort where high < low < normal)
lives in numeric-field; this task only adopts it for priority and ships priority's
defaults.

== Scope

- DSL: `priority: 4`, or a band-label sugar. `[tasks.priority]` selects the scale
  (default 0..9) and band classification.
- Migrate default-config.toml, the dogfood `.mindtape/config.toml`, README, and
  the prelude to numeric priority.
- Migrate existing `truth/tasks` priorities to the numeric scale when landing.

== Boundary

Priority frameworks (computed scores) are a separate layer above this; difficulty
is the sibling instance in anchor style.
