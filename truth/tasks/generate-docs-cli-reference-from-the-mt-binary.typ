#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Generate docs CLI reference from the mt binary",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 4,
  ),
  tags: (),
  relates-to: (link("render-task-files-as-typeset-documents.typ"),),
  status: proposed(2026, 7, 6),
)

== Summary

Use tola's external-input capability to render the docs CLI reference, help text, version, and sample output from the real mt binary at build time, so docs cannot drift. Deferred from the initial www rebuild (v1 is hand-written); settle structure first, then automate.

== Scope

- TODO
