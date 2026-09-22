#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Carry recur through the canonical task-JSON round-trip",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("mt", "docs"),
  estimate: (files: 6),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: helps[a backlog survives export and re-import without losing a field the files carry],
    ),
    process: (
      sound-codebase: helps[the one field that breaks the round-trip stops being a documented exception],
      docs-match-code: helps[the prose naming the gap goes away because the gap does],
    ),
  ),
  assignee: "agent",
  status: done(2026, 9, 22),
)

== Problem

+ File a recurring task, then run `mt ls --json` over it.
+ 🔴 The record carries no `recur` key at all. `CanonicalTask` has no field
  for it, so the recurrence never leaves the file.
+ Pipe that JSON back through `mt import`. The new task has no `recur:`
  either: `crates/mindtape-cli/src/import_map.rs` writes none.
+ Two places already apologise for this in prose rather than fixing it:
  `crates/mindtape/src/canonical.rs`'s module doc and
  `www/content/schema.typ`, which tells a user the record has a hole in it.


== TO-BE

- 🟢 `CanonicalTask` carries `recur`, and `mt ls --json` emits it for a
  recurring task and `null` for every other one.
- 🟢 `mt import` reads it back, so `mt ls --json | mt import` reproduces the
  recurrence rather than dropping it.
- 🟢 Both `every(...)` and the `rrule(...)` escape hatch survive the trip.
- 🟢 The prose naming the gap is deleted, not reworded: there is nothing left
  to warn about.


== Changes

- `crates/mindtape/src/canonical.rs` — a `recur` field on `CanonicalTask`,
  the round-trip test first, and the module doc's gap paragraph deleted once
  it passes.
- `crates/mindtape/src/recur.rs` — a serialisable shape for the recurrence if
  the model does not already expose one.
- `crates/mindtape-cli/src/import_map.rs` — stop hardcoding none; map the
  field through.
- `crates/mindtape-cli/tests/` — an end-to-end arm piping `mt ls --json`
  into `mt import` over `examples/recurring` and diffing the result.
- `www/content/schema.typ` — drop the gap sentence and add `recur` to the
  record table.


== Alternatives

- Leave the gap and keep documenting it. Rejected by the goal the schema page
  itself states, which is that the record is the whole task.
- Serialise the recurrence as an opaque string. Rejected because the filter
  language and `mt recur` both read the parsed shape, and a second encoding
  would drift from the first.


== Out of scope

- The recurrence semantics themselves. `mt recur` advances a task exactly as
  it does today; this is a serialisation hole, not a scheduling one.
- The `x-<ns>` extension fields, which already round-trip through `extras`.


== Dogfood

`examples/recurring` is the fixture and the proof: exporting it and importing
the result into a scratch copy must produce the same recurrence, which is a
claim the schema page currently has to deny.

