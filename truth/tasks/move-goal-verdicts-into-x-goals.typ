#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Move goal verdicts into x-goals",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("mt",),
  estimate: (files: 15),
  assignee: "agent",
  status: wip(2026, 9, 16),
  links: depends-on("check-a-task-field-with-a-typst-function.typ"),
  x-goals: (
    product: (
      readable-tasks: helps[the body runs from Problem straight to TO-BE, and the verdicts sit beside status and links],
      implementable-tasks: neutral,
      enforced-practice: helps[a skipped goal becomes a named `rules.goals` error that `mt add` refuses, not an eval panic],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
)

== Problem

+ Open `truth/tasks/give-every-body-rule-a-name.typ`.
+ 🔴 Its goal verdicts fill a 17-line `== Goals` section between Problem and TO-BE, so a reader skimming from what is broken to how it should work crosses a wall of `neutral`.
+ Run `mt eval truth/tasks/give-every-body-rule-a-name.typ --json`.
+ 🔴 No verdict is in the output, so no script can list the tasks that help `cheap-adoption`.


== TO-BE

- 🟢 Every open task and template carries its verdicts as `x-goals: (product: (...), process: (...))` in `task.with`, and no `== Goals` section.
- 🟢 `.mindtape/config.toml` checks them with one `[[tasks.rules]]` entry naming `check-goals` in `truth/lib.typ`, at `error` severity on non-terminal tasks.
- 🟢 `mt add` refuses a task that skips a goal, names an unknown one, gives `helps` or `hurts` no reason, or helps none, and the message names the goal.


== Changes

- `truth/lib.typ`: `goals(...)` becomes `check-goals(value)`, returning one string per problem instead of panicking; `helps`, `hurts` and `neutral` stay.
- `.mindtape/config.toml`: the `goals` rule; drop the Goals header, order and content rules.
- `.mindtape/templates/*.typ`: the goal slots move into `x-goals`; re-record `task-templates` and `sweep-templates` with `just outdatty-update`.
- `truth/tasks/*.typ`, every open task: move its verdicts, in a `task:` commit.
- `crates/mindtape-cli/tests/add_config_scaffold_e2e.rs`: the goals stub drops `goals`.
- `docs/src/agents.typ`: the repo map line; `just docs md`.


== Rollback

`git revert` the commits: the section and its rules come back whole, and nothing outside the repo changed.


== Out of scope

- Closed tasks: they carry no verdicts, and the rule skips terminal tasks.
- New goals or rewording the current ones.


== Dogfood

Built in: every open task in this repo carries `x-goals`.

