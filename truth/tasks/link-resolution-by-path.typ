#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Resolve task links by file path, not id-stem",
  status: done(2026, 6, 19),
  priority: 5,
  estimate: (hours: 3),
  depends-on: (link("task-identity.typ"),),
  tags: ("substrate",),
)

== Summary

`depends-on` / `relates-to` / `supersedes` / `duplicates` use `link("other.typ")`
— real file paths, Typst-checked and clickable in an IDE. Keep that syntax. But
`validate.rs` resolves link targets against the set of task ids, which only works
because today id == stem == filename. Once identity becomes explicit or a UUID
(task-identity), link resolution must match by file path, independent of id.
Decouple the two layers.

== Scope

- Resolve link targets to tasks by their file path (relative to the project
  root), not by id.
- Keep the `link()` DSL unchanged; Typst and IDE behaviour unaffected.
- Broken-link validation reports the unresolved path.
