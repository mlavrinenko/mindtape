#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Merge named on.flip entries across packs and layers",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("mt", "gating"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a pack's flip gates apply beside the project's own, and the project replaces or drops one by name],
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
      "merge-every-config-key-from-a-pack-below-the-layer-that-extends-it.typ",
    ),
)

== Problem

+ Extend a pack that declares an `[[on.flip]]` stale-WIP gate, in a project with an `[[on.flip]]` close gate of its own.
+ Run `mt config show`.
+ 🔴 Only the close gate is left: `on.flip` replaces whole per layer, and `FlipHook` in `crates/mindtape/src/config/hooks.rs` has no `name` for the project to replace or `disable` one by.


== TO-BE

```toml
[[on.flip]]
name = "stale-wip"
when = 'into("wip") && stale(30)'
actions = [{ gate = true, message = "re-read the scope first" }]
```

- 🟢 `[[on.flip]]` takes an optional `name`, and its entries append across packs and layers as `[[tasks.rules]]` entries do.
- 🟢 A later entry with the same `name` replaces the earlier one in place, and `disable = ["stale-wip"]` drops it.
- 🟢 An unnamed entry appends and can't be replaced. `mt config show --origin` prints it as `on.flip[<index>]`.


== Changes

- `crates/mindtape/src/config/hooks_tests.rs`: the red tests first: a pack's gate fires beside the project's, a same-name entry replaces it, `disable` drops it.
- `crates/mindtape/src/config/hooks.rs`: `name: Option<String>` on `FlipHook`.
- `crates/mindtape/src/config/packs.rs`: add `on.flip` to the arrays that append by name, and let `disable` reach it.
- `schema/config.schema.json`: regenerate.
- `www/content/hooks.typ`, `www/content/packs.typ`: the `name` key and the merge.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Require `name` on every entry: `[[tasks.rules]]` lets an unnamed one append, and one rule for both is simpler.
- Keep `on.flip` replacing whole: a pack's gate would vanish the moment a project adds one.


== Out of scope

- `[[tasks.waits]]` and the other arrays: none is shared across projects yet.


== Dogfood

This repo's stale-WIP gate moves into `mindtape:flow` and its close gate stays local, both firing on one flip.

