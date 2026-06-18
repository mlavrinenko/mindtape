#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Harvest reads task links via the standard Typst link()",
  status: done(2026, 6, 18)[typst-harvest projects link() dest; custom shim gone],
  priority: "normal",
  estimate: (hours: 1),
  relates-to: (link("converge-speconaut.typ"),),
  tags: ("substrate",),
)

== Summary

`link()` is a standard Typst function (a hyperlink element). The prelude must
not redefine it — a custom `#let link(target) = target` shadowed the builtin and
forced every consumer to depend on the shim. Removing the shim broke
`depends-on` / `relates-to`: the fields then held the builtin's content element,
which harvesting collapsed to its debug repr, so link resolution matched nothing
(`link target ... not found`).

== Decision

Author task↔task edges with the standard `link("stem.typ")`. Teach
`typst-harvest` to project a `LinkElem` with a URL destination to that
destination string in `HVal::convert`. The model keeps reading plain strings
(stem / path / SWHID tombstone); nothing task-specific leaks into the harvester —
projecting a link to its destination is a generic, consumer-blind rule.

== Scope

- `crates/typst-harvest`: `convert` maps `Value::Content` packing a `LinkElem`
  with a `Destination::Url` to `HVal::Str(dest)`; other destinations and exotic
  values keep the repr fallback.
- Regression test: a link element converts to its destination string.
- No prelude `link()` shim; no edits to the task artifacts (already standard).
