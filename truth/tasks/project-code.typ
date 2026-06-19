#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Project code config slot",
  status: done(2026, 6, 19),
  priority: ice(impact: 5, confidence: 0.5, ease: 8),
  estimate: (hours: 1),
  tags: ("substrate",),
)

== Summary

Add `[project] code`, a short project qualifier, falling back to the project
root directory name when absent. speconaut needs a project code for cross-repo
references, and the list can use it for display. Store and expose the code only —
no cross-project reference syntax or resolution yet.

== Scope

- `[project] code` in config; fallback to the root directory name.
- Expose the resolved code on the loaded project model.

== Out of scope

Cross-project link syntax and resolution. Deferred deliberately until the
single-project model is settled.
