#import "speconaut-stub.typ": *
#show: task.with(
  title: "Make speconaut-phpunit's step-stub appender class-aware",
  status: done(2026, 6, 11),
  priority: "low",
  grade: "middle",
  estimate-hours: 3,
  // Adapter trees are independent speconaut projects without a host
  // bounded-context grouping axis, so x-speconaut carries no contexts.
  x-speconaut: task-extras(
    decisions: (link("../decisions/class-aware-step-appender.decision-260605.typ"),),
  ),
)

== Summary
Stub body.

== Acceptance
just check passes.
