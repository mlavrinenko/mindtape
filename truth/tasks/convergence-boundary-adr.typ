#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "ADR: mindtape <-> speconaut task-layer convergence boundary",
  status: done(2026, 6, 18),
  priority: "high",
  estimate: (hours: 2),
  tags: ("docs", "speconaut"),
)

== Summary

Write ADR 004 fixing the boundary between MindTape (task tracking) and speconaut
(specs<->code). MindTape owns the task model, DSL, config-driven validation,
list/show/add, the dependency graph, ranking, and task cleanup. speconaut owns
specs<->code (extraction, codegen, audit-traces, diff, lock, status) plus the
spec-first session workflow. speconaut depends on `crates/mindtape`; MindTape
stays spec-blind and never references speconaut.

== Scope

- Decide the core-vs-extras field split. Candidates to promote to MindTape core
  (generic task lifecycle): `supersedes`, `duplicates`, `reviewed`. Keep
  speconaut-domain fields as `x-speconaut`: contexts (DDD), decisions, code
  links.
- One task DSL: `@local/mindtape` `task()`. speconaut's `@local/speconaut`
  package drops `task()`. Specs and tasks are separate packages — two concerns,
  two packages. No re-export shim: a MindTape-only checkout authors, lists, and
  validates tasks in a speconaut worktree unchanged, and a speconaut task stays
  a valid MindTape task.
- speconaut depends on the MindTape library for the typed model plus the graph
  queries (see `absorb-task-graph-into-lib.typ`); it does not re-implement them.
- The session FSM stays in speconaut — it is spec-first commit enforcement, not
  task tracking — but re-anchors onto a MindTape `Task` by id.

== Decision criteria

State the deprecation criterion: the branch-scoped convergence section in
`CLAUDE.md` is removed on merge; this ADR supersedes the convergence note in ADR
002. Next number is `docs/decisions/004-*`; include Status, Date, Context,
Decision, Consequences, Deprecation Criteria.
