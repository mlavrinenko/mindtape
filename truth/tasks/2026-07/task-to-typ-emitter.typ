#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Task-to-typ emitter primitive",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  tags: ("cli",),
  links: parent("import-tasks-from-other-trackers.typ"),
  status: done(2026, 7, 12),
)

== Summary

The one new primitive import needs: serialize a `Task` to `#show:
task.with(...)` Typst source that the loader round-trips and `mt check` accepts.
The existing commands stop short of this. `mt add` fills a template's slots, and
`mt recur` edits an existing `task.with(...)` call in place. Neither serializes
an arbitrary `Task` from scratch. Pure function in the domain lib, with no `IO`
and no config policy (the caller has already mapped values onto the config).

== Design / contract

- New module `crates/mindtape/src/emit.rs`, exported as
  `mindtape::emit_task(&Task) -> String`. It is pure, with a deterministic field
  order.
- Field-by-field emission (skip absent optional fields):
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
    custom kinds via `edge("kind", "t")[note]`. Join kinds with `+`.
  - `tags` -> tuple `("a", "b")` (authored only, never `derived_tags`).
  - `due` / `start` / `reviewed` -> `on(y, m, d)`.
  - `recur` -> the `every(...)` / `rrule(...)` shape if present (the
    implementation may wait if no source needs it in v1. Note the decision).
  - `extras` (`x-<ns>`) -> emit each as `x-ns: <literal>`, with `HVal` -> Typst
    literal (string/int/float/bool/array/dict).
- `body`: append the task's `body` field verbatim after the marker. The emitter
  is a pure serializer: it does not synthesize a `Summary`. Guaranteeing a
  non-empty body is the mapper's job (`canonical-json-to-task-mapping`). If
  `body` is empty, the marker ends the file.

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
  `Task`), so it can land in parallel with `canonical-task-json-schema`.
- Not in scope: JSON parsing, config mapping, the CLI, `Summary` synthesis.
