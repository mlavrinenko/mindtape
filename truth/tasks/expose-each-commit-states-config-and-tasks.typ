#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Expose each commit state's config and tasks",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.85,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("mt", "gating", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[a commit rule classifies against the state it is checking],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[the state config and task set stop being dropped between loading and checking],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 9),
)

== AS-IS

+ In a project with a commit rule but no task-scope rule, run `mt check --commit-msg`.
+ 🔴 The index state is never loaded (`task_states` returns `None`), and `CommitRecord::build` receives the worktree config and tasks.


== TO-BE

- 🟢 `ProjectState` carries its `Config` and task set; `StateLoader` retains the state config instead of dropping it.
- 🟢 `CommitStates` exposes the head, staged and worktree configs, and the staged task set.
- 🟢 `task_states` loads the staged state even when the project has no task-scope rules.
- 🟢 A state config that fails to load keeps its error and classification falls back to the working config, as today.

```
let states = task_states(project)?;
let staged = states.staged; // config + tasks
CommitRecord::build(&message, &changed, &staged.tasks, &staged.config)
```


== Changes

- `crates/mindtape/src/commit/tasks.rs`: carry the config and task set on `ProjectState`.
- `crates/mindtape/src/commit/state_loader.rs`: retain the state config.
- `crates/mindtape-cli/src/cmd/check/commit.rs`, `cmd/check/range.rs`: build the record from the state.
- `crates/mindtape/src/commit/tests.rs`: the red tests first, including no-task-rules and a config load failure.


== Out of scope

- Changing classification rules or the commit tree's task list.
- Validating historical state configs.


== Dogfood

This repo's commit rules already run against `truth/tasks`; the change makes the state config available to them instead of rebuilding it from the worktree.

