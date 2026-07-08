#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Import tasks from other trackers",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 4,
  ),
  tags: (),
  status: proposed(2026, 7, 8),
)

== Summary

Give MindTape an on-ramp: `mt import --from <fmt> <path>` reads another tracker's
export and writes native `.typ` task files under a root, mapping the source
fields onto the project config schema. The comparison research shows every
neighbour has an easy on-ramp — dstask ships a Taskwarrior/GitHub importer,
todo.txt is one grep-able line, Taskwarrior has `task export` JSON — while
MindTape starts from an empty backlog with no migration path. This fits the
model rather than bending it: import is file generation down the same path
`mt add` already walks, producing typed, validated tasks.

== Constraints

- Every written file must pass `mt check`. Map source fields onto the config's
  status legend, priority scale, `kind` enum, and tag allow-list; carry anything
  unmapped as a namespaced `x-<ns>` field or drop it with a report — never invent
  or force a core field.
- Keep each source behind a small adapter under one `mt import --from <fmt>`.
  First targets: todo.txt (line format, `(A)` priority, `+project`/`@context`),
  Taskwarrior JSON (`task export`), dstask YAML.
- Deterministic slugs from the source title; dedupe against existing stems;
  surface unmapped fields instead of silently losing them.

== Scope

- Define the importer trait and the field-mapping onto the config schema.
- Adapters: todo.txt, Taskwarrior JSON, dstask YAML.
- `mt import --from <fmt> <path> [--to <subdir>] [--dry-run]`; validate every
  written file before it lands.
- Tests against real fixture exports from each source tracker.
