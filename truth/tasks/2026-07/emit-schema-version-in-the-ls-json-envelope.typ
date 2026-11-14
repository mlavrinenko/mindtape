#import "@local/mindtape:0.3.0": *

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

The canonical schema documents a `schema_version` marker that the binary never serializes. Emit it in the `mt ls --json` envelope and have mt import read it. The documented contract then becomes real rather than aspirational.

== Scope

- Add `schema_version` to the top-level `mt ls --json` envelope alongside tasks and applied
- mt import accepts it when present, tolerates absence for bare arrays, and rejects a future higher version with a clear message
- Cover both directions with tests
