#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Give each project one prelude module",
  kind: "decision",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("mt", "tooling", "stateless"),
  estimate: (src-files: 0),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[every task file opens with one import line instead of two, and the project's own helpers sit beside the library],
        cheap-adoption: helps[a project copies one prelude file and pins the library once],
      ),
      ..(
        "implementable-tasks",
        "enforced-practice",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[a prelude version bump has one site to change],
        docs-match-code: helps[the import a task shows is the one the docs show],
      ),
      ..(),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 8),
)

== AS-IS

+ Open any template file in this repo. It imports `@local/mindtape:0.4.0` and `../lib.typ`; a nested folder needs a different `../` depth.
+ 🔴 The library version repeats in every template, and a library bump rewrites each one, while a helper such as `surfaced-by(...)` has no task-facing module of its own.
+ Old tasks keep their literal pins, so the tree already carries three DSL versions.

```
$ head -2 .mindtape/templates/task.typ
#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *
```

== TO-BE

- 🟢 A numbered generation under `truth/prelude/<n>/lib.typ` is the task-facing module: it pins one `@local/mindtape:<version>` and owns the helpers that generation exposes.
- 🟢 Tasks and templates import it root-absolutely, so the path does not depend on folder depth.
- 🟢 A compatible change edits the newest generation in place; an incompatible change adds the next generation and rewires the templates.
- 🟢 Old tasks keep the literal pin they were written against; the repo never migrates history just to follow a bump.

```typ
// truth/prelude/1/lib.typ
#import "@local/mindtape:0.4.0": *
#let surfaced-by(..args) = edge("surfaced-by", ..args)

// a task
#import "/truth/prelude/1/lib.typ": *
```

== Changes

- `truth/tasks/give-each-project-one-prelude-module.typ`: records this decision; no code changes.

== Alternatives

- One prelude for every task: a library bump moves closed tasks to the new DSL too, and the generation number is the compatibility record.
- `@project/prelude:2`: Typst requires `2.0.0`, a project pin is `mt`-only, and commit states hardcode the `local` namespace.
- A relative `../prelude.typ` for every task: a nested folder changes the `../` depth.

== Out of scope

- Old task files. Each keeps the literal pin it was written against; a generation does not migrate history.
- Rust, schema, config and `mt fix`: a prelude is a project convention, not an `mt` concept.
- Product docs and scaffolding for preludes; another project may choose a different layout.

== Dogfood

The next task and template this repo files import `truth/prelude/1/lib.typ`.
