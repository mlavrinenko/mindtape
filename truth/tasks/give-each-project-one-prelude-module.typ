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
  assignee: "max",
  status: proposed(2026, 11, 4),
)

== AS-IS

+ Open any task file in this repo. It imports `@local/mindtape:0.3.0` and `../lib.typ`; a task in a nested folder needs a different `../` depth.
+ 🔴 The version string repeats in 908 tasks and 16 templates. A prelude bump rewrites every file, and a project helper such as `surfaced-by(...)` has no single module of its own.
+ A template or task that needs a shared helper copies an import line or reaches into `truth/lib.typ`, whose path depends on the task's folder.

```
$ head -2 truth/tasks/list-a-flip-gate-with-no-message-in-mt-rules-and-mt-tags.typ
#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *
```

== TO-BE

- 🟢 Every task and template imports one project module, so the library and the project's helpers arrive together.
- 🟢 `prelude.typ` re-exports the `@local/mindtape` prelude and every project helper, so `surfaced-by(...)` and `check-goals(...)` share one import.
- 🟢 The library version appears once, in the prelude, so a bump edits one file.
- 🟢 A flat task root imports `../prelude.typ`; a nested folder or a second root imports the prelude through a `[[typst.local-packages]]` package spec, so the path does not depend on depth.

```
# prelude.typ
#import "@local/mindtape:0.3.0": *
#import "lib.typ": *

// A task file
#import "../prelude.typ": *
```

== Changes

- `truth/tasks/give-each-project-one-prelude-module.typ`: records this decision; no code changes.

== Alternatives

- Keep the two imports in every file: 908 tasks repeat the version and the helper path.
- A pack ships the helper: a pack declares config and templates, but its files are not importable by tasks, and a git pack lives under a cache hash.
- A relative `../prelude.typ` for every task: a nested folder changes the `../` depth, so the import is not one line.

== Out of scope

- The migration of 908 tasks and 16 templates. A later task rewrites them once the prelude exists.
- `mt fix`: it rewrites `@local/mindtape` imports today, and a prelude changes the file it targets. A later task updates it.

== Dogfood

This repo's next task and template import one prelude, and a prelude bump changes one line instead of 900.
