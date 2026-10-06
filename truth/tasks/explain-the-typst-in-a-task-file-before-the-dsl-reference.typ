#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Explain the Typst in a task file before the DSL reference",
  kind: "docs",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[hand-editing a task needs no Typst course first],
      typst-over-markdown: helps[a reader who has never written Typst can read and write a task file, so the Typst pitch is not lost on the first file],
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[every invalid example on the page is checked by the build and shows the error mt prints today],
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 6),
)

== Problem

+ Open the Quickstart and read the first task file it writes, `tasks/fix-the-login-bug.typ`.
+ 🔴 Its two code lines, `#import "@local/mindtape:0.2.0": *` and `#show: task.with(...)`, are explained on no page, and `rg -n 'typst.app' www/content` prints nothing: no page links the Typst documentation.
+ 🔴 The syntax the fields use (named arguments, the dictionary `(days: 1)`, the one-item array `("backend",)`, the content block `[...]`, links joined with `+`) shows up only in passing on the DSL page, which is a field reference.

Every task is a Typst file, so a reader who does not know Typst stalls at the first file they open by hand.


== TO-BE

- 🟢 A new page, before the DSL reference in the Tasks group, walks one annotated task file: what the import does and why `@local` resolves after `mt install`, what the show rule does, and that everything below it is the body in Typst markup.
- 🟢 Each syntax trap a field meets is a valid and an invalid pair, and the invalid one shows the error `mt check` prints for it, captured in the build.
- 🟢 The page links the Typst syntax and scripting docs for everything a task file does not use.
- 🟢 The Quickstart links the page where it first shows a task file, and the DSL page links it from its intro.


== Source of truth

- `lib/prelude.typ`: the fields and helpers the examples call.
- `mt check` output on the invalid examples, captured by `www/capture.sh`.
- The Typst reference at typst.app/docs for the links.


== Changes

- `www/content/task-files.typ`, new: the annotated file and the pairs.
- `examples/task-files/`, new: the fixture holding the valid and invalid task files the page checks.
- `www/generate-task-files.sh`, new, called from `www/generate.sh`: the captures.
- `www/utils/site.typ`: the page in `NAV`, first in Tasks.
- `www/content/quickstart.typ`, `www/content/dsl.typ`: link the page.


== Out of scope

- Teaching Typst beyond what a task file uses. The page links the Typst docs for the rest.
- The DSL page's field table, which stays the reference.


== Dogfood

An agent hand-editing a task under `truth/tasks/` reads this page for the syntax instead of guessing it from other tasks.

