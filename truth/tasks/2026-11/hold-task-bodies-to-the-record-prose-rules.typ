#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Hold task bodies to the record prose rules",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("docs", "gating", "tooling", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[A task bullet reads under the same rules as a docs page.],
      implementable-tasks: neutral,
      enforced-practice: helps[The gate holds a prose rule that memory cannot skip.],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: cancelled(
    2026,
    11,
    14,
  )[superseded by the widened gate from 2026-12, which holds new tasks and leaves the older bodies exempt.],
)

== AS-IS

+ Copy `.config/vale.ini` to `/tmp/tasks.ini`, keep only the seven record exceptions, and set `StylesPath` to the repo's absolute `.config/vale/styles`.
+ Run `vale --no-global --config=/tmp/tasks.ini truth/tasks/2026-10 truth/tasks/2026-11`.
+ 🔴 It reports 872 alerts over 390 files, led by `Prose.BareNames` (235), `Prose.NegatedObject` (178) and `Prose.ConsequenceClause` (145).

== TO-BE

- 🟢 The 32 AI-tell lines under `[truth/tasks/*/*.typ]` disappear, and `just docs vale-tasks` passes with the seven record exceptions left.
- 🟢 A task bullet gives its actor, states an effect as its own sentence, and says "set" or "state" instead of "name".
- 🟢 `docs/src/contributing/documentation.typ` drops the 32 rules from the off list for a task body.

```ini
[truth/tasks/*/*.typ]
Prose.History = NO
Prose.ProductName = NO
Prose.HeadingCase = NO
Prose.Spelling = NO
Prose.SentenceLength = NO
Prose.PassiveVoice = NO
Prose.Semicolon = NO
```

== Source of truth

`.config/vale.ini` and the rule files under `.config/vale/styles/Prose/`.

== Changes

- `.config/vale.ini`: drop the 32 AI-tell lines under `[truth/tasks/*/*.typ]` as each rule's alerts clear.
- `truth/tasks/2026-10/*.typ` and `truth/tasks/2026-11/*.typ`: the 872 alerts.
- `docs/src/contributing/documentation.typ`: the Task bodies paragraph, which drops the 32 rules from the off list.

== Out of scope

- The seven record exceptions, which a task record's shape needs.
- `truth/tasks/2026-07` to `2026-09`, which `backfill-the-older-task-months-and-widen-the-vale-task-gate` widens first.
- Code comments and commit messages, which have tasks of their own.

== Dogfood

This repo's `just check` holds every new task to the AI-tell rules too.
