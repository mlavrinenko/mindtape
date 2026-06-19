#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "List handles: ordinal plus stable check-letter",
  status: done(2026, 6, 19),
  priority: ice(impact: 8, confidence: 0.8, ease: 9),
  estimate: (hours: 4),
  relates-to: (link("task-identity.typ"), link("list-plaintext-render.typ")),
  tags: ("cli",),
)

== Summary

Replace the long ID column with a short, copyable handle per listed task: an
ordinal (1..N over the displayed list) plus a check character. The check is
derived from the task's stable id, not from the whole set, so it is intrinsic
and reproducible — a task is "the d one" in every view. `mt review 5d` recomputes
the default view, indexes ordinal 5, and compares its check letter to `d`; a
mismatch (the list drifted) errors, naming the current handle and title. Guards
against acting on stale output.

== Scope

- Handle = `<ordinal><check>`, with `check = base26(hash(stable id))`. The check
  length is configurable under `[handle] check_len`, default 1.
- `resolve_task` gains a handle mode beside slug / relative path / absolute path.
  Handles resolve against the default view (config filter + default sort).
- Every mutating command echoes the resolved title.

== Boundary

The handle is ephemeral UI: never stored, never a link key, and excluded from
(or explicitly marked volatile in) json output.
