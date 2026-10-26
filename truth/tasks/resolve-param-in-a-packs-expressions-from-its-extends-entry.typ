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
  status: done(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on(
      "pass-parameters-to-a-packs-expressions.typ",
      "merge-every-config-key-from-a-pack-below-the-layer-that-extends-it.typ",
    ),
)

== Problem

+ Write a path pack with `[params] max-files = 5` and a rule asserting `estimate.src-files <= param("max-files")`.
+ Extend it with `extends = [{ pack = "./team.toml", with = { max-files = 8 } }]`.
+ 🔴 `mt check` refuses both: it prints `extends: entries must be strings`, and `[params]` reaches the merged config as an unknown field.


== TO-BE

```toml
extends = [{ pack = "./team.toml", with = { max-files = 8 } }]
```

- 🟢 The rule caps at 8, and at 5 when the entry is the bare string.
- 🟢 At config load, each `param("name")` outside a string literal becomes the value's literal in every expression key of the pack: rule `when` and `assert`, commit rules, `on.flip` `when`, `views.*.filter`, every `required` and `values-when`, a priority framework's `score` and category `when`, `[[ls.rules]]` `condition` and `filter`, `[ls.columns]` `when`, and `[rm] default-filter`. A rule's `message` is prose and keeps its text.
- 🟢 `mt` refuses, naming the pack: a `with` key the pack doesn't declare, with the nearest name; a value of another type than the default; `param()` naming no parameter; `param()` in a layer's own expression; one pack extended twice in a layer.


== Changes

- `crates/mindtape/src/config/packs_tests_params.rs` (new, since `packs_tests.rs` nears its cap): the red tests first, one per refusal plus the default, the override and a `param(` inside a string literal left alone. `params_tests.rs` covers the call scanner.
- `crates/mindtape/src/config/packs_toml.rs`: parse an `extends` entry as a string or a `{ pack, with }` table.
- `crates/mindtape/src/config/params.rs` (new): parse `[params]`, check `with` against it, and rewrite `param()` calls with a string-literal-aware pass like `join_kebab_names` in `crates/mindtape/src/filter/expr.rs`. `EXPRESSION_KEYS` lists the expression keys as TOML paths. `PackEntry` types `Config::extends` for the schema.
- `crates/mindtape/src/config/mod.rs`: list the file in the module header, and type `extends` as `Vec<PackEntry>`.
- `crates/mindtape/src/config/packs.rs`: in `fold_layer_packs`, rewrite each pack between `resolve_pack` and `take_rule_arrays`, so its rule expressions are rewritten too, and take `[params]` out of the pack before `PackLayer::add` merges the rest. Reject `param()` in a layer's own expressions.
- `crates/mindtape/src/config/pack_fold.rs`: `PackLayer::add` refuses a pack its layer already added.
- `schema/config.schema.json`: regenerate.
- `www/content/packs.typ`, `www/generate-packs.sh`: a Parameters section, with the override and the unknown-key refusal as captures.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- A `param()` function in the filter evaluator: every call site would need the pack's values at evaluation time.
- Rewrite the parsed `evalexpr` tree: its nodes carry no spans to write the text back from.
- A plain regex: it would rewrite a `param(` inside a string literal too.


== Out of scope

- Parameters in values that aren't expressions, and set-valued parameters: `pass-parameters-to-a-packs-expressions.typ` rules both out.
- Printing parameter values: `print-each-pack-key-and-parameter-origin-in-mt-config-show.typ`.


== Dogfood

This repo extends `mindtape:budget` with its default cap of 5 once `ship-the-wip-caps-and-budget-packs.typ` lands.

