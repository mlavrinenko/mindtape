#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "A link() target in an edge helper silently yields zero edges",
  priority: ice(
    impact: 8,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("substrate",),
  estimate: (files: 2),
  status: proposed(2026, 7, 31),
)

== Summary

The edge helpers classify every positional by type: \`content\` becomes the note, everything else becomes a target. \`link("a.typ")\` returns content, so \`depends-on(link("a.typ"))\` produces a note and no target at all. The task loads clean, \`mt check\` passes, and the dependency is simply gone.

== Reproduction

Confirmed 2026-07-31 in this repository. A task authored with

```
links: depends-on(link("add-mt-check-quiet.typ")),
```

evaluates to `"links": []` under `mt eval --json`, and `mt check` reports it
valid. `mt ls 'blocked'` therefore reports the task as ready.

The prelude documents plain strings as the authoring surface and says helpers
take "never `link(...)`", so this is user error — but it is user error that
loses data in silence. speconaut used to catch the adjacent mistake with a
`Dependency links carry no label` rule; that rule was retired on 2026-07-31
because its premise (`depends-on:` as a task slot) no longer exists, which
leaves this trap with nothing in front of it.

An assertion naming the offending helper would be enough.

== Scope

- the \`edge\` helper in the prelude
- out of scope: accepting \`link()\` as a target; plain strings are the right authoring surface
