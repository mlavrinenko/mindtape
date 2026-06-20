#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Converge task tracking with speconaut",
  status: proposed(2026, 6, 14),
  priority: ice(impact: 8, confidence: 0.8, ease: 9),
  depends-on: (link("convergence-boundary-adr.typ"),),
  relates-to: (
    link("absorb-task-graph-into-lib.typ"),
    link("swh:1:rev:f4d7b80eda0be8df86149c81b6d149a5df8a6dc6;path=truth/tasks/extract-typst-harvest.typ"),
  ),
  tags: ("speconaut",),
)

== Summary

speconaut stops shipping its own task model and depends on the MindTape library
(`crates/mindtape`). Its task files import `@local/mindtape` directly — not a
re-export — so a MindTape-only user can author, list, and validate tasks inside a
speconaut worktree, and a speconaut task stays a valid MindTape task. speconaut
ships its own `.mindtape/config.toml` (its status lifecycle, its fields) and
passes speconaut-only fields through the `x-speconaut:` extension slot. MindTape
never references speconaut.

== Scope (mindtape side)

- No re-export shim: tasks use `#import "@local/mindtape:0.2.0"`. The
  `@local/speconaut` package keeps `feature` / `scenario` / `decision` / ... and
  drops `task`. Tasks and specs are separate packages — two concerns, two
  packages.
- `crates/mindtape` exposes the typed task model plus the graph logic speconaut
  hand-rolls today (see `absorb-task-graph-into-lib.typ`).
- The core-vs-extras field boundary is recorded in the convergence ADR: generic
  lifecycle fields land in MindTape core; speconaut-domain fields (contexts,
  decisions, code links) stay `x-speconaut`.

The speconaut-side migration (delete its task layer, depend on the library) is
tracked in the speconaut repo. Follow-up to
`docs/decisions/002-per-file-rewrite-and-speconaut-convergence.md`.
