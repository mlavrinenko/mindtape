#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Compare declared artifacts against the working tree",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 5,
  tags: ("mt",),
  estimate: (files: 3),
  kind: "feat",
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: depends-on("record-the-artifacts-a-task-expects-to-touch.typ"),
)

== Summary

The `created` and `changed` edges from `record-the-artifacts-a-task-expects-to-touch.typ` direct the implementer but check nothing at rest. The one live comparison that cannot hit a dead path is the drift check, which only runs against in-flight tasks and a live working tree.


== Scope

Extend `crates/mindtape/src/estimate_drift.rs` so an in-flight task that declares artifacts gets a warning naming each changed file outside every declared `changed` glob and `created` path, alongside the existing count comparison. Decide whether an undeclared `created` path that never appears also warns. Red tests in `estimate_drift_tests.rs`; describe the comparison in `www/content/estimates.typ`.

== Dogfood

Declare this task's own `created`/`changed` edges and confirm `mt check` compares them against the tree while it is wip.
