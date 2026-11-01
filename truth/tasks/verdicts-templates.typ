#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Use `verdicts` in the task templates",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt", "stateless"),
  estimate: (src-files: 14),
  x-goals: (
    product: (
      readable-tasks: helps[a task file lists each goal once],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 1),
  links: depends-on("prelude-verdicts.typ", "task-templates.typ"),
)


== Problem

+ Run `mt add --describe -t dev` and count the eight per-goal slots.
+ 🔴 The template writes eight verdict lines per task, so every filed task carries the long form.
+ 🔴 The short `verdicts` form is available but no template shows it.

== TO-BE

- 🟢 Each template with `x-goals` has one `*-goals` slot and one `*-neutral` slot per group.
- 🟢 The example uses the complete `verdicts` call and the neutral default lists the rest.
- 🟢 `mt check` fills the example and passes.

```typ
x-goals: (
  product: verdicts(
    (enforced-practice: helps[the rule names its waiver],),
    "readable-tasks", "implementable-tasks", "cheap-adoption", "typst-over-markdown", "plain-files",
  ),
  process: verdicts((), "sound-codebase", "docs-match-code"),
)
```

== Changes

- `.mindtape/templates/*.typ`: the goal slots and the `verdicts` example in every template with `x-goals`.

== Out of scope

- The prelude helper in `prelude-verdicts`, the docs in `verdicts-docs` and the corpus in `x-goals-sweep`.

== Dogfood

`mt add` files this repo's next task with the short form.
