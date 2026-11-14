#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Forbid warning levels in Vale configs",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("tooling", "gating", "stateless", "no-repro"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the gate treats every Vale rule as an error or turns it off, never as a warning],
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
  status: done(2026, 11, 15),
)

== AS-IS

+ Run `grep -rn '= warning' .config/vale*.ini`.
+ 🔴 The search finds an empty list today, and every gate ignores a level line. A new `Prose.Spelling = warning` would pass every gate.
+ Run `grep -c '^Prose\..* = NO$' .config/vale-code.ini` and see the rules the code gate turns off.


== TO-BE

- 🟢 `just lint vale-levels` fails on `MinAlertLevel` other than `error`, on any `= warning`, and on a style rule with neither `error` nor `NO`.
- 🟢 The check reads the rule names from `target/vale/styles/*/*.yml`. A new rule needs an explicit level.
- 🟢 Every current config uses `error` or `NO` for every rule.

```
[*.rs]
BasedOnStyles = Prose
View = RustDoc
Prose.History = error
Prose.Shouting = error
Prose.Spelling = NO
```


== Changes

- `.just/scripts/check-vale-levels.sh`: the gate reads every `.config/vale*.ini` and the style rule names.
- `.just/lint.just`: the `vale-levels` recipe.
- `.mmz/config.yaml`: the rule, its inputs and its `_check-fast` arm.
- `Justfile`: the arm in `_check-fast`.
- `docs/src/contributing/documentation.typ`: state the rule.


== Alternatives

- A prose rule in CONTRIBUTING.md: a reviewer can forget it, and no gate checks it.
- A `vale --minAlertLevel=warning` wrapper: it shows warnings without enforcing them.


== Out of scope

- The rule levels themselves, which `backfill-the-code-comment-prose` owns.
- A level below warning, which `MinAlertLevel = error` already hides.


== Dogfood

`just check` refuses a warning level in any Vale config.

