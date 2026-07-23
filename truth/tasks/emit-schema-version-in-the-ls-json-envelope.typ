#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Emit schema_version in the ls JSON envelope",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("cli",),
  estimate: (files: 4),
  status: done(2026, 7, 23),
)

== Summary

The canonical schema documents a schema\_version marker that the binary never serializes. Emit it in the mt ls --json envelope and have mt import read it, so the documented contract becomes real rather than aspirational.

== Scope

- Add schema\_version to the top-level ls --json envelope alongside tasks and applied
- mt import accepts it when present, tolerates absence for bare arrays, and rejects a future higher version with a clear message
- Cover both directions with tests
