#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Stop a waits checker that runs mt from spawning itself without end",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a checker that calls mt back answers once instead of forking without end],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 28),
)

== Problem

+ Writing `argv_run_e2e`, whose checker first ran `mt dicts {id}` against its own project
+ 🔴 #raw("crates/mindtape/src/waits.rs"): a checker whose command loads the same project runs every checker again in the child, which spawns the next child, with no bound

```
[> 60.000s] (─────────) mindtape-cli::it argv_run_e2e::an_array_checker_fills_its_id_into_one_argument
[>360.000s] (─────────) mindtape-cli::it argv_run_e2e::an_array_checker_fills_its_id_into_one_argument
     SIGTERM [ 369.707s] (4/4) mindtape-cli::it argv_run_e2e::an_array_checker_fills_its_id_into_one_argument
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/waits.rs").

== TO-BE

- 🟢 A checker that runs `mt` against its own project gets one answer, and the nested `mt` runs no checker of its own.


== Changes

- `crates/mindtape/src/waits_tests.rs`: the red test, a checker that calls back into the loader runs once.
- `crates/mindtape/src/waits.rs`: mark the checker's environment, and run no checker under that mark.


== Rollback

- Revert the `waits.rs` change.


== Out of scope

- `[[on.flip]]` commands. A read never runs them, so a gate running `mt ls` can't recurse.


== Dogfood

Not applicable. This repo declares no `[[tasks.waits]]` checker.

