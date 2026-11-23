#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Retire the commit state's dead symlinked-task path",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 4,
  links: (
    (
      task: "include-imported-modules-in-the-commit-state-reuse-key.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "tooling", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 24),
)

== AS-IS

+ Folding a task's imports into the commit-state reuse key
+ 🔴 #raw("crates/mindtape/src/commit/state_loader.rs"): the state loader's symlink resolution never runs, because `StateTree::task_entries` drops every `120000` entry before `build_tasks` can see one

```
$ rg -n 'entry.mode == "120000"' crates/mindtape/src/commit
crates/mindtape/src/commit/state_loader.rs:336:            let resolved = if entry.mode == "120000" {
crates/mindtape/src/commit/tree.rs:158:                if entry.mode == "120000" {
```
`task_entries` continues on that mode, so `resolved` is always `None`, `links` is always empty, and `evaluate_links` plus `TaskSource::Missing`/`Unsupported` are unreachable:
```
$ cargo llvm-cov report --show-missing-lines target/coverage/lcov.info | rg 'state_loader.rs.*3[0-9][0-9]'
```
Counted while fixing the reuse key: five lines of `build_tasks`/`evaluate_links` stay unreached by the whole suite.

== TO-BE

- 🟢 One path loads a state's task files: no `links` vector, no `evaluate_links`, and no `TaskSource` variant that no file can produce.
- 🟢 The state loader's docs say what a symlinked task file does today: it is not a source, as `a_state_ignores_a_symlinked_task_file` already asserts.
- 🟢 The suite covers every line of the loader that a state can reach.

```
$ rg -n 'evaluate_links|TaskSource' crates/mindtape/src/commit/state_loader.rs
28:use crate::commit::tree::{StateConfig, StateTree, TaskEntry, TaskSource};
72:    links: Vec<(TaskEntry, TaskSource)>,
315:        seen.extend(self.evaluate_links(links, config, packages, &mut parts));
375:    fn evaluate_links(
```

== Changes

- `crates/mindtape/src/commit/state_loader.rs`: drop `resolved`, `links`, `evaluate_links` and `Parts` from the build path.
- `crates/mindtape/src/commit/tree.rs`: drop `task_source`, `entry_at` and `TaskSource`, or keep them behind the one caller that needs them.
- `crates/mindtape/src/commit/tree_tests.rs`: keep the symlink cases that still apply to `task_entries`.

== Out of scope

- Making a symlinked task file a source again. That reverses `a_state_ignores_a_symlinked_task_file` and needs its own decision.

== Dogfood

The state loader loses a fallback nothing reaches, and the reuse key's data flow stops routing around it.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[because a test can cover every line a state reaches].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
