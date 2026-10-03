#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add a tweak template that files a small change done with its work",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: (),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a one-line change still names a task, at the cost of one `mt add` line],
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
  status: proposed(2026, 10, 3),
)

== Problem

+ Make a one-line config change, which needs a `Refs:` task.
+ Run `mt add --template dev --describe text`.
+ 🔴 It lists six required body slots, from `problem` to `dogfood`, and the task takes its own `task:` commit before the work.


== TO-BE

- 🟢 `mt add -t tweak --status done` files a task with a title, a kind and one line of why, in the same commit as the work.
- 🟢 An open tweak, or one whose commit changes more than one source file, is refused.


== Changes

- `.mindtape/templates/tweak.typ`: the template.
- `.mindtape/config.toml`: the `tweak` tag, a rule that a tweak lands done, and a commit rule capping its source files.
- `docs/src/contributing.typ`: when a tweak fits, and its one-commit flow.


== Rollback

Revert the commit; small changes file a `dev` task again.


== Out of scope

- The `mindtape-tasks` pack: the template is this repo's practice.


== Dogfood

Config wording, a gate flag or a typo fix here lands as a tweak.

