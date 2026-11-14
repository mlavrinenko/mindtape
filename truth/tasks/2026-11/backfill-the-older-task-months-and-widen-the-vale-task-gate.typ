#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Backfill the older task months and widen the Vale task gate",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs", "gating", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[an older task reads under the same prose rules as a new one],
      implementable-tasks: neutral,
      enforced-practice: neutral,
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
)

== AS-IS

+ Run `grep -n 'BasedOnStyles = NONE' .config/vale.ini`.
+ 🔴 It prints a section for `2026-07`, `2026-08`, `2026-09` and `swept/`, so those months never run `Prose.Shouting` or `Prose.EmDash`.
+ Run `vale --config=.config/vale.ini truth/tasks/2026-07` and it reports nothing, though 78 of the month's 115 files hold an em dash.
+ Run `vale --config=.config/vale.ini truth/tasks/2026-10`; it reports 20 em dashes in the oldest month the gate reads, so the gate is red today.

```
$ vale --config=.config/vale.ini truth/tasks/2026-07
✔ 0 errors, 0 warnings and 0 suggestions in 115 files.
$ vale --config=.config/vale.ini truth/tasks/2026-10
✖ 20 errors, 0 warnings and 0 suggestions in 502 files.
```

== TO-BE

- 🟢 Every month runs `Prose.Shouting` and `Prose.EmDash`, and the three month sections are gone from `.config/vale.ini`.
- 🟢 `swept/` stays out, because a sweep batch is a generated record rather than prose.
- 🟢 `just docs vale-tasks` passes over every task in `truth/tasks/`, `2026-10` included.

```ini
[truth/tasks/swept/*.typ]
BasedOnStyles = NONE
```

== Changes

- `.config/vale.ini`: drop the `2026-07`, `2026-08` and `2026-09` sections; keep `swept/`.
- `truth/tasks/2026-07/*.typ`: replace each em dash, and put each shouted word in a code span or its plain case.
- `truth/tasks/2026-08/*.typ`: the same for the month's 119 files with an em dash.
- `truth/tasks/2026-09/*.typ`: the same for the month's 79 files with an em dash.
- `truth/tasks/2026-10/*.typ`: the 20 em dashes the gate reports today, so the widened gate is green.
- `.config/vale/styles/Prose/Shouting.yml`: add each true acronym the older months use.

== Alternatives

- Leave the three months out: their prose stays unchecked while the rest of the backlog is gated.
- A one-off rewrite script: it cannot tell a shouted word from a name, so each file still needs a read.

== Out of scope

- New rules beyond `Prose.Shouting` and `Prose.EmDash`, which task 1 adds.
- Rewriting a closed decision: the pass changes punctuation, capitalisation and the acronym list, not what a task decided.

== Dogfood

`just docs vale-tasks` passes over every task month, not only the months filed after this gate.
