#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep the dogfood kind test green on an empty backlog",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("mt", "tests"),
  estimate: (files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the gate fails on a defect in the code, never on the backlog being finished],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a full run stays green whatever the backlog holds],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 9, 16),
)

== Problem

+ Close every open task in `truth/tasks/`, so `mt ls '!terminal'` prints no rows.
+ Run `cargo test -p mindtape --test dogfood_check`.
+ 🔴 `dogfood_config_requires_kind_on_open_tasks_only` (`crates/mindtape/tests/dogfood_check.rs:97`) panics on an `Option::unwrap()` of `None`: it takes the first non-terminal task out of the backlog, and a finished backlog holds none.
+ Run `just check` on the same tree.
+ 🔴 It passes, because `truth/tasks/**` sits outside every mmz scope, so closing the last task never busts the test arm. The break waits for the next cold run.


== Environment

Any checkout of this repo, on any platform. It fires whenever the backlog holds no open task and the memoized test arm runs cold.


== TO-BE

- 🟢 The test stamps its own open and closed status onto a loaded task, so it asserts the config's `required` rule whatever the backlog holds.
- 🟢 `cargo test -p mindtape --test dogfood_check` passes with every task closed.


== Changes

- `crates/mindtape/tests/dogfood_check.rs`: build the open/closed pair from `config.tasks.status.initial()` and `terminal()` instead of searching `outcome.tasks`. The red run is today's panic, reproduced by the Problem steps.


== Out of scope

- The other dogfood tests: they read `tasks.first()`, and a backlog with no task at all is not a state this repo reaches.
- The mmz scope: `truth/tasks/**` stays out of it so the close gate survives its own writes.


== Dogfood

Built in: this repo's own gate is what broke when its backlog emptied.

