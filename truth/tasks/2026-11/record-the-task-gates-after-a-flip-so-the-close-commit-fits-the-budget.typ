#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Record the task gates after a flip so the close commit fits the budget",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 2,
  links: (
    (
      task: "give-a-failed-hook-action-advice-that-matches-the-write.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "gating", "tooling", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: helps[the close path costs one commit, not two],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 21),
)

== AS-IS

+ Committing a close after `just check`, `mt wip`, the work and `mt done`
+ 🔴 #raw(".githooks/pre-commit"): the documented close order leaves the two untagged task arms stale. The closing commit then overruns the 10 s budget.

```
mt done writes truth/tasks/2026-11/<stem>.typ
  stale  just mt-check       (8.6 s)
  stale  just docs::vale-tasks (3.8 s)
$ git commit
pre-commit: the hook passed but took 12.8s, over its 10.0s budget.
```

The second attempt passed, because the first attempt had recorded both arms. Five of the thirteen workers in the milestone run hit this.

== TO-BE

- 🟢 The documented order leaves a closing commit inside the budget, with no retry.
- 🟢 `mt flip` into a terminal status refreshes the two task arms, since it is the write that staled them.
- 🟢 A retry is never the documented fix.

```
$ mt done <stem> && git commit
[mindtape <sha>] feat(cli): ...
```

== Changes

- `crates/mindtape-cli/src/cmd/flip/`: after the status write, run the project's `[[on.flip]]` actions and then the arms its own write staled, or record them.
- `.githooks/pre-commit`: report the two task arms by name when the flip staled them.
- `docs/src/contributing/gates.typ`: the close order, and what refreshes the task arms.
- `CHANGELOG.md`: an entry under `[Unreleased]` `Fixed`.

== Out of scope

- The budget itself. It is the guard that makes a slow hook visible, and raising it hides the next regression.
- The two arms' cost. They read the task corpus, which is the work they exist to do.

== Dogfood

Every close in this repo runs the order. Every close pays the retry until this lands.
