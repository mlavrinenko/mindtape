#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Move this repo's task flips into work commits",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 3),
  x-goals: (
    product: (
      readable-tasks: helps[git log stops carrying one flip-only commit per start and close],
      implementable-tasks: neutral,
      enforced-practice: helps[the flip-with-work rule holds the convention, not memory],
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
    "evaluate-task-scope-commit-rules.typ",
    "migrate-this-repos-refs-trailers-to-stems.typ",
  ),
)

== Problem

+ `git log --format=%s -300 | rg -c '^task: (start|close)'`.
+ 🔴 108 of 300 commits are flip-only. Starts are committed on main because `just fork` refuses a dirty tree (`.just/scripts/fork.sh:74`); closes land after the work.


== TO-BE

- 🟢 `just fork` runs first; the agent flips `wip` inside its worktree and commits it with the first work commit.
- 🟢 `mt done` rides in the last work commit, named by `Refs:`.
- 🟢 The `mindtape-tasks` pack ships `flip-with-work`, so a flip-only commit is refused.
- 🟢 Filing stays a standalone `task:` commit: a planned task is a deliverable.


== Changes

- `docs/src/contributing.typ`, regenerated `CONTRIBUTING.md`: the fork-first flow.
- `crates/mindtape/src/packs/mindtape-tasks.toml`: `flip-with-work`.
- the skill source under `docs/src/`: when to flip and commit.


== Rollback

Disable `flip-with-work` in `.mindtape/config.toml`; standalone flip commits pass again.


== Out of scope

- Squashing past flip commits: history stays.
- Reading in-flight tasks from main: `mt -w <worktree> ls` already does.


== Dogfood

Every task after this one lands with its flips inside its work commits.

