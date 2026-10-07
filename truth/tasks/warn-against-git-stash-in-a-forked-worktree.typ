#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Warn against git stash in a forked worktree",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 1,
  estimate: (src-files: 2),
  tags: ("found", "tooling"),
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
      sound-codebase: helps[parallel forks cannot pick up each other's half-done work],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 7)[one sentence in Worktrees],
)

== Problem

+ Three agents rewrote docs in parallel forked worktrees
+ 🔴 #raw(".just/scripts/fork.sh"): git keeps one stash list for every worktree of a repo, so git stash pop in one fork can apply another fork's entry

```
git stash pop in voice-contributor applied "On voice-site-guide: chunkB" (aa4a00dd) instead of its own entry
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("docs/src/contributing.typ").

== TO-BE

- 🟢 The Worktrees section in CONTRIBUTING.md says not to use `git stash` in a fork, and to make a WIP commit instead.

== Changes

- `docs/src/contributing.typ`: one sentence in Worktrees.
- `CONTRIBUTING.md`: re-rendered by `just docs md`.

== Rollback

- Revert both files.

== Out of scope

- A hook that refuses `git stash`. A sentence is enough until it recurs.

== Dogfood

Every parallel fork in this repo stashes nothing.
