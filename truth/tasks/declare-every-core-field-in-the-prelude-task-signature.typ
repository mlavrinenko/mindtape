#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Declare every core field in the prelude task signature",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("mt",),
  estimate: (files: 3),
  status: proposed(2026, 7, 28),
)

== Summary

lib/prelude.typ declares the task show-rule as \#let task(body, ..meta), so every core field is invisible to tooling: tinymist offers no completion, no signature help, and no arity or spelling feedback while authoring a task. The fields are not open-ended — \_core-slots already enumerates them and the marker builder reads all thirteen by name. Typed authoring with editor support is a main reason to keep tasks in Typst rather than YAML frontmatter, and today the DSL gives up that advantage at exactly the point a user notices it.

== Scope

- Give task an explicit named parameter per core field (title, status, priority, difficulty, kind, estimate, links, tags, due, start, reviewed, recur, id) with its default, keeping ..meta only for the x-\<ns> extension slots.
- Preserve the current assertion behaviour: unknown non-x- keys still fail with the same message.
- Verify tinymist actually surfaces the completions and signature help afterwards; record what it shows.
- Check the docs examples and templates still evaluate unchanged.
