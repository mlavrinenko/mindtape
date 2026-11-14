#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Backfill the code comments and enable the remaining prose rules",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 5,
  tags: ("tooling", "gating", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a doc comment meets every prose rule the docs follow],
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
  status: proposed(2026, 11, 15),
)

== AS-IS

+ Run `sed -n '/^Prose\..* = NO$/p' .config/vale-code.ini | wc -l`.
+ 🔴 It prints `43`: most of the `Prose` style does not read a Rust doc comment.
+ A `///` line that says "The value is written by the loader" passes `just lint vale-code`, because `Prose.PassiveVoice` is `NO`.

```
$ sed -n '/^Prose\..* = NO$/p' .config/vale-code.ini | wc -l
43
```


== TO-BE

- 🟢 Every `Prose` rule is `error`, and no rule is `NO` or `warning`.
- 🟢 The doc comments pass the full set, and `just lint vale-code` passes.
- 🟢 `grep -c '^Prose\..* = NO$' .config/vale-code.ini` prints `0`.

```
[*.rs]
BasedOnStyles = Prose
View = RustDoc
Prose.History = error
Prose.Shouting = error
Prose.PassiveVoice = error
```


== Changes

- `.config/vale-code.ini`: move each `NO` group to `error` as its alerts clear.
- `crates/**/*.rs`: rewrite the doc comments the enabled rules name.
- `.config/vale/local/Prose/words.txt`: add the technical words the spelling rule needs.
- `docs/src/contributing/documentation.typ`: state the full code-comment rule set.


== Alternatives

- Splitting the rules across 43 tasks would keep a partial rule set in the config for months.
- Keeping every rule at `warning` would flood an editor without enforcing any rule.


== Out of scope

- Shell comments, which join through the `[*.sh]` section the config carries.
- Inline `//` comments, which the `RustDoc` View leaves out.
- Other languages, which need a grammar and a View of their own.


== Dogfood

This repo's `just lint vale-code` holds every Rust doc comment to the full `Prose` set.

