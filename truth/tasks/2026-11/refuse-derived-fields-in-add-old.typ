#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Refuse a derived field in an add hook's old()",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 1,
  links: (
    (
      task: "let-rules-and-templates-choose-where-new-tasks-land.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "tests", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
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
  status: done(2026, 11, 12),
  assignee: "agent",
)

== AS-IS

+ Reviewing the on-add branch after `just check`.
+ 🔴 #raw("crates/mindtape/src/filter/strings.rs"): `old("derived.changes")` loads and reads the empty draft record instead of refusing a field the add cannot answer.

The draft carries an empty `derived` record, so this `when` is simply false:

```toml
[[on.add]]
when = 'old("derived.changes") == "x"'
actions = [{ set = { kind = "fix" } }]
```


== TO-BE

- 🟢 `old()` refuses every field whose root the draft cannot answer, including `derived.<name>`.

```
error: on.add[0].when: old("derived.changes") reads a value the add cannot know until the task is filed (for example old("kind") == "feat")
```


== Changes

- `crates/mindtape/src/filter/strings.rs`: check the field's root against the add-unsafe list.
- `crates/mindtape/src/filter/strings_old_tests.rs`: a declared derived field fails.


== Out of scope

- `old()` on authored fields, which stays legal.


== Dogfood

A hook cannot silently miss a derived value on this repo's collectors.

