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
  status: cancelled(
    2026,
    9,
    15,
  )[Backlog review 2026-09-15: git already records the files a task touched, and nothing would consume a declared list.],
  links: depends-on("read-the-artifacts-field-into-the-task-model.typ"),
)

== Summary

The `artifacts` field from `record-the-artifacts-a-task-expects-to-touch.typ` directs the implementer but checks nothing at rest. The one live comparison that cannot hit a dead path is the drift check, which only runs against in-flight tasks and a live working tree.


== Scope

Extend `crates/mindtape/src/estimate_drift.rs` so an in-flight task that declares artifacts gets a warning naming each changed file outside every declared `changed` glob and `created` path, alongside the existing count comparison. A declared `created` path that does not exist yet never warns: the check runs while the task is wip, when a file still to be created is expected to be absent. Red tests in `estimate_drift_tests.rs`; describe the comparison in `www/content/estimates.typ`.

== Dogfood

Declare this task's own `artifacts` and confirm `mt check` compares them against the tree while it is wip.
