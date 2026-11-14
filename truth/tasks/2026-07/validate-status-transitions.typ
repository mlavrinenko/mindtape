#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Validate status transitions",
  priority: ice(
    impact: 4,
    confidence: 0.7,
    ease: 6,
  ),
  tags: (),
  status: done(
    2026,
    7,
    12,
  )[landed: per-status to-graph, `mt flip --force` plus an allowed-next message, `mt check` reachability, well-formedness in `Config::validate`. Dogfood declares the graph, and `mt check` is green],
)

== Summary

Add optional, config-declared allowed status transitions, enforced by `mt flip`
and `mt check`. Today any legend value is reachable from any other. dstask and
Taskwarrior model a lifecycle (pending → active → resolved), while this project's
legend is a flat enum. This extends the "typed and validated" edge the
comparison credits to the tool, and stays opt-in. Declare transitions to get a
state machine, or omit the section to keep today's any-to-any behaviour.

== Constraints

- Opt-in and backward-compatible: absent config means unconstrained, exactly as
  today. No existing backlog should start failing `mt check` on upgrade.
- Terminal states stay terminal unless the config declares an explicit reopen edge.

== Decision

Config shape: inline per-status `to = [...]` on each `[[tasks.status]]` entry:
an adjacency list colocated with the status it leaves. Chosen over a top-level
`[tasks.transitions]` map, to keep each state's outgoing edges next to its
declaration. Reopen and back-edges are ordinary entries in the target-less
state's `to` (or a terminal state gaining a `to`).

Enforcement is asymmetric by design, and `mt check` is deliberately weaker than
`mt flip`:

- `mt flip` sees `from → to` and enforces the edge fully.
- `mt check` sees only the current snapshot (a `Status` is a single
  `(name, date, note)`, no history). It cannot prove a task walked a legal path.
  It can only flag a status unreachable from the initial state under the declared
  graph, a structurally dead state. That catches a hand-edit into an orphan and
  legend/graph drift. Not a path check. The docs say so plainly.

Mode gate: the config enforces transitions exactly when at least one entry declares `to`. In that
mode a status with an empty `to` is a strict sink (nothing out but self-restamp),
so terminal states are dead-ends unless they declare a reopen edge.

Cross-cutting semantics:

- Self-restamp (`x → x`, date/note update) is always allowed, never gated.
- Unset status is exempt (the graph governs status-to-status only). `add`'s
  initial stamp and a first flip on a status-less task pass.
- Terminal reopen is illegal unless the terminal state declares a `to` edge.
- `mt flip --force` bypasses the check (corrections escape hatch).
- Absent `to` on every entry = today's any-to-any, backward compatible.
- Well-formedness: every `to` endpoint must be a legend value, checked in
  `Config::validate` (hard error, alongside band coverage).

== Scope

- Config: `to: Vec<String>` on `LegendEntry`, plus `Legend` methods for
  declared/allowed/reachable. `Config::validate` rejects a `to` naming a
  non-legend value.
- `mt flip`: `--force` flag. Reject an illegal transition with a message listing
  the allowed next states.
- `mt check`: flag a task whose current status is unreachable under the declared
  graph.
- Dogfood: declare a `to` graph in `.mindtape/config.toml`. `mt check` stays
  green.
- Tests: legal/illegal flips, the unconstrained default, a declared reopen edge,
  self-restamp, unset exemption, `--force`, orphan-state detection, and the
  well-formedness rejection.
