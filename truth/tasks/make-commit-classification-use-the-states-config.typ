#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Make commit classification use the state's config",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt", "gating", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[a template-only commit can no longer evade the `Refs:` trailer],
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
        sound-codebase: helps[the config that classifies a commit is the config that commit carries],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  links: depends-on(
    "define-and-export-one-task-source-policy.typ",
    "expose-each-commit-states-config-and-tasks.typ",
  ),
  status: proposed(2026, 11, 9),
)

== AS-IS

+ In a root-`.` project with `templates.dirs = ["templates"]`, edit `.mindtape/config.toml` in the worktree to point elsewhere while the index still names `templates/`, stage a change to `templates/task.typ`, and write a message with no `Refs:`.
+ 🔴 `mt check --commit-msg` uses the worktree config, classifies the template as a task source, and passes without a `Refs:` trailer:

```
$ mt check --commit-msg .git/COMMIT_EDITMSG
$ echo $?
0
```


== Environment

`mt 0.5.0` on Linux, in a root-`.` project whose index and worktree disagree about `[templates] dirs`.


== TO-BE

- 🟢 `--commit-msg` classifies with the index state's config and staged tasks; `--commits` uses each commit's own.
- 🟢 `task_entries`, `is_task_file`, `tasks_only` and path-form `Refs:` call the exported policy; a `120000` symlink entry is not a source.
- 🟢 A path that names no task contributes no stem to `refs` or `tasks`.
- 🟢 Classification is per path: a tracked-deleted file is a commit-state task, a root outside the project is working-tree only.
- 🟢 A template-only commit is not task-only and needs a `Refs:` trailer.

```
$ mt check --commit-msg .git/COMMIT_EDITMSG
error: name the task this commit works on in a `Refs: <stem>` footer
```


== Changes

- `crates/mindtape/src/commit/tree.rs`: filter `task_entries` and symlink entries with the shared policy.
- `crates/mindtape/src/commit/mod.rs`: use it in `is_task_file`, `tasks_only` and the ref accumulator.
- `crates/mindtape/src/commit/tests.rs`, `crates/mindtape-cli/tests/it/commit_msg_task_e2e.rs`, `crates/mindtape-cli/tests/it/commit_range_e2e.rs`: the red tests first, including worktree-vs-index config, a staged-only task, a tracked-deleted task and a rename across a template dir.


== Out of scope

- The history `touches()` rename map, which keeps the working config.
- Making `tasks_only` mean "loads a task": it stays path-based, so a markerless `.typ` under a root still counts.
- Historical configs whose roots are now reserved.


== Dogfood

This project's task root is `truth/tasks`, so template and metadata paths never count as tasks today; the rule keeps that true for a root-`.` adopter.

