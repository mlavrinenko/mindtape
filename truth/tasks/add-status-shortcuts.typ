#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Render the status shortcut form in `mindtape add`",
  status: done(2026, 6, 23),
  priority: ice(impact: 4, confidence: 0.9, ease: 9),
  relates-to: (
    link("add-template-opt-title-arg.typ"),
  ),
  tags: ("cli",),
)

== Summary

`add` injects the initial status verbosely: `status: status("proposed", 2026,
6, 22)`. The prelude ships ergonomic shortcut constructors (`proposed(y, m, d)`,
`wip(...)`, ...) and the dogfood backlog is hand-authored with them, so a filed
task reads differently from one a human wrote. Emit the shortcut form when the
status name has one, matching the authored style.

== Scope

- When the initial/overridden status name is one the prelude exposes as a
  shortcut, render `name(y, m, d)`; otherwise keep `status("name", y, m, d)` for
  a project's custom status names.
- Cover every `_mk(...)` binding in `lib/prelude.typ`: draft, proposed,
  accepted, todo, wip, doing, blocked, in-review, done, cancelled, failed —
  including the hyphenated `in-review`.

== Notes

- Touches `crates/mindtape-cli/src/add.rs` (`inject_header`) only; `mindtape`
  stays format-blind.
- The shortcut name list mirrors the prelude; a drift-guard test parses the
  `_mk(...)` bindings and asserts the const stays in sync, so adding a prelude
  shortcut without updating the list fails the suite.
