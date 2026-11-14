#import "@local/mindtape:0.3.0": *

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

lib/prelude.typ declares the task show-rule as \#let task(body, ..meta), which hides every core field from tooling. tinymist doesn't offer completion, signature help, or arity and spelling feedback while authoring a task. The fields are not open-ended: \_core-slots already enumerates them and the marker builder reads all thirteen by name. Typed authoring with editor support is a main reason to keep tasks in Typst rather than YAML frontmatter. Today the DSL gives up that advantage at exactly the point a user notices it.

== Scope

- Give task an explicit named parameter per core field, with its default: title, status, priority, difficulty, kind, estimate, links, tags, due, start, reviewed, recur, id. Keep ..meta only for the x-\<ns> extension slots.
- Preserve the current assertion behaviour: unknown non-x- keys still fail with the same message.
- Verify tinymist actually surfaces the completions and signature help afterwards. Record what it shows.
- Check the docs examples and templates still evaluate unchanged.

== Verification

Driven against tinymist 0.14.18 (Typst 0.14.2) over stdio LSP. The probe file
holds `\#show: task.with(` with the cursor in the argument list, with an
isolated `XDG_DATA_HOME` where the prelude resolves as `@local/mindtape:0.3.0`.

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
none of them a core field, only ambient global scope (`aqua`, `calc`, `mon`,
the `_`-prefixed prelude internals). After: the same globals plus the thirteen
field names as named-argument items (`difficulty`, `due`, `estimate`, `id`,
`kind`, `links`, `priority`, `recur`, `reviewed`, `start`, `status`, `tags`,
`title`). On a partial `ti` the list narrows to `estimate` and `title`. Before, it
didn't offer a field at all.

Note the field defaults double as the type hint tinymist shows (`estimate:
dictionary`, `links: array`, `tags: array`). The `none`-defaulted fields carry
no type, which is the cost of a default that says "unset".
