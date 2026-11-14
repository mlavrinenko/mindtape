#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Backfill the task prose to one rule set",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 3,
  ),
  difficulty: 4,
  tags: ("docs", "gating", "stateless", "tooling"),
  x-goals: (
    product: (
      readable-tasks: helps[a task bullet reads like a docs page],
      implementable-tasks: neutral,
      enforced-practice: helps[no filing month can skip the rule set],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the docs page states the one rule set the config holds],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 14),
)

== AS-IS

+ Run `grep -n '^\[truth/tasks' .config/vale.ini`. It lists the catch-all task section, the `2026-12` section, three older-month `BasedOnStyles = NONE` sections and `swept/`.
+ Run `grep -c '^Prose\..* = NO$' .config/vale.ini`. It prints 39, seven record exceptions and the 32 AI-tell rules, all under `[truth/tasks/*/*.typ]`.
+ 🔴 Lint a task filed in `2026-07` against the `[*.typ]` rules. Vale reports 1490 alerts. The month section exempts the prose instead of holding it to the rules.

```
$ grep -c '^Prose\..* = NO$' .config/vale.ini
39
$ vale --no-global --config=/tmp/vale-uniform.ini truth/tasks/2026-07
✖ 1490 errors in 113 files
```

== TO-BE

- 🟢 `.config/vale.ini` holds one `[*.typ]` section and no task, month or `swept/` override. Every task body passes the same 48 rules as a docs page.
- 🟢 `Prose.HeadingCase` lists `AS-IS` and `TO-BE` as terms, because `mt check` matches those headings literally.
- 🟢 `just docs vale-tasks` passes over all 1003 task files with no alert.
- 🟢 The Task bodies paragraph states the uniform rule set and the two heading terms.

```ini
[*.typ]
BasedOnStyles = Prose, Vale
# 48 rules, no task section below
```

== Changes

- `.config/vale.ini`: drop the `[truth/tasks/*/*.typ]`, `[truth/tasks/2026-12/*.typ]`, three older-month and `swept/` sections. Keep the `[*.typ]` rules.
- `.config/vale/styles/Prose/HeadingCase.yml`: add `AS-IS` and `TO-BE` to the terms.
- `.config/vale/styles/Prose/words.txt`: the technical words the task bodies use.
- `truth/tasks/2026-07/*.typ` through `truth/tasks/2026-11/*.typ`: rewrite the prose to clear 7334 alerts.
- `docs/src/contributing/documentation.typ`: the Task bodies paragraph describes one rule set.

== Alternatives

- Keep the seven record exceptions and clear only the 32 AI-tell rules. That leaves a path exception and a task body under different rules.
- Keep the month sections and copy the `2026-12` section each month. That gates new months and leaves the backlog unread.

== Out of scope

- The task fields, links, statuses and body structure. The rewrite changes prose only.
- `docs/src`, `www` and code comments, which pass their gates already.

== Dogfood

This repo's `truth/tasks` passes `just docs vale-tasks` with no task-specific rule override in `.config/vale.ini`.
