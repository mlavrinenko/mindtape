#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Add the committed state to changed and lines",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("mt",),
  estimate: (files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a budget sees the task's whole footprint, not only uncommitted work],
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
  status: cancelled(
    2026,
    9,
    27,
  )[No real problem behind it. Work here lands in one commit per task, so staged+worktree already sees the whole footprint.],
  links: depends-on(
    "count-changed-files-per-git-state-in-filters.typ",
    "check-commit-messages-against-commit-rules.typ",
  ),
)

== AS-IS

+ A task is `wip` across three commits, each carrying `Refs: <stem>`.
+ 🔴 `changed("staged+worktree")` sees only the uncommitted part. What the task already committed counts nowhere, and a budget resets on every commit.


== TO-BE

- 🟢 `committed` is a third state: paths changed by commits whose `[commits] trailer` points at the task under evaluation. `changed("committed+staged+worktree") <= estimate.files` is the task's whole footprint.
- 🟢 `committed` reads per task. `staged` and `worktree` stay repo-wide.
- 🟢 The command reads history once, cached by `HEAD`.


== Changes

- `crates/mindtape/src/git_state_tests.rs` (red first): commits attributed by trailer, a renamed stem, and a commit that covers two tasks.
- `crates/mindtape/src/git_state.rs`: the committed reader, sharing trailer parsing with `commit_refs.rs`.
- `www/content/filters.typ`: document the state.
- `CHANGELOG.md`: the addition.


== Alternatives

- Dating each commit from the `wip` flip: a date says nothing about which commit belongs to which task.


== Out of scope

- Trailer parsing and renamed stems themselves: owned by the commit-rule and touches tasks.


== Dogfood

This repo's file budget reads `committed+staged+worktree` once `Refs:` trailers are stems.

