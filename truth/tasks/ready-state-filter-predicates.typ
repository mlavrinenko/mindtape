#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Ready-state predicates in the filter language",
  status: accepted(2026, 6, 20),
  priority: ice(impact: 5, confidence: 0.5, ease: 8),
  estimate: (hours: 4),
  relates-to: (link("absorb-task-graph-into-lib.typ"),),
  tags: ("cli",),
)

== Summary

Teach the filter language to ask "what can I start now". Add the dependency-aware
predicates `ready` and `blocked`, surface readiness as a column in `mindtape list`,
and float ready work to the top of the default order. Today every filter function
is pure per-task (`has`, `stale`, `search`); readiness is the first predicate that
needs the whole task set, so the real work is a set-aware evaluation seam that a
family of relational predicates can later reuse.

== Semantics

One-hop, status-only. A task is:

- ready — non-terminal and every direct `depends-on` target is terminal.
- blocked — non-terminal and some direct `depends-on` target is non-terminal or
  unresolved.
- neither — terminal (already `done` / `cancelled` / `failed`).

`ready()` and `blocked()` partition the non-terminal tasks; `blocked() == !ready()`
only within that set, so define each directly rather than as the other's negation.

Decisions baked in (flag on review):

- One hop, not transitive. You can start a task once its direct prerequisites are
  terminal; a grandparent that is still open keeps the parent non-terminal, which
  keeps this task blocked anyway. So toposort is not needed and dependency cycles
  cannot hang the check.
- Unresolved / dangling `depends-on` target (names no loaded task) counts as not
  terminal, so the task reads blocked. Conservative: you cannot confirm a missing
  prerequisite is done. Validation already reports the dangling link separately.
- History-pointer targets (SWHID tombstones) are already dropped from `depends_on`
  at model build, so a dependency retired by `cleanup` stops blocking — a task
  whose only blocker was retired becomes ready for free.

== Scope

- Set-aware filter seam. Precompute a small readiness fact over the loaded set and
  thread it into `Filter::matches` / `select` alongside the existing `&Config`.
  Compute it over the *full* loaded set, before the filter narrows the view — a
  terminal dependency hidden by `default_filter = status != "done"` must still
  count as satisfied, or every leaf reads blocked.
- Readiness primitive in `crates/mindtape`, consumer-blind: resolve each task's
  `depends-on` to file paths (reuse `resolve_link_path`), index the set by path,
  and return the ready / blocked classification keyed by task `file`. This is the
  stone `absorb-task-graph-into-lib.typ` builds toposort and `next` ranking on top
  of (see Boundary), not a duplicate of it.
- `ready` / `blocked` predicates over that fact, usable bare (`mindtape list ready`)
  or as a call (`ready()`), and composable (`ready && priority >= 7`). No `--ready`
  flag: the bare predicate is already that short, and it composes where a flag would
  not.
- Readiness as a sort dimension: float ready before blocked before settled in the
  default order (replacing a `next` command — see Boundary), and expose `ready` as
  an explicit `--sort` field. Handle resolution shares the default order, so it
  threads the same fact.
- Readiness column in the list table, signal-gated: shown only when some displayed
  task is blocked, so flat backlogs stay clean. Rides the existing `CellSpec`
  machinery (glyph + short label, TTY-gated like the others); built-in glyphs for
  now, a `[tasks.readiness]` legend later if it earns its keep. Compute it from the
  same full-set primitive in `list::run_with` and pass it to the renderer.
- Tests: vacuous ready (no deps), partial-terminal deps blocked, all-terminal ready,
  terminal-self is settled, dangling dep blocked, tombstoned dep unblocks, readiness
  computed over the full set not the filtered subset, ready-first ordering, the
  signal-gated column.

== Boundary with absorb-task-graph-into-lib

This task owns the one-hop readiness primitive, the set-aware filter seam, and the
readiness sort, and ships them now — it does not wait on the graph-absorb work,
which is itself blocked on the convergence ADR. `absorb-task-graph-into-lib.typ`
owns toposort and cycle detection (the transitive, ordering-sensitive half) and
reuses this primitive rather than re-deriving readiness. No `next` command on
either side: ranked-ready work is just the default sort, so `next` collapses into
`list`. Trim the `next` / readiness bullets from the absorb task's scope when this
merges.

== Stretch

The seam, once it exists, makes other inbound-edge predicates cheap — each is one
more precomputed set on the same context:

- `superseded()` — this task is a `supersedes` target of some other task; hide work
  retired by a replacement.
- `duplicated()` — symmetric, over `duplicates`.

Land these only if they fall out cleanly; otherwise spin a follow-up. `supersedes`
and `duplicates` are already core model fields, so no DSL change is required.
