#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

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
  status: proposed(2026, 9, 27),
  links: depends-on(
    "count-changed-files-and-lines-per-git-state-in-filters.typ",
    "check-commit-messages-against-commit-rules.typ",
  ),
)

== Problem

+ A task is `wip` across three commits, each carrying `Refs: <stem>`.
+ 🔴 `changed("staged+worktree")` sees only the uncommitted part; what the task already committed counts nowhere, so a budget resets on every commit.


== TO-BE

- 🟢 `committed` is a third state: paths changed by commits whose `[commits] trailer` names the task being evaluated. `changed("committed+staged+worktree") <= estimate.files` is the task's whole footprint.
- 🟢 `committed` reads per task; `staged` and `worktree` stay repo-wide.
- 🟢 History is read once per command, cached by `HEAD`.


== Changes

- `crates/mindtape/src/git_state_tests.rs` (red first): commits attributed by trailer, a renamed stem, a commit naming two tasks.
- `crates/mindtape/src/git_state.rs`: the committed reader, sharing trailer parsing with `commit_refs.rs`.
- `www/content/filters.typ`: document the state.
- `CHANGELOG.md`: the addition.


== Alternatives

- Commits since the task went `wip` by date: a date says nothing about which commit belongs to which task.


== Out of scope

- Trailer parsing and renamed stems themselves: owned by the commit-rule and touches tasks.


== Dogfood

This repo's file budget reads `committed+staged+worktree` once `Refs:` trailers are stems.

