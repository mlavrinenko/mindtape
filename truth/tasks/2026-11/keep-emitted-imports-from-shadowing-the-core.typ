#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Keep emitted imports from shadowing the core",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  links: (
    (
      task: "rewire-goals-through-typst-markers.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "cli", "tooling", "stateless", "no-ui-ux"),
  assignee: "agent",
  status: done(2026, 11, 23),
)

== AS-IS

+ Reviewing the goal DSL milestone after it landed.
+ 🔴 #raw("crates/mindtape/src/emit/mod.rs"): `emit-imports` writes a wildcard import after the prelude, so a project module exporting `task` or `status` silently shadows the core.

`emit_task_with` writes `#import "...": *` for every configured module and `validate_emit_imports` checks only that the path is a file. A helper that exports `task` wins over the prelude.

== TO-BE

- 🟢 A rebuilt task cannot silently lose a core helper to a project wildcard.
- 🟢 The emitter imports the names the body needs and nothing more, or the config refuses a module whose exports collide with the core.
- 🟢 A test covers a project module that exports `task`.

```typ
#let task(..) = [shadow]
```


== Changes

- `crates/mindtape/src/emit/mod.rs`: choose named imports or a collision-safe order.
- `crates/mindtape/src/config/tasks.rs`, `crates/mindtape/src/emit/tests.rs`: the refusal or the named-import test.

== Out of scope

- The goal DSL move. This is the generic emitter.

== Dogfood

A generated task keeps the prelude's `task` and `status` even when a project module exports the same name.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[a rebuilt task cannot silently change its core helper].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).

