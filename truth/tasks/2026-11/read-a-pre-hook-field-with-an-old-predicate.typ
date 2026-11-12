#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Read a pre-hook field with an old() predicate",
  kind: "feat",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("mt", "tests", "stateless", "no-repro"),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[old("kind") reads as the value before the add hooks ran],
      ),
      ..(
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (:),
      ..("sound-codebase", "docs-match-code"),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 11),
  links: depends-on("let-rules-and-templates-choose-where-new-tasks-land.typ"),
)

== AS-IS

+ A later add hook wants to know whether `kind` was `feat` before an earlier hook changed it.
+ 🔴 `when` sees only the current draft, so the hook cannot ask for the pre-hook value; reordering may be impossible when another field must be set first.
+ There is no read-only view of the original draft.


== TO-BE

- 🟢 `old("kind")` resolves to the field's value in the draft before any `[[on.add]]` action ran.
- 🟢 It is read-only and available only in `on.add` `when`; `old("kind") == "feat"` and other comparisons work.
- 🟢 A later hook can see current `assignee` and original `kind` in one condition.

```toml
[[on.add]]
name = "regression"
when = 'old("kind") == "feat" && kind == "bug"'
actions = [{ append = { tags = ["regression"] } }]
```


== Changes

- `crates/mindtape/src/filter/mod.rs`: carry an optional pre-hook draft in the filter context.
- `crates/mindtape/src/filter/strings.rs`: resolve `old("field")` from that draft as a string value.
- `crates/mindtape/src/filter/eval.rs`: register `old` with one field-name argument.
- `crates/mindtape-cli/src/cmd/add/hooks.rs`: capture the pre-hook draft and bind it for each `when`.
- `crates/mindtape-cli/tests/it/add_e2e.rs`, `crates/mindtape/src/filter/strings_tests.rs`: an old/current pair, a later hook after `set`, and a missing field.
- `www/content/hooks.typ`, `www/content/filters.typ`: document `old()` and its add-only scope.


== Alternatives

- `was(field, value)`: equality only, while `old(field)` composes with any comparison or function.
- A per-hook `on = "original"|"current"` mode: it makes `append` and `set` bases ambiguous.
- Reordering hooks: impossible when another field must be set before the original condition can be read.


== Out of scope

- `old` outside `on.add` `when`; rules, `mt ls` and commit filters have no pre-hook draft.
- Actions against the old task; only `when` reads it.
- `old` on virtual fields (`dir`, `path`, id).


== Dogfood

This repo can tag a bug filed from a feature with `old("kind") == "feat"`.

