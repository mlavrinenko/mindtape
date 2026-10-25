#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Resolve param() in a pack's expressions from its extends entry",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 6,
  tags: ("mt", "gating"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a project sets a pack's cap with one `with` key and keeps the rule's later fixes],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on(
      "pass-parameters-to-a-packs-expressions.typ",
      "merge-every-config-key-from-a-pack-below-the-layer-that-extends-it.typ",
    ),
)

== Problem

+ Write a path pack with `[params] max-files = 5` and a rule asserting `estimate.src-files <= param("max-files")`.
+ Extend it with `extends = [{ pack = "./team.toml", with = { max-files = 8 } }]`.
+ 🔴 `mt check` refuses both: `extends` takes only strings, and the pack may not declare `params`.


== TO-BE

```toml
extends = [{ pack = "./team.toml", with = { max-files = 8 } }]
```

- 🟢 The rule caps at 8, and at 5 when the entry is the bare string.
- 🟢 `param()` works in every expression the pack declares and resolves at config load, so the filter evaluator never sees it.
- 🟢 `mt` refuses, naming the pack: a `with` key the pack doesn't declare, with the nearest name; a value of another type than the default; `param()` naming no parameter; `param()` in a layer's own expression; one pack extended twice in a layer.
- 🟢 `mt config show --origin` prints `param <pack>.<name> = <value>: <origin>`.


== Changes

- `crates/mindtape/src/config/packs_tests.rs`: the red tests first, one per refusal plus the default and the override.
- `crates/mindtape/src/config/packs_toml.rs`: parse an `extends` entry as a string or a `{ pack, with }` table.
- `crates/mindtape/src/config/params.rs` (new): parse `[params]`, check `with` against it, and rewrite each `param("name")` in the pack's expressions into its literal.
- `crates/mindtape/src/config/mod.rs`: list the file in the module header.
- `crates/mindtape/src/config/packs.rs`: run the rewrite on each pack before the fold, and reject `param()` in a layer's own expressions.
- `schema/config.schema.json`: regenerate.
- `www/content/packs.typ`: a Parameters section.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- A `param()` function in the filter evaluator: every call site would need the pack's values at evaluation time.
- Rewrite the expression text with a regex: a `param(` inside a string literal would change too, so the rewrite walks the parsed expression.


== Out of scope

- Parameters in values that aren't expressions, and set-valued parameters: `pass-parameters-to-a-packs-expressions.typ` rules both out.


== Dogfood

This repo extends `mindtape:budget` with its default cap of 5 once `ship-the-wip-caps-and-budget-packs.typ` lands.

