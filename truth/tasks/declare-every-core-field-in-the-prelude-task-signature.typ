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
  status: done(2026, 7, 29),
)

== Summary

lib/prelude.typ declares the task show-rule as \#let task(body, ..meta), so every core field is invisible to tooling: tinymist offers no completion, no signature help, and no arity or spelling feedback while authoring a task. The fields are not open-ended — \_core-slots already enumerates them and the marker builder reads all thirteen by name. Typed authoring with editor support is a main reason to keep tasks in Typst rather than YAML frontmatter, and today the DSL gives up that advantage at exactly the point a user notices it.

== Scope

- Give task an explicit named parameter per core field (title, status, priority, difficulty, kind, estimate, links, tags, due, start, reviewed, recur, id) with its default, keeping ..meta only for the x-\<ns> extension slots.
- Preserve the current assertion behaviour: unknown non-x- keys still fail with the same message.
- Verify tinymist actually surfaces the completions and signature help afterwards; record what it shows.
- Check the docs examples and templates still evaluate unchanged.

== Verification

Driven against tinymist 0.14.18 (Typst 0.14.2) over stdio LSP, on a probe file
holding `\#show: task.with(` with the cursor in the argument list, with the
prelude installed as `@local/mindtape:0.2.0` under an isolated `XDG_DATA_HOME`.

`textDocument/signatureHelp` before:

```
task(body: any, meta: arguments) -> none
```

after:

```
task(body: any, difficulty: none, due: none, estimate: dictionary, id: none,
     kind: none, links: array, priority: none, recur: none, reviewed: none,
     start: none, status: none, tags: array, title: "", meta: arguments) -> none
```

`textDocument/completion` in an empty argument slot returned 299 items before,
none of them a core field — only ambient global scope (`aqua`, `calc`, `mon`,
the `_`-prefixed prelude internals). After: the same globals plus the thirteen
field names as named-argument items (`difficulty`, `due`, `estimate`, `id`,
`kind`, `links`, `priority`, `recur`, `reviewed`, `start`, `status`, `tags`,
`title`). On a partial `ti` the list narrows to `estimate` and `title`; before,
it offered no field at all.

Note the field defaults double as the type hint tinymist shows (`estimate:
dictionary`, `links: array`, `tags: array`); the `none`-defaulted fields carry
no type, which is the cost of a default that says "unset".
