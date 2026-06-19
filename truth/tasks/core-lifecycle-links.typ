#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Add supersedes and duplicates as core task links",
  status: done(2026, 6, 18),
  priority: 5,
  estimate: (hours: 3),
  depends-on: (link("convergence-boundary-adr.typ"),),
  tags: ("substrate",),
)

== Summary

Promote `supersedes` and `duplicates` from speconaut-only fields to MindTape core
task slots. Both are generic task-lifecycle links: a task can supersede or
duplicate another, and the SWHID cleanup (ADR 003) wants them to record why a
retired task was tombstoned. Today MindTape core carries only `depends-on` (hard)
and `relates-to` (soft).

== Scope

- `lib/prelude.typ`: add `supersedes`, `duplicates` to `_core-slots` and the
  emitted marker fields (link lists, like `depends-on`).
- `crates/mindtape`: add the fields to the `Task` model, normalize as task-id
  links, surface in validation (resolve to known task ids).
- Config: no new enum; they are link lists. Broken-link resolution applies.
- Gated by the convergence ADR's core-vs-extras decision — land only the fields
  the ADR promotes.
