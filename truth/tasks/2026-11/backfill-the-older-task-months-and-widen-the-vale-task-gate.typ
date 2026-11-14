#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Widen the Vale task gate to new task months",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs", "gating", "stateless", "tooling"),
  x-goals: (
    product: (
      readable-tasks: helps[a new task bullet reads under the same rules as a docs page],
      implementable-tasks: neutral,
      enforced-practice: helps[a prose rule the gate holds cannot be skipped by memory],
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
  status: proposed(2026, 11, 14),
  links: supersedes("hold-task-bodies-to-the-record-prose-rules.typ"),
)

== AS-IS

+ Run `grep -n '^\[truth/tasks' .config/vale.ini`. It lists `[truth/tasks/*/*.typ]`, the three older-month sections and `swept/`, and no section for a month after `2026-11`.
+ Run `grep -c '^Prose\..* = NO$' .config/vale.ini`. It prints 39, the seven record exceptions and the 32 AI-tell rules, all under `[truth/tasks/*/*.typ]`.
+ 🔴 A task filed in `2026-12` inherits all 32 `NO` lines, so the gate skips them on a new task, and the Task bodies paragraph states no first month for them.

```
$ grep -c '^Prose\..* = NO$' .config/vale.ini
39
```

== TO-BE

- 🟢 `.config/vale.ini` adds `[truth/tasks/2026-12/*.typ]` with the 32 AI-tell rules at error. The seven record exceptions stay inherited from `[truth/tasks/*/*.typ]`.
- 🟢 `2026-07` to `2026-11` keep their current exemptions, and no existing task body changes.
- 🟢 `just docs vale-tasks` stays green over the backlog, and a task filed under `2026-12` that trips one of the 32 rules fails it.
- 🟢 The Task bodies paragraph states `2026-12` as the first month under the full rule set.

```ini
[truth/tasks/2026-12/*.typ]
Prose.OpeningCliches = error
# The other 31 AI-tell rules, each set back to error.
Prose.DoubleHyphen = error
```

== Changes

- `.config/vale.ini`: add the `2026-12` section with the 32 AI-tell rules at error; keep the `2026-07` to `2026-09` `BasedOnStyles = NONE` sections, the 32 `NO` lines under `[truth/tasks/*/*.typ]`, and `swept/`.
- `docs/src/contributing/documentation.typ`: the Task bodies paragraph, which states `2026-12` as the first month under the 32 rules.

== Alternatives

- Backfill every month and drop the 32 `NO` lines. That rewrites the 872 alerts in `2026-10` and `2026-11`, and the older months need their own cleanup.
- Gate from `2026-11` after the 208 alerts in that month are fixed. It covers the rest of November at the cost of a partial backfill.
- Leave the gate as is. New tasks keep skipping the 32 rules.

== Out of scope

- Backfilling `2026-07` to `2026-11`, or editing any existing task body.
- The `2026-11` month, which stays ungated under the cutoff.
- New rules beyond the 32 AI-tell rules.

== Dogfood

Copy a task that trips `Prose.BareNames` into `truth/tasks/2026-12/`; `just docs vale-tasks` fails it. Move the same file to `truth/tasks/2026-11/` and the gate passes.
