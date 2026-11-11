#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Update the minimal fixture and eval README for the field writer",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  links: (
    (
      task: "give-every-settable-field-a-direct-mt-add-flag.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "docs", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[the fixture and eval teach the behavior the binary now has],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the minimal example stops describing a refusal that was removed],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 11),
)

== AS-IS

+ Reviewing the field-writer commit, which updated eval 4 but left the fixture comment and the eval README.
+ 🔴 #raw("examples/minimal/.mindtape/config.toml"): The minimal fixture still says `mt add --set priority=8` is refused by name, and the eval README repeats it.

```
$ mt add --print --set priority=8 "Priority proof"
  title: "Priority proof",
  status: todo(2026, 11, 11),
  priority: 8,
```

The fixture comment says the command exits non-zero with `error: unknown slot: 'priority'`. The README line 33 says the same.


== TO-BE

- 🟢 The fixture comment describes the field writer: a non-slot writable field is inserted after fill.
- 🟢 The eval README no longer says `priority` is refused, matching eval 4's new expectation.

```
`--set priority=8` inserts `priority: 8` after the slots, and the filed task passes `mt check`.
```


== Changes

- `examples/minimal/.mindtape/config.toml`: rewrite the second absence comment around the field writer.
- `skills/mindtape/evals/README.md`: update the minimal-fixture sentence to match eval 4.


== Out of scope

- The eval JSON and its assertions, which the commit already updated.


== Dogfood

The next skill eval run reads a fixture that describes the current CLI.

