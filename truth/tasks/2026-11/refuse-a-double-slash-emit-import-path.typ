#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Refuse a double-slash emit-import path",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  links: (
    (
      task: "rewire-goals-through-typst-markers.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "cli", "tooling", "stateless", "no-ui-ux"),
  assignee: "agent",
  status: proposed(2026, 11, 22),
)

== AS-IS

+ Reviewing the goal DSL milestone after it landed.
+ 🔴 #raw("crates/mindtape/src/config/tasks.rs"): `emit-imports = ["//tmp/x.typ"]` loads because only one slash is stripped, and the emitted path is not the validated one.

`emit_import_problem` strips one `/`, joins the rest under the root, and tests `root.join(relative).is_file()`. A leading `//` makes `relative` start with `/`, so `root.join` is absolute. The emitted `#import "//tmp/x.typ"` is another path.

== TO-BE

- 🟢 The check refuses an entry that resolves outside the project root, including a `//` path.
- 🟢 Validation checks the exact path the emitter writes, not a joined approximation.
- 🟢 A test covers `//tmp/x.typ` and a normal `/truth/prelude/2/lib.typ`.

```toml
[tasks]
emit-imports = ["//tmp/goals.typ"]
```


== Changes

- `crates/mindtape/src/config/tasks.rs`: reject an absolute remainder or a non-normalized path, and compare the same path the emitter builds.
- `crates/mindtape/src/config/tasks_tests.rs`: the two cases.

== Out of scope

- The import shadowing. A separate task.

== Dogfood

A project cannot point the emitter at a file outside its root.

== Goals

+ #Helps(adopter, to: run-checks-at-the-write)[the config refuses a path that escapes the project].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).

