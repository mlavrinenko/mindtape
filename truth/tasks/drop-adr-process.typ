#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Drop the ADR process; track decisions as tasks",
  status: done(2026, 6, 19),
  priority: 2,
  estimate: (hours: 2),
  tags: ("docs", "tooling"),
)

== Summary

The ADR process is in bad shape; stop using it. Architectural and tooling
decisions live as task artifacts under `truth/tasks/` instead. Excise the ADR
mandate from the rules and reconcile existing references.

== Scope

- Remove the ADR mandate from CLAUDE.md (agent rule 5) and CONTRIBUTING.md
  ("Extending MindTape" and the commit `Refs:` policy).
- Commit `Refs:` footers always point at a task file now.
- Decide the fate of `docs/decisions/` (002 plus the convergence-boundary ADR
  task): keep frozen as history, or migrate to tasks.
- Drop stale ADR references in existing task bodies (e.g. "ADR 003").
