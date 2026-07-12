#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Task-to-typ emitter primitive",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  tags: ("cli",),
  links: parent("import-tasks-from-other-trackers.typ"),
  status: proposed(2026, 7, 13),
)

== Summary

The one new primitive import needs: serialize a `Task` to `#show:
task.with(...)` Typst source that the loader round-trips and `mt check` accepts.
Nothing does this today — `mt add` fills a template's slots, and `mt recur` edits
an existing `task.with(...)` call in place; neither serializes an arbitrary
`Task` from scratch. Pure function in the domain lib, no IO, no config policy
(the caller has already mapped values onto the config).

== Design / contract

- New module `crates/mindtape/src/emit.rs`, exported as e.g.
  `mindtape::emit_task(&Task) -> String`. Pure; deterministic field order.
- Field-by-field emission (skip absent optionals):
  - `title` -> string literal.
  - `status` -> `name(y, m, d)[note]` (note only when present).
  - `priority` -> scalar `n`, or the framework helper for the built-in set
    (`ice`/`rice`/`wsjf`/`eisenhower`/`moscow`) as `ice(impact: n, ...)`, else the
    generic `framework("name", k: v, ...)`. Drive off `NumValue::Framework
    { name, inputs }`.
  - `difficulty` -> scalar or label.
  - `kind` -> string literal.
  - `estimate` -> unit dict `(days: 1, hours: 2)`.
  - `links` -> group by kind, one helper call per kind with variadic targets and
    optional note: `depends-on("a.typ", "b.typ")`, `related("x.typ")[note]`,
    custom kinds via `edge("kind", "t")[note]`; join kinds with `+`.
  - `tags` -> tuple `("a", "b")` (authored only; never `derived_tags`).
  - `due` / `start` / `reviewed` -> `on(y, m, d)`.
  - `recur` -> the `every(...)` / `rrule(...)` shape if present (may be deferred
    if no source needs it v1 — note the decision).
  - `extras` (`x-<ns>`) -> emit each as `x-ns: <literal>`, HVal -> Typst literal
    (string/int/float/bool/array/dict).
- `body`: append the task's `body` field verbatim after the marker. The emitter
  is a pure serializer — it does NOT synthesize a `Summary`; guaranteeing a
  non-empty body is the mapper's job (`canonical-json-to-task-mapping`). If
  `body` is empty, emit nothing after the marker.

== Files

- `crates/mindtape/src/emit.rs` (+ `emit_tests.rs`), `lib.rs` export.

== Acceptance

- `just check` green.
- Round-trip property test: for a spread of `Task` values, `emit_task` then
  re-load (harvest) yields an equal `Task` (modulo `file`/`derived_tags`). Assert
  the emitted source parses and, given a matching config, passes validation.
- Cover: framework + scalar priority, multi-kind links with notes, estimate
  dict, `x-<ns>` extras, a note-bearing status.
- Commit `Refs: truth/tasks/task-to-typ-emitter.typ`.

== Scope

- The emitter and its tests only. Independent of the JSON schema (works from
  `Task`); can land in parallel with `canonical-task-json-schema`.
- Not in scope: JSON parsing, config mapping, the CLI, `Summary` synthesis.
